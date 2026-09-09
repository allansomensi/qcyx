//! In-ear wear detection and its "ANC" sub-option.
//! Written to the command characteristic (`00001001`, `0xFF`-framed).
//!
//! Enabling wear detection in the app reveals a second toggle, labelled
//! "ANC", that re-applies the last ANC scene automatically when the earbuds
//! are put back in the ear. Both flags live in the same opcode, wire-
//! confirmed by toggling each independently.

use crate::protocol::Command;

/// Opcode for wear detection (read and write).
pub const OPCODE: u8 = 0x2C;

/// The two flags exposed by opcode `0x2C`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WearDetection {
    /// Main wear-detection toggle.
    pub wear_detection: bool,
    /// The "ANC" sub-toggle revealed once wear detection is on: re-applies
    /// the last ANC scene when the earbuds detect they've been put back in.
    pub anc_on_wear: bool,
}

const fn flag(enabled: bool) -> u8 {
    if enabled { 0x01 } else { 0x00 }
}

/// Builds the wear-detection-set write command.
///
/// Wire shape is `[wear_flag, 0x01, anc_flag]` — the middle byte is a
/// constant reserved value, unaffected by either toggle.
pub fn set_wear_detection(state: WearDetection) -> Command {
    Command::new(
        OPCODE,
        vec![flag(state.wear_detection), 0x01, flag(state.anc_on_wear)],
    )
}

/// Parses a wear-detection reading from a query/echo payload.
/// Only the first and third bytes carry state; the second is reserved and
/// a fourth byte, present on reads, is a status flag.
pub fn parse_wear_detection(params: &[u8]) -> Option<WearDetection> {
    let wear = *params.first()?;
    let anc = *params.get(2)?;
    Some(WearDetection {
        wear_detection: wear != 0x00,
        anc_on_wear: anc != 0x00,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packs_enabling_wear_detection_matching_capture() {
        // `ff 05 2c 03 01 01 01` — main toggle on, ANC sub-toggle already on.
        let cmd = set_wear_detection(WearDetection {
            wear_detection: true,
            anc_on_wear: true,
        });
        assert_eq!(cmd.parameters, vec![0x01, 0x01, 0x01]);
        assert_eq!(cmd.pack(), vec![0xFF, 0x05, 0x2C, 0x03, 0x01, 0x01, 0x01]);
    }

    #[test]
    fn packs_disabling_anc_sub_toggle_matching_capture() {
        // `ff 05 2c 03 01 01 00` — main toggle stays on, ANC sub-toggle off.
        let cmd = set_wear_detection(WearDetection {
            wear_detection: true,
            anc_on_wear: false,
        });
        assert_eq!(cmd.parameters, vec![0x01, 0x01, 0x00]);
    }

    #[test]
    fn packs_disabling_main_toggle_matching_capture() {
        // `ff 05 2c 03 00 01 01` — main toggle off, ANC sub-toggle stays on.
        let cmd = set_wear_detection(WearDetection {
            wear_detection: false,
            anc_on_wear: true,
        });
        assert_eq!(cmd.parameters, vec![0x00, 0x01, 0x01]);
    }

    #[test]
    fn parses_initial_state_matching_capture() {
        // `ff 06 2c 04 00 01 01 00` — connect-time default: wear detection
        // off, ANC sub-toggle already on.
        assert_eq!(
            parse_wear_detection(&[0x00, 0x01, 0x01, 0x00]),
            Some(WearDetection {
                wear_detection: false,
                anc_on_wear: true,
            })
        );
    }

    #[test]
    fn parses_after_enabling_both_matching_capture() {
        // `ff 06 2c 04 01 01 01 00`
        assert_eq!(
            parse_wear_detection(&[0x01, 0x01, 0x01, 0x00]),
            Some(WearDetection {
                wear_detection: true,
                anc_on_wear: true,
            })
        );
    }

    #[test]
    fn rejects_short_payload() {
        assert_eq!(parse_wear_detection(&[0x01, 0x01]), None);
    }
}
