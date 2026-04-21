use eframe::egui;
use tracing::{info, warn};
use via_protocol::{ComboEntry, Keycode, ViaProtocol};

use crate::types::{StatusMessage, ViarApp};
use crate::util::is_disconnect_error;

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

        ui.add_space(12.0);

        let max_width = 600.0_f32.min(ui.available_width() - 40.0);
        ui.vertical_centered(|ui| {
            ui.set_max_width(max_width);
            ui.heading("Combos");
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

                let entries: Vec<(usize, ComboEntry)> = self
                    .dynamic_data
                    .as_ref()
                    .unwrap()
                    .combos
                    .iter()
                    .enumerate()
                    .map(|(i, e)| (i, e.clone()))
                    .collect();
                let editing = self.dynamic_data.as_ref().unwrap().editing_combo;

                for (idx, entry) in &entries {
                    let is_editing = editing == Some(*idx);
                    let is_empty = entry.is_empty();

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
                            } else {
                                egui::Color32::from_rgb(50, 50, 55)
                            },
                        ));

                    frame.show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(format!("Combo {idx}"))
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
                            }

                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if is_editing {
                                        if ui.button("Close").clicked() {
                                            self.dynamic_data.as_mut().unwrap().editing_combo =
                                                None;
                                        }
                                    } else if ui.button("Edit").clicked() {
                                        self.dynamic_data.as_mut().unwrap().editing_combo =
                                            Some(*idx);
                                    }
                                },
                            );
                        });

                        if is_editing {
                            ui.add_space(8.0);
                            let mut changed = false;
                            let dynamic = self.dynamic_data.as_mut().unwrap();
                            let entry = &mut dynamic.combos[*idx];

                            // Input keys (up to 4)
                            for input_idx in 0..4 {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        egui::RichText::new(format!("Input {}:", input_idx + 1))
                                            .size(13.0)
                                            .color(egui::Color32::from_rgb(160, 160, 175)),
                                    );
                                    ui.add_space(8.0);

                                    let kc = Keycode(entry.input[input_idx]);
                                    let name = kc.short_name();
                                    let btn = ui
                                        .button(egui::RichText::new(&name).monospace().size(13.0));
                                    if btn.secondary_clicked() {
                                        entry.input[input_idx] = 0;
                                        changed = true;
                                    }
                                    btn.on_hover_text(format!(
                                        "0x{:04X} — {}. Right-click to clear.",
                                        entry.input[input_idx],
                                        kc.name()
                                    ));

                                    let mut hex = format!("{:04X}", entry.input[input_idx]);
                                    let resp = ui.add(
                                        egui::TextEdit::singleline(&mut hex)
                                            .desired_width(60.0)
                                            .font(egui::TextStyle::Monospace),
                                    );
                                    if resp.changed() {
                                        if let Ok(v) = u16::from_str_radix(hex.trim(), 16) {
                                            entry.input[input_idx] = v;
                                            changed = true;
                                        }
                                    }
                                });
                            }

                            // Output key
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new("Output:")
                                        .size(13.0)
                                        .color(egui::Color32::from_rgb(160, 160, 175)),
                                );
                                ui.add_space(8.0);

                                let kc = Keycode(entry.output);
                                let name = kc.short_name();
                                let btn =
                                    ui.button(egui::RichText::new(&name).monospace().size(13.0));
                                if btn.secondary_clicked() {
                                    entry.output = 0;
                                    changed = true;
                                }
                                btn.on_hover_text(format!(
                                    "0x{:04X} — {}. Right-click to clear.",
                                    entry.output,
                                    kc.name()
                                ));

                                let mut hex = format!("{:04X}", entry.output);
                                let resp = ui.add(
                                    egui::TextEdit::singleline(&mut hex)
                                        .desired_width(60.0)
                                        .font(egui::TextStyle::Monospace),
                                );
                                if resp.changed() {
                                    if let Ok(v) = u16::from_str_radix(hex.trim(), 16) {
                                        entry.output = v;
                                        changed = true;
                                    }
                                }
                            });

                            if changed {
                                let entry_clone = entry.clone();
                                let i = *idx;
                                let _ = dynamic;
                                self.save_combo(i, &entry_clone);
                            }
                        } else if !is_empty {
                            // Show summary
                            ui.horizontal(|ui| {
                                let inputs: Vec<String> = entry
                                    .input
                                    .iter()
                                    .filter(|&&k| k != 0)
                                    .map(|&k| Keycode(k).short_name())
                                    .collect();
                                let output = Keycode(entry.output).short_name();
                                ui.label(
                                    egui::RichText::new(format!(
                                        "{} → {}",
                                        inputs.join(" + "),
                                        output
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

    fn save_combo(&mut self, idx: usize, entry: &ComboEntry) {
        if let Some(dev) = &self.connected_device {
            let proto = ViaProtocol::new(dev);
            match proto.set_combo(idx as u8, entry) {
                Ok(()) => {
                    info!(idx, "combo saved to device");
                    self.set_status(StatusMessage::info(format!("Combo {idx} saved")));
                }
                Err(e) => {
                    let msg = format!("{e}");
                    warn!(error = %e, idx, "failed to save combo");
                    if is_disconnect_error(&msg) {
                        self.handle_disconnect();
                    } else {
                        self.set_status(StatusMessage::error(format!(
                            "Failed to save Combo {idx}: {e}"
                        )));
                    }
                }
            }
        }
    }
}
