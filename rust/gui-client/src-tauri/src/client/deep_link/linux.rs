use anyhow::{bail, Context, Result};
use firezone_headless_client::known_dirs;
use secrecy::{ExposeSecret, Secret};
use std::{path::PathBuf, process::Command};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{UnixListener, UnixStream},
};

const SOCK_NAME: &str = "deep_link.sock";

pub(crate) struct Server {
    listener: UnixListener,
}

fn sock_path() -> Result<PathBuf> {
    Ok(known_dirs::runtime()
        .context("Couldn't find runtime dir")?
        .join(SOCK_NAME))
}

impl Server {
    /// Create a new deep link server to make sure we're the only instance
    ///
    /// Still uses `thiserror` so we can catch the deep_link `CantListen` error
    /// On Windows this uses async because of #5143 and #5566.
    #[allow(clippy::unused_async)]
    pub(crate) async fn new() -> Result<Self, super::Error> {
        let path = sock_path()?;
        let dir = path
            .parent()
            .context("Impossible, socket path should always have a parent")?;

        // Try to `connect` to the socket as a client.
        // If it succeeds, that means there is already a Firezone instance listening
        // as a server on that socket, and we should exit.
        // If it fails, it means nobody is listening on the socket, or the
        // socket does not exist, in which case we are the only instance
        // and should proceed.
        if std::os::unix::net::UnixStream::connect(&path).is_ok() {
            return Err(super::Error::CantListen);
        }
        std::fs::remove_file(&path).ok();
        std::fs::create_dir_all(dir).context("Can't create dir for deep link socket")?;

        // TODO: TOCTOU error here.
        // It's possible for 2 processes to see the `connect` call fail, then one
        // binds the socket, and the other deletes the socket and binds a different
        // socket at the same path, resulting in 2 instances with confusing behavior.
        // The `bind` call should probably go first, but without more testing and more
        // thought, I don't want to re-arrange it yet.

        let listener = UnixListener::bind(&path).context("Couldn't bind listener Unix socket")?;

        Ok(Self { listener })
    }

    /// Await one incoming deep link
    ///
    /// To match the Windows API, this consumes the `Server`.
    pub(crate) async fn accept(self) -> Result<Secret<Vec<u8>>> {
        tracing::debug!("deep_link::accept");
        let (mut stream, _) = self.listener.accept().await?;
        tracing::debug!("Accepted Unix domain socket connection");

        // TODO: Limit reads to 4,096 bytes. Partial reads will probably never happen
        // since it's a local socket transferring very small data.
        let mut bytes = vec![];
        stream
            .read_to_end(&mut bytes)
            .await
            .context("failed to read incoming deep link over Unix socket stream")?;
        if bytes.is_empty() {
            bail!("Got zero bytes from the deep link socket - probably a 2nd instance was blocked");
        }
        let bytes = Secret::new(bytes);
        tracing::debug!(
            len = bytes.expose_secret().len(),
            "Got data from Unix domain socket"
        );
        Ok(bytes)
    }
}

pub(crate) async fn open(url: &url::Url) -> Result<()> {
    firezone_headless_client::setup_stdout_logging()?;

    let path = sock_path()?;
    let mut stream = UnixStream::connect(&path).await?;

    stream.write_all(url.to_string().as_bytes()).await?;

    Ok(())
}

/// Register a URI scheme so that browser can deep link into our app for auth
///
/// Performs blocking I/O (Waits on `xdg-desktop-menu` subprocess)
pub(crate) fn register() -> Result<()> {
    // Write `$HOME/.local/share/applications/firezone-client.desktop`
    // According to <https://wiki.archlinux.org/title/Desktop_entries>, that's the place to put
    // per-user desktop entries.
    let dir = dirs::data_local_dir()
        .context("can't figure out where to put our desktop entry")?
        .join("applications");
    std::fs::create_dir_all(&dir)?;

    // Don't use atomic writes here - If we lose power, we'll just rewrite this file on
    // the next boot anyway.
    let path = dir.join("firezone-client.desktop");
    let exe = std::env::current_exe().context("failed to find our own exe path")?;
    let content = format!(
        "[Desktop Entry]
Version=1.0
Name=Firezone
Comment=Firezone GUI Client
Exec={} open-deep-link %U
Terminal=false
Type=Application
MimeType=x-scheme-handler/{}
Categories=Network;
",
        exe.display(),
        super::FZ_SCHEME
    );
    std::fs::write(&path, content).context("failed to write desktop entry file")?;

    // Run `xdg-desktop-menu install` with that desktop file
    let xdg_desktop_menu = "xdg-desktop-menu";
    let status = Command::new(xdg_desktop_menu)
        .arg("install")
        .arg(&path)
        .status()
        .with_context(|| format!("failed to run `{xdg_desktop_menu}`"))?;
    if !status.success() {
        bail!("{xdg_desktop_menu} returned failure exit code");
    }

    // Needed for Ubuntu 22.04, see issue #4880
    let update_desktop_database = "update-desktop-database";
    let status = Command::new(update_desktop_database)
        .arg(&dir)
        .status()
        .with_context(|| format!("failed to run `{update_desktop_database}`"))?;
    if !status.success() {
        bail!("{update_desktop_database} returned failure exit code");
    }

    Ok(())
}
