//! Simple `0xFF`-framed device actions: reset to default settings, factory
//! reset, and device rename.

use crate::protocol::Command;

/// Opcodes for device actions.
pub mod opcode {
    /// Resets settings to default.
    pub const RESET_DEFAULT: u8 = 0x01;
    /// Factory reset.
    pub const FACTORY_RESET: u8 = 0x03;
    /// Device rename. Opcode + length-prefixed UTF-8 name bytes.
    pub const PAIRNAME: u8 = 0x18;
}

/// Builds the reset-to-default write command.
pub fn reset_default() -> Command {
    Command::new(opcode::RESET_DEFAULT, Vec::new())
}

/// Builds the factory-reset write command.
pub fn factory_reset() -> Command {
    Command::new(opcode::FACTORY_RESET, Vec::new())
}

/// Builds the device-rename write command.
/// `name` is truncated at a char boundary if it exceeds 255 bytes.
pub fn set_name(name: &str) -> Command {
    let mut bytes = name.as_bytes();
    if bytes.len() > 255 {
        let mut end = 255;
        while !name.is_char_boundary(end) {
            end -= 1;
        }
        bytes = &bytes[..end];
    }
    Command::new(opcode::PAIRNAME, bytes.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packs_reset_default() {
        assert_eq!(reset_default().pack(), vec![0xFF, 0x02, 0x01, 0x00]);
    }

    #[test]
    fn packs_factory_reset() {
        assert_eq!(factory_reset().pack(), vec![0xFF, 0x02, 0x03, 0x00]);
    }

    #[test]
    fn packs_name() {
        let cmd = set_name("HT08");
        assert_eq!(cmd.parameters, b"HT08".to_vec());
        assert_eq!(
            cmd.pack(),
            vec![0xFF, 0x06, 0x18, 0x04, b'H', b'T', b'0', b'8']
        );
    }

    #[test]
    fn truncates_oversized_name_at_char_boundary() {
        let long_name = "a".repeat(300);
        let cmd = set_name(&long_name);
        assert_eq!(cmd.parameters.len(), 255);
    }
}
