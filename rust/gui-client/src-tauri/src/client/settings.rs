//! Everything related to the Settings window, including
//! advanced settings and code for manipulating diagnostic logs.

use crate::client::gui::{self, ControllerRequest, Managed};
use anyhow::{Context, Result};
use atomicwrites::{AtomicFile, OverwriteBehavior};
use connlib_shared::messages::ResourceId;
use firezone_headless_client::known_dirs;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, io::Write, path::PathBuf, time::Duration};
use tokio::sync::oneshot;
use url::Url;

#[derive(Clone, Deserialize, Serialize)]
pub(crate) struct AdvancedSettings {
    pub auth_base_url: Url,
    pub api_url: Url,
    #[serde(default)]
    pub favorite_resources: HashSet<ResourceId>,
    #[serde(default)]
    pub internet_resource_enabled: Option<bool>,
    pub log_filter: String,
}

#[cfg(debug_assertions)]
impl Default for AdvancedSettings {
    fn default() -> Self {
        Self {
            auth_base_url: Url::parse("https://app.firez.one").unwrap(),
            api_url: Url::parse("wss://api.firez.one").unwrap(),
            favorite_resources: Default::default(),
            internet_resource_enabled: Default::default(),
            log_filter: "firezone_gui_client=debug,info".to_string(),
        }
    }
}

#[cfg(not(debug_assertions))]
impl Default for AdvancedSettings {
    fn default() -> Self {
        Self {
            auth_base_url: Url::parse("https://app.firezone.dev").unwrap(),
            api_url: Url::parse("wss://api.firezone.dev").unwrap(),
            favorite_resources: Default::default(),
            internet_resource_enabled: Default::default(),
            log_filter: "info".to_string(),
        }
    }
}

impl AdvancedSettings {
    pub fn internet_resource_enabled(&self) -> bool {
        self.internet_resource_enabled.is_some_and(|v| v)
    }
}

pub(crate) fn advanced_settings_path() -> Result<PathBuf> {
    Ok(known_dirs::settings()
        .context("`known_dirs::settings` failed")?
        .join("advanced_settings.json"))
}

/// Saves the settings to disk and then applies them in-memory (except for logging)
#[tauri::command]
pub(crate) async fn apply_advanced_settings(
    managed: tauri::State<'_, Managed>,
    settings: AdvancedSettings,
) -> Result<(), String> {
    if managed.inner().inject_faults {
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
    apply_inner(&managed.ctlr_tx, settings)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub(crate) async fn reset_advanced_settings(
    managed: tauri::State<'_, Managed>,
) -> Result<AdvancedSettings, String> {
    let settings = AdvancedSettings::default();
    if managed.inner().inject_faults {
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
    apply_inner(&managed.ctlr_tx, settings.clone())
        .await
        .map_err(|e| e.to_string())?;

    Ok(settings)
}

#[tauri::command]
pub(crate) async fn get_advanced_settings(
    managed: tauri::State<'_, Managed>,
) -> Result<AdvancedSettings, String> {
    let (tx, rx) = oneshot::channel();
    if let Err(error) = managed
        .ctlr_tx
        .send(ControllerRequest::GetAdvancedSettings(tx))
        .await
    {
        tracing::error!(
            ?error,
            "couldn't request advanced settings from controller task"
        );
    }
    rx.await.map_err(|_| {
        "Couldn't get settings from `Controller`, maybe the program is crashing".to_string()
    })
}

/// Saves the settings to disk and then tells `Controller` to apply them in-memory
pub(crate) async fn apply_inner(ctlr_tx: &gui::CtlrTx, settings: AdvancedSettings) -> Result<()> {
    save(&settings).await?;
    // TODO: Errors aren't handled here. But there isn't much that can go wrong
    // since it's just applying a new `Settings` object in memory.
    ctlr_tx
        .send(ControllerRequest::ApplySettings(settings))
        .await?;
    Ok(())
}

/// Saves the settings to disk
pub(crate) async fn save(settings: &AdvancedSettings) -> Result<()> {
    let path = advanced_settings_path()?;
    let dir = path
        .parent()
        .context("settings path should have a parent")?;
    tokio::fs::create_dir_all(dir).await?;
    tokio::fs::write(&path, serde_json::to_string(settings)?).await?;
    // Don't create the dir for the log filter file, that's the IPC service's job.
    // If it isn't there for some reason yet, just log an error and move on.
    let log_filter_path = known_dirs::ipc_log_filter().context("`ipc_log_filter` failed")?;
    let f = AtomicFile::new(&log_filter_path, OverwriteBehavior::AllowOverwrite);
    // Note: Blocking file write in async function
    if let Err(error) = f.write(|f| f.write_all(settings.log_filter.as_bytes())) {
        tracing::error!(
            ?error,
            ?log_filter_path,
            "Couldn't write log filter file for IPC service"
        );
    }
    tracing::debug!(?path, "Saved settings");
    Ok(())
}

/// Return advanced settings if they're stored on disk
///
/// Uses std::fs, so stick it in `spawn_blocking` for async contexts
pub(crate) fn load_advanced_settings() -> Result<AdvancedSettings> {
    let path = advanced_settings_path()?;
    let text = std::fs::read_to_string(path)?;
    let settings = serde_json::from_str(&text)?;
    Ok(settings)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_old_formats() {
        let s = r#"{
            "auth_base_url": "https://example.com/",
            "api_url": "wss://example.com/",
            "log_filter": "info"
        }"#;

        let actual = serde_json::from_str::<AdvancedSettings>(s).unwrap();
        // Apparently the trailing slash here matters
        assert_eq!(actual.auth_base_url.to_string(), "https://example.com/");
        assert_eq!(actual.api_url.to_string(), "wss://example.com/");
        assert_eq!(actual.log_filter, "info");
    }
}
