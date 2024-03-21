use crate::ip_packet::{IpPacket, MutableIpPacket};
use crate::peer::{PacketTransformClient, Peer};
use crate::peer_store::PeerStore;
use crate::{dns, dns::DnsQuery};
use bimap::BiMap;
use connlib_shared::error::{ConnlibError as Error, ConnlibError};
use connlib_shared::messages::{
    Answer, ClientPayload, DnsServer, DomainResponse, GatewayId, Interface as InterfaceConfig,
    IpDnsServer, Key, Offer, Relay, RequestConnection, ResourceDescription,
    ResourceDescriptionCidr, ResourceDescriptionDns, ResourceId, ReuseConnection,
};
use connlib_shared::{Callbacks, Dname, PublicKey, StaticSecret};
use domain::base::Rtype;
use ip_network::{IpNetwork, Ipv4Network, Ipv6Network};
use ip_network_table::IpNetworkTable;
use itertools::Itertools;

use crate::utils::{earliest, stun, turn};
use crate::ClientTunnel;
use secrecy::{ExposeSecret as _, Secret};
use snownet::ClientNode;
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet, VecDeque};
use std::iter;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::str::FromStr;
use std::time::{Duration, Instant};

// Using str here because Ipv4/6Network doesn't support `const` 🙃
const IPV4_RESOURCES: &str = "100.96.0.0/11";
const IPV6_RESOURCES: &str = "fd00:2021:1111:8000::/107";

const DNS_PORT: u16 = 53;
const DNS_SENTINELS_V4: &str = "100.100.111.0/24";
const DNS_SENTINELS_V6: &str = "fd00:2021:1111:8000:100:100:111:0/120";

// With this single timer this might mean that some DNS are refreshed too often
// however... this also mean any resource is refresh within a 5 mins interval
// therefore, only the first time it's added that happens, after that it doesn't matter.
const DNS_REFRESH_INTERVAL: Duration = Duration::from_secs(300);

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Event {
    SignalIceCandidate {
        conn_id: GatewayId,
        candidate: String,
    },
    ConnectionIntent {
        resource: ResourceId,
        connected_gateway_ids: HashSet<GatewayId>,
    },
    RefreshResources {
        connections: Vec<ReuseConnection>,
    },
    RefreshInterfance,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct DnsResource {
    pub id: ResourceId,
    pub address: Dname,
}

impl DnsResource {
    pub fn from_description(description: &ResourceDescriptionDns, address: Dname) -> DnsResource {
        DnsResource {
            id: description.id,
            address,
        }
    }
}

impl<CB> ClientTunnel<CB>
where
    CB: Callbacks + 'static,
{
    /// Adds a the given resource to the tunnel.
    ///
    /// Once added, when a packet for the resource is intercepted a new data channel will be created
    /// and packets will be wrapped with wireguard and sent through it.
    pub fn add_resources(
        &mut self,
        resources: &[ResourceDescription],
    ) -> connlib_shared::Result<()> {
        for resource_description in resources {
            if let Some(resource) = self.role_state.resource_ids.get(&resource_description.id()) {
                if resource.has_different_address(resource) {
                    self.remove_resource(resource.id());
                }
            }

            match &resource_description {
                ResourceDescription::Dns(dns) => {
                    self.role_state
                        .dns_resources
                        .insert(dns.address.clone(), dns.clone());
                }
                ResourceDescription::Cidr(cidr) => {
                    self.role_state
                        .cidr_resources
                        .insert(cidr.address, cidr.clone());
                }
            }

            self.role_state
                .resource_ids
                .insert(resource_description.id(), resource_description.clone());
        }

        self.update_resource_list();
        self.update_routes()?;

        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all, fields(%id))]
    pub fn remove_resource(&mut self, id: ResourceId) {
        self.role_state.awaiting_connection.remove(&id);
        self.role_state
            .dns_resources_internal_ips
            .retain(|r, _| r.id != id);
        self.role_state.dns_resources.retain(|_, r| r.id != id);
        self.role_state.cidr_resources.retain(|_, r| r.id != id);
        self.role_state
            .deferred_dns_queries
            .retain(|(r, _), _| r.id != id);

        self.role_state.resource_ids.remove(&id);

        if let Err(err) = self.update_routes() {
            tracing::error!(%id, "Failed to update routes: {err:?}");
        }

        self.update_resource_list();

        let Some(gateway_id) = self.role_state.resources_gateways.remove(&id) else {
            tracing::debug!("No gateway associated with resource");
            return;
        };

        let Some(peer) = self.role_state.peers.get_mut(&gateway_id) else {
            return;
        };

        // First we remove the id from all allowed ips
        for (network, resources) in peer
            .allowed_ips
            .iter_mut()
            .filter(|(_, resources)| resources.contains(&id))
        {
            resources.remove(&id);

            if !resources.is_empty() {
                continue;
            }

            // If the allowed_ips doesn't correspond to any resource anymore we
            // clean up any related translation.
            peer.transform
                .translations
                .remove_by_left(&network.network_address());
        }

        // We remove all empty allowed ips entry since there's no resource that corresponds to it
        peer.allowed_ips.retain(|_, r| !r.is_empty());

        // If there's no allowed ip left we remove the whole peer because there's no point on keeping it around
        if peer.allowed_ips.is_empty() {
            self.role_state.peers.remove(&gateway_id);
            // TODO: should we have a Node::remove_connection?
        }

        tracing::debug!("Resource removed")
    }

    fn update_resource_list(&self) {
        self.callbacks.on_update_resources(
            self.role_state
                .resource_ids
                .values()
                .sorted()
                .cloned()
                .collect_vec(),
        );
    }

    pub fn set_dns(&mut self, new_dns: Vec<IpAddr>, now: Instant) {
        self.role_state.update_system_resolvers(new_dns, now);
    }

    pub(crate) fn update_interface(&mut self) -> connlib_shared::Result<()> {
        let Some(config) = self.role_state.interface_config.as_ref().cloned() else {
            return Ok(());
        };

        let effective_dns_servers = effective_dns_servers(
            config.upstream_dns.clone(),
            self.role_state.system_resolvers.clone(),
        );

        let dns_mapping = sentinel_dns_mapping(&effective_dns_servers);
        self.role_state.set_dns_mapping(dns_mapping.clone());
        self.io.set_upstream_dns_servers(dns_mapping.clone());

        let callbacks = self.callbacks.clone();

        self.io.device_mut().initialize(
            &config,
            // We can just sort in here because sentinel ips are created in order
            dns_mapping.left_values().copied().sorted().collect(),
            &callbacks,
        )?;

        self.io
            .device_mut()
            .set_routes(self.role_state.routes().collect(), &self.callbacks)?;
        let name = self.io.device_mut().name().to_owned();

        self.callbacks.on_tunnel_ready();

        tracing::debug!(ip4 = %config.ipv4, ip6 = %config.ipv6, %name, "TUN device initialized");

        Ok(())
    }

    /// Sets the interface configuration and starts background tasks.
    #[tracing::instrument(level = "trace", skip(self))]
    pub fn set_interface(&mut self, config: InterfaceConfig) -> connlib_shared::Result<()> {
        self.role_state.interface_config = Some(config);
        self.update_interface()
    }

    /// Clean up a connection to a resource.
    // FIXME: this cleanup connection is wrong!
    pub fn cleanup_connection(&mut self, id: ResourceId) {
        self.role_state.on_connection_failed(id);
        // self.peer_connections.lock().remove(&id.into());
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub fn update_routes(&mut self) -> connlib_shared::Result<()> {
        self.io
            .device_mut()
            .set_routes(self.role_state.routes().collect(), &self.callbacks)?;

        Ok(())
    }

    pub fn add_ice_candidate(&mut self, conn_id: GatewayId, ice_candidate: String) {
        self.role_state
            .node
            .add_remote_candidate(conn_id, ice_candidate, Instant::now());
    }

    /// Initiate an ice connection request.
    ///
    /// Given a resource id and a list of relay creates a [RequestConnection]
    /// and prepares the tunnel to handle the connection once initiated.
    ///
    /// # Parameters
    /// - `resource_id`: Id of the resource we are going to request the connection to.
    /// - `relays`: The list of relays used for that connection.
    ///
    /// # Returns
    /// A [RequestConnection] that should be sent to the gateway through the control-plane.
    #[tracing::instrument(level = "trace", skip_all, fields(%resource_id, %gateway_id))]
    pub fn request_connection(
        &mut self,
        resource_id: ResourceId,
        gateway_id: GatewayId,
        relays: Vec<Relay>,
    ) -> connlib_shared::Result<Request> {
        self.role_state.create_or_reuse_connection(
            resource_id,
            gateway_id,
            stun(&relays, |addr| self.io.sockets_ref().can_handle(addr)),
            turn(&relays, |addr| self.io.sockets_ref().can_handle(addr)),
        )
    }

    /// Called when a response to [ClientTunnel::request_connection] is ready.
    ///
    /// Once this is called, if everything goes fine, a new tunnel should be started between the 2 peers.
    pub fn received_offer_response(
        &mut self,
        resource_id: ResourceId,
        answer: Answer,
        domain_response: Option<DomainResponse>,
        gateway_public_key: PublicKey,
    ) -> connlib_shared::Result<()> {
        self.role_state
            .accept_answer(answer, resource_id, gateway_public_key, domain_response)?;

        Ok(())
    }

    #[tracing::instrument(level = "trace", skip(self, resource_id))]
    pub fn received_domain_parameters(
        &mut self,
        resource_id: ResourceId,
        domain_response: DomainResponse,
    ) -> connlib_shared::Result<()> {
        self.role_state
            .received_domain_parameters(resource_id, domain_response)?;

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Request {
    NewConnection(RequestConnection),
    ReuseConnection(ReuseConnection),
}

fn send_dns_answer(
    role_state: &mut ClientState,
    qtype: Rtype,
    resource_description: &DnsResource,
    addrs: &HashSet<IpAddr>,
) {
    let packet = role_state
        .deferred_dns_queries
        .remove(&(resource_description.clone(), qtype));
    if let Some(packet) = packet {
        let Some(packet) = dns::create_local_answer(addrs, packet) else {
            return;
        };
        role_state.buffered_packets.push_back(packet);
    }
}

pub struct ClientState {
    awaiting_connection: HashMap<ResourceId, AwaitingConnectionDetails>,
    resources_gateways: HashMap<ResourceId, GatewayId>,

    pub dns_resources_internal_ips: HashMap<DnsResource, HashSet<IpAddr>>,
    dns_resources: HashMap<String, ResourceDescriptionDns>,
    cidr_resources: IpNetworkTable<ResourceDescriptionCidr>,
    pub resource_ids: HashMap<ResourceId, ResourceDescription>,
    pub deferred_dns_queries: HashMap<(DnsResource, Rtype), IpPacket<'static>>,

    pub peers: PeerStore<GatewayId, PacketTransformClient, HashSet<ResourceId>>,

    node: ClientNode<GatewayId>,

    pub ip_provider: IpProvider,

    dns_mapping: BiMap<IpAddr, DnsServer>,

    buffered_events: VecDeque<Event>,
    interface_config: Option<InterfaceConfig>,
    buffered_packets: VecDeque<IpPacket<'static>>,

    /// DNS queries that we need to forward to the system resolver.
    buffered_dns_queries: VecDeque<DnsQuery<'static>>,

    next_dns_refresh: Option<Instant>,
    next_system_resolver_refresh: Option<Instant>,

    system_resolvers: Vec<IpAddr>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AwaitingConnectionDetails {
    pub domain: Option<Dname>,
    gateways: HashSet<GatewayId>,
    pub last_intent_sent_at: Instant,
}

impl ClientState {
    pub(crate) fn new(private_key: StaticSecret) -> Self {
        Self {
            awaiting_connection: Default::default(),
            resources_gateways: Default::default(),
            ip_provider: IpProvider::for_resources(),
            dns_resources_internal_ips: Default::default(),
            dns_resources: Default::default(),
            cidr_resources: IpNetworkTable::new(),
            resource_ids: Default::default(),
            peers: Default::default(),
            deferred_dns_queries: Default::default(),
            dns_mapping: Default::default(),
            buffered_events: Default::default(),
            interface_config: Default::default(),
            buffered_packets: Default::default(),
            buffered_dns_queries: Default::default(),
            next_dns_refresh: Default::default(),
            node: ClientNode::new(private_key),
            system_resolvers: Default::default(),
            next_system_resolver_refresh: Default::default(),
        }
    }

    pub(crate) fn encapsulate<'s>(
        &'s mut self,
        packet: MutableIpPacket<'_>,
        now: Instant,
    ) -> Option<snownet::Transmit<'s>> {
        let (packet, dest) = match self.handle_dns(packet, now) {
            Ok(response) => {
                self.buffered_packets.push_back(response?.to_owned());
                return None;
            }
            Err(non_dns_packet) => non_dns_packet,
        };

        let Some(peer) = self.peers.peer_by_ip_mut(dest) else {
            self.on_connection_intent_ip(dest, now);
            return None;
        };

        let packet = peer.transform(packet)?;

        let transmit = self
            .node
            .encapsulate(peer.conn_id, packet.as_immutable().into(), Instant::now())
            .inspect_err(|e| tracing::debug!("Failed to encapsulate: {e}"))
            .ok()??;

        Some(transmit)
    }

    pub(crate) fn decapsulate<'b>(
        &mut self,
        local: SocketAddr,
        from: SocketAddr,
        packet: &[u8],
        now: Instant,
        buffer: &'b mut [u8],
    ) -> Option<IpPacket<'b>> {
        let (conn_id, packet) = self.node.decapsulate(
            local,
            from,
            packet.as_ref(),
            now,
            buffer,
        )
        .inspect_err(|e| tracing::warn!(%local, %from, num_bytes = %packet.len(), "Failed to decapsulate incoming packet: {e}"))
        .ok()??;

        let Some(peer) = self.peers.get_mut(&conn_id) else {
            tracing::error!(%conn_id, %local, %from, "Couldn't find connection");

            return None;
        };

        let packet = match peer.untransform(packet.into()) {
            Ok(packet) => packet,
            Err(e) => {
                tracing::warn!(%conn_id, %local, %from, "Failed to transform packet: {e}");

                return None;
            }
        };

        Some(packet.into_immutable())
    }

    #[tracing::instrument(level = "trace", skip_all, fields(%resource_id))]
    fn accept_answer(
        &mut self,
        answer: Answer,
        resource_id: ResourceId,
        gateway: PublicKey,
        domain_response: Option<DomainResponse>,
    ) -> connlib_shared::Result<()> {
        let gateway_id = self
            .gateway_by_resource(&resource_id)
            .ok_or(Error::UnknownResource)?;

        self.node.accept_answer(
            gateway_id,
            gateway,
            snownet::Answer {
                credentials: snownet::Credentials {
                    username: answer.username,
                    password: answer.password,
                },
            },
            Instant::now(),
        );

        let desc = self
            .resource_ids
            .get(&resource_id)
            .ok_or(Error::ControlProtocolError)?;

        let ips = self.get_resource_ip(desc, &domain_response.as_ref().map(|d| d.domain.clone()));

        // Tidy up state once everything succeeded.
        self.awaiting_connection.remove(&resource_id);

        let resource_ids = HashSet::from([resource_id]);
        let mut peer: Peer<_, PacketTransformClient, _> =
            Peer::new(gateway_id, Default::default(), &ips, resource_ids);
        peer.transform.set_dns(self.dns_mapping());
        self.peers.insert(peer, &[]);

        let peer_ips = if let Some(domain_response) = domain_response {
            self.dns_response(&resource_id, &domain_response, &gateway_id)?
        } else {
            ips
        };

        self.peers
            .add_ips_with_resource(&gateway_id, &peer_ips, &resource_id);

        Ok(())
    }

    fn create_or_reuse_connection(
        &mut self,
        resource_id: ResourceId,
        gateway_id: GatewayId,
        allowed_stun_servers: HashSet<SocketAddr>,
        allowed_turn_servers: HashSet<(SocketAddr, String, String, String)>,
    ) -> connlib_shared::Result<Request> {
        tracing::trace!("request_connection");

        let desc = self
            .resource_ids
            .get(&resource_id)
            .ok_or(Error::UnknownResource)?;

        let domain = self.get_awaiting_connection(&resource_id)?.domain.clone();

        if self.is_connected_to(resource_id, &domain) {
            return Err(Error::UnexpectedConnectionDetails);
        }

        let awaiting_connection = self
            .awaiting_connection
            .get(&resource_id)
            .ok_or(Error::UnexpectedConnectionDetails)?
            .clone();

        self.resources_gateways.insert(resource_id, gateway_id);

        if self.peers.get(&gateway_id).is_some() {
            self.peers.add_ips_with_resource(
                &gateway_id,
                &self.get_resource_ip(desc, &domain),
                &resource_id,
            );

            self.awaiting_connection.remove(&resource_id);

            return Ok(Request::ReuseConnection(ReuseConnection {
                resource_id,
                gateway_id,
                payload: domain.clone(),
            }));
        };

        if self.node.is_expecting_answer(gateway_id) {
            return Err(Error::PendingConnection);
        }

        let offer = self.node.new_connection(
            gateway_id,
            allowed_stun_servers,
            allowed_turn_servers,
            awaiting_connection.last_intent_sent_at,
            Instant::now(),
        );

        return Ok(Request::NewConnection(RequestConnection {
            resource_id,
            gateway_id,
            client_preshared_key: Secret::new(Key(*offer.session_key.expose_secret())),
            client_payload: ClientPayload {
                ice_parameters: Offer {
                    username: offer.credentials.username,
                    password: offer.credentials.password,
                },
                domain: awaiting_connection.domain,
            },
        }));
    }

    fn received_domain_parameters(
        &mut self,
        resource_id: ResourceId,
        domain_response: DomainResponse,
    ) -> connlib_shared::Result<()> {
        let gateway_id = self
            .gateway_by_resource(&resource_id)
            .ok_or(Error::UnknownResource)?;

        let peer_ips = self.dns_response(&resource_id, &domain_response, &gateway_id)?;

        self.peers
            .add_ips_with_resource(&gateway_id, &peer_ips, &resource_id);

        Ok(())
    }

    fn dns_response(
        &mut self,
        resource_id: &ResourceId,
        domain_response: &DomainResponse,
        peer_id: &GatewayId,
    ) -> connlib_shared::Result<Vec<IpNetwork>> {
        let peer = self
            .peers
            .get_mut(peer_id)
            .ok_or(Error::ControlProtocolError)?;

        let resource_description = self
            .resource_ids
            .get(resource_id)
            .ok_or(Error::UnknownResource)?
            .clone();

        let ResourceDescription::Dns(resource_description) = resource_description else {
            // We should never get a domain_response for a CIDR resource!
            return Err(Error::ControlProtocolError);
        };

        let resource_description =
            DnsResource::from_description(&resource_description, domain_response.domain.clone());

        let addrs: HashSet<_> = domain_response
            .address
            .iter()
            .filter_map(|external_ip| {
                peer.transform
                    .get_or_assign_translation(external_ip, &mut self.ip_provider)
            })
            .collect();

        self.dns_resources_internal_ips
            .insert(resource_description.clone(), addrs.clone());

        send_dns_answer(self, Rtype::Aaaa, &resource_description, &addrs);
        send_dns_answer(self, Rtype::A, &resource_description, &addrs);

        Ok(addrs.iter().copied().map(Into::into).collect())
    }

    /// Attempt to handle the given packet as a DNS packet.
    ///
    /// Returns `Ok` if the packet is in fact a DNS query with an optional response to send back.
    /// Returns `Err` if the packet is not a DNS query.
    fn handle_dns<'a>(
        &mut self,
        packet: MutableIpPacket<'a>,
        now: Instant,
    ) -> Result<Option<IpPacket<'a>>, (MutableIpPacket<'a>, IpAddr)> {
        match dns::parse(
            &self.dns_resources,
            &self.dns_resources_internal_ips,
            &self.dns_mapping,
            packet.as_immutable(),
        ) {
            Some(dns::ResolveStrategy::LocalResponse(query)) => Ok(Some(query)),
            Some(dns::ResolveStrategy::ForwardQuery(query)) => {
                // There's an edge case here, where the resolver's ip has been resolved before as
                // a dns resource... we will ignore that weird case for now.
                // Assuming a single upstream dns until #3123 lands
                if let Some(upstream_dns) = self.dns_mapping.get_by_left(&query.query.destination())
                {
                    if self
                        .cidr_resources
                        .longest_match(upstream_dns.ip())
                        .is_some()
                    {
                        return Err((packet, upstream_dns.ip()));
                    }
                }

                self.buffered_dns_queries.push_back(query.into_owned());

                Ok(None)
            }
            Some(dns::ResolveStrategy::DeferredResponse(resource)) => {
                self.on_connection_intent_dns(&resource.0, now);
                self.deferred_dns_queries
                    .insert(resource, packet.as_immutable().to_owned());

                Ok(None)
            }
            None => {
                let dest = packet.destination();
                Err((packet, dest))
            }
        }
    }

    pub(crate) fn get_awaiting_connection(
        &self,
        resource: &ResourceId,
    ) -> Result<&AwaitingConnectionDetails, ConnlibError> {
        self.awaiting_connection
            .get(resource)
            .ok_or(Error::UnexpectedConnectionDetails)
    }

    pub fn on_connection_failed(&mut self, resource: ResourceId) {
        self.awaiting_connection.remove(&resource);
        self.resources_gateways.remove(&resource);
    }

    #[tracing::instrument(level = "debug", skip_all, fields(resource_address = %resource.address, resource_id = %resource.id))]
    fn on_connection_intent_dns(&mut self, resource: &DnsResource, now: Instant) {
        self.on_connection_intent_to_resource(resource.id, Some(resource.address.clone()), now)
    }

    #[tracing::instrument(level = "debug", skip_all, fields(resource_ip = %destination, resource_id))]
    fn on_connection_intent_ip(&mut self, destination: IpAddr, now: Instant) {
        if is_definitely_not_a_resource(destination) {
            return;
        }

        let Some(resource_id) = self.get_cidr_resource_by_destination(destination) else {
            if let Some(resource) = self
                .dns_resources_internal_ips
                .iter()
                .find_map(|(r, i)| i.contains(&destination).then_some(r))
                .cloned()
            {
                self.on_connection_intent_dns(&resource, now);
            }

            tracing::trace!("Unknown resource");

            return;
        };

        tracing::Span::current().record("resource_id", tracing::field::display(&resource_id));

        self.on_connection_intent_to_resource(resource_id, None, now)
    }

    fn on_connection_intent_to_resource(
        &mut self,
        resource: ResourceId,
        domain: Option<Dname>,
        now: Instant,
    ) {
        debug_assert!(self.resource_ids.contains_key(&resource));

        let gateways = self
            .resources_gateways
            .values()
            .copied()
            .collect::<HashSet<_>>();

        match self.awaiting_connection.entry(resource) {
            Entry::Occupied(mut occupied) => {
                let time_since_last_intent = now.duration_since(occupied.get().last_intent_sent_at);

                if time_since_last_intent < Duration::from_secs(2) {
                    tracing::trace!(?time_since_last_intent, "Skipping connection intent");

                    return;
                }

                occupied.get_mut().last_intent_sent_at = now;
            }
            Entry::Vacant(vacant) => {
                vacant.insert(AwaitingConnectionDetails {
                    domain,
                    gateways: gateways.clone(),
                    last_intent_sent_at: now,
                });
            }
        }

        tracing::debug!("Sending connection intent");

        self.buffered_events.push_back(Event::ConnectionIntent {
            resource,
            connected_gateway_ids: gateways,
        });
    }

    pub fn gateway_by_resource(&self, resource: &ResourceId) -> Option<GatewayId> {
        self.resources_gateways.get(resource).copied()
    }

    fn set_dns_mapping(&mut self, new_mapping: BiMap<IpAddr, DnsServer>) {
        self.dns_mapping = new_mapping.clone();
        self.peers
            .iter_mut()
            .for_each(|p| p.transform.set_dns(new_mapping.clone()));
    }

    pub fn dns_mapping(&self) -> BiMap<IpAddr, DnsServer> {
        self.dns_mapping.clone()
    }

    fn is_connected_to(&self, resource: ResourceId, domain: &Option<Dname>) -> bool {
        let Some(resource) = self.resource_ids.get(&resource) else {
            return false;
        };

        let ips = self.get_resource_ip(resource, domain);
        ips.iter().any(|ip| self.peers.exact_match(*ip).is_some())
    }

    fn get_resource_ip(
        &self,
        resource: &ResourceDescription,
        domain: &Option<Dname>,
    ) -> Vec<IpNetwork> {
        match resource {
            ResourceDescription::Dns(dns_resource) => {
                let Some(domain) = domain else {
                    return vec![];
                };

                let description = DnsResource::from_description(dns_resource, domain.clone());
                self.dns_resources_internal_ips
                    .get(&description)
                    .cloned()
                    .unwrap_or_default()
                    .into_iter()
                    .map(Into::into)
                    .collect()
            }
            ResourceDescription::Cidr(cidr) => vec![cidr.address],
        }
    }

    pub fn cleanup_connected_gateway(&mut self, gateway_id: &GatewayId) {
        self.peers.remove(gateway_id);
        self.dns_resources_internal_ips.retain(|resource, _| {
            !self
                .resources_gateways
                .get(&resource.id)
                .is_some_and(|r_gateway_id| r_gateway_id == gateway_id)
        });
    }

    fn routes(&self) -> impl Iterator<Item = IpNetwork> + '_ {
        self.cidr_resources
            .iter()
            .map(|(ip, _)| ip)
            .chain(iter::once(IpNetwork::from_str(IPV4_RESOURCES).unwrap()))
            .chain(iter::once(IpNetwork::from_str(IPV6_RESOURCES).unwrap()))
            .chain(self.dns_mapping.left_values().copied().map(Into::into))
    }

    fn get_cidr_resource_by_destination(&self, destination: IpAddr) -> Option<ResourceId> {
        self.cidr_resources
            .longest_match(destination)
            .map(|(_, res)| res.id)
    }

    fn update_system_resolvers(&mut self, new_dns: Vec<IpAddr>, now: Instant) {
        if !dns_updated(&self.system_resolvers, &new_dns) {
            tracing::debug!("Updated dns called but no change to system's resolver");
            return;
        }

        self.next_system_resolver_refresh = Some(now + std::time::Duration::from_millis(500));
        self.system_resolvers = new_dns;
    }

    pub fn poll_packets(&mut self) -> Option<IpPacket<'static>> {
        self.buffered_packets.pop_front()
    }

    pub fn poll_dns_queries(&mut self) -> Option<DnsQuery<'static>> {
        self.buffered_dns_queries.pop_front()
    }

    pub fn poll_timeout(&mut self) -> Option<Instant> {
        let timeout = earliest(self.next_dns_refresh, self.node.poll_timeout());
        earliest(timeout, self.next_system_resolver_refresh)
    }

    pub fn handle_timeout(&mut self, now: Instant) {
        self.node.handle_timeout(now);

        match self.next_dns_refresh {
            Some(next_dns_refresh) if now >= next_dns_refresh => {
                let mut connections = Vec::new();

                self.peers
                    .iter_mut()
                    .for_each(|p| p.transform.expire_dns_track());

                for resource in self.dns_resources_internal_ips.keys() {
                    let Some(gateway_id) = self.resources_gateways.get(&resource.id) else {
                        continue;
                    };
                    // filter inactive connections
                    if self.peers.get(gateway_id).is_none() {
                        continue;
                    }

                    connections.push(ReuseConnection {
                        resource_id: resource.id,
                        gateway_id: *gateway_id,
                        payload: Some(resource.address.clone()),
                    });
                }

                self.buffered_events
                    .push_back(Event::RefreshResources { connections });

                self.next_dns_refresh = Some(now + DNS_REFRESH_INTERVAL);
            }
            None => self.next_dns_refresh = Some(now + DNS_REFRESH_INTERVAL),
            Some(_) => {}
        }

        if self.next_system_resolver_refresh.is_some_and(|e| now >= e) {
            self.buffered_events.push_back(Event::RefreshInterfance);
            self.next_system_resolver_refresh = None;
        }

        while let Some(event) = self.node.poll_event() {
            match event {
                snownet::Event::ConnectionFailed(id) => {
                    self.cleanup_connected_gateway(&id);
                }
                snownet::Event::SignalIceCandidate {
                    connection,
                    candidate,
                } => self.buffered_events.push_back(Event::SignalIceCandidate {
                    conn_id: connection,
                    candidate,
                }),
                _ => {}
            }
        }
    }

    pub(crate) fn poll_event(&mut self) -> Option<Event> {
        self.buffered_events.pop_front()
    }

    pub(crate) fn reconnect(&mut self, now: Instant) {
        self.node.reconnect(now)
    }

    pub(crate) fn poll_transmit(&mut self) -> Option<snownet::Transmit<'_>> {
        self.node.poll_transmit()
    }
}

fn dns_updated(old_dns: &[IpAddr], new_dns: &[IpAddr]) -> bool {
    HashSet::<&IpAddr>::from_iter(old_dns.iter()) != HashSet::<&IpAddr>::from_iter(new_dns.iter())
}

fn effective_dns_servers(
    upstream_dns: Vec<DnsServer>,
    default_resolvers: Vec<IpAddr>,
) -> Vec<DnsServer> {
    if !upstream_dns.is_empty() {
        return upstream_dns;
    }

    let mut dns_servers = default_resolvers
        .into_iter()
        .filter(|ip| !IpNetwork::from_str(DNS_SENTINELS_V4).unwrap().contains(*ip))
        .filter(|ip| !IpNetwork::from_str(DNS_SENTINELS_V6).unwrap().contains(*ip))
        .peekable();

    if dns_servers.peek().is_none() {
        tracing::error!("No system default DNS servers available! Can't initialize resolver. DNS interception will be disabled.");
        return Vec::new();
    }

    dns_servers
        .map(|ip| {
            DnsServer::IpPort(IpDnsServer {
                address: (ip, DNS_PORT).into(),
            })
        })
        .collect()
}

fn sentinel_dns_mapping(dns: &[DnsServer]) -> BiMap<IpAddr, DnsServer> {
    let mut ip_provider = IpProvider::for_stub_dns_servers();

    dns.iter()
        .cloned()
        .map(|i| {
            (
                ip_provider
                    .get_proxy_ip_for(&i.ip())
                    .expect("We only support up to 256 IPv4 DNS servers and 256 IPv6 DNS servers"),
                i,
            )
        })
        .collect()
}
/// Compares the given [`IpAddr`] against a static set of ignored IPs that are definitely not resources.
fn is_definitely_not_a_resource(ip: IpAddr) -> bool {
    /// Source: https://en.wikipedia.org/wiki/Multicast_address#Notable_IPv4_multicast_addresses
    const IPV4_IGMP_MULTICAST: Ipv4Addr = Ipv4Addr::new(224, 0, 0, 22);

    /// Source: <https://en.wikipedia.org/wiki/Multicast_address#Notable_IPv6_multicast_addresses>
    const IPV6_MULTICAST_ALL_ROUTERS: Ipv6Addr = Ipv6Addr::new(0xFF02, 0, 0, 0, 0, 0, 0, 0x0002);

    match ip {
        IpAddr::V4(ip4) => {
            if ip4 == IPV4_IGMP_MULTICAST {
                return true;
            }
        }
        IpAddr::V6(ip6) => {
            if ip6 == IPV6_MULTICAST_ALL_ROUTERS {
                return true;
            }
        }
    }

    false
}

pub struct IpProvider {
    ipv4: Box<dyn Iterator<Item = Ipv4Addr> + Send + Sync>,
    ipv6: Box<dyn Iterator<Item = Ipv6Addr> + Send + Sync>,
}

impl IpProvider {
    pub fn for_resources() -> Self {
        IpProvider::new(
            IPV4_RESOURCES.parse().unwrap(),
            IPV6_RESOURCES.parse().unwrap(),
            Some(DNS_SENTINELS_V4.parse().unwrap()),
            Some(DNS_SENTINELS_V6.parse().unwrap()),
        )
    }

    pub fn for_stub_dns_servers() -> Self {
        IpProvider::new(
            DNS_SENTINELS_V4.parse().unwrap(),
            DNS_SENTINELS_V6.parse().unwrap(),
            None,
            None,
        )
    }

    fn new(
        ipv4: Ipv4Network,
        ipv6: Ipv6Network,
        exclusion_v4: Option<Ipv4Network>,
        exclusion_v6: Option<Ipv6Network>,
    ) -> Self {
        Self {
            ipv4: Box::new(
                ipv4.hosts()
                    .filter(move |ip| !exclusion_v4.is_some_and(|e| e.contains(*ip))),
            ),
            ipv6: Box::new(
                ipv6.subnets_with_prefix(128)
                    .map(|ip| ip.network_address())
                    .filter(move |ip| !exclusion_v6.is_some_and(|e| e.contains(*ip))),
            ),
        }
    }

    pub fn get_proxy_ip_for(&mut self, ip: &IpAddr) -> Option<IpAddr> {
        let proxy_ip = match ip {
            IpAddr::V4(_) => self.ipv4.next().map(Into::into),
            IpAddr::V6(_) => self.ipv6.next().map(Into::into),
        };

        if proxy_ip.is_none() {
            // TODO: we might want to make the iterator cyclic or another strategy to prevent ip exhaustion
            // this might happen in ipv4 if tokens are too long lived.
            tracing::error!("IP exhaustion: Please reset your client");
        }

        proxy_ip
    }
}

#[cfg(test)]
mod tests {
    use rand_core::OsRng;

    use super::*;

    fn client_state_fixture() -> ClientState {
        ClientState::new(StaticSecret::random_from_rng(OsRng))
    }

    #[test]
    fn ignores_ip4_igmp_multicast() {
        assert!(is_definitely_not_a_resource("224.0.0.22".parse().unwrap()))
    }

    #[test]
    fn ignores_ip6_multicast_all_routers() {
        assert!(is_definitely_not_a_resource("ff02::2".parse().unwrap()))
    }

    #[test]
    fn dns_updated_when_dns_changes() {
        assert!(dns_updated(
            &["1.0.0.1".parse().unwrap()],
            &["1.1.1.1".parse().unwrap()]
        ))
    }

    #[test]
    fn dns_not_updated_when_dns_remains_the_same() {
        assert!(!dns_updated(
            &["1.1.1.1".parse().unwrap()],
            &["1.1.1.1".parse().unwrap()]
        ))
    }

    #[test]
    fn dns_updated_ignores_order() {
        assert!(!dns_updated(
            &["1.0.0.1".parse().unwrap(), "1.1.1.1".parse().unwrap()],
            &["1.1.1.1".parse().unwrap(), "1.0.0.1".parse().unwrap()]
        ))
    }

    #[test]
    fn update_system_dns_works() {
        let mut mock_state = client_state_fixture();

        let now = Instant::now();
        mock_state.update_system_resolvers(vec!["1.1.1.1".parse().unwrap()], now);
        let now = now + Duration::from_millis(500);
        mock_state.handle_timeout(now);

        assert_eq!(mock_state.poll_event(), Some(Event::RefreshInterfance));
    }

    #[test]
    fn update_system_dns_without_change_is_a_no_op() {
        let mut mock_state = client_state_fixture();

        let now = Instant::now();
        mock_state.update_system_resolvers(vec!["1.1.1.1".parse().unwrap()], now);
        let now = now + Duration::from_millis(500);
        mock_state.handle_timeout(now);
        mock_state.poll_event();

        mock_state.update_system_resolvers(vec!["1.1.1.1".parse().unwrap()], now);
        let now = now + Duration::from_millis(500);
        mock_state.handle_timeout(now);
        assert!(mock_state.poll_event().is_none());
    }

    #[test]
    fn update_system_dns_with_change_works() {
        let mut mock_state = client_state_fixture();

        let now = Instant::now();
        mock_state.update_system_resolvers(vec!["1.1.1.1".parse().unwrap()], now);
        let now = now + Duration::from_millis(500);
        mock_state.handle_timeout(now);
        mock_state.poll_event();

        mock_state.update_system_resolvers(vec!["1.0.0.1".parse().unwrap()], now);
        let now = now + Duration::from_millis(500);
        mock_state.handle_timeout(now);
        assert_eq!(mock_state.poll_event(), Some(Event::RefreshInterfance));
    }
}
