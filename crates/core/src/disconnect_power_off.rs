//! Power-off-after-disconnect timer.
//! Written to the command characteristic (`00001001`, `0xFF`-framed).
//!
//! Distinct from [`crate::scheduled_power_off`]: this timer only starts once
//! the earbuds lose the Bluetooth connection, and powers them off after it
//! elapses unless reconnected first. The official app defaults to 5 minutes,
//! with 10/30/60-minute presets and a "never" option; wire-confirmed with
//! all five states.

use crate::protocol::Command;

/// Opcode for the disconnect power-off timer (read and write).
pub const OPCODE: u8 = 0x1F;

/// Sentinel `u16` value meaning "never power off".
pub const NEVER: u16 = 0xFFFF;

/// The device's default on first connect, before any user change.
pub const DEFAULT_MINUTES: u16 = 5;

/// A disconnect power-off setting: never, or a number of minutes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisconnectPowerOff {
    Never,
    /// Minutes after disconnect until power-off. Official app presets are
    /// 5/10/30/60.
    Minutes(u16),
}

impl DisconnectPowerOff {
    const fn raw(self) -> u16 {
        match self {
            DisconnectPowerOff::Never => NEVER,
            DisconnectPowerOff::Minutes(m) => m,
        }
    }

    const fn from_raw(value: u16) -> Self {
        if value == NEVER {
            DisconnectPowerOff::Never
        } else {
            DisconnectPowerOff::Minutes(value)
        }
    }
}

/// Builds the disconnect-power-off-set write command.
///
/// Wire shape is `[lo, hi]` (little-endian minutes) — unlike
/// [`crate::scheduled_power_off`], there is no trailing echo pair.
pub fn set_disconnect_power_off(value: DisconnectPowerOff) -> Command {
    let [lo, hi] = value.raw().to_le_bytes();
    Command::new(OPCODE, vec![lo, hi])
}

/// Parses a disconnect-power-off reading from a query/echo payload.
pub fn parse_disconnect_power_off(params: &[u8]) -> Option<DisconnectPowerOff> {
    let lo = *params.first()?;
    let hi = *params.get(1)?;
    Some(DisconnectPowerOff::from_raw(u16::from_le_bytes([lo, hi])))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packs_presets_matching_capture() {
        let cases = [
            (5, [0x05, 0x00]),
            (10, [0x0A, 0x00]),
            (30, [0x1E, 0x00]),
            (60, [0x3C, 0x00]),
        ];

        for (minutes, [lo, hi]) in cases {
            let cmd = set_disconnect_power_off(DisconnectPowerOff::Minutes(minutes));
            assert_eq!(cmd.parameters, vec![lo, hi]);
            assert_eq!(cmd.pack(), vec![0xFF, 0x04, 0x1F, 0x02, lo, hi]);
        }
    }

    #[test]
    fn packs_never_matching_capture() {
        let cmd = set_disconnect_power_off(DisconnectPowerOff::Never);
        assert_eq!(cmd.parameters, vec![0xFF, 0xFF]);
    }

    #[test]
    fn parses_query_response_matching_capture() {
        // `ff 04 1f 02 05 00` — the connect-time default.
        assert_eq!(
            parse_disconnect_power_off(&[0x05, 0x00]),
            Some(DisconnectPowerOff::Minutes(DEFAULT_MINUTES))
        );
        // `ff 04 1f 02 ff ff` — never.
        assert_eq!(
            parse_disconnect_power_off(&[0xFF, 0xFF]),
            Some(DisconnectPowerOff::Never)
        );
    }

    #[test]
    fn rejects_short_payload() {
        assert_eq!(parse_disconnect_power_off(&[0x05]), None);
    }
}
