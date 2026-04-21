use via_protocol::LightingProtocol;

/// Lighting state loaded from the device.
pub struct LightingData {
    pub protocol: LightingProtocol,
    pub brightness: u8,
    pub effect_id: u16,
    pub speed: u8,
    pub hue: u8,
    pub saturation: u8,
    /// Supported VialRGB effect IDs (empty for non-VialRGB protocols)
    pub supported_effects: Vec<u16>,
    /// Whether lighting values have been modified since last save
    pub dirty: bool,
}
