//! The Tauri-based GUI Client for Windows and Linux
//!
//! Most of this Client is stubbed out with panics on macOS.
//! The real macOS Client is in `swift/apple`

use crate::client::{
    self, about, deep_link,
    ipc::{self, CallbackHandler},
    logging, network_changes,
    settings::{self, AdvancedSettings},
    Failure,
};
use anyhow::{anyhow, bail, Context, Result};
use secrecy::{ExposeSecret, SecretString};
use std::{path::PathBuf, str::FromStr, sync::Arc, time::Duration};
use system_tray_menu::Event as TrayMenuEvent;
use tauri::{Manager, SystemTray, SystemTrayEvent};
use tokio::sync::{mpsc, oneshot, Notify};
use tracing::instrument;
use url::Url;

use ControllerRequest as Req;

mod errors;
mod ran_before;
pub(crate) mod system_tray_menu;

#[cfg(target_os = "linux")]
#[path = "gui/os_linux.rs"]
#[allow(clippy::unnecessary_wraps)]
mod os;

// Stub only
#[cfg(target_os = "macos")]
#[path = "gui/os_macos.rs"]
#[allow(clippy::unnecessary_wraps)]
mod os;

#[cfg(target_os = "windows")]
#[path = "gui/os_windows.rs"]
#[allow(clippy::unnecessary_wraps)]
mod os;

pub(crate) use errors::{show_error_dialog, Error};
pub(crate) use os::set_autostart;

pub(crate) type CtlrTx = mpsc::Sender<ControllerRequest>;

const TRAY_ICON_TOOLTIP: &str = "Firezone";

/// All managed state that we might need to access from odd places like Tauri commands.
///
/// Note that this never gets Dropped because of
/// <https://github.com/tauri-apps/tauri/issues/8631>
pub(crate) struct Managed {
    pub ctlr_tx: CtlrTx,
    pub inject_faults: bool,
}

/// Runs the Tauri GUI and returns on exit or unrecoverable error
///
/// Still uses `thiserror` so we can catch the deep_link `CantListen` error
#[instrument(skip_all)]
pub(crate) fn run(
    cli: client::Cli,
    advanced_settings: settings::AdvancedSettings,
    reloader: logging::Reloader,
) -> Result<(), Error> {
    // Need to keep this alive so crashes will be handled. Dropping detaches it.
    let _crash_handler = match client::crash_handling::attach_handler() {
        Ok(x) => Some(x),
        Err(error) => {
            // TODO: None of these logs are actually written yet
            // <https://github.com/firezone/firezone/issues/3211>
            tracing::warn!(?error, "Did not set up crash handler");
            None
        }
    };

    // Needed for the deep link server
    let rt = tokio::runtime::Runtime::new().context("Couldn't start Tokio runtime")?;
    let _guard = rt.enter();
    rt.spawn(firezone_headless_client::heartbeat::heartbeat());

    let (ctlr_tx, ctlr_rx) = mpsc::channel(5);

    let managed = Managed {
        ctlr_tx: ctlr_tx.clone(),
        inject_faults: cli.inject_faults,
    };

    // We can't call `refresh_system_tray_menu` yet because `Controller`
    // is built inside Tauri's setup
    let tray = SystemTray::new()
        .with_menu(system_tray_menu::loading())
        .with_tooltip(TRAY_ICON_TOOLTIP);

    tracing::info!("Setting up Tauri app instance...");
    let (setup_result_tx, mut setup_result_rx) =
        tokio::sync::oneshot::channel::<Result<(), Error>>();
    let app = tauri::Builder::default()
        .manage(managed)
        .on_window_event(|event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event.event() {
                // Keep the frontend running but just hide this webview
                // Per https://tauri.app/v1/guides/features/system-tray/#preventing-the-app-from-closing
                // Closing the window fully seems to deallocate it or something.

                event.window().hide().unwrap();
                api.prevent_close();
            }
        })
        .invoke_handler(tauri::generate_handler![
            about::get_cargo_version,
            about::get_git_version,
            logging::clear_logs,
            logging::count_logs,
            logging::export_logs,
            settings::apply_advanced_settings,
            settings::reset_advanced_settings,
            settings::get_advanced_settings,
            crate::client::welcome::sign_in,
        ])
        .system_tray(tray)
        .on_system_tray_event(|app, event| {
            if let SystemTrayEvent::MenuItemClick { id, .. } = event {
                tracing::debug!(?id, "SystemTrayEvent::MenuItemClick");
                let event = match serde_json::from_str::<TrayMenuEvent>(&id) {
                    Ok(x) => x,
                    Err(e) => {
                        tracing::error!("{e}");
                        return;
                    }
                };
                match handle_system_tray_event(app, event) {
                    Ok(_) => {}
                    Err(e) => tracing::error!("{e}"),
                }
            }
        })
        .setup(move |app| {
            tracing::info!("Entered Tauri's `setup`");

            let setup_inner = move || {
                // Check for updates
                let ctlr_tx_clone = ctlr_tx.clone();
                let always_show_update_notification = cli.always_show_update_notification;
                tokio::spawn(async move {
                    if let Err(error) = check_for_updates(ctlr_tx_clone, always_show_update_notification).await
                    {
                        tracing::error!(?error, "Error in check_for_updates");
                    }
                });

                // Make sure we're single-instance
                // We register our deep links to call the `open-deep-link` subcommand,
                // so if we're at this point, we know we've been launched manually
                let server = deep_link::Server::new()?;

                if let Some(client::Cmd::SmokeTest) = &cli.command {
                    let ctlr_tx = ctlr_tx.clone();
                    tokio::spawn(async move {
                        if let Err(error) = smoke_test(ctlr_tx).await {
                            tracing::error!(?error, "Error during smoke test, crashing on purpose so a dev can see our stacktraces");
                            unsafe { sadness_generator::raise_segfault() }
                        }
                    });
                }

                tracing::debug!(cli.no_deep_links);
                if !cli.no_deep_links {
                    // The single-instance check is done, so register our exe
                    // to handle deep links
                    deep_link::register().context("Failed to register deep link handler")?;
                    tokio::spawn(accept_deep_links(server, ctlr_tx.clone()));
                }

                if let Some(failure) = cli.fail_on_purpose() {
                    let ctlr_tx = ctlr_tx.clone();
                    tokio::spawn(async move {
                        let delay = 5;
                        tracing::info!(
                            "Will crash / error / panic on purpose in {delay} seconds to test error handling."
                        );
                        tokio::time::sleep(Duration::from_secs(delay)).await;
                        tracing::info!("Crashing / erroring / panicking on purpose");
                        ctlr_tx.send(ControllerRequest::Fail(failure)).await?;
                        Ok::<_, anyhow::Error>(())
                    });
                }

                assert_eq!(
                    connlib_shared::BUNDLE_ID,
                    app.handle().config().tauri.bundle.identifier,
                    "BUNDLE_ID should match bundle ID in tauri.conf.json"
                );

                let app_handle = app.handle();
                let _ctlr_task = tokio::spawn(async move {
                    let app_handle_2 = app_handle.clone();
                    // Spawn two nested Tasks so the outer can catch panics from the inner
                    let task = tokio::spawn(async move {
                        run_controller(
                            app_handle_2,
                            ctlr_tx,
                            ctlr_rx,
                            advanced_settings,
                            reloader,
                        )
                        .await
                    });

                    // See <https://github.com/tauri-apps/tauri/issues/8631>
                    // This should be the ONLY place we call `app.exit` or `app_handle.exit`,
                    // because it exits the entire process without dropping anything.
                    //
                    // This seems to be a platform limitation that Tauri is unable to hide
                    // from us. It was the source of much consternation at time of writing.

                    let exit_code = match task.await {
                        Err(error) => {
                            tracing::error!(?error, "run_controller panicked");
                            1
                        }
                        Ok(Err(error)) => {
                            tracing::error!(?error, "run_controller returned an error");
                            errors::show_error_dialog(&error).unwrap();
                            1
                        }
                        Ok(Ok(_)) => 0,
                    };

                    tracing::info!(?exit_code);
                    app_handle.exit(exit_code);
                });
                Ok(())
            };

            setup_result_tx.send(setup_inner()).expect("should be able to send setup result");

            Ok(())
        });
    tracing::debug!("Building Tauri app...");
    let app = app.build(tauri::generate_context!());

    setup_result_rx
        .try_recv()
        .context("couldn't receive result of setup")??;

    let app = match app {
        Ok(x) => x,
        Err(error) => {
            tracing::error!(?error, "Failed to build Tauri app instance");
            #[allow(clippy::wildcard_enum_match_arm)]
            match error {
                tauri::Error::Runtime(tauri_runtime::Error::CreateWebview(_)) => {
                    return Err(Error::WebViewNotInstalled);
                }
                error => Err(anyhow::Error::from(error).context("Tauri error"))?,
            }
        }
    };

    app.run(|_app_handle, event| {
        if let tauri::RunEvent::ExitRequested { api, .. } = event {
            // Don't exit if we close our main window
            // https://tauri.app/v1/guides/features/system-tray/#preventing-the-app-from-closing

            api.prevent_exit();
        }
    });
    Ok(())
}

/// Runs a smoke test and then asks Controller to exit gracefully
///
/// You can purposely fail this test by deleting the exported zip file during
/// the 10-second sleep.
async fn smoke_test(ctlr_tx: CtlrTx) -> Result<()> {
    let delay = 10;
    tracing::info!("Will quit on purpose in {delay} seconds as part of the smoke test.");
    let quit_time = tokio::time::Instant::now() + Duration::from_secs(delay);

    // Test log exporting
    let path = PathBuf::from("smoke_test_log_export.zip");

    let stem = "connlib-smoke-test".into();
    match tokio::fs::remove_file(&path).await {
        Ok(()) => {}
        Err(error) => {
            if error.kind() != std::io::ErrorKind::NotFound {
                bail!("Error while removing old zip file")
            }
        }
    }
    ctlr_tx
        .send(ControllerRequest::ExportLogs {
            path: path.clone(),
            stem,
        })
        .await
        .context("Failed to send ExportLogs request")?;
    ctlr_tx
        .send(ControllerRequest::ClearLogs)
        .await
        .context("Failed to send ClearLogs request")?;

    // Give the app some time to export the zip and reach steady state
    tokio::time::sleep_until(quit_time).await;

    // Write the settings so we can check the path for those
    settings::save(&settings::AdvancedSettings::default()).await?;

    // Check results of tests
    let zip_len = tokio::fs::metadata(&path)
        .await
        .context("Failed to get zip file metadata")?
        .len();
    if zip_len <= 22 {
        bail!("Exported log zip just has the file header");
    }
    tokio::fs::remove_file(&path)
        .await
        .context("Failed to remove zip file")?;
    tracing::info!(?path, ?zip_len, "Exported log zip looks okay");

    tracing::info!("Quitting on purpose because of `smoke-test` subcommand");
    ctlr_tx
        .send(ControllerRequest::SystemTrayMenu(TrayMenuEvent::Quit))
        .await
        .context("Failed to send Quit request")?;

    Ok::<_, anyhow::Error>(())
}

async fn check_for_updates(ctlr_tx: CtlrTx, always_show_update_notification: bool) -> Result<()> {
    let release = client::updates::check()
        .await
        .context("Error in client::updates::check")?;
    let latest_version = release.version.clone();

    let our_version = client::updates::current_version()?;

    if always_show_update_notification || (our_version < latest_version) {
        tracing::info!(?our_version, ?latest_version, "There is a new release");
        // We don't necessarily need to route through the Controller here, but if we
        // want a persistent "Click here to download the new MSI" button, this would allow that.
        ctlr_tx
            .send(ControllerRequest::UpdateAvailable(release))
            .await
            .context("Error while sending UpdateAvailable to Controller")?;
        return Ok(());
    }

    tracing::info!(
        ?our_version,
        ?latest_version,
        "Our release is newer than, or the same as, the latest"
    );
    Ok(())
}

/// Worker task to accept deep links from a named pipe forever
///
/// * `server` An initial named pipe server to consume before making new servers. This lets us also use the named pipe to enforce single-instance
async fn accept_deep_links(mut server: deep_link::Server, ctlr_tx: CtlrTx) -> Result<()> {
    loop {
        match server.accept().await {
            Ok(bytes) => {
                let url = SecretString::from_str(
                    std::str::from_utf8(bytes.expose_secret())
                        .context("Incoming deep link was not valid UTF-8")?,
                )
                .context("Impossible: can't wrap String into SecretString")?;
                // Ignore errors from this, it would only happen if the app is shutting down, otherwise we would wait
                ctlr_tx
                    .send(ControllerRequest::SchemeRequest(url))
                    .await
                    .ok();
            }
            Err(error) => tracing::error!(?error, "error while accepting deep link"),
        }
        // We re-create the named pipe server every time we get a link, because of an oddity in the Windows API.
        server = deep_link::Server::new()?;
    }
}

fn handle_system_tray_event(app: &tauri::AppHandle, event: TrayMenuEvent) -> Result<()> {
    app.try_state::<Managed>()
        .context("can't get Managed struct from Tauri")?
        .ctlr_tx
        .blocking_send(ControllerRequest::SystemTrayMenu(event))?;
    Ok(())
}

// Allow dead code because `UpdateNotificationClicked` doesn't work on Linux yet
#[allow(dead_code)]
pub(crate) enum ControllerRequest {
    /// The GUI wants us to use these settings in-memory, they've already been saved to disk
    ApplySettings(AdvancedSettings),
    /// Only used for smoke tests
    ClearLogs,
    Disconnected {
        error_msg: String,
        is_authentication_error: bool,
    },
    /// The same as the arguments to `client::logging::export_logs_to`
    ExportLogs {
        path: PathBuf,
        stem: PathBuf,
    },
    Fail(Failure),
    GetAdvancedSettings(oneshot::Sender<AdvancedSettings>),
    SchemeRequest(SecretString),
    SignIn,
    SystemTrayMenu(TrayMenuEvent),
    TunnelReady,
    UpdateAvailable(crate::client::updates::Release),
    UpdateNotificationClicked(Url),
}

struct Controller {
    /// Debugging-only settings like API URL, auth URL, log filter
    advanced_settings: AdvancedSettings,
    app: tauri::AppHandle,
    // Sign-in state with the portal / deep links
    auth: client::auth::Auth,
    ctlr_tx: CtlrTx,
    /// connlib session for the currently signed-in user, if there is one
    session: Option<Session>,
    log_filter_reloader: logging::Reloader,
    /// Tells us when to wake up and look for a new resource list. Tokio docs say that memory reads and writes are synchronized when notifying, so we don't need an extra mutex on the resources.
    notify_controller: Arc<Notify>,
    tunnel_ready: bool,
    uptime: client::uptime::Tracker,
}

/// Everything related to a signed-in user session
struct Session {
    callback_handler: CallbackHandler,
    connlib: ipc::Client,
}

impl Controller {
    /// Pre-req: the auth module must be signed in
    async fn start_session(&mut self, token: SecretString) -> Result<(), Error> {
        if self.session.is_some() {
            Err(anyhow!("can't start session, we're already in a session"))?;
        }

        let callback_handler = CallbackHandler {
            ctlr_tx: self.ctlr_tx.clone(),
            notify_controller: Arc::clone(&self.notify_controller),
            resources: Default::default(),
        };

        let api_url = self.advanced_settings.api_url.clone();
        tracing::info!(api_url = api_url.to_string(), "Starting connlib...");

        let connlib = ipc::Client::connect(
            api_url.as_str(),
            token,
            callback_handler.clone(),
            tokio::runtime::Handle::current(),
        )
        .await?;

        self.session = Some(Session {
            callback_handler,
            connlib,
        });
        self.refresh_system_tray_menu()?;

        ran_before::set().await?;
        Ok(())
    }

    async fn handle_deep_link(&mut self, url: &SecretString) -> Result<(), Error> {
        let auth_response =
            client::deep_link::parse_auth_callback(url).context("Couldn't parse scheme request")?;

        tracing::info!("Received deep link over IPC");
        // Uses `std::fs`
        let token = self
            .auth
            .handle_response(auth_response)
            .context("Couldn't handle auth response")?;
        self.start_session(token).await?;
        Ok(())
    }

    async fn handle_request(&mut self, req: ControllerRequest) -> Result<(), Error> {
        match req {
            Req::ApplySettings(settings) => {
                let filter =
                    tracing_subscriber::EnvFilter::try_new(&self.advanced_settings.log_filter)
                        .context("Couldn't parse new log filter directives")?;
                self.advanced_settings = settings;
                self.log_filter_reloader
                    .reload(filter)
                    .context("Couldn't reload log filter")?;
                tracing::debug!(
                    "Applied new settings. Log level will take effect immediately for the GUI and later for the IPC service."
                );
            }
            Req::ClearLogs => logging::clear_logs_inner()
                .await
                .context("Failed to clear logs")?,
            Req::Disconnected {
                error_msg,
                is_authentication_error,
            } => {
                self.sign_out().await?;
                if is_authentication_error {
                    tracing::info!(?error_msg, "Auth error");
                    os::show_notification(
                        "Firezone disconnected",
                        "To access resources, sign in again.",
                    )?;
                } else {
                    tracing::error!(?error_msg, "Disconnected");
                    native_dialog::MessageDialog::new()
                        .set_title("Firezone Error")
                        .set_text(&error_msg)
                        .set_type(native_dialog::MessageType::Error)
                        .show_alert()
                        .context("Couldn't show Disconnected alert")?;
                }
            }
            Req::ExportLogs { path, stem } => logging::export_logs_to(path, stem)
                .await
                .context("Failed to export logs to zip")?,
            Req::Fail(_) => Err(anyhow!(
                "Impossible error: `Fail` should be handled before this"
            ))?,
            Req::GetAdvancedSettings(tx) => {
                tx.send(self.advanced_settings.clone()).ok();
            }
            Req::SchemeRequest(url) => self.handle_deep_link(&url).await?,
            Req::SignIn | Req::SystemTrayMenu(TrayMenuEvent::SignIn) => {
                if let Some(req) = self
                    .auth
                    .start_sign_in()
                    .context("Couldn't start sign-in flow")?
                {
                    let url = req.to_url(&self.advanced_settings.auth_base_url);
                    self.refresh_system_tray_menu()?;
                    tauri::api::shell::open(&self.app.shell_scope(), url.expose_secret(), None)
                        .context("Couldn't open auth page")?;
                    self.app
                        .get_window("welcome")
                        .context("Couldn't get handle to Welcome window")?
                        .hide()
                        .context("Couldn't hide Welcome window")?;
                }
            }
            Req::SystemTrayMenu(TrayMenuEvent::AdminPortal) => tauri::api::shell::open(
                &self.app.shell_scope(),
                &self.advanced_settings.auth_base_url,
                None,
            )
            .context("Couldn't open auth page")?,
            Req::SystemTrayMenu(TrayMenuEvent::Copy(s)) => arboard::Clipboard::new()
                .context("Couldn't access clipboard")?
                .set_text(s)
                .context("Couldn't copy resource URL or other text to clipboard")?,
            Req::SystemTrayMenu(TrayMenuEvent::CancelSignIn) => {
                if self.session.is_some() {
                    if self.tunnel_ready {
                        tracing::error!("Can't cancel sign-in, the tunnel is already up. This is a logic error in the code.");
                    } else {
                        tracing::warn!(
                            "Connlib is already raising the tunnel, calling `sign_out` anyway"
                        );
                        self.sign_out().await?;
                    }
                } else {
                    tracing::info!("Calling `sign_out` to cancel sign-in");
                    self.sign_out().await?;
                }
            }
            Req::SystemTrayMenu(TrayMenuEvent::ShowWindow(window)) => {
                self.show_window(window)?;
                // When the About or Settings windows are hidden / shown, log the
                // run ID and uptime. This makes it easy to check client stability on
                // dev or test systems without parsing the whole log file.
                let uptime_info = self.uptime.info();
                tracing::debug!(
                    uptime_s = uptime_info.uptime.as_secs(),
                    run_id = uptime_info.run_id.to_string(),
                    "Uptime info"
                );
            }
            Req::SystemTrayMenu(TrayMenuEvent::SignOut) => {
                tracing::info!("User asked to sign out");
                self.sign_out().await?;
            }
            Req::SystemTrayMenu(TrayMenuEvent::Url(url)) => {
                tauri::api::shell::open(&self.app.shell_scope(), url, None)
                    .context("Couldn't open URL from system tray")?
            }
            Req::SystemTrayMenu(TrayMenuEvent::Quit) => Err(anyhow!(
                "Impossible error: `Quit` should be handled before this"
            ))?,
            Req::TunnelReady => {
                if !self.tunnel_ready {
                    os::show_notification(
                        "Firezone connected",
                        "You are now signed in and able to access resources.",
                    )?;
                }
                self.tunnel_ready = true;
                self.refresh_system_tray_menu()?;
            }
            Req::UpdateAvailable(release) => {
                let title = format!("Firezone {} available for download", release.version);

                // We don't need to route through the controller here either, we could
                // use the `open` crate directly instead of Tauri's wrapper
                // `tauri::api::shell::open`
                os::show_update_notification(self.ctlr_tx.clone(), &title, release.download_url)?;
            }
            Req::UpdateNotificationClicked(download_url) => {
                tracing::info!("UpdateNotificationClicked in run_controller!");
                tauri::api::shell::open(&self.app.shell_scope(), download_url, None)
                    .context("Couldn't open update page")?;
            }
        }
        Ok(())
    }

    /// Returns a new system tray menu
    fn build_system_tray_menu(&self) -> tauri::SystemTrayMenu {
        // TODO: Refactor this and the auth module so that "Are we logged in"
        // doesn't require such complicated control flow to answer.
        // TODO: Show some "Waiting for portal..." state if we got the deep link but
        // haven't got `on_tunnel_ready` yet.
        if let Some(auth_session) = self.auth.session() {
            if let Some(connlib_session) = &self.session {
                if self.tunnel_ready {
                    // Signed in, tunnel ready
                    let resources = connlib_session.callback_handler.resources.load();
                    system_tray_menu::signed_in(&auth_session.actor_name, &resources)
                } else {
                    // Signed in, raising tunnel
                    system_tray_menu::signing_in("Signing In...")
                }
            } else {
                tracing::error!("We have an auth session but no connlib session");
                system_tray_menu::signed_out()
            }
        } else if self.auth.ongoing_request().is_ok() {
            // Signing in, waiting on deep link callback
            system_tray_menu::signing_in("Waiting for browser...")
        } else {
            system_tray_menu::signed_out()
        }
    }

    /// Builds a new system tray menu and applies it to the app
    fn refresh_system_tray_menu(&self) -> Result<()> {
        let tray = self.app.tray_handle();
        tray.set_tooltip(TRAY_ICON_TOOLTIP)?;
        tray.set_menu(self.build_system_tray_menu())?;
        Ok(())
    }

    /// Deletes the auth token, stops connlib, and refreshes the tray menu
    async fn sign_out(&mut self) -> Result<()> {
        self.auth.sign_out()?;
        self.tunnel_ready = false;
        if let Some(session) = self.session.take() {
            tracing::debug!("disconnecting connlib");
            // This is redundant if the token is expired, in that case
            // connlib already disconnected itself.
            session.connlib.disconnect().await?;
        } else {
            // Might just be because we got a double sign-out or
            // the user canceled the sign-in or something innocent.
            tracing::info!("Tried to sign out but there's no session, cancelled sign-in");
        }
        self.refresh_system_tray_menu()?;
        Ok(())
    }

    fn show_window(&self, window: system_tray_menu::Window) -> Result<()> {
        let id = match window {
            system_tray_menu::Window::About => "about",
            system_tray_menu::Window::Settings => "settings",
        };

        let win = self
            .app
            .get_window(id)
            .context("Couldn't get handle to `{id}` window")?;

        win.show()?;
        win.unminimize()?;
        Ok(())
    }
}

// TODO: Move this into `impl Controller`
async fn run_controller(
    app: tauri::AppHandle,
    ctlr_tx: CtlrTx,
    mut rx: mpsc::Receiver<ControllerRequest>,
    advanced_settings: AdvancedSettings,
    log_filter_reloader: logging::Reloader,
) -> Result<(), Error> {
    tracing::info!("Entered `run_controller`");
    let mut controller = Controller {
        advanced_settings,
        app: app.clone(),
        auth: client::auth::Auth::new(),
        ctlr_tx,
        session: None,
        log_filter_reloader,
        notify_controller: Arc::new(Notify::new()), // TODO: Fix cancel-safety
        tunnel_ready: false,
        uptime: Default::default(),
    };

    if let Some(token) = controller
        .auth
        .token()
        .context("Failed to load token from disk during app start")?
    {
        controller.start_session(token).await?;
    } else {
        tracing::info!("No token / actor_name on disk, starting in signed-out state");
        controller.refresh_system_tray_menu()?;
    }

    if !ran_before::get().await? {
        let win = app
            .get_window("welcome")
            .context("Couldn't get handle to Welcome window")?;
        win.show().context("Couldn't show Welcome window")?;
    }

    let mut have_internet =
        network_changes::check_internet().context("Failed initial check for internet")?;
    tracing::info!(?have_internet);

    let mut com_worker =
        network_changes::Worker::new().context("Failed to listen for network changes")?;

    let mut dns_listener = network_changes::DnsListener::new()?;

    loop {
        tokio::select! {
            () = controller.notify_controller.notified() => {
                tracing::debug!("Controller notified of new resources");
                if let Err(error) = controller.refresh_system_tray_menu() {
                    tracing::error!(?error, "Failed to reload resource list");
                }
            }
            () = com_worker.notified() => {
                let new_have_internet = network_changes::check_internet().context("Failed to check for internet")?;
                if new_have_internet != have_internet {
                    have_internet = new_have_internet;
                    if let Some(session) = controller.session.as_mut() {
                        tracing::debug!("Internet up/down changed, calling `Session::reconnect`");
                        session.connlib.reconnect().await?;
                    }
                }
            },
            resolvers = dns_listener.notified() => {
                let resolvers = resolvers?;
                if let Some(session) = controller.session.as_mut() {
                    tracing::debug!(?resolvers, "New DNS resolvers, calling `Session::set_dns`");
                    session.connlib.set_dns(resolvers).await?;
                }
            },
            req = rx.recv() => {
                let Some(req) = req else {
                    break;
                };

                #[allow(clippy::wildcard_enum_match_arm)]
                match req {
                    // SAFETY: Crashing is unsafe
                    Req::Fail(Failure::Crash) => {
                        tracing::error!("Crashing on purpose");
                        unsafe { sadness_generator::raise_segfault() }
                    },
                    Req::Fail(Failure::Error) => Err(anyhow!("Test error"))?,
                    Req::Fail(Failure::Panic) => panic!("Test panic"),
                    Req::SystemTrayMenu(TrayMenuEvent::Quit) => {
                        tracing::info!("User clicked Quit in the menu");
                        break
                    }
                    req => controller.handle_request(req).await?,
                }
            },
        }
    }

    if let Err(error) = com_worker.close() {
        tracing::error!(?error, "com_worker");
    }

    // Last chance to do any drops / cleanup before the process crashes.

    Ok(())
}
