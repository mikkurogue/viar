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
            0x2000..=0x3FFF => KeycodeCategory::LayerTap,
            0x4000..=0x4FFF => KeycodeCategory::LayerOn,
            0x5000..=0x50FF => KeycodeCategory::LayerMomentary,
            0x5100..=0x51FF => KeycodeCategory::LayerDefault,
            0x5200..=0x52FF => KeycodeCategory::LayerToggle,
            0x5300..=0x53FF => KeycodeCategory::LayerOneShotLayer,
            0x5400..=0x54FF => KeycodeCategory::LayerOneShotMod,
            0x5700 => KeycodeCategory::TriLayer,
            0x5701 => KeycodeCategory::TriLayer,
            0x5C00..=0x5CFF => KeycodeCategory::TapDance,
            0x7C00..=0x7FFF => KeycodeCategory::Unicode,
            0x7000..=0x7BFF => KeycodeCategory::ModTap,
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
            0x5700 => "TL_LO".to_string(),
            0x5701 => "TL_HI".to_string(),
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

    /// Decode a complex (non-basic) keycode into a descriptive string.
    fn decode_complex(self) -> String {
        let raw = self.0;
        match self.category() {
            KeycodeCategory::LayerTap => {
                // LT(layer, kc): bits [13:8] = layer, bits [7:0] = keycode
                let layer = (raw >> 8) & 0x1F;
                let kc = raw & 0xFF;
                let kc_name = basic_keycode_name(kc).unwrap_or("??");
                format!("LT({layer},{kc_name})")
            }
            KeycodeCategory::LayerMomentary => {
                let layer = raw & 0xFF;
                format!("MO({layer})")
            }
            KeycodeCategory::LayerDefault => {
                let layer = raw & 0xFF;
                format!("DF({layer})")
            }
            KeycodeCategory::LayerToggle => {
                let layer = raw & 0xFF;
                format!("TG({layer})")
            }
            KeycodeCategory::LayerOneShotLayer => {
                let layer = raw & 0xFF;
                format!("OSL({layer})")
            }
            KeycodeCategory::LayerOneShotMod => {
                let mods = raw & 0xFF;
                format!("OSM({mods:#04x})")
            }
            KeycodeCategory::LayerOn => {
                // TO(layer)
                let layer = (raw >> 4) & 0xF;
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
    pub name: &'static str,
    pub codes: Vec<Keycode>,
}

/// Get keycodes organised into groups for the picker.
pub fn keycode_groups() -> Vec<KeycodeGroup> {
    vec![
        KeycodeGroup {
            name: "Letters",
            codes: (0x04..=0x1Du16).map(Keycode).collect(),
        },
        KeycodeGroup {
            name: "Numbers",
            codes: (0x1E..=0x27u16).map(Keycode).collect(),
        },
        KeycodeGroup {
            name: "Symbols",
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
            name: "Shifted",
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
            name: "Editing",
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
            name: "Navigation",
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
            name: "F-Keys",
            codes: (0x3A..=0x45u16).map(Keycode).collect(),
        },
        KeycodeGroup {
            name: "Modifiers",
            codes: (0xE0..=0xE7u16).map(Keycode).collect(),
        },
        KeycodeGroup {
            name: "Media",
            codes: vec![
                Keycode(0xA8), // Mute
                Keycode(0xA9), // VolUp
                Keycode(0xAA), // VolDn
            ],
        },
        KeycodeGroup {
            name: "Layers",
            codes: {
                let mut v = Vec::new();
                // MO(0)..MO(9)
                for i in 0..10u16 {
                    v.push(Keycode(0x5000 | i));
                }
                // TG(0)..TG(9)
                for i in 0..10u16 {
                    v.push(Keycode(0x5200 | i));
                }
                // DF(0)..DF(9)
                for i in 0..10u16 {
                    v.push(Keycode(0x5100 | i));
                }
                // OSL(0)..OSL(9)
                for i in 0..10u16 {
                    v.push(Keycode(0x5300 | i));
                }
                v
            },
        },
        KeycodeGroup {
            name: "Numpad",
            codes: (0x53..=0x63u16).map(Keycode).collect(),
        },
        KeycodeGroup {
            name: "Special",
            codes: vec![
                Keycode(0x0000), // NONE
                Keycode(0x0001), // TRNS
                Keycode(0x46),   // PrtSc
                Keycode(0x47),   // ScrLk
                Keycode(0x48),   // Pause
                Keycode(0x65),   // App/Menu
                Keycode(0x66),   // Power
                Keycode(0x5700), // TL_LO (Tri-Layer Lower)
                Keycode(0x5701), // TL_HI (Tri-Layer Upper)
            ],
        },
    ]
}
