//! VIA/Vial keyboard configuration protocol over USB HID.
//!
//! This crate provides device discovery and communication with keyboards
//! running QMK firmware with VIA or Vial enabled.

mod command;
pub mod device;
mod error;
mod keycodes;
pub mod layout;
mod protocol;

pub use command::{
    LightingChannel, LightingProtocol, RgbValueId, ViaCommand, ViaCommandId, VialRgbEffect,
    VialRgbValueId,
};
pub use device::{HidAccessStatus, KeyboardDevice, KeyboardInfo};
pub use error::{ViaError, ViaResult};
pub use keycodes::{all_basic_keycodes, keycode_groups, Keycode, KeycodeCategory, KeycodeGroup};
pub use layout::{KeyPosition, KeyboardLayout};
pub use protocol::{LightingValues, ViaProtocol, VialRgbInfo};

/// VIA HID usage page used to identify VIA-enabled keyboards.
pub const VIA_USAGE_PAGE: u16 = 0xFF60;

/// VIA HID usage ID.
pub const VIA_USAGE: u16 = 0x61;

/// Standard VIA HID report size in bytes.
pub const VIA_REPORT_SIZE: usize = 32;
