mod app;
mod types;
mod ui;
mod util;

use eframe::egui;
use tracing::info;
use types::ViarApp;

fn main() -> eframe::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                tracing_subscriber::EnvFilter::new(
                    "info,winit=warn,eframe=warn,egui=warn,wgpu=warn,naga=warn",
                )
            }),
        )
        .init();

    info!("starting viar");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1100.0, 750.0])
            .with_min_inner_size([700.0, 500.0])
            .with_title("Viar — Keyboard Configurator"),
        ..Default::default()
    };

    eframe::run_native(
        "Viar",
        options,
        Box::new(|cc| {
            // Load JetBrains Mono as the default proportional + monospace font
            let mut fonts = egui::FontDefinitions::default();
            let font_data = std::sync::Arc::new(egui::FontData::from_static(include_bytes!(
                "../assets/JetBrainsMono-Regular.ttf"
            )));
            fonts
                .font_data
                .insert("JetBrainsMono".to_owned(), font_data);
            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .insert(0, "JetBrainsMono".to_owned());
            fonts
                .families
                .entry(egui::FontFamily::Monospace)
                .or_default()
                .insert(0, "JetBrainsMono".to_owned());
            cc.egui_ctx.set_fonts(fonts);

            Ok(Box::new(ViarApp::new()))
        }),
    )
}
