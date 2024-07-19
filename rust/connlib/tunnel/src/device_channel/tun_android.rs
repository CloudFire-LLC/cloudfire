use super::utils;
use crate::device_channel::ioctl;
use std::task::{Context, Poll};
use std::{
    io,
    os::fd::{AsRawFd, RawFd},
};
use tokio::io::unix::AsyncFd;

#[derive(Debug)]
pub struct Tun {
    fd: AsyncFd<RawFd>,
    name: String,
}

impl Drop for Tun {
    fn drop(&mut self) {
        unsafe { libc::close(self.fd.as_raw_fd()) };
    }
}

impl Tun {
    pub fn write4(&self, src: &[u8]) -> std::io::Result<usize> {
        write(self.fd.as_raw_fd(), src)
    }

    pub fn write6(&self, src: &[u8]) -> std::io::Result<usize> {
        write(self.fd.as_raw_fd(), src)
    }

    pub fn poll_read(&self, buf: &mut [u8], cx: &mut Context<'_>) -> Poll<io::Result<usize>> {
        utils::poll_raw_fd(&self.fd, |fd| read(fd, buf), cx)
    }

    /// Create a new [`Tun`] from a raw file descriptor.
    ///
    /// # Safety
    ///
    /// The file descriptor must be open.
    pub unsafe fn from_fd(fd: RawFd) -> io::Result<Self> {
        let name = interface_name(fd)?;

        Ok(Tun {
            fd: AsyncFd::new(fd)?,
            name,
        })
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }
}

/// Retrieves the name of the interface pointed to by the provided file descriptor.
///
/// # Safety
///
/// The file descriptor must be open.
unsafe fn interface_name(fd: RawFd) -> io::Result<String> {
    const TUNGETIFF: libc::c_ulong = 0x800454d2;
    let mut request = ioctl::Request::<GetInterfaceNamePayload>::new();

    ioctl::exec(fd, TUNGETIFF, &mut request)?;

    Ok(request.name().to_string())
}

impl ioctl::Request<GetInterfaceNamePayload> {
    fn new() -> Self {
        Self {
            name: [0u8; libc::IF_NAMESIZE],
            payload: Default::default(),
        }
    }

    fn name(&self) -> std::borrow::Cow<'_, str> {
        // Safety: The memory of `self.name` is always initialized.
        let cstr = unsafe { std::ffi::CStr::from_ptr(self.name.as_ptr() as _) };

        cstr.to_string_lossy()
    }
}

#[derive(Default)]
#[repr(C)]
struct GetInterfaceNamePayload {
    // Fixes a nasty alignment bug on 32-bit architectures on Android.
    // The `name` field in `ioctl::Request` is only 16 bytes long and accessing it causes a NPE without this alignment.
    // Why? Not sure. It seems to only happen in release mode which hints at an optimisation issue.
    alignment: [std::ffi::c_uchar; 16],
}

/// Read from the given file descriptor in the buffer.
fn read(fd: RawFd, dst: &mut [u8]) -> io::Result<usize> {
    // Safety: Within this module, the file descriptor is always valid.
    match unsafe { libc::read(fd, dst.as_mut_ptr() as _, dst.len()) } {
        -1 => Err(io::Error::last_os_error()),
        n => Ok(n as usize),
    }
}

/// Write the buffer to the given file descriptor.
fn write(fd: RawFd, buf: &[u8]) -> io::Result<usize> {
    // Safety: Within this module, the file descriptor is always valid.
    match unsafe { libc::write(fd.as_raw_fd(), buf.as_ptr() as _, buf.len() as _) } {
        -1 => Err(io::Error::last_os_error()),
        n => Ok(n as usize),
    }
}
