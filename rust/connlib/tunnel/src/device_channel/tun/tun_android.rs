use crate::InterfaceConfig;
use connlib_shared::{CallbackErrorFacade, Callbacks, Error, Result, DNS_SENTINEL};
use ip_network::IpNetwork;
use libc::{
    close, ioctl, read, sockaddr, sockaddr_in, write, AF_INET, IFNAMSIZ, IPPROTO_IP, SIOCGIFMTU,
    SOCK_STREAM,
};
use std::{
    ffi::{c_int, c_short, c_uchar},
    io,
    os::fd::{AsRawFd, RawFd},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use tokio::io::unix::AsyncFd;

mod wrapped_socket;
// Android doesn't support Split DNS. So we intercept all requests and forward
// the non-Firezone name resolution requests to the upstream DNS resolver.
const DNS_FALLBACK_STRATEGY: &str = "upstream_resolver";

#[repr(C)]
union IfrIfru {
    ifru_addr: sockaddr,
    ifru_addr_v4: sockaddr_in,
    ifru_addr_v6: sockaddr_in,
    ifru_dstaddr: sockaddr,
    ifru_broadaddr: sockaddr,
    ifru_flags: c_short,
    ifru_metric: c_int,
    ifru_mtu: c_int,
    ifru_phys: c_int,
    ifru_media: c_int,
    ifru_intval: c_int,
    ifru_wake_flags: u32,
    ifru_route_refcnt: u32,
    ifru_cap: [c_int; 2],
    ifru_functional_type: u32,
}

#[repr(C)]
pub struct ifreq {
    ifr_name: [c_uchar; IFNAMSIZ],
    ifr_ifru: IfrIfru,
}

const TUNGETIFF: u64 = 0x800454d2;

#[derive(Debug)]
pub(crate) struct IfaceDevice(Arc<AsyncFd<IfaceStream>>);

#[derive(Debug)]
pub(crate) struct IfaceStream {
    closed: AtomicBool,
    fd: RawFd,
}

impl AsRawFd for IfaceStream {
    fn as_raw_fd(&self) -> RawFd {
        self.fd
    }
}

impl Drop for IfaceStream {
    fn drop(&mut self) {
        unsafe { close(self.fd) };
    }
}

impl IfaceStream {
    fn write(&self, buf: &[u8]) -> std::io::Result<usize> {
        if !self.closed.load(Ordering::Acquire) {
            match unsafe { write(self.fd, buf.as_ptr() as _, buf.len() as _) } {
                -1 => Err(io::Error::last_os_error()),
                n => Ok(n as usize),
            }
        } else {
            Err(std::io::Error::from_raw_os_error(9))
        }
    }

    pub fn write4(&self, src: &[u8]) -> std::io::Result<usize> {
        self.write(src)
    }

    pub fn write6(&self, src: &[u8]) -> std::io::Result<usize> {
        self.write(src)
    }

    pub fn read(&self, dst: &mut [u8]) -> std::io::Result<usize> {
        // We don't read(or write) again from the fd because the given fd number might be reclaimed
        // so this could make an spurious read/write to another fd and we DEFINETLY don't want that
        if !self.closed.load(Ordering::Acquire) {
            match unsafe { read(self.fd, dst.as_mut_ptr() as _, dst.len()) } {
                -1 => Err(io::Error::last_os_error()),
                n => Ok(n as usize),
            }
        } else {
            Err(std::io::Error::from_raw_os_error(9))
        }
    }

    pub fn close(&self) {
        self.closed.store(true, Ordering::Release);
    }
}

impl IfaceDevice {
    pub async fn new(
        config: &InterfaceConfig,
        callbacks: &CallbackErrorFacade<impl Callbacks>,
    ) -> Result<(Self, Arc<AsyncFd<IfaceStream>>)> {
        let fd = callbacks.on_set_interface_config(
            config.ipv4,
            config.ipv6,
            DNS_SENTINEL,
            DNS_FALLBACK_STRATEGY.to_string(),
        )?;
        let iface_stream = Arc::new(AsyncFd::new(IfaceStream {
            fd: fd.into(),
            closed: AtomicBool::new(false),
        })?);
        let this = Self(Arc::clone(&iface_stream));

        Ok((this, iface_stream))
    }

    fn name(&self) -> Result<String> {
        let mut ifr = ifreq {
            ifr_name: [0; IFNAMSIZ],
            ifr_ifru: unsafe { std::mem::zeroed() },
        };

        match unsafe { ioctl(self.0.get_ref().fd, TUNGETIFF as _, &mut ifr) } {
            0 => {
                let name_cstr = unsafe { std::ffi::CStr::from_ptr(ifr.ifr_name.as_ptr() as _) };
                Ok(name_cstr.to_string_lossy().into_owned())
            }
            _ => Err(get_last_error()),
        }
    }

    /// Get the current MTU value
    pub async fn mtu(&self) -> Result<usize> {
        let socket = wrapped_socket::WrappedSocket::new(AF_INET, SOCK_STREAM, IPPROTO_IP);
        let fd = match socket.as_raw_fd() {
            -1 => return Err(get_last_error()),
            fd => fd,
        };

        let name = self.name()?;
        let iface_name: &[u8] = name.as_ref();
        let mut ifr = ifreq {
            ifr_name: [0; IFNAMSIZ],
            ifr_ifru: IfrIfru { ifru_mtu: 0 },
        };

        ifr.ifr_name[..iface_name.len()].copy_from_slice(iface_name);

        if unsafe { ioctl(fd, SIOCGIFMTU as _, &ifr) } < 0 {
            return Err(get_last_error());
        }

        let mtu = unsafe { ifr.ifr_ifru.ifru_mtu };

        Ok(mtu as _)
    }

    pub async fn add_route(
        &self,
        route: IpNetwork,
        callbacks: &CallbackErrorFacade<impl Callbacks>,
    ) -> Result<Option<(Self, Arc<AsyncFd<IfaceStream>>)>> {
        self.0.get_ref().close();
        let fd = callbacks.on_add_route(route)?;
        let iface_stream = Arc::new(AsyncFd::new(IfaceStream {
            fd: fd.into(),
            closed: AtomicBool::new(false),
        })?);
        let this = Self(Arc::clone(&iface_stream));

        Ok(Some((this, iface_stream)))
    }

    pub async fn up(&self) -> Result<()> {
        Ok(())
    }
}

fn get_last_error() -> Error {
    Error::Io(io::Error::last_os_error())
}
