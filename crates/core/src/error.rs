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

    #[error("{}", fl!("error-connection-dropped"))]
    ConnectionDropped,

    /// A call into the platform BLE stack exceeded its budget.
    ///
    /// Carries the operation label so a hang is attributable to the call that
    /// caused it. See [`crate::timeout`].
    #[error("{}", fl!("error-operation-timeout"))]
    OperationTimeout(&'static str),

    /// A command carries more parameter bytes than the frame's one-byte body
    /// length can express. See [`crate::protocol::MAX_PARAMETERS`].
    #[error("Frame too large: {0} parameter bytes (protocol limit is 253)")]
    FrameTooLarge(usize),

    /// The requested device name was empty once trimmed and stripped of
    /// control characters — see [`crate::device_actions::set_name`].
    #[error("Device name is empty after trimming and removing control characters")]
    InvalidName,
}
