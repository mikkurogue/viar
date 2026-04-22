use via_protocol::{
    ComboEntry,
    DynamicEntryCounts,
    KeyOverrideEntry,
    TapDanceEntry,
};

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
        }
    }
}
