use std::collections::HashMap;

/// Pointing device / trackpad settings loaded from QMK Settings.
pub struct PointingData {
    /// Available setting IDs on this keyboard
    pub available_settings: Vec<u16>,
    /// Current values (raw bytes) keyed by setting ID
    pub values: HashMap<u16, Vec<u8>>,
    /// Whether we've modified values since last save
    pub dirty: bool,
}

impl PointingData {
    pub fn new(available_settings: Vec<u16>, values: HashMap<u16, Vec<u8>>) -> Self {
        Self {
            available_settings,
            values,
            dirty: false,
        }
    }

    /// Get a u8 setting value.
    pub fn get_u8(&self, id: u16) -> Option<u8> {
        self.values.get(&id).and_then(|v| v.first().copied())
    }

    /// Get a u16 setting value (little-endian).
    pub fn get_u16(&self, id: u16) -> Option<u16> {
        self.values.get(&id).and_then(|v| {
            if v.len() >= 2 {
                Some(u16::from_le_bytes([v[0], v[1]]))
            } else {
                None
            }
        })
    }

    /// Set a u8 setting value.
    pub fn set_u8(&mut self, id: u16, val: u8) {
        self.values.insert(id, vec![val]);
        self.dirty = true;
    }

    /// Set a u16 setting value.
    pub fn set_u16(&mut self, id: u16, val: u16) {
        self.values.insert(id, val.to_le_bytes().to_vec());
        self.dirty = true;
    }

    /// Check if a setting is available.
    pub fn has(&self, id: u16) -> bool {
        self.available_settings.contains(&id)
    }
}
