//! Main connlib library for clients.
pub use connlib_shared::messages::ResourceDescription;
pub use connlib_shared::{
    keypair, Callbacks, Cidrv4, Cidrv6, Error, LoginUrl, LoginUrlError, StaticSecret,
};
pub use tracing_appender::non_blocking::WorkerGuard;

use backoff::ExponentialBackoffBuilder;
use connlib_shared::{get_user_agent, CallbackErrorFacade};
use firezone_tunnel::ClientTunnel;
use phoenix_channel::PhoenixChannel;
use std::time::Duration;

mod eventloop;
pub mod file_logger;
mod messages;

const PHOENIX_TOPIC: &str = "client";

use eventloop::Command;
pub use eventloop::Eventloop;
use secrecy::Secret;
use tokio::task::JoinHandle;

/// A session is the entry-point for connlib, maintains the runtime and the tunnel.
///
/// A session is created using [Session::connect], then to stop a session we use [Session::disconnect].
pub struct Session {
    channel: tokio::sync::mpsc::Sender<Command>,
}

impl Session {
    /// Creates a new [`Session`].
    ///
    /// This connects to the portal a specified using [`LoginUrl`] and creates a wireguard tunnel using the provided private key.
    pub fn connect<CB: Callbacks + 'static>(
        url: LoginUrl,
        private_key: StaticSecret,
        os_version_override: Option<String>,
        callbacks: CB,
        max_partition_time: Option<Duration>,
        handle: tokio::runtime::Handle,
    ) -> connlib_shared::Result<Self> {
        let callbacks = CallbackErrorFacade(callbacks);
        let (tx, rx) = tokio::sync::mpsc::channel(1);

        let connect_handle = handle.spawn(connect(
            url,
            private_key,
            os_version_override,
            callbacks.clone(),
            max_partition_time,
            rx,
        ));
        handle.spawn(connect_supervisor(connect_handle, callbacks));

        Ok(Self { channel: tx })
    }

    /// Attempts to reconnect a [`Session`].
    ///
    /// This can and should be called by client applications on any network state changes.
    /// It is a signal to connlib to:
    ///
    /// - validate all currently used network paths to relays and peers
    /// - ensure we are connected to the portal
    ///
    /// Reconnect is non-destructive and can be called several times in a row.
    ///
    /// In case of destructive network state changes, i.e. the user switched from wifi to cellular,
    /// reconnect allows connlib to re-establish connections faster because we don't have to wait for timeouts first.
    pub fn reconnect(&mut self) {
        let _ = self.channel.try_send(Command::Reconnect);
    }

    /// Disconnect a [`Session`].
    ///
    /// This consumes [`Session`] which cleans up all state associated with it.
    pub fn disconnect(self) {
        let _ = self.channel.try_send(Command::Stop);
    }
}

/// Connects to the portal and starts a tunnel.
///
/// When this function exits, the tunnel failed unrecoverably and you need to call it again.
async fn connect<CB>(
    url: LoginUrl,
    private_key: StaticSecret,
    os_version_override: Option<String>,
    callbacks: CB,
    max_partition_time: Option<Duration>,
    rx: tokio::sync::mpsc::Receiver<Command>,
) -> Result<(), Error>
where
    CB: Callbacks + 'static,
{
    let tunnel = ClientTunnel::new(private_key, callbacks.clone())?;

    let portal = PhoenixChannel::connect(
        Secret::new(url),
        get_user_agent(os_version_override),
        PHOENIX_TOPIC,
        (),
        ExponentialBackoffBuilder::default()
            .with_max_elapsed_time(max_partition_time)
            .build(),
    );

    let mut eventloop = Eventloop::new(tunnel, portal, rx);

    std::future::poll_fn(|cx| eventloop.poll(cx))
        .await
        .map_err(Error::PortalConnectionFailed)?;

    Ok(())
}

/// A supervisor task that handles, when [`connect`] exits.
async fn connect_supervisor<CB>(connect_handle: JoinHandle<Result<(), Error>>, callbacks: CB)
where
    CB: Callbacks,
{
    match connect_handle.await {
        Ok(Ok(())) => {
            tracing::info!("connlib exited gracefully");
        }
        Ok(Err(e)) => {
            tracing::error!("connlib failed: {e}");
            callbacks.on_disconnect(&e);
        }
        Err(e) => match e.try_into_panic() {
            Ok(panic) => {
                if let Some(msg) = panic.downcast_ref::<&str>() {
                    callbacks.on_disconnect(&Error::Panic(msg.to_string()));
                    return;
                }

                callbacks.on_disconnect(&Error::PanicNonStringPayload);
            }
            Err(_) => {
                tracing::error!("connlib task was cancelled");
                callbacks.on_disconnect(&Error::Cancelled);
            }
        },
    }
}
