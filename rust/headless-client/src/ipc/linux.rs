use super::{Error, ServiceId};
use anyhow::{Context as _, Result};
use std::{io::ErrorKind, os::unix::fs::PermissionsExt, path::PathBuf};
use tokio::net::{UnixListener, UnixStream};

pub(crate) struct Server {
    listener: UnixListener,
}

/// Opaque wrapper around the client's half of a platform-specific IPC stream
pub type ClientStream = UnixStream;

/// Opaque wrapper around the server's half of a platform-specific IPC stream
///
/// On Windows `ClientStream` and `ServerStream` differ
pub(crate) type ServerStream = UnixStream;

/// Connect to the IPC service
#[allow(clippy::unused_async)]
#[allow(clippy::wildcard_enum_match_arm)]
pub async fn connect_to_service(id: ServiceId) -> Result<ClientStream, Error> {
    let path = ipc_path(id);
    let stream = UnixStream::connect(&path)
        .await
        .map_err(|error| match error.kind() {
            ErrorKind::NotFound => Error::NotFound(path.display().to_string()),
            ErrorKind::PermissionDenied => Error::PermissionDenied,
            _ => Error::Other(error.into()),
        })?;
    let cred = stream
        .peer_cred()
        .context("Couldn't get PID of UDS server")
        .map_err(Error::Other)?;
    tracing::info!(
        uid = cred.uid(),
        gid = cred.gid(),
        pid = cred.pid(),
        "Made an IPC connection"
    );
    Ok(stream)
}

impl Server {
    /// Platform-specific setup
    pub(crate) async fn new(id: ServiceId) -> Result<Self> {
        let sock_path = ipc_path(id);
        // Remove the socket if a previous run left it there
        tokio::fs::remove_file(&sock_path).await.ok();
        // Create the dir if possible, needed for test paths under `/run/user`
        let dir = sock_path
            .parent()
            .context("`sock_path` should always have a parent")?;
        tokio::fs::create_dir_all(dir).await?;
        let listener = UnixListener::bind(&sock_path)
            .with_context(|| format!("Couldn't bind UDS `{}`", sock_path.display()))?;
        let perms = std::fs::Permissions::from_mode(0o660);
        tokio::fs::set_permissions(&sock_path, perms).await?;
        sd_notify::notify(true, &[sd_notify::NotifyState::Ready])?;
        Ok(Self { listener })
    }

    pub(crate) async fn next_client(&mut self) -> Result<ServerStream> {
        tracing::info!("Listening for GUI to connect over IPC...");
        let (stream, _) = self.listener.accept().await?;
        let cred = stream.peer_cred()?;
        tracing::info!(
            uid = cred.uid(),
            gid = cred.gid(),
            pid = cred.pid(),
            "Accepted an IPC connection"
        );
        Ok(stream)
    }
}

/// The path for our Unix Domain Socket
///
/// Docker keeps theirs in `/run` and also appears to use filesystem permissions
/// for security, so we're following their lead. `/run` and `/var/run` are symlinked
/// on some systems, `/run` should be the newer version.
///
/// Also systemd can create this dir with the `RuntimeDir=` directive which is nice.
///
/// Test sockets live in e.g. `/run/user/1000/dev.firezone.client/data/`
fn ipc_path(id: ServiceId) -> PathBuf {
    match id {
        ServiceId::Prod => PathBuf::from("/run")
            .join(connlib_shared::BUNDLE_ID)
            .join("ipc.sock"),
        ServiceId::Test(id) => crate::known_dirs::runtime()
            .expect("`known_dirs::runtime()` should always work")
            .join(format!("ipc_test_{id}.sock")),
    }
}
