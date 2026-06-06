use eframe::egui::{self, Context};

use crate::{RustyAutoClickerApp, types::InteractionMode};

impl RustyAutoClickerApp {
    pub fn show_hotkeys_window(&mut self, ctx: &Context) {
        let idle = self.is_idle();
        egui::Window::new("Hotkeys")
            .fixed_size(egui::vec2(220f32, 100f32))
            .anchor(egui::Align2::CENTER_CENTER, [0f32, 0f32])
            .collapsible(false)
            .open(&mut self.hotkey_window_open)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui
                        .add_sized(
                            [100.0f32, 32.0f32],
                            egui::widgets::Button::new("Start/Stop"),
                        )
                        .clicked()
                    {
                        // Allow keybind only if app is not busy
                        if idle {
                            self.mode = InteractionMode::SettingAutoclickKey;
                            self.key_autoclick = None;
                        }
                    };
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                        ui.disable();
                        let text: String = if let Some(pressed_keys) = self.key_autoclick {
                            format!("{:}", pressed_keys)
                        } else {
                            "PRESS ANY KEY".to_string()
                        };
                        ui.add_sized([110.0f32, 32.0f32], egui::widgets::Button::new(text));
                    });
                });
                ui.horizontal(|ui| {
                    if ui
                        .add_sized(
                            [100.0f32, 32.0f32],
                            egui::widgets::Button::new("Confirm Coords"),
                        )
                        .on_hover_text("Note: L Click cannot be changed")
                        .clicked()
                    {
                        // Allow keybind only if app is not busy
                        if idle {
                            self.key_set_coord = None;
                            self.mode = InteractionMode::SettingSetCoordKey;
                        }
                    };
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                        ui.disable();
                        let text: String = if let Some(pressed_keys) = self.key_set_coord {
                            format!("{:} / L Click", pressed_keys)
                        } else {
                            "PRESS ANY KEY".to_string()
                        };
                        ui.add_sized([110.0f32, 32.0f32], egui::widgets::Button::new(text));
                    });
                });
            });
    }
}
