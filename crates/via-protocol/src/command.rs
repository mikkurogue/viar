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
    QmkBacklight = 0x01,
    QmkRgblight = 0x02,
    QmkRgbMatrix = 0x03,
    QmkAudio = 0x04,
    QmkLedMatrix = 0x05,
}

/// RGB lighting value sub-IDs for stock VIA protocol (v12+, channel-based).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum RgbValueId {
    Brightness = 0x01,
    Effect = 0x02,
    EffectSpeed = 0x03,
    Color = 0x04, // HSV: hue(u16), sat(u8), val(u8) — but brightness is typically separate
}

/// RGB lighting value IDs for Vial firmware (no channel byte, uses rgblight-style IDs).
/// These are used directly as value_id in `[cmd_id, value_id, data...]` messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum VialRgbValueId {
    Brightness = 0x80,
    Effect = 0x81,
    EffectSpeed = 0x82,
    Color = 0x83,
}

/// Whether we're talking to stock VIA or Vial firmware.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightingProtocol {
    /// Stock VIA (v12+): `[cmd, channel, value_id, data...]`
    Via { channel: LightingChannel },
    /// Vial firmware with VIA_QMK_RGB_MATRIX_ENABLE (no VIALRGB):
    /// `[cmd, value_id, data...]` — no channel byte, uses 0x80+ value IDs
    VialLegacy,
    /// VialRGB protocol: uses sub-commands 0x40-0x44, 16-bit effect IDs,
    /// all-in-one set_mode command
    VialRgb,
}

/// Tap Dance entry (10 bytes, matches firmware struct layout).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TapDanceEntry {
    pub on_tap:        u16,
    pub on_hold:       u16,
    pub on_double_tap: u16,
    pub on_tap_hold:   u16,
    pub tapping_term:  u16,
}

impl TapDanceEntry {
    pub fn from_bytes(data: &[u8]) -> Self {
        Self {
            on_tap:        u16::from_le_bytes([data[0], data[1]]),
            on_hold:       u16::from_le_bytes([data[2], data[3]]),
            on_double_tap: u16::from_le_bytes([data[4], data[5]]),
            on_tap_hold:   u16::from_le_bytes([data[6], data[7]]),
            tapping_term:  u16::from_le_bytes([data[8], data[9]]),
        }
    }

    pub fn to_bytes(&self) -> [u8; 10] {
        let mut buf = [0u8; 10];
        buf[0..2].copy_from_slice(&self.on_tap.to_le_bytes());
        buf[2..4].copy_from_slice(&self.on_hold.to_le_bytes());
        buf[4..6].copy_from_slice(&self.on_double_tap.to_le_bytes());
        buf[6..8].copy_from_slice(&self.on_tap_hold.to_le_bytes());
        buf[8..10].copy_from_slice(&self.tapping_term.to_le_bytes());
        buf
    }

    pub fn is_empty(&self) -> bool {
        self.on_tap == 0 && self.on_hold == 0 && self.on_double_tap == 0 && self.on_tap_hold == 0
    }
}

/// Combo entry (10 bytes, matches firmware struct layout).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ComboEntry {
    pub input:  [u16; 4],
    pub output: u16,
}

impl ComboEntry {
    pub fn from_bytes(data: &[u8]) -> Self {
        Self {
            input:  [
                u16::from_le_bytes([data[0], data[1]]),
                u16::from_le_bytes([data[2], data[3]]),
                u16::from_le_bytes([data[4], data[5]]),
                u16::from_le_bytes([data[6], data[7]]),
            ],
            output: u16::from_le_bytes([data[8], data[9]]),
        }
    }

    pub fn to_bytes(&self) -> [u8; 10] {
        let mut buf = [0u8; 10];
        for (i, kc) in self.input.iter().enumerate() {
            buf[i * 2..i * 2 + 2].copy_from_slice(&kc.to_le_bytes());
        }
        buf[8..10].copy_from_slice(&self.output.to_le_bytes());
        buf
    }

    pub fn is_empty(&self) -> bool {
        self.input.iter().all(|&k| k == 0) && self.output == 0
    }
}

/// Key Override entry (10 bytes, matches firmware struct layout).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct KeyOverrideEntry {
    pub trigger: u16,
    pub replacement: u16,
    pub layers: u16,
    pub trigger_mods: u8,
    pub negative_mod_mask: u8,
    pub suppressed_mods: u8,
    pub options: u8, // bit 7 = enabled
}

impl KeyOverrideEntry {
    pub fn from_bytes(data: &[u8]) -> Self {
        Self {
            trigger: u16::from_le_bytes([data[0], data[1]]),
            replacement: u16::from_le_bytes([data[2], data[3]]),
            layers: u16::from_le_bytes([data[4], data[5]]),
            trigger_mods: data[6],
            negative_mod_mask: data[7],
            suppressed_mods: data[8],
            options: data[9],
        }
    }

    pub fn to_bytes(&self) -> [u8; 10] {
        let mut buf = [0u8; 10];
        buf[0..2].copy_from_slice(&self.trigger.to_le_bytes());
        buf[2..4].copy_from_slice(&self.replacement.to_le_bytes());
        buf[4..6].copy_from_slice(&self.layers.to_le_bytes());
        buf[6] = self.trigger_mods;
        buf[7] = self.negative_mod_mask;
        buf[8] = self.suppressed_mods;
        buf[9] = self.options;
        buf
    }

    pub fn is_enabled(&self) -> bool {
        self.options & 0x80 != 0
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        if enabled {
            self.options |= 0x80;
        } else {
            self.options &= !0x80;
        }
    }

    pub fn is_empty(&self) -> bool {
        self.trigger == 0 && self.replacement == 0
    }
}

/// Dynamic entry counts returned by the keyboard.
#[derive(Debug, Clone, Default)]
pub struct DynamicEntryCounts {
    pub tap_dance:    u8,
    pub combo:        u8,
    pub key_override: u8,
    pub alt_repeat:   u8,
}

/// A QMK Settings entry descriptor returned by the query command.
#[derive(Debug, Clone)]
pub struct QmkSettingDescriptor {
    /// Setting ID
    pub id:    u16,
    /// Setting type: 1=u8, 2=u16, 4=u32
    pub qtype: u8,
    /// Human-readable name (if available from Vial definition)
    pub name:  Option<String>,
}

/// Known QMK pointing device settings IDs.
/// These correspond to the Vial QMK Settings protocol for pointing device configuration.
pub mod pointing_settings {
    /// Pointing device DPI (u16)
    pub const DPI: u16 = 0x0100;
    /// Pointing device scroll divisor (u8)  
    pub const SCROLL_DIVISOR: u16 = 0x0101;
    /// Pointing device scroll divisor horizontal (u8)
    pub const SCROLL_DIVISOR_H: u16 = 0x0102;
    /// Pointing device invert X axis (u8, bool)
    pub const INVERT_X: u16 = 0x0103;
    /// Pointing device invert Y axis (u8, bool)
    pub const INVERT_Y: u16 = 0x0104;
    /// Pointing device invert scroll (u8, bool)
    pub const INVERT_SCROLL: u16 = 0x0105;
    /// Drag scroll enable (u8, bool)
    pub const DRAG_SCROLL: u16 = 0x0106;
    /// Drag scroll divisor (u8)
    pub const DRAG_SCROLL_DIVISOR: u16 = 0x0107;
    /// CPI / DPI for second pointing device (u16)
    pub const DPI_2: u16 = 0x0108;
    /// Sniping DPI (u16)
    pub const SNIPING_DPI: u16 = 0x0109;
    /// Auto mouse enable (u8, bool)
    pub const AUTO_MOUSE_ENABLE: u16 = 0x0110;
    /// Auto mouse layer (u8)
    pub const AUTO_MOUSE_LAYER: u16 = 0x0111;
    /// Auto mouse timeout (u16, ms)
    pub const AUTO_MOUSE_TIMEOUT: u16 = 0x0112;
}

/// VialRGB sub-command IDs (used as data[1] with CustomGetValue/CustomSetValue).
/// Note: GET and SET share the same sub-command IDs (0x41, 0x42) but the
/// parent command (CustomGetValue vs CustomSetValue) differentiates them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum VialRgbCmd {
    GetInfo = 0x40,
    GetModeOrSetMode = 0x41,
    GetSupportedOrDirectFastSet = 0x42,
    #[allow(dead_code)]
    GetNumLeds = 0x43,
    #[allow(dead_code)]
    GetLedInfo = 0x44,
}

/// VialRGB effect IDs (16-bit, sequential).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum VialRgbEffect {
    Off = 0,
    Direct = 1,
    SolidColor = 2,
    AlphasMods = 3,
    GradientUpDown = 4,
    GradientLeftRight = 5,
    Breathing = 6,
    BandSat = 7,
    BandVal = 8,
    BandPinwheelSat = 9,
    BandPinwheelVal = 10,
    BandSpiralSat = 11,
    BandSpiralVal = 12,
    CycleAll = 13,
    CycleLeftRight = 14,
    CycleUpDown = 15,
    RainbowMovingChevron = 16,
    CycleOutIn = 17,
    CycleOutInDual = 18,
    CyclePinwheel = 19,
    CycleSpiral = 20,
    DualBeacon = 21,
    RainbowBeacon = 22,
    RainbowPinwheels = 23,
    Raindrops = 24,
    JellybeanRaindrops = 25,
    HueBreathing = 26,
    HuePendulum = 27,
    HueWave = 28,
    TypingHeatmap = 29,
    DigitalRain = 30,
    SolidReactiveSimple = 31,
    SolidReactive = 32,
    SolidReactiveWide = 33,
    SolidReactiveMultiwide = 34,
    SolidReactiveCross = 35,
    SolidReactiveMulticross = 36,
    SolidReactiveNexus = 37,
    SolidReactiveMultinexus = 38,
    Splash = 39,
    Multisplash = 40,
    SolidSplash = 41,
    SolidMultisplash = 42,
    PixelRain = 43,
    PixelFractal = 44,
}

impl VialRgbEffect {
    pub fn from_u16(v: u16) -> Option<Self> {
        if v <= 44 {
            // Safety: all values 0-44 are valid variants
            // Safety: all values 0-44 are valid variants
            #[allow(unsafe_code)]
            Some(unsafe { std::mem::transmute::<u16, VialRgbEffect>(v) })
        } else {
            None
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Off => "Off",
            Self::Direct => "Direct",
            Self::SolidColor => "Solid Color",
            Self::AlphasMods => "Alphas Mods",
            Self::GradientUpDown => "Gradient Up/Down",
            Self::GradientLeftRight => "Gradient Left/Right",
            Self::Breathing => "Breathing",
            Self::BandSat => "Band Sat",
            Self::BandVal => "Band Val",
            Self::BandPinwheelSat => "Band Pinwheel Sat",
            Self::BandPinwheelVal => "Band Pinwheel Val",
            Self::BandSpiralSat => "Band Spiral Sat",
            Self::BandSpiralVal => "Band Spiral Val",
            Self::CycleAll => "Cycle All",
            Self::CycleLeftRight => "Cycle Left/Right",
            Self::CycleUpDown => "Cycle Up/Down",
            Self::RainbowMovingChevron => "Rainbow Moving Chevron",
            Self::CycleOutIn => "Cycle Out/In",
            Self::CycleOutInDual => "Cycle Out/In Dual",
            Self::CyclePinwheel => "Cycle Pinwheel",
            Self::CycleSpiral => "Cycle Spiral",
            Self::DualBeacon => "Dual Beacon",
            Self::RainbowBeacon => "Rainbow Beacon",
            Self::RainbowPinwheels => "Rainbow Pinwheels",
            Self::Raindrops => "Raindrops",
            Self::JellybeanRaindrops => "Jellybean Raindrops",
            Self::HueBreathing => "Hue Breathing",
            Self::HuePendulum => "Hue Pendulum",
            Self::HueWave => "Hue Wave",
            Self::TypingHeatmap => "Typing Heatmap",
            Self::DigitalRain => "Digital Rain",
            Self::SolidReactiveSimple => "Solid Reactive Simple",
            Self::SolidReactive => "Solid Reactive",
            Self::SolidReactiveWide => "Solid Reactive Wide",
            Self::SolidReactiveMultiwide => "Solid Reactive Multiwide",
            Self::SolidReactiveCross => "Solid Reactive Cross",
            Self::SolidReactiveMulticross => "Solid Reactive Multicross",
            Self::SolidReactiveNexus => "Solid Reactive Nexus",
            Self::SolidReactiveMultinexus => "Solid Reactive Multinexus",
            Self::Splash => "Splash",
            Self::Multisplash => "Multisplash",
            Self::SolidSplash => "Solid Splash",
            Self::SolidMultisplash => "Solid Multisplash",
            Self::PixelRain => "Pixel Rain",
            Self::PixelFractal => "Pixel Fractal",
        }
    }
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
    pub id:   ViaCommandId,
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
    /// Must include the channel so QMK routes to the correct save handler.
    pub fn custom_save(channel: u8) -> Self {
        Self::with_data(ViaCommandId::CustomSave, &[channel])
    }

    /// Get a Vial lighting value. Format: `[cmd_id, value_id]`
    pub fn vial_get_lighting_value(value_id: u8) -> Self {
        Self::with_data(ViaCommandId::CustomGetValue, &[value_id])
    }

    /// Set a Vial lighting value. Format: `[cmd_id, value_id, data...]`
    pub fn vial_set_lighting_value(value_id: u8, payload: &[u8]) -> Self {
        let mut data = vec![value_id];
        data.extend_from_slice(payload);
        Self::with_data(ViaCommandId::CustomSetValue, &data)
    }

    /// Save custom values for Vial (no channel byte needed).
    pub fn vial_custom_save() -> Self {
        Self::simple(ViaCommandId::CustomSave)
    }

    /// VialRGB: get info. Response: [cmd, 0x40, version_lo, version_hi, max_brightness]
    pub fn vialrgb_get_info() -> Self {
        Self::with_data(ViaCommandId::CustomGetValue, &[VialRgbCmd::GetInfo as u8])
    }

    /// VialRGB: get current mode. Response: [cmd, 0x41, mode_lo, mode_hi, speed, hue, sat, val]
    pub fn vialrgb_get_mode() -> Self {
        Self::with_data(
            ViaCommandId::CustomGetValue,
            &[VialRgbCmd::GetModeOrSetMode as u8],
        )
    }

    /// VialRGB: set mode (all-in-one). Payload: [0x41, mode_lo, mode_hi, speed, hue, sat, val]
    pub fn vialrgb_set_mode(mode: u16, speed: u8, hue: u8, sat: u8, val: u8) -> Self {
        Self::with_data(
            ViaCommandId::CustomSetValue,
            &[
                VialRgbCmd::GetModeOrSetMode as u8,
                (mode & 0xFF) as u8,
                (mode >> 8) as u8,
                speed,
                hue,
                sat,
                val,
            ],
        )
    }

    /// Vial: get keyboard ID. Response: [0xFE, vial_protocol_version(u32 LE), uid(8 bytes)]
    pub fn vial_get_keyboard_id() -> Self {
        Self::with_data(ViaCommandId::VialPrefix, &[0x00])
    }

    /// Vial: get compressed definition size. Response: [0xFE, size(u32 LE)]
    pub fn vial_get_size() -> Self {
        Self::with_data(ViaCommandId::VialPrefix, &[0x01])
    }

    /// Vial: get compressed definition page. Each page is 32 bytes of the compressed data.
    pub fn vial_get_def(page: u16) -> Self {
        Self::with_data(
            ViaCommandId::VialPrefix,
            &[0x02, (page & 0xFF) as u8, (page >> 8) as u8],
        )
    }

    /// Vial: get dynamic entry counts (tap dance, combos, key overrides, alt repeat keys).
    /// Response: [td_count, combo_count, ko_count, arep_count, ..., feature_flags]
    pub fn vial_get_dynamic_entry_count() -> Self {
        Self::with_data(ViaCommandId::VialPrefix, &[0x0D, 0x00])
    }

    /// Vial: get tap dance entry at index.
    /// Response: [status, <10 bytes: on_tap(u16 LE), on_hold(u16 LE), on_double_tap(u16 LE),
    /// on_tap_hold(u16 LE), tapping_term(u16 LE)>]
    pub fn vial_tap_dance_get(idx: u8) -> Self {
        Self::with_data(ViaCommandId::VialPrefix, &[0x0D, 0x01, idx])
    }

    /// Vial: set tap dance entry at index. Payload is 10 bytes (raw struct).
    pub fn vial_tap_dance_set(idx: u8, data: &[u8; 10]) -> Self {
        let mut payload = vec![0x0D, 0x02, idx];
        payload.extend_from_slice(data);
        Self::with_data(ViaCommandId::VialPrefix, &payload)
    }

    /// Vial: get combo entry at index.
    /// Response: [status, <10 bytes: input0..3(u16 LE x4), output(u16 LE)>]
    pub fn vial_combo_get(idx: u8) -> Self {
        Self::with_data(ViaCommandId::VialPrefix, &[0x0D, 0x03, idx])
    }

    /// Vial: set combo entry at index. Payload is 10 bytes (raw struct).
    pub fn vial_combo_set(idx: u8, data: &[u8; 10]) -> Self {
        let mut payload = vec![0x0D, 0x04, idx];
        payload.extend_from_slice(data);
        Self::with_data(ViaCommandId::VialPrefix, &payload)
    }

    /// Vial: get key override entry at index.
    /// Response: [status, <10 bytes: trigger(u16 LE), replacement(u16 LE), layers(u16 LE),
    /// trigger_mods, negative_mod_mask, suppressed_mods, options>]
    pub fn vial_key_override_get(idx: u8) -> Self {
        Self::with_data(ViaCommandId::VialPrefix, &[0x0D, 0x05, idx])
    }

    /// Vial: set key override entry at index. Payload is 10 bytes (raw struct).
    pub fn vial_key_override_set(idx: u8, data: &[u8; 10]) -> Self {
        let mut payload = vec![0x0D, 0x06, idx];
        payload.extend_from_slice(data);
        Self::with_data(ViaCommandId::VialPrefix, &payload)
    }

    /// VialRGB: get supported effects. Pass gt=0 first, then gt=last_id to paginate.
    /// Response: list of u16 effect IDs, terminated by 0xFFFF.
    pub fn vialrgb_get_supported(gt: u16) -> Self {
        Self::with_data(
            ViaCommandId::CustomGetValue,
            &[
                VialRgbCmd::GetSupportedOrDirectFastSet as u8,
                (gt & 0xFF) as u8,
                (gt >> 8) as u8,
            ],
        )
    }

    /// Vial: get QMK Settings value count.
    /// Used for pointing device/trackpad configuration.
    /// Response: [count_lo, count_hi]
    pub fn vial_qmk_settings_query(page: u16) -> Self {
        Self::with_data(
            ViaCommandId::VialPrefix,
            &[0x0E, 0x00, (page & 0xFF) as u8, (page >> 8) as u8],
        )
    }

    /// Vial: get a QMK setting value by setting ID.
    /// Response: [status, value bytes...]
    pub fn vial_qmk_settings_get(setting_id: u16) -> Self {
        Self::with_data(
            ViaCommandId::VialPrefix,
            &[0x0E, 0x01, (setting_id & 0xFF) as u8, (setting_id >> 8) as u8],
        )
    }

    /// Vial: set a QMK setting value.
    /// Payload: [setting_id_lo, setting_id_hi, value bytes...]
    pub fn vial_qmk_settings_set(setting_id: u16, value: &[u8]) -> Self {
        let mut payload = vec![
            0x0E,
            0x02,
            (setting_id & 0xFF) as u8,
            (setting_id >> 8) as u8,
        ];
        payload.extend_from_slice(value);
        Self::with_data(ViaCommandId::VialPrefix, &payload)
    }

    /// Vial: reset QMK settings to defaults.
    pub fn vial_qmk_settings_reset() -> Self {
        Self::with_data(ViaCommandId::VialPrefix, &[0x0E, 0x03])
    }

    /// Get encoder keycode for a layer/encoder/direction.
    pub fn get_encoder(layer: u8, encoder: u8, clockwise: bool) -> Self {
        Self::with_data(
            ViaCommandId::DynamicKeymapGetEncoder,
            &[layer, encoder, clockwise as u8],
        )
    }

    /// Set encoder keycode for a layer/encoder/direction.
    pub fn set_encoder(layer: u8, encoder: u8, clockwise: bool, keycode: u16) -> Self {
        Self::with_data(
            ViaCommandId::DynamicKeymapSetEncoder,
            &[
                layer,
                encoder,
                clockwise as u8,
                (keycode >> 8) as u8,
                (keycode & 0xFF) as u8,
            ],
        )
    }
}
