//! QMK Settings definitions — IDs, types, metadata, and helpers.
//!
//! Setting IDs are defined by `qmk_settings.c` in vial-qmk firmware.
//! IDs 1–27 are core QMK settings (tapping, autoshift, combos, etc.).
//! IDs 0x0100–0x01FF are pointing device settings.
//!
//! The `magic` setting (ID 21) is special: it packs 10 boolean flags into a u32.

use std::collections::HashMap;

// ─── Core QMK Settings IDs (from qmk_settings.c protos[] array) ─────────────

/// Grave Escape override flags (u8, bitfield: bit0=alt, bit1=ctrl, bit2=gui, bit3=shift)
pub const QS_GRAVE_ESC_OVERRIDE: u16 = 1;
/// Combo term in ms (u16)
pub const QS_COMBO_TERM: u16 = 2;
/// Auto Shift flags (u8, bitfield)
pub const QS_AUTO_SHIFT: u16 = 3;
/// Auto Shift timeout in ms (u16)
pub const QS_AUTO_SHIFT_TIMEOUT: u16 = 4;
/// One Shot Key tap toggle count (u8)
pub const QS_OSK_TAP_TOGGLE: u16 = 5;
/// One Shot Key timeout in ms (u16)
pub const QS_OSK_TIMEOUT: u16 = 6;
/// Tapping term in ms (u16)
pub const QS_TAPPING_TERM: u16 = 7;
/// Legacy tapping behavior flags (u8, bitfield — older firmware uses this instead of IDs 22-24)
/// Bit 0 = Permissive Hold, Bit 1 = Ignore Mod Tap Interrupt, Bit 2 = Tapping Force Hold, Bit 3 =
/// Retro Tapping
pub const QS_TAPPING_V2_LEGACY: u16 = 8;
// ID 8 was also used in older firmware for tapping_v2
/// Mouse key delay in ms (u16)
pub const QS_MOUSEKEY_DELAY: u16 = 9;
/// Mouse key interval in ms (u16)
pub const QS_MOUSEKEY_INTERVAL: u16 = 10;
/// Mouse key move delta (u16)
pub const QS_MOUSEKEY_MOVE_DELTA: u16 = 11;
/// Mouse key max speed (u16)
pub const QS_MOUSEKEY_MAX_SPEED: u16 = 12;
/// Mouse key time to max speed (u16)
pub const QS_MOUSEKEY_TIME_TO_MAX: u16 = 13;
/// Mouse key wheel delay in ms (u16)
pub const QS_MOUSEKEY_WHEEL_DELAY: u16 = 14;
/// Mouse key wheel interval in ms (u16)
pub const QS_MOUSEKEY_WHEEL_INTERVAL: u16 = 15;
/// Mouse key wheel max speed (u16)
pub const QS_MOUSEKEY_WHEEL_MAX_SPEED: u16 = 16;
/// Mouse key wheel time to max speed (u16)
pub const QS_MOUSEKEY_WHEEL_TIME_TO_MAX: u16 = 17;
/// Tap code delay in ms (u16)
pub const QS_TAP_CODE_DELAY: u16 = 18;
/// Tap hold caps delay in ms (u16)
pub const QS_TAP_HOLD_CAPS_DELAY: u16 = 19;
/// Tapping toggle count (u8)
pub const QS_TAPPING_TOGGLE: u16 = 20;
/// Magic settings (u32, bitfield with 10 boolean flags from keymap_config)
pub const QS_MAGIC: u16 = 21;
/// Permissive Hold (u8/bool, bit of tapping_v2)
pub const QS_PERMISSIVE_HOLD: u16 = 22;
/// Hold On Other Key Press (u8/bool, bit of tapping_v2)
pub const QS_HOLD_ON_OTHER_KEY_PRESS: u16 = 23;
/// Retro Tapping (u8/bool, bit of tapping_v2)
pub const QS_RETRO_TAPPING: u16 = 24;
/// Quick Tap Term in ms (u16)
pub const QS_QUICK_TAP_TERM: u16 = 25;
/// Chordal Hold (u8/bool, bit of tapping_v2)
pub const QS_CHORDAL_HOLD: u16 = 26;
/// Flow Tap Term in ms (u16)
pub const QS_FLOW_TAP_TERM: u16 = 27;

// ─── Pointing Device Settings IDs (0x0100 range) ────────────────────────────

/// Pointing device DPI (u16)
pub const QS_POINTING_DPI: u16 = 0x0100;
/// Pointing device scroll divisor (u8)
pub const QS_POINTING_SCROLL_DIVISOR: u16 = 0x0101;
/// Pointing device scroll divisor horizontal (u8)
pub const QS_POINTING_SCROLL_DIVISOR_H: u16 = 0x0102;
/// Pointing device invert X axis (u8, bool)
pub const QS_POINTING_INVERT_X: u16 = 0x0103;
/// Pointing device invert Y axis (u8, bool)
pub const QS_POINTING_INVERT_Y: u16 = 0x0104;
/// Pointing device invert scroll (u8, bool)
pub const QS_POINTING_INVERT_SCROLL: u16 = 0x0105;
/// Drag scroll enable (u8, bool)
pub const QS_POINTING_DRAG_SCROLL: u16 = 0x0106;
/// Drag scroll divisor (u8)
pub const QS_POINTING_DRAG_SCROLL_DIVISOR: u16 = 0x0107;
/// CPI / DPI for second pointing device (u16)
pub const QS_POINTING_DPI_2: u16 = 0x0108;
/// Sniping DPI (u16)
pub const QS_POINTING_SNIPING_DPI: u16 = 0x0109;
/// Auto mouse enable (u8, bool)
pub const QS_POINTING_AUTO_MOUSE_ENABLE: u16 = 0x0110;
/// Auto mouse layer (u8)
pub const QS_POINTING_AUTO_MOUSE_LAYER: u16 = 0x0111;
/// Auto mouse timeout in ms (u16)
pub const QS_POINTING_AUTO_MOUSE_TIMEOUT: u16 = 0x0112;

// ─── Magic Setting Bit Positions (for QS_MAGIC, ID 21) ──────────────────────

/// Swap Control and Caps Lock
pub const MAGIC_SWAP_CONTROL_CAPSLOCK: u8 = 0;
/// Caps Lock acts as Control
pub const MAGIC_CAPSLOCK_TO_CONTROL: u8 = 1;
/// Swap Left Alt and Left GUI
pub const MAGIC_SWAP_LALT_LGUI: u8 = 2;
/// Swap Right Alt and Right GUI
pub const MAGIC_SWAP_RALT_RGUI: u8 = 3;
/// Disable GUI key
pub const MAGIC_NO_GUI: u8 = 4;
/// Swap Grave and Escape
pub const MAGIC_SWAP_GRAVE_ESC: u8 = 5;
/// Swap Backslash and Backspace
pub const MAGIC_SWAP_BACKSLASH_BACKSPACE: u8 = 6;
/// Enable NKRO (N-Key Rollover)
pub const MAGIC_NKRO: u8 = 7;
/// Swap Left Ctrl and Left GUI
pub const MAGIC_SWAP_LCTL_LGUI: u8 = 8;
/// Swap Right Ctrl and Right GUI
pub const MAGIC_SWAP_RCTL_RGUI: u8 = 9;

// ─── Legacy Tapping V2 Bit Positions (for QS_TAPPING_V2_LEGACY, ID 8) ───────

/// Legacy: Permissive Hold
pub const LEGACY_TAPPING_PERMISSIVE_HOLD: u8 = 0;
/// Legacy: Ignore Mod Tap Interrupt (Hold On Other Key Press in newer firmware)
pub const LEGACY_TAPPING_IGNORE_MOD_TAP: u8 = 1;
/// Legacy: Tapping Force Hold (no quick tap in newer firmware)
pub const LEGACY_TAPPING_FORCE_HOLD: u8 = 2;
/// Legacy: Retro Tapping
pub const LEGACY_TAPPING_RETRO: u8 = 3;

// ─── Auto Shift Bit Positions (for QS_AUTO_SHIFT, ID 3) ─────────────────────

/// Enable Auto Shift
pub const AUTO_SHIFT_ENABLE: u8 = 0;
/// Auto Shift applies to modified keys
pub const AUTO_SHIFT_MODIFIERS: u8 = 1;
/// Disable Auto Shift for special keys
pub const AUTO_SHIFT_NO_SPECIAL: u8 = 2;
/// Disable Auto Shift for numeric keys
pub const AUTO_SHIFT_NO_NUMERIC: u8 = 3;
/// Disable Auto Shift for alpha keys
pub const AUTO_SHIFT_NO_ALPHA: u8 = 4;
/// Enable Auto Shift repeat
pub const AUTO_SHIFT_REPEAT: u8 = 5;
/// Disable Auto Shift auto-repeat
pub const AUTO_SHIFT_NO_AUTO_REPEAT: u8 = 6;

// ─── Grave Escape Override Bit Positions (for QS_GRAVE_ESC_OVERRIDE, ID 1) ──

/// Override when Alt is held
pub const GRAVE_ESC_ALT_OVERRIDE: u8 = 0;
/// Override when Ctrl is held
pub const GRAVE_ESC_CTRL_OVERRIDE: u8 = 1;
/// Override when GUI is held
pub const GRAVE_ESC_GUI_OVERRIDE: u8 = 2;
/// Override when Shift is held
pub const GRAVE_ESC_SHIFT_OVERRIDE: u8 = 3;

// ─── Setting Metadata ───────────────────────────────────────────────────────

/// How a setting value should be interpreted and rendered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingType {
    /// Unsigned 8-bit value
    U8,
    /// Unsigned 16-bit value
    U16,
    /// Unsigned 32-bit value
    U32,
    /// Single boolean (u8, 0 or 1)
    Bool,
    /// A bitfield within a larger value — the parent ID and bit position
    Bitfield { bit: u8 },
}

/// Category for grouping settings in the UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SettingCategory {
    /// Grave Escape overrides
    GraveEscape,
    /// Auto Shift configuration
    AutoShift,
    /// One Shot Keys
    OneShotKeys,
    /// Tapping behavior
    Tapping,
    /// Combo settings
    Combo,
    /// Mouse Keys
    MouseKeys,
    /// Magic / Key Swap settings
    Magic,
    /// Tap timing
    TapTiming,
    /// Pointing device / trackpad
    Pointing,
}

impl SettingCategory {
    /// Human-readable label for this category.
    pub fn label(&self) -> &'static str {
        match self {
            Self::GraveEscape => "Grave Escape",
            Self::AutoShift => "Auto Shift",
            Self::OneShotKeys => "One Shot Keys",
            Self::Tapping => "Tapping Behavior",
            Self::Combo => "Combos",
            Self::MouseKeys => "Mouse Keys",
            Self::Magic => "Key Swaps & Toggles",
            Self::TapTiming => "Tap Timing",
            Self::Pointing => "Pointing Device",
        }
    }

    /// All categories in display order.
    pub fn all() -> &'static [SettingCategory] {
        &[
            Self::Magic,
            Self::Tapping,
            Self::AutoShift,
            Self::GraveEscape,
            Self::Combo,
            Self::OneShotKeys,
            Self::MouseKeys,
            Self::TapTiming,
            Self::Pointing,
        ]
    }
}

/// UI hint for how to render a numeric setting.
#[derive(Debug, Clone, Copy)]
pub struct SliderRange {
    pub min:    f32,
    pub max:    f32,
    pub step:   f32,
    pub suffix: &'static str,
}

/// Complete metadata for a single QMK setting.
#[derive(Debug, Clone)]
pub struct SettingDef {
    /// Setting ID as sent over the wire
    pub id:           u16,
    /// Human-readable name
    pub name:         &'static str,
    /// Description / tooltip
    pub description:  &'static str,
    /// Category for UI grouping
    pub category:     SettingCategory,
    /// Value type
    pub setting_type: SettingType,
    /// Optional slider range for numeric settings
    pub range:        Option<SliderRange>,
}

/// Returns a map of all known QMK setting definitions, keyed by setting ID.
pub fn all_setting_defs() -> HashMap<u16, SettingDef> {
    let defs = vec![
        // ── Grave Escape ──
        SettingDef {
            id:           QS_GRAVE_ESC_OVERRIDE,
            name:         "Grave Escape Override",
            description:  "Configure which modifiers cause Grave Escape to send Grave instead of Escape",
            category:     SettingCategory::GraveEscape,
            setting_type: SettingType::U8,
            range:        None,
        },
        // ── Auto Shift ──
        SettingDef {
            id:           QS_AUTO_SHIFT,
            name:         "Auto Shift Flags",
            description:  "Auto Shift enable and behavior flags",
            category:     SettingCategory::AutoShift,
            setting_type: SettingType::U8,
            range:        None,
        },
        SettingDef {
            id:           QS_AUTO_SHIFT_TIMEOUT,
            name:         "Auto Shift Timeout",
            description:  "How long a key must be held before auto-shifting (ms)",
            category:     SettingCategory::AutoShift,
            setting_type: SettingType::U16,
            range:        Some(SliderRange {
                min:    50.0,
                max:    500.0,
                step:   5.0,
                suffix: " ms",
            }),
        },
        // ── One Shot Keys ──
        SettingDef {
            id:           QS_OSK_TAP_TOGGLE,
            name:         "One Shot Tap Toggle",
            description:  "Number of taps to toggle one-shot key lock",
            category:     SettingCategory::OneShotKeys,
            setting_type: SettingType::U8,
            range:        Some(SliderRange {
                min:    1.0,
                max:    10.0,
                step:   1.0,
                suffix: "",
            }),
        },
        SettingDef {
            id:           QS_OSK_TIMEOUT,
            name:         "One Shot Timeout",
            description:  "Time before one-shot key expires (ms)",
            category:     SettingCategory::OneShotKeys,
            setting_type: SettingType::U16,
            range:        Some(SliderRange {
                min:    100.0,
                max:    10000.0,
                step:   50.0,
                suffix: " ms",
            }),
        },
        // ── Tapping ──
        SettingDef {
            id:           QS_TAPPING_TERM,
            name:         "Tapping Term",
            description:  "Time window for tap vs hold detection (ms)",
            category:     SettingCategory::Tapping,
            setting_type: SettingType::U16,
            range:        Some(SliderRange {
                min:    50.0,
                max:    500.0,
                step:   5.0,
                suffix: " ms",
            }),
        },
        SettingDef {
            id:           QS_TAPPING_V2_LEGACY,
            name:         "Tapping Behavior (Legacy)",
            description:  "Legacy tapping behavior flags (older firmware)",
            category:     SettingCategory::Tapping,
            setting_type: SettingType::U8,
            range:        None,
        },
        SettingDef {
            id:           QS_TAPPING_TOGGLE,
            name:         "Tapping Toggle",
            description:  "Number of taps to toggle a layer with TT()",
            category:     SettingCategory::Tapping,
            setting_type: SettingType::U8,
            range:        Some(SliderRange {
                min:    1.0,
                max:    10.0,
                step:   1.0,
                suffix: "",
            }),
        },
        SettingDef {
            id:           QS_PERMISSIVE_HOLD,
            name:         "Permissive Hold",
            description:  "Allow hold action when another key is tapped during the tapping term",
            category:     SettingCategory::Tapping,
            setting_type: SettingType::Bool,
            range:        None,
        },
        SettingDef {
            id:           QS_HOLD_ON_OTHER_KEY_PRESS,
            name:         "Hold On Other Key Press",
            description:  "Immediately activate hold when another key is pressed",
            category:     SettingCategory::Tapping,
            setting_type: SettingType::Bool,
            range:        None,
        },
        SettingDef {
            id:           QS_RETRO_TAPPING,
            name:         "Retro Tapping",
            description:  "Send the tap keycode when releasing a held dual-function key without pressing another key",
            category:     SettingCategory::Tapping,
            setting_type: SettingType::Bool,
            range:        None,
        },
        SettingDef {
            id:           QS_QUICK_TAP_TERM,
            name:         "Quick Tap Term",
            description:  "Time window for quick tap repeat (ms)",
            category:     SettingCategory::Tapping,
            setting_type: SettingType::U16,
            range:        Some(SliderRange {
                min:    0.0,
                max:    500.0,
                step:   5.0,
                suffix: " ms",
            }),
        },
        SettingDef {
            id:           QS_CHORDAL_HOLD,
            name:         "Chordal Hold",
            description:  "Use chordal hold detection for tap-hold keys",
            category:     SettingCategory::Tapping,
            setting_type: SettingType::Bool,
            range:        None,
        },
        SettingDef {
            id:           QS_FLOW_TAP_TERM,
            name:         "Flow Tap Term",
            description:  "Time window for flow tap detection (ms, 0 = disabled)",
            category:     SettingCategory::Tapping,
            setting_type: SettingType::U16,
            range:        Some(SliderRange {
                min:    0.0,
                max:    500.0,
                step:   5.0,
                suffix: " ms",
            }),
        },
        // ── Combo ──
        SettingDef {
            id:           QS_COMBO_TERM,
            name:         "Combo Term",
            description:  "Time window for combo key detection (ms)",
            category:     SettingCategory::Combo,
            setting_type: SettingType::U16,
            range:        Some(SliderRange {
                min:    10.0,
                max:    500.0,
                step:   5.0,
                suffix: " ms",
            }),
        },
        // ── Mouse Keys ──
        SettingDef {
            id:           QS_MOUSEKEY_DELAY,
            name:         "Mouse Key Delay",
            description:  "Initial delay before mouse movement starts (ms)",
            category:     SettingCategory::MouseKeys,
            setting_type: SettingType::U16,
            range:        Some(SliderRange {
                min:    0.0,
                max:    1000.0,
                step:   10.0,
                suffix: " ms",
            }),
        },
        SettingDef {
            id:           QS_MOUSEKEY_INTERVAL,
            name:         "Mouse Key Interval",
            description:  "Time between mouse movement steps (ms)",
            category:     SettingCategory::MouseKeys,
            setting_type: SettingType::U16,
            range:        Some(SliderRange {
                min:    1.0,
                max:    100.0,
                step:   1.0,
                suffix: " ms",
            }),
        },
        SettingDef {
            id:           QS_MOUSEKEY_MOVE_DELTA,
            name:         "Mouse Key Move Delta",
            description:  "Base mouse movement speed (pixels)",
            category:     SettingCategory::MouseKeys,
            setting_type: SettingType::U16,
            range:        Some(SliderRange {
                min:    1.0,
                max:    50.0,
                step:   1.0,
                suffix: " px",
            }),
        },
        SettingDef {
            id:           QS_MOUSEKEY_MAX_SPEED,
            name:         "Mouse Key Max Speed",
            description:  "Maximum mouse movement speed multiplier",
            category:     SettingCategory::MouseKeys,
            setting_type: SettingType::U16,
            range:        Some(SliderRange {
                min:    1.0,
                max:    50.0,
                step:   1.0,
                suffix: "x",
            }),
        },
        SettingDef {
            id:           QS_MOUSEKEY_TIME_TO_MAX,
            name:         "Mouse Key Time to Max",
            description:  "Time to reach max mouse speed (steps)",
            category:     SettingCategory::MouseKeys,
            setting_type: SettingType::U16,
            range:        Some(SliderRange {
                min:    1.0,
                max:    100.0,
                step:   1.0,
                suffix: "",
            }),
        },
        SettingDef {
            id:           QS_MOUSEKEY_WHEEL_DELAY,
            name:         "Mouse Wheel Delay",
            description:  "Initial delay before scroll starts (ms)",
            category:     SettingCategory::MouseKeys,
            setting_type: SettingType::U16,
            range:        Some(SliderRange {
                min:    0.0,
                max:    1000.0,
                step:   10.0,
                suffix: " ms",
            }),
        },
        SettingDef {
            id:           QS_MOUSEKEY_WHEEL_INTERVAL,
            name:         "Mouse Wheel Interval",
            description:  "Time between scroll steps (ms)",
            category:     SettingCategory::MouseKeys,
            setting_type: SettingType::U16,
            range:        Some(SliderRange {
                min:    1.0,
                max:    200.0,
                step:   1.0,
                suffix: " ms",
            }),
        },
        SettingDef {
            id:           QS_MOUSEKEY_WHEEL_MAX_SPEED,
            name:         "Mouse Wheel Max Speed",
            description:  "Maximum scroll speed multiplier",
            category:     SettingCategory::MouseKeys,
            setting_type: SettingType::U16,
            range:        Some(SliderRange {
                min:    1.0,
                max:    20.0,
                step:   1.0,
                suffix: "x",
            }),
        },
        SettingDef {
            id:           QS_MOUSEKEY_WHEEL_TIME_TO_MAX,
            name:         "Mouse Wheel Time to Max",
            description:  "Time to reach max scroll speed (steps)",
            category:     SettingCategory::MouseKeys,
            setting_type: SettingType::U16,
            range:        Some(SliderRange {
                min:    1.0,
                max:    100.0,
                step:   1.0,
                suffix: "",
            }),
        },
        // ── Magic (Key Swaps & Toggles) ──
        SettingDef {
            id:           QS_MAGIC,
            name:         "Magic Settings",
            description:  "Key swap and toggle flags (NKRO, GUI disable, key swaps)",
            category:     SettingCategory::Magic,
            setting_type: SettingType::U32,
            range:        None,
        },
        // ── Tap Timing ──
        SettingDef {
            id:           QS_TAP_CODE_DELAY,
            name:         "Tap Code Delay",
            description:  "Delay between key down and key up when sending tap codes (ms)",
            category:     SettingCategory::TapTiming,
            setting_type: SettingType::U16,
            range:        Some(SliderRange {
                min:    0.0,
                max:    100.0,
                step:   1.0,
                suffix: " ms",
            }),
        },
        SettingDef {
            id:           QS_TAP_HOLD_CAPS_DELAY,
            name:         "Tap Hold Caps Delay",
            description:  "Delay for Caps Lock tap-hold (ms)",
            category:     SettingCategory::TapTiming,
            setting_type: SettingType::U16,
            range:        Some(SliderRange {
                min:    0.0,
                max:    500.0,
                step:   5.0,
                suffix: " ms",
            }),
        },
        // ── Pointing Device ──
        SettingDef {
            id:           QS_POINTING_DPI,
            name:         "DPI / CPI",
            description:  "Pointing device sensitivity",
            category:     SettingCategory::Pointing,
            setting_type: SettingType::U16,
            range:        Some(SliderRange {
                min:    100.0,
                max:    16000.0,
                step:   100.0,
                suffix: " DPI",
            }),
        },
        SettingDef {
            id:           QS_POINTING_SCROLL_DIVISOR,
            name:         "Scroll Divisor",
            description:  "Scroll speed divisor (higher = slower)",
            category:     SettingCategory::Pointing,
            setting_type: SettingType::U8,
            range:        Some(SliderRange {
                min:    1.0,
                max:    64.0,
                step:   1.0,
                suffix: "",
            }),
        },
        SettingDef {
            id:           QS_POINTING_SCROLL_DIVISOR_H,
            name:         "Horizontal Scroll Divisor",
            description:  "Horizontal scroll speed divisor",
            category:     SettingCategory::Pointing,
            setting_type: SettingType::U8,
            range:        Some(SliderRange {
                min:    1.0,
                max:    64.0,
                step:   1.0,
                suffix: "",
            }),
        },
        SettingDef {
            id:           QS_POINTING_INVERT_X,
            name:         "Invert X Axis",
            description:  "Invert horizontal pointing axis",
            category:     SettingCategory::Pointing,
            setting_type: SettingType::Bool,
            range:        None,
        },
        SettingDef {
            id:           QS_POINTING_INVERT_Y,
            name:         "Invert Y Axis",
            description:  "Invert vertical pointing axis",
            category:     SettingCategory::Pointing,
            setting_type: SettingType::Bool,
            range:        None,
        },
        SettingDef {
            id:           QS_POINTING_INVERT_SCROLL,
            name:         "Invert Scroll",
            description:  "Invert scroll direction",
            category:     SettingCategory::Pointing,
            setting_type: SettingType::Bool,
            range:        None,
        },
        SettingDef {
            id:           QS_POINTING_DRAG_SCROLL,
            name:         "Drag Scroll Mode",
            description:  "Enable drag scroll",
            category:     SettingCategory::Pointing,
            setting_type: SettingType::Bool,
            range:        None,
        },
        SettingDef {
            id:           QS_POINTING_DRAG_SCROLL_DIVISOR,
            name:         "Drag Scroll Divisor",
            description:  "Drag scroll speed divisor",
            category:     SettingCategory::Pointing,
            setting_type: SettingType::U8,
            range:        Some(SliderRange {
                min:    1.0,
                max:    64.0,
                step:   1.0,
                suffix: "",
            }),
        },
        SettingDef {
            id:           QS_POINTING_DPI_2,
            name:         "Secondary DPI",
            description:  "DPI for second pointing device",
            category:     SettingCategory::Pointing,
            setting_type: SettingType::U16,
            range:        Some(SliderRange {
                min:    100.0,
                max:    16000.0,
                step:   100.0,
                suffix: " DPI",
            }),
        },
        SettingDef {
            id:           QS_POINTING_SNIPING_DPI,
            name:         "Sniping DPI",
            description:  "DPI when sniping mode is active",
            category:     SettingCategory::Pointing,
            setting_type: SettingType::U16,
            range:        Some(SliderRange {
                min:    50.0,
                max:    4000.0,
                step:   50.0,
                suffix: " DPI",
            }),
        },
        SettingDef {
            id:           QS_POINTING_AUTO_MOUSE_ENABLE,
            name:         "Auto Mouse Enable",
            description:  "Automatically switch to mouse layer on trackpad movement",
            category:     SettingCategory::Pointing,
            setting_type: SettingType::Bool,
            range:        None,
        },
        SettingDef {
            id:           QS_POINTING_AUTO_MOUSE_LAYER,
            name:         "Auto Mouse Layer",
            description:  "Layer to activate for auto mouse",
            category:     SettingCategory::Pointing,
            setting_type: SettingType::U8,
            range:        Some(SliderRange {
                min:    0.0,
                max:    15.0,
                step:   1.0,
                suffix: "",
            }),
        },
        SettingDef {
            id:           QS_POINTING_AUTO_MOUSE_TIMEOUT,
            name:         "Auto Mouse Timeout",
            description:  "Time before auto mouse layer deactivates (ms)",
            category:     SettingCategory::Pointing,
            setting_type: SettingType::U16,
            range:        Some(SliderRange {
                min:    50.0,
                max:    5000.0,
                step:   50.0,
                suffix: " ms",
            }),
        },
    ];

    defs.into_iter().map(|d| (d.id, d)).collect()
}

/// Metadata for the individual boolean flags within the Magic setting (ID 21).
#[derive(Debug, Clone)]
pub struct MagicFlag {
    /// Bit position within the u32
    pub bit:         u8,
    /// Human-readable name
    pub name:        &'static str,
    /// Description
    pub description: &'static str,
}

/// Returns all magic flags in display order.
pub fn magic_flags() -> Vec<MagicFlag> {
    vec![
        MagicFlag {
            bit:         MAGIC_SWAP_CONTROL_CAPSLOCK,
            name:        "Swap Ctrl ↔ Caps Lock",
            description: "Swap Control and Caps Lock keys",
        },
        MagicFlag {
            bit:         MAGIC_CAPSLOCK_TO_CONTROL,
            name:        "Caps Lock → Control",
            description: "Make Caps Lock act as Control",
        },
        MagicFlag {
            bit:         MAGIC_SWAP_LALT_LGUI,
            name:        "Swap Left Alt ↔ Left GUI",
            description: "Swap Left Alt and Left GUI (macOS mode)",
        },
        MagicFlag {
            bit:         MAGIC_SWAP_RALT_RGUI,
            name:        "Swap Right Alt ↔ Right GUI",
            description: "Swap Right Alt and Right GUI",
        },
        MagicFlag {
            bit:         MAGIC_NO_GUI,
            name:        "Disable GUI Key",
            description: "Disable the GUI/Windows/Command key",
        },
        MagicFlag {
            bit:         MAGIC_SWAP_GRAVE_ESC,
            name:        "Swap Grave ↔ Escape",
            description: "Swap ` (grave) and Escape keys",
        },
        MagicFlag {
            bit:         MAGIC_SWAP_BACKSLASH_BACKSPACE,
            name:        "Swap \\ ↔ Backspace",
            description: "Swap Backslash and Backspace keys",
        },
        MagicFlag {
            bit:         MAGIC_NKRO,
            name:        "NKRO (N-Key Rollover)",
            description: "Enable full N-Key Rollover over USB",
        },
        MagicFlag {
            bit:         MAGIC_SWAP_LCTL_LGUI,
            name:        "Swap Left Ctrl ↔ Left GUI",
            description: "Swap Left Control and Left GUI",
        },
        MagicFlag {
            bit:         MAGIC_SWAP_RCTL_RGUI,
            name:        "Swap Right Ctrl ↔ Right GUI",
            description: "Swap Right Control and Right GUI",
        },
    ]
}

/// Metadata for the individual boolean flags within the Auto Shift setting (ID 3).
#[derive(Debug, Clone)]
pub struct AutoShiftFlag {
    /// Bit position within the u8
    pub bit:         u8,
    /// Human-readable name
    pub name:        &'static str,
    /// Description
    pub description: &'static str,
}

/// Returns all auto shift flags in display order.
pub fn auto_shift_flags() -> Vec<AutoShiftFlag> {
    vec![
        AutoShiftFlag {
            bit:         AUTO_SHIFT_ENABLE,
            name:        "Enable Auto Shift",
            description: "Enable auto shift feature",
        },
        AutoShiftFlag {
            bit:         AUTO_SHIFT_MODIFIERS,
            name:        "Include Modifiers",
            description: "Auto shift also applies when modifiers are held",
        },
        AutoShiftFlag {
            bit:         AUTO_SHIFT_NO_SPECIAL,
            name:        "Exclude Special Keys",
            description: "Don't auto shift special keys",
        },
        AutoShiftFlag {
            bit:         AUTO_SHIFT_NO_NUMERIC,
            name:        "Exclude Numeric Keys",
            description: "Don't auto shift number keys",
        },
        AutoShiftFlag {
            bit:         AUTO_SHIFT_NO_ALPHA,
            name:        "Exclude Alpha Keys",
            description: "Don't auto shift letter keys",
        },
        AutoShiftFlag {
            bit:         AUTO_SHIFT_REPEAT,
            name:        "Enable Repeat",
            description: "Allow key repeat with auto shift",
        },
        AutoShiftFlag {
            bit:         AUTO_SHIFT_NO_AUTO_REPEAT,
            name:        "Disable Auto Repeat",
            description: "Disable automatic key repeat",
        },
    ]
}

/// Metadata for the Grave Escape override flags (ID 1).
#[derive(Debug, Clone)]
pub struct GraveEscFlag {
    /// Bit position within the u8
    pub bit:         u8,
    /// Human-readable name
    pub name:        &'static str,
    /// Description
    pub description: &'static str,
}

/// Returns all grave escape override flags in display order.
pub fn grave_esc_flags() -> Vec<GraveEscFlag> {
    vec![
        GraveEscFlag {
            bit:         GRAVE_ESC_ALT_OVERRIDE,
            name:        "Alt Override",
            description: "Send Grave when Alt is held",
        },
        GraveEscFlag {
            bit:         GRAVE_ESC_CTRL_OVERRIDE,
            name:        "Ctrl Override",
            description: "Send Grave when Ctrl is held",
        },
        GraveEscFlag {
            bit:         GRAVE_ESC_GUI_OVERRIDE,
            name:        "GUI Override",
            description: "Send Grave when GUI is held",
        },
        GraveEscFlag {
            bit:         GRAVE_ESC_SHIFT_OVERRIDE,
            name:        "Shift Override",
            description: "Send Grave when Shift is held",
        },
    ]
}

/// Returns true if the given setting ID is in the pointing device range.
pub fn is_pointing_setting(id: u16) -> bool {
    (0x0100..=0x01FF).contains(&id)
}

/// Returns true if the given setting ID is a core QMK setting (non-pointing).
pub fn is_core_setting(id: u16) -> bool {
    id >= 1 && id <= 27
}

/// Metadata for legacy tapping v2 flags (ID 8, older firmware).
#[derive(Debug, Clone)]
pub struct LegacyTappingFlag {
    /// Bit position within the u8
    pub bit:         u8,
    /// Human-readable name
    pub name:        &'static str,
    /// Description
    pub description: &'static str,
}

/// Returns all legacy tapping v2 flags in display order.
pub fn legacy_tapping_flags() -> Vec<LegacyTappingFlag> {
    vec![
        LegacyTappingFlag {
            bit:         LEGACY_TAPPING_PERMISSIVE_HOLD,
            name:        "Permissive Hold",
            description: "Allow hold action when another key is tapped during the tapping term",
        },
        LegacyTappingFlag {
            bit:         LEGACY_TAPPING_IGNORE_MOD_TAP,
            name:        "Ignore Mod Tap Interrupt",
            description: "Equivalent to Hold On Other Key Press in newer firmware",
        },
        LegacyTappingFlag {
            bit:         LEGACY_TAPPING_FORCE_HOLD,
            name:        "Tapping Force Hold",
            description: "Force hold action, disabling quick tap",
        },
        LegacyTappingFlag {
            bit:         LEGACY_TAPPING_RETRO,
            name:        "Retro Tapping",
            description: "Send tap keycode when releasing a held dual-function key",
        },
    ]
}
