//! High-level BLE orchestration for ANC control: device discovery,
//! connection, and command flow.
//!
//! Every call into `btleplug` runs under a [`crate::timeout`] budget. The
//! platform stacks can leave a GATT call pending forever when a link dies
//! uncleanly, and a hang here strands the shared session for the whole process.

use crate::balance;
use crate::battery::BatteryStatus;
use crate::command::{self, AncScene, opcode};
use crate::device::profile::{
    BATTERY_UUID, COMMAND_UUID, NOTIFY_UUID, QCY_COMPANY_ID, SERVICE_UUID, TOUCH_ACTION_UUID,
    VERSION_UUID,
};
use crate::device_actions;
use crate::disconnect_power_off::{self, DisconnectPowerOff};
use crate::eq::EqPreset;
use crate::error::CoreError;
use crate::game_mode::{self, GameMode};
use crate::ldac::{self, Ldac};
use crate::multipoint::{self, Multipoint};
use crate::notification_volume::{self, NotificationVolume};
use crate::protocol::Command;
use crate::query;
use crate::response::{self, Event};
use crate::scheduled_power_off::{self, ScheduledPowerOff};
use crate::sleep_mode::{self, SleepMode};
use crate::timeout::{budget, guard};
use crate::touch_action;
use crate::version::FirmwareVersion;
use crate::wear_detection::{self, WearDetection};
use btleplug::api::{
    Central, CentralEvent, CharPropFlags, Characteristic, Manager as _, Peripheral as _,
    ScanFilter, WriteType,
};
use btleplug::platform::{Adapter, Manager, Peripheral};
use futures::stream::{FuturesUnordered, StreamExt};
use qcyx_i18n::fl;
use std::time::Duration;
use tokio::time;

/// Ceiling on how long to wait for a QCY advertisement.
///
/// The scan resolves as soon as a matching advertisement arrives, so this is an
/// upper bound rather than a fixed cost — the old fixed 5s sleep was paid in
/// full even when the device answered on the first advertisement.
const SCAN_TIMEOUT: Duration = Duration::from_secs(8);

/// How long to wait for the asynchronous `ANC_RESULT` confirmation.
const ANC_CONFIRM_TIMEOUT: Duration = Duration::from_secs(10);

/// How long to wait for a queried parameter's notify reply.
const QUERY_TIMEOUT: Duration = Duration::from_secs(5);

/// Retries for the initial BLE connect.
const CONNECT_RETRIES: u32 = 3;

/// Retries for service/characteristic discovery.
const DISCOVERY_RETRIES: u32 = 3;

/// Window over which a fresh link must stay up before it is trusted.
///
/// WinRT can report `connect()` as `Ok` and tear the link down a moment later.
const CONNECT_SETTLE: Duration = Duration::from_millis(400);

/// Liveness poll interval inside [`CONNECT_SETTLE`].
const CONNECT_SETTLE_POLL: Duration = Duration::from_millis(50);

/// Flush margin after a `WriteWithoutResponse`, which carries no ATT ack.
const WRITE_FLUSH: Duration = Duration::from_millis(60);

/// Flush margin before a deliberate disconnect, so queued writes leave the stack.
const DISCONNECT_FLUSH: Duration = Duration::from_millis(250);

/// Settle margin after subscribing, before the first command may be written.
const SUBSCRIBE_SETTLE: Duration = Duration::from_millis(150);

/// ATT header every write PDU pays (1-byte opcode + 2-byte handle), on top
/// of the negotiated MTU, regardless of `WriteType`.
const ATT_WRITE_HEADER: u16 = 3;

/// Returns `true` when the peripheral advertises the QCY Company ID.
async fn is_qcy(peripheral: &Peripheral) -> bool {
    matches!(
        peripheral.properties().await,
        Ok(Some(properties)) if properties.manufacturer_data.contains_key(&QCY_COMPANY_ID)
    )
}

/// Checks peripherals the platform already knows about, before scanning.
///
/// A device paired in the OS is usually cached by both WinRT and BlueZ, which
/// makes the whole scan avoidable on the common path.
async fn cached_qcy_peripheral(adapter: &Adapter) -> Result<Option<Peripheral>, CoreError> {
    let peripherals = guard(budget::SCAN_CONTROL, "peripherals", async {
        Ok(adapter.peripherals().await?)
    })
    .await?;

    for peripheral in peripherals {
        if is_qcy(&peripheral).await {
            return Ok(Some(peripheral));
        }
    }

    Ok(None)
}

/// Looks for a QCY device on one adapter: cache first, then an event-driven scan.
///
/// The scan consumes [`CentralEvent`]s and resolves on the first advertisement
/// carrying the QCY Company ID. The caller is responsible for stopping the scan
/// — see [`find_qcy_device`], which does so for every adapter unconditionally.
async fn find_on_adapter(adapter: &Adapter) -> Result<Option<Peripheral>, CoreError> {
    if let Some(peripheral) = cached_qcy_peripheral(adapter).await? {
        return Ok(Some(peripheral));
    }

    let mut events = guard(budget::SCAN_CONTROL, "events", async {
        Ok(adapter.events().await?)
    })
    .await?;

    guard(budget::SCAN_CONTROL, "start_scan", async {
        adapter.start_scan(ScanFilter::default()).await?;
        Ok(())
    })
    .await?;

    // `ScanFilter::default()` is deliberate: QCY advertises no service UUID, so
    // a service-filtered scan finds nothing on BlueZ.
    let found = time::timeout(SCAN_TIMEOUT, async {
        while let Some(event) = events.next().await {
            let id = match event {
                CentralEvent::DeviceDiscovered(id)
                | CentralEvent::DeviceUpdated(id)
                | CentralEvent::ManufacturerDataAdvertisement { id, .. } => id,
                _ => continue,
            };

            let Ok(peripheral) = adapter.peripheral(&id).await else {
                continue;
            };

            if is_qcy(&peripheral).await {
                return Some(peripheral);
            }
        }
        None
    })
    .await
    .ok()
    .flatten();

    Ok(found)
}

/// Finds a QCY peripheral, searching every adapter the platform reports.
///
/// Machines with both an internal radio and a USB dongle enumerate adapters in
/// a non-deterministic order, so taking the first one made the app work or not
/// depending on the boot. Adapters are searched concurrently and the first
/// match wins; a failing adapter is logged, not propagated.
async fn find_qcy_device() -> Result<Peripheral, CoreError> {
    let manager = Manager::new().await?;
    let adapters = guard(budget::SCAN_CONTROL, "adapters", async {
        Ok(manager.adapters().await?)
    })
    .await?;

    if adapters.is_empty() {
        return Err(CoreError::AdapterNotFound);
    }

    let outcome = {
        let mut searches: FuturesUnordered<_> = adapters
            .iter()
            .map(|adapter| async move {
                let label = adapter
                    .adapter_info()
                    .await
                    .unwrap_or_else(|_| "unknown adapter".to_string());
                (label, find_on_adapter(adapter).await)
            })
            .collect();

        let mut result = Err(CoreError::DeviceNotFound);

        while let Some((label, search)) = searches.next().await {
            match search {
                Ok(Some(peripheral)) => {
                    tracing::info!(adapter = %label, "QCY device found");
                    result = Ok(peripheral);
                    break;
                }
                Ok(None) => tracing::debug!(adapter = %label, "no QCY advertisement seen"),
                Err(e) => tracing::warn!(adapter = %label, error = %e, "adapter unusable"),
            }
        }

        result
    };

    // Unconditional, on every path: an adapter left in active discovery drains
    // power and degrades BLE for the whole system, and blocks the connect that
    // follows on BlueZ. Cancelled searches never get to clean up after
    // themselves, so cleanup lives here.
    for adapter in &adapters {
        let _ = adapter.stop_scan().await;
    }

    outcome
}

/// The outcomes an ANC write can produce.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AncConfirmation {
    /// `ANC_RESULT` reported success.
    Applied,
    /// `ANC_RESULT` reported failure.
    Rejected,
    /// The write was echoed back, but no `ANC_RESULT` arrived before timeout.
    EchoedOnly,
    /// No response on the notify channel.
    NoResponse,
}

/// Device state extracted from a sync read.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct StateSync {
    pub anc: Option<response::AncSetting>,
    pub balance: Option<u8>,
}

/// A connected QCY device, ready to send commands.
pub struct DeviceHandle {
    peripheral: Peripheral,
    write_char: Characteristic,
    write_type: WriteType,
    notify_char: Characteristic,
    battery_char: Option<Characteristic>,
    version_char: Option<Characteristic>,
    touch_action_char: Option<Characteristic>,
    device_name: Option<String>,
}

impl DeviceHandle {
    /// Picks the write type for a payload of `payload_len` bytes.
    ///
    /// `WriteWithoutResponse` has no long-write path: if the payload doesn't
    /// fit inside the peripheral's *currently negotiated* MTU, it is
    /// truncated silently rather than split. The EQ preset command is 145
    /// bytes and the characteristic does not reliably accept `WriteRequest`
    /// for it, so this must go out as `WriteWithoutResponse` against an
    /// actually-negotiated MTU rather than the unnegotiated 20-byte default
    /// — `btleplug` negotiates that MTU per-platform after service discovery
    /// without the app asking (BlueZ, WinRT and CoreBluetooth all update
    /// `Peripheral::mtu()` on their own), so checking the real value here is
    /// enough.
    fn write_type_for(&self, payload_len: usize) -> WriteType {
        let usable_mtu = self.peripheral.mtu().saturating_sub(ATT_WRITE_HEADER) as usize;
        if payload_len > usable_mtu {
            WriteType::WithResponse
        } else {
            self.write_type
        }
    }

    async fn send(&self, cmd: Command) -> Result<(), CoreError> {
        cmd.validate()?;

        let payload = cmd.pack();
        let write_type = self.write_type_for(payload.len());

        guard(budget::GATT_OP, "write", async {
            self.peripheral
                .write(&self.write_char, &payload, write_type)
                .await?;
            Ok(())
        })
        .await?;

        // `WithResponse` is acknowledged at the ATT layer, so there is nothing
        // left to wait for. `WithoutResponse` is not: a short margin keeps the
        // packet from being dropped by a disconnect that follows immediately.
        // The larger margin a deliberate teardown needs lives in `finish`, not
        // on every write — the persistent GUI session never disconnects, and
        // was paying it on every slider commit.
        if matches!(write_type, WriteType::WithoutResponse) {
            time::sleep(WRITE_FLUSH).await;
        }

        Ok(())
    }

    /// Sends an ANC-setting write and waits for confirmation.
    pub async fn send_anc_and_confirm(&self, cmd: Command) -> Result<AncConfirmation, CoreError> {
        let mut notifications = guard(budget::GATT_OP, "notifications", async {
            Ok(self.peripheral.notifications().await?)
        })
        .await?;

        self.send(cmd).await?;

        let deadline = time::Instant::now() + ANC_CONFIRM_TIMEOUT;
        let mut echoed = false;

        loop {
            let remaining = deadline.saturating_duration_since(time::Instant::now());
            if remaining.is_zero() {
                break;
            }

            let slice = remaining.min(Duration::from_millis(500));
            let data = match time::timeout(slice, notifications.next()).await {
                Ok(Some(data)) => data,
                Ok(None) => {
                    tracing::warn!("notification stream ended");
                    break;
                }
                Err(_elapsed) => {
                    let connected = self.is_connected().await;
                    tracing::trace!(connected = ?connected, "waiting for ANC confirmation");

                    // A link that died mid-wait will never deliver the result;
                    // burning the rest of the budget only delays the report.
                    if matches!(connected, Ok(false)) {
                        return Err(CoreError::ConnectionDropped);
                    }
                    continue;
                }
            };

            tracing::trace!(raw = ?data.value, "notification received");
            let Ok(commands) = Command::parse(&data.value) else {
                continue;
            };

            for cmd in commands {
                tracing::trace!(opcode = format!("0x{:02X}", cmd.opcode), params = ?cmd.parameters, "parsed notification");
                match cmd.opcode {
                    opcode::ANC_SETTING => echoed = true,
                    opcode::ANC_RESULT => {
                        if let Event::AncResult(result) =
                            response::dispatch(cmd.opcode, &cmd.parameters)
                        {
                            return Ok(if result.applied {
                                AncConfirmation::Applied
                            } else {
                                AncConfirmation::Rejected
                            });
                        }
                    }
                    _ => {}
                }
            }
        }

        Ok(if echoed {
            AncConfirmation::EchoedOnly
        } else {
            AncConfirmation::NoResponse
        })
    }

    /// Disconnects from the device.
    pub async fn disconnect(&self) -> Result<(), CoreError> {
        guard(budget::GATT_OP, "disconnect", async {
            self.peripheral.disconnect().await?;
            Ok(())
        })
        .await
    }

    /// Checks if the BLE link is active.
    pub async fn is_connected(&self) -> Result<bool, CoreError> {
        guard(budget::GATT_OP, "is_connected", async {
            Ok(self.peripheral.is_connected().await?)
        })
        .await
    }

    /// Reads current battery status.
    pub async fn read_battery(&self) -> Result<BatteryStatus, CoreError> {
        let battery_char = self
            .battery_char
            .as_ref()
            .ok_or_else(|| CoreError::CharacteristicNotFound("battery (00000008)".into()))?;

        let data = guard(budget::GATT_OP, "read_battery", async {
            Ok(self.peripheral.read(battery_char).await?)
        })
        .await?;

        BatteryStatus::parse(&data)
            .ok_or_else(|| CoreError::InvalidPacket("battery payload shorter than 3 bytes".into()))
    }

    /// Reads device settings via a direct GATT read on the notify characteristic.
    pub async fn read_state_sync(&self) -> Result<StateSync, CoreError> {
        let data = guard(budget::GATT_OP, "read_state_sync", async {
            Ok(self.peripheral.read(&self.notify_char).await?)
        })
        .await?;

        let commands = Command::parse(&data)?;

        let mut state = StateSync::default();
        for cmd in commands {
            match cmd.opcode {
                opcode::ANC_SETTING => {
                    state.anc = response::AncSetting::parse(&cmd.parameters);
                }
                balance::OPCODE => {
                    state.balance = balance::parse_balance(&cmd.parameters);
                }
                _ => {}
            }
        }

        Ok(state)
    }

    /// Returns the device's advertised BLE name.
    pub fn device_name(&self) -> Option<&str> {
        self.device_name.as_deref()
    }

    /// Reads the current firmware version.
    pub async fn read_version(&self) -> Result<FirmwareVersion, CoreError> {
        let version_char = self
            .version_char
            .as_ref()
            .ok_or_else(|| CoreError::CharacteristicNotFound("version (00000007)".into()))?;

        let data = guard(budget::GATT_OP, "read_version", async {
            Ok(self.peripheral.read(version_char).await?)
        })
        .await?;

        FirmwareVersion::parse(&data)
            .ok_or_else(|| CoreError::InvalidPacket("version payload was empty".into()))
    }

    /// Reads the live BLE signal strength (RSSI, in dBm) for the connected
    /// peripheral, when the platform's BLE stack reports one. `Ok(None)`
    /// (not an error) means the adapter simply didn't include it in the
    /// advertisement/connection properties — this is common right after a
    /// fresh connection on some platforms.
    pub async fn read_rssi(&self) -> Result<Option<i16>, CoreError> {
        let props = guard(budget::GATT_OP, "read_rssi", async {
            Ok(self.peripheral.properties().await?)
        })
        .await?;

        Ok(props.and_then(|p| p.rssi))
    }

    /// Sends a balance-set write.
    pub async fn set_balance(&self, value: u8) -> Result<(), CoreError> {
        self.send(balance::set_balance(value)).await
    }

    /// Sends a `0xFE` query for `target_opcode` and waits for its notify
    /// reply, returning the raw parameter bytes.
    ///
    /// Covers every setting the official app polls on screen load but that
    /// [`Self::read_state_sync`]'s single blob read doesn't carry — see
    /// [`crate::query`].
    pub async fn query_param(&self, target_opcode: u8) -> Result<Vec<u8>, CoreError> {
        let mut notifications = guard(budget::GATT_OP, "notifications", async {
            Ok(self.peripheral.notifications().await?)
        })
        .await?;

        self.send(query::request(target_opcode)).await?;

        let deadline = time::Instant::now() + QUERY_TIMEOUT;
        loop {
            let remaining = deadline.saturating_duration_since(time::Instant::now());
            if remaining.is_zero() {
                return Err(CoreError::OperationTimeout("parameter query"));
            }

            let data = match time::timeout(remaining, notifications.next()).await {
                Ok(Some(data)) => data,
                Ok(None) => return Err(CoreError::ConnectionDropped),
                Err(_elapsed) => return Err(CoreError::OperationTimeout("parameter query")),
            };

            let Ok(commands) = Command::parse(&data.value) else {
                continue;
            };

            if let Some(cmd) = commands.into_iter().find(|c| c.opcode == target_opcode) {
                return Ok(cmd.parameters);
            }
        }
    }

    /// Sets the notification volume.
    pub async fn set_notification_volume(
        &self,
        level: NotificationVolume,
    ) -> Result<(), CoreError> {
        self.send(notification_volume::set_notification_volume(level))
            .await
    }

    /// Reads the current notification volume.
    pub async fn get_notification_volume(&self) -> Result<Option<NotificationVolume>, CoreError> {
        let params = self.query_param(notification_volume::OPCODE).await?;
        Ok(notification_volume::parse_notification_volume(&params))
    }

    /// Sets the scheduled (idle-independent) power-off timer.
    pub async fn set_scheduled_power_off(&self, value: ScheduledPowerOff) -> Result<(), CoreError> {
        self.send(scheduled_power_off::set_scheduled_power_off(value))
            .await
    }

    /// Reads the current scheduled power-off timer.
    pub async fn get_scheduled_power_off(&self) -> Result<Option<ScheduledPowerOff>, CoreError> {
        let params = self.query_param(scheduled_power_off::OPCODE).await?;
        Ok(scheduled_power_off::parse_scheduled_power_off(&params))
    }

    /// Sets the power-off-after-disconnect timer.
    pub async fn set_disconnect_power_off(
        &self,
        value: DisconnectPowerOff,
    ) -> Result<(), CoreError> {
        self.send(disconnect_power_off::set_disconnect_power_off(value))
            .await
    }

    /// Reads the current power-off-after-disconnect timer.
    pub async fn get_disconnect_power_off(&self) -> Result<Option<DisconnectPowerOff>, CoreError> {
        let params = self.query_param(disconnect_power_off::OPCODE).await?;
        Ok(disconnect_power_off::parse_disconnect_power_off(&params))
    }

    /// Sets in-ear wear detection and its ANC-on-wear sub-toggle.
    pub async fn set_wear_detection(&self, state: WearDetection) -> Result<(), CoreError> {
        self.send(wear_detection::set_wear_detection(state)).await
    }

    /// Reads the current wear-detection state.
    pub async fn get_wear_detection(&self) -> Result<Option<WearDetection>, CoreError> {
        let params = self.query_param(wear_detection::OPCODE).await?;
        Ok(wear_detection::parse_wear_detection(&params))
    }

    /// Sets game (low-latency) mode.
    pub async fn set_game_mode(&self, state: GameMode) -> Result<(), CoreError> {
        self.send(game_mode::set_game_mode(state)).await
    }

    /// Reads the current game-mode state.
    pub async fn get_game_mode(&self) -> Result<Option<GameMode>, CoreError> {
        let params = self.query_param(game_mode::OPCODE).await?;
        Ok(game_mode::parse_game_mode(&params))
    }

    /// Writes a raw payload directly to a characteristic, bypassing the
    /// `0xFF` frame. Used only for the touch-action characteristic.
    async fn write_raw(
        &self,
        characteristic: &Characteristic,
        payload: &[u8],
    ) -> Result<(), CoreError> {
        let write_type = self.write_type_for(payload.len());

        guard(budget::GATT_OP, "write_raw", async {
            self.peripheral
                .write(characteristic, payload, write_type)
                .await?;
            Ok(())
        })
        .await?;

        if matches!(write_type, WriteType::WithoutResponse) {
            time::sleep(WRITE_FLUSH).await;
        }

        Ok(())
    }

    /// Assigns a tap action to one earbud/click-count control.
    pub async fn set_touch_action(
        &self,
        control: touch_action::TouchControl,
        action: touch_action::TouchAction,
    ) -> Result<(), CoreError> {
        let touch_action_char = self
            .touch_action_char
            .as_ref()
            .ok_or_else(|| CoreError::CharacteristicNotFound("touch action (0000000d)".into()))?;

        let payload = touch_action::set_touch_action(control, action);
        self.write_raw(touch_action_char, &payload).await
    }

    /// Reads the full six-control touch-action map.
    pub async fn read_touch_actions(&self) -> Result<touch_action::TouchActionMap, CoreError> {
        let touch_action_char = self
            .touch_action_char
            .as_ref()
            .ok_or_else(|| CoreError::CharacteristicNotFound("touch action (0000000d)".into()))?;

        let data = guard(budget::GATT_OP, "read_touch_actions", async {
            Ok(self.peripheral.read(touch_action_char).await?)
        })
        .await?;

        Ok(touch_action::TouchActionMap::parse(&data))
    }

    /// Sets the LDAC codec toggle.
    pub async fn set_ldac(&self, state: Ldac) -> Result<(), CoreError> {
        self.send(ldac::set_ldac(state)).await
    }

    /// Reads the current LDAC toggle state.
    pub async fn get_ldac(&self) -> Result<Option<Ldac>, CoreError> {
        let params = self.query_param(ldac::OPCODE).await?;
        Ok(ldac::parse_ldac(&params))
    }

    /// Sets the dual-device (multipoint) connection toggle.
    pub async fn set_multipoint(&self, state: Multipoint) -> Result<(), CoreError> {
        self.send(multipoint::set_multipoint(state)).await
    }

    /// Reads the current multipoint toggle state.
    pub async fn get_multipoint(&self) -> Result<Option<Multipoint>, CoreError> {
        let params = self.query_param(multipoint::OPCODE).await?;
        Ok(multipoint::parse_multipoint(&params))
    }

    /// Sets sleep mode.
    pub async fn set_sleep_mode(&self, state: SleepMode) -> Result<(), CoreError> {
        self.send(sleep_mode::set_sleep_mode(state)).await
    }

    /// Reads the current sleep-mode state.
    pub async fn get_sleep_mode(&self) -> Result<Option<SleepMode>, CoreError> {
        let params = self.query_param(sleep_mode::OPCODE).await?;
        Ok(sleep_mode::parse_sleep_mode(&params))
    }

    /// Resets settings to default.
    pub async fn reset_default(&self) -> Result<(), CoreError> {
        self.send(device_actions::reset_default()).await
    }

    /// Factory-resets the device.
    pub async fn factory_reset(&self) -> Result<(), CoreError> {
        self.send(device_actions::factory_reset()).await
    }

    /// Renames the device's Bluetooth pairing name.
    ///
    /// Rejects a name that sanitizes to nothing (blank, or only control
    /// characters): the CLI takes a raw `String` with no client-side check,
    /// so without this an empty `PAIRNAME` write goes out silently and the
    /// caller is told the rename "succeeded" for a name that was never sent.
    pub async fn set_name(&self, name: &str) -> Result<(), CoreError> {
        let cmd = device_actions::set_name(name);
        if cmd.parameters.is_empty() {
            return Err(CoreError::InvalidName);
        }
        self.send(cmd).await
    }

    /// Selects an equalizer preset.
    pub async fn set_eq_preset(&self, preset: EqPreset) -> Result<(), CoreError> {
        self.send(crate::eq::set_preset(preset)).await
    }

    /// Applies a custom (per-band) equalizer curve. `gains_db` is in
    /// [`crate::eq::CUSTOM_BAND_FREQS_HZ`] order (31 Hz .. 16 kHz), each
    /// clamped to [`crate::eq::CUSTOM_GAIN_MIN_DB`]..=[`crate::eq::CUSTOM_GAIN_MAX_DB`].
    pub async fn set_eq_custom(
        &self,
        gains_db: [i16; crate::eq::CUSTOM_BAND_COUNT],
    ) -> Result<(), CoreError> {
        self.send(crate::eq::set_custom(gains_db)).await
    }
}

/// Connects to the device, discovers services, and sets up notifications.
pub async fn connect() -> Result<DeviceHandle, CoreError> {
    tracing::info!("{}", fl!("core-connecting"));
    let peripheral = find_qcy_device().await?;

    if let Err(e) = connect_with_retry(&peripheral).await {
        let _ = peripheral.disconnect().await;
        return Err(e);
    }

    match setup_device(&peripheral).await {
        Ok(setup) => {
            let device_name = peripheral
                .properties()
                .await
                .ok()
                .flatten()
                .and_then(|p| p.local_name);

            match &device_name {
                Some(name) => {
                    tracing::info!("{}", fl!("core-connected-named", name = name.clone()))
                }
                None => tracing::info!("{}", fl!("core-connected")),
            }
            tracing::info!("{}", fl!("core-subscribed"));

            // CCCD writes are acknowledged before the firmware is necessarily
            // ready to emit on them; the first command must not race that.
            time::sleep(SUBSCRIBE_SETTLE).await;

            Ok(DeviceHandle {
                peripheral,
                write_char: setup.write_char,
                write_type: setup.write_type,
                notify_char: setup.notify_char,
                battery_char: setup.battery_char,
                version_char: setup.version_char,
                touch_action_char: setup.touch_action_char,
                device_name,
            })
        }
        Err(e) => {
            let _ = peripheral.disconnect().await;
            Err(e)
        }
    }
}

/// Attempts connection, retrying on immediate drops.
async fn connect_with_retry(peripheral: &Peripheral) -> Result<(), CoreError> {
    let mut last_err = None;

    for attempt in 0..CONNECT_RETRIES {
        if attempt > 0 {
            tracing::info!(
                "BLE link dropped, retrying ({}/{CONNECT_RETRIES})",
                attempt + 1
            );
            time::sleep(Duration::from_millis(500 * u64::from(attempt))).await;
        }

        let connected = guard(budget::CONNECT, "connect", async {
            peripheral.connect().await?;
            Ok(())
        })
        .await;

        if let Err(e) = connected {
            last_err = Some(e);
            continue;
        }

        match settle_link(peripheral).await {
            Ok(true) => return Ok(()),
            Ok(false) => last_err = Some(CoreError::ConnectionDropped),
            Err(e) => last_err = Some(e),
        }
    }

    Err(last_err.unwrap_or(CoreError::ConnectionDropped))
}

/// Holds a fresh link under observation for [`CONNECT_SETTLE`].
///
/// The window itself is the Windows workaround and is kept in full on the happy
/// path. Polling it rather than sleeping through it means a link that drops at
/// 50ms starts its retry immediately instead of after the whole window.
async fn settle_link(peripheral: &Peripheral) -> Result<bool, CoreError> {
    let deadline = time::Instant::now() + CONNECT_SETTLE;

    loop {
        time::sleep(CONNECT_SETTLE_POLL).await;

        let alive = guard(budget::GATT_OP, "is_connected", async {
            Ok(peripheral.is_connected().await?)
        })
        .await?;

        if !alive {
            return Ok(false);
        }
        if time::Instant::now() >= deadline {
            return Ok(true);
        }
    }
}

/// Discovers characteristics, filtering strictly by `SERVICE_UUID`.
async fn discover_characteristics_with_retry(
    peripheral: &Peripheral,
) -> Result<Vec<Characteristic>, CoreError> {
    let mut last_err = None;

    for attempt in 0..DISCOVERY_RETRIES {
        if attempt > 0 {
            tracing::info!(
                "Service discovery retrying ({}/{DISCOVERY_RETRIES})",
                attempt + 1
            );
            time::sleep(Duration::from_millis(500 * u64::from(attempt))).await;
        }

        let discovered = guard(budget::DISCOVERY, "discover_services", async {
            peripheral.discover_services().await?;
            Ok(())
        })
        .await;

        match discovered {
            Ok(()) => {
                if peripheral.services().iter().any(|s| s.uuid == SERVICE_UUID) {
                    let chars: Vec<Characteristic> = peripheral
                        .characteristics()
                        .into_iter()
                        .filter(|c| c.service_uuid == SERVICE_UUID)
                        .collect();
                    if !chars.is_empty() {
                        return Ok(chars);
                    }
                }
                last_err = Some(CoreError::ServiceNotFound);
            }
            Err(e) => last_err = Some(e),
        }
    }

    Err(last_err.unwrap_or(CoreError::ServiceNotFound))
}

struct SetupResult {
    write_char: Characteristic,
    write_type: WriteType,
    notify_char: Characteristic,
    battery_char: Option<Characteristic>,
    version_char: Option<Characteristic>,
    touch_action_char: Option<Characteristic>,
}

async fn setup_device(peripheral: &Peripheral) -> Result<SetupResult, CoreError> {
    let chars = discover_characteristics_with_retry(peripheral).await?;

    let notify_char = chars
        .iter()
        .find(|c| c.uuid == NOTIFY_UUID)
        .ok_or_else(|| CoreError::CharacteristicNotFound("notify (00001002)".into()))?
        .clone();

    let write_char = chars
        .iter()
        .find(|c| c.uuid == COMMAND_UUID)
        .ok_or_else(|| CoreError::CharacteristicNotFound("command (00001001)".into()))?
        .clone();

    let battery_char = chars.iter().find(|c| c.uuid == BATTERY_UUID).cloned();
    if battery_char.is_none() {
        tracing::debug!("battery characteristic (00000008) not found");
    }

    let version_char = chars.iter().find(|c| c.uuid == VERSION_UUID).cloned();
    if version_char.is_none() {
        tracing::debug!("version characteristic (00000007) not found");
    }

    let touch_action_char = chars.iter().find(|c| c.uuid == TOUCH_ACTION_UUID).cloned();
    if touch_action_char.is_none() {
        tracing::debug!("touch action characteristic (0000000d) not found");
    }

    let write_type = if write_char
        .properties
        .contains(CharPropFlags::WRITE_WITHOUT_RESPONSE)
    {
        WriteType::WithoutResponse
    } else {
        WriteType::WithResponse
    };

    let _ = peripheral.unsubscribe(&notify_char).await;
    guard(budget::GATT_OP, "subscribe_notify", async {
        peripheral.subscribe(&notify_char).await?;
        Ok(())
    })
    .await?;

    for characteristic in &chars {
        if characteristic.uuid == NOTIFY_UUID {
            continue;
        }
        if !characteristic.properties.contains(CharPropFlags::NOTIFY) {
            continue;
        }

        let _ = peripheral.unsubscribe(characteristic).await;

        let subscribed = guard(budget::GATT_OP, "subscribe_secondary", async {
            peripheral.subscribe(characteristic).await?;
            Ok(())
        })
        .await;

        if let Err(e) = subscribed {
            tracing::debug!(uuid = %characteristic.uuid, error = %e, "failed to subscribe to secondary notify characteristic");
        }
    }

    Ok(SetupResult {
        write_char,
        write_type,
        notify_char,
        battery_char,
        version_char,
        touch_action_char,
    })
}

/// Executes a block and guarantees disconnection afterwards.
async fn finish<T>(handle: DeviceHandle, result: Result<T, CoreError>) -> Result<T, CoreError> {
    // The margin every `WriteWithoutResponse` used to pay, charged once, here,
    // where it is actually needed: the stack must flush before the teardown.
    time::sleep(DISCONNECT_FLUSH).await;

    // A failed disconnect is operational noise, never the command's verdict.
    // Returning it told the user a write had failed after it had applied.
    if let Err(e) = handle.disconnect().await {
        tracing::warn!(error = %e, "disconnect failed after command");
    }

    result
}

fn print_anc_confirmation(confirmation: AncConfirmation, mode: u8, sub_scene: u8, noise_value: u8) {
    match confirmation {
        AncConfirmation::Applied => println!(
            "{}",
            fl!(
                "cli-anc-set",
                mode = mode.to_string(),
                sub_scene = sub_scene.to_string(),
                noise_value = noise_value.to_string()
            )
        ),
        AncConfirmation::Rejected => println!("{}", fl!("cli-anc-unconfirmed")),
        AncConfirmation::EchoedOnly => println!("{}", fl!("cli-anc-echoed")),
        AncConfirmation::NoResponse => println!("{}", fl!("cli-anc-timeout")),
    }
}

/// Connects, writes ANC scene, waits for confirmation, and disconnects.
pub async fn set_anc_scene(scene: AncScene) -> Result<(), CoreError> {
    let handle = connect().await?;

    let result = async {
        let (mode, sub_scene, noise_value) = scene.triplet();
        let cmd = command::anc_scene(scene);

        let confirmation = handle.send_anc_and_confirm(cmd).await?;
        print_anc_confirmation(confirmation, mode, sub_scene, noise_value);
        Ok(())
    }
    .await;

    finish(handle, result).await
}

/// Connects, reads battery status, and disconnects.
pub async fn get_battery() -> Result<BatteryStatus, CoreError> {
    let handle = connect().await?;
    let result = handle.read_battery().await;
    finish(handle, result).await
}

/// Device info including name and firmware version.
#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub name: Option<String>,
    pub firmware: FirmwareVersion,
}

/// Connects, reads device info, and disconnects.
pub async fn get_version() -> Result<DeviceInfo, CoreError> {
    let handle = connect().await?;
    let result = handle.read_version().await.map(|firmware| DeviceInfo {
        name: handle.device_name().map(str::to_string),
        firmware,
    });
    finish(handle, result).await
}

/// Connects, sets channel balance, and disconnects.
pub async fn set_balance(value: u8) -> Result<(), CoreError> {
    let handle = connect().await?;
    let result = handle.set_balance(value).await;
    finish(handle, result).await
}

/// Connects, resets to defaults, and disconnects.
pub async fn reset_default() -> Result<(), CoreError> {
    let handle = connect().await?;
    let result = handle.reset_default().await;
    finish(handle, result).await
}

/// Connects, applies factory reset, and disconnects.
pub async fn factory_reset() -> Result<(), CoreError> {
    let handle = connect().await?;
    let result = handle.factory_reset().await;
    finish(handle, result).await
}

/// Connects, sets Bluetooth pairing name, and disconnects.
pub async fn set_name(name: &str) -> Result<(), CoreError> {
    let handle = connect().await?;
    let result = handle.set_name(name).await;
    finish(handle, result).await
}

/// Connects, sets equalizer preset, and disconnects.
pub async fn set_eq_preset(preset: EqPreset) -> Result<(), CoreError> {
    let handle = connect().await?;
    let result = handle.set_eq_preset(preset).await;
    finish(handle, result).await
}

/// Connects, applies a custom (per-band) equalizer curve, and disconnects.
pub async fn set_eq_custom(gains_db: [i16; crate::eq::CUSTOM_BAND_COUNT]) -> Result<(), CoreError> {
    let handle = connect().await?;
    let result = handle.set_eq_custom(gains_db).await;
    finish(handle, result).await
}

/// Connects, sets notification volume, and disconnects.
pub async fn set_notification_volume(level: NotificationVolume) -> Result<(), CoreError> {
    let handle = connect().await?;
    let result = handle.set_notification_volume(level).await;
    finish(handle, result).await
}

/// Connects, reads notification volume, and disconnects.
pub async fn get_notification_volume() -> Result<Option<NotificationVolume>, CoreError> {
    let handle = connect().await?;
    let result = handle.get_notification_volume().await;
    finish(handle, result).await
}

/// Connects, sets the scheduled power-off timer, and disconnects.
pub async fn set_scheduled_power_off(value: ScheduledPowerOff) -> Result<(), CoreError> {
    let handle = connect().await?;
    let result = handle.set_scheduled_power_off(value).await;
    finish(handle, result).await
}

/// Connects, reads the scheduled power-off timer, and disconnects.
pub async fn get_scheduled_power_off() -> Result<Option<ScheduledPowerOff>, CoreError> {
    let handle = connect().await?;
    let result = handle.get_scheduled_power_off().await;
    finish(handle, result).await
}

/// Connects, sets the power-off-after-disconnect timer, and disconnects.
pub async fn set_disconnect_power_off(value: DisconnectPowerOff) -> Result<(), CoreError> {
    let handle = connect().await?;
    let result = handle.set_disconnect_power_off(value).await;
    finish(handle, result).await
}

/// Connects, reads the power-off-after-disconnect timer, and disconnects.
pub async fn get_disconnect_power_off() -> Result<Option<DisconnectPowerOff>, CoreError> {
    let handle = connect().await?;
    let result = handle.get_disconnect_power_off().await;
    finish(handle, result).await
}

/// Connects, sets wear detection and its ANC-on-wear sub-toggle, and disconnects.
pub async fn set_wear_detection(state: WearDetection) -> Result<(), CoreError> {
    let handle = connect().await?;
    let result = handle.set_wear_detection(state).await;
    finish(handle, result).await
}

/// Connects, reads wear-detection state, and disconnects.
pub async fn get_wear_detection() -> Result<Option<WearDetection>, CoreError> {
    let handle = connect().await?;
    let result = handle.get_wear_detection().await;
    finish(handle, result).await
}

/// Connects, sets game mode, and disconnects.
pub async fn set_game_mode(state: GameMode) -> Result<(), CoreError> {
    let handle = connect().await?;
    let result = handle.set_game_mode(state).await;
    finish(handle, result).await
}

/// Connects, reads game-mode state, and disconnects.
pub async fn get_game_mode() -> Result<Option<GameMode>, CoreError> {
    let handle = connect().await?;
    let result = handle.get_game_mode().await;
    finish(handle, result).await
}

/// Connects, assigns a touch action, and disconnects.
pub async fn set_touch_action(
    control: touch_action::TouchControl,
    action: touch_action::TouchAction,
) -> Result<(), CoreError> {
    let handle = connect().await?;
    let result = handle.set_touch_action(control, action).await;
    finish(handle, result).await
}

/// Connects, reads the touch-action map, and disconnects.
pub async fn read_touch_actions() -> Result<touch_action::TouchActionMap, CoreError> {
    let handle = connect().await?;
    let result = handle.read_touch_actions().await;
    finish(handle, result).await
}

/// Connects, sets the LDAC toggle, and disconnects.
pub async fn set_ldac(state: Ldac) -> Result<(), CoreError> {
    let handle = connect().await?;
    let result = handle.set_ldac(state).await;
    finish(handle, result).await
}

/// Connects, reads the LDAC toggle, and disconnects.
pub async fn get_ldac() -> Result<Option<Ldac>, CoreError> {
    let handle = connect().await?;
    let result = handle.get_ldac().await;
    finish(handle, result).await
}

/// Connects, sets the multipoint toggle, and disconnects.
pub async fn set_multipoint(state: Multipoint) -> Result<(), CoreError> {
    let handle = connect().await?;
    let result = handle.set_multipoint(state).await;
    finish(handle, result).await
}

/// Connects, reads the multipoint toggle, and disconnects.
pub async fn get_multipoint() -> Result<Option<Multipoint>, CoreError> {
    let handle = connect().await?;
    let result = handle.get_multipoint().await;
    finish(handle, result).await
}

/// Connects, sets sleep mode, and disconnects.
pub async fn set_sleep_mode(state: SleepMode) -> Result<(), CoreError> {
    let handle = connect().await?;
    let result = handle.set_sleep_mode(state).await;
    finish(handle, result).await
}

/// Connects, reads sleep-mode state, and disconnects.
pub async fn get_sleep_mode() -> Result<Option<SleepMode>, CoreError> {
    let handle = connect().await?;
    let result = handle.get_sleep_mode().await;
    finish(handle, result).await
}
