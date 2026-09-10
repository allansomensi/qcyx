//! Notification volume control.
//! Written to the command characteristic (`00001001`, `0xFF`-framed).

use crate::protocol::Command;
use serde::{Deserialize, Serialize};

/// Opcode for notification volume (read and write).
pub const OPCODE: u8 = 0x1D;

/// The four levels the official app exposes for notification volume.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NotificationVolume {
    Low,
    Medium,
    High,
    Max,
}

impl NotificationVolume {
    const fn raw(self) -> u8 {
        match self {
            NotificationVolume::Low => 0x04,
            NotificationVolume::Medium => 0x06,
            NotificationVolume::High => 0x08,
            NotificationVolume::Max => 0x0A,
        }
    }

    const fn from_raw(value: u8) -> Option<Self> {
        match value {
            0x04 => Some(NotificationVolume::Low),
            0x06 => Some(NotificationVolume::Medium),
            0x08 => Some(NotificationVolume::High),
            0x0A => Some(NotificationVolume::Max),
            _ => None,
        }
    }
}

/// Builds the notification-volume-set write command.
///
/// Wire shape is `[level, 0x00]` — the trailing byte is always `0x00` on a
/// write; the device echoes back `[level, 0x0F]` on notify.
pub fn set_notification_volume(level: NotificationVolume) -> Command {
    Command::new(OPCODE, vec![level.raw(), 0x00])
}

/// Parses a notification-volume reading from a query/echo payload.
/// Only the first byte carries the level; the second is a status flag.
pub fn parse_notification_volume(params: &[u8]) -> Option<NotificationVolume> {
    NotificationVolume::from_raw(*params.first()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packs_each_level_matching_capture() {
        let cases = [
            (NotificationVolume::Low, 0x04),
            (NotificationVolume::Medium, 0x06),
            (NotificationVolume::High, 0x08),
            (NotificationVolume::Max, 0x0A),
        ];

        for (level, raw) in cases {
            let cmd = set_notification_volume(level);
            assert_eq!(cmd.parameters, vec![raw, 0x00]);
            assert_eq!(cmd.pack(), vec![0xFF, 0x04, 0x1D, 0x02, raw, 0x00]);
        }
    }

    #[test]
    fn parses_query_response_matching_capture() {
        // `ff 04 1d 02 08 0f` — initial state, medium-high level, before any change.
        assert_eq!(
            parse_notification_volume(&[0x08, 0x0F]),
            Some(NotificationVolume::High)
        );
        // `ff 04 1d 02 04 0f` — after setting to the lowest level.
        assert_eq!(
            parse_notification_volume(&[0x04, 0x0F]),
            Some(NotificationVolume::Low)
        );
    }

    #[test]
    fn rejects_unknown_raw_value() {
        assert_eq!(parse_notification_volume(&[0x99, 0x0F]), None);
    }

    #[test]
    fn rejects_empty_payload() {
        assert_eq!(parse_notification_volume(&[]), None);
    }
}
