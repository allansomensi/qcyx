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
//!
//! Imported files are untrusted: [`Profile::from_json`] checks every field
//! against the range the device accepts before a profile can reach the wire.

use crate::command::AncScene;
use crate::eq::{CUSTOM_GAIN_MAX_DB, CUSTOM_GAIN_MIN_DB, EqPreset};
use crate::game_mode::GameMode;
use crate::notification_volume::NotificationVolume;
use qcyx_i18n::fl;
use serde::{Deserialize, Serialize};

/// Longest accepted profile name, in characters.
pub const MAX_NAME_CHARS: usize = 64;

/// Largest JSON document [`Profile::from_json`] accepts. A real profile is a
/// few hundred bytes; the cap bounds what an arbitrary file can make the app
/// allocate.
pub const MAX_JSON_BYTES: usize = 64 * 1024;

/// Name prefix reserved for [`Profile::built_in`] translation keys.
const BUILT_IN_PREFIX: &str = "built-in-";

/// Why a profile was rejected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProfileError {
    /// Not JSON, or not shaped like a profile.
    Parse(String),
    /// Larger than [`MAX_JSON_BYTES`].
    TooLarge,
    /// Empty once sanitized, or reserved for a built-in profile.
    InvalidName,
    /// Sets no device setting at all.
    Empty,
    /// Sets both a built-in EQ preset and a custom curve.
    EqConflict,
    /// A field is outside the range the device accepts.
    OutOfRange(&'static str),
}

impl std::fmt::Display for ProfileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            ProfileError::Parse(reason) => fl!("profile-error-parse", reason = reason.clone()),
            ProfileError::TooLarge => fl!("profile-error-too-large"),
            ProfileError::InvalidName => fl!("profile-error-name"),
            ProfileError::Empty => fl!("profile-error-empty"),
            ProfileError::EqConflict => fl!("profile-error-eq-conflict"),
            ProfileError::OutOfRange(field) => {
                fl!("profile-error-out-of-range", field = field.to_string())
            }
        };
        f.write_str(&message)
    }
}

impl std::error::Error for ProfileError {}

/// A named, partial snapshot of device settings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub anc_scene: Option<AncScene>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub eq_preset: Option<EqPreset>,
    /// Present only for a custom (per-band) EQ curve; mutually exclusive
    /// with `eq_preset` — see [`Profile::validate_settings`].
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

    /// Returns an EQ-only copy, every non-EQ field cleared.
    pub fn eq_only(self) -> Profile {
        Profile {
            anc_scene: None,
            balance: None,
            notification_volume: None,
            game_mode: None,
            ..self
        }
    }

    /// Sanitizes a user-supplied name: control characters removed,
    /// surrounding whitespace trimmed, capped at [`MAX_NAME_CHARS`].
    /// Idempotent.
    pub fn sanitize_name(name: &str) -> String {
        let stripped: String = name.chars().filter(|c| !c.is_control()).collect();
        let capped: String = stripped.trim().chars().take(MAX_NAME_CHARS).collect();
        capped.trim_end().to_string()
    }

    /// `true` for names reserved for [`Profile::built_in`] translation keys.
    pub fn is_reserved_name(name: &str) -> bool {
        name.starts_with(BUILT_IN_PREFIX)
    }

    /// `true` when the profile sets no device setting.
    pub fn is_empty(&self) -> bool {
        self.anc_scene.is_none()
            && self.eq_preset.is_none()
            && self.eq_custom.is_none()
            && self.balance.is_none()
            && self.notification_volume.is_none()
            && self.game_mode.is_none()
    }

    /// Checks everything that reaches the device: every set field is within
    /// the range the device accepts, a preset and a custom curve aren't both
    /// set, and at least one setting is present.
    pub fn validate_settings(&self) -> Result<(), ProfileError> {
        if self.anc_scene.is_some_and(|scene| !scene.is_valid()) {
            return Err(ProfileError::OutOfRange("anc_scene"));
        }
        if self
            .balance
            .is_some_and(|balance| balance > crate::balance::MAX_VALUE)
        {
            return Err(ProfileError::OutOfRange("balance"));
        }
        if self.eq_custom.is_some_and(|gains| {
            gains
                .iter()
                .any(|gain| !(CUSTOM_GAIN_MIN_DB..=CUSTOM_GAIN_MAX_DB).contains(gain))
        }) {
            return Err(ProfileError::OutOfRange("eq_custom"));
        }
        if self.eq_preset.is_some() && self.eq_custom.is_some() {
            return Err(ProfileError::EqConflict);
        }
        if self.is_empty() {
            return Err(ProfileError::Empty);
        }
        Ok(())
    }

    /// [`Self::validate_settings`], plus the name: non-empty, already
    /// sanitized, and not reserved unless the profile is a built-in.
    pub fn validate(&self) -> Result<(), ProfileError> {
        let name_ok = !self.name.is_empty()
            && self.name == Self::sanitize_name(&self.name)
            && (self.built_in || !Self::is_reserved_name(&self.name));

        if !name_ok {
            return Err(ProfileError::InvalidName);
        }
        self.validate_settings()
    }

    /// Serializes this profile to pretty-printed JSON, for export.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Parses a profile from JSON, for import: size-capped, name sanitized,
    /// then [`Self::validate`]d.
    pub fn from_json(data: &str) -> Result<Profile, ProfileError> {
        if data.len() > MAX_JSON_BYTES {
            return Err(ProfileError::TooLarge);
        }

        let mut profile: Profile =
            serde_json::from_str(data).map_err(|e| ProfileError::Parse(e.to_string()))?;
        profile.name = Self::sanitize_name(&profile.name);
        profile.validate()?;
        Ok(profile)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::TransparencyMode;
    use crate::eq::CUSTOM_BAND_COUNT;

    fn sample() -> Profile {
        Profile::from_current(
            "My profile".into(),
            Some(AncScene::Normal),
            Some(EqPreset::Rock),
            None,
            Some(60),
            Some(NotificationVolume::Low),
            Some(GameMode::Off),
        )
    }

    fn import(profile: &Profile) -> Result<Profile, ProfileError> {
        Profile::from_json(&profile.to_json().unwrap())
    }

    #[test]
    fn built_in_profiles_are_reserved_and_valid() {
        let profiles = Profile::built_in();
        assert!(!profiles.is_empty());
        for profile in profiles {
            assert!(profile.built_in);
            assert!(Profile::is_reserved_name(&profile.name));
            assert_eq!(profile.validate(), Ok(()));
        }
    }

    #[test]
    fn round_trips_through_json() {
        let parsed = import(&sample()).unwrap();

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
        let profile = Profile::from_current_eq("Bass".into(), None, Some([1; CUSTOM_BAND_COUNT]));
        assert_eq!(profile.anc_scene, None);
        assert_eq!(profile.balance, None);
        assert!(profile.eq_custom.is_some());
    }

    #[test]
    fn eq_only_strips_non_eq_fields() {
        let profile = sample().eq_only();
        assert_eq!(profile.eq_preset, Some(EqPreset::Rock));
        assert!(profile.anc_scene.is_none() && profile.balance.is_none());
        assert!(profile.notification_volume.is_none() && profile.game_mode.is_none());
    }

    #[test]
    fn sanitizes_the_name_on_import() {
        let mut profile = sample();
        profile.name = "  Night\u{7} mode ".into();
        assert_eq!(import(&profile).unwrap().name, "Night mode");
    }

    #[test]
    fn sanitize_name_caps_length_and_is_idempotent() {
        let once = Profile::sanitize_name(&format!("{} tail", "n".repeat(MAX_NAME_CHARS - 1)));
        assert_eq!(once, "n".repeat(MAX_NAME_CHARS - 1));
        assert_eq!(Profile::sanitize_name(&once), once);
    }

    #[test]
    fn rejects_blank_and_reserved_names() {
        let mut profile = sample();
        profile.name = " \u{0} ".into();
        assert_eq!(import(&profile), Err(ProfileError::InvalidName));

        profile.name = "built-in-focus".into();
        assert_eq!(import(&profile), Err(ProfileError::InvalidName));
    }

    #[test]
    fn rejects_out_of_range_values() {
        let mut profile = sample();
        profile.anc_scene = Some(AncScene::Transparency(TransparencyMode::AmbientSound {
            level: 9,
        }));
        assert_eq!(import(&profile), Err(ProfileError::OutOfRange("anc_scene")));

        let mut profile = sample();
        profile.balance = Some(101);
        assert_eq!(import(&profile), Err(ProfileError::OutOfRange("balance")));

        let mut profile = sample();
        profile.eq_preset = None;
        profile.eq_custom = Some([CUSTOM_GAIN_MAX_DB + 1; CUSTOM_BAND_COUNT]);
        assert_eq!(import(&profile), Err(ProfileError::OutOfRange("eq_custom")));
    }

    #[test]
    fn rejects_a_preset_and_a_custom_curve_together() {
        let mut profile = sample();
        profile.eq_custom = Some([0; CUSTOM_BAND_COUNT]);
        assert_eq!(import(&profile), Err(ProfileError::EqConflict));
    }

    #[test]
    fn rejects_profiles_without_settings() {
        let profile = Profile::from_current_eq("Empty".into(), None, None);
        assert_eq!(import(&profile), Err(ProfileError::Empty));
    }

    #[test]
    fn rejects_malformed_and_oversized_documents() {
        assert!(matches!(
            Profile::from_json("{"),
            Err(ProfileError::Parse(_))
        ));

        let padded = format!("{{\"name\":\"{}\"}}", "a".repeat(MAX_JSON_BYTES));
        assert_eq!(Profile::from_json(&padded), Err(ProfileError::TooLarge));
    }

    #[test]
    fn error_messages_resolve_at_runtime() {
        for error in [
            ProfileError::Parse("eof".into()),
            ProfileError::TooLarge,
            ProfileError::InvalidName,
            ProfileError::Empty,
            ProfileError::EqConflict,
            ProfileError::OutOfRange("balance"),
        ] {
            assert!(!error.to_string().contains("No localization"), "{error:?}");
        }
    }
}
