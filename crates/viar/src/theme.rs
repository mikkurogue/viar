use std::path::PathBuf;

use eframe::egui;
use serde::{
    Deserialize,
    Serialize,
};
use tracing::{
    info,
    warn,
};

fn hex_to_color32(hex: &str) -> egui::Color32 {
    let hex = hex.trim_start_matches('#');
    if hex.len() == 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
        egui::Color32::from_rgb(r, g, b)
    } else {
        egui::Color32::from_rgb(128, 128, 128)
    }
}

/// All the semantic colors used throughout the application.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeColors {
    // Backgrounds
    pub bg_primary:   String,
    pub bg_secondary: String,
    pub bg_tertiary:  String,
    pub bg_elevated:  String,

    // Text
    pub text_primary:   String,
    pub text_secondary: String,
    pub text_muted:     String,
    pub text_on_accent: String,

    // Accents
    pub accent:        String,
    pub accent_hover:  String,
    pub accent_subtle: String,

    // Borders
    pub border:        String,
    pub border_active: String,

    // Status
    pub success: String,
    pub error:   String,
    pub warning: String,

    // Key categories (for keymap rendering)
    pub key_basic:     String,
    pub key_mod:       String,
    pub key_layer:     String,
    pub key_mod_tap:   String,
    pub key_tap_dance: String,
    pub key_empty:     String,
    pub key_selected:  String,
    pub key_hover:     String,
}

/// A resolved theme with actual egui colors.
#[derive(Debug, Clone)]
pub struct Theme {
    pub name:   String,
    pub colors: ThemeColors,
}

#[allow(dead_code)]
impl Theme {
    /// Resolve a hex color string to an egui Color32.
    pub fn color(&self, hex: &str) -> egui::Color32 {
        hex_to_color32(hex)
    }

    // Convenience accessors
    pub fn bg_primary(&self) -> egui::Color32 {
        hex_to_color32(&self.colors.bg_primary)
    }
    pub fn bg_secondary(&self) -> egui::Color32 {
        hex_to_color32(&self.colors.bg_secondary)
    }
    pub fn bg_tertiary(&self) -> egui::Color32 {
        hex_to_color32(&self.colors.bg_tertiary)
    }
    pub fn bg_elevated(&self) -> egui::Color32 {
        hex_to_color32(&self.colors.bg_elevated)
    }
    pub fn text_primary(&self) -> egui::Color32 {
        hex_to_color32(&self.colors.text_primary)
    }
    pub fn text_secondary(&self) -> egui::Color32 {
        hex_to_color32(&self.colors.text_secondary)
    }
    pub fn text_muted(&self) -> egui::Color32 {
        hex_to_color32(&self.colors.text_muted)
    }
    pub fn text_on_accent(&self) -> egui::Color32 {
        hex_to_color32(&self.colors.text_on_accent)
    }
    pub fn accent(&self) -> egui::Color32 {
        hex_to_color32(&self.colors.accent)
    }
    pub fn accent_hover(&self) -> egui::Color32 {
        hex_to_color32(&self.colors.accent_hover)
    }
    pub fn accent_subtle(&self) -> egui::Color32 {
        hex_to_color32(&self.colors.accent_subtle)
    }
    pub fn border(&self) -> egui::Color32 {
        hex_to_color32(&self.colors.border)
    }
    pub fn border_active(&self) -> egui::Color32 {
        hex_to_color32(&self.colors.border_active)
    }
    pub fn success(&self) -> egui::Color32 {
        hex_to_color32(&self.colors.success)
    }
    pub fn error(&self) -> egui::Color32 {
        hex_to_color32(&self.colors.error)
    }
    pub fn warning(&self) -> egui::Color32 {
        hex_to_color32(&self.colors.warning)
    }
    pub fn key_basic(&self) -> egui::Color32 {
        hex_to_color32(&self.colors.key_basic)
    }
    pub fn key_mod(&self) -> egui::Color32 {
        hex_to_color32(&self.colors.key_mod)
    }
    pub fn key_layer(&self) -> egui::Color32 {
        hex_to_color32(&self.colors.key_layer)
    }
    pub fn key_mod_tap(&self) -> egui::Color32 {
        hex_to_color32(&self.colors.key_mod_tap)
    }
    pub fn key_tap_dance(&self) -> egui::Color32 {
        hex_to_color32(&self.colors.key_tap_dance)
    }
    pub fn key_empty(&self) -> egui::Color32 {
        hex_to_color32(&self.colors.key_empty)
    }
    pub fn key_selected(&self) -> egui::Color32 {
        hex_to_color32(&self.colors.key_selected)
    }
    pub fn key_hover(&self) -> egui::Color32 {
        hex_to_color32(&self.colors.key_hover)
    }

    /// Apply this theme to the egui context's visuals.
    pub fn apply(&self, ctx: &egui::Context) {
        let mut visuals = egui::Visuals::dark();

        visuals.panel_fill = self.bg_primary();
        visuals.window_fill = self.bg_elevated();
        visuals.faint_bg_color = self.bg_secondary();
        visuals.extreme_bg_color = self.bg_tertiary();

        visuals.override_text_color = Some(self.text_primary());
        visuals.selection.bg_fill = self.accent();
        visuals.selection.stroke = egui::Stroke::new(1.0_f32, self.accent());

        visuals.widgets.noninteractive.bg_fill = self.bg_secondary();
        visuals.widgets.noninteractive.fg_stroke =
            egui::Stroke::new(1.0_f32, self.text_secondary());
        visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0_f32, self.border());

        visuals.widgets.inactive.bg_fill = self.bg_tertiary();
        visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0_f32, self.text_secondary());
        visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0_f32, self.border());

        visuals.widgets.hovered.bg_fill = self.accent_subtle();
        visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0_f32, self.text_primary());
        visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0_f32, self.accent_hover());

        visuals.widgets.active.bg_fill = self.accent();
        visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0_f32, self.text_on_accent());
        visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0_f32, self.accent());

        ctx.set_visuals(visuals);
    }
}

pub fn builtin_themes() -> Vec<Theme> {
    vec![
        catppuccin_mocha(),
        catppuccin_macchiato(),
        catppuccin_frappe(),
        catppuccin_latte(),
        onedark_dark(),
        onedark_light(),
        rosepine_main(),
        rosepine_moon(),
        rosepine_dawn(),
        gruvbox_dark(),
        gruvbox_light(),
    ]
}

fn catppuccin_mocha() -> Theme {
    Theme {
        name:   "Catppuccin Mocha".into(),
        colors: ThemeColors {
            bg_primary:     "#1E1E2E".into(),
            bg_secondary:   "#313244".into(),
            bg_tertiary:    "#45475A".into(),
            bg_elevated:    "#181825".into(),
            text_primary:   "#CDD6F4".into(),
            text_secondary: "#BAC2DE".into(),
            text_muted:     "#6C7086".into(),
            text_on_accent: "#1E1E2E".into(),
            accent:         "#89B4FA".into(),
            accent_hover:   "#B4D0FB".into(),
            accent_subtle:  "#2A2B3D".into(),
            border:         "#45475A".into(),
            border_active:  "#89B4FA".into(),
            success:        "#A6E3A1".into(),
            error:          "#F38BA8".into(),
            warning:        "#F9E2AF".into(),
            key_basic:      "#313244".into(),
            key_mod:        "#45475A".into(),
            key_layer:      "#2A3040".into(),
            key_mod_tap:    "#3B3040".into(),
            key_tap_dance:  "#302D40".into(),
            key_empty:      "#1E1E2E".into(),
            key_selected:   "#89B4FA".into(),
            key_hover:      "#45475A".into(),
        },
    }
}

fn catppuccin_macchiato() -> Theme {
    Theme {
        name:   "Catppuccin Macchiato".into(),
        colors: ThemeColors {
            bg_primary:     "#24273A".into(),
            bg_secondary:   "#363A4F".into(),
            bg_tertiary:    "#494D64".into(),
            bg_elevated:    "#1E2030".into(),
            text_primary:   "#CAD3F5".into(),
            text_secondary: "#B8C0E0".into(),
            text_muted:     "#6E738D".into(),
            text_on_accent: "#24273A".into(),
            accent:         "#8AADF4".into(),
            accent_hover:   "#B4CDFA".into(),
            accent_subtle:  "#2E3148".into(),
            border:         "#494D64".into(),
            border_active:  "#8AADF4".into(),
            success:        "#A6DA95".into(),
            error:          "#ED8796".into(),
            warning:        "#EED49F".into(),
            key_basic:      "#363A4F".into(),
            key_mod:        "#494D64".into(),
            key_layer:      "#2E3548".into(),
            key_mod_tap:    "#3E3448".into(),
            key_tap_dance:  "#343148".into(),
            key_empty:      "#24273A".into(),
            key_selected:   "#8AADF4".into(),
            key_hover:      "#494D64".into(),
        },
    }
}

fn catppuccin_frappe() -> Theme {
    Theme {
        name:   "Catppuccin Frappe".into(),
        colors: ThemeColors {
            bg_primary:     "#303446".into(),
            bg_secondary:   "#414559".into(),
            bg_tertiary:    "#51576D".into(),
            bg_elevated:    "#292C3C".into(),
            text_primary:   "#C6D0F5".into(),
            text_secondary: "#B5BFE2".into(),
            text_muted:     "#737994".into(),
            text_on_accent: "#303446".into(),
            accent:         "#8CAAEE".into(),
            accent_hover:   "#B4C8F4".into(),
            accent_subtle:  "#383C50".into(),
            border:         "#51576D".into(),
            border_active:  "#8CAAEE".into(),
            success:        "#A6D189".into(),
            error:          "#E78284".into(),
            warning:        "#E5C890".into(),
            key_basic:      "#414559".into(),
            key_mod:        "#51576D".into(),
            key_layer:      "#384050".into(),
            key_mod_tap:    "#483E50".into(),
            key_tap_dance:  "#3E3B50".into(),
            key_empty:      "#303446".into(),
            key_selected:   "#8CAAEE".into(),
            key_hover:      "#51576D".into(),
        },
    }
}

fn catppuccin_latte() -> Theme {
    Theme {
        name:   "Catppuccin Latte".into(),
        colors: ThemeColors {
            bg_primary:     "#EFF1F5".into(),
            bg_secondary:   "#E6E9EF".into(),
            bg_tertiary:    "#DCE0E8".into(),
            bg_elevated:    "#CCD0DA".into(),
            text_primary:   "#4C4F69".into(),
            text_secondary: "#5C5F77".into(),
            text_muted:     "#9CA0B0".into(),
            text_on_accent: "#EFF1F5".into(),
            accent:         "#1E66F5".into(),
            accent_hover:   "#4B80F7".into(),
            accent_subtle:  "#D5DAE8".into(),
            border:         "#BCC0CC".into(),
            border_active:  "#1E66F5".into(),
            success:        "#40A02B".into(),
            error:          "#D20F39".into(),
            warning:        "#DF8E1D".into(),
            key_basic:      "#E6E9EF".into(),
            key_mod:        "#DCE0E8".into(),
            key_layer:      "#D5E0EF".into(),
            key_mod_tap:    "#E0D5E8".into(),
            key_tap_dance:  "#DDD5E8".into(),
            key_empty:      "#EFF1F5".into(),
            key_selected:   "#1E66F5".into(),
            key_hover:      "#CCD0DA".into(),
        },
    }
}

fn onedark_dark() -> Theme {
    Theme {
        name:   "One Dark".into(),
        colors: ThemeColors {
            bg_primary:     "#282C34".into(),
            bg_secondary:   "#2C313A".into(),
            bg_tertiary:    "#3E4452".into(),
            bg_elevated:    "#21252B".into(),
            text_primary:   "#ABB2BF".into(),
            text_secondary: "#9DA5B4".into(),
            text_muted:     "#5C6370".into(),
            text_on_accent: "#282C34".into(),
            accent:         "#61AFEF".into(),
            accent_hover:   "#8BC5F5".into(),
            accent_subtle:  "#2E3440".into(),
            border:         "#3E4452".into(),
            border_active:  "#61AFEF".into(),
            success:        "#98C379".into(),
            error:          "#E06C75".into(),
            warning:        "#E5C07B".into(),
            key_basic:      "#2C313A".into(),
            key_mod:        "#3E4452".into(),
            key_layer:      "#2A3540".into(),
            key_mod_tap:    "#3A2E40".into(),
            key_tap_dance:  "#342E40".into(),
            key_empty:      "#21252B".into(),
            key_selected:   "#61AFEF".into(),
            key_hover:      "#3E4452".into(),
        },
    }
}

fn onedark_light() -> Theme {
    Theme {
        name:   "One Light".into(),
        colors: ThemeColors {
            bg_primary:     "#FAFAFA".into(),
            bg_secondary:   "#F0F0F0".into(),
            bg_tertiary:    "#E5E5E6".into(),
            bg_elevated:    "#DBDBDC".into(),
            text_primary:   "#383A42".into(),
            text_secondary: "#4B4D55".into(),
            text_muted:     "#A0A1A7".into(),
            text_on_accent: "#FAFAFA".into(),
            accent:         "#4078F2".into(),
            accent_hover:   "#6690F5".into(),
            accent_subtle:  "#E8ECF5".into(),
            border:         "#D3D3D4".into(),
            border_active:  "#4078F2".into(),
            success:        "#50A14F".into(),
            error:          "#E45649".into(),
            warning:        "#C18401".into(),
            key_basic:      "#F0F0F0".into(),
            key_mod:        "#E5E5E6".into(),
            key_layer:      "#E0E5F0".into(),
            key_mod_tap:    "#EBE0F0".into(),
            key_tap_dance:  "#E8E0F0".into(),
            key_empty:      "#FAFAFA".into(),
            key_selected:   "#4078F2".into(),
            key_hover:      "#DBDBDC".into(),
        },
    }
}

fn rosepine_main() -> Theme {
    Theme {
        name:   "Rose Pine".into(),
        colors: ThemeColors {
            bg_primary:     "#191724".into(),
            bg_secondary:   "#1F1D2E".into(),
            bg_tertiary:    "#26233A".into(),
            bg_elevated:    "#2A2738".into(),
            text_primary:   "#E0DEF4".into(),
            text_secondary: "#908CAA".into(),
            text_muted:     "#6E6A86".into(),
            text_on_accent: "#191724".into(),
            accent:         "#C4A7E7".into(),
            accent_hover:   "#D4BFF0".into(),
            accent_subtle:  "#251F38".into(),
            border:         "#26233A".into(),
            border_active:  "#C4A7E7".into(),
            success:        "#9CCFD8".into(),
            error:          "#EB6F92".into(),
            warning:        "#F6C177".into(),
            key_basic:      "#1F1D2E".into(),
            key_mod:        "#26233A".into(),
            key_layer:      "#1F2530".into(),
            key_mod_tap:    "#2A1F38".into(),
            key_tap_dance:  "#261F38".into(),
            key_empty:      "#191724".into(),
            key_selected:   "#C4A7E7".into(),
            key_hover:      "#26233A".into(),
        },
    }
}

fn rosepine_moon() -> Theme {
    Theme {
        name:   "Rose Pine Moon".into(),
        colors: ThemeColors {
            bg_primary:     "#232136".into(),
            bg_secondary:   "#2A273F".into(),
            bg_tertiary:    "#393552".into(),
            bg_elevated:    "#2A2740".into(),
            text_primary:   "#E0DEF4".into(),
            text_secondary: "#908CAA".into(),
            text_muted:     "#6E6A86".into(),
            text_on_accent: "#232136".into(),
            accent:         "#C4A7E7".into(),
            accent_hover:   "#D4BFF0".into(),
            accent_subtle:  "#2E2948".into(),
            border:         "#393552".into(),
            border_active:  "#C4A7E7".into(),
            success:        "#9CCFD8".into(),
            error:          "#EB6F92".into(),
            warning:        "#F6C177".into(),
            key_basic:      "#2A273F".into(),
            key_mod:        "#393552".into(),
            key_layer:      "#282F40".into(),
            key_mod_tap:    "#322848".into(),
            key_tap_dance:  "#2E2848".into(),
            key_empty:      "#232136".into(),
            key_selected:   "#C4A7E7".into(),
            key_hover:      "#393552".into(),
        },
    }
}

fn rosepine_dawn() -> Theme {
    Theme {
        name:   "Rose Pine Dawn".into(),
        colors: ThemeColors {
            bg_primary:     "#FAF4ED".into(),
            bg_secondary:   "#FFFAF3".into(),
            bg_tertiary:    "#F2E9E1".into(),
            bg_elevated:    "#E6DFD8".into(),
            text_primary:   "#575279".into(),
            text_secondary: "#797593".into(),
            text_muted:     "#9893A5".into(),
            text_on_accent: "#FAF4ED".into(),
            accent:         "#907AA9".into(),
            accent_hover:   "#A68EC0".into(),
            accent_subtle:  "#EDE5DF".into(),
            border:         "#DFDAD9".into(),
            border_active:  "#907AA9".into(),
            success:        "#56949F".into(),
            error:          "#B4637A".into(),
            warning:        "#EA9D34".into(),
            key_basic:      "#F2E9E1".into(),
            key_mod:        "#E6DFD8".into(),
            key_layer:      "#E0E5E8".into(),
            key_mod_tap:    "#EBE0EB".into(),
            key_tap_dance:  "#E8E0EB".into(),
            key_empty:      "#FAF4ED".into(),
            key_selected:   "#907AA9".into(),
            key_hover:      "#E6DFD8".into(),
        },
    }
}

fn gruvbox_dark() -> Theme {
    Theme {
        name:   "Gruvbox Dark".into(),
        colors: ThemeColors {
            bg_primary:     "#282828".into(),
            bg_secondary:   "#3C3836".into(),
            bg_tertiary:    "#504945".into(),
            bg_elevated:    "#1D2021".into(),
            text_primary:   "#EBDBB2".into(),
            text_secondary: "#D5C4A1".into(),
            text_muted:     "#928374".into(),
            text_on_accent: "#282828".into(),
            accent:         "#83A598".into(),
            accent_hover:   "#A8C4B8".into(),
            accent_subtle:  "#2E3530".into(),
            border:         "#504945".into(),
            border_active:  "#83A598".into(),
            success:        "#B8BB26".into(),
            error:          "#FB4934".into(),
            warning:        "#FABD2F".into(),
            key_basic:      "#3C3836".into(),
            key_mod:        "#504945".into(),
            key_layer:      "#2E3830".into(),
            key_mod_tap:    "#452E30".into(),
            key_tap_dance:  "#3E2E40".into(),
            key_empty:      "#282828".into(),
            key_selected:   "#83A598".into(),
            key_hover:      "#504945".into(),
        },
    }
}

fn gruvbox_light() -> Theme {
    Theme {
        name:   "Gruvbox Light".into(),
        colors: ThemeColors {
            bg_primary:     "#FBF1C7".into(),
            bg_secondary:   "#EBDBB2".into(),
            bg_tertiary:    "#D5C4A1".into(),
            bg_elevated:    "#BDAE93".into(),
            text_primary:   "#3C3836".into(),
            text_secondary: "#504945".into(),
            text_muted:     "#928374".into(),
            text_on_accent: "#FBF1C7".into(),
            accent:         "#427B58".into(),
            accent_hover:   "#5C9A78".into(),
            accent_subtle:  "#E8DEB8".into(),
            border:         "#BDAE93".into(),
            border_active:  "#427B58".into(),
            success:        "#79740E".into(),
            error:          "#9D0006".into(),
            warning:        "#B57614".into(),
            key_basic:      "#EBDBB2".into(),
            key_mod:        "#D5C4A1".into(),
            key_layer:      "#D0D8C0".into(),
            key_mod_tap:    "#DBC8D0".into(),
            key_tap_dance:  "#D8C8D0".into(),
            key_empty:      "#FBF1C7".into(),
            key_selected:   "#427B58".into(),
            key_hover:      "#BDAE93".into(),
        },
    }
}

/// Persistent configuration saved to ~/.config/viar/config.toml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViarConfig {
    /// Name of the active theme (built-in or custom)
    #[serde(default = "default_theme_name")]
    pub theme: String,
}

fn default_theme_name() -> String {
    "Catppuccin Mocha".into()
}

impl Default for ViarConfig {
    fn default() -> Self {
        Self {
            theme: default_theme_name(),
        }
    }
}

/// Get the config directory path: ~/.config/viar/
pub fn config_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("viar"))
}

/// Get the custom themes directory: ~/.config/viar/themes/
pub fn themes_dir() -> Option<PathBuf> {
    config_dir().map(|d| d.join("themes"))
}

/// Load config from disk, returning default if not found.
pub fn load_config() -> ViarConfig {
    let Some(dir) = config_dir() else {
        return ViarConfig::default();
    };
    let path = dir.join("config.toml");
    match std::fs::read_to_string(&path) {
        Ok(content) => match toml::from_str(&content) {
            Ok(config) => {
                info!(?path, "loaded config");
                config
            }
            Err(e) => {
                warn!(?path, error = %e, "failed to parse config, using defaults");
                ViarConfig::default()
            }
        },
        Err(_) => ViarConfig::default(),
    }
}

/// Save config to disk.
pub fn save_config(config: &ViarConfig) {
    let Some(dir) = config_dir() else {
        warn!("could not determine config directory");
        return;
    };
    if let Err(e) = std::fs::create_dir_all(&dir) {
        warn!(error = %e, "failed to create config directory");
        return;
    }
    let path = dir.join("config.toml");
    match toml::to_string_pretty(config) {
        Ok(content) => {
            if let Err(e) = std::fs::write(&path, content) {
                warn!(?path, error = %e, "failed to write config");
            } else {
                info!(?path, "config saved");
            }
        }
        Err(e) => {
            warn!(error = %e, "failed to serialize config");
        }
    }
}

/// Load custom themes from ~/.config/viar/themes/*.json
pub fn load_custom_themes() -> Vec<Theme> {
    let Some(dir) = themes_dir() else {
        return Vec::new();
    };
    let entries = match std::fs::read_dir(&dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };
    let mut themes = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "json") {
            match std::fs::read_to_string(&path) {
                Ok(content) => match serde_json::from_str::<ThemeColors>(&content) {
                    Ok(colors) => {
                        let name = path
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("Custom")
                            .to_string();
                        info!(?path, name, "loaded custom theme");
                        themes.push(Theme { name, colors });
                    }
                    Err(e) => {
                        warn!(?path, error = %e, "failed to parse custom theme");
                    }
                },
                Err(e) => {
                    warn!(?path, error = %e, "failed to read custom theme file");
                }
            }
        }
    }
    themes
}

/// Resolve a theme by name from built-in + custom themes.
pub fn resolve_theme(name: &str) -> Theme {
    let builtins = builtin_themes();
    if let Some(t) = builtins.iter().find(|t| t.name == name) {
        return t.clone();
    }
    let customs = load_custom_themes();
    if let Some(t) = customs.iter().find(|t| t.name == name) {
        return t.clone();
    }
    // Fallback to default
    catppuccin_mocha()
}

/// Get all available themes (built-in + custom).
pub fn all_themes() -> Vec<Theme> {
    let mut themes = builtin_themes();
    themes.extend(load_custom_themes());
    themes
}

/// Export the default theme as a JSON template for users to customize.
pub fn export_theme_template() -> String {
    let theme = catppuccin_mocha();
    serde_json::to_string_pretty(&theme.colors).unwrap_or_default()
}
