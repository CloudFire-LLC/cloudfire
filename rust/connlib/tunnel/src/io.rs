use crate::{device_channel::Device, dns, sockets::Sockets, TunConfig};
use domain::base::Message;
use futures::{
    future::{self, Either},
    stream, Stream, StreamExt,
};
use futures_bounded::FuturesTupleSet;
use futures_util::FutureExt as _;
use ip_packet::{IpPacket, IpPacketBuf, MAX_DATAGRAM_PAYLOAD};
use smoltcp::{iface::SocketSet, wire::HardwareAddress};
use snownet::{EncryptBuffer, EncryptedPacket};
use socket_factory::{DatagramIn, DatagramOut, SocketFactory, TcpSocket, UdpSocket};
use std::{
    collections::VecDeque,
    io,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    pin::Pin,
    sync::Arc,
    task::{ready, Context, Poll},
    time::{Duration, Instant},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    sync::mpsc,
};
use tun::Tun;

/// Bundles together all side-effects that connlib needs to have access to.
pub struct Io {
    /// The UDP sockets used to send & receive packets from the network.
    sockets: Sockets,
    unwritten_packet: Option<EncryptedPacket>,

    tcp_socket_factory: Arc<dyn SocketFactory<TcpSocket>>,
    udp_socket_factory: Arc<dyn SocketFactory<UdpSocket>>,

    dns_queries: FuturesTupleSet<io::Result<Message<Vec<u8>>>, DnsQueryMetaData>,

    timeout: Option<Pin<Box<tokio::time::Sleep>>>,
    tun_tx: mpsc::Sender<Box<dyn Tun>>,
    outbound_packet_tx: mpsc::Sender<IpPacket>,
    inbound_packet_rx: mpsc::Receiver<IpPacket>,

    device: SmolDeviceAdapter,
    interface: smoltcp::iface::Interface,
}

#[derive(Debug)]
struct DnsQueryMetaData {
    query: Message<Vec<u8>>,
    server: SocketAddr,
    transport: dns::Transport,
}

#[expect(
    clippy::large_enum_variant,
    reason = "We purposely don't want to allocate each IP packet."
)]
pub enum Input<I> {
    Timeout(Instant),
    Device(IpPacket),
    Network(I),
    DnsResponse(dns::RecursiveResponse),
    TcpSocketsChanged,
}

const DNS_QUERY_TIMEOUT: Duration = Duration::from_secs(5);
const IP_CHANNEL_SIZE: usize = 1000;

impl Io {
    /// Creates a new I/O abstraction
    ///
    /// Must be called within a Tokio runtime context so we can bind the sockets.
    pub fn new(
        tcp_socket_factory: Arc<dyn SocketFactory<TcpSocket>>,
        udp_socket_factory: Arc<dyn SocketFactory<UdpSocket>>,
    ) -> Self {
        let mut sockets = Sockets::default();
        sockets.rebind(udp_socket_factory.as_ref()); // Bind sockets on startup. Must happen within a tokio runtime context.

        let (inbound_packet_tx, inbound_packet_rx) = mpsc::channel(IP_CHANNEL_SIZE);
        let (outbound_packet_tx, outbound_packet_rx) = mpsc::channel(IP_CHANNEL_SIZE);
        let (tun_tx, tun_rx) = mpsc::channel(10);

        let mut device = SmolDeviceAdapter {
            outbound_packet_tx: outbound_packet_tx.clone(),
            received_packets: VecDeque::default(),
        };

        let interface = smoltcp::iface::Interface::new(
            smoltcp::iface::Config::new(HardwareAddress::Ip),
            &mut device,
            smoltcp::time::Instant::now(),
        );

        std::thread::spawn(|| {
            futures::executor::block_on(tun_send_recv(
                tun_rx,
                outbound_packet_rx,
                inbound_packet_tx,
            ))
        });

        Self {
            tun_tx,
            outbound_packet_tx,
            inbound_packet_rx,
            timeout: None,
            sockets,
            tcp_socket_factory,
            udp_socket_factory,
            unwritten_packet: None,
            device,
            interface,
            dns_queries: FuturesTupleSet::new(DNS_QUERY_TIMEOUT, 1000),
        }
    }

    pub fn poll_has_sockets(&mut self, cx: &mut Context<'_>) -> Poll<()> {
        self.sockets.poll_has_sockets(cx)
    }

    pub fn on_tun_device_changed(&mut self, config: &TunConfig) {
        if config.dns_by_sentinel.len() > 8 {
            tracing::warn!("TCP DNS only works for up to 8 DNS servers")
        }

        self.interface.update_ip_addrs(|ips| {
            ips.clear();

            let sentinel_ips = config
                .dns_by_sentinel
                .left_values()
                .copied()
                .map(|ip| match ip {
                    IpAddr::V4(_) => smoltcp::wire::IpCidr::new(ip.into(), 32),
                    IpAddr::V6(_) => smoltcp::wire::IpCidr::new(ip.into(), 128),
                });

            ips.extend(sentinel_ips.take(smoltcp::config::IFACE_MAX_ADDR_COUNT));
        });
    }

    pub fn poll<'b>(
        &mut self,
        cx: &mut Context<'_>,
        ip4_buffer: &'b mut [u8],
        ip6_bffer: &'b mut [u8],
        encrypt_buffer: &EncryptBuffer,
        tcp_sockets: &mut SocketSet,
    ) -> Poll<io::Result<Input<impl Iterator<Item = DatagramIn<'b>>>>> {
        ready!(self.poll_send_unwritten(cx, encrypt_buffer)?);

        if let Poll::Ready(network) = self.sockets.poll_recv_from(ip4_buffer, ip6_bffer, cx)? {
            return Poll::Ready(Ok(Input::Network(network.filter(is_max_wg_packet_size))));
        }

        while let Poll::Ready(Some(packet)) = self.inbound_packet_rx.poll_recv(cx) {
            // TCP traffic for the smoltcp interface needs to be redirected.
            if packet.is_tcp()
                && self
                    .interface
                    .ip_addrs()
                    .iter()
                    .any(|ip| ip.address() == packet.destination().into())
            {
                self.device.received_packets.push_back(packet);
                continue;
            }

            return Poll::Ready(Ok(Input::Device(packet)));
        }

        match self.dns_queries.poll_unpin(cx) {
            Poll::Ready((result, meta)) => {
                let response = result
                    .map(|result| dns::RecursiveResponse {
                        server: meta.server,
                        query: meta.query.clone(),
                        message: result,
                        transport: meta.transport,
                    })
                    .unwrap_or_else(|_| dns::RecursiveResponse {
                        server: meta.server,
                        query: meta.query,
                        message: Err(io::Error::from(io::ErrorKind::TimedOut)),
                        transport: meta.transport,
                    });

                return Poll::Ready(Ok(Input::DnsResponse(response)));
            }
            Poll::Pending => {}
        }

        if self
            .interface
            .poll(smoltcp::time::Instant::now(), &mut self.device, tcp_sockets)
        {
            return Poll::Ready(Ok(Input::TcpSocketsChanged));
        };

        if let Some(timeout) = self.timeout.as_mut() {
            if timeout.poll_unpin(cx).is_ready() {
                let deadline = timeout.deadline().into();
                self.timeout.as_mut().take(); // Clear the timeout.

                return Poll::Ready(Ok(Input::Timeout(deadline)));
            }
        }

        Poll::Pending
    }

    fn poll_send_unwritten(
        &mut self,
        cx: &mut Context<'_>,
        buf: &EncryptBuffer,
    ) -> Poll<io::Result<()>> {
        ready!(self.sockets.poll_send_ready(cx))?;

        // If the `unwritten_packet` is set, `EncryptBuffer` is still holding a packet that we need so send.
        let Some(unwritten_packet) = self.unwritten_packet.take() else {
            return Poll::Ready(Ok(()));
        };

        self.send_encrypted_packet(unwritten_packet, buf)?;

        Poll::Ready(Ok(()))
    }

    pub fn set_tun(&mut self, tun: Box<dyn Tun>) {
        // If we can't set a new TUN device, shut down connlib.

        self.tun_tx
            .try_send(tun)
            .expect("Channel to set new TUN device should always have capacity");
    }

    pub fn send_tun(&mut self, packet: IpPacket) -> io::Result<()> {
        let Err(e) = self.outbound_packet_tx.try_send(packet) else {
            return Ok(());
        };

        match e {
            mpsc::error::TrySendError::Full(_) => {
                Err(io::Error::other("Outbound packet channel is at capacity"))
            }
            mpsc::error::TrySendError::Closed(_) => Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "Outbound packet channel is disconnected",
            )),
        }
    }

    pub fn rebind_sockets(&mut self) {
        self.sockets.rebind(self.udp_socket_factory.as_ref());
    }

    pub fn reset_timeout(&mut self, timeout: Instant) {
        let timeout = tokio::time::Instant::from_std(timeout);

        match self.timeout.as_mut() {
            Some(existing_timeout) if existing_timeout.deadline() != timeout => {
                existing_timeout.as_mut().reset(timeout)
            }
            Some(_) => {}
            None => self.timeout = Some(Box::pin(tokio::time::sleep_until(timeout))),
        }
    }

    pub fn send_network(&mut self, transmit: snownet::Transmit) -> io::Result<()> {
        self.sockets.send(DatagramOut {
            src: transmit.src,
            dst: transmit.dst,
            packet: transmit.payload,
        })?;

        Ok(())
    }

    pub fn send_dns_query(&mut self, query: dns::RecursiveQuery) {
        match query.transport {
            dns::Transport::Udp => {
                let factory = self.udp_socket_factory.clone();
                let server = query.server;
                let bind_addr = match query.server {
                    SocketAddr::V4(_) => SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 0),
                    SocketAddr::V6(_) => SocketAddr::new(Ipv6Addr::UNSPECIFIED.into(), 0),
                };
                let meta = DnsQueryMetaData {
                    query: query.message.clone(),
                    server,
                    transport: dns::Transport::Udp,
                };

                if self
                    .dns_queries
                    .try_push(
                        async move {
                            // To avoid fragmentation, IP and thus also UDP packets can only reliably sent with an MTU of <= 1500 on the public Internet.
                            const BUF_SIZE: usize = 1500;

                            let udp_socket = factory(&bind_addr)?;

                            let response = udp_socket
                                .handshake::<BUF_SIZE>(server, query.message.as_slice())
                                .await?;

                            let message = Message::from_octets(response)
                                .map_err(|_| io::Error::other("Failed to parse DNS message"))?;

                            Ok(message)
                        },
                        meta,
                    )
                    .is_err()
                {
                    tracing::debug!("Failed to queue UDP DNS query")
                }
            }
            dns::Transport::Tcp => {
                let factory = self.tcp_socket_factory.clone();
                let server = query.server;
                let meta = DnsQueryMetaData {
                    query: query.message.clone(),
                    server,
                    transport: dns::Transport::Tcp,
                };

                if self
                    .dns_queries
                    .try_push(
                        async move {
                            let tcp_socket = factory(&server)?;
                            let mut tcp_stream = tcp_socket.connect(server).await?;

                            let query = query.message.into_octets();
                            let dns_message_length = (query.len() as u16).to_be_bytes();

                            tcp_stream.write_all(&dns_message_length).await?;
                            tcp_stream.write_all(&query).await?;

                            let mut response_length = [0u8; 2];
                            tcp_stream.read_exact(&mut response_length).await?;
                            let response_length = u16::from_be_bytes(response_length) as usize;

                            // A u16 is at most 65k, meaning we are okay to allocate here based on what the remote is sending.
                            let mut response = vec![0u8; response_length];
                            tcp_stream.read_exact(&mut response).await?;

                            let message = Message::from_octets(response)
                                .map_err(|_| io::Error::other("Failed to parse DNS message"))?;

                            Ok(message)
                        },
                        meta,
                    )
                    .is_err()
                {
                    tracing::debug!("Failed to queue TCP DNS query")
                }
            }
        }
    }

    pub fn send_encrypted_packet(
        &mut self,
        packet: EncryptedPacket,
        buf: &EncryptBuffer,
    ) -> io::Result<()> {
        let transmit = packet.to_transmit(buf);
        let res = self.send_network(transmit);

        if res
            .as_ref()
            .err()
            .is_some_and(|e| e.kind() == io::ErrorKind::WouldBlock)
        {
            tracing::debug!(dst = %packet.dst(), "Socket busy");
            self.unwritten_packet = Some(packet);
        }

        res?;

        Ok(())
    }
}

async fn tun_send_recv(
    mut tun_rx: mpsc::Receiver<Box<dyn Tun>>,
    mut outbound_packet_rx: mpsc::Receiver<IpPacket>,
    inbound_packet_tx: mpsc::Sender<IpPacket>,
) {
    let mut device = Device::new();

    let mut command_stream = stream::select_all([
        new_tun_stream(&mut tun_rx),
        outgoing_packet_stream(&mut outbound_packet_rx),
    ]);

    loop {
        match future::select(
            command_stream.next(),
            future::poll_fn(|cx| device.poll_read(cx)),
        )
        .await
        {
            Either::Left((Some(Command::SendPacket(p)), _)) => {
                if let Err(e) = device.write(p) {
                    tracing::debug!("Failed to write TUN packet: {e}");
                };
            }
            Either::Left((Some(Command::UpdateTun(tun)), _)) => {
                device.set_tun(tun);
            }
            Either::Left((None, _)) => {
                tracing::debug!("Command stream closed");
                return;
            }
            Either::Right((Ok(p), _)) => {
                if inbound_packet_tx.send(p).await.is_err() {
                    tracing::debug!("Inbound packet channel closed");
                    return;
                };
            }
            Either::Right((Err(e), _)) => {
                tracing::debug!("Failed to read packet from TUN device: {e}");
                return;
            }
        };
    }
}

#[expect(
    clippy::large_enum_variant,
    reason = "We purposely don't want to allocate each IP packet."
)]
enum Command {
    UpdateTun(Box<dyn Tun>),
    SendPacket(IpPacket),
}

fn new_tun_stream(
    tun_rx: &mut mpsc::Receiver<Box<dyn Tun>>,
) -> Pin<Box<dyn Stream<Item = Command> + '_>> {
    Box::pin(stream::poll_fn(|cx| {
        tun_rx
            .poll_recv(cx)
            .map(|maybe_t| maybe_t.map(Command::UpdateTun))
    }))
}

fn outgoing_packet_stream(
    outbound_packet_rx: &mut mpsc::Receiver<IpPacket>,
) -> Pin<Box<dyn Stream<Item = Command> + '_>> {
    Box::pin(stream::poll_fn(|cx| {
        outbound_packet_rx
            .poll_recv(cx)
            .map(|maybe_p| maybe_p.map(Command::SendPacket))
    }))
}

fn is_max_wg_packet_size(d: &DatagramIn) -> bool {
    let len = d.packet.len();
    if len > MAX_DATAGRAM_PAYLOAD {
        tracing::debug!(from = %d.from, %len, "Dropping too large datagram (max allowed: {MAX_DATAGRAM_PAYLOAD} bytes)");

        return false;
    }

    true
}

/// An adapter struct between our managed TUN device and [`smoltcp`].
struct SmolDeviceAdapter {
    outbound_packet_tx: mpsc::Sender<IpPacket>,
    /// Packets that we have received on the TUN device and selected to be processed by [`smoltcp`].
    received_packets: VecDeque<IpPacket>,
}

impl smoltcp::phy::Device for SmolDeviceAdapter {
    type RxToken<'a> = SmolRxToken;
    type TxToken<'a> = SmolTxToken;

    fn receive(
        &mut self,
        _timestamp: smoltcp::time::Instant,
    ) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        let packet = self.received_packets.pop_front()?;

        Some((
            SmolRxToken { packet },
            SmolTxToken {
                outbound_packet_tx: self.outbound_packet_tx.clone(),
            },
        ))
    }

    fn transmit(&mut self, _timestamp: smoltcp::time::Instant) -> Option<Self::TxToken<'_>> {
        Some(SmolTxToken {
            outbound_packet_tx: self.outbound_packet_tx.clone(),
        })
    }

    fn capabilities(&self) -> smoltcp::phy::DeviceCapabilities {
        let mut caps = smoltcp::phy::DeviceCapabilities::default();
        caps.medium = smoltcp::phy::Medium::Ip;
        caps.max_transmission_unit = ip_packet::PACKET_SIZE;

        caps
    }
}

struct SmolTxToken {
    outbound_packet_tx: mpsc::Sender<IpPacket>,
}

impl smoltcp::phy::TxToken for SmolTxToken {
    fn consume<R, F>(self, len: usize, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        let mut ip_packet_buf = IpPacketBuf::new();
        let result = f(ip_packet_buf.buf());

        let mut ip_packet = IpPacket::new(ip_packet_buf, len).unwrap();
        ip_packet.update_checksum();
        self.outbound_packet_tx.try_send(ip_packet).unwrap();

        result
    }
}

struct SmolRxToken {
    packet: IpPacket,
}

impl smoltcp::phy::RxToken for SmolRxToken {
    fn consume<R, F>(mut self, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        f(self.packet.packet_mut())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn max_ip_channel_size_is_reasonable() {
        let one_ip_packet = std::mem::size_of::<IpPacket>();
        let max_channel_size = IP_CHANNEL_SIZE * one_ip_packet;

        assert_eq!(max_channel_size, 1_360_000); // 1.36MB is fine, we only have 2 of these channels, meaning less than 3MB additional buffer in total.
    }
}
