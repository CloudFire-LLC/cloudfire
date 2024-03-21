use crate::messages::ResourceDescription;
use ip_network::{Ipv4Network, Ipv6Network};
use serde::Serialize;
use std::fmt::Debug;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::path::PathBuf;

// Avoids having to map types for Windows
type RawFd = i32;

#[derive(Serialize, Clone, Copy, Debug)]
/// Identical to `ip_network::Ipv4Network` except we implement `Serialize` on the Rust side and the equivalent of `Deserialize` on the Swift / Kotlin side to avoid manually serializing and deserializing.
pub struct Cidrv4 {
    address: Ipv4Addr,
    prefix: u8,
}

/// Identical to `ip_network::Ipv6Network` except we implement `Serialize` on the Rust side and the equivalent of `Deserialize` on the Swift / Kotlin side to avoid manually serializing and deserializing.
#[derive(Serialize, Clone, Copy, Debug)]
pub struct Cidrv6 {
    address: Ipv6Addr,
    prefix: u8,
}

impl From<Ipv4Network> for Cidrv4 {
    fn from(value: Ipv4Network) -> Self {
        Self {
            address: value.network_address(),
            prefix: value.netmask(),
        }
    }
}

impl From<Ipv6Network> for Cidrv6 {
    fn from(value: Ipv6Network) -> Self {
        Self {
            address: value.network_address(),
            prefix: value.netmask(),
        }
    }
}

/// Traits that will be used by connlib to callback the client upper layers.
pub trait Callbacks: Clone + Send + Sync {
    /// Called when the tunnel address is set.
    ///
    /// This should return a new `fd` if there is one.
    /// (Only happens on android for now)
    fn on_set_interface_config(&self, _: Ipv4Addr, _: Ipv6Addr, _: Vec<IpAddr>) -> Option<RawFd> {
        None
    }

    /// Called when the tunnel is connected.
    fn on_tunnel_ready(&self) {
        tracing::trace!("tunnel_connected");
    }

    /// Called when the route list changes.
    fn on_update_routes(&self, _: Vec<Cidrv4>, _: Vec<Cidrv6>) -> Option<RawFd> {
        None
    }

    /// Called when the resource list changes.
    fn on_update_resources(&self, _: Vec<ResourceDescription>) {}

    /// Called when the tunnel is disconnected.
    ///
    /// If the tunnel disconnected due to a fatal error, `error` is the error
    /// that caused the disconnect.
    fn on_disconnect(&self, error: &crate::Error) {
        tracing::error!(error = ?error, "tunnel_disconnected");
        // Note that we can't panic here, since we already hooked the panic to this function.
        std::process::exit(0);
    }

    /// Protects the socket file descriptor from routing loops.
    #[cfg(target_os = "android")]
    fn protect_file_descriptor(&self, file_descriptor: std::os::fd::RawFd);

    fn roll_log_file(&self) -> Option<PathBuf> {
        None
    }
}
