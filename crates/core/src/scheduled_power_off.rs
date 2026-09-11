//! Scheduled power-off timer.
//! Written to the command characteristic (`00001001`, `0xFF`-framed).
//!
//! An idle-independent countdown: once armed the device powers off after the
//! given number of minutes regardless of playback or connection state. The
//! official app offers 15/30/60/90-minute presets plus a free-form custom
//! value; wire-confirmed with all four presets and a custom value of 183.

use crate::protocol::Command;

/// Opcode for the scheduled power-off timer (read and write).
pub const OPCODE: u8 = 0x14;

/// Sentinel `u16` value meaning "disabled".
pub const DISABLED: u16 = 0xFFFF;

/// Smallest arming value accepted from user input; `0` was never observed
/// on the wire.
pub const MIN_MINUTES: u16 = 1;

/// Largest arming value accepted from user input: [`DISABLED`] itself would
/// silently turn the timer off.
pub const MAX_MINUTES: u16 = DISABLED - 1;

/// A scheduled power-off setting: disabled, or armed for a number of minutes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScheduledPowerOff {
    Disabled,
    /// Minutes until power-off. The official app's presets are 15/30/60/90,
    /// plus a free-form custom field. User input is limited to
    /// [`MIN_MINUTES`]`..=`[`MAX_MINUTES`]; the device places no confirmed
    /// limit of its own.
    Minutes(u16),
}

impl ScheduledPowerOff {
    const fn raw(self) -> u16 {
        match self {
            ScheduledPowerOff::Disabled => DISABLED,
            ScheduledPowerOff::Minutes(m) => m,
        }
    }

    const fn from_raw(value: u16) -> Self {
        if value == DISABLED {
            ScheduledPowerOff::Disabled
        } else {
            ScheduledPowerOff::Minutes(value)
        }
    }
}

/// Builds the scheduled-power-off-set write command.
///
/// Wire shape is `[lo, hi, 0x00, 0x00]` (little-endian minutes, then two
/// zero bytes the device fills in on its own echo).
pub fn set_scheduled_power_off(value: ScheduledPowerOff) -> Command {
    let raw = value.raw();
    let [lo, hi] = raw.to_le_bytes();
    Command::new(OPCODE, vec![lo, hi, 0x00, 0x00])
}

/// Parses a scheduled-power-off reading from a query/echo payload.
/// Only the first two bytes (little-endian minutes) are read.
pub fn parse_scheduled_power_off(params: &[u8]) -> Option<ScheduledPowerOff> {
    let lo = *params.first()?;
    let hi = *params.get(1)?;
    Some(ScheduledPowerOff::from_raw(u16::from_le_bytes([lo, hi])))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packs_presets_matching_capture() {
        let cases = [
            (15, [0x0F, 0x00]),
            (30, [0x1E, 0x00]),
            (60, [0x3C, 0x00]),
            (90, [0x5A, 0x00]),
        ];

        for (minutes, [lo, hi]) in cases {
            let cmd = set_scheduled_power_off(ScheduledPowerOff::Minutes(minutes));
            assert_eq!(cmd.parameters, vec![lo, hi, 0x00, 0x00]);
            assert_eq!(cmd.pack(), vec![0xFF, 0x06, 0x14, 0x04, lo, hi, 0x00, 0x00]);
        }
    }

    #[test]
    fn packs_custom_value_matching_capture() {
        // 183 minutes, entered as a custom value in the app.
        let cmd = set_scheduled_power_off(ScheduledPowerOff::Minutes(183));
        assert_eq!(cmd.parameters, vec![0xB7, 0x00, 0x00, 0x00]);
    }

    #[test]
    fn packs_disabled_matching_capture() {
        let cmd = set_scheduled_power_off(ScheduledPowerOff::Disabled);
        assert_eq!(cmd.parameters, vec![0xFF, 0xFF, 0x00, 0x00]);
    }

    #[test]
    fn parses_query_response_matching_capture() {
        // `ff 06 14 04 3c 00 3c 00` — 60-minute timer, echoed twice.
        assert_eq!(
            parse_scheduled_power_off(&[0x3C, 0x00, 0x3C, 0x00]),
            Some(ScheduledPowerOff::Minutes(60))
        );
        // `ff 06 14 04 ff ff 00 00` — disabled.
        assert_eq!(
            parse_scheduled_power_off(&[0xFF, 0xFF, 0x00, 0x00]),
            Some(ScheduledPowerOff::Disabled)
        );
    }

    #[test]
    fn rejects_short_payload() {
        assert_eq!(parse_scheduled_power_off(&[0x3C]), None);
    }

    #[test]
    fn max_minutes_stays_clear_of_the_disabled_sentinel() {
        let cmd = set_scheduled_power_off(ScheduledPowerOff::Minutes(MAX_MINUTES));
        assert_eq!(
            parse_scheduled_power_off(&cmd.parameters),
            Some(ScheduledPowerOff::Minutes(MAX_MINUTES))
        );
    }
}
