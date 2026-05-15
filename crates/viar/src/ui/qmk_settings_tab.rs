use eframe::egui;
use tracing::{
    info,
    warn,
};
use via_protocol::ViaProtocol;

use crate::{
    types::{
        StatusMessage,
        ViarApp,
    },
    util::is_disconnect_error,
};

impl ViarApp {
    pub fn render_qmk_settings_tab(&mut self, ui: &mut egui::Ui) {
        let Some(qmk_data) = &self.qmk_settings_data else {
            ui.vertical_centered(|ui| {
                ui.add_space(ui.available_height() / 3.0);
                ui.heading("No QMK Settings Detected");
                ui.label("This keyboard does not support QMK Settings via Vial protocol.");
            });
            return;
        };

        let available = qmk_data.available_settings.clone();
        let all_defs = qmk_rs::all_setting_defs();

        ui.add_space(12.0);
        ui.horizontal(|ui| {
            ui.add_space(16.0);
            ui.label(
                egui::RichText::new("QMK Settings")
                    .size(22.0)
                    .strong()
                    .color(egui::Color32::from_rgb(200, 200, 215)),
            );
        });
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.add_space(16.0);
            ui.label(
                egui::RichText::new(
                    "Configure firmware behavior: key swaps, tapping, auto shift, combos, mouse keys, and more.\n\
                     Unavailable settings on this keyboard are grayed out.",
                )
                .size(15.0)
                .color(egui::Color32::from_rgb(110, 110, 125)),
            );
        });
        ui.add_space(12.0);
        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
            let frame = egui::Frame::default().inner_margin(egui::Margin::symmetric(24, 8));

            frame.show(ui, |ui| {
                for &category in qmk_rs::SettingCategory::all() {
                    // Skip Pointing — that has its own tab
                    if category == qmk_rs::SettingCategory::Pointing {
                        continue;
                    }

                    self.render_settings_category(ui, category, &available, &all_defs);
                }

                // Reset button
                ui.add_space(16.0);
                ui.horizontal(|ui| {
                    if ui
                        .add(
                            egui::Button::new(
                                egui::RichText::new("Reset All QMK Settings to Defaults")
                                    .size(16.0)
                                    .color(egui::Color32::from_rgb(200, 120, 120)),
                            )
                            .fill(egui::Color32::from_rgb(50, 35, 35))
                            .corner_radius(egui::CornerRadius::same(4)),
                        )
                        .clicked()
                    {
                        self.reset_qmk_settings();
                    }
                });
                ui.add_space(16.0);
            });
        });
    }

    fn render_settings_category(
        &mut self,
        ui: &mut egui::Ui,
        category: qmk_rs::SettingCategory,
        available: &[u16],
        all_defs: &std::collections::HashMap<u16, qmk_rs::SettingDef>,
    ) {
        // Collect defs for this category
        let mut category_defs: Vec<&qmk_rs::SettingDef> = all_defs
            .values()
            .filter(|d| d.category == category)
            .collect();
        category_defs.sort_by_key(|d| d.id);

        if category_defs.is_empty() {
            return;
        }

        ui.add_space(12.0);
        ui.label(
            egui::RichText::new(category.label())
                .size(19.0)
                .strong()
                .color(egui::Color32::from_rgb(180, 180, 200)),
        );
        ui.add_space(4.0);

        // Special handling for bitfield settings
        match category {
            qmk_rs::SettingCategory::Magic => {
                self.render_magic_settings(ui, available);
            }
            qmk_rs::SettingCategory::AutoShift => {
                self.render_auto_shift_settings(ui, available, all_defs);
            }
            qmk_rs::SettingCategory::GraveEscape => {
                self.render_grave_esc_settings(ui, available);
            }
            qmk_rs::SettingCategory::Tapping => {
                self.render_tapping_settings(ui, &category_defs, available);
            }
            _ => {
                self.render_generic_settings(ui, &category_defs, available);
            }
        }

        ui.separator();
    }

    fn render_magic_settings(&mut self, ui: &mut egui::Ui, available: &[u16]) {
        let is_available = available.contains(&qmk_rs::QS_MAGIC);

        egui::Grid::new("magic_settings_grid")
            .num_columns(2)
            .spacing([16.0, 6.0])
            .min_col_width(250.0)
            .show(ui, |ui| {
                for flag in qmk_rs::magic_flags() {
                    let enabled = is_available
                        && self
                            .qmk_settings_data
                            .as_ref()
                            .and_then(|d| d.get_bit(qmk_rs::QS_MAGIC, flag.bit))
                            .unwrap_or(false);

                    ui.add_enabled(
                        is_available,
                        egui::Label::new(egui::RichText::new(flag.name).size(16.0).color(
                            if is_available {
                                egui::Color32::from_rgb(180, 180, 195)
                            } else {
                                egui::Color32::from_rgb(90, 90, 100)
                            },
                        )),
                    )
                    .on_hover_text(flag.description);

                    let mut checked = enabled;
                    let response =
                        ui.add_enabled(is_available, egui::Checkbox::new(&mut checked, ""));
                    if response.changed() {
                        if let Some(data) = self.qmk_settings_data.as_mut() {
                            data.set_bit(qmk_rs::QS_MAGIC, flag.bit, checked);
                            if let Some(raw) = data.get_raw(qmk_rs::QS_MAGIC) {
                                let bytes = raw.to_vec();
                                self.save_qmk_setting(qmk_rs::QS_MAGIC, &bytes);
                            }
                        }
                    }
                    ui.end_row();
                }
            });
    }

    fn render_auto_shift_settings(
        &mut self,
        ui: &mut egui::Ui,
        available: &[u16],
        all_defs: &std::collections::HashMap<u16, qmk_rs::SettingDef>,
    ) {
        let flags_available = available.contains(&qmk_rs::QS_AUTO_SHIFT);

        egui::Grid::new("autoshift_flags_grid")
            .num_columns(2)
            .spacing([16.0, 6.0])
            .min_col_width(250.0)
            .show(ui, |ui| {
                for flag in qmk_rs::auto_shift_flags() {
                    let enabled = flags_available
                        && self
                            .qmk_settings_data
                            .as_ref()
                            .and_then(|d| d.get_bit(qmk_rs::QS_AUTO_SHIFT, flag.bit))
                            .unwrap_or(false);

                    ui.add_enabled(
                        flags_available,
                        egui::Label::new(egui::RichText::new(flag.name).size(16.0).color(
                            if flags_available {
                                egui::Color32::from_rgb(180, 180, 195)
                            } else {
                                egui::Color32::from_rgb(90, 90, 100)
                            },
                        )),
                    )
                    .on_hover_text(flag.description);

                    let mut checked = enabled;
                    let response =
                        ui.add_enabled(flags_available, egui::Checkbox::new(&mut checked, ""));
                    if response.changed() {
                        if let Some(data) = self.qmk_settings_data.as_mut() {
                            data.set_bit(qmk_rs::QS_AUTO_SHIFT, flag.bit, checked);
                            if let Some(raw) = data.get_raw(qmk_rs::QS_AUTO_SHIFT) {
                                let bytes = raw.to_vec();
                                self.save_qmk_setting(qmk_rs::QS_AUTO_SHIFT, &bytes);
                            }
                        }
                    }
                    ui.end_row();
                }
            });

        // Also render the Auto Shift Timeout slider
        if let Some(def) = all_defs.get(&qmk_rs::QS_AUTO_SHIFT_TIMEOUT) {
            let timeout_defs = vec![def];
            self.render_generic_settings(ui, &timeout_defs, available);
        }
    }

    fn render_grave_esc_settings(&mut self, ui: &mut egui::Ui, available: &[u16]) {
        let is_available = available.contains(&qmk_rs::QS_GRAVE_ESC_OVERRIDE);

        egui::Grid::new("grave_esc_grid")
            .num_columns(2)
            .spacing([16.0, 6.0])
            .min_col_width(250.0)
            .show(ui, |ui| {
                for flag in qmk_rs::grave_esc_flags() {
                    let enabled = is_available
                        && self
                            .qmk_settings_data
                            .as_ref()
                            .and_then(|d| d.get_bit(qmk_rs::QS_GRAVE_ESC_OVERRIDE, flag.bit))
                            .unwrap_or(false);

                    ui.add_enabled(
                        is_available,
                        egui::Label::new(egui::RichText::new(flag.name).size(16.0).color(
                            if is_available {
                                egui::Color32::from_rgb(180, 180, 195)
                            } else {
                                egui::Color32::from_rgb(90, 90, 100)
                            },
                        )),
                    )
                    .on_hover_text(flag.description);

                    let mut checked = enabled;
                    let response =
                        ui.add_enabled(is_available, egui::Checkbox::new(&mut checked, ""));
                    if response.changed() {
                        if let Some(data) = self.qmk_settings_data.as_mut() {
                            data.set_bit(qmk_rs::QS_GRAVE_ESC_OVERRIDE, flag.bit, checked);
                            if let Some(raw) = data.get_raw(qmk_rs::QS_GRAVE_ESC_OVERRIDE) {
                                let bytes = raw.to_vec();
                                self.save_qmk_setting(qmk_rs::QS_GRAVE_ESC_OVERRIDE, &bytes);
                            }
                        }
                    }
                    ui.end_row();
                }
            });
    }

    fn render_tapping_settings(
        &mut self,
        ui: &mut egui::Ui,
        category_defs: &[&qmk_rs::SettingDef],
        available: &[u16],
    ) {
        // Filter out the legacy tapping def (ID 8) from generic rendering — we handle it specially
        let non_legacy_defs: Vec<&qmk_rs::SettingDef> = category_defs
            .iter()
            .filter(|d| d.id != qmk_rs::QS_TAPPING_V2_LEGACY)
            .copied()
            .collect();
        self.render_generic_settings(ui, &non_legacy_defs, available);

        // If the board has legacy tapping (ID 8), render its bitfield flags
        if available.contains(&qmk_rs::QS_TAPPING_V2_LEGACY) {
            ui.add_space(8.0);
            ui.label(
                egui::RichText::new("Legacy Tapping Flags")
                    .size(16.0)
                    .color(egui::Color32::from_rgb(160, 160, 175)),
            );

            egui::Grid::new("legacy_tapping_grid")
                .num_columns(2)
                .spacing([16.0, 6.0])
                .min_col_width(250.0)
                .show(ui, |ui| {
                    for flag in qmk_rs::legacy_tapping_flags() {
                        let enabled = self
                            .qmk_settings_data
                            .as_ref()
                            .and_then(|d| d.get_bit(qmk_rs::QS_TAPPING_V2_LEGACY, flag.bit))
                            .unwrap_or(false);

                        ui.label(
                            egui::RichText::new(flag.name)
                                .size(16.0)
                                .color(egui::Color32::from_rgb(180, 180, 195)),
                        )
                        .on_hover_text(flag.description);

                        let mut checked = enabled;
                        if ui.checkbox(&mut checked, "").changed() {
                            if let Some(data) = self.qmk_settings_data.as_mut() {
                                data.set_bit(qmk_rs::QS_TAPPING_V2_LEGACY, flag.bit, checked);
                                if let Some(raw) = data.get_raw(qmk_rs::QS_TAPPING_V2_LEGACY) {
                                    let bytes = raw.to_vec();
                                    self.save_qmk_setting(qmk_rs::QS_TAPPING_V2_LEGACY, &bytes);
                                }
                            }
                        }
                        ui.end_row();
                    }
                });
        }
    }

    fn render_generic_settings(
        &mut self,
        ui: &mut egui::Ui,
        defs: &[&qmk_rs::SettingDef],
        available: &[u16],
    ) {
        egui::Grid::new(format!(
            "generic_settings_{}",
            defs.first().map_or(0, |d| d.id)
        ))
        .num_columns(2)
        .spacing([16.0, 8.0])
        .min_col_width(250.0)
        .show(ui, |ui| {
            for def in defs {
                let is_available = available.contains(&def.id);
                let label_color = if is_available {
                    egui::Color32::from_rgb(180, 180, 195)
                } else {
                    egui::Color32::from_rgb(90, 90, 100)
                };

                ui.add_enabled(
                    is_available,
                    egui::Label::new(egui::RichText::new(def.name).size(16.0).color(label_color)),
                )
                .on_hover_text(def.description);

                match def.setting_type {
                    qmk_rs::SettingType::Bool => {
                        let current = is_available
                            && self
                                .qmk_settings_data
                                .as_ref()
                                .and_then(|d| d.get_u8(def.id))
                                .unwrap_or(0)
                                != 0;
                        let mut checked = current;
                        let response =
                            ui.add_enabled(is_available, egui::Checkbox::new(&mut checked, ""));
                        if response.changed() {
                            let val = checked as u8;
                            if let Some(data) = self.qmk_settings_data.as_mut() {
                                data.set_u8(def.id, val);
                            }
                            self.save_qmk_setting(def.id, &[val]);
                        }
                    }
                    qmk_rs::SettingType::U8 => {
                        if let Some(range) = &def.range {
                            let current = self
                                .qmk_settings_data
                                .as_ref()
                                .and_then(|d| d.get_u8(def.id))
                                .unwrap_or(0);
                            let mut val = current as f32;
                            let slider = egui::Slider::new(&mut val, range.min..=range.max)
                                .step_by(range.step as f64);
                            let slider = if range.suffix.is_empty() {
                                slider.integer()
                            } else {
                                slider.suffix(range.suffix)
                            };
                            let response = ui.add_enabled(is_available, slider);
                            if response.changed() {
                                if let Some(data) = self.qmk_settings_data.as_mut() {
                                    data.set_u8(def.id, val as u8);
                                }
                                self.save_qmk_setting(def.id, &[val as u8]);
                            }
                        } else {
                            ui.add_enabled(is_available, egui::Label::new("--"));
                        }
                    }
                    qmk_rs::SettingType::U16 => {
                        if let Some(range) = &def.range {
                            let current = self
                                .qmk_settings_data
                                .as_ref()
                                .and_then(|d| d.get_u16(def.id))
                                .unwrap_or(0);
                            let mut val = current as f32;
                            let slider = egui::Slider::new(&mut val, range.min..=range.max)
                                .step_by(range.step as f64);
                            let slider = if range.suffix.is_empty() {
                                slider.integer()
                            } else {
                                slider.suffix(range.suffix)
                            };
                            let response = ui.add_enabled(is_available, slider);
                            if response.changed() {
                                if let Some(data) = self.qmk_settings_data.as_mut() {
                                    data.set_u16(def.id, val as u16);
                                }
                                self.save_qmk_setting(def.id, &(val as u16).to_le_bytes());
                            }
                        } else {
                            ui.add_enabled(is_available, egui::Label::new("--"));
                        }
                    }
                    qmk_rs::SettingType::U32 | qmk_rs::SettingType::Bitfield { .. } => {
                        // U32 and bitfield types are handled by specialized renderers
                        ui.add_enabled(is_available, egui::Label::new("--"));
                    }
                }
                ui.end_row();
            }
        });
    }

    fn save_qmk_setting(&mut self, setting_id: u16, value: &[u8]) {
        let Some(dev) = &self.connected_device else {
            return;
        };
        let proto = ViaProtocol::new(dev);
        match proto.qmk_settings_set(setting_id, value) {
            Ok(()) => {
                info!(setting_id, "QMK setting saved");
                self.set_status(StatusMessage::info(format!(
                    "Setting 0x{setting_id:04X} saved",
                )));
            }
            Err(e) => {
                let msg = format!("{e}");
                warn!(error = %e, setting_id, "failed to save QMK setting");
                if is_disconnect_error(&msg) {
                    self.handle_disconnect();
                } else {
                    self.set_status(StatusMessage::error(format!("Failed to save setting: {e}")));
                }
            }
        }
    }

    fn reset_qmk_settings(&mut self) {
        let Some(dev) = &self.connected_device else {
            return;
        };
        let proto = ViaProtocol::new(dev);

        let result = match proto.qmk_settings_reset() {
            Ok(()) => {
                // Re-read all values
                let ids = self
                    .qmk_settings_data
                    .as_ref()
                    .map(|d| d.available_settings.clone())
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
                info!("QMK settings reset to defaults");

                // Update pointing data too since reset affects everything
                let pointing_ids: Vec<u16> = ids
                    .iter()
                    .copied()
                    .filter(|id| (0x0100..=0x01FF).contains(id))
                    .collect();
                if !pointing_ids.is_empty() {
                    let pointing_values = pointing_ids
                        .iter()
                        .filter_map(|id| values.get(id).map(|v| (*id, v.clone())))
                        .collect();
                    self.pointing_data = Some(crate::types::PointingData::new(
                        pointing_ids,
                        pointing_values,
                    ));
                }

                self.qmk_settings_data = Some(crate::types::QmkSettingsData::new(ids, values));
                self.set_status(StatusMessage::info("All QMK settings reset to defaults"));
            }
            Err(e) => {
                let msg = format!("{e}");
                warn!(error = %e, "failed to reset QMK settings");
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
