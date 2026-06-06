use eframe::egui;

use crate::{
    RustyAutoClickerApp,
    types::{AppMode, InteractionMode},
};

impl RustyAutoClickerApp {
    pub fn show_topbar(&mut self, ui: &mut egui::Ui) {
        egui::Panel::top("top_panel").show_inside(ui, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::MenuBar::new().ui(ui, |ui| {
                if self.is_autoclicking() {
                    if ui.button(self.autoclick_button_label()).clicked() {
                        self.mode = InteractionMode::Idle;
                    };
                } else {
                    if self.hotkey_window_open {
                        ui.disable();
                    }
                    if ui.button(self.autoclick_button_label()).clicked() {
                        self.start_autoclick(0u64);
                    }
                }

                ui.separator();
                ui.label("Settings: ");

                if ui
                    .add_enabled(!self.is_autoclicking(), egui::Button::new("⌨ Hotkeys"))
                    .clicked()
                {
                    self.hotkey_window_open = true
                };

                ui.separator();
                ui.label("App Mode: ");

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
                    ui.hyperlink_to("eframe", "https://github.com/emilk/egui/tree/master/eframe");
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
