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
}
