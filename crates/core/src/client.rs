//! High-level BLE orchestration for ANC control: device discovery,
//! connection, and command flow.

use crate::balance;
use crate::battery::BatteryStatus;
use crate::command::{self, AncScene, opcode};
use crate::device::profile::{
    BATTERY_UUID, COMMAND_UUID, NOTIFY_UUID, QCY_COMPANY_ID, SERVICE_UUID, VERSION_UUID,
};
use crate::device_actions;
use crate::eq::EqPreset;
use crate::error::CoreError;
use crate::protocol::Command;
use crate::response::{self, Event};
use crate::version::FirmwareVersion;
use btleplug::api::{
    Central, CharPropFlags, Characteristic, Manager as _, Peripheral as _, ScanFilter, WriteType,
};
use btleplug::platform::{Manager, Peripheral};
use futures::stream::StreamExt;
use qcyx_i18n::fl;
use std::time::Duration;
use tokio::time;

/// How long to scan for QCY advertisements.
const SCAN_DURATION: Duration = Duration::from_secs(5);

/// How long to wait for the asynchronous `ANC_RESULT` confirmation.
const ANC_CONFIRM_TIMEOUT: Duration = Duration::from_secs(10);

/// Retries for the initial BLE connect.
const CONNECT_RETRIES: u32 = 3;

/// Retries for service/characteristic discovery.
const DISCOVERY_RETRIES: u32 = 3;

/// Scans nearby BLE advertisements for the QCY manufacturer Company ID.
async fn find_qcy_device() -> Result<Peripheral, CoreError> {
    let manager = Manager::new().await?;
    let adapters = manager.adapters().await?;
    let central = adapters
        .into_iter()
        .next()
        .ok_or(CoreError::AdapterNotFound)?;

    central.start_scan(ScanFilter::default()).await?;
    time::sleep(SCAN_DURATION).await;
    central.stop_scan().await?;

    for peripheral in central.peripherals().await? {
        let Some(properties) = peripheral.properties().await? else {
            continue;
        };

        if properties.manufacturer_data.contains_key(&QCY_COMPANY_ID) {
            return Ok(peripheral);
        }
    }

    Err(CoreError::DeviceNotFound)
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
    device_name: Option<String>,
}

impl DeviceHandle {
    async fn send(&self, cmd: Command) -> Result<(), CoreError> {
        self.peripheral
            .write(&self.write_char, &cmd.pack(), self.write_type)
            .await?;

        time::sleep(Duration::from_millis(300)).await;
        Ok(())
    }

    /// Sends an ANC-setting write and waits for confirmation.
    pub async fn send_anc_and_confirm(&self, cmd: Command) -> Result<AncConfirmation, CoreError> {
        let mut notifications = self.peripheral.notifications().await?;
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
                    let connected = self.peripheral.is_connected().await;
                    tracing::trace!(connected = ?connected, "waiting for ANC confirmation");
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
        self.peripheral.disconnect().await?;
        Ok(())
    }

    /// Checks if the BLE link is active.
    pub async fn is_connected(&self) -> Result<bool, CoreError> {
        Ok(self.peripheral.is_connected().await?)
    }

    /// Reads current battery status.
    pub async fn read_battery(&self) -> Result<BatteryStatus, CoreError> {
        let battery_char = self
            .battery_char
            .as_ref()
            .ok_or_else(|| CoreError::CharacteristicNotFound("battery (00000008)".into()))?;

        let data = self.peripheral.read(battery_char).await?;
        BatteryStatus::parse(&data)
            .ok_or_else(|| CoreError::InvalidPacket("battery payload shorter than 3 bytes".into()))
    }

    /// Reads device settings via a direct GATT read on the notify characteristic.
    pub async fn read_state_sync(&self) -> Result<StateSync, CoreError> {
        let data = self.peripheral.read(&self.notify_char).await?;
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

        let data = self.peripheral.read(version_char).await?;
        FirmwareVersion::parse(&data)
            .ok_or_else(|| CoreError::InvalidPacket("version payload was empty".into()))
    }

    /// Sends a balance-set write.
    pub async fn set_balance(&self, value: u8) -> Result<(), CoreError> {
        self.send(balance::set_balance(value)).await
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
    pub async fn set_name(&self, name: &str) -> Result<(), CoreError> {
        self.send(device_actions::set_name(name)).await
    }

    /// Selects an equalizer preset.
    pub async fn set_eq_preset(&self, preset: EqPreset) -> Result<(), CoreError> {
        self.send(crate::eq::set_preset(preset)).await
    }
}

/// Connects to the device, discovers services, and sets up notifications.
pub async fn connect() -> Result<DeviceHandle, CoreError> {
    println!("{}", fl!("core-connecting"));
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
                Some(name) => println!("{}", fl!("core-connected-named", name = name.clone())),
                None => println!("{}", fl!("core-connected")),
            }
            println!("{}", fl!("core-subscribed"));

            time::sleep(Duration::from_millis(150)).await;

            Ok(DeviceHandle {
                peripheral,
                write_char: setup.write_char,
                write_type: setup.write_type,
                notify_char: setup.notify_char,
                battery_char: setup.battery_char,
                version_char: setup.version_char,
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

        match peripheral.connect().await {
            Ok(()) => {
                time::sleep(Duration::from_millis(400)).await;
                match peripheral.is_connected().await {
                    Ok(true) => return Ok(()),
                    Ok(false) => last_err = Some(CoreError::ConnectionDropped),
                    Err(e) => last_err = Some(CoreError::from(e)),
                }
            }
            Err(e) => last_err = Some(CoreError::from(e)),
        }
    }

    Err(last_err.unwrap_or(CoreError::ConnectionDropped))
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

        match peripheral.discover_services().await {
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
            Err(e) => last_err = Some(CoreError::from(e)),
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

    let write_type = if write_char
        .properties
        .contains(CharPropFlags::WRITE_WITHOUT_RESPONSE)
    {
        WriteType::WithoutResponse
    } else {
        WriteType::WithResponse
    };

    let _ = peripheral.unsubscribe(&notify_char).await;
    peripheral.subscribe(&notify_char).await?;

    for characteristic in &chars {
        if characteristic.uuid == NOTIFY_UUID {
            continue;
        }
        if !characteristic.properties.contains(CharPropFlags::NOTIFY) {
            continue;
        }

        let _ = peripheral.unsubscribe(characteristic).await;
        if let Err(e) = peripheral.subscribe(characteristic).await {
            tracing::debug!(uuid = %characteristic.uuid, error = %e, "failed to subscribe to secondary notify characteristic");
        }
    }

    Ok(SetupResult {
        write_char,
        write_type,
        notify_char,
        battery_char,
        version_char,
    })
}

/// Executes a block and guarantees disconnection afterwards.
async fn finish<T>(handle: DeviceHandle, result: Result<T, CoreError>) -> Result<T, CoreError> {
    let disconnect_result = handle.disconnect().await;
    let value = result?;
    disconnect_result?;
    Ok(value)
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
