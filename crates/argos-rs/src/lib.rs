//! Rust implementation of the Bastard Keyboards Argos protocol.
//!
//! Argos is a custom extension to the VIA protocol used by Bastard Keyboards
//! (Charybdis, Dilemma, etc.) for advanced configuration features like combos,
//! tap dances, pointing device settings, and theming.
//!
//! This crate provides a typed interface over the raw HID protocol, reusing
//! the HID transport from [`via_protocol`].
//!
//! # Usage
//!
//! ```no_run
//! use argos_rs::ArgosProtocol;
//! use via_protocol::{
//!     KeyboardDevice,
//!     device::discover_keyboards,
//! };
//!
//! // After connecting to a keyboard via via-protocol:
//! // let device: KeyboardDevice = ...;
//! // let argos = ArgosProtocol::new(&device);
//! //
//! // // Check if the keyboard supports Argos
//! // if let Some(info) = argos.probe() {
//! //     println!("Argos v{:#06x} detected", info.protocol_version);
//! // }
//! ```

pub mod command;
pub mod error;
pub mod protocol;
pub mod types;

pub use command::ArgosCommandId;
pub use error::{
    ArgosError,
    ArgosResult,
};
pub use protocol::ArgosProtocol;
pub use types::{
    ArgosCombo,
    ArgosKbInfo,
    ArgosTapDance,
    CapturedKeycode,
    PointingDeviceInfo,
    PointingDeviceType,
};

/// Argos command prefix byte. Distinguishes Argos commands from standard VIA commands.
pub const ARGOS_CMD_PREFIX: u8 = 0x90;

/// Current Argos protocol version.
pub const ARGOS_PROTOCOL_VERSION: u16 = 0x0001;

/// Maximum number of combo entries supported by the firmware.
pub const ARGOS_COMBO_ENTRIES: u8 = 16;

/// Number of trigger keys per combo.
pub const ARGOS_KEYS_PER_COMBO: u8 = 4;

/// Maximum number of tap dance entries supported by the firmware.
pub const ARGOS_TAP_DANCE_ENTRIES: u8 = 16;
