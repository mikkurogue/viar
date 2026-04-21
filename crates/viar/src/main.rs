use eframe::egui;
use tracing::{debug, info, warn};
use via_protocol::{
    device::{check_hid_permissions, discover_keyboards},
    keycode_groups,
    layout::{find_layout, generic_layout},
    HidAccessStatus, KeyboardDevice, KeyboardInfo, KeyboardLayout, Keycode, KeycodeGroup,
    LightingChannel, ViaProtocol,
};

fn main() -> eframe::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                tracing_subscriber::EnvFilter::new(
                    "info,winit=warn,eframe=warn,egui=warn,wgpu=warn,naga=warn",
                )
            }),
        )
        .init();

    info!("starting viar");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1100.0, 750.0])
            .with_min_inner_size([700.0, 500.0])
            .with_title("Viar — Keyboard Configurator"),
        ..Default::default()
    };

    eframe::run_native(
        "Viar",
        options,
        Box::new(|cc| {
            // Load JetBrains Mono as the default proportional + monospace font
            let mut fonts = egui::FontDefinitions::default();
            let font_data = std::sync::Arc::new(egui::FontData::from_static(include_bytes!(
                "../assets/JetBrainsMono-Regular.ttf"
            )));
            fonts
                .font_data
                .insert("JetBrainsMono".to_owned(), font_data);
            // Put it first in both proportional and monospace families
            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .insert(0, "JetBrainsMono".to_owned());
            fonts
                .families
                .entry(egui::FontFamily::Monospace)
                .or_default()
                .insert(0, "JetBrainsMono".to_owned());
            cc.egui_ctx.set_fonts(fonts);

            Ok(Box::new(ViarApp::new()))
        }),
    )
}

/// The current screen/state of the application.
enum AppScreen {
    Detecting,
    NoPermission(String),
    NoKeyboards,
    SelectKeyboard,
    Loading,
    Connected,
}

/// Which main tab is active in the connected view.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConnectedTab {
    Keymap,
    Lighting,
}

/// A single keycode change for undo tracking.
#[derive(Clone)]
struct KeyChange {
    layer: usize,
    row: u8,
    col: u8,
    key_idx: usize,
    old_keycode: u16,
    new_keycode: u16,
}

/// Loaded keymap data for display.
struct KeymapData {
    layout: KeyboardLayout,
    /// keymap[layer][row][col] = raw keycode u16
    keymap: Vec<Vec<Vec<u16>>>,
    layer_count: u8,
    selected_layer: usize,
    selected_key: Option<usize>,
    /// Whether keymap has unsaved changes
    dirty: bool,
    /// Undo history
    undo_stack: Vec<KeyChange>,
}

/// Lighting state loaded from the device.
struct LightingData {
    channel: LightingChannel,
    brightness: u8,
    effect: u8,
    speed: u8,
    hue: u8,
    saturation: u8,
    /// Whether lighting values have been modified since last save
    dirty: bool,
}

/// Status message shown temporarily after an action.
struct StatusMessage {
    text: String,
    is_error: bool,
    expire_at: std::time::Instant,
}

impl StatusMessage {
    fn info(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            is_error: false,
            expire_at: std::time::Instant::now() + std::time::Duration::from_secs(3),
        }
    }
    fn error(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            is_error: true,
            expire_at: std::time::Instant::now() + std::time::Duration::from_secs(5),
        }
    }
    fn is_expired(&self) -> bool {
        std::time::Instant::now() >= self.expire_at
    }
}

struct ViarApp {
    hid_api: Option<hidapi::HidApi>,
    keyboards: Vec<KeyboardInfo>,
    connected_device: Option<KeyboardDevice>,
    protocol_version: Option<u16>,
    screen: AppScreen,
    keymap_data: Option<KeymapData>,
    /// Picker state
    picker_groups: Vec<KeycodeGroup>,
    picker_selected_group: usize,
    /// Status bar message
    status: Option<StatusMessage>,
    /// Pending confirmation dialog
    confirm_dialog: Option<ConfirmDialog>,
    /// Active tab in connected view
    active_tab: ConnectedTab,
    /// Lighting state
    lighting_data: Option<LightingData>,
}

/// A modal confirmation dialog.
struct ConfirmDialog {
    title: String,
    message: String,
    action: ConfirmAction,
}

enum ConfirmAction {
    Import,
}

impl ViarApp {
    fn new() -> Self {
        let mut app = Self {
            hid_api: None,
            keyboards: Vec::new(),
            connected_device: None,
            protocol_version: None,
            screen: AppScreen::Detecting,
            keymap_data: None,
            picker_groups: keycode_groups(),
            picker_selected_group: 0,
            status: None,
            confirm_dialog: None,
            active_tab: ConnectedTab::Keymap,
            lighting_data: None,
        };
        app.detect();
        app
    }

    fn set_status(&mut self, msg: StatusMessage) {
        self.status = Some(msg);
    }

    fn detect(&mut self) {
        info!("detecting keyboards");
        match check_hid_permissions() {
            HidAccessStatus::InitFailed(msg) => {
                warn!(error = %msg, "HID init failed");
                self.screen = AppScreen::NoPermission(msg);
            }
            HidAccessStatus::NoPermission => {
                warn!("no HID devices visible — likely permission issue");
                self.screen = AppScreen::NoPermission(
                    "No permission to access HID devices without root.\n\
                     Consider adding a udev rule for your keyboard.\n\n\
                     Example: KERNEL==\"hidraw*\", SUBSYSTEM==\"hidraw\", MODE=\"0666\""
                        .to_string(),
                );
            }
            HidAccessStatus::NoViaDevices => {
                info!("no VIA keyboards found");
                self.screen = AppScreen::NoKeyboards;
            }
            HidAccessStatus::Ok => match hidapi::HidApi::new() {
                Ok(api) => {
                    self.keyboards = discover_keyboards(&api);
                    self.hid_api = Some(api);
                    if self.keyboards.len() == 1 {
                        self.connect_to_keyboard(0);
                    } else {
                        self.screen = AppScreen::SelectKeyboard;
                    }
                }
                Err(e) => {
                    self.screen = AppScreen::NoPermission(format!("HID init failed: {e}"));
                }
            },
        }
    }

    fn refresh(&mut self) {
        self.connected_device = None;
        self.protocol_version = None;
        self.keymap_data = None;
        self.keyboards.clear();

        if let Some(api) = &mut self.hid_api {
            if let Err(e) = api.refresh_devices() {
                warn!(error = %e, "failed to refresh HID devices");
                self.screen = AppScreen::NoPermission(format!("Failed to refresh: {e}"));
                return;
            }
            self.keyboards = discover_keyboards(api);
            if self.keyboards.is_empty() {
                self.screen = AppScreen::NoKeyboards;
            } else if self.keyboards.len() == 1 {
                self.connect_to_keyboard(0);
            } else {
                self.screen = AppScreen::SelectKeyboard;
            }
        } else {
            self.detect();
        }
    }

    fn connect_to_keyboard(&mut self, idx: usize) {
        let info = self.keyboards[idx].clone();
        if let Some(api) = &self.hid_api {
            match KeyboardDevice::open(api, info.clone()) {
                Ok(dev) => {
                    let proto = ViaProtocol::new(&dev);
                    self.protocol_version = proto.get_protocol_version().ok();
                    info!(keyboard = %dev.info, "connected, deferring keymap load");
                    self.connected_device = Some(dev);
                    self.screen = AppScreen::Loading;
                }
                Err(e) => {
                    warn!(error = %e, "failed to connect to keyboard");
                }
            }
        }
    }

    fn load_keymap(&mut self) {
        let Some(dev) = &self.connected_device else {
            return;
        };
        let info = &dev.info;
        let layout = find_layout(info.vendor_id, info.product_id).unwrap_or_else(|| {
            debug!(
                "no built-in layout for {:04x}:{:04x}, using generic",
                info.vendor_id, info.product_id
            );
            generic_layout(4, 12)
        });

        let proto = ViaProtocol::new(dev);
        let layer_count = proto.get_layer_count().unwrap_or(4);
        info!(
            layers = layer_count,
            rows = layout.rows,
            cols = layout.cols,
            "reading keymap from device"
        );

        let keymap = match proto.read_entire_keymap(layer_count, layout.rows, layout.cols) {
            Ok(km) => {
                info!("keymap loaded successfully");
                km
            }
            Err(e) => {
                warn!(error = %e, "failed to read keymap, using empty");
                vec![
                    vec![vec![0u16; layout.cols as usize]; layout.rows as usize];
                    layer_count as usize
                ]
            }
        };

        self.keymap_data = Some(KeymapData {
            layout,
            keymap,
            layer_count,
            selected_layer: 0,
            selected_key: None,
            dirty: false,
            undo_stack: Vec::new(),
        });

        // Try to detect and load lighting
        self.lighting_data = None;
        if let Some(dev) = &self.connected_device {
            let proto = ViaProtocol::new(dev);
            if let Some(channel) = proto.detect_lighting_channel() {
                let brightness = proto.get_rgb_brightness(channel).unwrap_or(0);
                let effect = proto.get_rgb_effect(channel).unwrap_or(0);
                let speed = proto.get_rgb_speed(channel).unwrap_or(0);
                let (hue, sat) = proto.get_rgb_color(channel).unwrap_or((0, 0));
                info!(
                    ?channel,
                    brightness, effect, speed, hue, sat, "lighting loaded from device"
                );
                self.lighting_data = Some(LightingData {
                    channel,
                    brightness,
                    effect,
                    speed,
                    hue,
                    saturation: sat,
                    dirty: false,
                });
            }
        }

        self.screen = AppScreen::Connected;
    }

    fn disconnect(&mut self) {
        if self.connected_device.is_some() {
            info!("disconnecting from keyboard");
            self.connected_device = None;
            self.protocol_version = None;
            self.keymap_data = None;
            self.lighting_data = None;
            self.active_tab = ConnectedTab::Keymap;
            self.refresh();
        }
    }

    fn handle_disconnect(&mut self) {
        warn!("device disconnected unexpectedly");
        self.connected_device = None;
        self.protocol_version = None;
        self.lighting_data = None;
        self.active_tab = ConnectedTab::Keymap;
        self.screen = AppScreen::NoKeyboards;
        self.set_status(StatusMessage::error(
            "Keyboard disconnected. Plug it back in and click Refresh.",
        ));
    }

    /// Write a keycode to the device and update local state.
    fn apply_keycode(&mut self, key_idx: usize, new_keycode: u16) {
        let Some(data) = &mut self.keymap_data else {
            return;
        };
        let key_pos = &data.layout.keys[key_idx];
        let layer = data.selected_layer;
        let row = key_pos.row;
        let col = key_pos.col;

        let old_keycode = data
            .keymap
            .get(layer)
            .and_then(|l| l.get(row as usize))
            .and_then(|r| r.get(col as usize))
            .copied()
            .unwrap_or(0);

        if old_keycode == new_keycode {
            return;
        }

        // Update local state
        if let Some(layer_data) = data.keymap.get_mut(layer) {
            if let Some(row_data) = layer_data.get_mut(row as usize) {
                if let Some(cell) = row_data.get_mut(col as usize) {
                    *cell = new_keycode;
                }
            }
        }

        // Track for undo
        data.undo_stack.push(KeyChange {
            layer,
            row,
            col,
            key_idx,
            old_keycode,
            new_keycode,
        });
        data.dirty = true;

        // Write to device
        if let Some(dev) = &self.connected_device {
            let proto = ViaProtocol::new(dev);
            match proto.set_keycode(layer as u8, row, col, new_keycode) {
                Ok(()) => {
                    let kc_name = Keycode(new_keycode).name();
                    info!(
                        layer,
                        row,
                        col,
                        keycode = kc_name,
                        "keycode written to device"
                    );
                    self.set_status(StatusMessage::info(format!(
                        "Set [{row},{col}] -> {kc_name}"
                    )));
                }
                Err(e) => {
                    let err_str = format!("{e}");
                    warn!(error = %e, "failed to write keycode to device");
                    self.set_status(StatusMessage::error(format!("Write failed: {e}")));
                    // Detect device disconnect
                    if is_disconnect_error(&err_str) {
                        self.handle_disconnect();
                    }
                }
            }
        }
    }

    fn reload_keymap(&mut self) {
        if let (Some(dev), Some(data)) = (&self.connected_device, &mut self.keymap_data) {
            let proto = ViaProtocol::new(dev);
            match proto.read_entire_keymap(data.layer_count, data.layout.rows, data.layout.cols) {
                Ok(km) => {
                    info!("keymap reloaded");
                    data.keymap = km;
                    data.dirty = false;
                    data.undo_stack.clear();
                    self.set_status(StatusMessage::info("Keymap reloaded from device"));
                }
                Err(e) => {
                    warn!(error = %e, "failed to reload keymap");
                    self.set_status(StatusMessage::error(format!("Reload failed: {e}")));
                }
            }
        }
    }

    fn undo(&mut self) {
        let Some(data) = &mut self.keymap_data else {
            return;
        };
        let Some(change) = data.undo_stack.pop() else {
            return;
        };

        // Revert local state
        if let Some(layer_data) = data.keymap.get_mut(change.layer) {
            if let Some(row_data) = layer_data.get_mut(change.row as usize) {
                if let Some(cell) = row_data.get_mut(change.col as usize) {
                    *cell = change.old_keycode;
                }
            }
        }

        if data.undo_stack.is_empty() {
            data.dirty = false;
        }

        // Write old keycode back to device
        if let Some(dev) = &self.connected_device {
            let proto = ViaProtocol::new(dev);
            match proto.set_keycode(
                change.layer as u8,
                change.row,
                change.col,
                change.old_keycode,
            ) {
                Ok(()) => {
                    let name = Keycode(change.old_keycode).name();
                    info!(
                        layer = change.layer,
                        row = change.row,
                        col = change.col,
                        keycode = name,
                        "undo applied"
                    );
                    self.set_status(StatusMessage::info(format!(
                        "Undo: [{},{}] -> {name}",
                        change.row, change.col
                    )));
                }
                Err(e) => {
                    let err_str = format!("{e}");
                    warn!(error = %e, "undo write failed");
                    self.set_status(StatusMessage::error(format!("Undo write failed: {e}")));
                    if is_disconnect_error(&err_str) {
                        self.handle_disconnect();
                    }
                }
            }
        }
    }

    fn export_keymap(&mut self) {
        let Some(data) = &self.keymap_data else {
            return;
        };

        let mut layers = Vec::new();
        for (layer_idx, layer) in data.keymap.iter().enumerate() {
            let mut rows = Vec::new();
            for (row_idx, row) in layer.iter().enumerate() {
                let keys: Vec<serde_json::Value> = row
                    .iter()
                    .enumerate()
                    .map(|(col_idx, &raw_kc)| {
                        serde_json::json!({
                            "col": col_idx,
                            "raw": raw_kc,
                            "name": Keycode(raw_kc).name(),
                        })
                    })
                    .collect();
                rows.push(serde_json::json!({
                    "row": row_idx,
                    "keys": keys,
                }));
            }
            layers.push(serde_json::json!({
                "layer": layer_idx,
                "rows": rows,
            }));
        }

        let dump = serde_json::json!({
            "viar_version": 1,
            "layout": data.layout.name,
            "matrix_rows": data.layout.rows,
            "matrix_cols": data.layout.cols,
            "layer_count": data.layer_count,
            "layers": layers,
        });

        let path = "viar_keymap.json";
        match std::fs::write(path, serde_json::to_string_pretty(&dump).unwrap()) {
            Ok(_) => {
                info!("keymap exported to {path}");
                if let Some(data) = &mut self.keymap_data {
                    data.dirty = false;
                }
                self.set_status(StatusMessage::info(format!("Exported to {path}")));
            }
            Err(e) => {
                warn!(error = %e, "failed to export keymap");
                self.set_status(StatusMessage::error(format!("Export failed: {e}")));
            }
        }
    }

    fn import_keymap(&mut self) {
        let path = "viar_keymap.json";
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                warn!(error = %e, "failed to read keymap file");
                self.set_status(StatusMessage::error(format!("Import failed: {e}")));
                return;
            }
        };

        let json: serde_json::Value = match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(e) => {
                self.set_status(StatusMessage::error(format!("Invalid JSON: {e}")));
                return;
            }
        };

        let Some(data) = &self.keymap_data else {
            return;
        };

        // Validate matrix dimensions
        let file_rows = json["matrix_rows"].as_u64().unwrap_or(0) as u8;
        let file_cols = json["matrix_cols"].as_u64().unwrap_or(0) as u8;
        let expected_rows = data.layout.rows;
        let expected_cols = data.layout.cols;
        let _ = data;

        if file_rows != expected_rows || file_cols != expected_cols {
            self.set_status(StatusMessage::error(format!(
                "Matrix mismatch: file is {file_rows}x{file_cols}, keyboard is {expected_rows}x{expected_cols}",
            )));
            return;
        }

        let Some(layers) = json["layers"].as_array() else {
            self.set_status(StatusMessage::error("No layers array in file"));
            return;
        };

        let Some(data) = &mut self.keymap_data else {
            return;
        };

        // Parse keymap from JSON
        let mut new_keymap = data.keymap.clone();
        for layer_obj in layers {
            let layer_idx = layer_obj["layer"].as_u64().unwrap_or(0) as usize;
            if layer_idx >= new_keymap.len() {
                continue;
            }
            let Some(rows) = layer_obj["rows"].as_array() else {
                continue;
            };
            for row_obj in rows {
                let row_idx = row_obj["row"].as_u64().unwrap_or(0) as usize;
                if row_idx >= new_keymap[layer_idx].len() {
                    continue;
                }
                let Some(keys) = row_obj["keys"].as_array() else {
                    continue;
                };
                for key_obj in keys {
                    let col_idx = key_obj["col"].as_u64().unwrap_or(0) as usize;
                    let raw = key_obj["raw"].as_u64().unwrap_or(0) as u16;
                    if col_idx < new_keymap[layer_idx][row_idx].len() {
                        new_keymap[layer_idx][row_idx][col_idx] = raw;
                    }
                }
            }
        }

        // Write all changed keys to device
        let mut changed = 0usize;
        let mut errors = 0usize;
        if let Some(dev) = &self.connected_device {
            let proto = ViaProtocol::new(dev);
            for layer in 0..new_keymap.len() {
                for row in 0..new_keymap[layer].len() {
                    for col in 0..new_keymap[layer][row].len() {
                        let old = data.keymap[layer][row][col];
                        let new = new_keymap[layer][row][col];
                        if old != new {
                            match proto.set_keycode(layer as u8, row as u8, col as u8, new) {
                                Ok(()) => changed += 1,
                                Err(e) => {
                                    warn!(error = %e, layer, row, col, "failed to write key");
                                    errors += 1;
                                }
                            }
                        }
                    }
                }
            }
        }

        data.keymap = new_keymap;

        if errors > 0 {
            self.set_status(StatusMessage::error(format!(
                "Imported with {errors} write errors ({changed} keys updated)"
            )));
        } else {
            info!(changed, "keymap imported from {path}");
            self.set_status(StatusMessage::info(format!(
                "Imported {changed} key changes from {path}"
            )));
        }
    }
}

impl eframe::App for ViarApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Expire status messages
        if let Some(ref s) = self.status {
            if s.is_expired() {
                self.status = None;
            }
        }

        // Ctrl+Z for undo
        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::Z)) {
            self.undo();
        }

        // Update title with dirty indicator
        let dirty = self.keymap_data.as_ref().map_or(false, |d| d.dirty);
        let title = if dirty {
            "Viar — Keyboard Configurator *"
        } else {
            "Viar — Keyboard Configurator"
        };
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(title.to_string()));

        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("Viar", |ui| {
                    if ui.button("Refresh Devices").clicked() {
                        self.refresh();
                        ui.close_menu();
                    }
                    if self.connected_device.is_some() {
                        if ui.button("Disconnect").clicked() {
                            self.disconnect();
                            ui.close_menu();
                        }
                    }
                    ui.separator();
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                if self.connected_device.is_some() {
                    ui.menu_button("Device", |ui| {
                        if ui.button("Switch Keyboard...").clicked() {
                            self.connected_device = None;
                            self.protocol_version = None;
                            self.keymap_data = None;
                            self.screen = AppScreen::SelectKeyboard;
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.button("Reload Keymap").clicked() {
                            self.reload_keymap();
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.button("Export Keymap...").clicked() {
                            self.export_keymap();
                            ui.close_menu();
                        }
                        if ui.button("Import Keymap...").clicked() {
                            self.confirm_dialog = Some(ConfirmDialog {
                                title: "Import Keymap".to_string(),
                                message: "This will overwrite your current keymap with the contents of viar_keymap.json.\nAny unsaved changes will be lost.".to_string(),
                                action: ConfirmAction::Import,
                            });
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.button("Lock Keyboard").clicked() {
                            debug!("lock keyboard (not yet implemented)");
                            ui.close_menu();
                        }
                        if ui.button("Unlock Keyboard").clicked() {
                            debug!("unlock keyboard (not yet implemented)");
                            ui.close_menu();
                        }
                    });
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Status message
                    if let Some(ref status) = self.status {
                        let color = if status.is_error {
                            egui::Color32::from_rgb(220, 80, 80)
                        } else {
                            egui::Color32::from_rgb(80, 180, 80)
                        };
                        ui.colored_label(color, &status.text);
                        ui.separator();
                    }
                    if let Some(dev) = &self.connected_device {
                        ui.label(format!("Connected: {}", dev.info));
                    }
                });
            });
        });

        // Confirmation dialog
        let mut confirm_result: Option<bool> = None;
        if let Some(dialog) = &self.confirm_dialog {
            egui::Window::new(&dialog.title)
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label(&dialog.message);
                    ui.add_space(12.0);
                    ui.horizontal(|ui| {
                        if ui.button("Cancel").clicked() {
                            confirm_result = Some(false);
                        }
                        if ui
                            .add(egui::Button::new(egui::RichText::new("Import").strong()))
                            .clicked()
                        {
                            confirm_result = Some(true);
                        }
                    });
                });
        }
        if let Some(confirmed) = confirm_result {
            if confirmed {
                if let Some(dialog) = self.confirm_dialog.take() {
                    match dialog.action {
                        ConfirmAction::Import => self.import_keymap(),
                    }
                }
            } else {
                self.confirm_dialog = None;
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| match &self.screen {
            AppScreen::Detecting => self.render_detecting(ui),
            AppScreen::NoPermission(_) => self.render_no_permission(ui),
            AppScreen::NoKeyboards => self.render_no_keyboards(ui),
            AppScreen::SelectKeyboard => self.render_select_keyboard(ui),
            AppScreen::Loading => {
                ui.vertical_centered(|ui| {
                    ui.add_space(ui.available_height() / 3.0);
                    ui.heading("Loading keymap...");
                    ui.spinner();
                });
                ctx.request_repaint();
                self.load_keymap();
            }
            AppScreen::Connected => self.render_connected(ui),
        });
    }
}

impl ViarApp {
    fn render_detecting(&self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(ui.available_height() / 3.0);
            ui.heading("Detecting keyboards...");
            ui.spinner();
        });
    }

    fn render_no_permission(&self, ui: &mut egui::Ui) {
        let msg = match &self.screen {
            AppScreen::NoPermission(m) => m.clone(),
            _ => String::new(),
        };
        ui.vertical_centered(|ui| {
            ui.add_space(ui.available_height() / 3.0);
            ui.heading("Cannot access HID devices");
            ui.add_space(12.0);
            ui.label(&msg);
            ui.add_space(20.0);
            ui.label(
                "After adding a udev rule, unplug and replug your keyboard,\n\
                 or reload udev rules with: sudo udevadm control --reload-rules",
            );
        });
    }

    fn render_no_keyboards(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(ui.available_height() / 3.0);
            ui.heading("No VIA keyboards found");
            ui.add_space(12.0);
            ui.label("Make sure your keyboard firmware has VIA enabled.");
            ui.add_space(20.0);
            if ui.button("Retry").clicked() {
                self.refresh();
            }
        });
    }

    fn render_select_keyboard(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(40.0);
            ui.heading("Select a keyboard");
            ui.add_space(20.0);
        });

        let mut connect_idx = None;
        let width = 400.0_f32.min(ui.available_width() - 40.0);

        ui.vertical_centered(|ui| {
            for (i, kb) in self.keyboards.iter().enumerate() {
                let label = format!(
                    "{} {}\n{:04x}:{:04x}",
                    kb.manufacturer, kb.product, kb.vendor_id, kb.product_id
                );
                let button = egui::Button::new(egui::RichText::new(&label).size(14.0))
                    .min_size(egui::vec2(width, 50.0));
                if ui.add(button).clicked() {
                    connect_idx = Some(i);
                }
                ui.add_space(4.0);
            }
        });

        if let Some(idx) = connect_idx {
            self.connect_to_keyboard(idx);
        }
    }

    fn render_connected(&mut self, ui: &mut egui::Ui) {
        // Top tab bar: Keymap | Lighting
        ui.horizontal(|ui| {
            ui.add_space(8.0);
            if ui
                .selectable_label(self.active_tab == ConnectedTab::Keymap, "Keymap")
                .clicked()
            {
                self.active_tab = ConnectedTab::Keymap;
            }
            if self.lighting_data.is_some() {
                if ui
                    .selectable_label(self.active_tab == ConnectedTab::Lighting, "Lighting")
                    .clicked()
                {
                    self.active_tab = ConnectedTab::Lighting;
                }
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if let Some(ver) = self.protocol_version {
                    ui.label(format!("VIA v{ver}"));
                    ui.separator();
                }
                if let Some(data) = &self.keymap_data {
                    ui.label(&data.layout.name);
                }
            });
        });

        ui.separator();

        match self.active_tab {
            ConnectedTab::Keymap => self.render_keymap_tab(ui),
            ConnectedTab::Lighting => self.render_lighting_tab(ui),
        }
    }

    fn render_keymap_tab(&mut self, ui: &mut egui::Ui) {
        let Some(data) = &mut self.keymap_data else {
            ui.label("No keymap data loaded.");
            return;
        };

        // Layer tabs
        ui.horizontal(|ui| {
            ui.add_space(8.0);
            for layer in 0..data.layer_count as usize {
                let label = format!("Layer {layer}");
                let selected = data.selected_layer == layer;
                if ui.selectable_label(selected, &label).clicked() {
                    data.selected_layer = layer;
                    data.selected_key = None;
                }
            }
        });

        ui.separator();

        // Keyboard visualization area
        let available = ui.available_size();
        let layout = &data.layout;

        let layout_w = layout.width();
        let layout_h = layout.height();
        let padding = 40.0;
        let key_size = ((available.x - padding * 2.0) / layout_w)
            .min((available.y - padding * 2.0) / layout_h)
            .min(64.0)
            .max(32.0);
        let gap = 8.0; // gap between keys

        let total_w = layout_w * key_size;
        let total_h = layout_h * key_size;
        let x_offset = (available.x - total_w) / 2.0;
        let y_offset = (available.y - total_h) / 2.0;

        let origin = ui.min_rect().min + egui::vec2(x_offset, y_offset);
        let painter = ui.painter();

        let layer_idx = data.selected_layer;
        let mut clicked_key = None;
        let mut selected_key_rect: Option<egui::Rect> = None;

        for (key_idx, key_pos) in layout.keys.iter().enumerate() {
            let raw_kc = data
                .keymap
                .get(layer_idx)
                .and_then(|l| l.get(key_pos.row as usize))
                .and_then(|r| r.get(key_pos.col as usize))
                .copied()
                .unwrap_or(0);
            let keycode = Keycode(raw_kc);

            let px = origin.x + key_pos.x * key_size;
            let py = origin.y + key_pos.y * key_size;
            let pw = key_pos.w * key_size - gap;
            let ph = key_pos.h * key_size - gap;

            let rect = egui::Rect::from_min_size(egui::pos2(px, py), egui::vec2(pw, ph));

            let is_selected = data.selected_key == Some(key_idx);
            let is_hovered = ui.rect_contains_pointer(rect);

            if is_selected {
                selected_key_rect = Some(rect);
            }

            let bg_color = if is_selected {
                egui::Color32::from_rgb(70, 130, 180)
            } else if is_hovered {
                egui::Color32::from_rgb(80, 80, 90)
            } else {
                key_bg_color(&keycode)
            };

            let border_color = if is_selected {
                egui::Color32::from_rgb(100, 180, 255)
            } else {
                egui::Color32::from_rgb(60, 60, 65)
            };

            let rounding = egui::CornerRadius::same(4);
            painter.rect_filled(rect, rounding, bg_color);
            painter.rect_stroke(
                rect,
                rounding,
                egui::Stroke::new(1.0, border_color),
                egui::StrokeKind::Outside,
            );

            let label = keycode.name();
            let font_size = if label.len() <= 2 {
                key_size * 0.35
            } else if label.len() <= 5 {
                key_size * 0.25
            } else {
                key_size * 0.18
            };

            let text_color = if is_selected {
                egui::Color32::WHITE
            } else if raw_kc == 0 || raw_kc == 1 {
                egui::Color32::from_rgb(100, 100, 110)
            } else {
                egui::Color32::from_rgb(220, 220, 230)
            };

            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                &label,
                egui::FontId::proportional(font_size),
                text_color,
            );

            if is_hovered && ui.input(|i| i.pointer.primary_clicked()) {
                clicked_key = Some(key_idx);
            }
        }

        if let Some(idx) = clicked_key {
            if data.selected_key == Some(idx) {
                data.selected_key = None;
            } else {
                data.selected_key = Some(idx);
            }
        }

        // Close picker on Escape
        if data.selected_key.is_some() && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            data.selected_key = None;
        }

        // Close picker on click outside keys and picker
        if data.selected_key.is_some() && clicked_key.is_none() {
            let click_pos = ui.input(|i| {
                if i.pointer.primary_clicked() {
                    i.pointer.interact_pos()
                } else {
                    None
                }
            });
            if let Some(pos) = click_pos {
                // Check if click is inside the picker window area
                let picker_id = egui::Id::new("kc_picker");
                let in_picker = ui
                    .ctx()
                    .memory(|mem| mem.area_rect(picker_id))
                    .map_or(false, |r| r.contains(pos));
                if !in_picker {
                    data.selected_key = None;
                }
            }
        }

        // Floating popover picker
        if let (Some(key_idx), Some(key_rect)) = (data.selected_key, selected_key_rect) {
            let key_pos = &layout.keys[key_idx];
            let raw_kc = data
                .keymap
                .get(layer_idx)
                .and_then(|l| l.get(key_pos.row as usize))
                .and_then(|r| r.get(key_pos.col as usize))
                .copied()
                .unwrap_or(0);
            let keycode = Keycode(raw_kc);
            let kc_name = keycode.name();
            let kc_category = format!("{:?}", keycode.category());

            // Position the popover below the key, clamped to viewport
            let popover_w = 420.0_f32;
            let popover_h = 260.0_f32;
            let screen_rect = ui.ctx().screen_rect();

            // Try to center horizontally on the key
            let mut pop_x = key_rect.center().x - popover_w / 2.0;
            let mut pop_y = key_rect.max.y + 8.0;

            // If it would go below viewport, show above the key
            if pop_y + popover_h > screen_rect.max.y - 10.0 {
                pop_y = key_rect.min.y - popover_h - 8.0;
            }
            // Clamp horizontally
            pop_x = pop_x.clamp(screen_rect.min.x + 5.0, screen_rect.max.x - popover_w - 5.0);
            // Clamp vertically
            pop_y = pop_y.clamp(screen_rect.min.y + 5.0, screen_rect.max.y - popover_h - 5.0);

            let mut open = true;
            egui::Window::new("Keycode Picker")
                .id(egui::Id::new("kc_picker"))
                .open(&mut open)
                .fixed_pos(egui::pos2(pop_x, pop_y))
                .fixed_size(egui::vec2(popover_w, popover_h))
                .collapsible(false)
                .title_bar(false)
                .show(ui.ctx(), |ui| {
                    // Header
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(format!(
                                "Layer {} [{},{}]",
                                layer_idx, key_pos.row, key_pos.col
                            ))
                            .strong()
                            .size(13.0),
                        );
                        ui.separator();
                        ui.label(
                            egui::RichText::new(format!("{kc_name}  {:#06x}", raw_kc))
                                .size(12.0)
                                .color(egui::Color32::from_rgb(160, 160, 175)),
                        );
                        ui.separator();
                        ui.label(
                            egui::RichText::new(&kc_category)
                                .size(11.0)
                                .color(egui::Color32::from_rgb(120, 120, 135)),
                        );
                    });

                    ui.add_space(4.0);

                    // Group tabs
                    ui.horizontal_wrapped(|ui| {
                        for (i, group) in self.picker_groups.iter().enumerate() {
                            let sel = self.picker_selected_group == i;
                            let label = egui::RichText::new(group.name).size(11.5);
                            if ui.selectable_label(sel, label).clicked() {
                                self.picker_selected_group = i;
                            }
                        }
                    });

                    ui.add_space(2.0);
                    ui.separator();
                    ui.add_space(2.0);

                    // Keycode grid
                    let group_idx = self.picker_selected_group;
                    let mut picked_kc: Option<u16> = None;

                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            ui.horizontal_wrapped(|ui| {
                                ui.spacing_mut().item_spacing = egui::vec2(4.0, 4.0);
                                if let Some(group) = self.picker_groups.get(group_idx) {
                                    for kc in &group.codes {
                                        let name = kc.name();
                                        let is_current = kc.0 == raw_kc;
                                        let size = egui::vec2(44.0, 28.0);
                                        let (rect, response) =
                                            ui.allocate_exact_size(size, egui::Sense::click());
                                        let is_hovered = response.hovered();

                                        let bg = if is_current {
                                            egui::Color32::from_rgb(70, 130, 180)
                                        } else if is_hovered {
                                            egui::Color32::from_rgb(80, 80, 90)
                                        } else {
                                            key_bg_color(kc)
                                        };
                                        let border = if is_current {
                                            egui::Color32::from_rgb(100, 180, 255)
                                        } else {
                                            egui::Color32::from_rgb(60, 60, 65)
                                        };
                                        let text_col = if is_current {
                                            egui::Color32::WHITE
                                        } else if kc.0 == 0 || kc.0 == 1 {
                                            egui::Color32::from_rgb(100, 100, 110)
                                        } else {
                                            egui::Color32::from_rgb(220, 220, 230)
                                        };

                                        let rounding = egui::CornerRadius::same(4);
                                        ui.painter().rect_filled(rect, rounding, bg);
                                        ui.painter().rect_stroke(
                                            rect,
                                            rounding,
                                            egui::Stroke::new(1.0, border),
                                            egui::StrokeKind::Outside,
                                        );

                                        let font_size = if name.len() <= 2 {
                                            12.0
                                        } else if name.len() <= 5 {
                                            10.5
                                        } else {
                                            8.5
                                        };
                                        ui.painter().text(
                                            rect.center(),
                                            egui::Align2::CENTER_CENTER,
                                            &name,
                                            egui::FontId::proportional(font_size),
                                            text_col,
                                        );

                                        if response.clicked() {
                                            picked_kc = Some(kc.0);
                                        }
                                    }
                                }
                            });
                        });

                    if let Some(new_kc) = picked_kc {
                        ui.memory_mut(|mem| {
                            mem.data
                                .insert_temp(egui::Id::new("pending_keycode"), new_kc);
                            mem.data
                                .insert_temp(egui::Id::new("pending_key_idx"), key_idx);
                        });
                    }
                });

            // Close popover => deselect key
            if !open {
                if let Some(data) = &mut self.keymap_data {
                    data.selected_key = None;
                }
            }
        }

        // Handle deferred keycode application
        let pending: Option<(usize, u16)> = ui.memory(|mem| {
            let kc: Option<u16> = mem.data.get_temp(egui::Id::new("pending_keycode"));
            let idx: Option<usize> = mem.data.get_temp(egui::Id::new("pending_key_idx"));
            match (kc, idx) {
                (Some(kc), Some(idx)) => Some((idx, kc)),
                _ => None,
            }
        });

        if let Some((key_idx, new_kc)) = pending {
            ui.memory_mut(|mem| {
                mem.data.remove::<u16>(egui::Id::new("pending_keycode"));
                mem.data.remove::<usize>(egui::Id::new("pending_key_idx"));
            });
            self.apply_keycode(key_idx, new_kc);
        }
    }

    fn render_lighting_tab(&mut self, ui: &mut egui::Ui) {
        let Some(lighting) = &mut self.lighting_data else {
            ui.label("No lighting data available.");
            return;
        };

        let channel_name = match lighting.channel {
            LightingChannel::QmkRgblight => "QMK Rgblight",
            LightingChannel::QmkRgbMatrix => "QMK RGB Matrix",
            LightingChannel::QmkLed => "QMK LED",
        };

        ui.add_space(20.0);

        // Center the controls
        let max_width = 500.0_f32.min(ui.available_width() - 40.0);
        ui.vertical_centered(|ui| {
            ui.set_max_width(max_width);

            ui.heading("Lighting Configuration");
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new(channel_name)
                    .size(12.0)
                    .color(egui::Color32::from_rgb(140, 140, 155)),
            );
            ui.add_space(16.0);

            // Brightness slider
            ui.horizontal(|ui| {
                ui.label("Brightness");
                ui.add_space(8.0);
                let mut val = lighting.brightness as f32;
                if ui
                    .add(egui::Slider::new(&mut val, 0.0..=255.0).integer())
                    .changed()
                {
                    lighting.brightness = val as u8;
                    lighting.dirty = true;
                }
            });
            ui.add_space(8.0);

            // Effect slider
            ui.horizontal(|ui| {
                ui.label("Effect");
                ui.add_space(8.0);
                let mut val = lighting.effect as f32;
                // rgb_matrix has ~45 effects typically, but cap at 255
                if ui
                    .add(egui::Slider::new(&mut val, 0.0..=48.0).integer())
                    .changed()
                {
                    lighting.effect = val as u8;
                    lighting.dirty = true;
                }
            });
            ui.add_space(8.0);

            // Speed slider
            ui.horizontal(|ui| {
                ui.label("Speed");
                ui.add_space(8.0);
                let mut val = lighting.speed as f32;
                if ui
                    .add(egui::Slider::new(&mut val, 0.0..=255.0).integer())
                    .changed()
                {
                    lighting.speed = val as u8;
                    lighting.dirty = true;
                }
            });
            ui.add_space(8.0);

            // Hue slider
            ui.horizontal(|ui| {
                ui.label("Hue");
                ui.add_space(8.0);
                let mut val = lighting.hue as f32;
                if ui
                    .add(egui::Slider::new(&mut val, 0.0..=255.0).integer())
                    .changed()
                {
                    lighting.hue = val as u8;
                    lighting.dirty = true;
                }
            });
            ui.add_space(8.0);

            // Saturation slider
            ui.horizontal(|ui| {
                ui.label("Saturation");
                ui.add_space(8.0);
                let mut val = lighting.saturation as f32;
                if ui
                    .add(egui::Slider::new(&mut val, 0.0..=255.0).integer())
                    .changed()
                {
                    lighting.saturation = val as u8;
                    lighting.dirty = true;
                }
            });

            ui.add_space(4.0);

            // Color preview swatch
            let (r, g, b) = hsv_to_rgb(lighting.hue, lighting.saturation, lighting.brightness);
            let swatch_size = egui::vec2(max_width, 30.0);
            let (rect, _) = ui.allocate_exact_size(swatch_size, egui::Sense::hover());
            ui.painter().rect_filled(
                rect,
                egui::CornerRadius::same(4),
                egui::Color32::from_rgb(r, g, b),
            );
            ui.painter().rect_stroke(
                rect,
                egui::CornerRadius::same(4),
                egui::Stroke::new(1.0, egui::Color32::from_rgb(80, 80, 90)),
                egui::StrokeKind::Outside,
            );

            ui.add_space(20.0);

            // Buttons
            ui.horizontal(|ui| {
                // Apply (send to device without saving to EEPROM)
                let apply_enabled = lighting.dirty;
                if ui
                    .add_enabled(apply_enabled, egui::Button::new("Apply"))
                    .clicked()
                {
                    ui.memory_mut(|mem| {
                        mem.data.insert_temp(egui::Id::new("lighting_action"), 1u8);
                    });
                }

                // Save to EEPROM
                if ui.button("Save to EEPROM").clicked() {
                    ui.memory_mut(|mem| {
                        mem.data.insert_temp(egui::Id::new("lighting_action"), 2u8);
                    });
                }

                // Reload from device
                if ui.button("Reload").clicked() {
                    ui.memory_mut(|mem| {
                        mem.data.insert_temp(egui::Id::new("lighting_action"), 3u8);
                    });
                }
            });

            if lighting.dirty {
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new("Unsaved changes")
                        .size(11.0)
                        .color(egui::Color32::from_rgb(200, 170, 60)),
                );
            }
        });

        // Handle deferred lighting actions (to avoid borrow conflicts)
        let action: Option<u8> =
            ui.memory(|mem| mem.data.get_temp(egui::Id::new("lighting_action")));
        if let Some(action) = action {
            ui.memory_mut(|mem| {
                mem.data.remove::<u8>(egui::Id::new("lighting_action"));
            });
            match action {
                1 => self.apply_lighting(),
                2 => {
                    self.apply_lighting();
                    self.save_lighting();
                }
                3 => self.reload_lighting(),
                _ => {}
            }
        }
    }
}

/// Send current lighting values to the device.
impl ViarApp {
    fn apply_lighting(&mut self) {
        let Some(lighting) = &self.lighting_data else {
            return;
        };
        let Some(dev) = &self.connected_device else {
            return;
        };
        let proto = ViaProtocol::new(dev);
        let channel = lighting.channel;
        let mut errors = Vec::new();

        if let Err(e) = proto.set_rgb_brightness(channel, lighting.brightness) {
            errors.push(format!("brightness: {e}"));
        }
        if let Err(e) = proto.set_rgb_effect(channel, lighting.effect) {
            errors.push(format!("effect: {e}"));
        }
        if let Err(e) = proto.set_rgb_speed(channel, lighting.speed) {
            errors.push(format!("speed: {e}"));
        }
        if let Err(e) = proto.set_rgb_color(channel, lighting.hue, lighting.saturation) {
            errors.push(format!("color: {e}"));
        }

        if errors.is_empty() {
            info!("lighting values applied to device");
            if let Some(l) = &mut self.lighting_data {
                l.dirty = false;
            }
            self.set_status(StatusMessage::info("Lighting applied"));
        } else {
            let msg = format!("Lighting errors: {}", errors.join(", "));
            warn!("{msg}");
            self.set_status(StatusMessage::error(msg));
            // Check for disconnect
            for e in &errors {
                if is_disconnect_error(e) {
                    self.handle_disconnect();
                    return;
                }
            }
        }
    }

    fn save_lighting(&mut self) {
        let Some(dev) = &self.connected_device else {
            return;
        };
        let proto = ViaProtocol::new(dev);
        match proto.custom_save() {
            Ok(()) => {
                info!("lighting saved to EEPROM");
                self.set_status(StatusMessage::info("Lighting saved to EEPROM"));
            }
            Err(e) => {
                let err_str = format!("{e}");
                warn!(error = %e, "failed to save lighting");
                self.set_status(StatusMessage::error(format!("Save failed: {e}")));
                if is_disconnect_error(&err_str) {
                    self.handle_disconnect();
                }
            }
        }
    }

    fn reload_lighting(&mut self) {
        let Some(dev) = &self.connected_device else {
            return;
        };
        let Some(lighting) = &self.lighting_data else {
            return;
        };
        let channel = lighting.channel;
        let proto = ViaProtocol::new(dev);

        let brightness = proto.get_rgb_brightness(channel).unwrap_or(0);
        let effect = proto.get_rgb_effect(channel).unwrap_or(0);
        let speed = proto.get_rgb_speed(channel).unwrap_or(0);
        let (hue, sat) = proto.get_rgb_color(channel).unwrap_or((0, 0));

        if let Some(l) = &mut self.lighting_data {
            l.brightness = brightness;
            l.effect = effect;
            l.speed = speed;
            l.hue = hue;
            l.saturation = sat;
            l.dirty = false;
        }
        info!(brightness, effect, speed, hue, sat, "lighting reloaded");
        self.set_status(StatusMessage::info("Lighting reloaded from device"));
    }
}

/// Convert QMK-style HSV (hue 0-255, sat 0-255, val 0-255) to RGB.
fn hsv_to_rgb(h: u8, s: u8, v: u8) -> (u8, u8, u8) {
    if s == 0 {
        return (v, v, v);
    }
    let h = h as f32 / 255.0 * 360.0;
    let s = s as f32 / 255.0;
    let v = v as f32 / 255.0;
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;
    let (r1, g1, b1) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };
    (
        ((r1 + m) * 255.0) as u8,
        ((g1 + m) * 255.0) as u8,
        ((b1 + m) * 255.0) as u8,
    )
}

/// Check if an error string indicates a device disconnect.
fn is_disconnect_error(err: &str) -> bool {
    let lower = err.to_lowercase();
    lower.contains("no device")
        || lower.contains("disconnected")
        || lower.contains("i/o error")
        || lower.contains("device not found")
        || lower.contains("broken pipe")
}

/// Pick a background color based on keycode category.
fn key_bg_color(kc: &Keycode) -> egui::Color32 {
    use via_protocol::KeycodeCategory;
    match kc.category() {
        KeycodeCategory::None => egui::Color32::from_rgb(35, 35, 40),
        KeycodeCategory::Transparent => egui::Color32::from_rgb(35, 35, 40),
        KeycodeCategory::Basic => egui::Color32::from_rgb(50, 50, 58),
        KeycodeCategory::Mod => egui::Color32::from_rgb(60, 50, 70),
        KeycodeCategory::LayerTap => egui::Color32::from_rgb(50, 60, 70),
        KeycodeCategory::LayerMomentary => egui::Color32::from_rgb(45, 65, 55),
        KeycodeCategory::LayerToggle => egui::Color32::from_rgb(55, 55, 70),
        KeycodeCategory::ModTap => egui::Color32::from_rgb(65, 55, 50),
        KeycodeCategory::TapDance => egui::Color32::from_rgb(60, 55, 65),
        _ => egui::Color32::from_rgb(45, 45, 52),
    }
}
