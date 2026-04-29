use eframe::egui;
use tracing::{
    info,
    warn,
};
use via_protocol::{
    KeyOverrideEntry,
    Keycode,
    ViaProtocol,
};

use crate::{
    types::{
        ActiveKeycodeField,
        KeyOverrideField,
        StatusMessage,
        ViarApp,
    },
    util::{
        is_disconnect_error,
        keycode_chip,
        shared_keycode_picker,
    },
};

/// Modifier flag names for display.
const MOD_FLAGS: [(u8, &str); 8] = [
    (0x01, "LCtrl"),
    (0x02, "LShift"),
    (0x04, "LAlt"),
    (0x08, "LGui"),
    (0x10, "RCtrl"),
    (0x20, "RShift"),
    (0x40, "RAlt"),
    (0x80, "RGui"),
];

fn _mods_string(mods: u8) -> String {
    if mods == 0 {
        return "None".to_string();
    }
    let names: Vec<&str> = MOD_FLAGS
        .iter()
        .filter(|(bit, _)| mods & bit != 0)
        .map(|(_, name)| *name)
        .collect();
    names.join("+")
}

fn render_mod_checkboxes(ui: &mut egui::Ui, label: &str, hint: &str, mods: &mut u8) -> bool {
    let mut changed = false;
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(format!("{label}:"))
                .size(12.0)
                .color(egui::Color32::from_rgb(140, 140, 155)),
        );
        ui.label(
            egui::RichText::new(hint)
                .size(10.0)
                .color(egui::Color32::from_rgb(100, 100, 115)),
        );
    });
    ui.horizontal_wrapped(|ui| {
        for (bit, name) in &MOD_FLAGS {
            let mut on = *mods & bit != 0;
            if ui.checkbox(&mut on, *name).changed() {
                if on {
                    *mods |= bit;
                } else {
                    *mods &= !bit;
                }
                changed = true;
            }
        }
    });
    changed
}

impl ViarApp {
    pub fn render_key_overrides_tab(&mut self, ui: &mut egui::Ui) {
        let Some(dynamic) = &self.dynamic_data else {
            ui.label("Dynamic entries not supported by this keyboard.");
            return;
        };

        let count = dynamic.key_overrides.len();
        if count == 0 {
            ui.vertical_centered(|ui| {
                ui.add_space(ui.available_height() / 3.0);
                ui.heading("No key override entries configured");
                ui.label("This keyboard has 0 key override slots.");
            });
            return;
        }

        let entries: Vec<(usize, KeyOverrideEntry)> = dynamic
            .key_overrides
            .iter()
            .enumerate()
            .map(|(i, e)| (i, e.clone()))
            .collect();
        let editing = dynamic.editing_key_override;
        let active_field = dynamic.active_field.clone();

        // List panel
        egui::Panel::left("ko_list_panel")
            .resizable(true)
            .default_size(200.0)
            .min_size(150.0)
            .max_size(300.0)
            .show_inside(ui, |ui| {
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new("Key Overrides")
                        .size(16.0)
                        .strong()
                        .color(egui::Color32::from_rgb(200, 200, 215)),
                );
                ui.label(
                    egui::RichText::new(format!("{count} slots"))
                        .size(11.0)
                        .color(egui::Color32::from_rgb(120, 120, 135)),
                );
                ui.add_space(8.0);
                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (idx, entry) in &entries {
                        let is_selected = editing == Some(*idx);
                        let is_empty = entry.is_empty();
                        let is_enabled = entry.is_enabled();

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
                                    ui.label(
                                        egui::RichText::new(format!("KO{idx}"))
                                            .monospace()
                                            .size(12.0)
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
                                                .size(10.0)
                                                .color(egui::Color32::from_rgb(90, 90, 100)),
                                        );
                                    } else {
                                        let trig = Keycode(entry.trigger).name();
                                        let repl = Keycode(entry.replacement).name();
                                        let status = if !is_enabled { " [OFF]" } else { "" };
                                        ui.label(
                                            egui::RichText::new(format!("{trig}->{repl}{status}"))
                                                .size(9.0)
                                                .color(egui::Color32::from_rgb(130, 130, 145)),
                                        );
                                    }
                                });
                            })
                            .response;

                        if resp.interact(egui::Sense::click()).clicked()
                            && let Some(dynamic) = self.dynamic_data.as_mut()
                        {
                            dynamic.editing_key_override = Some(*idx);
                            dynamic.active_field = None;
                        }
                    }
                });
            });

        // Editor panel
        egui::CentralPanel::default().show_inside(ui, |ui| {
            let Some(editing_idx) = editing else {
                ui.vertical_centered(|ui| {
                    ui.add_space(ui.available_height() / 3.0);
                    ui.label(
                        egui::RichText::new("Select a key override from the list")
                            .size(14.0)
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
                ui.label(
                    egui::RichText::new(format!("Key Override {editing_idx}"))
                        .monospace()
                        .size(18.0)
                        .strong()
                        .color(egui::Color32::from_rgb(200, 200, 215)),
                );
            });
            ui.label(
                egui::RichText::new("Override what a key does when specific modifiers are held.")
                    .size(11.0)
                    .color(egui::Color32::from_rgb(110, 110, 125)),
            );
            ui.add_space(8.0);
            ui.separator();
            ui.add_space(4.0);

            // Enabled toggle
            let mut enabled = entry.is_enabled();
            if ui.checkbox(&mut enabled, "Enabled").changed()
                && let Some(dynamic) = self.dynamic_data.as_mut()
            {
                dynamic.key_overrides[editing_idx].set_enabled(enabled);
                let e = dynamic.key_overrides[editing_idx].clone();
                self.save_key_override(editing_idx, &e);
            }

            ui.add_space(4.0);

            // Trigger key chip
            let is_active = active_field
                == Some(ActiveKeycodeField::KeyOverride(
                    editing_idx,
                    KeyOverrideField::Trigger,
                ));
            if keycode_chip(ui, "Trigger Key", entry.trigger, is_active)
                && let Some(dynamic) = self.dynamic_data.as_mut()
            {
                dynamic.active_field = Some(ActiveKeycodeField::KeyOverride(
                    editing_idx,
                    KeyOverrideField::Trigger,
                ));
            }

            // Replacement key chip
            let is_active = active_field
                == Some(ActiveKeycodeField::KeyOverride(
                    editing_idx,
                    KeyOverrideField::Replacement,
                ));
            if keycode_chip(ui, "Replacement", entry.replacement, is_active)
                && let Some(dynamic) = self.dynamic_data.as_mut()
            {
                dynamic.active_field = Some(ActiveKeycodeField::KeyOverride(
                    editing_idx,
                    KeyOverrideField::Replacement,
                ));
            }

            ui.add_space(4.0);
            ui.separator();

            // Modifier checkboxes — these mutate directly
            egui::ScrollArea::vertical()
                .id_salt("ko_editor_scroll")
                .show(ui, |ui| {
                    let mut changed = false;

                    // We need mutable access for checkboxes
                    if let Some(dynamic) = self.dynamic_data.as_mut() {
                        let e = &mut dynamic.key_overrides[editing_idx];

                        if render_mod_checkboxes(
                            ui,
                            "Trigger Mods",
                            "Must be held for override to activate",
                            &mut e.trigger_mods,
                        ) {
                            changed = true;
                        }

                        if render_mod_checkboxes(
                            ui,
                            "Negative Mods",
                            "Prevent override when held",
                            &mut e.negative_mod_mask,
                        ) {
                            changed = true;
                        }

                        if render_mod_checkboxes(
                            ui,
                            "Suppressed Mods",
                            "Removed from output when override activates",
                            &mut e.suppressed_mods,
                        ) {
                            changed = true;
                        }

                        // Layers
                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("Active Layers:")
                                    .size(12.0)
                                    .color(egui::Color32::from_rgb(140, 140, 155)),
                            );
                        });
                        ui.horizontal_wrapped(|ui| {
                            for layer in 0..16u16 {
                                let mut on = e.layers & (1 << layer) != 0;
                                if ui.checkbox(&mut on, format!("{layer}")).changed() {
                                    if on {
                                        e.layers |= 1 << layer;
                                    } else {
                                        e.layers &= !(1 << layer);
                                    }
                                    changed = true;
                                }
                            }
                        });

                        if changed {
                            let entry_clone = e.clone();
                            let idx = editing_idx;
                            let _ = dynamic;
                            self.save_key_override(idx, &entry_clone);
                        }
                    }

                    // Shared picker for keycode fields
                    if let Some(ActiveKeycodeField::KeyOverride(eidx, ref field)) = active_field
                        && eidx == editing_idx
                    {
                        ui.add_space(12.0);
                        ui.separator();
                        ui.add_space(4.0);

                        let current_value = match field {
                            KeyOverrideField::Trigger => entry.trigger,
                            KeyOverrideField::Replacement => entry.replacement,
                        };

                        let field_label = match field {
                            KeyOverrideField::Trigger => "Trigger Key",
                            KeyOverrideField::Replacement => "Replacement Key",
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
                            let e = &mut dynamic.key_overrides[editing_idx];
                            match field {
                                KeyOverrideField::Trigger => e.trigger = val,
                                KeyOverrideField::Replacement => e.replacement = val,
                            }
                            let entry_clone = e.clone();
                            self.save_key_override(editing_idx, &entry_clone);
                        }
                    }
                });
        });
    }

    fn save_key_override(&mut self, idx: usize, entry: &KeyOverrideEntry) {
        let Some(dev) = &self.connected_device else {
            return;
        };
        let proto = ViaProtocol::new(dev);
        match proto.set_key_override(idx as u8, entry) {
            Ok(()) => {
                info!(idx, "key override saved to device");
                self.set_status(StatusMessage::info(format!("KO {idx} saved")));
            }
            Err(e) => {
                let msg = format!("{e}");
                warn!(error = %e, idx, "failed to save key override");
                if is_disconnect_error(&msg) {
                    self.handle_disconnect();
                } else {
                    self.set_status(StatusMessage::error(format!(
                        "Failed to save KO {idx}: {e}"
                    )));
                }
            }
        }
    }
}
