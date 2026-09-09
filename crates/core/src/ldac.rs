//! LDAC codec toggle.
//! Written to the command characteristic (`00001001`, `0xFF`-framed).
//!
//! Write and read use different encodings: writes take a `0x01`/`0x02`
//! mode selector, the same convention as [`crate::game_mode`], but the
//! device reports state back as a plain `0x00`/`0x01` boolean. Encode and
//! decode are therefore asymmetric.

use crate::protocol::Command;

/// Opcode for the LDAC toggle (read and write).
pub const OPCODE: u8 = 0x23;

/// LDAC codec state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ldac {
    On,
    Off,
}

impl Ldac {
    /// Write-side selector: On = `0x01`, Off = `0x02`.
    const fn raw(self) -> u8 {
        match self {
            Ldac::On => 0x01,
            Ldac::Off => 0x02,
        }
    }

    /// Read-side boolean: Off = `0x00`, On = `0x01` — see the module doc.
    const fn from_raw(value: u8) -> Option<Self> {
        match value {
            0x00 => Some(Ldac::Off),
            0x01 => Some(Ldac::On),
            _ => None,
        }
    }
}

/// Builds the LDAC-set write command.
pub fn set_ldac(state: Ldac) -> Command {
    Command::new(OPCODE, vec![state.raw()])
}

/// Parses an LDAC reading from a query/echo payload.
pub fn parse_ldac(params: &[u8]) -> Option<Ldac> {
    Ldac::from_raw(*params.first()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packs_on() {
        assert_eq!(
            set_ldac(Ldac::On).pack(),
            vec![0xFF, 0x03, 0x23, 0x01, 0x01]
        );
    }

    #[test]
    fn packs_off() {
        assert_eq!(
            set_ldac(Ldac::Off).pack(),
            vec![0xFF, 0x03, 0x23, 0x01, 0x02]
        );
    }

    #[test]
    fn parses_off() {
        assert_eq!(parse_ldac(&[0x00]), Some(Ldac::Off));
    }

    #[test]
    fn parses_on() {
        assert_eq!(parse_ldac(&[0x01]), Some(Ldac::On));
    }

    #[test]
    fn rejects_write_only_off_selector() {
        assert_eq!(parse_ldac(&[0x02]), None);
    }

    #[test]
    fn rejects_empty_payload() {
        assert_eq!(parse_ldac(&[]), None);
    }
}
