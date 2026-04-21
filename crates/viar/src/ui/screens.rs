use eframe::egui;

use crate::types::{AppScreen, ViarApp};

impl ViarApp {
    pub fn render_detecting(&self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(ui.available_height() / 3.0);
            ui.heading("Detecting keyboards...");
            ui.spinner();
        });
    }

    pub fn render_no_permission(&self, ui: &mut egui::Ui) {
        let msg = match &self.screen {
            AppScreen::NoPermission(m) => m.clone(),
            _ => String::new(),
        };
        ui.vertical_centered(|ui| {
            ui.add_space(ui.available_height() / 3.0);
            ui.heading("Cannot access HID devices");
            ui.add_space(12.0);
            ui.label(&msg);
            ui.add_space(20.0);
            ui.label(
                "After adding a udev rule, unplug and replug your keyboard,\n\
                 or reload udev rules with: sudo udevadm control --reload-rules",
            );
        });
    }

    pub fn render_no_keyboards(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(ui.available_height() / 3.0);
            ui.heading("No VIA keyboards found");
            ui.add_space(12.0);
            ui.label("Make sure your keyboard firmware has VIA enabled.");
            ui.add_space(20.0);
            if ui.button("Retry").clicked() {
                self.refresh();
            }
        });
    }

    pub fn render_select_keyboard(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(40.0);
            ui.heading("Select a keyboard");
            ui.add_space(20.0);
        });

        let mut connect_idx = None;
        let width = 400.0_f32.min(ui.available_width() - 40.0);

        ui.vertical_centered(|ui| {
            for (i, kb) in self.keyboards.iter().enumerate() {
                let label = format!(
                    "{} {}\n{:04x}:{:04x}",
                    kb.manufacturer, kb.product, kb.vendor_id, kb.product_id
                );
                let button = egui::Button::new(egui::RichText::new(&label).size(14.0))
                    .min_size(egui::vec2(width, 50.0));
                if ui.add(button).clicked() {
                    connect_idx = Some(i);
                }
                ui.add_space(4.0);
            }
        });

        if let Some(idx) = connect_idx {
            self.connect_to_keyboard(idx);
        }
    }
}
