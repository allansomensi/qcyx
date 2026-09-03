use crate::state::Tab;
use qcyx_core::battery::BatteryStatus;
use qcyx_core::client::AncConfirmation;
use qcyx_core::command::AncScene;
use qcyx_core::eq::EqPreset;
use qcyx_core::session::ConnectionInfo;

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
}
