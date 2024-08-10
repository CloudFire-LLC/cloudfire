mod network_changes;
mod tun_device_manager;

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "linux")]
pub use linux as platform;

#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "windows")]
pub use windows as platform;

use tracing_log::LogTracer;
use tracing_subscriber::{
    fmt, prelude::__tracing_subscriber_SubscriberExt, EnvFilter, Layer, Registry,
};

/// Output of `git describe` at compile time
/// e.g. `1.0.0-pre.4-20-ged5437c88-modified` where:
///
/// * `1.0.0-pre.4` is the most recent ancestor tag
/// * `20` is the number of commits since then
/// * `g` doesn't mean anything
/// * `ed5437c88` is the Git commit hash
/// * `-modified` is present if the working dir has any changes from that commit number
pub const GIT_VERSION: &str = git_version::git_version!(
    args = ["--always", "--dirty=-modified", "--tags"],
    fallback = "unknown"
);

pub const TOKEN_ENV_KEY: &str = "FIREZONE_TOKEN";

// wintun automatically append " Tunnel" to this
pub const TUNNEL_NAME: &str = "Firezone";

/// Bundle ID / App ID that the client uses to distinguish itself from other programs on the system
///
/// e.g. In ProgramData and AppData we use this to name our subdirectories for configs and data,
/// and Windows may use it to track things like the MSI installer, notification titles,
/// deep link registration, etc.
///
/// This should be identical to the `tauri.bundle.identifier` over in `tauri.conf.json`,
/// but sometimes I need to use this before Tauri has booted up, or in a place where
/// getting the Tauri app handle would be awkward.
///
/// Luckily this is also the AppUserModelId that Windows uses to label notifications,
/// so if your dev system has Firezone installed by MSI, the notifications will look right.
/// <https://learn.microsoft.com/en-us/windows/configuration/find-the-application-user-model-id-of-an-installed-app>
pub const BUNDLE_ID: &str = "dev.firezone.client";

/// Mark for Firezone sockets to prevent routing loops on Linux.
pub const FIREZONE_MARK: u32 = 0xfd002021;

#[cfg(any(target_os = "linux", target_os = "windows"))]
pub use network_changes::{new_dns_notifier, new_network_notifier};

#[cfg(any(target_os = "linux", target_os = "windows"))]
pub use tun_device_manager::TunDeviceManager;

pub fn setup_global_subscriber<L>(additional_layer: L)
where
    L: Layer<Registry> + Send + Sync,
{
    let subscriber = Registry::default()
        .with(additional_layer)
        .with(fmt::layer())
        .with(EnvFilter::from_default_env());
    tracing::subscriber::set_global_default(subscriber).expect("Could not set global default");
    LogTracer::init().unwrap();
}
