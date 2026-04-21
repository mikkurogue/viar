/// VIA protocol command IDs.
///
/// Reference: <https://github.com/the-via/app/blob/master/src/utils/hid-message-types.ts>
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ViaCommandId {
    GetProtocolVersion = 0x01,
    GetKeyboardValue = 0x02,
    SetKeyboardValue = 0x03,
    DynamicKeymapGetKeycode = 0x04,
    DynamicKeymapSetKeycode = 0x05,
    DynamicKeymapReset = 0x06,
    CustomSetValue = 0x07,
    CustomGetValue = 0x08,
    CustomSave = 0x09,
    EepromReset = 0x0A,
    BootloaderJump = 0x0B,
    DynamicKeymapMacroGetCount = 0x0C,
    DynamicKeymapMacroGetBufferSize = 0x0D,
    DynamicKeymapMacroGetBuffer = 0x0E,
    DynamicKeymapMacroSetBuffer = 0x0F,
    DynamicKeymapMacroReset = 0x10,
    DynamicKeymapGetLayerCount = 0x11,
    DynamicKeymapGetBuffer = 0x12,
    DynamicKeymapSetBuffer = 0x13,
    DynamicKeymapGetEncoder = 0x14,
    DynamicKeymapSetEncoder = 0x15,

    // Vial extensions
    VialPrefix = 0xFE,

    Unhandled = 0xFF,
}

/// Keyboard value IDs used with GetKeyboardValue / SetKeyboardValue.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum KeyboardValueId {
    Uptime = 0x01,
    LayoutOptions = 0x02,
    SwitchMatrixState = 0x03,
    FirmwareVersion = 0x04,
    DeviceIndication = 0x05,
}

/// Lighting value IDs used with CustomGetValue / CustomSetValue.
/// The first byte after the command ID selects the lighting channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum LightingChannel {
    QmkRgblight = 0x01,
    QmkRgbMatrix = 0x02,
    QmkLed = 0x03,
}

/// RGB lighting value sub-IDs (for rgblight / rgb_matrix).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum RgbValueId {
    Brightness = 0x01,
    Effect = 0x02,
    EffectSpeed = 0x03,
    Color = 0x04, // HSV: hue(u16), sat(u8), val(u8) — but brightness is typically separate
}

impl ViaCommandId {
    pub fn from_u8(v: u8) -> Self {
        match v {
            0x01 => Self::GetProtocolVersion,
            0x02 => Self::GetKeyboardValue,
            0x03 => Self::SetKeyboardValue,
            0x04 => Self::DynamicKeymapGetKeycode,
            0x05 => Self::DynamicKeymapSetKeycode,
            0x06 => Self::DynamicKeymapReset,
            0x07 => Self::CustomSetValue,
            0x08 => Self::CustomGetValue,
            0x09 => Self::CustomSave,
            0x0A => Self::EepromReset,
            0x0B => Self::BootloaderJump,
            0x0C => Self::DynamicKeymapMacroGetCount,
            0x0D => Self::DynamicKeymapMacroGetBufferSize,
            0x0E => Self::DynamicKeymapMacroGetBuffer,
            0x0F => Self::DynamicKeymapMacroSetBuffer,
            0x10 => Self::DynamicKeymapMacroReset,
            0x11 => Self::DynamicKeymapGetLayerCount,
            0x12 => Self::DynamicKeymapGetBuffer,
            0x13 => Self::DynamicKeymapSetBuffer,
            0x14 => Self::DynamicKeymapGetEncoder,
            0x15 => Self::DynamicKeymapSetEncoder,
            0xFE => Self::VialPrefix,
            _ => Self::Unhandled,
        }
    }
}

/// A VIA command to be sent to the keyboard.
#[derive(Debug, Clone)]
pub struct ViaCommand {
    pub id: ViaCommandId,
    pub data: Vec<u8>,
}

impl ViaCommand {
    /// Create a simple command with no extra data.
    pub fn simple(id: ViaCommandId) -> Self {
        Self {
            id,
            data: Vec::new(),
        }
    }

    /// Create a command with additional payload bytes.
    pub fn with_data(id: ViaCommandId, data: &[u8]) -> Self {
        Self {
            id,
            data: data.to_vec(),
        }
    }

    /// Serialize this command into a 32-byte HID report buffer.
    /// Byte 0 is the report ID (0x00), byte 1 is the command ID,
    /// remaining bytes are the payload.
    pub fn to_report(&self) -> [u8; crate::VIA_REPORT_SIZE + 1] {
        let mut buf = [0u8; crate::VIA_REPORT_SIZE + 1];
        buf[0] = 0x00; // HID report ID
        buf[1] = self.id as u8;
        let copy_len = self.data.len().min(crate::VIA_REPORT_SIZE - 1);
        buf[2..2 + copy_len].copy_from_slice(&self.data[..copy_len]);
        buf
    }

    // -- Convenience constructors --

    pub fn get_protocol_version() -> Self {
        Self::simple(ViaCommandId::GetProtocolVersion)
    }

    pub fn get_layer_count() -> Self {
        Self::simple(ViaCommandId::DynamicKeymapGetLayerCount)
    }

    pub fn get_keycode(layer: u8, row: u8, col: u8) -> Self {
        Self::with_data(ViaCommandId::DynamicKeymapGetKeycode, &[layer, row, col])
    }

    pub fn set_keycode(layer: u8, row: u8, col: u8, keycode: u16) -> Self {
        Self::with_data(
            ViaCommandId::DynamicKeymapSetKeycode,
            &[
                layer,
                row,
                col,
                (keycode >> 8) as u8,
                (keycode & 0xFF) as u8,
            ],
        )
    }

    /// Get a chunk of the dynamic keymap buffer.
    /// offset and size are in bytes.
    pub fn get_keymap_buffer(offset: u16, size: u8) -> Self {
        Self::with_data(
            ViaCommandId::DynamicKeymapGetBuffer,
            &[(offset >> 8) as u8, (offset & 0xFF) as u8, size],
        )
    }

    pub fn get_macro_count() -> Self {
        Self::simple(ViaCommandId::DynamicKeymapMacroGetCount)
    }

    pub fn get_macro_buffer_size() -> Self {
        Self::simple(ViaCommandId::DynamicKeymapMacroGetBufferSize)
    }

    // -- Lighting commands --

    /// Get a lighting value. channel = LightingChannel, value_id = RgbValueId.
    pub fn get_lighting_value(channel: u8, value_id: u8) -> Self {
        Self::with_data(ViaCommandId::CustomGetValue, &[channel, value_id])
    }

    /// Set a lighting value. channel + value_id + payload bytes.
    pub fn set_lighting_value(channel: u8, value_id: u8, payload: &[u8]) -> Self {
        let mut data = vec![channel, value_id];
        data.extend_from_slice(payload);
        Self::with_data(ViaCommandId::CustomSetValue, &data)
    }

    /// Save custom values (lighting etc.) to persistent storage.
    pub fn custom_save() -> Self {
        Self::simple(ViaCommandId::CustomSave)
    }
}
