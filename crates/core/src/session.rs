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

/// Runs a user-initiated operation against the shared session, connecting
/// first if no session is open.
///
/// The session is dropped, and best-effort disconnected, only when the
/// operation fails at the transport level ([`CoreError::is_transport`]). A
/// dead handle left in the slot fails every later call the same way; a
/// healthy one torn down over a validation or parse error forces a full
/// scan/connect cycle for nothing.
macro_rules! with_session {
    ($handle:ident => $body:expr) => {{
        let mut slot = shared_slot().lock().await;

        if slot.is_none() {
            *slot = Some(client::connect().await?);
        }

        let $handle = slot.as_ref().expect("just initialized above");
        let result = $body;

        if let Err(error) = &result
            && error.is_transport()
            && let Some(handle) = slot.take()
        {
            tracing::debug!(%error, "dropping the shared session after a transport failure");
            let _ = handle.disconnect().await;
        }

        result
    }};
}

/// Runs a background poll against the shared session, only if one is open.
///
/// Never connects and never drops the session. A poll that connected on its
/// own raced `connect_poll` for the link, and a GUI state change could cancel
/// it mid-connect — leaving the scan running and the peripheral connected
/// with no `disconnect()`. Detecting a dead link is [`is_connected`]'s job; a
/// failed poll only reports its own failure.
macro_rules! with_open_session {
    ($handle:ident => $body:expr) => {{
        let slot = shared_slot().lock().await;

        match slot.as_ref() {
            Some($handle) => $body,
            None => Err(CoreError::NotConnected),
        }
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

/// Connects the shared session if it isn't already open, and reads the
/// connection-scoped device state.
///
/// The state is read even when the session was already open: another
/// operation may have reconnected it first, and an empty [`ConnectionInfo`]
/// would blank every setting in the GUI.
pub async fn ensure_connected() -> Result<ConnectionInfo, CoreError> {
    let mut slot = shared_slot().lock().await;

    if slot.is_none() {
        *slot = Some(client::connect().await?);
    }

    let handle = slot.as_ref().expect("just initialized above");
    Ok(read_connection_info(handle).await)
}

/// Reads everything [`ConnectionInfo`] carries. Every field is best-effort:
/// a failed read is logged and left `None`.
async fn read_connection_info(handle: &DeviceHandle) -> ConnectionInfo {
    let (initial_anc_scene, initial_balance) =
        match best_effort("device state", handle.read_state_sync().await) {
            Some(state) => (
                state
                    .anc
                    .and_then(|s| AncScene::from_triplet(s.mode, s.sub_scene, s.noise_value)),
                state.balance,
            ),
            None => (None, None),
        };

    let firmware_version = best_effort("firmware version", handle.read_version().await);
    let initial_wear_detection =
        best_effort("wear-detection state", handle.get_wear_detection().await).flatten();
    let initial_notification_volume = best_effort(
        "notification volume",
        handle.get_notification_volume().await,
    )
    .flatten();
    let initial_scheduled_power_off = best_effort(
        "scheduled power-off timer",
        handle.get_scheduled_power_off().await,
    )
    .flatten();
    let initial_disconnect_power_off = best_effort(
        "disconnect power-off timer",
        handle.get_disconnect_power_off().await,
    )
    .flatten();
    let initial_game_mode = best_effort("game-mode state", handle.get_game_mode().await).flatten();
    let initial_sleep_mode =
        best_effort("sleep-mode state", handle.get_sleep_mode().await).flatten();
    let initial_ldac = best_effort("LDAC state", handle.get_ldac().await).flatten();
    let initial_multipoint =
        best_effort("multipoint state", handle.get_multipoint().await).flatten();
    let initial_touch_actions = best_effort("touch-action map", handle.read_touch_actions().await);

    ConnectionInfo {
        initial_anc_scene,
        initial_balance,
        device_name: handle.device_name().map(str::to_string),
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
    }
}

/// Logs and discards a failed connect-time read.
fn best_effort<T>(what: &str, result: Result<T, CoreError>) -> Option<T> {
    result
        .inspect_err(|error| tracing::debug!(%error, "failed to read initial {what}"))
        .ok()
}

/// Sends an ANC scene over the shared connection.
pub async fn set_anc_scene(scene: AncScene) -> Result<AncConfirmation, CoreError> {
    with_session!(handle => handle.send_anc_and_confirm(command::anc_scene(scene)).await)
}

/// Reads the battery status over an already-open shared connection.
/// Background poll: never connects, never drops the session.
pub async fn read_battery() -> Result<BatteryStatus, CoreError> {
    with_open_session!(handle => handle.read_battery().await)
}

/// Reads the live BLE signal strength (RSSI) over an already-open shared
/// connection. Background poll: never connects, never drops the session.
pub async fn read_rssi() -> Result<Option<i16>, CoreError> {
    with_open_session!(handle => handle.read_rssi().await)
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
///
/// A handle that fails the check is released with a best-effort disconnect:
/// the link may still be up at the OS level (the check itself can time out),
/// and a connected peripheral stops advertising, so the next scan would find
/// nothing.
pub async fn is_connected() -> bool {
    let mut slot = shared_slot().lock().await;

    let Some(handle) = slot.as_ref() else {
        return false;
    };

    if matches!(handle.is_connected().await, Ok(true)) {
        return true;
    }

    if let Some(handle) = slot.take() {
        let _ = handle.disconnect().await;
    }
    false
}

/// Drops the shared connection, disconnecting from the device if one is open.
pub async fn disconnect_shared() {
    let mut slot = shared_slot().lock().await;
    if let Some(handle) = slot.take() {
        let _ = handle.disconnect().await;
    }
}
