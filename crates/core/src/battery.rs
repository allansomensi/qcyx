//! Battery status reading and parsing for [`crate::device::profile::BATTERY_UUID`].
//!
//! The payload is 3 raw bytes: `[left, right, case]`.
//! For each byte, the high bit is the charging flag and the low 7 bits are the percentage (0-100).

/// A single component's charge level and charging state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BatteryComponent {
    /// Level from 0 to 100.
    pub level: u8,
    pub charging: bool,
}

/// Battery status for both earbuds and the case.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BatteryStatus {
    pub left: BatteryComponent,
    pub right: BatteryComponent,
    pub case: BatteryComponent,
}

impl BatteryStatus {
    /// Parses the raw 3-byte `[left, right, case]` payload.
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 3 {
            return None;
        }

        let component = |byte: u8| BatteryComponent {
            level: byte & 0x7F,
            charging: byte & 0x80 != 0,
        };

        Some(Self {
            left: component(data[0]),
            right: component(data[1]),
            case: component(data[2]),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_non_charging_levels() {
        let status = BatteryStatus::parse(&[80, 75, 90]).unwrap();
        assert_eq!(
            status.left,
            BatteryComponent {
                level: 80,
                charging: false
            }
        );
        assert_eq!(
            status.right,
            BatteryComponent {
                level: 75,
                charging: false
            }
        );
        assert_eq!(
            status.case,
            BatteryComponent {
                level: 90,
                charging: false
            }
        );
    }

    #[test]
    fn parses_charging_flag_from_high_bit() {
        let status = BatteryStatus::parse(&[0xB7, 50, 0xFF]).unwrap();
        assert_eq!(
            status.left,
            BatteryComponent {
                level: 55,
                charging: true
            }
        );
        assert_eq!(
            status.right,
            BatteryComponent {
                level: 50,
                charging: false
            }
        );
        assert_eq!(
            status.case,
            BatteryComponent {
                level: 127,
                charging: true
            }
        );
    }

    #[test]
    fn rejects_short_payload() {
        assert_eq!(BatteryStatus::parse(&[1, 2]), None);
    }

    #[test]
    fn parses_real_capture_reading() {
        let status = BatteryStatus::parse(&[0x5f, 0x5f, 0x00]).unwrap();
        assert_eq!(
            status.left,
            BatteryComponent {
                level: 95,
                charging: false
            }
        );
        assert_eq!(
            status.right,
            BatteryComponent {
                level: 95,
                charging: false
            }
        );
        assert_eq!(
            status.case,
            BatteryComponent {
                level: 0,
                charging: false
            }
        );
    }
}
