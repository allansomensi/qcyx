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

/// Maximum pairing-name length, in bytes.
///
/// The device's own read of `0x18` returns a fixed 32-byte, null-padded field.
/// Writing past it means writing outside what the firmware allocates for the
/// name, on a device that offers no undo.
pub const MAX_NAME_BYTES: usize = 32;

/// Sanitizes and truncates a name exactly as it will be written to the
/// device: trimmed, stripped of control characters, and cut at a char
/// boundary to [`MAX_NAME_BYTES`].
///
/// Exposed so callers (the CLI in particular, which takes a raw `String`
/// with no input mask) can validate or display the name that will actually
/// reach the device, rather than the untouched argument the user typed.
pub fn sanitize(name: &str) -> String {
    let sanitized: String = name.trim().chars().filter(|c| !c.is_control()).collect();

    let mut end = sanitized.len().min(MAX_NAME_BYTES);
    while !sanitized.is_char_boundary(end) {
        end -= 1;
    }

    sanitized[..end].to_string()
}

/// Builds the device-rename write command.
///
/// The name is trimmed, stripped of control characters, and truncated at a char
/// boundary to [`MAX_NAME_BYTES`].
pub fn set_name(name: &str) -> Command {
    Command::new(opcode::PAIRNAME, sanitize(name).into_bytes())
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
    fn truncates_oversized_name_to_the_firmware_field() {
        let long_name = "a".repeat(300);
        let cmd = set_name(&long_name);
        assert_eq!(cmd.parameters.len(), MAX_NAME_BYTES);
    }

    #[test]
    fn truncates_multibyte_names_at_a_char_boundary() {
        // "ç" is two bytes, so a 17-char name is 34 bytes and must land on 32.
        let cmd = set_name(&"ç".repeat(17));
        assert_eq!(cmd.parameters.len(), MAX_NAME_BYTES);
        assert!(std::str::from_utf8(&cmd.parameters).is_ok());
    }

    #[test]
    fn strips_control_characters_and_surrounding_space() {
        let cmd = set_name("  QCY\u{0}Melo\nBuds  ");
        assert_eq!(cmd.parameters, b"QCYMeloBuds".to_vec());
    }

    #[test]
    fn every_name_stays_framable() {
        assert!(set_name(&"a".repeat(300)).validate().is_ok());
    }

    #[test]
    fn blank_or_control_only_input_sanitizes_to_empty_parameters() {
        // `DeviceHandle::set_name` treats empty `parameters` as the signal to
        // reject the rename instead of writing a blank `PAIRNAME` — this pins
        // the sanitizer's half of that contract.
        assert!(set_name("   ").parameters.is_empty());
        assert!(set_name("\u{0}\u{1}\n\t").parameters.is_empty());
        assert!(set_name("").parameters.is_empty());
    }
}
