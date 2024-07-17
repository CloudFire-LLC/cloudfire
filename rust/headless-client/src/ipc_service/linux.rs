use super::CliCommon;
use crate::{known_dirs, signals};
use anyhow::{bail, Result};

/// Cross-platform entry point for systemd / Windows services
///
/// Linux uses the CLI args from here, Windows does not
pub(crate) fn run_ipc_service(cli: CliCommon) -> Result<()> {
    let _handle = super::setup_logging(cli.log_dir.or_else(|| known_dirs::ipc_service_logs()))?;
    if !nix::unistd::getuid().is_root() {
        anyhow::bail!("This is the IPC service binary, it's not meant to run interactively.");
    }
    let rt = tokio::runtime::Runtime::new()?;
    let _guard = rt.enter();
    let mut signals = signals::Terminate::new()?;

    rt.block_on(super::ipc_listen(&mut signals))
}

pub(crate) fn install_ipc_service() -> Result<()> {
    bail!("`install_ipc_service` not implemented and not needed on Linux")
}
