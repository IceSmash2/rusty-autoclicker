#![forbid(unsafe_code)]
#![cfg_attr(not(debug_assertions), deny(warnings))]
#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;

mod app;
mod defines;
mod gui;
mod settings;
mod types;
mod utils;

use crate::{
    app::RustyAutoClickerApp,
    defines::{APP_NAME, WINDOW_DEFAULT_X, WINDOW_DEFAULT_Y, WINDOW_HEIGHT, WINDOW_WIDTH},
    settings::load_settings,
    utils::load_icon,
};

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    use eframe::egui::ViewportBuilder;

    // Read the saved position before constructing NativeOptions so the OS can
    // place the window at the correct location from the very first frame —
    // eliminating the "flash at default pos → jump to saved pos" flicker.
    let saved = load_settings();
    let start_pos = saved
        .as_ref()
        .and_then(|s| s.window_position)
        .map(|[x, y]| egui::pos2(x, y))
        .unwrap_or_else(|| egui::pos2(WINDOW_DEFAULT_X, WINDOW_DEFAULT_Y));
    let start_size = saved
        .as_ref()
        .and_then(|s| s.window_size)
        .map(|[w, h]| egui::vec2(w, h))
        .unwrap_or_else(|| egui::vec2(WINDOW_WIDTH, WINDOW_HEIGHT));

    let native_options = eframe::NativeOptions {
        renderer: eframe::Renderer::Wgpu,
        viewport: ViewportBuilder::default()
            .with_always_on_top()
            .with_decorations(true)
            // Start at the saved size — no flicker
            .with_inner_size(start_size)
            // Start at the saved position — no flicker
            .with_position(start_pos)
            .with_resizable(true)
            .with_transparent(true)
            .with_icon(load_icon()),
        ..Default::default()
    };

    if let Err(e) = eframe::run_native(
        &format!("{} v{}", APP_NAME, env!("CARGO_PKG_VERSION")),
        native_options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Ok(Box::new(RustyAutoClickerApp::new(cc)))
        }),
    ) {
        native_dialog::DialogBuilder::message()
            .set_level(native_dialog::MessageLevel::Error)
            .set_title("Failed to Initialize Graphics")
            .set_text(format!(
                "{e}\n\n\
                Rusty AutoClicker could not start due to a graphics initialization error.\n\n\
                Please ensure your system has a compatible graphics driver installed and supports Vulkan, Metal or DirectX 12.\n\n\
                If the problem persists, try updating your graphics drivers or running the application on a different machine."
            ))
            .alert()
            .show()
            .unwrap();
    };
}
