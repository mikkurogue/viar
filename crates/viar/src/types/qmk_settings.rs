use std::collections::HashMap;

/// QMK Settings data loaded from the keyboard via Vial protocol.
///
/// Holds raw values for all QMK settings the keyboard reports,
/// keyed by setting ID. This includes core settings (IDs 1–27)
/// and pointing device settings (IDs 0x0100+).
pub struct QmkSettingsData {
    /// All setting IDs the keyboard reported as available
    pub available_settings: Vec<u16>,
    /// Current raw values keyed by setting ID
    pub values: HashMap<u16, Vec<u8>>,
    /// Whether any value has been modified since last save/load
    pub dirty: bool,
}

impl QmkSettingsData {
    pub fn new(available_settings: Vec<u16>, values: HashMap<u16, Vec<u8>>) -> Self {
        Self {
            available_settings,
            values,
            dirty: false,
        }
    }

    /// Check if a setting is available on this keyboard.
    pub fn has(&self, id: u16) -> bool {
        self.available_settings.contains(&id)
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

    /// Get a u32 setting value (little-endian).
    pub fn get_u32(&self, id: u16) -> Option<u32> {
        self.values.get(&id).and_then(|v| {
            if v.len() >= 4 {
                Some(u32::from_le_bytes([v[0], v[1], v[2], v[3]]))
            } else {
                None
            }
        })
    }

    /// Set a u8 setting value in local state.
    pub fn set_u8(&mut self, id: u16, val: u8) {
        self.values.insert(id, vec![val]);
        self.dirty = true;
    }

    /// Set a u16 setting value in local state.
    pub fn set_u16(&mut self, id: u16, val: u16) {
        self.values.insert(id, val.to_le_bytes().to_vec());
        self.dirty = true;
    }

    /// Set a u32 setting value in local state.
    pub fn set_u32(&mut self, id: u16, val: u32) {
        self.values.insert(id, val.to_le_bytes().to_vec());
        self.dirty = true;
    }

    /// Get a single bit from a bitfield setting.
    pub fn get_bit(&self, id: u16, bit: u8) -> Option<bool> {
        match bit {
            0..=7 => self.get_u8(id).map(|v| v & (1 << bit) != 0),
            8..=31 => self.get_u32(id).map(|v| v & (1 << bit) != 0),
            _ => None,
        }
    }

    /// Set a single bit in a bitfield setting.
    pub fn set_bit(&mut self, id: u16, bit: u8, enabled: bool) {
        if bit <= 7 {
            let current = self.get_u8(id).unwrap_or(0);
            let new_val = if enabled {
                current | (1 << bit)
            } else {
                current & !(1 << bit)
            };
            self.set_u8(id, new_val);
        } else if bit <= 31 {
            let current = self.get_u32(id).unwrap_or(0);
            let new_val = if enabled {
                current | (1 << bit)
            } else {
                current & !(1 << bit)
            };
            self.set_u32(id, new_val);
        }
    }

    /// Get the raw bytes for a setting, suitable for sending over the wire.
    pub fn get_raw(&self, id: u16) -> Option<&[u8]> {
        self.values.get(&id).map(|v| v.as_slice())
    }
}
