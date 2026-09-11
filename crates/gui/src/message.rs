use crate::state::Tab;
use qcyx_core::battery::BatteryStatus;
use qcyx_core::client::AncConfirmation;
use qcyx_core::command::AncScene;
use qcyx_core::disconnect_power_off::DisconnectPowerOff;
use qcyx_core::eq::EqPreset;
use qcyx_core::game_mode::GameMode;
use qcyx_core::ldac::Ldac;
use qcyx_core::multipoint::Multipoint;
use qcyx_core::notification_volume::NotificationVolume;
use qcyx_core::profile::Profile;
use qcyx_core::scheduled_power_off::ScheduledPowerOff;
use qcyx_core::session::ConnectionInfo;
use qcyx_core::sleep_mode::SleepMode;
use qcyx_core::touch_action::{TouchAction, TouchControl};
use qcyx_core::wear_detection::WearDetection;

/// Which saved-profile library a [`Message`] profile variant targets: the
/// full-device "Profiles" tab, or the Equalizer tab's EQ-only profiles.
/// One shared set of messages/handlers serves both — they differ only in
/// which list they read from/write to and, for `Full`, in also driving
/// the sidebar's active-profile indicator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProfileScope {
    Full,
    Eq,
}

#[derive(Debug, Clone)]
pub enum Message {
    /// The persistent session connected successfully.
    Connected(ConnectionInfo),
    /// Result of a battery status read.
    BatteryResult(Result<BatteryStatus, String>),
    /// Result of a BLE signal-strength (RSSI) read.
    RssiResult(Result<Option<i16>, String>),
    /// The persistent connection was lost.
    Disconnected,
    /// Connection failed.
    ConnectionError(String),
    /// Retry connection.
    Retry,
    /// Request to set an ANC scene.
    SetAnc(AncScene),
    /// Result of an ANC scene set request.
    AncResult(AncScene, Result<AncConfirmation, String>),
    /// Switched sidebar tab.
    TabSelected(Tab),
    /// Selected UI theme.
    ThemeSelected(iced::Theme),
    /// Local channel balance change (slider drag).
    BalanceChanged(u8),
    /// Commit channel balance change (BLE write).
    BalanceCommit(u8),
    /// Result of a balance write.
    BalanceResult(Result<(), String>),
    /// Rename input field changed.
    RenameInputChanged(String),
    /// Toggles the device-name field between disabled/display and
    /// enabled/editable. Pressed while editing, this also submits.
    RenameEditToggled,
    /// Cancels an in-progress rename, discarding the typed input and
    /// reverting the field to disabled/display mode.
    RenameEditCancelled,
    RenameResult(Result<(), String>),
    /// Request reset to defaults (arms on first press, commits on second).
    ResetDefaultPressed,
    ResetDefaultResult(Result<(), String>),
    /// Request factory reset (arms on first press, commits on second).
    FactoryResetPressed,
    FactoryResetResult(Result<(), String>),
    /// Local transparency ambient level change (slider drag).
    TransparencyLevelChanged(u8),
    /// Request equalizer preset.
    SetEqPreset(EqPreset),
    EqPresetResult(EqPreset, Result<(), String>),
    /// Local custom-EQ band gain change (slider drag), by band index (0-9).
    EqCustomBandChanged(usize, i16),
    /// Commit a custom-EQ band gain (BLE write of the full 10-band curve).
    EqCustomBandCommit(usize, i16),
    /// Reset all custom-EQ bands to 0 dB (BLE write of a flat curve).
    EqCustomReset,
    EqCustomResult(Result<(), String>),
    /// In-ear detection toggle flipped (preserves the ANC-on-wear sub-flag).
    WearDetectionToggled(bool),
    WearDetectionResult(WearDetection, Result<(), String>),
    /// Request to set the notification volume.
    SetNotificationVolume(NotificationVolume),
    NotificationVolumeResult(NotificationVolume, Result<(), String>),
    /// Request to set the scheduled power-off timer.
    SetScheduledPowerOff(ScheduledPowerOff),
    ScheduledPowerOffResult(ScheduledPowerOff, Result<(), String>),
    /// Custom scheduled-power-off minutes input field changed.
    ScheduledPowerOffCustomInputChanged(String),
    /// Submit the custom scheduled-power-off minutes value.
    ScheduledPowerOffCustomSubmit,
    /// Request to set the disconnect power-off timer.
    SetDisconnectPowerOff(DisconnectPowerOff),
    DisconnectPowerOffResult(DisconnectPowerOff, Result<(), String>),
    /// Request to set game mode.
    SetGameMode(GameMode),
    GameModeResult(GameMode, Result<(), String>),
    /// Request to set sleep mode.
    SetSleepMode(SleepMode),
    SleepModeResult(SleepMode, Result<(), String>),
    /// Request to set the LDAC toggle.
    SetLdac(Ldac),
    LdacResult(Ldac, Result<(), String>),
    /// Request to set the multipoint toggle.
    SetMultipoint(Multipoint),
    MultipointResult(Multipoint, Result<(), String>),
    /// Request to assign a touch action to one earbud/click-count control.
    SetTouchAction(TouchControl, TouchAction),
    TouchActionResult(TouchControl, TouchAction, Result<(), String>),
    /// Selected UI language (a Fluent language id, e.g. `"pt-BR"`).
    LanguageSelected(String),

    // --- Profiles (shared between the "Profiles" tab and the Equalizer
    // tab's EQ-only profiles — see `ProfileScope`) ---
    /// Apply a profile (built-in or saved) to the device.
    ApplyProfile(ProfileScope, Profile),
    ApplyProfileResult(ProfileScope, String, Result<(), String>),
    /// The "new profile name" text field changed.
    NewProfileNameChanged(ProfileScope, String),
    /// Save the current live settings as a new profile under the typed name.
    SaveCurrentAsProfile(ProfileScope),
    /// Delete a saved (non-built-in) profile by name.
    DeleteProfile(ProfileScope, String),
    /// Export a profile to a JSON file via a native save dialog.
    ExportProfile(ProfileScope, Profile),
    /// `Ok(false)`: the save dialog was cancelled.
    ExportProfileResult(ProfileScope, Result<bool, String>),
    /// Import a profile from a JSON file via a native open dialog.
    ImportProfile(ProfileScope),
    /// `Ok(None)`: the open dialog was cancelled.
    ImportProfileResult(ProfileScope, Result<Option<Profile>, String>),
    /// Clears the active-profile indicator without touching any device
    /// setting — a profile is just a label for "these are the settings I
    /// last applied together", not a device-side mode, so there's nothing
    /// to undo on the device itself.
    ClearActiveProfile,
}
