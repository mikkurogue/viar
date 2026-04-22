use via_protocol::KeyboardLayout;

/// A single keycode change for undo tracking.
#[derive(Clone)]
pub struct KeyChange {
    pub layer:       usize,
    pub row:         u8,
    pub col:         u8,
    pub key_idx:     usize,
    pub old_keycode: u16,
    pub new_keycode: u16,
}

/// Loaded keymap data for display.
pub struct KeymapData {
    pub layout:         KeyboardLayout,
    /// keymap[layer][row][col] = raw keycode u16
    pub keymap:         Vec<Vec<Vec<u16>>>,
    pub layer_count:    u8,
    pub selected_layer: usize,
    pub selected_key:   Option<usize>,
    /// Whether keymap has unsaved changes
    pub dirty:          bool,
    /// Undo history
    pub undo_stack:     Vec<KeyChange>,
}
