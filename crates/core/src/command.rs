//! ANC command builder for the QCY protocol.
//! Written to characteristic `00001001` (`0xFF`-framed).

use crate::protocol::Command;
use serde::{Deserialize, Serialize};

/// Opcodes used by the ANC control flow.
pub mod opcode {
    /// ANC control write.
    pub const ANC_SETTING: u8 = 0x17;

    /// Asynchronous ANC apply-result notification.
    pub const ANC_RESULT: u8 = 0x28;
}

/// Lowest wire-confirmed level for [`TransparencyMode::AmbientSound`].
pub const AMBIENT_LEVEL_MIN: u8 = 1;

/// Highest wire-confirmed level for [`TransparencyMode::AmbientSound`].
pub const AMBIENT_LEVEL_MAX: u8 = 6;

/// A three-level intensity setting used by several [`NoiseCancellingMode`] variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NcLevel {
    One,
    Two,
    Three,
}

impl NcLevel {
    const fn raw(self) -> u8 {
        match self {
            NcLevel::One => 0x00,
            NcLevel::Two => 0x01,
            NcLevel::Three => 0x02,
        }
    }

    const fn from_raw(value: u8) -> Option<Self> {
        match value {
            0x00 => Some(NcLevel::One),
            0x01 => Some(NcLevel::Two),
            0x02 => Some(NcLevel::Three),
            _ => None,
        }
    }
}

/// The "Noise Cancelling" submenu modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NoiseCancellingMode {
    Adaptive,
    Indoor(NcLevel),
    DailyCommute(NcLevel),
    Noisy(NcLevel),
    WindResistance,
}

/// The "Transparency" submenu modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransparencyMode {
    VocalEnhancement,
    /// Ambient sound passthrough. Valid `level` range is
    /// [`AMBIENT_LEVEL_MIN`]`..=`[`AMBIENT_LEVEL_MAX`]; [`anc_scene`] clamps
    /// anything outside it.
    AmbientSound {
        level: u8,
    },
}

/// ANC scenes on opcode `0x17` ([`opcode::ANC_SETTING`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AncScene {
    /// Plain listening, no ANC/transparency processing.
    Normal,
    /// Transparency / ambient sound passthrough.
    Transparency(TransparencyMode),
    /// Noise Cancelling modes.
    NoiseCancelling(NoiseCancellingMode),
}

impl AncScene {
    /// `false` for an ambient-sound level outside the wire-confirmed range.
    pub fn is_valid(self) -> bool {
        match self {
            AncScene::Transparency(TransparencyMode::AmbientSound { level }) => {
                (AMBIENT_LEVEL_MIN..=AMBIENT_LEVEL_MAX).contains(&level)
            }
            _ => true,
        }
    }

    /// Returns the `(mode, sub_scene, noise_value)` triplet for this scene.
    pub const fn triplet(self) -> (u8, u8, u8) {
        match self {
            AncScene::Normal => (0x02, 0x00, 0x00),
            AncScene::Transparency(TransparencyMode::VocalEnhancement) => (0x03, 0x02, 0x00),
            AncScene::Transparency(TransparencyMode::AmbientSound { level }) => (0x03, 0x01, level),
            AncScene::NoiseCancelling(NoiseCancellingMode::Adaptive) => (0x01, 0x05, 0x00),
            AncScene::NoiseCancelling(NoiseCancellingMode::Indoor(level)) => {
                (0x01, 0x01, level.raw())
            }
            AncScene::NoiseCancelling(NoiseCancellingMode::DailyCommute(level)) => {
                (0x01, 0x02, level.raw())
            }
            AncScene::NoiseCancelling(NoiseCancellingMode::Noisy(level)) => {
                (0x01, 0x03, level.raw())
            }
            AncScene::NoiseCancelling(NoiseCancellingMode::WindResistance) => (0x01, 0x04, 0x00),
        }
    }

    /// Parses a `(mode, sub_scene, noise_value)` triplet into an [`AncScene`].
    pub const fn from_triplet(mode: u8, sub_scene: u8, noise_value: u8) -> Option<Self> {
        match (mode, sub_scene, noise_value) {
            (0x02, 0x00, 0x00) => Some(AncScene::Normal),
            (0x03, 0x02, 0x00) => Some(AncScene::Transparency(TransparencyMode::VocalEnhancement)),
            (0x03, 0x01, level) => Some(AncScene::Transparency(TransparencyMode::AmbientSound {
                level,
            })),
            (0x01, 0x05, 0x00) => Some(AncScene::NoiseCancelling(NoiseCancellingMode::Adaptive)),
            (0x01, 0x04, 0x00) => Some(AncScene::NoiseCancelling(
                NoiseCancellingMode::WindResistance,
            )),
            (0x01, 0x01, raw) => match NcLevel::from_raw(raw) {
                Some(level) => Some(AncScene::NoiseCancelling(NoiseCancellingMode::Indoor(
                    level,
                ))),
                None => None,
            },
            (0x01, 0x02, raw) => match NcLevel::from_raw(raw) {
                Some(level) => Some(AncScene::NoiseCancelling(
                    NoiseCancellingMode::DailyCommute(level),
                )),
                None => None,
            },
            (0x01, 0x03, raw) => match NcLevel::from_raw(raw) {
                Some(level) => Some(AncScene::NoiseCancelling(NoiseCancellingMode::Noisy(level))),
                None => None,
            },
            _ => None,
        }
    }
}

/// Builds the ANC-setting write command for a given [`AncScene`].
///
/// An ambient-sound level outside the wire-confirmed range is clamped into
/// it, so no unconfirmed value reaches the firmware.
pub fn anc_scene(scene: AncScene) -> Command {
    let (mode, sub_scene, mut noise_value) = scene.triplet();
    if matches!(
        scene,
        AncScene::Transparency(TransparencyMode::AmbientSound { .. })
    ) {
        noise_value = noise_value.clamp(AMBIENT_LEVEL_MIN, AMBIENT_LEVEL_MAX);
    }
    Command::new(opcode::ANC_SETTING, vec![mode, sub_scene, noise_value])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anc_scene_normal_matches_capture() {
        let cmd = anc_scene(AncScene::Normal);
        assert_eq!(cmd.parameters, vec![0x02, 0x00, 0x00]);
        assert_eq!(cmd.pack(), vec![0xFF, 0x05, 0x17, 0x03, 0x02, 0x00, 0x00]);
    }

    #[test]
    fn anc_scene_vocal_enhancement_matches_capture() {
        let cmd = anc_scene(AncScene::Transparency(TransparencyMode::VocalEnhancement));
        assert_eq!(cmd.parameters, vec![0x03, 0x02, 0x00]);
        assert_eq!(cmd.pack(), vec![0xFF, 0x05, 0x17, 0x03, 0x03, 0x02, 0x00]);
    }

    #[test]
    fn anc_scene_ambient_sound_matches_capture() {
        for level in [0x01, 0x03, 0x06] {
            let cmd = anc_scene(AncScene::Transparency(TransparencyMode::AmbientSound {
                level,
            }));
            assert_eq!(cmd.parameters, vec![0x03, 0x01, level]);
        }
    }

    #[test]
    fn anc_scene_noise_cancelling_submodes_match_capture() {
        use NoiseCancellingMode::*;

        let cases = [
            (Adaptive, [0x01, 0x05, 0x00]),
            (Indoor(NcLevel::One), [0x01, 0x01, 0x00]),
            (Indoor(NcLevel::Two), [0x01, 0x01, 0x01]),
            (Indoor(NcLevel::Three), [0x01, 0x01, 0x02]),
            (DailyCommute(NcLevel::One), [0x01, 0x02, 0x00]),
            (DailyCommute(NcLevel::Two), [0x01, 0x02, 0x01]),
            (DailyCommute(NcLevel::Three), [0x01, 0x02, 0x02]),
            (Noisy(NcLevel::One), [0x01, 0x03, 0x00]),
            (Noisy(NcLevel::Two), [0x01, 0x03, 0x01]),
            (Noisy(NcLevel::Three), [0x01, 0x03, 0x02]),
            (WindResistance, [0x01, 0x04, 0x00]),
        ];

        for (mode, expected_params) in cases {
            let cmd = anc_scene(AncScene::NoiseCancelling(mode));
            assert_eq!(cmd.parameters, expected_params.to_vec());
        }
    }

    #[test]
    fn from_triplet_recovers_known_scenes() {
        assert_eq!(
            AncScene::from_triplet(0x02, 0x00, 0x00),
            Some(AncScene::Normal)
        );
        assert_eq!(
            AncScene::from_triplet(0x03, 0x02, 0x00),
            Some(AncScene::Transparency(TransparencyMode::VocalEnhancement))
        );
        assert_eq!(
            AncScene::from_triplet(0x01, 0x01, 0x02),
            Some(AncScene::NoiseCancelling(NoiseCancellingMode::Indoor(
                NcLevel::Three
            )))
        );
    }

    #[test]
    fn from_triplet_rejects_unknown_level() {
        assert_eq!(AncScene::from_triplet(0x01, 0x01, 0x05), None);
    }

    #[test]
    fn from_triplet_rejects_unknown_values() {
        assert_eq!(AncScene::from_triplet(0x00, 0x00, 0x00), None);
        assert_eq!(AncScene::from_triplet(0xFF, 0xFF, 0xFF), None);
    }

    #[test]
    fn anc_scene_clamps_ambient_level_to_the_confirmed_range() {
        let build = |level| {
            anc_scene(AncScene::Transparency(TransparencyMode::AmbientSound {
                level,
            }))
            .parameters
        };
        assert_eq!(build(0), vec![0x03, 0x01, AMBIENT_LEVEL_MIN]);
        assert_eq!(build(200), vec![0x03, 0x01, AMBIENT_LEVEL_MAX]);
    }

    #[test]
    fn is_valid_rejects_out_of_range_ambient_levels() {
        let ambient = |level| AncScene::Transparency(TransparencyMode::AmbientSound { level });
        assert!(ambient(AMBIENT_LEVEL_MIN).is_valid());
        assert!(ambient(AMBIENT_LEVEL_MAX).is_valid());
        assert!(!ambient(0).is_valid());
        assert!(!ambient(AMBIENT_LEVEL_MAX + 1).is_valid());
        assert!(AncScene::Normal.is_valid());
    }
}
