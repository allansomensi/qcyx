//! Decoders for notifications received on characteristic `00001002`, and the
//! [`dispatch`] function that turns a parsed [`crate::protocol::Command`]
//! into a typed [`Event`].

use crate::balance;
use crate::command::opcode;

/// Echo of the last ANC-setting write, as reported back by the device.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AncSetting {
    pub mode: u8,
    pub sub_scene: u8,
    pub noise_value: u8,
}

impl AncSetting {
    pub fn parse(params: &[u8]) -> Option<Self> {
        if params.len() < 3 {
            return None;
        }
        Some(Self {
            mode: params[0],
            sub_scene: params[1],
            noise_value: params[2],
        })
    }
}

/// Asynchronous confirmation that an [`opcode::ANC_SETTING`] write was applied.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AncResult {
    pub applied: bool,
}

impl AncResult {
    pub fn parse(params: &[u8]) -> Option<Self> {
        Some(Self {
            applied: *params.first()? == 0x01,
        })
    }
}

/// A decoded device notification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    AncSetting(AncSetting),
    /// Asynchronous confirmation that an ANC scene write was applied.
    AncResult(AncResult),
    /// Channel balance, `0..=100` (`50` = centered). See [`crate::balance`].
    Balance(u8),
    /// Any other notification that wasn't specifically decoded.
    Raw {
        opcode: u8,
        params: Vec<u8>,
    },
}

pub fn dispatch(opcode_byte: u8, params: &[u8]) -> Event {
    let known = match opcode_byte {
        opcode::ANC_SETTING => AncSetting::parse(params).map(Event::AncSetting),
        opcode::ANC_RESULT => AncResult::parse(params).map(Event::AncResult),
        balance::OPCODE => balance::parse_balance(params).map(Event::Balance),
        _ => None,
    };

    known.unwrap_or_else(|| Event::Raw {
        opcode: opcode_byte,
        params: params.to_vec(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dispatches_anc_result_from_capture() {
        assert_eq!(
            dispatch(opcode::ANC_RESULT, &[0x01]),
            Event::AncResult(AncResult { applied: true })
        );
    }

    #[test]
    fn dispatches_anc_setting_from_capture() {
        assert_eq!(
            dispatch(opcode::ANC_SETTING, &[0x03, 0x02, 0x00]),
            Event::AncSetting(AncSetting {
                mode: 0x03,
                sub_scene: 0x02,
                noise_value: 0x00,
            })
        );
    }

    #[test]
    fn falls_back_to_raw_for_unknown_opcode() {
        assert_eq!(
            dispatch(0x99, &[0x07, 0x08]),
            Event::Raw {
                opcode: 0x99,
                params: vec![0x07, 0x08]
            }
        );
    }

    #[test]
    fn dispatches_balance_from_capture() {
        assert_eq!(dispatch(balance::OPCODE, &[0x32]), Event::Balance(50));
    }
}
