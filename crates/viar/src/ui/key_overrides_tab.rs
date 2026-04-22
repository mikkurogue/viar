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
        StatusMessage,
        ViarApp,
    },
    util::is_disconnect_error,
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

fn mods_string(mods: u8) -> String {
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

fn render_mod_checkboxes(ui: &mut egui::Ui, label: &str, mods: &mut u8) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(format!("{label}:"))
                .size(13.0)
                .color(egui::Color32::from_rgb(160, 160, 175)),
        );
        ui.add_space(8.0);
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

        ui.add_space(12.0);

        let max_width = 700.0_f32.min(ui.available_width() - 40.0);
        ui.vertical_centered(|ui| {
            ui.set_max_width(max_width);
            ui.heading("Key Overrides");
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new(format!("{count} slots"))
                    .size(12.0)
                    .color(egui::Color32::from_rgb(140, 140, 155)),
            );
            ui.add_space(16.0);
        });

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.set_max_width(max_width);
            ui.vertical_centered(|ui| {
                ui.set_max_width(max_width);

                let entries: Vec<(usize, KeyOverrideEntry)> = self
                    .dynamic_data
                    .as_ref()
                    .unwrap()
                    .key_overrides
                    .iter()
                    .enumerate()
                    .map(|(i, e)| (i, e.clone()))
                    .collect();
                let editing = self.dynamic_data.as_ref().unwrap().editing_key_override;

                for (idx, entry) in &entries {
                    let is_editing = editing == Some(*idx);
                    let is_empty = entry.is_empty();
                    let is_enabled = entry.is_enabled();

                    let frame = egui::Frame::default()
                        .inner_margin(egui::Margin::same(12))
                        .outer_margin(egui::Margin::symmetric(0, 4))
                        .corner_radius(egui::CornerRadius::same(6))
                        .fill(if is_editing {
                            egui::Color32::from_rgb(40, 45, 55)
                        } else {
                            egui::Color32::from_rgb(30, 30, 35)
                        })
                        .stroke(egui::Stroke::new(
                            1.0,
                            if is_editing {
                                egui::Color32::from_rgb(80, 120, 180)
                            } else if !is_empty && is_enabled {
                                egui::Color32::from_rgb(60, 70, 60)
                            } else {
                                egui::Color32::from_rgb(50, 50, 55)
                            },
                        ));

                    frame.show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(format!("KO {idx}"))
                                    .monospace()
                                    .strong()
                                    .color(egui::Color32::from_rgb(180, 180, 200)),
                            );

                            if is_empty {
                                ui.label(
                                    egui::RichText::new("(empty)")
                                        .italics()
                                        .color(egui::Color32::from_rgb(100, 100, 110)),
                                );
                            } else if !is_enabled {
                                ui.label(
                                    egui::RichText::new("(disabled)")
                                        .italics()
                                        .color(egui::Color32::from_rgb(130, 100, 100)),
                                );
                            }

                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if is_editing {
                                        if ui.button("Close").clicked() {
                                            self.dynamic_data
                                                .as_mut()
                                                .unwrap()
                                                .editing_key_override = None;
                                        }
                                    } else if ui.button("Edit").clicked() {
                                        self.dynamic_data.as_mut().unwrap().editing_key_override =
                                            Some(*idx);
                                    }
                                },
                            );
                        });

                        if is_editing {
                            ui.add_space(8.0);
                            let mut changed = false;
                            let dynamic = self.dynamic_data.as_mut().unwrap();
                            let entry = &mut dynamic.key_overrides[*idx];

                            // Enabled toggle
                            ui.horizontal(|ui| {
                                let mut enabled = entry.is_enabled();
                                if ui.checkbox(&mut enabled, "Enabled").changed() {
                                    entry.set_enabled(enabled);
                                    changed = true;
                                }
                            });
                            ui.add_space(4.0);

                            // Trigger key
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new("Trigger:")
                                        .size(13.0)
                                        .color(egui::Color32::from_rgb(160, 160, 175)),
                                );
                                ui.add_space(8.0);
                                let kc = Keycode(entry.trigger);
                                ui.button(
                                    egui::RichText::new(kc.short_name()).monospace().size(13.0),
                                )
                                .on_hover_text(format!(
                                    "0x{:04X} — {}",
                                    entry.trigger,
                                    kc.name()
                                ));

                                let mut hex = format!("{:04X}", entry.trigger);
                                if ui
                                    .add(
                                        egui::TextEdit::singleline(&mut hex)
                                            .desired_width(60.0)
                                            .font(egui::TextStyle::Monospace),
                                    )
                                    .changed()
                                {
                                    if let Ok(v) = u16::from_str_radix(hex.trim(), 16) {
                                        entry.trigger = v;
                                        changed = true;
                                    }
                                }
                            });

                            // Replacement key
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new("Replacement:")
                                        .size(13.0)
                                        .color(egui::Color32::from_rgb(160, 160, 175)),
                                );
                                ui.add_space(8.0);
                                let kc = Keycode(entry.replacement);
                                ui.button(
                                    egui::RichText::new(kc.short_name()).monospace().size(13.0),
                                )
                                .on_hover_text(format!(
                                    "0x{:04X} — {}",
                                    entry.replacement,
                                    kc.name()
                                ));

                                let mut hex = format!("{:04X}", entry.replacement);
                                if ui
                                    .add(
                                        egui::TextEdit::singleline(&mut hex)
                                            .desired_width(60.0)
                                            .font(egui::TextStyle::Monospace),
                                    )
                                    .changed()
                                {
                                    if let Ok(v) = u16::from_str_radix(hex.trim(), 16) {
                                        entry.replacement = v;
                                        changed = true;
                                    }
                                }
                            });

                            ui.add_space(4.0);

                            // Trigger mods
                            if render_mod_checkboxes(ui, "Trigger Mods", &mut entry.trigger_mods) {
                                changed = true;
                            }

                            // Negative mods
                            if render_mod_checkboxes(
                                ui,
                                "Negative Mods",
                                &mut entry.negative_mod_mask,
                            ) {
                                changed = true;
                            }

                            // Suppressed mods
                            if render_mod_checkboxes(
                                ui,
                                "Suppressed Mods",
                                &mut entry.suppressed_mods,
                            ) {
                                changed = true;
                            }

                            ui.add_space(4.0);

                            // Layers bitmask
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new("Active Layers:")
                                        .size(13.0)
                                        .color(egui::Color32::from_rgb(160, 160, 175)),
                                );
                                ui.add_space(8.0);
                                for layer in 0..16u16 {
                                    let mut on = entry.layers & (1 << layer) != 0;
                                    if ui.checkbox(&mut on, format!("{layer}")).changed() {
                                        if on {
                                            entry.layers |= 1 << layer;
                                        } else {
                                            entry.layers &= !(1 << layer);
                                        }
                                        changed = true;
                                    }
                                }
                            });

                            if changed {
                                let entry_clone = entry.clone();
                                let i = *idx;
                                let _ = dynamic;
                                self.save_key_override(i, &entry_clone);
                            }
                        } else if !is_empty {
                            // Show summary
                            ui.horizontal(|ui| {
                                let trigger = Keycode(entry.trigger).short_name();
                                let replacement = Keycode(entry.replacement).short_name();
                                let mods = mods_string(entry.trigger_mods);
                                let status = if is_enabled { "" } else { " [OFF]" };
                                ui.label(
                                    egui::RichText::new(format!(
                                        "{mods}+{trigger} → {replacement}{status}"
                                    ))
                                    .size(12.0)
                                    .color(egui::Color32::from_rgb(140, 150, 170)),
                                );
                            });
                        }
                    });
                }
            });
        });
    }

    fn save_key_override(&mut self, idx: usize, entry: &KeyOverrideEntry) {
        if let Some(dev) = &self.connected_device {
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
}
