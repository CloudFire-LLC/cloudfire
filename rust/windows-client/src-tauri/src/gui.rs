//! The Tauri GUI for Windows

// TODO: `git grep` for unwraps before 1.0, especially this gui module

use crate::settings::{self, AdvancedSettings};
use anyhow::{anyhow, bail, Result};
use connlib_client_shared::file_logger;
use firezone_cli_utils::setup_global_subscriber;
use secrecy::SecretString;
use std::{path::PathBuf, str::FromStr};
use tauri::{
    CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem,
    SystemTraySubmenu,
};
use tokio::sync::{mpsc, oneshot};
use url::Url;
use ControllerRequest as Req;

pub(crate) type CtlrTx = mpsc::Sender<ControllerRequest>;

/// All managed state that we might need to access from odd places like Tauri commands.
pub(crate) struct Managed {
    pub ctlr_tx: CtlrTx,
    pub inject_faults: bool,
}

/// Runs the Tauri GUI and returns on exit or unrecoverable error
pub(crate) fn run(params: crate::GuiParams) -> Result<()> {
    let crate::GuiParams {
        deep_link,
        inject_faults,
    } = params;

    // Make sure we're single-instance
    tauri_plugin_deep_link::prepare("dev.firezone");

    let rt = tokio::runtime::Runtime::new()?;
    let _guard = rt.enter();

    let (ctlr_tx, ctlr_rx) = mpsc::channel(5);
    let managed = Managed {
        ctlr_tx,
        inject_faults,
    };

    let tray = SystemTray::new().with_menu(signed_out_menu());

    tauri::Builder::default()
        .manage(managed)
        .on_window_event(|event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event.event() {
                // Keep the frontend running but just hide this webview
                // Per https://tauri.app/v1/guides/features/system-tray/#preventing-the-app-from-closing

                event.window().hide().unwrap();
                api.prevent_close();
            }
        })
        .invoke_handler(tauri::generate_handler![
            settings::apply_advanced_settings,
            settings::clear_logs,
            settings::export_logs,
            settings::get_advanced_settings,
        ])
        .system_tray(tray)
        .on_system_tray_event(|app, event| {
            if let SystemTrayEvent::MenuItemClick { id, .. } = event {
                let event = match TrayMenuEvent::from_str(&id) {
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
        .setup(|app| {
            // Change to data dir so the file logger will write there and not in System32 if we're launching from an app link
            let cwd = app
                .path_resolver()
                .app_local_data_dir()
                .ok_or_else(|| anyhow::anyhow!("can't get app_local_data_dir"))?
                .join("data");
            std::fs::create_dir_all(&cwd)?;
            std::env::set_current_dir(&cwd)?;

            // Set up logger with connlib_client_shared
            let (layer, _handle) = file_logger::layer(std::path::Path::new("logs"));
            setup_global_subscriber(layer);

            let _ctlr_task = tokio::spawn(run_controller(app.handle(), ctlr_rx));

            if let Some(_deep_link) = deep_link {
                // TODO: Handle app links that we catch at startup here
            }

            // From https://github.com/FabianLars/tauri-plugin-deep-link/blob/main/example/main.rs
            let handle = app.handle();
            tauri_plugin_deep_link::register(crate::DEEP_LINK_SCHEME, move |url| {
                match handle_deep_link(&handle, url) {
                    Ok(()) => {}
                    Err(e) => tracing::error!("{e}"),
                }
            })?;
            Ok(())
        })
        .build(tauri::generate_context!())?
        .run(|_app_handle, event| {
            if let tauri::RunEvent::ExitRequested { api, .. } = event {
                // Don't exit if we close our main window
                // https://tauri.app/v1/guides/features/system-tray/#preventing-the-app-from-closing

                api.prevent_exit();
            }
        });
    Ok(())
}

fn handle_deep_link(app: &tauri::AppHandle, url: String) -> Result<()> {
    Ok(app
        .try_state::<Managed>()
        .ok_or_else(|| anyhow!("can't get Managed object from Tauri"))?
        .ctlr_tx
        .blocking_send(ControllerRequest::SchemeRequest(SecretString::new(url)))?)
}

#[derive(Debug, PartialEq)]
enum TrayMenuEvent {
    About,
    Resource { id: String },
    Settings,
    SignIn,
    SignOut,
    Quit,
}

impl FromStr for TrayMenuEvent {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        Ok(match s {
            "/about" => Self::About,
            "/settings" => Self::Settings,
            "/sign_in" => Self::SignIn,
            "/sign_out" => Self::SignOut,
            "/quit" => Self::Quit,
            s => {
                if let Some(id) = s.strip_prefix("/resource/") {
                    Self::Resource { id: id.to_string() }
                } else {
                    anyhow::bail!("unknown system tray menu event");
                }
            }
        })
    }
}

fn handle_system_tray_event(app: &tauri::AppHandle, event: TrayMenuEvent) -> Result<()> {
    match event {
        TrayMenuEvent::About => {
            let win = app
                .get_window("about")
                .ok_or_else(|| anyhow!("getting handle to About window"))?;

            if win.is_visible()? {
                win.hide()?;
            } else {
                win.show()?;
            }
        }
        TrayMenuEvent::Resource { id } => tracing::warn!("TODO copy {id} to clipboard"),
        TrayMenuEvent::Settings => {
            let win = app
                .get_window("settings")
                .ok_or_else(|| anyhow!("getting handle to Settings window"))?;

            if win.is_visible()? {
                // If we close the window here, we can't re-open it, we'd have to fully re-create it. Not needed for MVP - We agreed 100 MB is fine for the GUI client.
                win.hide()?;
            } else {
                win.show()?;
            }
        }
        TrayMenuEvent::SignIn => app
            .try_state::<Managed>()
            .ok_or_else(|| anyhow!("getting ctlr_tx state"))?
            .ctlr_tx
            .blocking_send(ControllerRequest::SignIn)?,
        TrayMenuEvent::SignOut => app.tray_handle().set_menu(signed_out_menu())?,
        TrayMenuEvent::Quit => app.exit(0),
    }
    Ok(())
}

pub(crate) enum ControllerRequest {
    ExportLogs(PathBuf),
    GetAdvancedSettings(oneshot::Sender<AdvancedSettings>),
    // Secret because it will have the token in it
    SchemeRequest(SecretString),
    SignIn,
    UpdateResources(Vec<connlib_client_shared::ResourceDescription>),
}

// TODO: Should these be keyed to the Google ID or email or something?
// The callback returns a human-readable name but those aren't good keys.
fn keyring_entry() -> Result<keyring::Entry> {
    Ok(keyring::Entry::new_with_target(
        "token",
        "firezone_windows_client",
        "",
    )?)
}

#[derive(Clone)]
struct CallbackHandler {
    ctlr_tx: CtlrTx,
    handle: Option<file_logger::Handle>,
}

impl connlib_client_shared::Callbacks for CallbackHandler {
    // TODO: add thiserror type
    type Error = std::convert::Infallible;

    fn on_disconnect(
        &self,
        error: Option<&connlib_client_shared::Error>,
    ) -> Result<(), Self::Error> {
        tracing::error!("on_disconnect {error:?}");
        Ok(())
    }

    fn on_error(&self, error: &connlib_client_shared::Error) -> Result<(), Self::Error> {
        tracing::error!("on_error not implemented. Error: {error:?}");
        Ok(())
    }

    fn on_update_resources(
        &self,
        resources: Vec<connlib_client_shared::ResourceDescription>,
    ) -> Result<(), Self::Error> {
        tracing::debug!("on_update_resources");
        self.ctlr_tx
            .blocking_send(ControllerRequest::UpdateResources(resources))
            .unwrap();
        Ok(())
    }

    fn roll_log_file(&self) -> Option<PathBuf> {
        self.handle
            .as_ref()?
            .roll_to_new_file()
            .unwrap_or_else(|e| {
                tracing::debug!("Failed to roll over to new file: {e}");
                let _ = self.on_error(&connlib_client_shared::Error::LogFileRollError(e));

                None
            })
    }
}

struct Controller {
    advanced_settings: AdvancedSettings,
    ctlr_tx: CtlrTx,
    session: Option<connlib_client_shared::Session<CallbackHandler>>,
    token: Option<SecretString>,
}

impl Controller {
    async fn new(app: tauri::AppHandle) -> Result<Self> {
        let ctlr_tx = app
            .try_state::<Managed>()
            .ok_or_else(|| anyhow::anyhow!("can't get Managed object from Tauri"))?
            .ctlr_tx
            .clone();
        let advanced_settings = settings::load_advanced_settings(&app).await?;

        tracing::trace!("re-loading token");
        let token: Option<SecretString> = tokio::task::spawn_blocking(|| {
            let entry = keyring_entry()?;
            match entry.get_password() {
                Ok(token) => {
                    tracing::debug!("re-loaded token from Windows credential manager");
                    Ok(Some(SecretString::new(token)))
                }
                Err(keyring::Error::NoEntry) => {
                    tracing::debug!("no token in Windows credential manager");
                    Ok(None)
                }
                Err(e) => Err(anyhow::Error::from(e)),
            }
        })
        .await??;

        let session = if let Some(token) = token.as_ref() {
            Some(Self::start_session(
                &advanced_settings,
                ctlr_tx.clone(),
                token,
            )?)
        } else {
            None
        };

        Ok(Self {
            advanced_settings,
            ctlr_tx,
            session,
            token,
        })
    }

    fn start_session(
        advanced_settings: &settings::AdvancedSettings,
        ctlr_tx: CtlrTx,
        token: &SecretString,
    ) -> Result<connlib_client_shared::Session<CallbackHandler>> {
        let (layer, handle) = file_logger::layer(std::path::Path::new("logs"));
        // TODO: How can I set up the tracing subscriber if the Session isn't ready yet? Check what other clients do.
        if false {
            // This helps the type inference
            setup_global_subscriber(layer);
        }

        tracing::info!("Session::connect");
        Ok(connlib_client_shared::Session::connect(
            advanced_settings.api_url.clone(),
            token.clone(),
            crate::device_id::get(),
            CallbackHandler {
                ctlr_tx,
                handle: Some(handle),
            },
        )?)
    }
}

async fn run_controller(
    app: tauri::AppHandle,
    mut rx: mpsc::Receiver<ControllerRequest>,
) -> Result<()> {
    let mut controller = Controller::new(app.clone()).await?;

    tracing::debug!("GUI controller main loop start");

    while let Some(req) = rx.recv().await {
        match req {
            Req::ExportLogs(file_path) => settings::export_logs_to(file_path).await?,
            Req::GetAdvancedSettings(tx) => {
                tx.send(controller.advanced_settings.clone()).ok();
            }
            Req::SchemeRequest(req) => {
                use secrecy::ExposeSecret;

                if let Ok(auth) = parse_auth_callback(&req) {
                    tracing::debug!("setting new token");
                    let entry = keyring_entry()?;
                    entry.set_password(auth.token.expose_secret())?;
                    controller.session = Some(Controller::start_session(
                        &controller.advanced_settings,
                        controller.ctlr_tx.clone(),
                        &auth.token,
                    )?);
                    controller.token = Some(auth.token);
                } else {
                    tracing::warn!("couldn't handle scheme request");
                }
            }
            Req::SignIn => {
                // TODO: Put the platform and local server callback in here
                tauri::api::shell::open(
                    &app.shell_scope(),
                    &controller.advanced_settings.auth_base_url,
                    None,
                )?;
            }
            Req::UpdateResources(resources) => {
                tracing::debug!("got {} resources", resources.len());
            }
        }
    }
    tracing::debug!("GUI controller task exiting cleanly");
    Ok(())
}

pub(crate) struct AuthCallback {
    token: SecretString,
    _identifier: SecretString,
}

fn parse_auth_callback(input: &SecretString) -> Result<AuthCallback> {
    use secrecy::ExposeSecret;

    let url = url::Url::parse(input.expose_secret())?;

    let mut token = None;
    let mut identifier = None;

    for (key, value) in url.query_pairs() {
        match key.as_ref() {
            "client_auth_token" => {
                if token.is_some() {
                    bail!("client_auth_token must appear exactly once");
                }
                token = Some(SecretString::new(value.to_string()));
            }
            "identity_provider_identifier" => {
                if identifier.is_some() {
                    bail!("identity_provider_identifier must appear exactly once");
                }
                identifier = Some(SecretString::new(value.to_string()));
            }
            _ => {}
        }
    }

    Ok(AuthCallback {
        token: token.ok_or_else(|| anyhow!("expected client_auth_token"))?,
        _identifier: identifier.ok_or_else(|| anyhow!("expected identity_provider_identifier"))?,
    })
}

/// The information needed for the GUI to display a resource inside the Firezone VPN
struct _ResourceDisplay {
    /// UUIDv4 (Fully random)
    /// This should be stable over time even if the DNS / IP / name change, so we can use it for callbacks from the tray menu
    id: String,
    /// User-friendly name, e.g. "GitLab"
    name: String,
    /// What will be copied to the clipboard to paste into a web browser
    url: Url,
}

fn _signed_in_menu(user_email: &str, resources: &[_ResourceDisplay]) -> SystemTrayMenu {
    let mut menu = SystemTrayMenu::new()
        .add_item(
            CustomMenuItem::new("".to_string(), format!("Signed in as {user_email}")).disabled(),
        )
        .add_item(CustomMenuItem::new("/sign_out".to_string(), "Sign out"))
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(CustomMenuItem::new("".to_string(), "Resources").disabled());

    for _ResourceDisplay { id, name, url } in resources {
        let submenu = SystemTrayMenu::new().add_item(CustomMenuItem::new(
            format!("/resource/{id}"),
            url.to_string(),
        ));
        menu = menu.add_submenu(SystemTraySubmenu::new(name, submenu));
    }

    menu = menu
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(CustomMenuItem::new("/about".to_string(), "About"))
        .add_item(CustomMenuItem::new("/settings".to_string(), "Settings"))
        .add_item(CustomMenuItem::new("/quit".to_string(), "Disconnect and quit Firezone").accelerator("Ctrl+Q"));

    menu
}

fn signed_out_menu() -> SystemTrayMenu {
    SystemTrayMenu::new()
        .add_item(CustomMenuItem::new("/sign_in".to_string(), "Sign In"))
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(CustomMenuItem::new("/about".to_string(), "About"))
        .add_item(CustomMenuItem::new("/settings".to_string(), "Settings"))
        .add_item(CustomMenuItem::new("/quit".to_string(), "Quit Firezone").accelerator("Ctrl+Q"))
}

#[cfg(test)]
mod tests {
    use super::TrayMenuEvent;
    use anyhow::Result;
    use secrecy::{ExposeSecret, SecretString};
    use std::str::FromStr;

    #[test]
    fn parse_auth_callback() -> Result<()> {
        let input = "firezone://handle_client_auth_callback/?actor_name=Reactor+Scram&client_auth_token=a_very_secret_string&identity_provider_identifier=12345";

        let actual = super::parse_auth_callback(&SecretString::from_str(input)?)?;

        assert_eq!(actual.token.expose_secret(), "a_very_secret_string");

        Ok(())
    }

    #[test]
    fn systray_parse() {
        assert_eq!(
            TrayMenuEvent::from_str("/about").unwrap(),
            TrayMenuEvent::About
        );
        assert_eq!(
            TrayMenuEvent::from_str("/resource/1234").unwrap(),
            TrayMenuEvent::Resource {
                id: "1234".to_string()
            }
        );
        assert_eq!(
            TrayMenuEvent::from_str("/resource/quit").unwrap(),
            TrayMenuEvent::Resource {
                id: "quit".to_string()
            }
        );
        assert_eq!(
            TrayMenuEvent::from_str("/sign_out").unwrap(),
            TrayMenuEvent::SignOut
        );
        assert_eq!(
            TrayMenuEvent::from_str("/quit").unwrap(),
            TrayMenuEvent::Quit
        );

        assert!(TrayMenuEvent::from_str("/unknown").is_err());
    }
}
