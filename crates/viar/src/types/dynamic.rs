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
        }
    }
}
