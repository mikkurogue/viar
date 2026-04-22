use eframe::egui;

use crate::{
    theme::{
        all_themes,
        config_dir,
        export_theme_template,
        resolve_theme,
        save_config,
        themes_dir,
    },
    types::{
        StatusMessage,
        ViarApp,
    },
};

impl ViarApp {
    pub fn render_settings_tab(&mut self, ui: &mut egui::Ui) {
        ui.add_space(12.0);

        let max_width = 600.0_f32.min(ui.available_width() - 40.0);

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.set_max_width(max_width);

                ui.heading("Settings");
                ui.add_space(16.0);

                // Theme section
                egui::Frame::default()
                    .inner_margin(egui::Margin::same(16))
                    .corner_radius(egui::CornerRadius::same(6))
                    .fill(self.theme.bg_secondary())
                    .stroke(egui::Stroke::new(1.0, self.theme.border()))
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new("Theme")
                                .size(16.0)
                                .strong()
                                .color(self.theme.text_primary()),
                        );
                        ui.add_space(4.0);
                        ui.label(
                            egui::RichText::new(format!("Current: {}", self.config.theme))
                                .size(12.0)
                                .color(self.theme.text_secondary()),
                        );
                        ui.add_space(12.0);

                        let themes = all_themes();

                        // Group themes by family
                        let families = [
                            (
                                "Catppuccin",
                                vec![
                                    "Catppuccin Mocha",
                                    "Catppuccin Macchiato",
                                    "Catppuccin Frappe",
                                    "Catppuccin Latte",
                                ],
                            ),
                            ("One Dark", vec!["One Dark", "One Light"]),
                            (
                                "Rose Pine",
                                vec!["Rose Pine", "Rose Pine Moon", "Rose Pine Dawn"],
                            ),
                            ("Gruvbox", vec!["Gruvbox Dark", "Gruvbox Light"]),
                        ];

                        for (family_name, members) in &families {
                            ui.add_space(4.0);
                            ui.label(
                                egui::RichText::new(*family_name)
                                    .size(12.0)
                                    .strong()
                                    .color(self.theme.text_secondary()),
                            );
                            ui.horizontal_wrapped(|ui| {
                                for name in members {
                                    let is_active = self.config.theme == *name;
                                    // Find theme to preview its colors
                                    let preview_theme = themes.iter().find(|t| t.name == *name);

                                    let (bg, text_col, border) = if let Some(pt) = preview_theme {
                                        if is_active {
                                            (pt.accent(), pt.text_on_accent(), pt.accent())
                                        } else {
                                            (pt.bg_primary(), pt.text_primary(), pt.border())
                                        }
                                    } else {
                                        (
                                            self.theme.bg_tertiary(),
                                            self.theme.text_primary(),
                                            self.theme.border(),
                                        )
                                    };

                                    let btn = ui.add(
                                        egui::Button::new(
                                            egui::RichText::new(*name).size(11.0).color(text_col),
                                        )
                                        .fill(bg)
                                        .stroke(egui::Stroke::new(
                                            if is_active { 2.0 } else { 1.0 },
                                            border,
                                        ))
                                        .corner_radius(egui::CornerRadius::same(4))
                                        .min_size(egui::vec2(100.0, 28.0)),
                                    );

                                    if btn.clicked() && !is_active {
                                        self.config.theme = name.to_string();
                                        self.theme = resolve_theme(&self.config.theme);
                                        save_config(&self.config);
                                        self.set_status(StatusMessage::info(format!(
                                            "Theme set to {name}"
                                        )));
                                    }
                                }
                            });
                        }

                        // Custom themes section
                        let custom_themes: Vec<&crate::theme::Theme> = themes
                            .iter()
                            .filter(|t| {
                                !families
                                    .iter()
                                    .any(|(_, members)| members.iter().any(|m| *m == t.name))
                            })
                            .collect();

                        if !custom_themes.is_empty() {
                            ui.add_space(8.0);
                            ui.label(
                                egui::RichText::new("Custom Themes")
                                    .size(12.0)
                                    .strong()
                                    .color(self.theme.text_secondary()),
                            );
                            ui.horizontal_wrapped(|ui| {
                                for t in &custom_themes {
                                    let is_active = self.config.theme == t.name;
                                    let bg = if is_active {
                                        t.accent()
                                    } else {
                                        t.bg_primary()
                                    };
                                    let text_col = if is_active {
                                        t.text_on_accent()
                                    } else {
                                        t.text_primary()
                                    };

                                    let btn = ui.add(
                                        egui::Button::new(
                                            egui::RichText::new(&t.name).size(11.0).color(text_col),
                                        )
                                        .fill(bg)
                                        .stroke(egui::Stroke::new(
                                            if is_active { 2.0 } else { 1.0 },
                                            t.border(),
                                        ))
                                        .corner_radius(egui::CornerRadius::same(4))
                                        .min_size(egui::vec2(100.0, 28.0)),
                                    );

                                    if btn.clicked() && !is_active {
                                        self.config.theme = t.name.clone();
                                        self.theme = resolve_theme(&self.config.theme);
                                        save_config(&self.config);
                                        self.set_status(StatusMessage::info(format!(
                                            "Theme set to {}",
                                            t.name
                                        )));
                                    }
                                }
                            });
                        }

                        ui.add_space(12.0);
                        ui.separator();
                        ui.add_space(8.0);

                        // Custom theme info
                        ui.label(
                            egui::RichText::new("Custom Themes")
                                .size(13.0)
                                .strong()
                                .color(self.theme.text_primary()),
                        );
                        ui.add_space(4.0);

                        let themes_path = themes_dir()
                            .map(|p| p.display().to_string())
                            .unwrap_or_else(|| "~/.config/viar/themes/".into());

                        ui.label(
                            egui::RichText::new(format!(
                                "Place .json theme files in:\n{}",
                                themes_path,
                            ))
                            .size(11.0)
                            .color(self.theme.text_secondary()),
                        );
                        ui.add_space(4.0);
                        ui.label(
                            egui::RichText::new(
                                "The filename (without .json) becomes the theme name.\n\
                                 Set it in config.toml: theme = \"my-theme-name\"",
                            )
                            .size(10.0)
                            .color(self.theme.text_muted()),
                        );

                        ui.add_space(8.0);

                        if ui.button("Export theme template").clicked() {
                            let template = export_theme_template();
                            if let Some(dir) = themes_dir() {
                                let _ = std::fs::create_dir_all(&dir);
                                let path = dir.join("custom-template.json");
                                match std::fs::write(&path, &template) {
                                    Ok(()) => {
                                        self.set_status(StatusMessage::info(format!(
                                            "Template exported to {}",
                                            path.display()
                                        )));
                                    }
                                    Err(e) => {
                                        self.set_status(StatusMessage::error(format!(
                                            "Failed to export: {e}"
                                        )));
                                    }
                                }
                            }
                        }
                    });

                ui.add_space(16.0);

                // Config file info
                egui::Frame::default()
                    .inner_margin(egui::Margin::same(16))
                    .corner_radius(egui::CornerRadius::same(6))
                    .fill(self.theme.bg_secondary())
                    .stroke(egui::Stroke::new(1.0, self.theme.border()))
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new("Configuration")
                                .size(16.0)
                                .strong()
                                .color(self.theme.text_primary()),
                        );
                        ui.add_space(4.0);

                        let config_path = config_dir()
                            .map(|p| p.join("config.toml").display().to_string())
                            .unwrap_or_else(|| "~/.config/viar/config.toml".into());

                        ui.label(
                            egui::RichText::new(format!("Config file: {config_path}"))
                                .size(11.0)
                                .monospace()
                                .color(self.theme.text_secondary()),
                        );
                    });
            });
        });
    }
}
