use eframe::egui;
use tracing::{debug, info, warn};
use via_protocol::{
    device::{check_hid_permissions, discover_keyboards},
    keycode_groups,
    layout::{find_layout, generic_layout, parse_vial_definition},
    HidAccessStatus, KeyboardDevice, LightingProtocol, ViaProtocol,
};

use crate::types::*;

impl ViarApp {
    pub fn new() -> Self {
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

    pub fn set_status(&mut self, msg: StatusMessage) {
        self.status = Some(msg);
    }

    pub fn detect(&mut self) {
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

    pub fn refresh(&mut self) {
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

    pub fn connect_to_keyboard(&mut self, idx: usize) {
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

    pub fn load_keymap(&mut self) {
        let Some(dev) = &self.connected_device else {
            return;
        };
        let info = &dev.info;
        let proto = ViaProtocol::new(dev);

        // Try Vial definition first, then hardcoded, then generic
        let layout = match proto.vial_get_definition() {
            Ok(json) => {
                info!("got Vial definition from firmware, parsing KLE layout");
                match parse_vial_definition(&json) {
                    Ok(mut layout) => {
                        // Set the name from device info if the definition didn't have one
                        if layout.name == "Vial Keyboard" {
                            layout.name = format!("{}", info);
                        }
                        layout
                    }
                    Err(e) => {
                        warn!(error = %e, "failed to parse Vial definition, falling back");
                        find_layout(info.vendor_id, info.product_id).unwrap_or_else(|| {
                            debug!("no built-in layout, using generic");
                            generic_layout(4, 12)
                        })
                    }
                }
            }
            Err(e) => {
                debug!(error = %e, "no Vial definition available, trying built-in layouts");
                find_layout(info.vendor_id, info.product_id).unwrap_or_else(|| {
                    debug!(
                        "no built-in layout for {:04x}:{:04x}, using generic",
                        info.vendor_id, info.product_id
                    );
                    generic_layout(4, 12)
                })
            }
        };

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
            if let Some(lighting_proto) = proto.detect_lighting_protocol() {
                info!(?lighting_proto, "detected lighting protocol");
                match proto.read_lighting_values(&lighting_proto) {
                    Ok(vals) => {
                        info!(
                            brightness = vals.brightness,
                            effect_id = vals.effect_id,
                            speed = vals.speed,
                            hue = vals.hue,
                            sat = vals.saturation,
                            "lighting values loaded"
                        );
                        let supported_effects =
                            if matches!(lighting_proto, LightingProtocol::VialRgb) {
                                proto.vialrgb_get_supported_effects().unwrap_or_default()
                            } else {
                                Vec::new()
                            };
                        if !supported_effects.is_empty() {
                            info!(count = supported_effects.len(), effects = ?supported_effects, "supported VialRGB effects");
                        }
                        self.lighting_data = Some(LightingData {
                            protocol: lighting_proto,
                            brightness: vals.brightness,
                            effect_id: vals.effect_id,
                            speed: vals.speed,
                            hue: vals.hue,
                            saturation: vals.saturation,
                            supported_effects,
                            dirty: false,
                        });
                    }
                    Err(e) => {
                        warn!(error = %e, "failed to read lighting values");
                    }
                }
            }
        }

        self.screen = AppScreen::Connected;
    }

    pub fn disconnect(&mut self) {
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

    pub fn handle_disconnect(&mut self) {
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

        self.render_menu_bar(ctx);
        self.render_confirm_dialog(ctx);

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
