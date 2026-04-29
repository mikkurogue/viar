/// A QMK keycode value (u16).
///
/// QMK keycodes are 16-bit values where the upper bits encode the type
/// (basic, mod-tap, layer-tap, etc.) and the lower bits encode the specific key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Keycode(pub u16);

impl Keycode {
    pub const NONE: Self = Self(0x0000);
    pub const TRANSPARENT: Self = Self(0x0001);

    pub fn raw(self) -> u16 {
        self.0
    }

    /// Construct a Layer-Tap keycode: LT(layer, kc)
    /// tap produces `kc`, hold activates `layer`.
    pub fn layer_tap(layer: u8, kc: u8) -> Self {
        Self(0x4000 | ((layer as u16 & 0x0F) << 8) | kc as u16)
    }

    /// Construct a Mod-Tap keycode: MT(mod_mask, kc)
    /// tap produces `kc`, hold activates modifier(s).
    pub fn mod_tap(mods: u8, kc: u8) -> Self {
        Self(0x2000 | ((mods as u16 & 0x1F) << 8) | kc as u16)
    }

    /// Construct a Layer-Mod keycode: LM(layer, mod)
    /// Activates `layer` with modifier(s) applied.
    pub fn layer_mod(layer: u8, mods: u8) -> Self {
        Self(0x5000 | ((layer as u16 & 0xF) << 4) | (mods as u16 & 0xF))
    }

    /// Construct a One-Shot Modifier keycode: OSM(mod_mask)
    pub fn one_shot_mod(mods: u8) -> Self {
        Self(0x52A0 | (mods as u16 & 0x1F))
    }

    /// Construct a Mod+Key combo: e.g. C(kc), S(kc), C+S(kc)
    pub fn mod_key(mods: u8, kc: u8) -> Self {
        Self(((mods as u16 & 0x1F) << 8) | kc as u16)
    }

    /// Construct a Swap Hands keycode: SH(kc)
    pub fn swap_hands(kc: u8) -> Self {
        Self(0x5600 | kc as u16)
    }

    /// Extract the modifier mask for ModTap, Mod, or OSM keycodes.
    pub fn mod_mask(self) -> u8 {
        match self.category() {
            KeycodeCategory::ModTap => ((self.0 >> 8) & 0x1F) as u8,
            KeycodeCategory::Mod => ((self.0 >> 8) & 0x1F) as u8,
            KeycodeCategory::LayerOneShotMod => (self.0 & 0x1F) as u8,
            KeycodeCategory::LayerMod => (self.0 & 0xF) as u8,
            _ => 0,
        }
    }

    /// Extract the basic keycode for ModTap, Mod, or LayerTap keycodes.
    pub fn base_keycode(self) -> u8 {
        match self.category() {
            KeycodeCategory::ModTap | KeycodeCategory::Mod | KeycodeCategory::LayerTap => {
                (self.0 & 0xFF) as u8
            }
            _ => 0,
        }
    }

    /// Extract the layer number for layer keycodes.
    pub fn layer(self) -> u8 {
        match self.category() {
            KeycodeCategory::LayerTap => ((self.0 >> 8) & 0x0F) as u8,
            KeycodeCategory::LayerMod => ((self.0 >> 4) & 0xF) as u8,
            KeycodeCategory::LayerMomentary
            | KeycodeCategory::LayerToggle
            | KeycodeCategory::LayerOn
            | KeycodeCategory::LayerDefault
            | KeycodeCategory::LayerOneShotLayer
            | KeycodeCategory::LayerTapToggle
            | KeycodeCategory::PersistentDefLayer => (self.0 & 0x1F) as u8,
            _ => 0,
        }
    }

    /// Determine the category of this keycode.
    pub fn category(self) -> KeycodeCategory {
        match self.0 {
            0x0000 => KeycodeCategory::None,
            0x0001 => KeycodeCategory::Transparent,
            // Mouse keys: QMK uses 0x00CD-0x00D9 range
            0x00CD..=0x00D9 => KeycodeCategory::Mouse,
            0x0004..=0x00FF => KeycodeCategory::Basic,
            0x0100..=0x1FFF => KeycodeCategory::Mod,
            0x2000..=0x3FFF => KeycodeCategory::ModTap,
            0x4000..=0x4FFF => KeycodeCategory::LayerTap,
            0x5000..=0x51FF => KeycodeCategory::LayerMod,
            0x5200..=0x521F => KeycodeCategory::LayerOn, // TO(layer)
            0x5220..=0x523F => KeycodeCategory::LayerMomentary, // MO(layer)
            0x5240..=0x525F => KeycodeCategory::LayerDefault, // DF(layer)
            0x5260..=0x527F => KeycodeCategory::LayerToggle, // TG(layer)
            0x5280..=0x529F => KeycodeCategory::LayerOneShotLayer, // OSL(layer)
            0x52A0..=0x52BF => KeycodeCategory::LayerOneShotMod, // OSM(mod)
            0x52C0..=0x52DF => KeycodeCategory::LayerTapToggle, // TT(layer)
            0x52E0..=0x52FF => KeycodeCategory::PersistentDefLayer,
            0x5600..=0x56FF => KeycodeCategory::SwapHands,
            0x5700..=0x57FF => KeycodeCategory::TapDance,
            0x7C77 => KeycodeCategory::TriLayer,
            0x7C78 => KeycodeCategory::TriLayer,
            0x7000..=0x70FF => KeycodeCategory::Magic,
            0x7800..=0x78FF => KeycodeCategory::Lighting,
            0x7C00..=0x7DFF => KeycodeCategory::Quantum,
            _ => KeycodeCategory::Unknown,
        }
    }

    /// Get a human-readable name for this keycode.
    /// Returns an owned String since complex keycodes need formatting.
    pub fn name(self) -> String {
        if let Some(name) = basic_keycode_name(self.0) {
            return name.to_string();
        }
        if let Some(name) = mouse_keycode_name(self.0) {
            return name.to_string();
        }
        if let Some(name) = magic_keycode_name(self.0) {
            return name.to_string();
        }
        if let Some(name) = lighting_keycode_name(self.0) {
            return name.to_string();
        }
        if let Some(name) = quantum_keycode_name(self.0) {
            return name.to_string();
        }
        match self.0 {
            0x0000 => "NONE".to_string(),
            0x0001 => "TRNS".to_string(),
            0x7C77 => "TL_LO".to_string(),
            0x7C78 => "TL_HI".to_string(),
            _ => self.decode_complex(),
        }
    }

    /// Get a short label (for rendering on small key caps).
    /// Truncates to fit in tight spaces.
    pub fn short_name(self) -> String {
        let full = self.name();
        if full.len() <= 5 {
            full
        } else {
            // For complex names, abbreviate
            full
        }
    }

    /// For dual-function keys (mod-tap, layer-tap, etc.), return separate
    /// tap and hold labels for split rendering on keycaps.
    /// Returns `Some((tap_label, hold_label))` or `None` for simple keys.
    pub fn dual_labels(self) -> Option<(String, String)> {
        let raw = self.0;
        match self.category() {
            KeycodeCategory::ModTap => {
                let mods = (raw >> 8) & 0x1F;
                let kc = raw & 0xFF;
                let tap = basic_keycode_name(kc).unwrap_or("??").to_string();
                let hold = mod_mask_to_string(mods as u8);
                Some((tap, hold))
            }
            KeycodeCategory::LayerTap => {
                let layer = (raw >> 8) & 0x0F;
                let kc = raw & 0xFF;
                let tap = basic_keycode_name(kc).unwrap_or("??").to_string();
                let hold = format!("LT{layer}");
                Some((tap, hold))
            }
            KeycodeCategory::LayerMod => {
                let layer = (raw >> 4) & 0xF;
                let mods = raw & 0xF;
                let hold = mod_mask_to_string(mods as u8);
                let tap = format!("LM{layer}");
                Some((tap, hold))
            }
            KeycodeCategory::LayerTapToggle => {
                let layer = raw & 0x1F;
                Some((format!("TT{layer}"), format!("L{layer}")))
            }
            KeycodeCategory::LayerOneShotLayer => {
                let layer = raw & 0x1F;
                Some(("OSL".to_string(), format!("L{layer}")))
            }
            KeycodeCategory::LayerOneShotMod => {
                let mods = (raw & 0x1F) as u8;
                let hold = mod_mask_to_string(mods);
                Some(("OSM".to_string(), hold))
            }
            _ => None,
        }
    }

    /// Decode a complex (non-basic) keycode into a descriptive string.
    fn decode_complex(self) -> String {
        let raw = self.0;
        match self.category() {
            KeycodeCategory::LayerTap => {
                // LT(layer, kc): bits [11:8] = layer (4 bits), bits [7:0] = keycode
                let layer = (raw >> 8) & 0x0F;
                let kc = raw & 0xFF;
                let kc_name = basic_keycode_name(kc).unwrap_or("??");
                format!("LT({layer},{kc_name})")
            }
            KeycodeCategory::LayerMod => {
                // LM(layer, mod): bits [12:8] = mod, bits [7:4] = layer...
                // Actually: QK_LAYER_MOD = 0x5000, layer in bits [8:4], mods in bits [3:0] shifted
                // QMK: #define LM(layer, mod) (QK_LAYER_MOD | (((layer) & 0xF) << 4) | ((mod) &
                // 0xF))
                let layer = (raw >> 4) & 0xF;
                let mods = raw & 0xF;
                let mod_str = mod_mask_to_string(mods as u8);
                format!("LM({layer},{mod_str})")
            }
            KeycodeCategory::LayerMomentary => {
                let layer = raw & 0x1F;
                format!("MO({layer})")
            }
            KeycodeCategory::LayerDefault => {
                let layer = raw & 0x1F;
                format!("DF({layer})")
            }
            KeycodeCategory::LayerToggle => {
                let layer = raw & 0x1F;
                format!("TG({layer})")
            }
            KeycodeCategory::LayerOneShotLayer => {
                let layer = raw & 0x1F;
                format!("OSL({layer})")
            }
            KeycodeCategory::LayerOneShotMod => {
                let mods = raw & 0x1F;
                format!("OSM({mods:#04x})")
            }
            KeycodeCategory::LayerTapToggle => {
                let layer = raw & 0x1F;
                format!("TT({layer})")
            }
            KeycodeCategory::PersistentDefLayer => {
                let layer = raw & 0x1F;
                format!("PDF({layer})")
            }
            KeycodeCategory::LayerOn => {
                // TO(layer): QK_TO | layer
                let layer = raw & 0x1F;
                format!("TO({layer})")
            }
            KeycodeCategory::ModTap => {
                // MT(mod, kc): bits [12:8] = mod mask, bits [7:0] = keycode
                let mods = (raw >> 8) & 0x1F;
                let kc = raw & 0xFF;
                let kc_name = basic_keycode_name(kc).unwrap_or("??");
                let mod_str = mod_mask_to_string(mods as u8);
                format!("MT({mod_str},{kc_name})")
            }
            KeycodeCategory::Mod => {
                // Modifier + basic key: bits [12:8] = mod, bits [7:0] = key
                let mods = ((raw >> 8) & 0x1F) as u8;
                let kc = raw & 0xFF;
                if kc == 0 {
                    mod_mask_to_string(mods)
                } else if mods == 0x02 {
                    // Shift-only: show the actual shifted symbol if possible
                    if let Some(sym) = shifted_symbol(kc) {
                        return sym.to_string();
                    }
                    let kc_name = basic_keycode_name(kc).unwrap_or("??");
                    format!("S({kc_name})")
                } else {
                    let kc_name = basic_keycode_name(kc).unwrap_or("??");
                    let mod_str = mod_mask_to_string(mods);
                    format!("{mod_str}({kc_name})")
                }
            }
            KeycodeCategory::TapDance => {
                let idx = raw & 0xFF;
                format!("TD({idx})")
            }
            KeycodeCategory::SwapHands => {
                let kc = raw & 0xFF;
                if kc == 0xF0 {
                    "SH_TG".to_string()
                } else if kc == 0xF1 {
                    "SH_TT".to_string()
                } else if kc == 0xF2 {
                    "SH_MON".to_string()
                } else if kc == 0xF3 {
                    "SH_MOFF".to_string()
                } else if kc == 0xF4 {
                    "SH_OFF".to_string()
                } else if kc == 0xF5 {
                    "SH_ON".to_string()
                } else if kc == 0xF6 {
                    "SH_OS".to_string()
                } else {
                    let kc_name = basic_keycode_name(kc).unwrap_or("??");
                    format!("SH({kc_name})")
                }
            }
            KeycodeCategory::Mouse => mouse_keycode_name(raw).unwrap_or("Mouse??").to_string(),
            _ => format!("{raw:#06x}"),
        }
    }

    /// Get a human-friendly description for tooltips.
    pub fn description(self) -> String {
        // Special keycodes
        match self.0 {
            0x0000 => return "No action (transparent to layers below)".into(),
            0x0001 => return "Transparent — falls through to the layer below".into(),
            0x7C77 => return "Tri-Layer Lower — activates tri-layer when held".into(),
            0x7C78 => return "Tri-Layer Upper — activates tri-layer when held".into(),
            _ => {}
        }
        // Basic keycodes with richer descriptions
        if let Some(desc) = basic_keycode_description(self.0) {
            return desc.into();
        }
        // Letters
        if self.0 >= 0x04 && self.0 <= 0x1D {
            let ch = (b'A' + (self.0 - 0x04) as u8) as char;
            return format!("Character {ch}");
        }
        // F-keys
        if self.0 >= 0x3A && self.0 <= 0x45 {
            let n = self.0 - 0x3A + 1;
            return format!("Function key F{n}");
        }
        // Numpad 1-9
        if self.0 >= 0x59 && self.0 <= 0x61 {
            let n = self.0 - 0x59 + 1;
            return format!("Numpad {n}");
        }
        // Complex keycodes by category
        match self.category() {
            KeycodeCategory::Mod => {
                let mods = ((self.0 >> 8) & 0x1F) as u8;
                let kc = self.0 & 0xFF;
                let mod_str = mod_mask_to_string(mods);
                if kc == 0 {
                    format!("{mod_str} modifier")
                } else {
                    let kc_name = basic_keycode_name(kc).unwrap_or("??");
                    format!("{kc_name} with {mod_str} held")
                }
            }
            KeycodeCategory::ModTap => {
                let mods = ((self.0 >> 8) & 0x1F) as u8;
                let kc = self.0 & 0xFF;
                let kc_name = basic_keycode_name(kc).unwrap_or("??");
                let mod_str = mod_mask_to_string(mods);
                format!("{kc_name} on tap, {mod_str} on hold")
            }
            KeycodeCategory::LayerTap => {
                let layer = (self.0 >> 8) & 0x0F;
                let kc = self.0 & 0xFF;
                let kc_name = basic_keycode_name(kc).unwrap_or("??");
                format!("{kc_name} on tap, Layer {layer} on hold")
            }
            KeycodeCategory::LayerMod => {
                let layer = (self.0 >> 4) & 0xF;
                let mods = (self.0 & 0xF) as u8;
                let mod_str = mod_mask_to_string(mods);
                format!("Activate Layer {layer} with {mod_str}")
            }
            KeycodeCategory::LayerMomentary => {
                let layer = self.0 & 0x1F;
                format!("Momentary Layer {layer} — active while held")
            }
            KeycodeCategory::LayerToggle => {
                let layer = self.0 & 0x1F;
                format!("Toggle Layer {layer} on/off")
            }
            KeycodeCategory::LayerOn => {
                let layer = self.0 & 0x1F;
                format!("Turn on Layer {layer} (deactivate all others)")
            }
            KeycodeCategory::LayerDefault => {
                let layer = self.0 & 0x1F;
                format!("Set Layer {layer} as the default base layer")
            }
            KeycodeCategory::LayerOneShotLayer => {
                let layer = self.0 & 0x1F;
                format!("One-Shot Layer {layer} — active for the next keypress only")
            }
            KeycodeCategory::LayerOneShotMod => {
                let mods = (self.0 & 0x1F) as u8;
                let mod_str = mod_mask_to_string(mods);
                format!("One-Shot {mod_str} — applies to the next keypress only")
            }
            KeycodeCategory::LayerTapToggle => {
                let layer = self.0 & 0x1F;
                format!("Layer {layer} on hold, toggle on tap")
            }
            KeycodeCategory::PersistentDefLayer => {
                let layer = self.0 & 0x1F;
                format!("Persistently set Layer {layer} as default (survives reboot)")
            }
            KeycodeCategory::TapDance => {
                let idx = self.0 & 0xFF;
                format!("Tap Dance {idx} — different actions for tap/hold/double-tap")
            }
            KeycodeCategory::TriLayer => "Tri-Layer key".into(),
            KeycodeCategory::Mouse => mouse_keycode_description(self.0)
                .unwrap_or("Mouse key")
                .to_string(),
            KeycodeCategory::SwapHands => {
                let kc = self.0 & 0xFF;
                match kc {
                    0xF0 => "Toggle swap hands".into(),
                    0xF1 => "Tap-toggle swap hands".into(),
                    0xF2 => "Momentary swap on".into(),
                    0xF3 => "Momentary swap off".into(),
                    0xF4 => "Turn off swap hands".into(),
                    0xF5 => "Turn on swap hands".into(),
                    0xF6 => "One-shot swap hands".into(),
                    _ => {
                        let kc_name = basic_keycode_name(kc).unwrap_or("??");
                        format!("{kc_name} on tap, swap hands on hold")
                    }
                }
            }
            KeycodeCategory::Magic => magic_keycode_name(self.0)
                .map(|n| format!("Magic: {n}"))
                .unwrap_or_else(|| format!("Magic keycode (0x{:04X})", self.0)),
            KeycodeCategory::Lighting => lighting_keycode_name(self.0)
                .map(|n| format!("Lighting: {n}"))
                .unwrap_or_else(|| format!("Lighting keycode (0x{:04X})", self.0)),
            KeycodeCategory::Quantum => quantum_keycode_name(self.0)
                .map(|n| format!("Quantum: {n}"))
                .unwrap_or_else(|| format!("Quantum keycode (0x{:04X})", self.0)),
            _ => {
                let name = self.name();
                format!("{name} (0x{:04X})", self.0)
            }
        }
    }
}

impl std::fmt::Display for Keycode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeycodeCategory {
    None,
    Transparent,
    Basic,
    Mouse,
    Mod,
    LayerTap,
    LayerOn,
    LayerMomentary,
    LayerDefault,
    LayerToggle,
    LayerOneShotLayer,
    LayerOneShotMod,
    TapDance,
    ModTap,
    TriLayer,
    LayerMod,
    LayerTapToggle,
    PersistentDefLayer,
    SwapHands,
    Magic,
    Lighting,
    Quantum,
    Unicode,
    Unknown,
}

/// Convert a QMK modifier bitmask to a human-readable string.
pub fn mod_mask_to_string(mods: u8) -> String {
    let mut parts = Vec::new();
    // Left modifiers
    if mods & 0x01 != 0 {
        parts.push("C");
    } // Ctrl
    if mods & 0x02 != 0 {
        parts.push("S");
    } // Shift
    if mods & 0x04 != 0 {
        parts.push("A");
    } // Alt
    if mods & 0x08 != 0 {
        parts.push("G");
    } // GUI
    // Right modifiers (bit 4 = use right side)
    if mods & 0x10 != 0 {
        // Right-side flag — modify the labels
        parts.iter_mut().for_each(|p| {
            *p = match *p {
                "C" => "RC",
                "S" => "RS",
                "A" => "RA",
                "G" => "RG",
                _ => *p,
            }
        });
    }
    if parts.is_empty() {
        "MOD".to_string()
    } else {
        parts.join("+")
    }
}

/// Map a basic HID keycode to its US-ANSI shifted symbol.
/// Returns None if there's no common symbol (e.g. Shift+A is just 'A').
fn shifted_symbol(kc: u16) -> Option<&'static str> {
    Some(match kc {
        0x1E => "!",  // 1
        0x1F => "@",  // 2
        0x20 => "#",  // 3
        0x21 => "$",  // 4
        0x22 => "%",  // 5
        0x23 => "^",  // 6
        0x24 => "&",  // 7
        0x25 => "*",  // 8
        0x26 => "(",  // 9
        0x27 => ")",  // 0
        0x2D => "_",  // -
        0x2E => "+",  // =
        0x2F => "{",  // [
        0x30 => "}",  // ]
        0x31 => "|",  // backslash
        0x33 => ":",  // ;
        0x34 => "\"", // '
        0x35 => "~",  // `
        0x36 => "<",  // ,
        0x37 => ">",  // .
        0x38 => "?",  // /
        _ => return None,
    })
}

/// Human-friendly descriptions for basic HID keycodes (for tooltips).
fn basic_keycode_description(kc: u16) -> Option<&'static str> {
    Some(match kc {
        0x04..=0x1D => {
            // Letters A-Z — return None to fall through to generic
            return None;
        }
        0x1E => "Number 1",
        0x1F => "Number 2",
        0x20 => "Number 3",
        0x21 => "Number 4",
        0x22 => "Number 5",
        0x23 => "Number 6",
        0x24 => "Number 7",
        0x25 => "Number 8",
        0x26 => "Number 9",
        0x27 => "Number 0",
        0x28 => "Enter / Return",
        0x29 => "Escape",
        0x2A => "Backspace",
        0x2B => "Tab",
        0x2C => "Spacebar",
        0x2D => "Minus / Hyphen",
        0x2E => "Equals",
        0x2F => "Left Bracket",
        0x30 => "Right Bracket",
        0x31 => "Backslash",
        0x33 => "Semicolon",
        0x34 => "Apostrophe / Quote",
        0x35 => "Grave / Backtick",
        0x36 => "Comma",
        0x37 => "Period / Dot",
        0x38 => "Forward Slash",
        0x39 => "Caps Lock",
        0x3A..=0x45 => return None, // F-keys handled below
        0x46 => "Print Screen",
        0x47 => "Scroll Lock",
        0x48 => "Pause / Break",
        0x49 => "Insert",
        0x4A => "Home",
        0x4B => "Page Up",
        0x4C => "Delete",
        0x4D => "End",
        0x4E => "Page Down",
        0x4F => "Right Arrow",
        0x50 => "Left Arrow",
        0x51 => "Down Arrow",
        0x52 => "Up Arrow",
        0x53 => "Num Lock",
        0x54 => "Numpad Divide",
        0x55 => "Numpad Multiply",
        0x56 => "Numpad Minus",
        0x57 => "Numpad Plus",
        0x58 => "Numpad Enter",
        0x59..=0x61 => return None, // Numpad 1-9 — obvious
        0x62 => "Numpad 0",
        0x63 => "Numpad Decimal",
        0x65 => "Application / Menu key",
        0x66 => "System Power",
        0xA5 => "Media Play/Pause",
        0xA6 => "Media Stop",
        0xA7 => "Previous Track",
        0xA8 => "Mute audio",
        0xA9 => "Volume Up",
        0xAA => "Volume Down",
        0xAB => "Next Track",
        0xAC => "Media Eject",
        0xAD => "Fast Forward",
        0xAE => "Rewind",
        0xAF => "Screen Brightness Up",
        0xB0 => "Screen Brightness Down",
        0xB1 => "Media Select",
        0xB2 => "Launch Mail",
        0xB3 => "Launch Calculator",
        0xB4 => "Launch My Computer",
        0xB5 => "Browser Search",
        0xB6 => "Browser Home",
        0xB7 => "Browser Back",
        0xB8 => "Browser Forward",
        0xB9 => "Browser Stop",
        0xBA => "Browser Refresh",
        0xBB => "Browser Favorites",
        0xE0 => "Left Control",
        0xE1 => "Left Shift",
        0xE2 => "Left Alt / Option",
        0xE3 => "Left GUI / Super / Command",
        0xE4 => "Right Control",
        0xE5 => "Right Shift",
        0xE6 => "Right Alt / AltGr",
        0xE7 => "Right GUI / Super / Command",
        _ => return None,
    })
}

/// Look up the name of a basic HID keycode (0x04..0xFF range).
fn basic_keycode_name(kc: u16) -> Option<&'static str> {
    Some(match kc {
        0x04 => "A",
        0x05 => "B",
        0x06 => "C",
        0x07 => "D",
        0x08 => "E",
        0x09 => "F",
        0x0A => "G",
        0x0B => "H",
        0x0C => "I",
        0x0D => "J",
        0x0E => "K",
        0x0F => "L",
        0x10 => "M",
        0x11 => "N",
        0x12 => "O",
        0x13 => "P",
        0x14 => "Q",
        0x15 => "R",
        0x16 => "S",
        0x17 => "T",
        0x18 => "U",
        0x19 => "V",
        0x1A => "W",
        0x1B => "X",
        0x1C => "Y",
        0x1D => "Z",
        0x1E => "1",
        0x1F => "2",
        0x20 => "3",
        0x21 => "4",
        0x22 => "5",
        0x23 => "6",
        0x24 => "7",
        0x25 => "8",
        0x26 => "9",
        0x27 => "0",
        0x28 => "Enter",
        0x29 => "Esc",
        0x2A => "Bksp",
        0x2B => "Tab",
        0x2C => "Space",
        0x2D => "-",
        0x2E => "=",
        0x2F => "[",
        0x30 => "]",
        0x31 => "\\",
        0x33 => ";",
        0x34 => "'",
        0x35 => "`",
        0x36 => ",",
        0x37 => ".",
        0x38 => "/",
        0x39 => "CapsLk",
        0x3A => "F1",
        0x3B => "F2",
        0x3C => "F3",
        0x3D => "F4",
        0x3E => "F5",
        0x3F => "F6",
        0x40 => "F7",
        0x41 => "F8",
        0x42 => "F9",
        0x43 => "F10",
        0x44 => "F11",
        0x45 => "F12",
        0x46 => "PrtSc",
        0x47 => "ScrLk",
        0x48 => "Pause",
        0x49 => "Ins",
        0x4A => "Home",
        0x4B => "PgUp",
        0x4C => "Del",
        0x4D => "End",
        0x4E => "PgDn",
        0x4F => "→",
        0x50 => "←",
        0x51 => "↓",
        0x52 => "↑",
        // Numpad
        0x53 => "NLck",
        0x54 => "N/",
        0x55 => "N*",
        0x56 => "N-",
        0x57 => "N+",
        0x58 => "NEnt",
        0x59 => "N1",
        0x5A => "N2",
        0x5B => "N3",
        0x5C => "N4",
        0x5D => "N5",
        0x5E => "N6",
        0x5F => "N7",
        0x60 => "N8",
        0x61 => "N9",
        0x62 => "N0",
        0x63 => "N.",
        // Modifiers
        0xE0 => "LCtrl",
        0xE1 => "LShft",
        0xE2 => "LAlt",
        0xE3 => "LGui",
        0xE4 => "RCtrl",
        0xE5 => "RShft",
        0xE6 => "RAlt",
        0xE7 => "RGui",
        // Media
        0xA5 => "MPlay",
        0xA6 => "MStop",
        0xA7 => "MPrev",
        0xA8 => "Mute",
        0xA9 => "VolUp",
        0xAA => "VolDn",
        0xAB => "MNext",
        0xAC => "MEjct",
        0xAD => "MFfwd",
        0xAE => "MRwnd",
        0xAF => "BriUp",
        0xB0 => "BriDn",
        0xB1 => "MSlct",
        0xB2 => "Mail",
        0xB3 => "Calc",
        0xB4 => "MyCmp",
        0xB5 => "WwwSr",
        0xB6 => "WwwHm",
        0xB7 => "WwwBk",
        0xB8 => "WwwFw",
        0xB9 => "WwwSp",
        0xBA => "WwwRf",
        0xBB => "WwwFv",
        // Misc
        0x65 => "App",
        0x66 => "Power",
        // F13-F24
        0x68 => "F13",
        0x69 => "F14",
        0x6A => "F15",
        0x6B => "F16",
        0x6C => "F17",
        0x6D => "F18",
        0x6E => "F19",
        0x6F => "F20",
        0x70 => "F21",
        0x71 => "F22",
        0x72 => "F23",
        0x73 => "F24",
        // International keys
        0x87 => "RO",
        0x88 => "Kana",
        0x89 => "Yen",
        0x8A => "Henk",
        0x8B => "Mhen",
        // Language keys
        0x90 => "HGL",
        0x91 => "Hanja",
        _ => return None,
    })
}

/// Look up the name of a mouse keycode (QMK 0x00CD-0x00D9 range).
fn mouse_keycode_name(kc: u16) -> Option<&'static str> {
    Some(match kc {
        0x00CD => "MsUp",
        0x00CE => "MsDn",
        0x00CF => "MsLt",
        0x00D0 => "MsRt",
        0x00D1 => "Btn1",
        0x00D2 => "Btn2",
        0x00D3 => "Btn3",
        0x00D4 => "Btn4",
        0x00D5 => "Btn5",
        0x00D6 => "WhUp",
        0x00D7 => "WhDn",
        0x00D8 => "WhLt",
        0x00D9 => "WhRt",
        _ => return None,
    })
}

/// Human-friendly descriptions for mouse keycodes.
fn mouse_keycode_description(kc: u16) -> Option<&'static str> {
    Some(match kc {
        0x00CD => "Mouse Cursor Up",
        0x00CE => "Mouse Cursor Down",
        0x00CF => "Mouse Cursor Left",
        0x00D0 => "Mouse Cursor Right",
        0x00D1 => "Mouse Button 1 (Left Click)",
        0x00D2 => "Mouse Button 2 (Right Click)",
        0x00D3 => "Mouse Button 3 (Middle Click)",
        0x00D4 => "Mouse Button 4 (Back)",
        0x00D5 => "Mouse Button 5 (Forward)",
        0x00D6 => "Mouse Wheel Up (Scroll Up)",
        0x00D7 => "Mouse Wheel Down (Scroll Down)",
        0x00D8 => "Mouse Wheel Left (Scroll Left)",
        0x00D9 => "Mouse Wheel Right (Scroll Right)",
        _ => return None,
    })
}

/// Look up the name of a Magic keycode (0x7000-0x70FF range).
fn magic_keycode_name(kc: u16) -> Option<&'static str> {
    Some(match kc {
        0x7000 => "MG_SWNU", // Swap Control and GUI (on)
        0x7001 => "MG_SWCU", // Swap Control and Caps Lock (on)
        0x7002 => "MG_SWLA", // Swap Left Alt and GUI (on)
        0x7003 => "MG_SWRA", // Swap Right Alt and GUI (on)
        0x7004 => "MG_NKRO", // N-Key Rollover (on)
        0x7005 => "MG_GESC", // Grave Escape (on)
        0x7006 => "MG_BSPC", // Swap Backspace and Backslash (on)
        0x7007 => "CG_NORM", // Unswap Control and GUI
        0x7008 => "MG_UNCC", // Unswap Caps Lock
        0x7009 => "MG_UNLA", // Unswap Left Alt and GUI
        0x700A => "MG_UNRA", // Unswap Right Alt and GUI
        0x700B => "MG_UNNK", // N-Key Rollover (off)
        0x700C => "MG_UNGE", // Grave Escape (off)
        0x700D => "MG_UNBS", // Unswap Backspace
        0x700E => "CG_TOGG", // Toggle Control and GUI swap
        0x700F => "MG_TOGN", // Toggle NKRO
        _ => return None,
    })
}

/// Look up the name of a Lighting keycode (0x7800-0x78FF range).
fn lighting_keycode_name(kc: u16) -> Option<&'static str> {
    Some(match kc {
        0x7800 => "RGB_TOG",
        0x7801 => "RGB_MOD",
        0x7802 => "RGB_RMOD",
        0x7803 => "RGB_HUI",
        0x7804 => "RGB_HUD",
        0x7805 => "RGB_SAI",
        0x7806 => "RGB_SAD",
        0x7807 => "RGB_VAI",
        0x7808 => "RGB_VAD",
        0x7809 => "RGB_SPI",
        0x780A => "RGB_SPD",
        0x780B => "RGB_M_P",  // Plain
        0x780C => "RGB_M_B",  // Breathe
        0x780D => "RGB_M_R",  // Rainbow
        0x780E => "RGB_M_SW", // Swirl
        0x780F => "RGB_M_SN", // Snake
        0x7810 => "RGB_M_K",  // Knight
        0x7811 => "RGB_M_X",  // Xmas
        0x7812 => "RGB_M_G",  // Gradient
        0x7813 => "RGB_M_T",  // Test
        0x7820 => "BL_TOGG",  // Backlight toggle
        0x7821 => "BL_STEP",  // Backlight step
        0x7822 => "BL_ON",
        0x7823 => "BL_OFF",
        0x7824 => "BL_INC",
        0x7825 => "BL_DEC",
        0x7826 => "BL_BRTG", // Backlight breathing toggle
        _ => return None,
    })
}

/// Look up the name of a Quantum keycode (0x7C00-0x7DFF range).
fn quantum_keycode_name(kc: u16) -> Option<&'static str> {
    Some(match kc {
        0x7C00 => "QK_BOOT", // Bootloader
        0x7C01 => "QK_RBT",  // Reboot (soft reset)
        0x7C02 => "DB_TOGG", // Debug toggle
        0x7C03 => "EE_CLR",  // EEPROM clear
        0x7C10 => "AU_ON",   // Audio on
        0x7C11 => "AU_OFF",
        0x7C12 => "AU_TOGG",
        0x7C20 => "MU_ON", // Music on
        0x7C21 => "MU_OFF",
        0x7C22 => "MU_TOGG",
        0x7C23 => "MU_NEXT", // Music mode next
        0x7C30 => "CK_TOGG", // Clicky toggle
        0x7C31 => "CK_RST",
        0x7C32 => "CK_UP",
        0x7C33 => "CK_DN",
        0x7C40 => "HF_TOGG", // Haptic feedback toggle
        0x7C41 => "HF_RST",
        0x7C42 => "HF_NEXT",
        0x7C43 => "HF_CONT", // Continuous haptic
        0x7C44 => "HF_CONI", // Continuous haptic increase
        0x7C45 => "HF_COND", // Continuous haptic decrease
        0x7C46 => "HF_BUZZ", // Haptic buzz toggle
        0x7C77 => "TL_LO",   // Tri-Layer Lower
        0x7C78 => "TL_HI",   // Tri-Layer Upper
        0x7C7C => "AS_TOGG", // Auto Shift toggle
        0x7C7D => "AS_ON",
        0x7C7E => "AS_OFF",
        0x7C7F => "AS_RPT",  // Auto Shift repeat
        0x7C80 => "SE_LOCK", // Secure Lock
        0x7C81 => "SE_UNLK", // Secure Unlock
        0x7C82 => "SE_TOGG", // Secure Toggle
        0x7C83 => "SE_REQ",  // Secure Request
        0x7CA0 => "CM_ON",   // Combo on
        0x7CA1 => "CM_OFF",
        0x7CA2 => "CM_TOGG",
        0x7CB0 => "KL_TOGG", // Key Lock toggle
        0x7CC0 => "PM_TOGG", // Pointing Mode toggle
        0x7CC1 => "PM_NEXT",
        0x7CC2 => "PM_PREV",
        0x7CC3 => "PM_DMOD", // DPI mode
        0x7CC4 => "PM_UMOD", // DPI mode up
        _ => return None,
    })
}

/// Get all basic keycodes for display in a keycode picker.
pub fn all_basic_keycodes() -> Vec<Keycode> {
    let mut codes = Vec::new();
    // Letters
    for kc in 0x04..=0x1D {
        codes.push(Keycode(kc));
    }
    // Numbers
    for kc in 0x1E..=0x27 {
        codes.push(Keycode(kc));
    }
    // Common keys
    for kc in 0x28..=0x38 {
        codes.push(Keycode(kc));
    }
    // F-keys
    for kc in 0x3A..=0x45 {
        codes.push(Keycode(kc));
    }
    // Nav cluster
    for kc in 0x46..=0x52 {
        codes.push(Keycode(kc));
    }
    // Numpad
    for kc in 0x53..=0x63 {
        codes.push(Keycode(kc));
    }
    // Modifiers
    for kc in 0xE0..=0xE7 {
        codes.push(Keycode(kc));
    }
    codes
}

/// A named group of keycodes for the picker UI.
pub struct KeycodeGroup {
    pub name:  &'static str,
    pub codes: Vec<Keycode>,
}

/// Get keycodes organised into groups for the picker.
pub fn keycode_groups() -> Vec<KeycodeGroup> {
    vec![
        KeycodeGroup {
            name:  "Letters",
            codes: (0x04..=0x1Du16).map(Keycode).collect(),
        },
        KeycodeGroup {
            name:  "Numbers",
            codes: (0x1E..=0x27u16).map(Keycode).collect(),
        },
        KeycodeGroup {
            name:  "Symbols",
            codes: vec![
                Keycode(0x2D),
                Keycode(0x2E),
                Keycode(0x2F),
                Keycode(0x30),
                Keycode(0x31),
                Keycode(0x33),
                Keycode(0x34),
                Keycode(0x35),
                Keycode(0x36),
                Keycode(0x37),
                Keycode(0x38),
            ],
        },
        KeycodeGroup {
            name:  "Shifted",
            codes: vec![
                // S(1) through S(0)
                Keycode(0x021E),
                Keycode(0x021F),
                Keycode(0x0220),
                Keycode(0x0221),
                Keycode(0x0222),
                Keycode(0x0223),
                Keycode(0x0224),
                Keycode(0x0225),
                Keycode(0x0226),
                Keycode(0x0227),
                // S(-) S(=) S([) S(]) S(\) S(;) S(') S(`) S(,) S(.) S(/)
                Keycode(0x022D),
                Keycode(0x022E),
                Keycode(0x022F),
                Keycode(0x0230),
                Keycode(0x0231),
                Keycode(0x0233),
                Keycode(0x0234),
                Keycode(0x0235),
                Keycode(0x0236),
                Keycode(0x0237),
                Keycode(0x0238),
            ],
        },
        KeycodeGroup {
            name:  "Editing",
            codes: vec![
                Keycode(0x28), // Enter
                Keycode(0x29), // Esc
                Keycode(0x2A), // Backspace
                Keycode(0x2B), // Tab
                Keycode(0x2C), // Space
                Keycode(0x39), // CapsLock
                Keycode(0x49), // Insert
                Keycode(0x4C), // Delete
            ],
        },
        KeycodeGroup {
            name:  "Navigation",
            codes: vec![
                Keycode(0x4A), // Home
                Keycode(0x4D), // End
                Keycode(0x4B), // PgUp
                Keycode(0x4E), // PgDn
                Keycode(0x50), // Left
                Keycode(0x51), // Down
                Keycode(0x52), // Up
                Keycode(0x4F), // Right
            ],
        },
        KeycodeGroup {
            name:  "F-Keys",
            codes: {
                let mut v: Vec<Keycode> = (0x3A..=0x45u16).map(Keycode).collect();
                // F13-F24
                v.extend((0x68..=0x73u16).map(Keycode));
                v
            },
        },
        KeycodeGroup {
            name:  "Modifiers",
            codes: (0xE0..=0xE7u16).map(Keycode).collect(),
        },
        KeycodeGroup {
            name:  "Media",
            codes: vec![
                Keycode(0xA5), // Play
                Keycode(0xA6), // Stop
                Keycode(0xA7), // Prev
                Keycode(0xA8), // Mute
                Keycode(0xA9), // VolUp
                Keycode(0xAA), // VolDn
                Keycode(0xAB), // Next
                Keycode(0xAC), // Eject
                Keycode(0xAD), // Ffwd
                Keycode(0xAE), // Rwnd
                Keycode(0xAF), // BriUp
                Keycode(0xB0), // BriDn
                Keycode(0xB1), // MSlct
                Keycode(0xB2), // Mail
                Keycode(0xB3), // Calc
                Keycode(0xB4), // MyComp
                Keycode(0xB5), // WwwSearch
                Keycode(0xB6), // WwwHome
                Keycode(0xB7), // WwwBack
                Keycode(0xB8), // WwwFwd
                Keycode(0xB9), // WwwStop
                Keycode(0xBA), // WwwRefresh
                Keycode(0xBB), // WwwFavorites
            ],
        },
        KeycodeGroup {
            name:  "Mouse",
            codes: (0x00CDu16..=0x00D9u16).map(Keycode).collect(),
        },
        KeycodeGroup {
            name:  "Layers",
            codes: {
                let mut v = Vec::new();
                // MO(0)..MO(9)
                for i in 0..10u16 {
                    v.push(Keycode(0x5220 | i));
                }
                // TG(0)..TG(9)
                for i in 0..10u16 {
                    v.push(Keycode(0x5260 | i));
                }
                // TO(0)..TO(9)
                for i in 0..10u16 {
                    v.push(Keycode(0x5200 | i));
                }
                // DF(0)..DF(9)
                for i in 0..10u16 {
                    v.push(Keycode(0x5240 | i));
                }
                // OSL(0)..OSL(9)
                for i in 0..10u16 {
                    v.push(Keycode(0x5280 | i));
                }
                // TT(0)..TT(9)
                for i in 0..10u16 {
                    v.push(Keycode(0x52C0 | i));
                }
                // OSM (one-shot modifiers)
                // OSM(Ctrl), OSM(Shift), OSM(Alt), OSM(GUI)
                // OSM(RCtrl), OSM(RShift), OSM(RAlt), OSM(RGUI)
                // OSM(C+S), OSM(C+A), OSM(C+G), OSM(S+A), OSM(S+G), OSM(A+G)
                for mods in [
                    0x01u16, 0x02, 0x04, 0x08, 0x11, 0x12, 0x14, 0x18, 0x03, 0x05, 0x09, 0x06,
                    0x0A, 0x0C,
                ] {
                    v.push(Keycode(0x52A0 | mods));
                }
                // PDF(0)..PDF(3)
                for i in 0..4u16 {
                    v.push(Keycode(0x52E0 | i));
                }
                v
            },
        },
        KeycodeGroup {
            name:  "Numpad",
            codes: (0x53..=0x63u16).map(Keycode).collect(),
        },
        KeycodeGroup {
            name:  "Intl",
            codes: vec![
                Keycode(0x87), // RO
                Keycode(0x88), // Kana
                Keycode(0x89), // Yen
                Keycode(0x8A), // Henkan
                Keycode(0x8B), // Muhenkan
                Keycode(0x90), // Hangul
                Keycode(0x91), // Hanja
            ],
        },
        KeycodeGroup {
            name:  "Tap Dance",
            codes: (0..32u16).map(|i| Keycode(0x5700 | i)).collect(),
        },
        KeycodeGroup {
            name:  "Lighting",
            codes: vec![
                Keycode(0x7800), // RGB_TOG
                Keycode(0x7801), // RGB_MOD
                Keycode(0x7802), // RGB_RMOD
                Keycode(0x7803), // RGB_HUI
                Keycode(0x7804), // RGB_HUD
                Keycode(0x7805), // RGB_SAI
                Keycode(0x7806), // RGB_SAD
                Keycode(0x7807), // RGB_VAI
                Keycode(0x7808), // RGB_VAD
                Keycode(0x7809), // RGB_SPI
                Keycode(0x780A), // RGB_SPD
                Keycode(0x780B), // RGB_M_P
                Keycode(0x780C), // RGB_M_B
                Keycode(0x780D), // RGB_M_R
                Keycode(0x780E), // RGB_M_SW
                Keycode(0x780F), // RGB_M_SN
                Keycode(0x7810), // RGB_M_K
                Keycode(0x7811), // RGB_M_X
                Keycode(0x7812), // RGB_M_G
                Keycode(0x7813), // RGB_M_T
                Keycode(0x7820), // BL_TOGG
                Keycode(0x7821), // BL_STEP
                Keycode(0x7822), // BL_ON
                Keycode(0x7823), // BL_OFF
                Keycode(0x7824), // BL_INC
                Keycode(0x7825), // BL_DEC
                Keycode(0x7826), // BL_BRTG
            ],
        },
        KeycodeGroup {
            name:  "Quantum",
            codes: vec![
                Keycode(0x7C00), // QK_BOOT
                Keycode(0x7C01), // QK_RBT
                Keycode(0x7C02), // DB_TOGG
                Keycode(0x7C03), // EE_CLR
                Keycode(0x7C10), // AU_ON
                Keycode(0x7C11), // AU_OFF
                Keycode(0x7C12), // AU_TOGG
                Keycode(0x7C7C), // AS_TOGG
                Keycode(0x7C7D), // AS_ON
                Keycode(0x7C7E), // AS_OFF
                Keycode(0x7CA0), // CM_ON
                Keycode(0x7CA1), // CM_OFF
                Keycode(0x7CA2), // CM_TOGG
                Keycode(0x7CB0), // KL_TOGG
                Keycode(0x7CC0), // PM_TOGG
                Keycode(0x7CC1), // PM_NEXT
                Keycode(0x7CC2), // PM_PREV
                Keycode(0x7CC3), // PM_DMOD
                Keycode(0x7CC4), // PM_UMOD
            ],
        },
        KeycodeGroup {
            name:  "Special",
            codes: vec![
                Keycode(0x0000), // NONE
                Keycode(0x0001), // TRNS
                Keycode(0x46),   // PrtSc
                Keycode(0x47),   // ScrLk
                Keycode(0x48),   // Pause
                Keycode(0x65),   // App/Menu
                Keycode(0x66),   // Power
                Keycode(0x7C77), // TL_LO (Tri-Layer Lower)
                Keycode(0x7C78), // TL_HI (Tri-Layer Upper)
            ],
        },
    ]
}
