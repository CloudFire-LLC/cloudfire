use super::utils;
use crate::device_channel::{ipv4, ipv6};
use connlib_shared::{messages::Interface as InterfaceConfig, Callbacks, Error, Result};
use ip_network::IpNetwork;
use libc::{
    ctl_info, fcntl, getpeername, getsockopt, ioctl, iovec, msghdr, recvmsg, sendmsg, sockaddr_ctl,
    socklen_t, AF_INET, AF_INET6, AF_SYSTEM, CTLIOCGINFO, F_GETFL, F_SETFL, IF_NAMESIZE,
    O_NONBLOCK, SYSPROTO_CONTROL, UTUN_OPT_IFNAME,
};
use std::net::IpAddr;
use std::task::{Context, Poll};
use std::{
    collections::HashSet,
    io,
    mem::size_of,
    os::fd::{AsRawFd, RawFd},
};
use tokio::io::unix::AsyncFd;

const CTL_NAME: &[u8] = b"com.apple.net.utun_control";
/// `libc` for darwin doesn't define this constant so we declare it here.
pub(crate) const SIOCGIFMTU: u64 = 0x0000_0000_c020_6933;

#[derive(Debug)]
pub(crate) struct Tun {
    name: String,
    fd: AsyncFd<RawFd>,
}

impl Tun {
    pub fn write4(&self, src: &[u8]) -> std::io::Result<usize> {
        self.write(src, AF_INET as u8)
    }

    pub fn write6(&self, src: &[u8]) -> std::io::Result<usize> {
        self.write(src, AF_INET6 as u8)
    }

    pub fn poll_read(&self, buf: &mut [u8], cx: &mut Context<'_>) -> Poll<io::Result<usize>> {
        utils::poll_raw_fd(&self.fd, |fd| read(fd, buf), cx)
    }

    fn write(&self, src: &[u8], af: u8) -> std::io::Result<usize> {
        let mut hdr = [0, 0, 0, af];
        let mut iov = [
            iovec {
                iov_base: hdr.as_mut_ptr() as _,
                iov_len: hdr.len(),
            },
            iovec {
                iov_base: src.as_ptr() as _,
                iov_len: src.len(),
            },
        ];

        let msg_hdr = msghdr {
            msg_name: std::ptr::null_mut(),
            msg_namelen: 0,
            msg_iov: &mut iov[0],
            msg_iovlen: iov.len() as _,
            msg_control: std::ptr::null_mut(),
            msg_controllen: 0,
            msg_flags: 0,
        };

        match unsafe { sendmsg(self.fd.as_raw_fd(), &msg_hdr, 0) } {
            -1 => Err(io::Error::last_os_error()),
            n => Ok(n as usize),
        }
    }

    pub fn new(
        config: &InterfaceConfig,
        dns_config: Vec<IpAddr>,
        callbacks: &impl Callbacks,
    ) -> Result<Self> {
        let mut info = ctl_info {
            ctl_id: 0,
            ctl_name: [0; 96],
        };
        info.ctl_name[..CTL_NAME.len()]
            // SAFETY: We only care about maintaining the same byte value not the same value,
            // meaning that the slice &[u8] here is just a blob of bytes for us, we need this conversion
            // just because `c_char` is i8 (for some reason).
            // One thing I don't like about this is that `ctl_name` is actually a nul-terminated string,
            // which we are only getting because `CTRL_NAME` is less than 96 bytes long and we are 0-value
            // initializing the array we should be using a CStr to be explicit... but this is slightly easier.
            .copy_from_slice(unsafe { &*(CTL_NAME as *const [u8] as *const [i8]) });

        // On Apple platforms, we must use a NetworkExtension for reading and writing
        // packets if we want to be allowed in the iOS and macOS App Stores. This has the
        // unfortunate side effect that we're not allowed to create or destroy the tunnel
        // interface ourselves. The file descriptor should already be opened by the NetworkExtension for us
        // by this point. So instead, we iterate through all file descriptors looking for the one corresponding
        // to the utun interface we have access to read and write from.
        //
        // Credit to Jason Donenfeld (@zx2c4) for this technique. See docs/NOTICE.txt for attribution.
        // https://github.com/WireGuard/wireguard-apple/blob/master/Sources/WireGuardKit/WireGuardAdapter.swift
        for fd in 0..1024 {
            tracing::debug!("Checking fd {}", fd);

            // initialize empty sockaddr_ctl to be populated by getpeername
            let mut addr = sockaddr_ctl {
                sc_len: size_of::<sockaddr_ctl>() as u8,
                sc_family: 0,
                ss_sysaddr: 0,
                sc_id: info.ctl_id,
                sc_unit: 0,
                sc_reserved: Default::default(),
            };

            let mut len = size_of::<sockaddr_ctl>() as u32;
            let ret = unsafe {
                getpeername(
                    fd,
                    &mut addr as *mut sockaddr_ctl as _,
                    &mut len as *mut socklen_t,
                )
            };
            if ret != 0 || addr.sc_family != AF_SYSTEM as u8 {
                continue;
            }

            if info.ctl_id == 0 {
                let ret = unsafe { ioctl(fd, CTLIOCGINFO, &mut info as *mut ctl_info) };

                if ret != 0 {
                    continue;
                }
            }

            if addr.sc_id == info.ctl_id {
                callbacks.on_set_interface_config(config.ipv4, config.ipv6, dns_config);

                set_non_blocking(fd)?;

                return Ok(Self {
                    name: name(fd)?,
                    fd: AsyncFd::new(fd)?,
                });
            }
        }

        Err(get_last_error())
    }

    pub fn set_routes(&self, routes: HashSet<IpNetwork>, callbacks: &impl Callbacks) -> Result<()> {
        // This will always be None in macos
        callbacks.on_update_routes(
            routes.iter().copied().filter_map(ipv4).collect(),
            routes.iter().copied().filter_map(ipv6).collect(),
        );

        Ok(())
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }
}

fn get_last_error() -> Error {
    Error::Io(io::Error::last_os_error())
}

fn set_non_blocking(fd: RawFd) -> Result<()> {
    match unsafe { fcntl(fd, F_GETFL) } {
        -1 => Err(get_last_error()),
        flags => match unsafe { fcntl(fd, F_SETFL, flags | O_NONBLOCK) } {
            -1 => Err(get_last_error()),
            _ => Ok(()),
        },
    }
}

fn read(fd: RawFd, dst: &mut [u8]) -> std::io::Result<usize> {
    let mut hdr = [0u8; 4];

    let mut iov = [
        iovec {
            iov_base: hdr.as_mut_ptr() as _,
            iov_len: hdr.len(),
        },
        iovec {
            iov_base: dst.as_mut_ptr() as _,
            iov_len: dst.len(),
        },
    ];

    let mut msg_hdr = msghdr {
        msg_name: std::ptr::null_mut(),
        msg_namelen: 0,
        msg_iov: &mut iov[0],
        msg_iovlen: iov.len() as _,
        msg_control: std::ptr::null_mut(),
        msg_controllen: 0,
        msg_flags: 0,
    };

    // Safety: Within this module, the file descriptor is always valid.
    match unsafe { recvmsg(fd, &mut msg_hdr, 0) } {
        -1 => Err(io::Error::last_os_error()),
        0..=4 => Ok(0),
        n => Ok((n - 4) as usize),
    }
}

fn name(fd: RawFd) -> Result<String> {
    let mut tunnel_name = [0u8; IF_NAMESIZE];
    let mut tunnel_name_len = tunnel_name.len() as socklen_t;
    if unsafe {
        getsockopt(
            fd,
            SYSPROTO_CONTROL,
            UTUN_OPT_IFNAME,
            tunnel_name.as_mut_ptr() as _,
            &mut tunnel_name_len,
        )
    } < 0
        || tunnel_name_len == 0
    {
        return Err(get_last_error());
    }

    Ok(String::from_utf8_lossy(&tunnel_name[..(tunnel_name_len - 1) as usize]).to_string())
}
