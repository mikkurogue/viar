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

    /// Determine the category of this keycode.
    pub fn category(self) -> KeycodeCategory {
        match self.0 {
            0x0000 => KeycodeCategory::None,
            0x0001 => KeycodeCategory::Transparent,
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
    Magic,
    Lighting,
    Quantum,
    Unicode,
    Unknown,
}

/// Convert a QMK modifier bitmask to a human-readable string.
fn mod_mask_to_string(mods: u8) -> String {
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
        0xA8 => "Mute audio",
        0xA9 => "Volume Up",
        0xAA => "Volume Down",
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
        0xA8 => "Mute",
        0xA9 => "VolUp",
        0xAA => "VolDn",
        // Misc
        0x65 => "App",
        0x66 => "Power",
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
            codes: (0x3A..=0x45u16).map(Keycode).collect(),
        },
        KeycodeGroup {
            name:  "Modifiers",
            codes: (0xE0..=0xE7u16).map(Keycode).collect(),
        },
        KeycodeGroup {
            name:  "Media",
            codes: vec![
                Keycode(0xA8), // Mute
                Keycode(0xA9), // VolUp
                Keycode(0xAA), // VolDn
            ],
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
                v
            },
        },
        KeycodeGroup {
            name:  "Numpad",
            codes: (0x53..=0x63u16).map(Keycode).collect(),
        },
        KeycodeGroup {
            name:  "Tap Dance",
            codes: (0..32u16).map(|i| Keycode(0x5700 | i)).collect(),
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
