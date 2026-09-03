//! Channel balance control.
//! Written to the command characteristic (`00001001`, `0xFF`-framed).

use crate::protocol::Command;

/// Opcode for channel balance (read and write).
pub const OPCODE: u8 = 0x16;

/// Builds the balance-set write command.
/// `value` is clamped to `0..=100` (0: full left, 100: full right, 50: centered).
pub fn set_balance(value: u8) -> Command {
    Command::new(OPCODE, vec![value.min(100)])
}

/// Parses a balance value from the payload.
pub fn parse_balance(params: &[u8]) -> Option<u8> {
    params.first().copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packs_centered_value() {
        let cmd = set_balance(50);
        assert_eq!(cmd.parameters, vec![50]);
        assert_eq!(cmd.pack(), vec![0xFF, 0x03, 0x16, 0x01, 50]);
    }

    #[test]
    fn clamps_above_100() {
        assert_eq!(set_balance(200).parameters, vec![100]);
    }

    #[test]
    fn parses_real_capture_reading() {
        assert_eq!(parse_balance(&[0x32]), Some(50));
    }

    #[test]
    fn rejects_empty_payload() {
        assert_eq!(parse_balance(&[]), None);
    }
}
