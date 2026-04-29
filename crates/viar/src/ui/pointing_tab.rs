use eframe::egui;
use tracing::{info, warn};
use via_protocol::{pointing_settings, ViaProtocol};

use crate::types::{StatusMessage, ViarApp};
use crate::util::is_disconnect_error;

impl ViarApp {
    pub fn render_pointing_tab(&mut self, ui: &mut egui::Ui) {
        let Some(pointing) = &self.pointing_data else {
            ui.label("No pointing device settings available.");
            return;
        };

        let available = pointing.available_settings.clone();

        if available.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(ui.available_height() / 3.0);
                ui.heading("No pointing device settings detected");
                ui.label("This keyboard does not expose pointing device settings via QMK Settings.");
            });
            return;
        }

        ui.add_space(12.0);
        ui.horizontal(|ui| {
            ui.add_space(16.0);
            ui.label(
                egui::RichText::new("Pointing Device / Trackpad Settings")
                    .size(18.0)
                    .strong()
                    .color(egui::Color32::from_rgb(200, 200, 215)),
            );
        });
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.add_space(16.0);
            ui.label(
                egui::RichText::new("Configure DPI, scroll, axis inversion, and auto-mouse for trackpads and trackballs.")
                    .size(11.0)
                    .color(egui::Color32::from_rgb(110, 110, 125)),
            );
        });
        ui.add_space(12.0);
        ui.separator();
        ui.add_space(8.0);

        let frame = egui::Frame::default()
            .inner_margin(egui::Margin::symmetric(24, 8));

        frame.show(ui, |ui| {
            egui::Grid::new("pointing_settings_grid")
                .num_columns(2)
                .spacing([16.0, 8.0])
                .min_col_width(180.0)
                .show(ui, |ui| {
                    // DPI
                    if available.contains(&pointing_settings::DPI) {
                        let current = self.pointing_data.as_ref()
                            .and_then(|p| p.get_u16(pointing_settings::DPI))
                            .unwrap_or(400);
                        ui.label(
                            egui::RichText::new("DPI / CPI")
                                .size(13.0)
                                .color(egui::Color32::from_rgb(180, 180, 195)),
                        );
                        let mut val = current as f32;
                        if ui
                            .add(egui::Slider::new(&mut val, 100.0..=16000.0).step_by(100.0).suffix(" DPI"))
                            .changed()
                        {
                            if let Some(p) = self.pointing_data.as_mut() {
                                p.set_u16(pointing_settings::DPI, val as u16);
                            }
                            self.save_pointing_setting(pointing_settings::DPI, &(val as u16).to_le_bytes());
                        }
                        ui.end_row();
                    }

                    // Sniping DPI
                    if available.contains(&pointing_settings::SNIPING_DPI) {
                        let current = self.pointing_data.as_ref()
                            .and_then(|p| p.get_u16(pointing_settings::SNIPING_DPI))
                            .unwrap_or(200);
                        ui.label(
                            egui::RichText::new("Sniping DPI")
                                .size(13.0)
                                .color(egui::Color32::from_rgb(180, 180, 195)),
                        );
                        let mut val = current as f32;
                        if ui
                            .add(egui::Slider::new(&mut val, 50.0..=4000.0).step_by(50.0).suffix(" DPI"))
                            .changed()
                        {
                            if let Some(p) = self.pointing_data.as_mut() {
                                p.set_u16(pointing_settings::SNIPING_DPI, val as u16);
                            }
                            self.save_pointing_setting(pointing_settings::SNIPING_DPI, &(val as u16).to_le_bytes());
                        }
                        ui.end_row();
                    }

                    // Scroll divisor
                    if available.contains(&pointing_settings::SCROLL_DIVISOR) {
                        let current = self.pointing_data.as_ref()
                            .and_then(|p| p.get_u8(pointing_settings::SCROLL_DIVISOR))
                            .unwrap_or(8);
                        ui.label(
                            egui::RichText::new("Scroll Divisor")
                                .size(13.0)
                                .color(egui::Color32::from_rgb(180, 180, 195)),
                        );
                        let mut val = current as f32;
                        if ui
                            .add(egui::Slider::new(&mut val, 1.0..=64.0).integer())
                            .changed()
                        {
                            if let Some(p) = self.pointing_data.as_mut() {
                                p.set_u8(pointing_settings::SCROLL_DIVISOR, val as u8);
                            }
                            self.save_pointing_setting(pointing_settings::SCROLL_DIVISOR, &[val as u8]);
                        }
                        ui.end_row();
                    }

                    // Drag scroll divisor
                    if available.contains(&pointing_settings::DRAG_SCROLL_DIVISOR) {
                        let current = self.pointing_data.as_ref()
                            .and_then(|p| p.get_u8(pointing_settings::DRAG_SCROLL_DIVISOR))
                            .unwrap_or(8);
                        ui.label(
                            egui::RichText::new("Drag Scroll Divisor")
                                .size(13.0)
                                .color(egui::Color32::from_rgb(180, 180, 195)),
                        );
                        let mut val = current as f32;
                        if ui
                            .add(egui::Slider::new(&mut val, 1.0..=64.0).integer())
                            .changed()
                        {
                            if let Some(p) = self.pointing_data.as_mut() {
                                p.set_u8(pointing_settings::DRAG_SCROLL_DIVISOR, val as u8);
                            }
                            self.save_pointing_setting(pointing_settings::DRAG_SCROLL_DIVISOR, &[val as u8]);
                        }
                        ui.end_row();
                    }

                    // Toggle settings (booleans)
                    let toggles = [
                        (pointing_settings::INVERT_X, "Invert X Axis"),
                        (pointing_settings::INVERT_Y, "Invert Y Axis"),
                        (pointing_settings::INVERT_SCROLL, "Invert Scroll"),
                        (pointing_settings::DRAG_SCROLL, "Drag Scroll Mode"),
                        (pointing_settings::AUTO_MOUSE_ENABLE, "Auto Mouse Enable"),
                    ];

                    for (id, label) in toggles {
                        if available.contains(&id) {
                            let current = self.pointing_data.as_ref()
                                .and_then(|p| p.get_u8(id))
                                .unwrap_or(0);
                            ui.label(
                                egui::RichText::new(label)
                                    .size(13.0)
                                    .color(egui::Color32::from_rgb(180, 180, 195)),
                            );
                            let mut checked = current != 0;
                            if ui.checkbox(&mut checked, "").changed() {
                                let val = checked as u8;
                                if let Some(p) = self.pointing_data.as_mut() {
                                    p.set_u8(id, val);
                                }
                                self.save_pointing_setting(id, &[val]);
                            }
                            ui.end_row();
                        }
                    }

                    // Auto mouse layer
                    if available.contains(&pointing_settings::AUTO_MOUSE_LAYER) {
                        let current = self.pointing_data.as_ref()
                            .and_then(|p| p.get_u8(pointing_settings::AUTO_MOUSE_LAYER))
                            .unwrap_or(0);
                        ui.label(
                            egui::RichText::new("Auto Mouse Layer")
                                .size(13.0)
                                .color(egui::Color32::from_rgb(180, 180, 195)),
                        );
                        let mut val = current as f32;
                        if ui
                            .add(egui::Slider::new(&mut val, 0.0..=15.0).integer())
                            .changed()
                        {
                            if let Some(p) = self.pointing_data.as_mut() {
                                p.set_u8(pointing_settings::AUTO_MOUSE_LAYER, val as u8);
                            }
                            self.save_pointing_setting(pointing_settings::AUTO_MOUSE_LAYER, &[val as u8]);
                        }
                        ui.end_row();
                    }

                    // Auto mouse timeout
                    if available.contains(&pointing_settings::AUTO_MOUSE_TIMEOUT) {
                        let current = self.pointing_data.as_ref()
                            .and_then(|p| p.get_u16(pointing_settings::AUTO_MOUSE_TIMEOUT))
                            .unwrap_or(500);
                        ui.label(
                            egui::RichText::new("Auto Mouse Timeout")
                                .size(13.0)
                                .color(egui::Color32::from_rgb(180, 180, 195)),
                        );
                        let mut val = current as f32;
                        if ui
                            .add(egui::Slider::new(&mut val, 50.0..=5000.0).step_by(50.0).suffix(" ms"))
                            .changed()
                        {
                            if let Some(p) = self.pointing_data.as_mut() {
                                p.set_u16(pointing_settings::AUTO_MOUSE_TIMEOUT, val as u16);
                            }
                            self.save_pointing_setting(pointing_settings::AUTO_MOUSE_TIMEOUT, &(val as u16).to_le_bytes());
                        }
                        ui.end_row();
                    }
                });

            // Reset button
            ui.add_space(16.0);
            ui.horizontal(|ui| {
                if ui
                    .add(
                        egui::Button::new(
                            egui::RichText::new("Reset to Defaults")
                                .size(12.0)
                                .color(egui::Color32::from_rgb(200, 120, 120)),
                        )
                        .fill(egui::Color32::from_rgb(50, 35, 35))
                        .corner_radius(egui::CornerRadius::same(4)),
                    )
                    .clicked()
                {
                    self.reset_pointing_settings();
                }
            });
        });
    }

    fn save_pointing_setting(&mut self, setting_id: u16, value: &[u8]) {
        let Some(dev) = &self.connected_device else {
            return;
        };
        let proto = ViaProtocol::new(dev);
        match proto.qmk_settings_set(setting_id, value) {
            Ok(()) => {
                info!(setting_id, "pointing setting saved");
                self.set_status(StatusMessage::info(format!(
                    "Setting 0x{:04X} saved",
                    setting_id
                )));
            }
            Err(e) => {
                let msg = format!("{e}");
                warn!(error = %e, setting_id, "failed to save pointing setting");
                if is_disconnect_error(&msg) {
                    self.handle_disconnect();
                } else {
                    self.set_status(StatusMessage::error(format!(
                        "Failed to save setting: {e}"
                    )));
                }
            }
        }
    }

    fn reset_pointing_settings(&mut self) {
        let Some(dev) = &self.connected_device else {
            return;
        };
        let proto = ViaProtocol::new(dev);
        let result = match proto.qmk_settings_reset() {
            Ok(()) => {
                // Re-read values
                let ids = self
                    .pointing_data
                    .as_ref()
                    .map(|p| p.available_settings.clone())
                    .unwrap_or_default();
                let mut values = std::collections::HashMap::new();
                for &id in &ids {
                    if let Ok(v) = proto.qmk_settings_get(id) {
                        values.insert(id, v);
                    }
                }
                Ok((ids, values))
            }
            Err(e) => Err(e),
        };

        match result {
            Ok((ids, values)) => {
                info!("pointing settings reset to defaults");
                self.pointing_data = Some(crate::types::PointingData::new(ids, values));
                self.set_status(StatusMessage::info("Pointing settings reset to defaults"));
            }
            Err(e) => {
                let msg = format!("{e}");
                warn!(error = %e, "failed to reset pointing settings");
                if is_disconnect_error(&msg) {
                    self.handle_disconnect();
                } else {
                    self.set_status(StatusMessage::error(format!(
                        "Failed to reset settings: {e}"
                    )));
                }
            }
        }
    }
}
