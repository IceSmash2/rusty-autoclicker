use eframe::egui;

use crate::{
    RustyAutoClickerApp,
    settings::{self, Settings},
    types::{AppMode, InteractionMode},
};

impl RustyAutoClickerApp {
    pub fn show_topbar(&mut self, ui: &mut egui::Ui) {
        egui::Panel::top("top_panel").show_inside(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                // ── Start / Stop button ──────────────────────────────────
                if self.is_autoclicking() {
                    if ui.button(self.autoclick_button_label()).clicked() {
                        self.mode = InteractionMode::Idle;
                    }
                } else {
                    if self.hotkey_window_open || self.is_holding() {
                        ui.disable();
                    }
                    if ui.button(self.autoclick_button_label()).clicked() {
                        self.start_autoclick(0u64);
                    }
                }

                ui.separator();
                ui.label("Settings:");

                // ── Hotkeys window ───────────────────────────────────────
                if ui
                    .add_enabled(
                        !self.is_autoclicking() && !self.is_holding(),
                        egui::Button::new("⌨ Hotkeys"),
                    )
                    .clicked()
                {
                    self.hotkey_window_open = true;
                }

                // ── Save button ──────────────────────────────────────────
                if ui
                    .add_enabled(
                        !self.is_busy(),
                        egui::Button::new(egui::RichText::new("💾").size(16.0))
                            .min_size(egui::vec2(28.0, 24.0)),
                    )
                    .on_hover_text("Save current settings to disk")
                    .clicked()
                {
                    settings::save_settings(&Settings::from_app(self));
                }

                // ── Reset button ─────────────────────────────────────────
                let ctx = ui.ctx().clone();
                if ui
                    .add_enabled(
                        !self.is_busy(),
                        egui::Button::new(egui::RichText::new("↺").size(20.0))
                            .min_size(egui::vec2(28.0, 24.0)),
                    )
                    .on_hover_text("Reset all settings to defaults")
                    .clicked()
                {
                    self.reset_to_defaults(&ctx);
                }

                ui.separator();
                ui.label("App Mode:");

                self.disable_if_busy(ui);
                ui.selectable_value(&mut self.app_mode, AppMode::Bot, "🖥 Bot")
                    .on_hover_text("Autoclick as fast as possible");
                ui.selectable_value(&mut self.app_mode, AppMode::Humanlike, "😆 Humanlike")
                    .on_hover_text("Autoclick emulating human clicking");
            });
        });
    }

    pub fn show_bottombar(&mut self, ui: &mut egui::Ui) {
        egui::Panel::bottom("bottom_panel").show_inside(ui, |ui| {
            ui.with_layout(egui::Layout::bottom_up(egui::Align::RIGHT), |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 5.0;
                    ui.hyperlink_to(
                        "eframe",
                        "https://github.com/emilk/egui/tree/master/eframe",
                    );
                    ui.label(" and ");
                    ui.hyperlink_to("egui", "https://github.com/emilk/egui");
                    ui.label("powered by ");
                    ui.hyperlink_to(
                        "rusty-autoclicker",
                        "https://github.com/MrTanoshii/rusty-autoclicker",
                    );
                    ui.separator();
                    egui::warn_if_debug_build(ui);
                });
            });
        });
    }
}
