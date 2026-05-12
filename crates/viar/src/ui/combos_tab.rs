use eframe::egui;
use tracing::{
    info,
    warn,
};
use via_protocol::{
    ComboEntry,
    Keycode,
    ViaProtocol,
};

use crate::{
    types::{
        ActiveKeycodeField,
        ComboField,
        StatusMessage,
        ViarApp,
    },
    util::{
        is_disconnect_error,
        keycode_chip,
        shared_keycode_picker,
    },
};

impl ViarApp {
    pub fn render_combos_tab(&mut self, ui: &mut egui::Ui) {
        let Some(dynamic) = &self.dynamic_data else {
            ui.label("Dynamic entries not supported by this keyboard.");
            return;
        };

        let count = dynamic.combos.len();
        if count == 0 {
            ui.vertical_centered(|ui| {
                ui.add_space(ui.available_height() / 3.0);
                ui.heading("No combo entries configured");
                ui.label("This keyboard has 0 combo slots.");
            });
            return;
        }

        let entries: Vec<(usize, ComboEntry)> = dynamic
            .combos
            .iter()
            .enumerate()
            .map(|(i, e)| (i, e.clone()))
            .collect();
        let editing = dynamic.editing_combo;
        let active_field = dynamic.active_field.clone();
        let editing_alias = dynamic.editing_alias.clone();
        let combo_names: Vec<String> = (0..count).map(|i| dynamic.combo_name(i)).collect();

        // List panel
        egui::Panel::left("combo_list_panel")
            .resizable(true)
            .default_size(200.0)
            .min_size(150.0)
            .max_size(300.0)
            .show_inside(ui, |ui| {
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new("Combos")
                        .size(20.0)
                        .strong()
                        .color(egui::Color32::from_rgb(200, 200, 215)),
                );
                ui.label(
                    egui::RichText::new(format!("{count} slots"))
                        .size(15.0)
                        .color(egui::Color32::from_rgb(120, 120, 135)),
                );
                ui.add_space(8.0);
                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (idx, entry) in &entries {
                        let is_selected = editing == Some(*idx);
                        let is_empty = entry.is_empty();
                        let alias_key = format!("combo:{idx}");
                        let is_renaming = editing_alias.as_deref() == Some(&alias_key);

                        let bg = if is_selected {
                            egui::Color32::from_rgb(45, 55, 75)
                        } else {
                            egui::Color32::TRANSPARENT
                        };

                        let frame = egui::Frame::default()
                            .inner_margin(egui::Margin::same(6))
                            .corner_radius(egui::CornerRadius::same(4))
                            .fill(bg);

                        let resp = frame
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    if is_renaming {
                                        let text_id = egui::Id::new(("combo_alias_edit", *idx));
                                        let first_frame_id = egui::Id::new(("combo_alias_first", *idx));
                                        let mut name: String = ui
                                            .memory(|mem| mem.data.get_temp(text_id))
                                            .unwrap_or_else(|| combo_names[*idx].clone());
                                        let resp = ui.add(
                                            egui::TextEdit::singleline(&mut name)
                                                .desired_width(100.0)
                                                .char_limit(12)
                                                .font(egui::TextStyle::Monospace),
                                        );
                                        ui.memory_mut(|mem| mem.data.insert_temp(text_id, name.clone()));

                                        let was_focused: bool = ui
                                            .memory(|mem| mem.data.get_temp(first_frame_id))
                                            .unwrap_or(false);
                                        if !was_focused {
                                            resp.request_focus();
                                            ui.memory_mut(|mem| mem.data.insert_temp(first_frame_id, true));
                                        }

                                        if resp.lost_focus() {
                                            if let Some(dynamic) = self.dynamic_data.as_mut() {
                                                let trimmed = name.trim().to_string();
                                                let default = format!("C{idx}");
                                                if trimmed.is_empty() || trimmed == default {
                                                    dynamic.aliases.remove(&alias_key);
                                                } else {
                                                    dynamic.aliases.insert(alias_key.clone(), trimmed);
                                                }
                                                dynamic.editing_alias = None;
                                                self.config.aliases = dynamic.aliases.clone();
                                                crate::theme::save_config(&self.config);
                                            }
                                            ui.memory_mut(|mem| {
                                                mem.data.remove::<String>(text_id);
                                                mem.data.remove::<bool>(first_frame_id);
                                            });
                                        }
                                    } else {
                                        ui.label(
                                            egui::RichText::new(&combo_names[*idx])
                                                .monospace()
                                                .size(16.0)
                                                .strong()
                                                .color(if is_selected {
                                                    egui::Color32::from_rgb(100, 180, 255)
                                                } else {
                                                    egui::Color32::from_rgb(170, 170, 185)
                                                }),
                                        );

                                        if is_empty {
                                            ui.label(
                                                egui::RichText::new("empty")
                                                    .italics()
                                                    .size(14.0)
                                                    .color(egui::Color32::from_rgb(90, 90, 100)),
                                            );
                                        } else {
                                            let inputs: Vec<String> = entry
                                                .input
                                                .iter()
                                                .filter(|&&k| k != 0)
                                                .map(|&k| Keycode(k).name())
                                                .collect();
                                            let out = Keycode(entry.output).name();
                                            ui.label(
                                                egui::RichText::new(format!(
                                                    "{}->{}",
                                                    inputs.join("+"),
                                                    out
                                                ))
                                                .size(13.0)
                                                .color(egui::Color32::from_rgb(130, 130, 145)),
                                            );
                                        }
                                    }
                                });
                            })
                            .response;

                        let resp = resp.interact(egui::Sense::click());
                        if resp.clicked() && !is_renaming
                            && let Some(dynamic) = self.dynamic_data.as_mut()
                        {
                            dynamic.editing_combo = Some(*idx);
                            dynamic.active_field = None;
                        }
                        if resp.double_clicked()
                            && let Some(dynamic) = self.dynamic_data.as_mut()
                        {
                            dynamic.editing_alias = Some(alias_key.clone());
                            let text_id = egui::Id::new(("combo_alias_edit", *idx));
                            let current = combo_names[*idx].clone();
                            ui.memory_mut(|mem| mem.data.insert_temp(text_id, current));
                        }
                        resp.on_hover_text("Double-click to rename");
                    }
                });
            });

        // Editor panel
        egui::CentralPanel::default().show_inside(ui, |ui| {
            let Some(editing_idx) = editing else {
                ui.vertical_centered(|ui| {
                    ui.add_space(ui.available_height() / 3.0);
                    ui.label(
                        egui::RichText::new("Select a combo from the list")
                            .size(18.0)
                            .color(egui::Color32::from_rgb(120, 120, 135)),
                    );
                });
                return;
            };

            let Some(entry) = entries.get(editing_idx).map(|(_, e)| e.clone()) else {
                return;
            };

            ui.add_space(8.0);
            ui.horizontal(|ui| {
                let display = combo_names[editing_idx].clone();
                ui.label(
                    egui::RichText::new(&display)
                        .monospace()
                        .size(22.0)
                        .strong()
                        .color(egui::Color32::from_rgb(200, 200, 215)),
                );
                let default_name = format!("C{editing_idx}");
                if display != default_name {
                    ui.label(
                        egui::RichText::new(format!("Combo {editing_idx}"))
                            .monospace()
                            .size(15.0)
                            .color(egui::Color32::from_rgb(120, 120, 135)),
                    );
                }
            });
            ui.label(
                egui::RichText::new(
                    "Press these keys together to output a different key. Click a field, then pick below.",
                )
                .size(15.0)
                .color(egui::Color32::from_rgb(110, 110, 125)),
            );
            ui.add_space(8.0);
            ui.separator();
            ui.add_space(4.0);

            // Input fields
            for i in 0..4usize {
                let label = format!("Input {}", i + 1);
                let field = ComboField::Input(i);
                let is_active = active_field
                    == Some(ActiveKeycodeField::Combo(editing_idx, field.clone()));
                if keycode_chip(ui, &label, entry.input[i], is_active)
                    && let Some(dynamic) = self.dynamic_data.as_mut() {
                        dynamic.active_field =
                            Some(ActiveKeycodeField::Combo(editing_idx, field));
                    }
            }

            ui.add_space(4.0);

            // Output field
            let is_active = active_field
                == Some(ActiveKeycodeField::Combo(editing_idx, ComboField::Output));
            if keycode_chip(ui, "Output", entry.output, is_active)
                && let Some(dynamic) = self.dynamic_data.as_mut() {
                    dynamic.active_field =
                        Some(ActiveKeycodeField::Combo(editing_idx, ComboField::Output));
                }

            // Shared picker
            if let Some(ActiveKeycodeField::Combo(eidx, ref field)) = active_field
                && eidx == editing_idx {
                    ui.add_space(12.0);
                    ui.separator();
                    ui.add_space(4.0);

                    let current_value = match field {
                        ComboField::Input(i) => entry.input[*i],
                        ComboField::Output => entry.output,
                    };

                    let field_label = match field {
                        ComboField::Input(i) => format!("Input {}", i + 1),
                        ComboField::Output => "Output".to_string(),
                    };

                    let mut group_idx = self
                        .dynamic_data
                        .as_ref()
                        .map(|d| d.picker_group_idx)
                        .unwrap_or(0);

                    let picker_result = shared_keycode_picker(
                        ui,
                        current_value,
                        &mut group_idx,
                        &self.picker_groups,
                        &field_label,
                        &self.theme,
                        self.dynamic_data.as_ref().map(|d| &d.aliases),
                    );

                    if let Some(dynamic) = self.dynamic_data.as_mut() {
                        dynamic.picker_group_idx = group_idx;
                    }

                    let new_val = if picker_result.cleared {
                        Some(0u16)
                    } else {
                        picker_result.selected
                    };

                    if let Some(val) = new_val
                        && let Some(dynamic) = self.dynamic_data.as_mut() {
                            let e = &mut dynamic.combos[editing_idx];
                            match field {
                                ComboField::Input(i) => e.input[*i] = val,
                                ComboField::Output => e.output = val,
                            }
                            let entry_clone = e.clone();
                            self.save_combo(editing_idx, &entry_clone);
                        }
                }
        });
    }

    fn save_combo(&mut self, idx: usize, entry: &ComboEntry) {
        let Some(dev) = &self.connected_device else {
            return;
        };
        let name = self
            .dynamic_data
            .as_ref()
            .map(|d| d.combo_name(idx))
            .unwrap_or_else(|| format!("C{idx}"));
        let proto = ViaProtocol::new(dev);
        match proto.set_combo(idx as u8, entry) {
            Ok(()) => {
                info!(idx, "combo saved to device");
                self.set_status(StatusMessage::info(format!("{name} saved")));
            }
            Err(e) => {
                let msg = format!("{e}");
                warn!(error = %e, idx, "failed to save combo");
                if is_disconnect_error(&msg) {
                    self.handle_disconnect();
                } else {
                    self.set_status(StatusMessage::error(format!(
                        "Failed to save {name}: {e}"
                    )));
                }
            }
        }
    }
}
