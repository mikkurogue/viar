use eframe::egui;
use tracing::{info, warn};
use via_protocol::{Keycode, TapDanceEntry, ViaProtocol};

use crate::types::{StatusMessage, ViarApp};
use crate::util::is_disconnect_error;

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

        ui.add_space(12.0);

        let max_width = 600.0_f32.min(ui.available_width() - 40.0);
        ui.vertical_centered(|ui| {
            ui.set_max_width(max_width);

            ui.heading("Tap Dance");
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
                // We need to clone data out to avoid borrow issues
                let entries: Vec<(usize, TapDanceEntry)> = self
                    .dynamic_data
                    .as_ref()
                    .unwrap()
                    .tap_dances
                    .iter()
                    .enumerate()
                    .map(|(i, e)| (i, e.clone()))
                    .collect();
                let editing = self.dynamic_data.as_ref().unwrap().editing_tap_dance;

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
                                egui::RichText::new(format!("TD({idx})"))
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
                                            self.dynamic_data.as_mut().unwrap().editing_tap_dance =
                                                None;
                                        }
                                    } else if ui.button("Edit").clicked() {
                                        self.dynamic_data.as_mut().unwrap().editing_tap_dance =
                                            Some(*idx);
                                    }
                                },
                            );
                        });

                        if is_editing {
                            ui.add_space(8.0);
                            let mut changed = false;
                            let dynamic = self.dynamic_data.as_mut().unwrap();
                            let entry = &mut dynamic.tap_dances[*idx];

                            let fields = [
                                ("On Tap", &mut entry.on_tap),
                                ("On Hold", &mut entry.on_hold),
                                ("On Double Tap", &mut entry.on_double_tap),
                                ("On Tap Hold", &mut entry.on_tap_hold),
                            ];

                            for (label, value) in fields {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        egui::RichText::new(format!("{label}:"))
                                            .size(13.0)
                                            .color(egui::Color32::from_rgb(160, 160, 175)),
                                    );
                                    ui.add_space(8.0);

                                    let kc = Keycode(*value);
                                    let name = kc.short_name();
                                    let btn = ui
                                        .button(egui::RichText::new(&name).monospace().size(13.0));
                                    if btn.secondary_clicked() {
                                        // Clear the keycode
                                        *value = 0;
                                        changed = true;
                                    }
                                    btn.on_hover_text(format!(
                                        "0x{:04X} — {}. Right-click to clear.",
                                        value,
                                        kc.name()
                                    ));

                                    // Simple keycode entry via text input
                                    let mut hex = format!("{:04X}", *value);
                                    let resp = ui.add(
                                        egui::TextEdit::singleline(&mut hex)
                                            .desired_width(60.0)
                                            .font(egui::TextStyle::Monospace),
                                    );
                                    if resp.changed() {
                                        if let Ok(v) = u16::from_str_radix(hex.trim(), 16) {
                                            *value = v;
                                            changed = true;
                                        }
                                    }
                                });
                            }

                            // Tapping term
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new("Tapping Term:")
                                        .size(13.0)
                                        .color(egui::Color32::from_rgb(160, 160, 175)),
                                );
                                ui.add_space(8.0);
                                let mut val = entry.tapping_term as f32;
                                if ui
                                    .add(
                                        egui::Slider::new(&mut val, 0.0..=500.0)
                                            .suffix(" ms")
                                            .integer(),
                                    )
                                    .changed()
                                {
                                    entry.tapping_term = val as u16;
                                    changed = true;
                                }
                            });

                            if changed {
                                let entry_clone = entry.clone();
                                let i = *idx;
                                // End the borrow of dynamic_data before calling save
                                let _ = dynamic;
                                self.save_tap_dance(i, &entry_clone);
                            }
                        } else if !is_empty {
                            // Show summary
                            ui.horizontal(|ui| {
                                let labels = [
                                    ("Tap", entry.on_tap),
                                    ("Hold", entry.on_hold),
                                    ("DTap", entry.on_double_tap),
                                    ("THold", entry.on_tap_hold),
                                ];
                                for (label, kc_raw) in labels {
                                    if kc_raw != 0 {
                                        let kc = Keycode(kc_raw);
                                        ui.label(
                                            egui::RichText::new(format!(
                                                "{}: {}",
                                                label,
                                                kc.short_name()
                                            ))
                                            .size(12.0)
                                            .color(egui::Color32::from_rgb(140, 150, 170)),
                                        );
                                        ui.add_space(8.0);
                                    }
                                }
                                if entry.tapping_term > 0 {
                                    ui.label(
                                        egui::RichText::new(format!("{}ms", entry.tapping_term))
                                            .size(12.0)
                                            .color(egui::Color32::from_rgb(120, 130, 140)),
                                    );
                                }
                            });
                        }
                    });
                }
            });
        });
    }

    fn save_tap_dance(&mut self, idx: usize, entry: &TapDanceEntry) {
        if let Some(dev) = &self.connected_device {
            let proto = ViaProtocol::new(dev);
            match proto.set_tap_dance(idx as u8, entry) {
                Ok(()) => {
                    info!(idx, "tap dance saved to device");
                    self.set_status(StatusMessage::info(format!("TD({idx}) saved")));
                }
                Err(e) => {
                    let msg = format!("{e}");
                    warn!(error = %e, idx, "failed to save tap dance");
                    if is_disconnect_error(&msg) {
                        self.handle_disconnect();
                    } else {
                        self.set_status(StatusMessage::error(format!(
                            "Failed to save TD({idx}): {e}"
                        )));
                    }
                }
            }
        }
    }
}
