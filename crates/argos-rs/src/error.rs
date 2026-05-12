use thiserror::Error;
use via_protocol::ViaError;

/// Errors that can occur during Argos protocol communication.
#[derive(Debug, Error)]
pub enum ArgosError {
    /// The underlying VIA/HID communication failed.
    #[error("VIA error: {0}")]
    Via(#[from] ViaError),

    /// The device does not support the Argos protocol.
    #[error("device does not support Argos")]
    NotSupported,

    /// A protocol-level error (unexpected response, invalid data, etc.).
    #[error("argos protocol error: {0}")]
    Protocol(String),

    /// An index was out of range.
    #[error("index out of range: {index} (max {max})")]
    IndexOutOfRange { index: u8, max: u8 },
}

pub type ArgosResult<T> = Result<T, ArgosError>;
