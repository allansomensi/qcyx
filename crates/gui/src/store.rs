//! Local persistence for things that live outside the BLE device: the
//! user's chosen UI theme/language, and the saved profile libraries.
//!
//! Everything here is plain JSON under a per-user config directory, with
//! best-effort semantics — a missing or unreadable file just means
//! "nothing saved yet", never a hard error the UI has to surface. Writes
//! are atomic, and a file that fails to parse is moved aside instead of
//! being overwritten by the next save.

use qcyx_core::profile::{MAX_JSON_BYTES, Profile, ProfileError};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

/// Resolves (without creating) the QCYx config directory for the current
/// platform. `None` only when the platform gives us no usable home/config
/// environment variable at all.
fn config_dir() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        std::env::var_os("APPDATA").map(|p| PathBuf::from(p).join("qcyx"))
    }
    #[cfg(target_os = "macos")]
    {
        std::env::var_os("HOME")
            .map(|home| PathBuf::from(home).join("Library/Application Support/qcyx"))
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        if let Some(xdg) = std::env::var_os("XDG_CONFIG_HOME") {
            Some(PathBuf::from(xdg).join("qcyx"))
        } else {
            std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config/qcyx"))
        }
    }
}

fn path_for(file_name: &str) -> Option<PathBuf> {
    config_dir().map(|dir| dir.join(file_name))
}

fn read_json<T: DeserializeOwned>(file_name: &str) -> Option<T> {
    let path = path_for(file_name)?;

    let data = match std::fs::read(&path) {
        Ok(data) => data,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return None,
        Err(e) => {
            tracing::warn!(path = %path.display(), error = %e, "failed to read config file");
            return None;
        }
    };

    match serde_json::from_slice(&data) {
        Ok(value) => Some(value),
        Err(e) => {
            // Left in place, it would be overwritten by the next save, along
            // with the only copy of whatever the user had saved.
            let backup = path.with_extension("json.corrupt");
            let moved = std::fs::rename(&path, &backup).is_ok();
            tracing::warn!(
                path = %path.display(),
                backup = %backup.display(),
                moved,
                error = %e,
                "config file failed to parse and was moved aside"
            );
            None
        }
    }
}

fn write_json<T: Serialize>(file_name: &str, value: &T) -> std::io::Result<()> {
    let path = path_for(file_name).ok_or_else(|| {
        std::io::Error::other("could not resolve a config directory for this platform")
    })?;

    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }

    let data = serde_json::to_vec_pretty(value).map_err(std::io::Error::other)?;

    // Write-then-rename: a crash mid-write leaves the previous file intact
    // instead of a truncated one.
    let tmp = path.with_extension("json.tmp");
    {
        let mut file = std::fs::File::create(&tmp)?;
        file.write_all(&data)?;
        file.sync_all()?;
    }

    std::fs::rename(&tmp, &path)
}

/// Persisted app-level preferences: UI theme and language.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    /// The exact string an `iced::Theme`'s `Display` impl produces (e.g.
    /// `"Nord"`), matched back against `Theme::ALL` on load.
    pub theme: Option<String>,
    /// Fluent language id, e.g. `"en"` or `"pt-BR"`.
    pub language: Option<String>,
}

const SETTINGS_FILE: &str = "settings.json";
const PROFILES_FILE: &str = "profiles.json";
const EQ_PROFILES_FILE: &str = "eq_profiles.json";

/// Loads saved app settings, or defaults (`None`/`None`, meaning "use the
/// system-detected language and the default theme") if nothing was saved yet.
pub fn load_settings() -> AppSettings {
    read_json(SETTINGS_FILE).unwrap_or(AppSettings {
        theme: None,
        language: None,
    })
}

/// Saves app settings. Errors are logged, not surfaced — a failed save
/// here should never interrupt using the app.
pub fn save_settings(settings: &AppSettings) {
    if let Err(e) = write_json(SETTINGS_FILE, settings) {
        tracing::warn!(error = %e, "failed to save app settings");
    }
}

/// Loads the user's saved full-device profiles (built-ins are not stored
/// on disk — they're always reconstructed from [`Profile::built_in`]).
pub fn load_profiles() -> Vec<Profile> {
    read_json(PROFILES_FILE).unwrap_or_default()
}

pub fn save_profiles(profiles: &[Profile]) {
    if let Err(e) = write_json(PROFILES_FILE, &profiles) {
        tracing::warn!(error = %e, "failed to save profiles");
    }
}

/// Loads the user's saved EQ-only profiles.
pub fn load_eq_profiles() -> Vec<Profile> {
    read_json(EQ_PROFILES_FILE).unwrap_or_default()
}

pub fn save_eq_profiles(profiles: &[Profile]) {
    if let Err(e) = write_json(EQ_PROFILES_FILE, &profiles) {
        tracing::warn!(error = %e, "failed to save EQ profiles");
    }
}

/// Opens a native "save file" dialog and writes `profile` to it as JSON.
/// `Ok(false)` means the dialog was cancelled. Blocking (uses the native
/// file-picker APIs), so callers must run this on a blocking thread (e.g.
/// via `tokio::task::spawn_blocking`).
pub fn export_profile_blocking(profile: &Profile) -> Result<bool, String> {
    let json = profile.to_json().map_err(|e| e.to_string())?;

    let default_name = format!("{}.json", sanitize_file_name(&profile.name));
    let Some(path) = rfd::FileDialog::new()
        .add_filter("JSON", &["json"])
        .set_file_name(&default_name)
        .save_file()
    else {
        return Ok(false);
    };

    std::fs::write(path, json).map_err(|e| e.to_string())?;
    Ok(true)
}

/// Opens a native "open file" dialog and parses the chosen file as a
/// validated [`Profile`]. `Ok(None)` means the dialog was cancelled.
/// Blocking — see [`export_profile_blocking`].
pub fn import_profile_blocking() -> Result<Option<Profile>, String> {
    let Some(path) = rfd::FileDialog::new()
        .add_filter("JSON", &["json"])
        .pick_file()
    else {
        return Ok(None);
    };

    let data = read_capped(&path, MAX_JSON_BYTES)?;
    Profile::from_json(&data)
        .map(Some)
        .map_err(|e| e.to_string())
}

/// Reads at most `max_bytes` from `path` as UTF-8. The cap is enforced while
/// reading, so an arbitrarily large file is never loaded whole.
fn read_capped(path: &Path, max_bytes: usize) -> Result<String, String> {
    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;

    let mut data = Vec::new();
    file.take(max_bytes as u64 + 1)
        .read_to_end(&mut data)
        .map_err(|e| e.to_string())?;

    if data.len() > max_bytes {
        return Err(ProfileError::TooLarge.to_string());
    }

    String::from_utf8(data).map_err(|e| ProfileError::Parse(e.to_string()).to_string())
}

/// Strips characters that are awkward or illegal in file names on common
/// platforms, so a profile name can be used directly as a suggested
/// export file name.
fn sanitize_file_name(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' || c == ' ' {
                c
            } else {
                '_'
            }
        })
        .collect();

    let trimmed = cleaned.trim();
    if trimmed.is_empty() {
        "profile".to_string()
    } else {
        trimmed.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_capped_enforces_the_cap() {
        let path =
            std::env::temp_dir().join(format!("qcyx-read-capped-{}.json", std::process::id()));
        std::fs::write(&path, "x".repeat(32)).unwrap();

        let exact = read_capped(&path, 32);
        let over = read_capped(&path, 31);
        let _ = std::fs::remove_file(&path);

        assert_eq!(exact, Ok("x".repeat(32)));
        assert!(over.is_err());
    }
}
