use std::collections::HashMap;

use via_protocol::{
    ComboEntry,
    DynamicEntryCounts,
    KeyOverrideEntry,
    TapDanceEntry,
};

/// Identifies which keycode field is currently selected for the shared picker.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ActiveKeycodeField {
    /// Tap dance field: (entry_index, field_name)
    TapDance(usize, TapDanceField),
    /// Combo field: (entry_index, field_variant)
    Combo(usize, ComboField),
    /// Key override field: (entry_index, field_variant)
    KeyOverride(usize, KeyOverrideField),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TapDanceField {
    OnTap,
    OnHold,
    OnDoubleTap,
    OnTapHold,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ComboField {
    Input(usize), // 0..3
    Output,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KeyOverrideField {
    Trigger,
    Replacement,
}

/// Dynamic entry data loaded from the device.
pub struct DynamicEntryData {
    pub counts: DynamicEntryCounts,
    pub tap_dances: Vec<TapDanceEntry>,
    pub combos: Vec<ComboEntry>,
    pub key_overrides: Vec<KeyOverrideEntry>,
    /// Index of the entry currently being edited (per type)
    pub editing_tap_dance: Option<usize>,
    pub editing_combo: Option<usize>,
    pub editing_key_override: Option<usize>,
    /// Which keycode field is active for the shared picker
    pub active_field: Option<ActiveKeycodeField>,
    /// Which picker group tab is selected
    pub picker_group_idx: usize,
    /// User-defined aliases for dynamic entries.
    /// Keys are like "td:0", "combo:3", "ko:1". Values are custom names.
    pub aliases: HashMap<String, String>,
    /// Which alias is currently being edited inline (the alias key, e.g. "td:0")
    pub editing_alias: Option<String>,
}

impl DynamicEntryData {
    pub fn new(
        counts: DynamicEntryCounts,
        tap_dances: Vec<TapDanceEntry>,
        combos: Vec<ComboEntry>,
        key_overrides: Vec<KeyOverrideEntry>,
    ) -> Self {
        Self {
            counts,
            tap_dances,
            combos,
            key_overrides,
            editing_tap_dance: None,
            editing_combo: None,
            editing_key_override: None,
            active_field: None,
            picker_group_idx: 0,
            aliases: HashMap::new(),
            editing_alias: None,
        }
    }

    /// Get the display name for a dynamic entry, using alias if set, otherwise the default.
    pub fn display_name(&self, key: &str, default: &str) -> String {
        self.aliases
            .get(key)
            .filter(|s| !s.is_empty())
            .cloned()
            .unwrap_or_else(|| default.to_string())
    }

    /// Get the tap dance display name.
    pub fn td_name(&self, idx: usize) -> String {
        self.display_name(&format!("td:{idx}"), &format!("TD({idx})"))
    }

    /// Get the combo display name.
    pub fn combo_name(&self, idx: usize) -> String {
        self.display_name(&format!("combo:{idx}"), &format!("C{idx}"))
    }

    /// Get the key override display name.
    pub fn ko_name(&self, idx: usize) -> String {
        self.display_name(&format!("ko:{idx}"), &format!("KO{idx}"))
    }
}
