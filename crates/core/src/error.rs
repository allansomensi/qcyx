use qcyx_i18n::fl;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CoreError {
    #[error("{}", fl!("error-bluetooth-adapter"))]
    AdapterNotFound,

    #[error("Protocol error: {0}")]
    InvalidPacket(String),

    #[error("Btleplug error: {0}")]
    BluetoothError(#[from] btleplug::Error),

    #[error("{}", fl!("error-device-not-found"))]
    DeviceNotFound,

    #[error("{}", fl!("error-service-not-found"))]
    ServiceNotFound,

    #[error("Characteristic not found: {0}")]
    CharacteristicNotFound(String),

    #[error("{}", connection_dropped_message())]
    ConnectionDropped,

    /// No persistent session is open, and the caller must not open one —
    /// see [`crate::session::read_battery`].
    #[error("{}", fl!("error-not-connected"))]
    NotConnected,

    /// A call into the platform BLE stack exceeded its budget.
    ///
    /// Carries the operation label so a hang is attributable to the call that
    /// caused it. See [`crate::timeout`].
    #[error("{}", fl!("error-operation-timeout"))]
    OperationTimeout(&'static str),

    /// A command carries more parameter bytes than the frame's one-byte body
    /// length can express. See [`crate::protocol::MAX_PARAMETERS`].
    #[error(
        "Frame too large: {0} parameter bytes (protocol limit is {max})",
        max = crate::protocol::MAX_PARAMETERS
    )]
    FrameTooLarge(usize),

    /// The requested device name was empty once trimmed and stripped of
    /// control characters — see [`crate::device_actions::set_name`].
    #[error("Device name is empty after trimming and removing control characters")]
    InvalidName,
}

impl CoreError {
    /// `true` when the BLE link itself can no longer be trusted, so a
    /// persistent session must be torn down and rebuilt.
    ///
    /// Validation, parsing, and missing-characteristic errors are not: the
    /// link that produced them is healthy, and reconnecting would only fail
    /// the same way.
    pub fn is_transport(&self) -> bool {
        matches!(
            self,
            CoreError::AdapterNotFound
                | CoreError::BluetoothError(_)
                | CoreError::DeviceNotFound
                | CoreError::ServiceNotFound
                | CoreError::ConnectionDropped
                | CoreError::NotConnected
                | CoreError::OperationTimeout(_)
        )
    }
}

/// The "dropped right after connecting" hint describes a WinRT pairing
/// problem; other platforms get generic wording.
fn connection_dropped_message() -> String {
    if cfg!(windows) {
        fl!("error-connection-dropped")
    } else {
        fl!("error-connection-dropped-generic")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn link_failures_are_transport_errors() {
        assert!(CoreError::ConnectionDropped.is_transport());
        assert!(CoreError::NotConnected.is_transport());
        assert!(CoreError::OperationTimeout("write").is_transport());
        assert!(CoreError::BluetoothError(btleplug::Error::NotConnected).is_transport());
    }

    #[test]
    fn validation_and_parse_failures_keep_the_session() {
        assert!(!CoreError::InvalidName.is_transport());
        assert!(!CoreError::FrameTooLarge(300).is_transport());
        assert!(!CoreError::InvalidPacket("short".into()).is_transport());
        assert!(!CoreError::CharacteristicNotFound("battery".into()).is_transport());
    }

    #[test]
    fn localized_messages_resolve_at_runtime() {
        // `fl!` checks keys at compile time; this catches the runtime side
        // (the embedded FTL actually carrying them).
        for error in [
            CoreError::AdapterNotFound,
            CoreError::DeviceNotFound,
            CoreError::ServiceNotFound,
            CoreError::ConnectionDropped,
            CoreError::NotConnected,
            CoreError::OperationTimeout("write"),
        ] {
            assert!(!error.to_string().contains("No localization"), "{error:?}");
        }
    }
}
