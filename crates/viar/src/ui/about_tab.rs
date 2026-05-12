use eframe::egui;

use crate::types::ViarApp;

impl ViarApp {
    pub fn render_about_tab(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.add_space(16.0);

            let max_width = 500.0_f32.min(ui.available_width() - 32.0);
            ui.vertical_centered(|ui| {
                ui.set_max_width(max_width);

                ui.heading(
                    egui::RichText::new("About Keyboard")
                        .size(24.0)
                        .strong()
                        .color(egui::Color32::from_rgb(200, 200, 215)),
                );
                ui.add_space(16.0);

                // Device info
                if let Some(dev) = &self.connected_device {
                    let info = &dev.info;

                    Self::info_section(ui, "Device");
                    Self::info_row(ui, "Product", &info.product);
                    Self::info_row(ui, "Manufacturer", &info.manufacturer);
                    Self::info_row(
                        ui,
                        "USB ID",
                        &format!("{:04X}:{:04X}", info.vendor_id, info.product_id),
                    );
                    if !info.serial_number.is_empty() {
                        Self::info_row(ui, "Serial", &info.serial_number);
                    }
                    Self::info_row(ui, "HID Path", &info.path);

                    ui.add_space(12.0);
                }

                // Protocol info
                Self::info_section(ui, "Protocol");
                if let Some(ver) = self.protocol_version {
                    Self::info_row(ui, "VIA Protocol", &format!("v{ver}"));
                }
                if let Some(vial_ver) = self.vial_protocol_version {
                    Self::info_row(ui, "Vial Protocol", &format!("v{vial_ver}"));
                }
                if let Some(fw_ver) = self.firmware_version {
                    // Firmware version is typically packed as: major.minor.patch or a raw u32
                    Self::info_row(ui, "Firmware Version", &format!("{fw_ver:#010X}"));
                }
                if let Some(uid) = &self.vial_uid {
                    let uid_str = uid
                        .iter()
                        .map(|b| format!("{:02X}", b))
                        .collect::<Vec<_>>()
                        .join(":");
                    Self::info_row(ui, "Keyboard UID", &uid_str);
                }
                if let Some(uptime_ms) = self.connect_uptime_ms {
                    let secs = uptime_ms / 1000;
                    let mins = secs / 60;
                    let hours = mins / 60;
                    let uptime_str = if hours > 0 {
                        format!("{}h {}m {}s", hours, mins % 60, secs % 60)
                    } else if mins > 0 {
                        format!("{}m {}s", mins, secs % 60)
                    } else {
                        format!("{}s", secs)
                    };
                    Self::info_row(ui, "Uptime (at connect)", &uptime_str);
                }

                ui.add_space(12.0);

                // Layout info
                if let Some(data) = &self.keymap_data {
                    Self::info_section(ui, "Layout");
                    Self::info_row(ui, "Layout Name", &data.layout.name);
                    Self::info_row(ui, "Layers", &format!("{}", data.layer_count));
                    Self::info_row(
                        ui,
                        "Matrix",
                        &format!("{} rows x {} cols", data.layout.rows, data.layout.cols),
                    );
                    Self::info_row(ui, "Keys", &format!("{}", data.layout.keys.len()));

                    ui.add_space(12.0);
                }

                // Lighting info
                if let Some(lighting) = &self.lighting_data {
                    Self::info_section(ui, "Lighting");
                    let protocol_name = match &lighting.protocol {
                        via_protocol::LightingProtocol::Via { channel } => {
                            use via_protocol::LightingChannel;
                            match channel {
                                LightingChannel::QmkBacklight => "QMK Backlight (VIA)",
                                LightingChannel::QmkRgblight => "QMK Rgblight (VIA)",
                                LightingChannel::QmkRgbMatrix => "QMK RGB Matrix (VIA)",
                                LightingChannel::QmkAudio => "QMK Audio (VIA)",
                                LightingChannel::QmkLedMatrix => "QMK LED Matrix (VIA)",
                            }
                        }
                        via_protocol::LightingProtocol::VialLegacy => "RGB Matrix (Vial Legacy)",
                        via_protocol::LightingProtocol::VialRgb => "RGB Matrix (VialRGB)",
                    };
                    Self::info_row(ui, "Protocol", protocol_name);
                    Self::info_row(
                        ui,
                        "Max Brightness",
                        &format!("{}/255", lighting.max_brightness),
                    );
                    Self::info_row(
                        ui,
                        "Effects",
                        &format!("{} supported", lighting.supported_effects.len()),
                    );

                    ui.add_space(12.0);
                }

                // Dynamic entries info
                if let Some(dynamic) = &self.dynamic_data {
                    Self::info_section(ui, "Dynamic Entries");
                    Self::info_row(
                        ui,
                        "Tap Dances",
                        &format!("{} slots", dynamic.tap_dances.len()),
                    );
                    Self::info_row(
                        ui,
                        "Combos",
                        &format!("{} slots", dynamic.combos.len()),
                    );
                    Self::info_row(
                        ui,
                        "Key Overrides",
                        &format!("{} slots", dynamic.key_overrides.len()),
                    );

                    ui.add_space(12.0);
                }

                // Pointing info
                if self.pointing_data.is_some() {
                    Self::info_section(ui, "Pointing Device");
                    Self::info_row(ui, "Status", "Supported");

                    ui.add_space(12.0);
                }

                // Detected features
                if !self.detected_features.is_empty() {
                    Self::info_section(ui, "Detected Features");
                    for feature in &self.detected_features {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("  \u{2713}")
                                    .size(15.0)
                                    .color(egui::Color32::from_rgb(135, 169, 135)),
                            );
                            ui.label(
                                egui::RichText::new(feature)
                                    .size(15.0)
                                    .color(egui::Color32::from_rgb(200, 200, 215)),
                            );
                        });
                        ui.add_space(1.0);
                    }

                    ui.add_space(12.0);
                }

                // App info
                Self::info_section(ui, "Application");
                Self::info_row(ui, "Viar Version", env!("CARGO_PKG_VERSION"));

                ui.add_space(24.0);
            });
        });
    }

    fn info_section(ui: &mut egui::Ui, title: &str) {
        ui.label(
            egui::RichText::new(title)
                .size(18.0)
                .strong()
                .color(egui::Color32::from_rgb(170, 170, 190)),
        );
        ui.add_space(4.0);
        ui.separator();
        ui.add_space(4.0);
    }

    fn info_row(ui: &mut egui::Ui, label: &str, value: &str) {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(format!("{label}:"))
                    .size(15.0)
                    .color(egui::Color32::from_rgb(140, 140, 155)),
            );
            ui.label(
                egui::RichText::new(value)
                    .size(15.0)
                    .monospace()
                    .color(egui::Color32::from_rgb(200, 200, 215)),
            );
        });
        ui.add_space(2.0);
    }
}
