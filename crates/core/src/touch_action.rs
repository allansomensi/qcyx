//! Touch (tap) action mapping for L/R earbud gestures.
//! Written to a dedicated raw characteristic (`0000000d`, service `a001`),
//! not the `0xFF`-framed command channel used by every other setting: each
//! write is a bare 2-byte `[control, action]` pair, and a read returns the
//! full assignment table.

/// Which earbud and click count a control slot addresses.
///
/// Wire encoding: `2 * (click_count - 1) + side`, with `side` = `1` for
/// the left earbud and `2` for the right.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TouchControl {
    LeftSingle,
    RightSingle,
    LeftDouble,
    RightDouble,
    LeftTriple,
    RightTriple,
}

impl TouchControl {
    pub const ALL: [TouchControl; 6] = [
        TouchControl::LeftSingle,
        TouchControl::RightSingle,
        TouchControl::LeftDouble,
        TouchControl::RightDouble,
        TouchControl::LeftTriple,
        TouchControl::RightTriple,
    ];

    const fn raw(self) -> u8 {
        match self {
            TouchControl::LeftSingle => 0x01,
            TouchControl::RightSingle => 0x02,
            TouchControl::LeftDouble => 0x03,
            TouchControl::RightDouble => 0x04,
            TouchControl::LeftTriple => 0x05,
            TouchControl::RightTriple => 0x06,
        }
    }

    const fn from_raw(value: u8) -> Option<Self> {
        match value {
            0x01 => Some(TouchControl::LeftSingle),
            0x02 => Some(TouchControl::RightSingle),
            0x03 => Some(TouchControl::LeftDouble),
            0x04 => Some(TouchControl::RightDouble),
            0x05 => Some(TouchControl::LeftTriple),
            0x06 => Some(TouchControl::RightTriple),
            _ => None,
        }
    }
}

/// The action a tap gesture triggers.
///
/// `0x07`/`0x08` exist as slots in the read-back table but are unmapped —
/// likely a long-press gesture, unconfirmed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TouchAction {
    None,
    PlayPause,
    Previous,
    Next,
    VoiceAssistant,
    VolumeUp,
    VolumeDown,
    GameMode,
    Anc,
}

impl TouchAction {
    /// All nine actions, in menu order — for GUI pickers.
    pub const ALL: [TouchAction; 9] = [
        TouchAction::None,
        TouchAction::PlayPause,
        TouchAction::Previous,
        TouchAction::Next,
        TouchAction::VoiceAssistant,
        TouchAction::VolumeUp,
        TouchAction::VolumeDown,
        TouchAction::GameMode,
        TouchAction::Anc,
    ];

    const fn raw(self) -> u8 {
        match self {
            TouchAction::None => 0x00,
            TouchAction::PlayPause => 0x01,
            TouchAction::Previous => 0x02,
            TouchAction::Next => 0x03,
            TouchAction::VoiceAssistant => 0x04,
            TouchAction::VolumeUp => 0x05,
            TouchAction::VolumeDown => 0x06,
            TouchAction::GameMode => 0x07,
            TouchAction::Anc => 0x0b,
        }
    }

    const fn from_raw(value: u8) -> Option<Self> {
        match value {
            0x00 => Some(TouchAction::None),
            0x01 => Some(TouchAction::PlayPause),
            0x02 => Some(TouchAction::Previous),
            0x03 => Some(TouchAction::Next),
            0x04 => Some(TouchAction::VoiceAssistant),
            0x05 => Some(TouchAction::VolumeUp),
            0x06 => Some(TouchAction::VolumeDown),
            0x07 => Some(TouchAction::GameMode),
            0x0b => Some(TouchAction::Anc),
            _ => None,
        }
    }
}

impl std::fmt::Display for TouchAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            TouchAction::None => "No action",
            TouchAction::PlayPause => "Play / Pause",
            TouchAction::Previous => "Previous track",
            TouchAction::Next => "Next track",
            TouchAction::VoiceAssistant => "Voice assistant",
            TouchAction::VolumeUp => "Volume up",
            TouchAction::VolumeDown => "Volume down",
            TouchAction::GameMode => "Game mode",
            TouchAction::Anc => "ANC",
        };
        write!(f, "{label}")
    }
}

/// Builds the raw 2-byte write payload for one control/action assignment.
///
/// Not a [`crate::protocol::Command`] — written verbatim, no `0xFF` frame.
pub fn set_touch_action(control: TouchControl, action: TouchAction) -> [u8; 2] {
    [control.raw(), action.raw()]
}

/// All six control/action assignments, parsed from a raw characteristic read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TouchActionMap {
    pub left_single: Option<TouchAction>,
    pub right_single: Option<TouchAction>,
    pub left_double: Option<TouchAction>,
    pub right_double: Option<TouchAction>,
    pub left_triple: Option<TouchAction>,
    pub right_triple: Option<TouchAction>,
}

impl TouchActionMap {
    /// Parses `[control, action]` pairs from a raw read. Unrecognized
    /// control bytes, including padding and the unmapped `0x07`/`0x08`
    /// slots, are skipped.
    pub fn parse(data: &[u8]) -> Self {
        let mut map = Self::default();

        for pair in data.as_chunks::<2>().0 {
            let Some(control) = TouchControl::from_raw(pair[0]) else {
                continue;
            };
            let action = TouchAction::from_raw(pair[1]);

            match control {
                TouchControl::LeftSingle => map.left_single = action,
                TouchControl::RightSingle => map.right_single = action,
                TouchControl::LeftDouble => map.left_double = action,
                TouchControl::RightDouble => map.right_double = action,
                TouchControl::LeftTriple => map.left_triple = action,
                TouchControl::RightTriple => map.right_triple = action,
            }
        }

        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packs_left_single_none() {
        assert_eq!(
            set_touch_action(TouchControl::LeftSingle, TouchAction::None),
            [0x01, 0x00]
        );
    }

    #[test]
    fn packs_right_triple_anc() {
        assert_eq!(
            set_touch_action(TouchControl::RightTriple, TouchAction::Anc),
            [0x06, 0x0b]
        );
    }

    #[test]
    fn parses_full_table() {
        let data = [
            0x01, 0x01, 0x02, 0x01, 0x03, 0x06, 0x04, 0x05, 0x05, 0x02, 0x06, 0x03, 0x07, 0x00,
            0x08, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        let map = TouchActionMap::parse(&data);
        assert_eq!(map.left_single, Some(TouchAction::PlayPause));
        assert_eq!(map.right_single, Some(TouchAction::PlayPause));
        assert_eq!(map.left_double, Some(TouchAction::VolumeDown));
        assert_eq!(map.right_double, Some(TouchAction::VolumeUp));
        assert_eq!(map.left_triple, Some(TouchAction::Previous));
        assert_eq!(map.right_triple, Some(TouchAction::Next));
    }

    #[test]
    fn parses_updated_table_after_a_write() {
        let data = [
            0x01, 0x00, 0x02, 0x01, 0x03, 0x06, 0x04, 0x05, 0x05, 0x02, 0x06, 0x03, 0x07, 0x00,
            0x08, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        assert_eq!(
            TouchActionMap::parse(&data).left_single,
            Some(TouchAction::None)
        );
    }
}
