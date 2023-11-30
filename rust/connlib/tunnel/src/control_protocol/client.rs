use std::sync::Arc;

use boringtun::x25519::PublicKey;
use connlib_shared::{
    control::Reference,
    messages::{
        ClientPayload, DomainResponse, GatewayId, Key, Relay, RequestConnection,
        ResourceDescription, ResourceId,
    },
    Callbacks,
};
use secrecy::Secret;
use webrtc::ice_transport::{
    ice_parameters::RTCIceParameters, ice_role::RTCIceRole,
    ice_transport_state::RTCIceTransportState, RTCIceTransport,
};

use crate::{
    control_protocol::{new_ice_connection, IceConnection},
    peer::PacketTransformClient,
    PEER_QUEUE_SIZE,
};
use crate::{peer::Peer, ClientState, ConnectedPeer, Error, Request, Result, Tunnel};

use super::{insert_peers, start_handlers};

#[tracing::instrument(level = "trace", skip(tunnel, ice))]
fn set_connection_state_update<CB>(
    tunnel: &Arc<Tunnel<CB, ClientState>>,
    ice: &Arc<RTCIceTransport>,
    gateway_id: GatewayId,
    resource_id: ResourceId,
) where
    CB: Callbacks + 'static,
{
    let tunnel = Arc::clone(tunnel);
    ice.on_connection_state_change(Box::new(move |state| {
        let tunnel = Arc::clone(&tunnel);
        tracing::trace!(%state, "peer_state");
        Box::pin(async move {
            if state == RTCIceTransportState::Failed {
                // There's a really unlikely race condition but this line needs to be before on_connection_failed.
                // if we clear up the gateway awaiting flag before removing the connection a new connection could be
                // established that replaces this one and this line removes it.
                let ice = tunnel.peer_connections.lock().remove(&gateway_id);

                if let Some(ice) = ice {
                    if let Err(err) = ice.stop().await {
                        tracing::warn!(%err, "couldn't stop ice transport: {err:#}");
                    }
                }

                tunnel.role_state.lock().on_connection_failed(resource_id);
            }
        })
    }));
}

impl<CB> Tunnel<CB, ClientState>
where
    CB: Callbacks + 'static,
{
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
    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn request_connection(
        self: &Arc<Self>,
        resource_id: ResourceId,
        gateway_id: GatewayId,
        relays: Vec<Relay>,
        reference: Option<Reference>,
    ) -> Result<Request> {
        tracing::trace!("request_connection");

        let reference: usize = reference
            .ok_or(Error::InvalidReference)?
            .parse()
            .map_err(|_| Error::InvalidReference)?;

        if let Some(connection) = self.role_state.lock().attempt_to_reuse_connection(
            resource_id,
            gateway_id,
            reference,
        )? {
            tracing::trace!("reusing_connection");
            return Ok(Request::ReuseConnection(connection));
        }

        let domain = self
            .role_state
            .lock()
            .get_awaiting_connection_domain(&resource_id)?
            .clone();

        let IceConnection {
            ice_parameters,
            ice_transport,
            ice_candidate_rx,
        } = new_ice_connection(&self.webrtc_api, relays).await?;
        let preshared_key = self
            .role_state
            .lock()
            .add_waiting_gateway(gateway_id, ice_candidate_rx);
        self.peer_connections
            .lock()
            .insert(gateway_id, Arc::clone(&ice_transport));

        set_connection_state_update(self, &ice_transport, gateway_id, resource_id);

        Ok(Request::NewConnection(RequestConnection {
            resource_id,
            gateway_id,
            client_preshared_key: Secret::new(Key(preshared_key.to_bytes())),
            client_payload: ClientPayload {
                ice_parameters,
                domain,
            },
        }))
    }

    fn new_tunnel(
        &self,
        resource_id: ResourceId,
        gateway_id: GatewayId,
        ice: Arc<RTCIceTransport>,
        domain_response: Option<DomainResponse>,
    ) -> Result<()> {
        let peer_config = self
            .role_state
            .lock()
            .create_peer_config_for_new_connection(
                resource_id,
                gateway_id,
                &domain_response.as_ref().map(|d| d.domain.clone()),
            )?;

        let peer = Arc::new(Peer::new(
            self.private_key.clone(),
            self.next_index(),
            peer_config.clone(),
            gateway_id,
            self.rate_limiter.clone(),
            PacketTransformClient::new(),
        ));

        if let Some(domain_response) = domain_response {
            let resource_description = self
                .role_state
                .lock()
                .resources_id
                .get(&resource_id)
                .ok_or(Error::UnknownResource)?
                .clone();

            let ResourceDescription::Dns(resource_description) = resource_description else {
                // We should never get a domain_response for a CIDR resource!
                return Err(Error::ControlProtocolError);
            };
            let resource_description = resource_description.subdomain(domain_response.domain);
            for ip in domain_response.address {
                let internal_ip = match ip {
                    std::net::IpAddr::V4(_) => self
                        .role_state
                        .lock()
                        .dns_resources_internal_ips
                        .get_v4_resoruce_description(&resource_description)
                        .ok_or(Error::ControlProtocolError)?
                        .to_owned()
                        .into(),
                    std::net::IpAddr::V6(_) => self
                        .role_state
                        .lock()
                        .dns_resources_internal_ips
                        .get_v6_resoruce_description(&resource_description)
                        .ok_or(Error::ControlProtocolError)?
                        .to_owned()
                        .into(),
                };
                peer.transform.insert_translation(internal_ip, ip);
            }
        }

        let (peer_sender, peer_receiver) = tokio::sync::mpsc::channel(PEER_QUEUE_SIZE);

        start_handlers(
            Arc::clone(&self.device),
            self.callbacks.clone(),
            peer.clone(),
            ice,
            peer_receiver,
        );

        // Partial reads of peers_by_ip can be problematic in the very unlikely case of an expiration
        // before inserting finishes.
        insert_peers(
            &mut self.role_state.lock().peers_by_ip,
            &peer_config.ips,
            ConnectedPeer {
                inner: peer,
                channel: peer_sender,
            },
        );

        Ok(())
    }

    /// Called when a response to [Tunnel::request_connection] is ready.
    ///
    /// Once this is called, if everything goes fine, a new tunnel should be started between the 2 peers.
    ///
    /// # Parameters
    /// - `resource_id`: Id of the resource that responded.
    /// - `rtc_sdp`: Remote SDP.
    /// - `gateway_public_key`: Public key of the gateway that is handling that resource for this connection.
    #[tracing::instrument(level = "trace", skip(self))]
    pub fn received_offer_response(
        self: &Arc<Self>,
        resource_id: ResourceId,
        rtc_ice_params: RTCIceParameters,
        domain_response: Option<DomainResponse>,
        gateway_public_key: PublicKey,
    ) -> Result<()> {
        let gateway_id = self
            .role_state
            .lock()
            .gateway_by_resource(&resource_id)
            .ok_or(Error::UnknownResource)?;
        let peer_connection = self
            .peer_connections
            .lock()
            .get(&gateway_id)
            .ok_or(Error::UnknownResource)?
            .clone();
        let resource_description = self
            .role_state
            .lock()
            .resources_id
            .get(&resource_id)
            .ok_or(Error::UnknownResource)?
            .clone();

        self.role_state
            .lock()
            .activate_ice_candidate_receiver(gateway_id, gateway_public_key);
        let tunnel = self.clone();
        // RTCIceTransport::start blocks until there's an ice connection.
        tokio::spawn(async move {
            if let Err(e) = peer_connection
                .start(&rtc_ice_params, Some(RTCIceRole::Controlling))
                .await
                .map_err(Into::into)
                .and_then(|_| {
                    tunnel.new_tunnel(resource_id, gateway_id, peer_connection, domain_response)
                })
            {
                tracing::warn!(%gateway_id, err = ?e, "Can't start tunnel: {e:#}");
                tunnel.role_state.lock().on_connection_failed(resource_id);
                let peer_connection = tunnel.peer_connections.lock().remove(&gateway_id);
                if let Some(peer_connection) = peer_connection {
                    let _ = peer_connection.stop().await;
                }
            }
        });

        Ok(())
    }
}
