//! Configuration profiles: named snapshots of a subset of device settings
//! that can be applied in one action, saved for later, and exported to/
//! imported from JSON for sharing between machines.
//!
//! A [`Profile`] is intentionally sparse — every field is optional, and
//! only the fields that are `Some` are written to the device when the
//! profile is applied. This is what lets the same type serve both
//! full-device profiles (the "Profiles" tab) and EQ-only profiles (the
//! Equalizer tab's "save current EQ" feature): an EQ-only profile is just
//! a `Profile` with every field `None` except `eq_preset`/`eq_custom`.

use crate::command::AncScene;
use crate::eq::EqPreset;
use crate::game_mode::GameMode;
use crate::notification_volume::NotificationVolume;
use serde::{Deserialize, Serialize};

/// A named, partial snapshot of device settings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub anc_scene: Option<AncScene>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub eq_preset: Option<EqPreset>,
    /// Present only for a custom (per-band) EQ curve; mutually exclusive
    /// with `eq_preset` in practice, though nothing enforces that here.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub eq_custom: Option<[i16; crate::eq::CUSTOM_BAND_COUNT]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub balance: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notification_volume: Option<NotificationVolume>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub game_mode: Option<GameMode>,
    /// `true` for the built-in profiles shipped with QCYx — kept out of
    /// user-facing save/delete flows and never persisted to disk (a
    /// built-in is always reconstructed from [`Profile::built_in`], so
    /// serializing this flag would be redundant either way).
    #[serde(default, skip_serializing, skip_deserializing)]
    pub built_in: bool,
}

impl Profile {
    /// The set of profiles QCYx ships with, covering common listening
    /// situations. Every field here maps to a wire-confirmed setting —
    /// nothing here invents new device behavior, it only bundles existing
    /// controls together.
    pub fn built_in() -> Vec<Profile> {
        vec![
            Profile {
                name: "built-in-focus".into(),
                anc_scene: Some(AncScene::NoiseCancelling(
                    crate::command::NoiseCancellingMode::Adaptive,
                )),
                eq_preset: Some(EqPreset::Default),
                eq_custom: None,
                balance: Some(50),
                notification_volume: Some(NotificationVolume::Medium),
                game_mode: Some(GameMode::Off),
                built_in: true,
            },
            Profile {
                name: "built-in-calls".into(),
                anc_scene: Some(AncScene::Transparency(
                    crate::command::TransparencyMode::VocalEnhancement,
                )),
                eq_preset: Some(EqPreset::Soft),
                eq_custom: None,
                balance: Some(50),
                notification_volume: Some(NotificationVolume::Low),
                game_mode: Some(GameMode::Off),
                built_in: true,
            },
            Profile {
                name: "built-in-workout".into(),
                anc_scene: Some(AncScene::NoiseCancelling(
                    crate::command::NoiseCancellingMode::Noisy(crate::command::NcLevel::Two),
                )),
                eq_preset: Some(EqPreset::BassBoost),
                eq_custom: None,
                balance: Some(50),
                notification_volume: Some(NotificationVolume::High),
                game_mode: Some(GameMode::Off),
                built_in: true,
            },
            Profile {
                name: "built-in-gaming".into(),
                anc_scene: Some(AncScene::Normal),
                eq_preset: Some(EqPreset::Popular),
                eq_custom: None,
                balance: Some(50),
                notification_volume: Some(NotificationVolume::Max),
                game_mode: Some(GameMode::On),
                built_in: true,
            },
        ]
    }

    /// Builds a full-device profile snapshot from the current app state,
    /// used when the user saves "what's applied right now" as a profile.
    #[allow(clippy::too_many_arguments)]
    pub fn from_current(
        name: String,
        anc_scene: Option<AncScene>,
        eq_preset: Option<EqPreset>,
        eq_custom: Option<[i16; crate::eq::CUSTOM_BAND_COUNT]>,
        balance: Option<u8>,
        notification_volume: Option<NotificationVolume>,
        game_mode: Option<GameMode>,
    ) -> Profile {
        Profile {
            name,
            anc_scene,
            eq_preset,
            eq_custom,
            balance,
            notification_volume,
            game_mode,
            built_in: false,
        }
    }

    /// Builds an EQ-only profile (used by the Equalizer tab's "save
    /// current EQ" feature) — every non-EQ field stays `None`.
    pub fn from_current_eq(
        name: String,
        eq_preset: Option<EqPreset>,
        eq_custom: Option<[i16; crate::eq::CUSTOM_BAND_COUNT]>,
    ) -> Profile {
        Profile {
            name,
            anc_scene: None,
            eq_preset,
            eq_custom,
            balance: None,
            notification_volume: None,
            game_mode: None,
            built_in: false,
        }
    }

    /// Serializes this profile to pretty-printed JSON, for export.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Parses a profile from JSON, for import.
    pub fn from_json(data: &str) -> Result<Profile, serde_json::Error> {
        serde_json::from_str(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn built_in_profiles_are_non_empty_and_named() {
        let profiles = Profile::built_in();
        assert!(!profiles.is_empty());
        assert!(profiles.iter().all(|p| p.built_in && !p.name.is_empty()));
    }

    #[test]
    fn round_trips_through_json() {
        let profile = Profile::from_current(
            "My profile".into(),
            Some(AncScene::Normal),
            Some(EqPreset::Rock),
            None,
            Some(60),
            Some(NotificationVolume::Low),
            Some(GameMode::Off),
        );

        let json = profile.to_json().unwrap();
        let parsed = Profile::from_json(&json).unwrap();

        assert_eq!(parsed.name, "My profile");
        assert_eq!(parsed.anc_scene, Some(AncScene::Normal));
        assert_eq!(parsed.eq_preset, Some(EqPreset::Rock));
        assert_eq!(parsed.balance, Some(60));
        // `built_in` is deliberately not serialized; a round trip always
        // comes back `false`.
        assert!(!parsed.built_in);
    }

    #[test]
    fn eq_only_profile_leaves_other_fields_none() {
        let profile =
            Profile::from_current_eq("Bass".into(), None, Some([1; crate::eq::CUSTOM_BAND_COUNT]));
        assert_eq!(profile.anc_scene, None);
        assert_eq!(profile.balance, None);
        assert!(profile.eq_custom.is_some());
    }
}
