use eframe::egui;
use via_protocol::{
    Keycode,
    KeycodeCategory,
    KeycodeGroup,
};

use crate::theme::Theme;

/// Render a tab-style selectable label with proper text contrast.
/// When selected, uses `text_on_accent` so text is legible against the accent background.
/// When hovered (not selected), uses `text_on_accent` against the accent hover background.
pub fn themed_tab(
    ui: &mut egui::Ui,
    selected: bool,
    label: impl Into<egui::WidgetText>,
    theme: &Theme,
) -> egui::Response {
    if selected {
        // Override text color for selected state to ensure contrast against accent bg
        ui.visuals_mut().override_text_color = Some(theme.text_on_accent());
        let resp = ui.selectable_label(true, label);
        ui.visuals_mut().override_text_color = Some(theme.text_primary());
        resp
    } else {
        ui.selectable_label(false, label)
    }
}

/// Convert QMK-style HSV (hue 0-255, sat 0-255, val 0-255) to RGB.
pub fn hsv_to_rgb(h: u8, s: u8, v: u8) -> (u8, u8, u8) {
    if s == 0 {
        return (v, v, v);
    }
    let h = h as f32 / 255.0 * 360.0;
    let s = s as f32 / 255.0;
    let v = v as f32 / 255.0;
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;
    let (r1, g1, b1) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };
    (
        ((r1 + m) * 255.0) as u8,
        ((g1 + m) * 255.0) as u8,
        ((b1 + m) * 255.0) as u8,
    )
}

/// Check if an error string indicates a device disconnect.
pub fn is_disconnect_error(err: &str) -> bool {
    let lower = err.to_lowercase();
    lower.contains("no device")
        || lower.contains("disconnected")
        || lower.contains("i/o error")
        || lower.contains("device not found")
        || lower.contains("broken pipe")
}

pub trait CategoryStyle {
    fn bg(&self) -> egui::Color32;
}

impl CategoryStyle for KeycodeCategory {
    fn bg(&self) -> egui::Color32 {
        match self {
            Self::None => egui::Color32::from_rgb(35, 35, 40),
            Self::Transparent => egui::Color32::from_rgb(35, 35, 40),
            Self::Basic => egui::Color32::from_rgb(50, 50, 58),
            Self::Mouse => egui::Color32::from_rgb(55, 50, 65),
            Self::Mod => egui::Color32::from_rgb(60, 50, 70),
            Self::LayerTap => egui::Color32::from_rgb(50, 60, 70),
            Self::LayerMod => egui::Color32::from_rgb(48, 62, 60),
            Self::LayerMomentary => egui::Color32::from_rgb(45, 65, 55),
            Self::LayerToggle => egui::Color32::from_rgb(55, 55, 70),
            Self::LayerTapToggle => egui::Color32::from_rgb(52, 58, 68),
            Self::LayerOn => egui::Color32::from_rgb(50, 62, 58),
            Self::LayerDefault => egui::Color32::from_rgb(48, 58, 62),
            Self::LayerOneShotLayer => egui::Color32::from_rgb(55, 52, 65),
            Self::LayerOneShotMod => egui::Color32::from_rgb(58, 50, 65),
            Self::PersistentDefLayer => egui::Color32::from_rgb(50, 55, 60),
            Self::ModTap => egui::Color32::from_rgb(65, 55, 50),
            Self::TapDance => egui::Color32::from_rgb(60, 55, 65),
            Self::SwapHands => egui::Color32::from_rgb(55, 58, 50),
            Self::Magic => egui::Color32::from_rgb(60, 50, 55),
            Self::Lighting => egui::Color32::from_rgb(50, 55, 65),
            Self::Quantum => egui::Color32::from_rgb(55, 48, 60),
            _ => egui::Color32::from_rgb(45, 45, 52),
        }
    }
}

/// Render a clickable keycode chip. Returns true if clicked (to select this field).
/// `is_active` indicates this field is currently selected for the shared picker.
pub fn keycode_chip(ui: &mut egui::Ui, label: &str, value: u16, is_active: bool) -> bool {
    let kc = Keycode(value);
    let name = if value == 0 {
        "---".to_string()
    } else {
        kc.name()
    };

    let mut clicked = false;

    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(format!("{label}:"))
                .size(12.0)
                .color(if is_active {
                    egui::Color32::from_rgb(100, 180, 255)
                } else {
                    egui::Color32::from_rgb(140, 140, 155)
                }),
        );

        let bg = if is_active {
            egui::Color32::from_rgb(50, 80, 120)
        } else if value == 0 {
            egui::Color32::from_rgb(40, 40, 45)
        } else {
            kc.category().bg()
        };

        let border = if is_active {
            egui::Stroke::new(2.0_f32, egui::Color32::from_rgb(100, 180, 255))
        } else {
            egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(60, 60, 65))
        };

        let btn = ui.add(
            egui::Button::new(egui::RichText::new(&name).monospace().size(12.0).color(
                if value == 0 {
                    egui::Color32::from_rgb(90, 90, 100)
                } else {
                    egui::Color32::from_rgb(220, 220, 230)
                },
            ))
            .fill(bg)
            .stroke(border)
            .corner_radius(egui::CornerRadius::same(4))
            .min_size(egui::vec2(60.0, 24.0)),
        );

        if btn.clicked() {
            clicked = true;
        }

        if value != 0 {
            btn.on_hover_text(format!("0x{:04X} — {}", value, kc.description()));
        } else {
            btn.on_hover_text("Click to select, then pick a key below");
        }
    });

    clicked
}

/// Result from the shared keycode picker.
pub struct PickerResult {
    pub selected: Option<u16>,
    pub cleared:  bool,
}

/// Render the shared keycode picker grid at the bottom of an editor panel.
/// `current_value` is the value of the currently active field.
/// `selected_group` / `groups` control the tab state.
/// Returns whether a key was picked and the new value.
pub fn shared_keycode_picker(
    ui: &mut egui::Ui,
    current_value: u16,
    selected_group: &mut usize,
    groups: &[KeycodeGroup],
    active_field_label: &str,
    theme: &Theme,
) -> PickerResult {
    let mut result = PickerResult {
        selected: None,
        cleared:  false,
    };

    // Header with active field indicator + clear button
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(format!("Setting: {active_field_label}"))
                .size(11.0)
                .strong()
                .color(egui::Color32::from_rgb(100, 180, 255)),
        );

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if current_value != 0
                && ui
                    .add(
                        egui::Button::new(
                            egui::RichText::new("Clear")
                                .size(10.0)
                                .color(egui::Color32::from_rgb(180, 100, 100)),
                        )
                        .fill(egui::Color32::from_rgb(50, 35, 35))
                        .corner_radius(egui::CornerRadius::same(3)),
                    )
                    .clicked()
            {
                result.cleared = true;
            }

            // Hex input
            let hex_id = ui.id().with("shared_hex");
            let mut hex: String = ui
                .memory(|mem| mem.data.get_temp(hex_id))
                .unwrap_or_else(|| format!("{:04X}", current_value));
            let resp = ui.add(
                egui::TextEdit::singleline(&mut hex)
                    .desired_width(45.0)
                    .font(egui::TextStyle::Monospace),
            );
            ui.memory_mut(|mem| mem.data.insert_temp(hex_id, hex.clone()));
            if resp.lost_focus()
                && ui.input(|i| i.key_pressed(egui::Key::Enter))
                && let Ok(v) = u16::from_str_radix(hex.trim(), 16)
            {
                result.selected = Some(v);
            }
            ui.label(
                egui::RichText::new("Hex:")
                    .size(10.0)
                    .color(egui::Color32::from_rgb(100, 100, 115)),
            );
        });
    });

    ui.add_space(2.0);

    // Group tabs
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(2.0, 2.0);
        for (i, group) in groups.iter().enumerate() {
            let sel = *selected_group == i;
            let label_text = egui::RichText::new(group.name).size(9.5);
            if themed_tab(ui, sel, label_text, theme).clicked() {
                *selected_group = i;
            }
        }
    });

    // Keycode buttons grid
    egui::ScrollArea::vertical()
        .max_height(180.0)
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(3.0, 3.0);
                if let Some(group) = groups.get(*selected_group) {
                    for kc in &group.codes {
                        let kc_name = kc.name();
                        let is_current = kc.0 == current_value;
                        let size = egui::vec2(38.0, 22.0);
                        let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());
                        let is_hov = response.hovered();

                        let bg = if is_current {
                            egui::Color32::from_rgb(70, 130, 180)
                        } else if is_hov {
                            egui::Color32::from_rgb(70, 70, 80)
                        } else {
                            egui::Color32::from_rgb(42, 42, 48)
                        };
                        let text_col = if is_current {
                            egui::Color32::WHITE
                        } else {
                            egui::Color32::from_rgb(190, 190, 200)
                        };

                        let rounding = egui::CornerRadius::same(3);
                        ui.painter().rect_filled(rect, rounding, bg);
                        if is_current {
                            ui.painter().rect_stroke(
                                rect,
                                rounding,
                                egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(100, 180, 255)),
                                egui::StrokeKind::Outside,
                            );
                        }

                        let font_size = if kc_name.len() <= 2 {
                            10.0
                        } else if kc_name.len() <= 5 {
                            9.0
                        } else {
                            7.5
                        };
                        ui.painter().text(
                            rect.center(),
                            egui::Align2::CENTER_CENTER,
                            &kc_name,
                            egui::FontId::proportional(font_size),
                            text_col,
                        );

                        if response.clicked() {
                            result.selected = Some(kc.0);
                        }
                        response.on_hover_text(kc.description());
                    }
                }
            });
        });

    result
}
