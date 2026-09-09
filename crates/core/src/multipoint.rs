//! Dual-device (multipoint) connection toggle.
//! Written to the command characteristic (`00001001`, `0xFF`-framed).
//!
//! Same write/read asymmetry as [`crate::ldac`]: write selector
//! `0x01`/`0x02`, but the device reports state back as `0x00`/`0x01`.

use crate::protocol::Command;

/// Opcode for the multipoint toggle (read and write).
pub const OPCODE: u8 = 0x24;

/// Dual-device (multipoint) connection state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Multipoint {
    On,
    Off,
}

impl Multipoint {
    /// Write-side selector: On = `0x01`, Off = `0x02`.
    const fn raw(self) -> u8 {
        match self {
            Multipoint::On => 0x01,
            Multipoint::Off => 0x02,
        }
    }

    /// Read-side boolean: Off = `0x00`, On = `0x01`.
    const fn from_raw(value: u8) -> Option<Self> {
        match value {
            0x00 => Some(Multipoint::Off),
            0x01 => Some(Multipoint::On),
            _ => None,
        }
    }
}

/// Builds the multipoint-set write command.
pub fn set_multipoint(state: Multipoint) -> Command {
    Command::new(OPCODE, vec![state.raw()])
}

/// Parses a multipoint reading from a query/echo payload.
pub fn parse_multipoint(params: &[u8]) -> Option<Multipoint> {
    Multipoint::from_raw(*params.first()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packs_on() {
        assert_eq!(
            set_multipoint(Multipoint::On).pack(),
            vec![0xFF, 0x03, 0x24, 0x01, 0x01]
        );
    }

    #[test]
    fn packs_off() {
        assert_eq!(
            set_multipoint(Multipoint::Off).pack(),
            vec![0xFF, 0x03, 0x24, 0x01, 0x02]
        );
    }

    #[test]
    fn parses_off() {
        assert_eq!(parse_multipoint(&[0x00]), Some(Multipoint::Off));
    }

    #[test]
    fn parses_on() {
        assert_eq!(parse_multipoint(&[0x01]), Some(Multipoint::On));
    }

    #[test]
    fn rejects_empty_payload() {
        assert_eq!(parse_multipoint(&[]), None);
    }
}
