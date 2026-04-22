use eframe::egui;

use crate::{
    types::{
        ConnectedTab,
        ViarApp,
    },
    util::themed_tab,
};

impl ViarApp {
    pub fn render_connected(&mut self, ui: &mut egui::Ui) {
        let theme = self.theme.clone();
        // Top tab bar
        ui.horizontal(|ui| {
            ui.add_space(8.0);
            if themed_tab(
                ui,
                self.active_tab == ConnectedTab::Keymap,
                "Keymap",
                &theme,
            )
            .clicked()
            {
                self.active_tab = ConnectedTab::Keymap;
            }
            if self.lighting_data.is_some()
                && themed_tab(
                    ui,
                    self.active_tab == ConnectedTab::Lighting,
                    "Lighting",
                    &theme,
                )
                .clicked()
            {
                self.active_tab = ConnectedTab::Lighting;
            }
            if self.dynamic_data.is_some() {
                if themed_tab(
                    ui,
                    self.active_tab == ConnectedTab::TapDance,
                    "Tap Dance",
                    &theme,
                )
                .clicked()
                {
                    self.active_tab = ConnectedTab::TapDance;
                }
                if themed_tab(
                    ui,
                    self.active_tab == ConnectedTab::Combos,
                    "Combos",
                    &theme,
                )
                .clicked()
                {
                    self.active_tab = ConnectedTab::Combos;
                }
                if themed_tab(
                    ui,
                    self.active_tab == ConnectedTab::KeyOverrides,
                    "Key Overrides",
                    &theme,
                )
                .clicked()
                {
                    self.active_tab = ConnectedTab::KeyOverrides;
                }
            }

            // Settings always available, pushed to the right
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if let Some(ver) = self.protocol_version {
                    ui.label(format!("VIA v{ver}"));
                    ui.separator();
                }
                if let Some(data) = &self.keymap_data {
                    ui.label(&data.layout.name);
                    ui.separator();
                }
                if themed_tab(
                    ui,
                    self.active_tab == ConnectedTab::Settings,
                    "Settings",
                    &theme,
                )
                .clicked()
                {
                    self.active_tab = ConnectedTab::Settings;
                }
            });
        });

        ui.separator();

        match self.active_tab {
            ConnectedTab::Keymap => self.render_keymap_tab(ui),
            ConnectedTab::Lighting => self.render_lighting_tab(ui),
            ConnectedTab::TapDance => self.render_tap_dance_tab(ui),
            ConnectedTab::Combos => self.render_combos_tab(ui),
            ConnectedTab::KeyOverrides => self.render_key_overrides_tab(ui),
            ConnectedTab::Settings => self.render_settings_tab(ui),
        }
    }
}
