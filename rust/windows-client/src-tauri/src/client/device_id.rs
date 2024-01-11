use known_folders::{get_known_folder_path, KnownFolder};
use tokio::fs;

#[derive(thiserror::Error, Debug)]
pub(crate) enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("Can't find well-known folder")]
    KnownFolder,
}

/// Returns the device ID, generating it and saving it to disk if needed.
///
/// Per <https://github.com/firezone/firezone/issues/2697> and <https://github.com/firezone/firezone/issues/2711>,
/// clients must generate their own random IDs and persist them to disk, to handle situations like VMs where a hardware ID is not unique or not available.
///
/// # Arguments
///
/// * `identifier` - Our Tauri bundle identifier, e.g. "dev.firezone.client"
///
/// Returns: The UUID as a String, suitable for sending verbatim to `connlib_client_shared::Session::connect`.
///
/// Errors: If the disk is unwritable when initially generating the ID, or unwritable when re-generating an invalid ID.
pub(crate) async fn device_id(identifier: &str) -> Result<String, Error> {
    let dir = get_known_folder_path(KnownFolder::ProgramData)
        .ok_or(Error::KnownFolder)?
        .join(identifier)
        .join("config");
    let path = dir.join("device_id.json");

    // Try to read it back from disk
    if let Some(j) = fs::read_to_string(&path)
        .await
        .ok()
        .and_then(|s| serde_json::from_str::<DeviceIdJson>(&s).ok())
    {
        let device_id = j.device_id();
        tracing::debug!(?device_id, "Loaded device ID from disk");
        return Ok(device_id);
    }

    // Couldn't read, it's missing or invalid, generate a new one and save it.
    let id = uuid::Uuid::new_v4();
    let j = DeviceIdJson { id };
    // TODO: This file write has the same possible problems with power loss as described here https://github.com/firezone/firezone/pull/2757#discussion_r1416374516
    // Since the device ID is random, typically only written once in the device's lifetime, and the read will error out if it's corrupted, it's low-risk.
    fs::create_dir_all(&dir).await?;
    fs::write(
        &path,
        serde_json::to_string(&j).expect("Device ID should always be serializable"),
    )
    .await?;

    let device_id = j.device_id();
    tracing::debug!(?device_id, "Saved device ID to disk");
    Ok(j.device_id())
}

#[derive(serde::Deserialize, serde::Serialize)]
struct DeviceIdJson {
    id: uuid::Uuid,
}

impl DeviceIdJson {
    fn device_id(&self) -> String {
        self.id.to_string()
    }
}
