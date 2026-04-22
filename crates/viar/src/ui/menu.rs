use eframe::egui;
use tracing::debug;

use crate::types::{
    AppScreen,
    ConfirmAction,
    ConfirmDialog,
    ViarApp,
};

impl ViarApp {
    /// Render the top menu bar.
    pub fn render_menu_bar(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        egui::Panel::top("menu_bar").show_inside(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("Viar", |ui| {
                    if ui.button("Refresh Devices").clicked() {
                        self.refresh();
                        ui.close();
                    }
                    if self.connected_device.is_some() {
                        if ui.button("Disconnect").clicked() {
                            self.disconnect();
                            ui.close();
                        }
                    }
                    ui.separator();
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                if self.connected_device.is_some() {
                    ui.menu_button("Device", |ui| {
                        if ui.button("Switch Keyboard...").clicked() {
                            self.connected_device = None;
                            self.protocol_version = None;
                            self.keymap_data = None;
                            self.screen = AppScreen::SelectKeyboard;
                            ui.close();
                        }
                        ui.separator();
                        if ui.button("Reload Keymap").clicked() {
                            self.reload_keymap();
                            ui.close();
                        }
                        ui.separator();
                        if ui.button("Export Keymap...").clicked() {
                            self.export_keymap();
                            ui.close();
                        }
                        if ui.button("Import Keymap...").clicked() {
                            self.confirm_dialog = Some(ConfirmDialog {
                                title: "Import Keymap".to_string(),
                                message: "This will overwrite your current keymap with the contents of viar_keymap.json.\nAny unsaved changes will be lost.".to_string(),
                                action: ConfirmAction::Import,
                            });
                            ui.close();
                        }
                        ui.separator();
                        if ui.button("Lock Keyboard").clicked() {
                            debug!("lock keyboard (not yet implemented)");
                            ui.close();
                        }
                        if ui.button("Unlock Keyboard").clicked() {
                            debug!("unlock keyboard (not yet implemented)");
                            ui.close();
                        }
                    });
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if let Some(ref status) = self.status {
                        let color = if status.is_error {
                            egui::Color32::from_rgb(220, 80, 80)
                        } else {
                            egui::Color32::from_rgb(80, 180, 80)
                        };
                        ui.colored_label(color, &status.text);
                        ui.separator();
                    }
                    if let Some(dev) = &self.connected_device {
                        ui.label(format!("Connected: {}", dev.info));
                    }
                });
            });
        });
    }

    /// Render and handle the confirmation dialog. Returns true if a dialog was shown.
    pub fn render_confirm_dialog(&mut self, ctx: &egui::Context) {
        let mut confirm_result: Option<bool> = None;
        if let Some(dialog) = &self.confirm_dialog {
            egui::Window::new(&dialog.title)
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label(&dialog.message);
                    ui.add_space(12.0);
                    ui.horizontal(|ui| {
                        if ui.button("Cancel").clicked() {
                            confirm_result = Some(false);
                        }
                        if ui
                            .add(egui::Button::new(egui::RichText::new("Import").strong()))
                            .clicked()
                        {
                            confirm_result = Some(true);
                        }
                    });
                });
        }
        if let Some(confirmed) = confirm_result {
            if confirmed {
                if let Some(dialog) = self.confirm_dialog.take() {
                    match dialog.action {
                        ConfirmAction::Import => self.import_keymap(),
                    }
                }
            } else {
                self.confirm_dialog = None;
            }
        }
    }
}
