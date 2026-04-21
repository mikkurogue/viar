use eframe::egui;

use crate::types::{ConnectedTab, ViarApp};

impl ViarApp {
    pub fn render_connected(&mut self, ui: &mut egui::Ui) {
        // Top tab bar: Keymap | Lighting
        ui.horizontal(|ui| {
            ui.add_space(8.0);
            if ui
                .selectable_label(self.active_tab == ConnectedTab::Keymap, "Keymap")
                .clicked()
            {
                self.active_tab = ConnectedTab::Keymap;
            }
            if self.lighting_data.is_some() {
                if ui
                    .selectable_label(self.active_tab == ConnectedTab::Lighting, "Lighting")
                    .clicked()
                {
                    self.active_tab = ConnectedTab::Lighting;
                }
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if let Some(ver) = self.protocol_version {
                    ui.label(format!("VIA v{ver}"));
                    ui.separator();
                }
                if let Some(data) = &self.keymap_data {
                    ui.label(&data.layout.name);
                }
            });
        });

        ui.separator();

        match self.active_tab {
            ConnectedTab::Keymap => self.render_keymap_tab(ui),
            ConnectedTab::Lighting => self.render_lighting_tab(ui),
        }
    }
}
