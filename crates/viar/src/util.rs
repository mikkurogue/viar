use eframe::egui;
use via_protocol::{Keycode, KeycodeCategory};

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

/// Pick a background color based on keycode category.
pub fn key_bg_color(kc: &Keycode) -> egui::Color32 {
    match kc.category() {
        KeycodeCategory::None => egui::Color32::from_rgb(35, 35, 40),
        KeycodeCategory::Transparent => egui::Color32::from_rgb(35, 35, 40),
        KeycodeCategory::Basic => egui::Color32::from_rgb(50, 50, 58),
        KeycodeCategory::Mod => egui::Color32::from_rgb(60, 50, 70),
        KeycodeCategory::LayerTap => egui::Color32::from_rgb(50, 60, 70),
        KeycodeCategory::LayerMod => egui::Color32::from_rgb(48, 62, 60),
        KeycodeCategory::LayerMomentary => egui::Color32::from_rgb(45, 65, 55),
        KeycodeCategory::LayerToggle => egui::Color32::from_rgb(55, 55, 70),
        KeycodeCategory::LayerTapToggle => egui::Color32::from_rgb(52, 58, 68),
        KeycodeCategory::LayerOn => egui::Color32::from_rgb(50, 62, 58),
        KeycodeCategory::LayerDefault => egui::Color32::from_rgb(48, 58, 62),
        KeycodeCategory::LayerOneShotLayer => egui::Color32::from_rgb(55, 52, 65),
        KeycodeCategory::LayerOneShotMod => egui::Color32::from_rgb(58, 50, 65),
        KeycodeCategory::PersistentDefLayer => egui::Color32::from_rgb(50, 55, 60),
        KeycodeCategory::ModTap => egui::Color32::from_rgb(65, 55, 50),
        KeycodeCategory::TapDance => egui::Color32::from_rgb(60, 55, 65),
        _ => egui::Color32::from_rgb(45, 45, 52),
    }
}
