//! Gives Firezone DNS privilege over other DNS resolvers on the system
//!
//! This uses NRPT and claims all domains, similar to the `systemd-resolved` control method
//! on Linux.
//! This allows us to "shadow" DNS resolvers that are configured by the user or DHCP on
//! physical interfaces, as long as they don't have any NRPT rules that outrank us.
//!
//! If Firezone crashes, restarting Firezone and closing it gracefully will resume
//! normal DNS operation. The Powershell command to remove the NRPT rule can also be run
//! by hand.
//!
//! The system default resolvers don't need to be reverted because they're never deleted.
//!
//! <https://superuser.com/a/1752670>

use super::DnsController;
use anyhow::{Context as _, Result};
use firezone_bin_shared::platform::{DnsControlMethod, CREATE_NO_WINDOW, TUNNEL_UUID};
use std::{
    io::ErrorKind, net::IpAddr, os::windows::process::CommandExt, path::Path, process::Command,
};
use windows::Win32::System::GroupPolicy::{RefreshPolicyEx, RP_FORCE};

// Unique magic number that we can use to delete our well-known NRPT rule.
// Copied from the deep link schema
const FZ_MAGIC: &str = "firezone-fd0020211111";

impl DnsController {
    /// Deactivate any control Firezone has over the computer's DNS
    ///
    /// Must be `sync` so we can call it from `Drop`
    pub fn deactivate(&mut self) -> Result<()> {
        let hklm = winreg::RegKey::predef(winreg::enums::HKEY_LOCAL_MACHINE);
        if let Err(error) = hklm.delete_subkey(local_nrpt_path().join(NRPT_REG_KEY)) {
            if error.kind() != ErrorKind::NotFound {
                tracing::error!(?error, "Couldn't delete local NRPT");
            }
        }
        if let Err(error) = hklm.delete_subkey(group_nrpt_path().join(NRPT_REG_KEY)) {
            if error.kind() != ErrorKind::NotFound {
                tracing::error!(?error, "Couldn't delete Group Policy NRPT");
            }
        }
        refresh_group_policy()?;
        tracing::info!("Deactivated DNS control");
        Ok(())
    }

    /// Set the computer's system-wide DNS servers
    ///
    /// The `mut` in `&mut self` is not needed by Rust's rules, but
    /// it would be bad if this was called from 2 threads at once.
    ///
    /// Must be async and an owned `Vec` to match the Linux signature
    #[expect(clippy::unused_async)]
    pub async fn set_dns(&mut self, dns_config: Vec<IpAddr>) -> Result<()> {
        match self.dns_control_method {
            DnsControlMethod::Disabled => {}
            DnsControlMethod::Nrpt => {
                activate(&dns_config).context("Failed to activate DNS control")?
            }
        }
        Ok(())
    }

    /// Flush Windows' system-wide DNS cache
    ///
    /// `&self` is needed to match the Linux signature
    pub fn flush(&self) -> Result<()> {
        tracing::debug!("Flushing Windows DNS cache...");
        Command::new("ipconfig")
            .creation_flags(CREATE_NO_WINDOW)
            .args(["/flushdns"])
            .status()?;
        tracing::debug!("Flushed DNS.");
        Ok(())
    }
}

pub(crate) fn system_resolvers(_method: DnsControlMethod) -> Result<Vec<IpAddr>> {
    let resolvers = ipconfig::get_adapters()?
        .iter()
        .flat_map(|adapter| adapter.dns_servers())
        .filter(|ip| match ip {
            IpAddr::V4(_) => true,
            // Filter out bogus DNS resolvers on my dev laptop that start with fec0:
            IpAddr::V6(ip) => !ip.octets().starts_with(&[0xfe, 0xc0]),
        })
        .copied()
        .collect();
    // This is private, so keep it at `debug` or `trace`
    tracing::debug!(?resolvers);
    Ok(resolvers)
}

/// A UUID for the Firezone Client NRPT rule, chosen randomly at dev time.
///
/// Our NRPT rule should always live in the registry at
/// `Computer\HKEY_LOCAL_MACHINE\SYSTEM\CurrentControlSet\Services\Dnscache\Parameters\DnsPolicyConfig\$NRPT_REG_KEY`
///
/// We can use this UUID as a handle to enable, disable, or modify the rule.
const NRPT_REG_KEY: &str = "{6C0507CB-C884-4A78-BC55-0ACEE21227F6}";

/// Tells Windows to send all DNS queries to our sentinels
///
/// Parameters:
/// - `dns_config_string`: Comma-separated IP addresses of DNS servers, e.g. "1.1.1.1,8.8.8.8"
fn activate(dns_config: &[IpAddr]) -> Result<()> {
    // TODO: Known issue where web browsers will keep a connection open to a site,
    // using QUIC, HTTP/2, or even HTTP/1.1, and so they won't resolve the DNS
    // again unless you let that connection time out:
    // <https://github.com/firezone/firezone/issues/3113#issuecomment-1882096111>
    tracing::info!("Activating DNS control...");

    let hklm = winreg::RegKey::predef(winreg::enums::HKEY_LOCAL_MACHINE);

    let dns_config_string = dns_config
        .iter()
        .map(|ip| ip.to_string())
        .collect::<Vec<_>>()
        .join(";");

    // Set our DNS servers on the interface itself too, so that `ipconfig` and WSL can pick them up (Fixes #6777)
    // `Computer\HKEY_LOCAL_MACHINE\SYSTEM\CurrentControlSet\Services\Tcpip\Parameters\Interfaces\{e9245bc1-b8c1-44ca-ab1d-c6aad4f13b9c}`
    let key = hklm.open_subkey(
        Path::new(r"SYSTEM\CurrentControlSet\Services\Tcpip\Parameters\Interfaces")
            .join(format!("{{{TUNNEL_UUID}}}")),
    )?;
    key.set_value("NameServer", &dns_config_string)?;

    // It's safe to always set the local rule.
    let (key, _) = hklm.create_subkey(local_nrpt_path().join(NRPT_REG_KEY))?;
    set_nrpt_rule(&key, &dns_config_string)?;

    // If this key exists, our local NRPT rules are ignored and we have to stick
    // them in with group policies for some reason.
    let group_policy_key_exists = hklm.open_subkey(group_nrpt_path()).is_ok();
    tracing::debug!(?group_policy_key_exists);
    if group_policy_key_exists {
        // TODO: Possible TOCTOU problem - We check whether the key exists, then create a subkey if it does. If Group Policy is disabled between those two steps, and something else removes that parent key, we'll re-create it, which might be bad. We can set up unit tests to see if it's possible to avoid this in the registry, but for now it's not a huge deal.
        let (key, _) = hklm.create_subkey(group_nrpt_path().join(NRPT_REG_KEY))?;
        set_nrpt_rule(&key, &dns_config_string)?;
        refresh_group_policy()?;
    }

    tracing::info!("DNS control active.");

    Ok(())
}

/// Returns the registry path we can use to set NRPT rules when Group Policy is not in effect.
fn local_nrpt_path() -> &'static Path {
    // Must be backslashes.
    Path::new(r"SYSTEM\CurrentControlSet\Services\Dnscache\Parameters\DnsPolicyConfig")
}

/// Returns the registry path we can use to set NRPT rules when Group Policy is in effect.
fn group_nrpt_path() -> &'static Path {
    // Must be backslashes.
    Path::new(r"SOFTWARE\Policies\Microsoft\Windows NT\DNSClient\DnsPolicyConfig")
}

fn refresh_group_policy() -> Result<()> {
    // SAFETY: No pointers involved, and the docs say nothing about threads.
    unsafe { RefreshPolicyEx(true, RP_FORCE) }?;
    Ok(())
}

/// Returns

/// Given the path of a registry key, sets the parameters of an NRPT rule on it.
fn set_nrpt_rule(key: &winreg::RegKey, dns_config_string: &str) -> Result<()> {
    key.set_value("Comment", &FZ_MAGIC)?;
    key.set_value("ConfigOptions", &0x8u32)?;
    key.set_value("DisplayName", &"Firezone SplitDNS")?;
    key.set_value("GenericDNSServers", &dns_config_string)?;
    key.set_value("IPSECCARestriction", &"")?;
    key.set_value("Name", &vec!["."])?;
    key.set_value("Version", &0x2u32)?;
    Ok(())
}
