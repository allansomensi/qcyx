//! Sleep mode toggle.
//! Written to the command characteristic (`00001001`, `0xFF`-framed).
//!
//! Same `0x01`/`0x02` on/off encoding as [`crate::game_mode`], on a
//! different opcode. The device did not echo a notification for either write
//! in the capture this was confirmed against — [`crate::client`] does not
//! wait for one, unlike ANC's two-stage confirmation.

use crate::protocol::Command;

/// Opcode for sleep mode (read and write).
pub const OPCODE: u8 = 0x10;

/// Sleep mode state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SleepMode {
    On,
    Off,
}

impl SleepMode {
    const fn raw(self) -> u8 {
        match self {
            SleepMode::On => 0x01,
            SleepMode::Off => 0x02,
        }
    }

    const fn from_raw(value: u8) -> Option<Self> {
        match value {
            0x01 => Some(SleepMode::On),
            0x02 => Some(SleepMode::Off),
            _ => None,
        }
    }
}

/// Builds the sleep-mode-set write command.
pub fn set_sleep_mode(state: SleepMode) -> Command {
    Command::new(OPCODE, vec![state.raw()])
}

/// Parses a sleep-mode reading from a query/echo payload.
pub fn parse_sleep_mode(params: &[u8]) -> Option<SleepMode> {
    SleepMode::from_raw(*params.first()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packs_on_matching_capture() {
        // `ff 03 10 01 01`
        let cmd = set_sleep_mode(SleepMode::On);
        assert_eq!(cmd.parameters, vec![0x01]);
        assert_eq!(cmd.pack(), vec![0xFF, 0x03, 0x10, 0x01, 0x01]);
    }

    #[test]
    fn packs_off_matching_capture() {
        // `ff 03 10 01 02`
        let cmd = set_sleep_mode(SleepMode::Off);
        assert_eq!(cmd.parameters, vec![0x02]);
        assert_eq!(cmd.pack(), vec![0xFF, 0x03, 0x10, 0x01, 0x02]);
    }

    #[test]
    fn parses_toggle_values() {
        assert_eq!(parse_sleep_mode(&[0x01]), Some(SleepMode::On));
        assert_eq!(parse_sleep_mode(&[0x02]), Some(SleepMode::Off));
    }

    #[test]
    fn rejects_empty_payload() {
        assert_eq!(parse_sleep_mode(&[]), None);
    }
}
