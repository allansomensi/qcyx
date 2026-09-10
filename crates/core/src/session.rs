//! Process-wide persistent BLE session.

use crate::battery::BatteryStatus;
use crate::client::{self, AncConfirmation, DeviceHandle};
use crate::command::{self, AncScene};
use crate::disconnect_power_off::DisconnectPowerOff;
use crate::eq::EqPreset;
use crate::error::CoreError;
use crate::game_mode::GameMode;
use crate::ldac::Ldac;
use crate::multipoint::Multipoint;
use crate::notification_volume::NotificationVolume;
use crate::scheduled_power_off::ScheduledPowerOff;
use crate::sleep_mode::SleepMode;
use crate::touch_action::{TouchAction, TouchActionMap, TouchControl};
use crate::version::FirmwareVersion;
use crate::wear_detection::WearDetection;
use std::sync::OnceLock;
use tokio::sync::Mutex;

static SHARED: OnceLock<Mutex<Option<DeviceHandle>>> = OnceLock::new();

fn shared_slot() -> &'static Mutex<Option<DeviceHandle>> {
    SHARED.get_or_init(|| Mutex::new(None))
}

/// Runs an operation against the shared session, connecting first if no
/// session is open, and drops the session whenever the operation errors.
///
/// Every setter/getter below used to repeat this by hand, and all but
/// `set_anc_scene` skipped the error cleanup: a single failed write (a GATT
/// timeout, an adapter hiccup) left a broken `DeviceHandle` sitting in the
/// slot. Because `slot.is_some()` was the only reconnect trigger, every
/// subsequent call reused that same dead handle and failed the same way —
/// silently, forever — until [`watch_disconnect`](crate) or an app restart
/// happened to clear it. Centralizing the pattern here makes that cleanup
/// unconditional instead of something each call site has to remember.
macro_rules! with_session {
    ($handle:ident => $body:expr) => {{
        let mut slot = shared_slot().lock().await;

        if slot.is_none() {
            *slot = Some(client::connect().await?);
        }

        let $handle = slot.as_ref().expect("just initialized above");
        let result = $body;

        if result.is_err()
            && let Some(handle) = slot.take()
        {
            let _ = handle.disconnect().await;
        }

        result
    }};
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
    /// In-ear wear detection and its ANC-on-wear sub-toggle, queried at
    /// connect time — not part of the state-sync blob, see [`crate::query`].
    pub initial_wear_detection: Option<WearDetection>,
    /// Notification volume, queried at connect time.
    pub initial_notification_volume: Option<NotificationVolume>,
    /// Scheduled (idle-independent) power-off timer, queried at connect time.
    pub initial_scheduled_power_off: Option<ScheduledPowerOff>,
    /// Power-off-after-disconnect timer, queried at connect time.
    pub initial_disconnect_power_off: Option<DisconnectPowerOff>,
    /// Game mode, queried at connect time.
    pub initial_game_mode: Option<GameMode>,
    /// Sleep mode, queried at connect time.
    pub initial_sleep_mode: Option<SleepMode>,
    /// LDAC codec toggle, queried at connect time.
    pub initial_ldac: Option<Ldac>,
    /// Dual-device (multipoint) connection toggle, queried at connect time.
    pub initial_multipoint: Option<Multipoint>,
    /// Touch-action map, read at connect time from its own characteristic.
    pub initial_touch_actions: Option<TouchActionMap>,
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

    let initial_wear_detection = match handle.get_wear_detection().await {
        Ok(state) => state,
        Err(e) => {
            tracing::debug!(error = %e, "failed to read initial wear-detection state");
            None
        }
    };

    let initial_notification_volume = match handle.get_notification_volume().await {
        Ok(level) => level,
        Err(e) => {
            tracing::debug!(error = %e, "failed to read initial notification volume");
            None
        }
    };

    let initial_scheduled_power_off = match handle.get_scheduled_power_off().await {
        Ok(value) => value,
        Err(e) => {
            tracing::debug!(error = %e, "failed to read initial scheduled power-off timer");
            None
        }
    };

    let initial_disconnect_power_off = match handle.get_disconnect_power_off().await {
        Ok(value) => value,
        Err(e) => {
            tracing::debug!(error = %e, "failed to read initial disconnect power-off timer");
            None
        }
    };

    let initial_game_mode = match handle.get_game_mode().await {
        Ok(state) => state,
        Err(e) => {
            tracing::debug!(error = %e, "failed to read initial game-mode state");
            None
        }
    };

    let initial_sleep_mode = match handle.get_sleep_mode().await {
        Ok(state) => state,
        Err(e) => {
            tracing::debug!(error = %e, "failed to read initial sleep-mode state");
            None
        }
    };

    let initial_ldac = match handle.get_ldac().await {
        Ok(state) => state,
        Err(e) => {
            tracing::debug!(error = %e, "failed to read initial LDAC state");
            None
        }
    };

    let initial_multipoint = match handle.get_multipoint().await {
        Ok(state) => state,
        Err(e) => {
            tracing::debug!(error = %e, "failed to read initial multipoint state");
            None
        }
    };

    let initial_touch_actions = match handle.read_touch_actions().await {
        Ok(map) => Some(map),
        Err(e) => {
            tracing::debug!(error = %e, "failed to read initial touch-action map");
            None
        }
    };

    *slot = Some(handle);
    Ok(ConnectionInfo {
        initial_anc_scene,
        initial_balance,
        device_name,
        firmware_version,
        initial_wear_detection,
        initial_notification_volume,
        initial_scheduled_power_off,
        initial_disconnect_power_off,
        initial_game_mode,
        initial_sleep_mode,
        initial_ldac,
        initial_multipoint,
        initial_touch_actions,
    })
}

/// Sends an ANC scene over the shared connection.
pub async fn set_anc_scene(scene: AncScene) -> Result<AncConfirmation, CoreError> {
    with_session!(handle => handle.send_anc_and_confirm(command::anc_scene(scene)).await)
}

/// Reads the battery status over the shared connection.
pub async fn read_battery() -> Result<BatteryStatus, CoreError> {
    with_session!(handle => handle.read_battery().await)
}

/// Reads the live BLE signal strength (RSSI) over the shared connection.
pub async fn read_rssi() -> Result<Option<i16>, CoreError> {
    with_session!(handle => handle.read_rssi().await)
}

/// Sends a balance write over the shared connection.
pub async fn set_balance(value: u8) -> Result<(), CoreError> {
    with_session!(handle => handle.set_balance(value).await)
}

/// Renames the device over the shared connection.
pub async fn set_name(name: &str) -> Result<(), CoreError> {
    with_session!(handle => handle.set_name(name).await)
}

/// Resets settings to default over the shared connection.
pub async fn reset_default() -> Result<(), CoreError> {
    with_session!(handle => handle.reset_default().await)
}

/// Factory-resets the device over the shared connection.
pub async fn factory_reset() -> Result<(), CoreError> {
    with_session!(handle => handle.factory_reset().await)
}

/// Sends a wear-detection write over the shared connection.
pub async fn set_wear_detection(state: WearDetection) -> Result<(), CoreError> {
    with_session!(handle => handle.set_wear_detection(state).await)
}

/// Sends a notification-volume write over the shared connection.
pub async fn set_notification_volume(level: NotificationVolume) -> Result<(), CoreError> {
    with_session!(handle => handle.set_notification_volume(level).await)
}

/// Sends a scheduled-power-off write over the shared connection.
pub async fn set_scheduled_power_off(value: ScheduledPowerOff) -> Result<(), CoreError> {
    with_session!(handle => handle.set_scheduled_power_off(value).await)
}

/// Sends a disconnect-power-off write over the shared connection.
pub async fn set_disconnect_power_off(value: DisconnectPowerOff) -> Result<(), CoreError> {
    with_session!(handle => handle.set_disconnect_power_off(value).await)
}

/// Sends a game-mode write over the shared connection.
pub async fn set_game_mode(state: GameMode) -> Result<(), CoreError> {
    with_session!(handle => handle.set_game_mode(state).await)
}

/// Sends a sleep-mode write over the shared connection.
pub async fn set_sleep_mode(state: SleepMode) -> Result<(), CoreError> {
    with_session!(handle => handle.set_sleep_mode(state).await)
}

/// Sends an LDAC-toggle write over the shared connection.
pub async fn set_ldac(state: Ldac) -> Result<(), CoreError> {
    with_session!(handle => handle.set_ldac(state).await)
}

/// Sends a multipoint-toggle write over the shared connection.
pub async fn set_multipoint(state: Multipoint) -> Result<(), CoreError> {
    with_session!(handle => handle.set_multipoint(state).await)
}

/// Sends a touch-action write over the shared connection.
pub async fn set_touch_action(control: TouchControl, action: TouchAction) -> Result<(), CoreError> {
    with_session!(handle => handle.set_touch_action(control, action).await)
}

/// Selects an equalizer preset over the shared connection.
pub async fn set_eq_preset(preset: EqPreset) -> Result<(), CoreError> {
    with_session!(handle => handle.set_eq_preset(preset).await)
}

/// Applies a custom (per-band) equalizer curve over the shared connection.
pub async fn set_eq_custom(gains_db: [i16; crate::eq::CUSTOM_BAND_COUNT]) -> Result<(), CoreError> {
    with_session!(handle => handle.set_eq_custom(gains_db).await)
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
