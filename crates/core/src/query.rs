//! Generic parameter query ("GET") for the `0xFF`-framed protocol.
//! Written to the command characteristic (`00001001`), answered on the
//! notify characteristic (`00001002`) with a block carrying the requested
//! opcode.
//!
//! Confirmed against a live capture of the official app: every settings
//! screen that isn't covered by [`crate::client::DeviceHandle::read_state_sync`]
//! (notification volume, both power-off timers, wear detection, game mode,
//! sleep mode) is populated by firing one of these per field on screen load.

use crate::protocol::Command;

/// Opcode for a parameter query.
pub const OPCODE: u8 = 0xFE;

/// Builds a query for the current value of `target_opcode`.
///
/// The device replies on notify with a block carrying `target_opcode` and
/// the parameter's current payload — the same shape a write's echo takes.
pub fn request(target_opcode: u8) -> Command {
    Command::new(OPCODE, vec![target_opcode])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packs_query_matching_capture() {
        // `ff 03 fe 01 1d` — query for notification volume (0x1D).
        let cmd = request(0x1D);
        assert_eq!(cmd.parameters, vec![0x1D]);
        assert_eq!(cmd.pack(), vec![0xFF, 0x03, 0xFE, 0x01, 0x1D]);
    }

    #[test]
    fn packs_query_for_pairname_matching_docs() {
        // `ff 03 fe 01 18` — query for the device pairing name (0x18).
        let cmd = request(0x18);
        assert_eq!(cmd.pack(), vec![0xFF, 0x03, 0xFE, 0x01, 0x18]);
    }
}
