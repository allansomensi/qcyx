//! Process-wide persistent BLE session.

use crate::battery::BatteryStatus;
use crate::client::{self, AncConfirmation, DeviceHandle};
use crate::command::{self, AncScene};
use crate::eq::EqPreset;
use crate::error::CoreError;
use crate::version::FirmwareVersion;
use std::sync::OnceLock;
use tokio::sync::Mutex;

static SHARED: OnceLock<Mutex<Option<DeviceHandle>>> = OnceLock::new();

fn shared_slot() -> &'static Mutex<Option<DeviceHandle>> {
    SHARED.get_or_init(|| Mutex::new(None))
}

/// Static, connection-scoped info read right after a fresh connection is established.
#[derive(Debug, Clone, Default)]
pub struct ConnectionInfo {
    /// The device's ANC scene at the moment of connecting.
    pub initial_anc_scene: Option<AncScene>,
    /// The device's channel balance at the moment of connecting (`0..=100`, 50 = centered).
    pub initial_balance: Option<u8>,
    /// The device's advertised BLE name.
    pub device_name: Option<String>,
    /// Firmware version.
    pub firmware_version: Option<FirmwareVersion>,
}

/// Connects the shared session if it isn't already open.
pub async fn ensure_connected() -> Result<ConnectionInfo, CoreError> {
    let mut slot = shared_slot().lock().await;

    if slot.is_some() {
        return Ok(ConnectionInfo::default());
    }

    let handle = client::connect().await?;

    let (initial_anc_scene, initial_balance) = match handle.read_state_sync().await {
        Ok(state) => {
            let scene = state
                .anc
                .and_then(|s| AncScene::from_triplet(s.mode, s.sub_scene, s.noise_value));
            (scene, state.balance)
        }
        Err(e) => {
            tracing::debug!(error = %e, "failed to read initial device state");
            (None, None)
        }
    };

    let firmware_version = match handle.read_version().await {
        Ok(version) => Some(version),
        Err(e) => {
            tracing::debug!(error = %e, "failed to read firmware version");
            None
        }
    };

    let device_name = handle.device_name().map(str::to_string);

    *slot = Some(handle);
    Ok(ConnectionInfo {
        initial_anc_scene,
        initial_balance,
        device_name,
        firmware_version,
    })
}

/// Sends an ANC scene over the shared connection.
pub async fn set_anc_scene(scene: AncScene) -> Result<AncConfirmation, CoreError> {
    let mut slot = shared_slot().lock().await;

    if slot.is_none() {
        *slot = Some(client::connect().await?);
    }

    let handle = slot.as_ref().expect("just initialized");
    let result = handle.send_anc_and_confirm(command::anc_scene(scene)).await;

    if result.is_err()
        && let Some(handle) = slot.take()
    {
        let _ = handle.disconnect().await;
    }

    result
}

/// Reads the battery status over the shared connection.
pub async fn read_battery() -> Result<BatteryStatus, CoreError> {
    let mut slot = shared_slot().lock().await;

    if slot.is_none() {
        *slot = Some(client::connect().await?);
    }

    let handle = slot.as_ref().expect("just initialized");
    handle.read_battery().await
}

/// Sends a balance write over the shared connection.
pub async fn set_balance(value: u8) -> Result<(), CoreError> {
    let mut slot = shared_slot().lock().await;

    if slot.is_none() {
        *slot = Some(client::connect().await?);
    }

    let handle = slot.as_ref().expect("just initialized");
    handle.set_balance(value).await
}

/// Renames the device over the shared connection.
pub async fn set_name(name: &str) -> Result<(), CoreError> {
    let mut slot = shared_slot().lock().await;

    if slot.is_none() {
        *slot = Some(client::connect().await?);
    }

    let handle = slot.as_ref().expect("just initialized");
    handle.set_name(name).await
}

/// Resets settings to default over the shared connection.
pub async fn reset_default() -> Result<(), CoreError> {
    let mut slot = shared_slot().lock().await;

    if slot.is_none() {
        *slot = Some(client::connect().await?);
    }

    let handle = slot.as_ref().expect("just initialized");
    handle.reset_default().await
}

/// Factory-resets the device over the shared connection.
pub async fn factory_reset() -> Result<(), CoreError> {
    let mut slot = shared_slot().lock().await;

    if slot.is_none() {
        *slot = Some(client::connect().await?);
    }

    let handle = slot.as_ref().expect("just initialized");
    handle.factory_reset().await
}

/// Selects an equalizer preset over the shared connection.
pub async fn set_eq_preset(preset: EqPreset) -> Result<(), CoreError> {
    let mut slot = shared_slot().lock().await;

    if slot.is_none() {
        *slot = Some(client::connect().await?);
    }

    let handle = slot.as_ref().expect("just initialized");
    handle.set_eq_preset(preset).await
}

/// Returns `true` if the shared connection is open and the underlying BLE link is alive.
pub async fn is_connected() -> bool {
    let mut slot = shared_slot().lock().await;

    let Some(handle) = slot.as_ref() else {
        return false;
    };

    match handle.is_connected().await {
        Ok(true) => true,
        _ => {
            *slot = None;
            false
        }
    }
}

/// Drops the shared connection, disconnecting from the device if one is open.
pub async fn disconnect_shared() {
    let mut slot = shared_slot().lock().await;
    if let Some(handle) = slot.take() {
        let _ = handle.disconnect().await;
    }
}
