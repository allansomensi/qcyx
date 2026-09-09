//! GATT profile constants for the QCY BLE protocol.
//!
//! The device exposes at least one *other* GATT service (`0x7033`) that reuses
//! the same 16-bit characteristic numbers as [`SERVICE_UUID`]. Every characteristic
//! lookup MUST filter by both `characteristic.uuid` AND `characteristic.service_uuid`
//! to avoid silently resolving to the wrong service.

use uuid::Uuid;

/// BLE manufacturer-data Company ID broadcast by QCY earbuds.
pub const QCY_COMPANY_ID: u16 = 0x521c;

/// Main QCY BLE GATT service.
pub const SERVICE_UUID: Uuid = Uuid::from_u128(0x0000a001_0000_1000_8000_00805f9b34fb);

/// Command write characteristic. Main protocol channel, `0xFF`-framed.
/// Uses [`btleplug::api::WriteType::WithoutResponse`].
pub const COMMAND_UUID: Uuid = Uuid::from_u128(0x00001001_0000_1000_8000_00805f9b34fb);

/// Notification characteristic. Receives `0xFF`-framed responses,
/// including the ANC echo and the async `ANC_RESULT` confirmation.
pub const NOTIFY_UUID: Uuid = Uuid::from_u128(0x00001002_0000_1000_8000_00805f9b34fb);

/// Battery-status characteristic. Read directly or via notifications.
/// Payload is 3 raw bytes: `[left, right, case]`. High bit = charging,
/// low 7 bits = percentage. See [`crate::battery`].
pub const BATTERY_UUID: Uuid = Uuid::from_u128(0x00000008_0000_1000_8000_00805f9b34fb);

/// Firmware version characteristic. Direct-read pattern. See [`crate::version`].
pub const VERSION_UUID: Uuid = Uuid::from_u128(0x00000007_0000_1000_8000_00805f9b34fb);

/// Touch-action characteristic. Raw 2-byte `[control, action]` writes —
/// NOT `0xFF`-framed, unlike every other characteristic here. See
/// [`crate::touch_action`].
pub const TOUCH_ACTION_UUID: Uuid = Uuid::from_u128(0x0000000d_0000_1000_8000_00805f9b34fb);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_and_command_uuids_differ() {
        assert_ne!(SERVICE_UUID, COMMAND_UUID);
        assert_ne!(COMMAND_UUID, NOTIFY_UUID);
        assert_ne!(BATTERY_UUID, COMMAND_UUID);
        assert_ne!(BATTERY_UUID, NOTIFY_UUID);
        assert_ne!(VERSION_UUID, BATTERY_UUID);
        assert_ne!(VERSION_UUID, COMMAND_UUID);
    }
}
