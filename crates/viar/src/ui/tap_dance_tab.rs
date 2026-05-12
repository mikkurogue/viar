use eframe::egui;
use tracing::{
    info,
    warn,
};
use via_protocol::{
    Keycode,
    TapDanceEntry,
    ViaProtocol,
};

use crate::{
    types::{
        ActiveKeycodeField,
        StatusMessage,
        TapDanceField,
        ViarApp,
    },
    util::{
        is_disconnect_error,
        keycode_chip,
        shared_keycode_picker,
    },
};

impl ViarApp {
    pub fn render_tap_dance_tab(&mut self, ui: &mut egui::Ui) {
        let Some(dynamic) = &self.dynamic_data else {
            ui.label("Dynamic entries not supported by this keyboard.");
            return;
        };

        let count = dynamic.tap_dances.len();
        if count == 0 {
            ui.vertical_centered(|ui| {
                ui.add_space(ui.available_height() / 3.0);
                ui.heading("No tap dance entries configured");
                ui.label("This keyboard has 0 tap dance slots.");
            });
            return;
        }

        // Clone state needed for rendering
        let entries: Vec<(usize, TapDanceEntry)> = dynamic
            .tap_dances
            .iter()
            .enumerate()
            .map(|(i, e)| (i, e.clone()))
            .collect();
        let editing = dynamic.editing_tap_dance;
        let active_field = dynamic.active_field.clone();
        let editing_alias = dynamic.editing_alias.clone();
        let td_names: Vec<String> = (0..count).map(|i| dynamic.td_name(i)).collect();

        // Split layout: list on left, editor on right
        egui::Panel::left("td_list_panel")
            .resizable(true)
            .default_size(180.0)
            .min_size(140.0)
            .max_size(280.0)
            .show_inside(ui, |ui| {
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new("Tap Dance")
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
                        let alias_key = format!("td:{idx}");
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
                                        // Inline text edit for renaming
                                        let text_id = egui::Id::new(("td_alias_edit", *idx));
                                        let first_frame_id = egui::Id::new(("td_alias_first", *idx));
                                        let mut name: String = ui
                                            .memory(|mem| mem.data.get_temp(text_id))
                                            .unwrap_or_else(|| td_names[*idx].clone());
                                        let resp = ui.add(
                                            egui::TextEdit::singleline(&mut name)
                                                .desired_width(100.0)
                                                .char_limit(12)
                                                .font(egui::TextStyle::Monospace),
                                        );
                                        ui.memory_mut(|mem| mem.data.insert_temp(text_id, name.clone()));

                                        // Auto-focus only on first frame
                                        let was_focused: bool = ui
                                            .memory(|mem| mem.data.get_temp(first_frame_id))
                                            .unwrap_or(false);
                                        if !was_focused {
                                            resp.request_focus();
                                            ui.memory_mut(|mem| mem.data.insert_temp(first_frame_id, true));
                                        }

                                        // Commit on Enter or focus loss
                                        if resp.lost_focus() {
                                            if let Some(dynamic) = self.dynamic_data.as_mut() {
                                                let trimmed = name.trim().to_string();
                                                let default = format!("TD({idx})");
                                                if trimmed.is_empty() || trimmed == default {
                                                    dynamic.aliases.remove(&alias_key);
                                                } else {
                                                    dynamic.aliases.insert(alias_key.clone(), trimmed);
                                                }
                                                dynamic.editing_alias = None;
                                                // Persist
                                                self.config.aliases = dynamic.aliases.clone();
                                                crate::theme::save_config(&self.config);
                                            }
                                            // Clean up temp data
                                            ui.memory_mut(|mem| {
                                                mem.data.remove::<String>(text_id);
                                                mem.data.remove::<bool>(first_frame_id);
                                            });
                                        }
                                    } else {
                                        ui.label(
                                            egui::RichText::new(&td_names[*idx])
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
                                            // Show tap key name as summary
                                            let tap_kc = Keycode(entry.on_tap);
                                            ui.label(
                                                egui::RichText::new(tap_kc.name())
                                                    .size(14.0)
                                                    .color(egui::Color32::from_rgb(140, 140, 155)),
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
                            dynamic.editing_tap_dance = Some(*idx);
                            dynamic.active_field = None;
                        }
                        if resp.double_clicked()
                            && let Some(dynamic) = self.dynamic_data.as_mut()
                        {
                            dynamic.editing_alias = Some(alias_key.clone());
                            // Seed the edit buffer
                            let text_id = egui::Id::new(("td_alias_edit", *idx));
                            let current = td_names[*idx].clone();
                            ui.memory_mut(|mem| mem.data.insert_temp(text_id, current));
                        }
                        resp.on_hover_text("Double-click to rename");
                    }
                });
            });

        // Editor panel (central)
        egui::CentralPanel::default().show_inside(ui, |ui| {
            let Some(editing_idx) = editing else {
                ui.vertical_centered(|ui| {
                    ui.add_space(ui.available_height() / 3.0);
                    ui.label(
                        egui::RichText::new("Select a tap dance from the list")
                            .size(18.0)
                            .color(egui::Color32::from_rgb(120, 120, 135)),
                    );
                });
                return;
            };

            let Some(entry) = entries.get(editing_idx).map(|(_, e)| e.clone()) else {
                return;
            };

            // Top: entry header
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                let display = td_names[editing_idx].clone();
                ui.label(
                    egui::RichText::new(&display)
                        .monospace()
                        .size(22.0)
                        .strong()
                        .color(egui::Color32::from_rgb(200, 200, 215)),
                );
                // Show the original TD index if renamed
                let default_name = format!("TD({editing_idx})");
                if display != default_name {
                    ui.label(
                        egui::RichText::new(&default_name)
                            .monospace()
                            .size(15.0)
                            .color(egui::Color32::from_rgb(120, 120, 135)),
                    );
                }
                ui.label(
                    egui::RichText::new(format!("0x{:04X}", 0x5700u16 | editing_idx as u16))
                        .monospace()
                        .size(15.0)
                        .color(egui::Color32::from_rgb(90, 90, 105)),
                );
            });
            ui.label(
                egui::RichText::new(
                    "Click a field below, then pick a key from the picker at the bottom.",
                )
                .size(15.0)
                .color(egui::Color32::from_rgb(110, 110, 125)),
            );
            ui.add_space(8.0);
            ui.separator();
            ui.add_space(4.0);

            // Fields as clickable chips
            let fields = [
                ("On Tap", entry.on_tap, TapDanceField::OnTap),
                ("On Hold", entry.on_hold, TapDanceField::OnHold),
                (
                    "On Double Tap",
                    entry.on_double_tap,
                    TapDanceField::OnDoubleTap,
                ),
                ("On Tap+Hold", entry.on_tap_hold, TapDanceField::OnTapHold),
            ];

            for (label, value, field) in &fields {
                let is_active =
                    active_field == Some(ActiveKeycodeField::TapDance(editing_idx, field.clone()));
                if keycode_chip(ui, label, *value, is_active)
                    && let Some(dynamic) = self.dynamic_data.as_mut()
                {
                    dynamic.active_field =
                        Some(ActiveKeycodeField::TapDance(editing_idx, field.clone()));
                }
            }

            // Tapping term slider
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("Tapping Term:")
                        .size(16.0)
                        .color(egui::Color32::from_rgb(140, 140, 155)),
                );
                let mut val = entry.tapping_term as f32;
                if ui
                    .add(
                        egui::Slider::new(&mut val, 0.0..=500.0)
                            .suffix(" ms")
                            .integer(),
                    )
                    .changed()
                    && let Some(dynamic) = self.dynamic_data.as_mut()
                {
                    dynamic.tap_dances[editing_idx].tapping_term = val as u16;
                    let e = dynamic.tap_dances[editing_idx].clone();
                    self.save_tap_dance(editing_idx, &e);
                }
                if entry.tapping_term == 0 {
                    ui.label(
                        egui::RichText::new("(global default)")
                            .size(14.0)
                            .color(egui::Color32::from_rgb(100, 100, 115)),
                    );
                }
            });

            // Shared picker at the bottom
            if let Some(ActiveKeycodeField::TapDance(eidx, ref field)) = active_field
                && eidx == editing_idx
            {
                ui.add_space(12.0);
                ui.separator();
                ui.add_space(4.0);

                let current_value = match field {
                    TapDanceField::OnTap => entry.on_tap,
                    TapDanceField::OnHold => entry.on_hold,
                    TapDanceField::OnDoubleTap => entry.on_double_tap,
                    TapDanceField::OnTapHold => entry.on_tap_hold,
                };

                let field_label = match field {
                    TapDanceField::OnTap => "On Tap",
                    TapDanceField::OnHold => "On Hold",
                    TapDanceField::OnDoubleTap => "On Double Tap",
                    TapDanceField::OnTapHold => "On Tap+Hold",
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
                    field_label,
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
                    && let Some(dynamic) = self.dynamic_data.as_mut()
                {
                    let e = &mut dynamic.tap_dances[editing_idx];
                    match field {
                        TapDanceField::OnTap => e.on_tap = val,
                        TapDanceField::OnHold => e.on_hold = val,
                        TapDanceField::OnDoubleTap => e.on_double_tap = val,
                        TapDanceField::OnTapHold => e.on_tap_hold = val,
                    }
                    let entry_clone = e.clone();
                    self.save_tap_dance(editing_idx, &entry_clone);
                }
            }
        });
    }

    fn save_tap_dance(&mut self, idx: usize, entry: &TapDanceEntry) {
        let Some(dev) = &self.connected_device else {
            return;
        };
        let name = self
            .dynamic_data
            .as_ref()
            .map(|d| d.td_name(idx))
            .unwrap_or_else(|| format!("TD({idx})"));
        let proto = ViaProtocol::new(dev);
        match proto.set_tap_dance(idx as u8, entry) {
            Ok(()) => {
                info!(idx, "tap dance saved to device");
                self.set_status(StatusMessage::info(format!("{name} saved")));
            }
            Err(e) => {
                let msg = format!("{e}");
                warn!(error = %e, idx, "failed to save tap dance");
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
