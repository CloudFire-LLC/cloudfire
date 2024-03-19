use connlib_shared::{messages::Interface as InterfaceConfig, Callbacks, Error, Result};
use ip_network::IpNetwork;
use std::{
    collections::HashSet,
    io,
    net::{IpAddr, SocketAddrV4, SocketAddrV6},
    os::windows::process::CommandExt,
    process::{Command, Stdio},
    str::FromStr,
    sync::Arc,
    task::{ready, Context, Poll},
};
use tokio::sync::mpsc;
use windows::Win32::{
    NetworkManagement::{
        IpHelper::{
            CreateIpForwardEntry2, DeleteIpForwardEntry2, GetIpInterfaceEntry,
            InitializeIpForwardEntry, SetIpInterfaceEntry, MIB_IPFORWARD_ROW2, MIB_IPINTERFACE_ROW,
        },
        Ndis::NET_LUID_LH,
    },
    Networking::WinSock::{AF_INET, AF_INET6},
};

// wintun automatically appends " Tunnel" to this
const TUNNEL_NAME: &str = "Firezone";

// TODO: Double-check that all these get dropped gracefully on disconnect
pub struct Tun {
    _adapter: Arc<wintun::Adapter>,
    /// The index of our network adapter, we can use this when asking Windows to add / remove routes / DNS rules
    /// It's stable across app restarts and I'm assuming across system reboots too.
    iface_idx: u32,
    // TODO: Get rid of this mutex. It's a hack to deal with `poll_read` taking a `&self` instead of `&mut self`
    packet_rx: std::sync::Mutex<mpsc::Receiver<wintun::Packet>>,
    _recv_thread: std::thread::JoinHandle<()>,
    session: Arc<wintun::Session>,
    routes: HashSet<IpNetwork>,
}

impl Drop for Tun {
    fn drop(&mut self) {
        if let Err(e) = self.session.shutdown() {
            tracing::error!("wintun::Session::shutdown: {e:#?}");
        }
    }
}

// Hides Powershell's console on Windows
// <https://stackoverflow.com/questions/59692146/is-it-possible-to-use-the-standard-library-to-spawn-a-process-without-showing-th#60958956>
const CREATE_NO_WINDOW: u32 = 0x08000000;
// Copied from tun_linux.rs
const DEFAULT_MTU: u32 = 1280;

impl Tun {
    pub fn new(config: &InterfaceConfig, dns_config: Vec<IpAddr>) -> Result<Self> {
        const TUNNEL_UUID: &str = "e9245bc1-b8c1-44ca-ab1d-c6aad4f13b9c";

        // SAFETY: we're loading a DLL from disk and it has arbitrary C code in it.
        // The Windows client, in `wintun_install` hashes the DLL at startup, before calling connlib, so it's unlikely for the DLL to be accidentally corrupted by the time we get here.
        let path = connlib_shared::windows::wintun_dll_path()?;
        let wintun = unsafe { wintun::load_from_path(path) }?;
        let uuid = uuid::Uuid::from_str(TUNNEL_UUID)?;
        let adapter =
            match wintun::Adapter::create(&wintun, "Firezone", TUNNEL_NAME, Some(uuid.as_u128())) {
                Ok(x) => x,
                Err(e) => {
                    tracing::error!(
                        "wintun::Adapter::create failed, probably need admin powers: {}",
                        e
                    );
                    return Err(e.into());
                }
            };

        tracing::debug!("Setting our IPv4 = {}", config.ipv4);
        tracing::debug!("Setting our IPv6 = {}", config.ipv6);

        // TODO: See if there's a good Win32 API for this
        // Using netsh directly instead of wintun's `set_network_addresses_tuple` because their code doesn't work for IPv6
        Command::new("netsh")
            .creation_flags(CREATE_NO_WINDOW)
            .arg("interface")
            .arg("ipv4")
            .arg("set")
            .arg("address")
            .arg(format!("name=\"{TUNNEL_NAME}\""))
            .arg("source=static")
            .arg(format!("address={}", config.ipv4))
            .arg("mask=255.255.255.255")
            .stdout(Stdio::null())
            .status()?;

        Command::new("netsh")
            .creation_flags(CREATE_NO_WINDOW)
            .arg("interface")
            .arg("ipv6")
            .arg("set")
            .arg("address")
            .arg(format!("interface=\"{TUNNEL_NAME}\""))
            .arg(format!("address={}", config.ipv6))
            .stdout(Stdio::null())
            .status()?;

        tracing::debug!("Our IPs are {:?}", adapter.get_addresses()?);

        let iface_idx = adapter.get_adapter_index()?;

        // Remove any routes that were previously associated with us
        // TODO: Pick a more elegant way to do this
        Command::new("powershell")
            .creation_flags(CREATE_NO_WINDOW)
            .arg("-Command")
            .arg(format!(
                "Remove-NetRoute -InterfaceIndex {iface_idx} -Confirm:$false"
            ))
            .stdout(Stdio::null())
            .status()?;

        set_iface_config(adapter.get_luid(), DEFAULT_MTU)?;

        // Set our DNS IP as the DNS server for our interface
        // TODO: Known issue where web browsers will keep a connection open to a site,
        // using QUIC, HTTP/2, or even HTTP/1.1, and so they won't resolve the DNS
        // again unless you let that connection time out:
        // <https://github.com/firezone/firezone/issues/3113#issuecomment-1882096111>
        // TODO: If we have a Windows gateway, it shouldn't configure DNS, right?
        Command::new("powershell")
            .creation_flags(CREATE_NO_WINDOW)
            .arg("-Command")
            .arg(format!(
                "Set-DnsClientServerAddress -InterfaceIndex {iface_idx} -ServerAddresses({})",
                dns_config
                    .iter()
                    .map(|ip| format!("\"{ip}\""))
                    .collect::<Vec<_>>()
                    .join(",")
            ))
            .stdout(Stdio::null())
            .status()?;

        let session = Arc::new(adapter.start_session(wintun::MAX_RING_CAPACITY)?);

        let (packet_tx, packet_rx) = mpsc::channel(5);

        let recv_thread = start_recv_thread(packet_tx, Arc::clone(&session))?;
        let packet_rx = std::sync::Mutex::new(packet_rx);

        Ok(Self {
            _adapter: adapter,
            iface_idx,
            _recv_thread: recv_thread,
            packet_rx,
            session: Arc::clone(&session),
            routes: HashSet::new(),
        })
    }

    // It's okay if this blocks until the route is added in the OS.
    pub fn set_routes(
        &mut self,
        new_routes: HashSet<IpNetwork>,
        _callbacks: &impl Callbacks,
    ) -> Result<()> {
        for new_route in new_routes.difference(&self.routes) {
            self.add_route(*new_route)?;
        }

        for old_route in self.routes.difference(&new_routes) {
            self.remove_route(*old_route)?;
        }

        self.routes = new_routes;

        Ok(())
    }

    // It's okay if this blocks until the route is added in the OS.
    fn add_route(&self, route: IpNetwork) -> Result<()> {
        const DUPLICATE_ERR: u32 = 0x80071392;
        let entry = self.forward_entry(route);

        // SAFETY: Windows shouldn't store the reference anywhere, it's just a way to pass lots of arguments at once. And no other thread sees this variable.
        match unsafe { CreateIpForwardEntry2(&entry) }.ok() {
            Ok(()) => Ok(()),
            Err(e) if e.code().0 as u32 == DUPLICATE_ERR => {
                tracing::debug!(%route, "Failed to add duplicate route, ignoring");
                Ok(())
            }
            Err(e) => Err(e.into()),
        }
    }

    // It's okay if this blocks until the route is removed in the OS.
    fn remove_route(&self, route: IpNetwork) -> Result<()> {
        let entry = self.forward_entry(route);

        // SAFETY: Windows shouldn't store the reference anywhere, it's just a way to pass lots of arguments at once. And no other thread sees this variable.
        unsafe { DeleteIpForwardEntry2(&entry) }.ok()?;
        Ok(())
    }

    pub fn poll_read(&self, buf: &mut [u8], cx: &mut Context<'_>) -> Poll<io::Result<usize>> {
        let mut packet_rx = self.packet_rx.try_lock().unwrap();

        let pkt = ready!(packet_rx.poll_recv(cx));

        match pkt {
            Some(pkt) => {
                let bytes = pkt.bytes();
                let len = bytes.len();
                if len > buf.len() {
                    // This shouldn't happen now that we set IPv4 and IPv6 MTU
                    // If it does, something is wrong.
                    tracing::warn!("Packet is too long to read ({len} bytes)");
                    return Poll::Ready(Ok(0));
                }
                buf[0..len].copy_from_slice(bytes);
                Poll::Ready(Ok(len))
            }
            None => {
                tracing::error!("error receiving packet from mpsc channel");
                Poll::Ready(Err(std::io::ErrorKind::Other.into()))
            }
        }
    }

    pub fn name(&self) -> &str {
        TUNNEL_NAME
    }

    pub fn write4(&self, bytes: &[u8]) -> io::Result<usize> {
        self.write(bytes)
    }

    pub fn write6(&self, bytes: &[u8]) -> io::Result<usize> {
        self.write(bytes)
    }

    fn write(&self, bytes: &[u8]) -> io::Result<usize> {
        let len = bytes
            .len()
            .try_into()
            .expect("Packet length should fit into u16");

        let Ok(mut pkt) = self.session.allocate_send_packet(len) else {
            // Ring buffer is full, just drop the packet since we're at the IP layer
            return Ok(0);
        };

        pkt.bytes_mut().copy_from_slice(bytes);
        // `send_packet` cannot fail to enqueue the packet, since we already allocated
        // space in the ring buffer.
        self.session.send_packet(pkt);
        Ok(bytes.len())
    }

    fn forward_entry(&self, route: IpNetwork) -> MIB_IPFORWARD_ROW2 {
        let mut row = MIB_IPFORWARD_ROW2::default();
        // SAFETY: Windows shouldn't store the reference anywhere, it's just setting defaults
        unsafe { InitializeIpForwardEntry(&mut row) };

        let prefix = &mut row.DestinationPrefix;
        match route {
            IpNetwork::V4(x) => {
                prefix.PrefixLength = x.netmask();
                prefix.Prefix.Ipv4 = SocketAddrV4::new(x.network_address(), 0).into();
            }
            IpNetwork::V6(x) => {
                prefix.PrefixLength = x.netmask();
                prefix.Prefix.Ipv6 = SocketAddrV6::new(x.network_address(), 0, 0, 0).into();
            }
        }

        row.InterfaceIndex = self.iface_idx;
        row.Metric = 0;

        row
    }
}

fn start_recv_thread(
    packet_tx: mpsc::Sender<wintun::Packet>,
    session: Arc<wintun::Session>,
) -> io::Result<std::thread::JoinHandle<()>> {
    std::thread::Builder::new()
        .name("Firezone wintun worker".into())
        .spawn(move || {
            loop {
                match session.receive_blocking() {
                    Ok(pkt) => {
                        if packet_tx.blocking_send(pkt).is_err() {
                            // Most likely the receiver was dropped and we're closing down the connlib session.
                            break;
                        }
                    }
                    Err(wintun::Error::ShuttingDown) => break,
                    Err(e) => {
                        tracing::error!("wintun::Session::receive_blocking: {e:#?}");
                        break;
                    }
                }
            }
            tracing::debug!("recv_task exiting gracefully");
        })
}

/// Sets MTU on the interface
/// TODO: Set IP and other things in here too, so the code is more organized
fn set_iface_config(luid: wintun::NET_LUID_LH, mtu: u32) -> Result<()> {
    // SAFETY: Both NET_LUID_LH unions should be the same. We're just copying out
    // the u64 value and re-wrapping it, since wintun doesn't refer to the windows
    // crate's version of NET_LUID_LH.
    let luid = NET_LUID_LH {
        Value: unsafe { luid.Value },
    };

    // Set MTU for IPv4
    {
        let mut row = MIB_IPINTERFACE_ROW {
            Family: AF_INET,
            InterfaceLuid: luid,
            ..Default::default()
        };

        // SAFETY: TODO
        unsafe { GetIpInterfaceEntry(&mut row) }.ok()?;

        // https://stackoverflow.com/questions/54857292/setipinterfaceentry-returns-error-invalid-parameter
        row.SitePrefixLength = 0;

        // Set MTU for IPv4
        row.NlMtu = mtu;

        // SAFETY: TODO
        unsafe { SetIpInterfaceEntry(&mut row) }.ok()?;
    }

    // Set MTU for IPv6
    {
        let mut row = MIB_IPINTERFACE_ROW {
            Family: AF_INET6,
            InterfaceLuid: luid,
            ..Default::default()
        };

        // SAFETY: TODO
        unsafe { GetIpInterfaceEntry(&mut row) }.ok()?;

        // https://stackoverflow.com/questions/54857292/setipinterfaceentry-returns-error-invalid-parameter
        row.SitePrefixLength = 0;

        // Set MTU for IPv4
        row.NlMtu = mtu;

        // SAFETY: TODO
        unsafe { SetIpInterfaceEntry(&mut row) }.ok()?;
    }
    Ok(())
}
