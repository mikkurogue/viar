/// Errors that can occur during VIA protocol communication.
#[derive(Debug, thiserror::Error)]
pub enum ViaError {
    /// Failed to initialize or communicate with HID subsystem.
    #[error("HID error: {0}")]
    Hid(String),

    /// The device did not respond or returned an unexpected response.
    #[error("protocol error: {0}")]
    Protocol(String),

    /// The device does not appear to support VIA.
    #[error("device does not support VIA")]
    NotViaDevice,

    /// Timeout waiting for device response.
    #[error("timeout waiting for device response")]
    Timeout,

    /// Invalid keycode value.
    #[error("invalid keycode: {0:#06x}")]
    InvalidKeycode(u16),
}

impl From<hidapi::HidError> for ViaError {
    fn from(e: hidapi::HidError) -> Self {
        ViaError::Hid(e.to_string())
    }
}

pub type ViaResult<T> = Result<T, ViaError>;
