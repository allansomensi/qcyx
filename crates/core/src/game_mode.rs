//! Game (low-latency) mode toggle.
//! Written to the command characteristic (`00001001`, `0xFF`-framed).

use crate::protocol::Command;
use serde::{Deserialize, Serialize};

/// Opcode for game mode (read and write).
pub const OPCODE: u8 = 0x09;

/// Game mode state. Unlike [`crate::wear_detection`]'s `0x00`/`0x01`
/// booleans, this opcode encodes on/off as `0x01`/`0x02` — confirmed by both
/// the connect-time default (`0x02`, off) and the write/echo pair for each
/// toggle direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameMode {
    On,
    Off,
}

impl GameMode {
    const fn raw(self) -> u8 {
        match self {
            GameMode::On => 0x01,
            GameMode::Off => 0x02,
        }
    }

    const fn from_raw(value: u8) -> Option<Self> {
        match value {
            0x01 => Some(GameMode::On),
            0x02 => Some(GameMode::Off),
            _ => None,
        }
    }
}

/// Builds the game-mode-set write command.
pub fn set_game_mode(state: GameMode) -> Command {
    Command::new(OPCODE, vec![state.raw()])
}

/// Parses a game-mode reading from a query/echo payload.
pub fn parse_game_mode(params: &[u8]) -> Option<GameMode> {
    GameMode::from_raw(*params.first()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packs_on_matching_capture() {
        // `ff 03 09 01 01`
        let cmd = set_game_mode(GameMode::On);
        assert_eq!(cmd.parameters, vec![0x01]);
        assert_eq!(cmd.pack(), vec![0xFF, 0x03, 0x09, 0x01, 0x01]);
    }

    #[test]
    fn packs_off_matching_capture() {
        // `ff 03 09 01 02`
        let cmd = set_game_mode(GameMode::Off);
        assert_eq!(cmd.parameters, vec![0x02]);
        assert_eq!(cmd.pack(), vec![0xFF, 0x03, 0x09, 0x01, 0x02]);
    }

    #[test]
    fn parses_default_off_matching_capture() {
        assert_eq!(parse_game_mode(&[0x02]), Some(GameMode::Off));
    }

    #[test]
    fn rejects_unknown_raw_value() {
        assert_eq!(parse_game_mode(&[0x00]), None);
    }

    #[test]
    fn rejects_empty_payload() {
        assert_eq!(parse_game_mode(&[]), None);
    }
}
