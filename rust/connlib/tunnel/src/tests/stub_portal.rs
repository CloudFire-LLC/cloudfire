use super::{
    sim_client::{ref_client_host, RefClient},
    sim_gateway::{ref_gateway_host, RefGateway},
    sim_net::Host,
    strategies::{resolved_ips, subdomain_records},
};
use crate::proptest::*;
use connlib_shared::{
    messages::{client, gateway, GatewayId, ResourceId},
    DomainName,
};
use ip_network::{Ipv4Network, Ipv6Network};
use itertools::Itertools;
use proptest::{
    sample::Selector,
    strategy::{Just, Strategy},
};
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    iter,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
};

/// Stub implementation of the portal.
#[derive(Clone, derivative::Derivative)]
#[derivative(Debug)]
pub(crate) struct StubPortal {
    gateways_by_site: HashMap<client::SiteId, HashSet<GatewayId>>,

    #[derivative(Debug = "ignore")]
    sites_by_resource: HashMap<ResourceId, client::SiteId>,
    cidr_resources: HashMap<ResourceId, client::ResourceDescriptionCidr>,
    dns_resources: HashMap<ResourceId, client::ResourceDescriptionDns>,
    internet_resource: client::ResourceDescriptionInternet,

    #[derivative(Debug = "ignore")]
    gateway_selector: Selector,
}

impl StubPortal {
    pub(crate) fn new(
        gateways_by_site: HashMap<client::SiteId, HashSet<GatewayId>>,
        gateway_selector: Selector,
        cidr_resources: HashSet<client::ResourceDescriptionCidr>,
        dns_resources: HashSet<client::ResourceDescriptionDns>,
        internet_resource: client::ResourceDescriptionInternet,
    ) -> Self {
        let cidr_resources = cidr_resources
            .into_iter()
            .map(|r| (r.id, r))
            .collect::<HashMap<_, _>>();
        let dns_resources = dns_resources
            .into_iter()
            .map(|r| (r.id, r))
            .collect::<HashMap<_, _>>();

        let cidr_sites = cidr_resources.iter().map(|(id, r)| {
            (
                *id,
                r.sites
                    .iter()
                    .exactly_one()
                    .expect("only single-site resources")
                    .id,
            )
        });
        let dns_sites = dns_resources.iter().map(|(id, r)| {
            (
                *id,
                r.sites
                    .iter()
                    .exactly_one()
                    .expect("only single-site resources")
                    .id,
            )
        });
        let internet_site = iter::once((
            internet_resource.id,
            internet_resource
                .sites
                .iter()
                .exactly_one()
                .expect("only single-site resources")
                .id,
        ));

        Self {
            gateways_by_site,
            gateway_selector,
            sites_by_resource: HashMap::from_iter(cidr_sites.chain(dns_sites).chain(internet_site)),
            cidr_resources,
            dns_resources,
            internet_resource,
        }
    }

    pub(crate) fn all_resources(&self) -> Vec<client::ResourceDescription> {
        self.cidr_resources
            .values()
            .cloned()
            .map(client::ResourceDescription::Cidr)
            .chain(
                self.dns_resources
                    .values()
                    .cloned()
                    .map(client::ResourceDescription::Dns),
            )
            // TODO: Enable once we actually implement the Internet resource
            // .chain(iter::once(client::ResourceDescription::Internet(
            //     self.internet_resource.clone(),
            // )))
            .collect()
    }

    /// Picks, which gateway and site we should connect to for the given resource.
    pub(crate) fn handle_connection_intent(
        &self,
        resource: ResourceId,
        _connected_gateway_ids: HashSet<GatewayId>,
    ) -> (GatewayId, client::SiteId) {
        let site_id = self
            .sites_by_resource
            .get(&resource)
            .expect("resource to be known");

        let gateways = self.gateways_by_site.get(site_id).unwrap();
        let gateway = self.gateway_selector.select(gateways);

        (*gateway, *site_id)
    }

    pub(crate) fn map_client_resource_to_gateway_resource(
        &self,
        resolved_ips: Vec<IpAddr>,
        resource_id: ResourceId,
    ) -> gateway::ResourceDescription<gateway::ResolvedResourceDescriptionDns> {
        let cidr_resource = self.cidr_resources.iter().find_map(|(_, r)| {
            (r.id == resource_id).then_some(gateway::ResourceDescription::Cidr(
                gateway::ResourceDescriptionCidr {
                    id: r.id,
                    address: r.address,
                    name: r.name.clone(),
                    filters: Vec::new(),
                },
            ))
        });
        let dns_resource = self.dns_resources.get(&resource_id).map(|r| {
            gateway::ResourceDescription::Dns(gateway::ResolvedResourceDescriptionDns {
                id: r.id,
                name: r.name.clone(),
                filters: Vec::new(),
                domain: r.address.clone(),
                addresses: resolved_ips.clone(),
            })
        });
        let internet_resource = Some(gateway::ResourceDescription::Internet(
            gateway::ResourceDescriptionInternet {
                id: self.internet_resource.id,
            },
        ));

        cidr_resource
            .or(dns_resource)
            .or(internet_resource)
            .expect("resource to be a known CIDR, DNS or Internet resource")
    }

    pub(crate) fn gateway_for_resource(&self, rid: ResourceId) -> Option<&GatewayId> {
        let cidr_site = self
            .cidr_resources
            .iter()
            .find_map(|(_, r)| (r.id == rid).then_some(r.sites.first()?.id));

        let dns_site = self
            .dns_resources
            .get(&rid)
            .and_then(|r| Some(r.sites.first()?.id));

        let internet_site = (self.internet_resource.id == rid)
            .then(|| Some(self.internet_resource.sites.first()?.id))
            .flatten();

        let sid = cidr_site.or(dns_site).or(internet_site)?;
        let gateways = self.gateways_by_site.get(&sid)?;
        let gid = self.gateway_selector.try_select(gateways)?;

        Some(gid)
    }

    pub(crate) fn gateways(&self) -> impl Strategy<Value = BTreeMap<GatewayId, Host<RefGateway>>> {
        self.gateways_by_site
            .values()
            .flatten()
            .map(|gid| (Just(*gid), ref_gateway_host())) // Map each ID to a strategy that samples a gateway.
            .collect::<Vec<_>>() // A `Vec<Strategy>` implements `Strategy<Value = Vec<_>>`
            .prop_map(BTreeMap::from_iter)
    }

    pub(crate) fn client(&self) -> impl Strategy<Value = Host<RefClient>> {
        let client_tunnel_ip4 = tunnel_ip4s().next().unwrap();
        let client_tunnel_ip6 = tunnel_ip6s().next().unwrap();

        ref_client_host(Just(client_tunnel_ip4), Just(client_tunnel_ip6))
    }

    pub(crate) fn dns_resource_records(
        &self,
    ) -> impl Strategy<Value = HashMap<DomainName, HashSet<IpAddr>>> {
        self.dns_resources
            .values()
            .map(|resource| {
                let address = resource.address.clone();

                // Only generate simple wildcard domains for these tests.
                // The matching logic is extensively unit-tested so we don't need to cover all cases here.
                // What we do want to cover is multiple domains pointing to the same resource.
                // For example, `*.example.com` and `app.example.com`.
                match address.split_once('.') {
                    Some(("*" | "**", base)) => {
                        subdomain_records(base.to_owned(), domain_label()).boxed()
                    }
                    _ => resolved_ips()
                        .prop_map(move |resolved_ips| {
                            HashMap::from([(address.parse().unwrap(), resolved_ips)])
                        })
                        .boxed(),
                }
            })
            .collect::<Vec<_>>()
            .prop_map(|records| {
                let mut map = HashMap::default();

                for record in records {
                    map.extend(record)
                }

                map
            })
    }
}

const IPV4_TUNNEL: Ipv4Network = match Ipv4Network::new(Ipv4Addr::new(100, 64, 0, 0), 11) {
    Ok(n) => n,
    Err(_) => unreachable!(),
};
const IPV6_TUNNEL: Ipv6Network =
    match Ipv6Network::new(Ipv6Addr::new(0xfd00, 0x2021, 0x1111, 0, 0, 0, 0, 0), 107) {
        Ok(n) => n,
        Err(_) => unreachable!(),
    };

/// An [`Iterator`] over the possible IPv4 addresses of a tunnel interface.
///
/// We use the CG-NAT range for IPv4.
/// See <https://github.com/firezone/firezone/blob/81dfa90f38299595e14ce9e022d1ee919909f124/elixir/apps/domain/lib/domain/network.ex#L7>.
fn tunnel_ip4s() -> impl Iterator<Item = Ipv4Addr> {
    IPV4_TUNNEL.hosts()
}

/// An [`Iterator`] over the possible IPv6 addresses of a tunnel interface.
///
/// See <https://github.com/firezone/firezone/blob/81dfa90f38299595e14ce9e022d1ee919909f124/elixir/apps/domain/lib/domain/network.ex#L8>.
fn tunnel_ip6s() -> impl Iterator<Item = Ipv6Addr> {
    IPV6_TUNNEL
        .subnets_with_prefix(128)
        .map(|n| n.network_address())
}
