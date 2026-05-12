/// Argos keyboard info returned by `get_kb_info`.
#[derive(Debug, Clone)]
pub struct ArgosKbInfo {
    /// Argos protocol version (e.g. 0x0001).
    pub protocol_version: u16,
    /// Number of tap dance entries the firmware supports.
    pub tap_dance_entries: u8,
    /// Number of combo entries the firmware supports.
    pub combo_entries: u8,
    /// Number of keys per combo.
    pub keys_per_combo: u8,
    /// Active theme ID.
    pub theme_id: u8,
    /// QMK keycodes version compatibility (3 bytes).
    pub keycodes_version: [u8; 3],
    /// Whether the welcome message has been displayed.
    pub welcome_displayed: bool,
    /// Global tapping term in milliseconds.
    pub global_tapping_term: u16,
    /// Global combo term in milliseconds.
    pub global_combo_term: u16,
}

/// An Argos combo entry.
#[derive(Debug, Clone, Default)]
pub struct ArgosCombo {
    /// Whether this combo is enabled.
    pub enabled: bool,
    /// The keycode produced when the combo triggers.
    pub keycode: u16,
    /// The trigger keys (up to `ARGOS_KEYS_PER_COMBO`).
    pub keys:    [u16; 4],
}

/// An Argos tap dance entry.
#[derive(Debug, Clone, Default)]
pub struct ArgosTapDance {
    /// Keycode on single tap.
    pub on_tap: u16,
    /// Keycode on hold.
    pub on_hold: u16,
    /// Keycode on double tap.
    pub on_double_tap: u16,
    /// Keycode on tap-then-hold.
    pub on_tap_hold: u16,
    /// Custom tapping term in milliseconds (bit 15 = enabled, bits 0-14 = timing).
    pub custom_tapping_term: u16,
}

/// Type of pointing device attached to the keyboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PointingDeviceType {
    Unknown = 0,
    TrackpadProcyon = 1,
    Trackball = 2,
}

impl From<u8> for PointingDeviceType {
    fn from(v: u8) -> Self {
        match v {
            1 => Self::TrackpadProcyon,
            2 => Self::Trackball,
            _ => Self::Unknown,
        }
    }
}

/// Pointing device information returned by `get_pointing_device_info`.
///
/// The exact layout depends on firmware; we store the raw response bytes
/// and provide accessors for known fields.
#[derive(Debug, Clone)]
pub struct PointingDeviceInfo {
    /// Raw response data (up to 30 bytes of command_data).
    pub raw: [u8; 30],
}

impl PointingDeviceInfo {
    /// The type of pointing device.
    pub fn device_type(&self) -> PointingDeviceType {
        PointingDeviceType::from(self.raw[0])
    }
}

/// A captured keycode from keymap test mode.
#[derive(Debug, Clone)]
pub struct CapturedKeycode {
    /// Whether the key was pressed (true) or released (false).
    pub pressed: bool,
    /// The QMK keycode.
    pub keycode: u16,
}
