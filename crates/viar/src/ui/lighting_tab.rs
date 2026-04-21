use eframe::egui;
use tracing::{info, warn};
use via_protocol::{LightingProtocol, ViaProtocol};

use crate::types::{StatusMessage, ViarApp};
use crate::util::{hsv_to_rgb, is_disconnect_error};

impl ViarApp {
    pub fn render_lighting_tab(&mut self, ui: &mut egui::Ui) {
        let Some(lighting) = &mut self.lighting_data else {
            ui.label("No lighting data available.");
            return;
        };

        let protocol_name = match &lighting.protocol {
            LightingProtocol::Via { channel } => {
                use via_protocol::LightingChannel;
                match channel {
                    LightingChannel::QmkBacklight => "QMK Backlight (VIA)",
                    LightingChannel::QmkRgblight => "QMK Rgblight (VIA)",
                    LightingChannel::QmkRgbMatrix => "QMK RGB Matrix (VIA)",
                    LightingChannel::QmkAudio => "QMK Audio (VIA)",
                    LightingChannel::QmkLedMatrix => "QMK LED Matrix (VIA)",
                }
            }
            LightingProtocol::VialLegacy => "RGB Matrix (Vial Legacy)",
            LightingProtocol::VialRgb => "RGB Matrix (VialRGB)",
        };
        let is_vialrgb = matches!(lighting.protocol, LightingProtocol::VialRgb);

        ui.add_space(20.0);

        let max_width = 500.0_f32.min(ui.available_width() - 40.0);
        ui.vertical_centered(|ui| {
            ui.set_max_width(max_width);

            ui.heading("Lighting Configuration");
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new(protocol_name)
                    .size(12.0)
                    .color(egui::Color32::from_rgb(140, 140, 155)),
            );
            ui.add_space(16.0);

            // Brightness slider
            ui.horizontal(|ui| {
                ui.label("Brightness");
                ui.add_space(8.0);
                let mut val = lighting.brightness as f32;
                if ui
                    .add(egui::Slider::new(&mut val, 0.0..=255.0).integer())
                    .changed()
                {
                    lighting.brightness = val as u8;
                    lighting.dirty = true;
                }
            });
            ui.add_space(8.0);

            // Effect selector
            if is_vialrgb && !lighting.supported_effects.is_empty() {
                ui.horizontal(|ui| {
                    ui.label("Effect");
                    ui.add_space(8.0);
                    let current_name = via_protocol::VialRgbEffect::from_u16(lighting.effect_id)
                        .map(|e| e.name())
                        .unwrap_or("Unknown");
                    egui::ComboBox::from_id_salt("effect_combo")
                        .selected_text(current_name)
                        .width(280.0)
                        .show_ui(ui, |ui| {
                            for &eid in &lighting.supported_effects {
                                let name = via_protocol::VialRgbEffect::from_u16(eid)
                                    .map(|e| e.name())
                                    .unwrap_or("Unknown");
                                let label = format!("{name} ({eid})");
                                if ui
                                    .selectable_label(lighting.effect_id == eid, &label)
                                    .clicked()
                                {
                                    lighting.effect_id = eid;
                                    lighting.dirty = true;
                                }
                            }
                        });
                });
            } else {
                ui.horizontal(|ui| {
                    ui.label("Effect");
                    ui.add_space(8.0);
                    let mut val = lighting.effect_id as f32;
                    if ui
                        .add(egui::Slider::new(&mut val, 0.0..=48.0).integer())
                        .changed()
                    {
                        lighting.effect_id = val as u16;
                        lighting.dirty = true;
                    }
                });
            }
            ui.add_space(8.0);

            // Speed slider
            ui.horizontal(|ui| {
                ui.label("Speed");
                ui.add_space(8.0);
                let mut val = lighting.speed as f32;
                if ui
                    .add(egui::Slider::new(&mut val, 0.0..=255.0).integer())
                    .changed()
                {
                    lighting.speed = val as u8;
                    lighting.dirty = true;
                }
            });
            ui.add_space(8.0);

            // Hue slider
            ui.horizontal(|ui| {
                ui.label("Hue");
                ui.add_space(8.0);
                let mut val = lighting.hue as f32;
                if ui
                    .add(egui::Slider::new(&mut val, 0.0..=255.0).integer())
                    .changed()
                {
                    lighting.hue = val as u8;
                    lighting.dirty = true;
                }
            });
            ui.add_space(8.0);

            // Saturation slider
            ui.horizontal(|ui| {
                ui.label("Saturation");
                ui.add_space(8.0);
                let mut val = lighting.saturation as f32;
                if ui
                    .add(egui::Slider::new(&mut val, 0.0..=255.0).integer())
                    .changed()
                {
                    lighting.saturation = val as u8;
                    lighting.dirty = true;
                }
            });

            ui.add_space(4.0);

            // Color preview swatch
            let (r, g, b) = hsv_to_rgb(lighting.hue, lighting.saturation, lighting.brightness);
            let swatch_size = egui::vec2(max_width, 30.0);
            let (rect, _) = ui.allocate_exact_size(swatch_size, egui::Sense::hover());
            ui.painter().rect_filled(
                rect,
                egui::CornerRadius::same(4),
                egui::Color32::from_rgb(r, g, b),
            );
            ui.painter().rect_stroke(
                rect,
                egui::CornerRadius::same(4),
                egui::Stroke::new(1.0, egui::Color32::from_rgb(80, 80, 90)),
                egui::StrokeKind::Outside,
            );

            ui.add_space(20.0);

            // Buttons
            ui.horizontal(|ui| {
                let apply_enabled = lighting.dirty;
                if ui
                    .add_enabled(apply_enabled, egui::Button::new("Apply"))
                    .clicked()
                {
                    ui.memory_mut(|mem| {
                        mem.data.insert_temp(egui::Id::new("lighting_action"), 1u8);
                    });
                }

                if ui.button("Save to EEPROM").clicked() {
                    ui.memory_mut(|mem| {
                        mem.data.insert_temp(egui::Id::new("lighting_action"), 2u8);
                    });
                }

                if ui.button("Reload").clicked() {
                    ui.memory_mut(|mem| {
                        mem.data.insert_temp(egui::Id::new("lighting_action"), 3u8);
                    });
                }
            });

            if lighting.dirty {
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new("Unsaved changes")
                        .size(11.0)
                        .color(egui::Color32::from_rgb(200, 170, 60)),
                );
            }
        });

        // Handle deferred lighting actions
        let action: Option<u8> =
            ui.memory(|mem| mem.data.get_temp(egui::Id::new("lighting_action")));
        if let Some(action) = action {
            ui.memory_mut(|mem| {
                mem.data.remove::<u8>(egui::Id::new("lighting_action"));
            });
            match action {
                1 => self.apply_lighting(),
                2 => {
                    self.apply_lighting();
                    self.save_lighting();
                }
                3 => self.reload_lighting(),
                _ => {}
            }
        }
    }

    pub fn apply_lighting(&mut self) {
        let Some(lighting) = &self.lighting_data else {
            return;
        };
        let Some(dev) = &self.connected_device else {
            return;
        };
        let proto = ViaProtocol::new(dev);
        let lp = lighting.protocol;
        let vals = via_protocol::LightingValues {
            effect_id: lighting.effect_id,
            brightness: lighting.brightness,
            speed: lighting.speed,
            hue: lighting.hue,
            saturation: lighting.saturation,
        };

        match proto.write_lighting_values(&lp, &vals) {
            Ok(()) => {
                info!("lighting values applied to device");
                if let Some(l) = &mut self.lighting_data {
                    l.dirty = false;
                }
                self.set_status(StatusMessage::info("Lighting applied"));
            }
            Err(e) => {
                let err_str = format!("{e}");
                warn!(error = %e, "failed to apply lighting");
                self.set_status(StatusMessage::error(format!("Lighting error: {e}")));
                if is_disconnect_error(&err_str) {
                    self.handle_disconnect();
                }
            }
        }
    }

    pub fn save_lighting(&mut self) {
        let Some(lighting) = &self.lighting_data else {
            return;
        };
        let lp = lighting.protocol;
        let Some(dev) = &self.connected_device else {
            return;
        };
        let proto = ViaProtocol::new(dev);
        match proto.save_lighting(&lp) {
            Ok(()) => {
                info!("lighting saved to EEPROM");
                self.set_status(StatusMessage::info("Lighting saved to EEPROM"));
            }
            Err(e) => {
                let err_str = format!("{e}");
                warn!(error = %e, "failed to save lighting");
                self.set_status(StatusMessage::error(format!("Save failed: {e}")));
                if is_disconnect_error(&err_str) {
                    self.handle_disconnect();
                }
            }
        }
    }

    pub fn reload_lighting(&mut self) {
        let Some(dev) = &self.connected_device else {
            return;
        };
        let Some(lighting) = &self.lighting_data else {
            return;
        };
        let lp = lighting.protocol;
        let proto = ViaProtocol::new(dev);

        match proto.read_lighting_values(&lp) {
            Ok(vals) => {
                if let Some(l) = &mut self.lighting_data {
                    l.brightness = vals.brightness;
                    l.effect_id = vals.effect_id;
                    l.speed = vals.speed;
                    l.hue = vals.hue;
                    l.saturation = vals.saturation;
                    l.dirty = false;
                }
                info!(
                    brightness = vals.brightness,
                    effect_id = vals.effect_id,
                    speed = vals.speed,
                    hue = vals.hue,
                    sat = vals.saturation,
                    "lighting reloaded"
                );
                self.set_status(StatusMessage::info("Lighting reloaded from device"));
            }
            Err(e) => {
                let err_str = format!("{e}");
                warn!(error = %e, "failed to reload lighting");
                self.set_status(StatusMessage::error(format!("Reload failed: {e}")));
                if is_disconnect_error(&err_str) {
                    self.handle_disconnect();
                }
            }
        }
    }
}
