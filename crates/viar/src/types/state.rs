use std::sync::mpsc;

use via_protocol::{
    KeyboardDevice,
    KeyboardInfo,
    KeycodeGroup,
};

use super::{
    AppScreen,
    ConfirmDialog,
    ConnectedTab,
    DynamicEntryData,
    KeymapData,
    LightingData,
    PointingData,
    StatusMessage,
};
use crate::theme::{
    Theme,
    ViarConfig,
};

/// Result of background HID detection.
pub enum DetectResult {
    Ok {
        api:       hidapi::HidApi,
        keyboards: Vec<KeyboardInfo>,
    },
    NoPermission,
    NoViaDevices,
    InitFailed(String),
}

/// The main application state.
pub struct ViarApp {
    pub hid_api: Option<hidapi::HidApi>,
    pub keyboards: Vec<KeyboardInfo>,
    pub connected_device: Option<KeyboardDevice>,
    pub protocol_version: Option<u16>,
    pub screen: AppScreen,
    pub keymap_data: Option<KeymapData>,
    /// Picker state
    pub picker_groups: Vec<KeycodeGroup>,
    pub picker_selected_group: usize,
    /// Status bar message
    pub status: Option<StatusMessage>,
    /// Pending confirmation dialog
    pub confirm_dialog: Option<ConfirmDialog>,
    /// Active tab in connected view
    pub active_tab: ConnectedTab,
    /// Lighting state
    pub lighting_data: Option<LightingData>,
    /// Dynamic entries (tap dance, combos, key overrides)
    pub dynamic_data: Option<DynamicEntryData>,
    /// Pointing device / trackpad settings
    pub pointing_data: Option<PointingData>,
    /// Receiver for background HID detection result
    pub detect_rx: Option<mpsc::Receiver<DetectResult>>,
    /// Persistent config
    pub config: ViarConfig,
    /// Active theme
    pub theme: Theme,
}
