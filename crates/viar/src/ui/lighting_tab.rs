use eframe::egui;
use tracing::{
    info,
    warn,
};
use via_protocol::{
    LightingProtocol,
    ViaProtocol,
    VialRgbEffect,
};

use crate::{
    types::{
        StatusMessage,
        ViarApp,
    },
    util::{
        hsv_to_rgb,
        is_disconnect_error,
    },
};

/// Effect category for visual grouping.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EffectCategory {
    Off,
    Static,
    Gradient,
    CycleRainbow,
    Reactive,
    Random,
}

impl EffectCategory {
    fn name(self) -> &'static str {
        match self {
            Self::Off => "Off",
            Self::Static => "Static",
            Self::Gradient => "Gradient",
            Self::CycleRainbow => "Cycle / Rainbow",
            Self::Reactive => "Reactive",
            Self::Random => "Random",
        }
    }

    fn icon(self) -> &'static str {
        match self {
            Self::Off => "○",
            Self::Static => "●",
            Self::Gradient => "◐",
            Self::CycleRainbow => "◎",
            Self::Reactive => "⚡",
            Self::Random => "✦",
        }
    }

    fn description(self) -> &'static str {
        match self {
            Self::Off => "No lighting",
            Self::Static => "Solid colors and breathing",
            Self::Gradient => "Color gradients across the keyboard",
            Self::CycleRainbow => "Cycling and rainbow animations",
            Self::Reactive => "Effects triggered by key presses",
            Self::Random => "Random and rain-style effects",
        }
    }
}

fn categorize_effect(id: u16) -> EffectCategory {
    match id {
        0 => EffectCategory::Off,
        1 | 2 | 3 | 6 | 26 => EffectCategory::Static,
        4 | 5 | 7 | 8 | 9 | 10 | 11 | 12 | 27 | 28 => EffectCategory::Gradient,
        13..=23 => EffectCategory::CycleRainbow,
        29 | 31 | 32 | 33 | 34 | 35 | 36 | 37 | 38 | 39 | 40 | 41 | 42 => EffectCategory::Reactive,
        24 | 25 | 30 | 43 | 44 => EffectCategory::Random,
        _ => EffectCategory::Static,
    }
}

/// Whether this effect uses the hue/saturation color setting.
fn effect_uses_color(id: u16) -> bool {
    // Effects that cycle through all hues don't benefit from hue/sat settings.
    // Solid color, alphas/mods, breathing, reactive solids, gradients use color.
    !matches!(
        id,
        0 | 1
            | 13
            | 14
            | 15
            | 16
            | 17
            | 18
            | 19
            | 20
            | 21
            | 22
            | 23
            | 24
            | 25
            | 29
            | 30
            | 39
            | 40
            | 43
            | 44
    )
}

/// Whether this effect uses the speed setting.
fn effect_uses_speed(id: u16) -> bool {
    id != 0 && id != 1 && id != 2
}

/// Paint a hue bar — horizontal gradient showing all hues.
fn paint_hue_bar(ui: &mut egui::Ui, hue: &mut u8, sat: u8, brightness: u8, width: f32) -> bool {
    let height = 24.0;
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::drag());

    let painter = ui.painter();

    // Draw the hue gradient
    let steps = 64;
    let step_w = rect.width() / steps as f32;
    for i in 0..steps {
        let h = (i as f32 / steps as f32 * 255.0) as u8;
        let (r, g, b) = hsv_to_rgb(h, sat, brightness.max(80));
        let x0 = rect.left() + i as f32 * step_w;
        let x1 = rect.left() + (i + 1) as f32 * step_w;
        let seg =
            egui::Rect::from_min_max(egui::pos2(x0, rect.top()), egui::pos2(x1, rect.bottom()));
        painter.rect_filled(seg, 0.0, egui::Color32::from_rgb(r, g, b));
    }

    // Border
    painter.rect_stroke(
        rect,
        egui::CornerRadius::same(3),
        egui::Stroke::new(1.0, egui::Color32::from_rgb(70, 70, 80)),
        egui::StrokeKind::Outside,
    );

    // Indicator
    let indicator_x = rect.left() + (*hue as f32 / 255.0) * rect.width();
    painter.vline(
        indicator_x,
        rect.y_range(),
        egui::Stroke::new(2.0, egui::Color32::WHITE),
    );
    painter.circle_filled(
        egui::pos2(indicator_x, rect.top()),
        5.0,
        egui::Color32::WHITE,
    );

    let mut changed = false;
    if (response.dragged() || response.clicked())
        && let Some(pos) = response.interact_pointer_pos()
    {
        let t = ((pos.x - rect.left()) / rect.width()).clamp(0.0, 1.0);
        let new_hue = (t * 255.0) as u8;
        if new_hue != *hue {
            *hue = new_hue;
            changed = true;
        }
    }

    changed
}

/// Paint a saturation bar — gradient from white/gray to full color.
fn paint_sat_bar(ui: &mut egui::Ui, sat: &mut u8, hue: u8, brightness: u8, width: f32) -> bool {
    let height = 24.0;
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::drag());

    let painter = ui.painter();

    let steps = 32;
    let step_w = rect.width() / steps as f32;
    for i in 0..steps {
        let s = (i as f32 / steps as f32 * 255.0) as u8;
        let (r, g, b) = hsv_to_rgb(hue, s, brightness.max(80));
        let x0 = rect.left() + i as f32 * step_w;
        let x1 = rect.left() + (i + 1) as f32 * step_w;
        let seg =
            egui::Rect::from_min_max(egui::pos2(x0, rect.top()), egui::pos2(x1, rect.bottom()));
        painter.rect_filled(seg, 0.0, egui::Color32::from_rgb(r, g, b));
    }

    painter.rect_stroke(
        rect,
        egui::CornerRadius::same(3),
        egui::Stroke::new(1.0, egui::Color32::from_rgb(70, 70, 80)),
        egui::StrokeKind::Outside,
    );

    let indicator_x = rect.left() + (*sat as f32 / 255.0) * rect.width();
    painter.vline(
        indicator_x,
        rect.y_range(),
        egui::Stroke::new(2.0, egui::Color32::WHITE),
    );
    painter.circle_filled(
        egui::pos2(indicator_x, rect.top()),
        5.0,
        egui::Color32::WHITE,
    );

    let mut changed = false;
    if (response.dragged() || response.clicked())
        && let Some(pos) = response.interact_pointer_pos()
    {
        let t = ((pos.x - rect.left()) / rect.width()).clamp(0.0, 1.0);
        let new_sat = (t * 255.0) as u8;
        if new_sat != *sat {
            *sat = new_sat;
            changed = true;
        }
    }

    changed
}

/// Paint a brightness bar — dark to light.
fn paint_brightness_bar(
    ui: &mut egui::Ui,
    brightness: &mut u8,
    hue: u8,
    sat: u8,
    width: f32,
) -> bool {
    let height = 24.0;
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::drag());

    let painter = ui.painter();

    let steps = 32;
    let step_w = rect.width() / steps as f32;
    for i in 0..steps {
        let v = (i as f32 / steps as f32 * 255.0) as u8;
        let (r, g, b) = hsv_to_rgb(hue, sat, v);
        let x0 = rect.left() + i as f32 * step_w;
        let x1 = rect.left() + (i + 1) as f32 * step_w;
        let seg =
            egui::Rect::from_min_max(egui::pos2(x0, rect.top()), egui::pos2(x1, rect.bottom()));
        painter.rect_filled(seg, 0.0, egui::Color32::from_rgb(r, g, b));
    }

    painter.rect_stroke(
        rect,
        egui::CornerRadius::same(3),
        egui::Stroke::new(1.0, egui::Color32::from_rgb(70, 70, 80)),
        egui::StrokeKind::Outside,
    );

    let indicator_x = rect.left() + (*brightness as f32 / 255.0) * rect.width();
    painter.vline(
        indicator_x,
        rect.y_range(),
        egui::Stroke::new(2.0, egui::Color32::WHITE),
    );
    painter.circle_filled(
        egui::pos2(indicator_x, rect.top()),
        5.0,
        egui::Color32::WHITE,
    );

    let mut changed = false;
    if (response.dragged() || response.clicked())
        && let Some(pos) = response.interact_pointer_pos()
    {
        let t = ((pos.x - rect.left()) / rect.width()).clamp(0.0, 1.0);
        let new_val = (t * 255.0) as u8;
        if new_val != *brightness {
            *brightness = new_val;
            changed = true;
        }
    }

    changed
}

impl ViarApp {
    pub fn render_lighting_tab(&mut self, ui: &mut egui::Ui) {
        let Some(lighting) = &mut self.lighting_data else {
            ui.label("No lighting data available.");
            return;
        };

        let is_vialrgb = matches!(lighting.protocol, LightingProtocol::VialRgb);

        ui.add_space(12.0);

        let max_width = 560.0_f32.min(ui.available_width() - 32.0);
        let bar_width = max_width - 100.0;

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.set_max_width(max_width);

                // ── Color preview swatch (big, prominent) ──
                let (r, g, b) = hsv_to_rgb(lighting.hue, lighting.saturation, lighting.brightness);
                let swatch_size = egui::vec2(max_width, 48.0);
                let (rect, _) = ui.allocate_exact_size(swatch_size, egui::Sense::hover());

                let painter = ui.painter();
                painter.rect_filled(
                    rect,
                    egui::CornerRadius::same(8),
                    egui::Color32::from_rgb(r, g, b),
                );
                painter.rect_stroke(
                    rect,
                    egui::CornerRadius::same(8),
                    egui::Stroke::new(1.0, egui::Color32::from_rgb(80, 80, 90)),
                    egui::StrokeKind::Outside,
                );

                // Overlay text on the swatch
                let text_color = if lighting.brightness > 128 && lighting.saturation < 100 {
                    egui::Color32::from_rgb(20, 20, 20)
                } else {
                    egui::Color32::from_rgb(240, 240, 240)
                };
                let effect_name = if is_vialrgb {
                    VialRgbEffect::from_u16(lighting.effect_id)
                        .map(|e| e.name())
                        .unwrap_or("Unknown")
                } else {
                    "Effect"
                };
                painter.text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    effect_name,
                    egui::FontId::proportional(16.0),
                    text_color,
                );

                ui.add_space(16.0);

                // ── Effect selector ──
                if is_vialrgb && !lighting.supported_effects.is_empty() {
                    // Group effects by category
                    let categories = [
                        EffectCategory::Off,
                        EffectCategory::Static,
                        EffectCategory::Gradient,
                        EffectCategory::CycleRainbow,
                        EffectCategory::Reactive,
                        EffectCategory::Random,
                    ];

                    for cat in &categories {
                        let effects_in_cat: Vec<u16> = lighting
                            .supported_effects
                            .iter()
                            .copied()
                            .filter(|&id| categorize_effect(id) == *cat)
                            .collect();

                        if effects_in_cat.is_empty() {
                            continue;
                        }

                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(format!("{} {}", cat.icon(), cat.name()))
                                    .size(13.0)
                                    .strong()
                                    .color(egui::Color32::from_rgb(170, 170, 190)),
                            );
                            ui.label(
                                egui::RichText::new(cat.description())
                                    .size(11.0)
                                    .color(egui::Color32::from_rgb(110, 110, 125)),
                            );
                        });
                        ui.add_space(2.0);

                        // Effect buttons in a flow layout
                        ui.horizontal_wrapped(|ui| {
                            ui.spacing_mut().item_spacing = egui::vec2(4.0, 4.0);
                            for eid in &effects_in_cat {
                                let name = VialRgbEffect::from_u16(*eid)
                                    .map(|e| e.name())
                                    .unwrap_or("?");
                                let is_active = lighting.effect_id == *eid;

                                let btn = if is_active {
                                    let (r, g, b) =
                                        hsv_to_rgb(lighting.hue, lighting.saturation, 60);
                                    egui::Button::new(
                                        egui::RichText::new(name)
                                            .size(12.0)
                                            .strong()
                                            .color(egui::Color32::WHITE),
                                    )
                                    .fill(egui::Color32::from_rgb(r, g, b))
                                    .corner_radius(egui::CornerRadius::same(12))
                                } else {
                                    egui::Button::new(
                                        egui::RichText::new(name)
                                            .size(12.0)
                                            .color(egui::Color32::from_rgb(180, 180, 195)),
                                    )
                                    .fill(egui::Color32::from_rgb(38, 38, 45))
                                    .corner_radius(egui::CornerRadius::same(12))
                                };

                                if ui.add(btn).clicked() && !is_active {
                                    lighting.effect_id = *eid;
                                    lighting.dirty = true;
                                }
                            }
                        });
                        ui.add_space(4.0);
                    }
                } else {
                    // Non-VialRGB fallback: simple slider
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("Effect")
                                .size(13.0)
                                .color(egui::Color32::from_rgb(160, 160, 175)),
                        );
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

                ui.add_space(16.0);
                ui.separator();
                ui.add_space(12.0);

                // ── Color controls ──
                let show_color = effect_uses_color(lighting.effect_id);
                let show_speed = effect_uses_speed(lighting.effect_id);

                // Brightness
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("Brightness")
                            .size(13.0)
                            .color(egui::Color32::from_rgb(160, 160, 175)),
                    );
                });
                ui.add_space(2.0);
                if paint_brightness_bar(
                    ui,
                    &mut lighting.brightness,
                    lighting.hue,
                    lighting.saturation,
                    bar_width,
                ) {
                    lighting.dirty = true;
                }
                // Percentage label
                ui.horizontal(|ui| {
                    ui.add_space(bar_width - 40.0);
                    ui.label(
                        egui::RichText::new(format!(
                            "{}%",
                            (lighting.brightness as f32 / 255.0 * 100.0).round() as u8
                        ))
                        .size(11.0)
                        .color(egui::Color32::from_rgb(120, 120, 135)),
                    );
                });

                if show_color {
                    ui.add_space(10.0);

                    // Hue
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("Hue")
                                .size(13.0)
                                .color(egui::Color32::from_rgb(160, 160, 175)),
                        );
                    });
                    ui.add_space(2.0);
                    if paint_hue_bar(
                        ui,
                        &mut lighting.hue,
                        lighting.saturation,
                        lighting.brightness,
                        bar_width,
                    ) {
                        lighting.dirty = true;
                    }

                    ui.add_space(10.0);

                    // Saturation
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("Saturation")
                                .size(13.0)
                                .color(egui::Color32::from_rgb(160, 160, 175)),
                        );
                    });
                    ui.add_space(2.0);
                    if paint_sat_bar(
                        ui,
                        &mut lighting.saturation,
                        lighting.hue,
                        lighting.brightness,
                        bar_width,
                    ) {
                        lighting.dirty = true;
                    }
                }

                if show_speed {
                    ui.add_space(10.0);

                    // Speed
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("Speed")
                                .size(13.0)
                                .color(egui::Color32::from_rgb(160, 160, 175)),
                        );
                        ui.add_space(8.0);
                        let mut val = lighting.speed as f32;
                        let speed_label = match lighting.speed {
                            0..=50 => "Very Slow",
                            51..=100 => "Slow",
                            101..=155 => "Medium",
                            156..=210 => "Fast",
                            211..=255 => "Very Fast",
                        };
                        if ui
                            .add(
                                egui::Slider::new(&mut val, 0.0..=255.0)
                                    .integer()
                                    .text(speed_label),
                            )
                            .changed()
                        {
                            lighting.speed = val as u8;
                            lighting.dirty = true;
                        }
                    });
                }

                ui.add_space(20.0);

                // ── Action buttons ──
                ui.horizontal(|ui| {
                    let apply_enabled = lighting.dirty;
                    if ui
                        .add_enabled(
                            apply_enabled,
                            egui::Button::new(egui::RichText::new("⟳ Apply").size(14.0))
                                .corner_radius(egui::CornerRadius::same(6))
                                .min_size(egui::vec2(90.0, 32.0)),
                        )
                        .clicked()
                    {
                        ui.memory_mut(|mem| {
                            mem.data.insert_temp(egui::Id::new("lighting_action"), 1u8);
                        });
                    }

                    ui.add_space(8.0);

                    if ui
                        .add(
                            egui::Button::new(egui::RichText::new("💾 Save").size(14.0))
                                .corner_radius(egui::CornerRadius::same(6))
                                .min_size(egui::vec2(90.0, 32.0)),
                        )
                        .on_hover_text("Apply and save to keyboard EEPROM")
                        .clicked()
                    {
                        ui.memory_mut(|mem| {
                            mem.data.insert_temp(egui::Id::new("lighting_action"), 2u8);
                        });
                    }

                    ui.add_space(8.0);

                    if ui
                        .add(
                            egui::Button::new(egui::RichText::new("↺ Reload").size(14.0))
                                .corner_radius(egui::CornerRadius::same(6))
                                .min_size(egui::vec2(90.0, 32.0)),
                        )
                        .on_hover_text("Reload values from keyboard")
                        .clicked()
                    {
                        ui.memory_mut(|mem| {
                            mem.data.insert_temp(egui::Id::new("lighting_action"), 3u8);
                        });
                    }
                });

                if lighting.dirty {
                    ui.add_space(8.0);
                    ui.label(
                        egui::RichText::new("● Unsaved changes")
                            .size(12.0)
                            .color(egui::Color32::from_rgb(220, 180, 60)),
                    );
                }

                ui.add_space(12.0);

                // Protocol info footer
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
                ui.label(
                    egui::RichText::new(format!("Protocol: {protocol_name}"))
                        .size(11.0)
                        .color(egui::Color32::from_rgb(90, 90, 105)),
                );
            });
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
            effect_id:  lighting.effect_id,
            brightness: lighting.brightness,
            speed:      lighting.speed,
            hue:        lighting.hue,
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
