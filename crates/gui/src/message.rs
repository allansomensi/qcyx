use crate::state::Tab;
use qcyx_core::battery::BatteryStatus;
use qcyx_core::client::AncConfirmation;
use qcyx_core::command::AncScene;
use qcyx_core::disconnect_power_off::DisconnectPowerOff;
use qcyx_core::eq::EqPreset;
use qcyx_core::game_mode::GameMode;
use qcyx_core::notification_volume::NotificationVolume;
use qcyx_core::scheduled_power_off::ScheduledPowerOff;
use qcyx_core::session::ConnectionInfo;
use qcyx_core::sleep_mode::SleepMode;
use qcyx_core::wear_detection::WearDetection;

#[derive(Debug, Clone)]
pub enum Message {
    /// The persistent session connected successfully.
    Connected(ConnectionInfo),
    /// Result of a battery status read.
    BatteryResult(Result<BatteryStatus, String>),
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
    /// Submit device rename.
    RenameSubmit,
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
}
