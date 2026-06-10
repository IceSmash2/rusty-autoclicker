use device_query::{Keycode, MouseState};
use eframe::egui;

use crate::{
    RustyAutoClickerApp,
    defines::{MOUSE_TWEEN_SPEED_MAX_PX_S, MOUSE_TWEEN_SPEED_MIN_PX_S},
    types::InteractionMode,
};

mod bars;
mod buttons;
mod click_config;

impl RustyAutoClickerApp {
    pub fn show_movement_speed(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            ui.label("Movement speed (Humanlike only)");

            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                ui.label("px/s");
                self.disable_if_busy(ui);
                ui.add(
                    egui::TextEdit::singleline(&mut self.speed_max_str)
                        .desired_width(40.0f32)
                        .hint_text(MOUSE_TWEEN_SPEED_MAX_PX_S.to_string()),
                );

                ui.label("to");
                ui.add(
                    egui::TextEdit::singleline(&mut self.speed_min_str)
                        .desired_width(40.0f32)
                        .hint_text(MOUSE_TWEEN_SPEED_MIN_PX_S.to_string()),
                );
            });
        });
    }

    pub fn show_infos(&self, ui: &mut egui::Ui, mouse: &MouseState, keys: &[Keycode]) {
        let mouse_txt = format!("Mouse position: {:?}", mouse.coords);
        ui.label(mouse_txt);
        let key_txt = format!("Key pressed: {keys:?}");
        ui.label(key_txt);
        let extra_buttons_pressed = mouse
            .button_pressed
            .iter()
            .enumerate()
            .skip(4)
            .map(|(button_number, pressed)| format!("{button_number:?}-{pressed:?}"))
            .collect::<Vec<String>>()
            .join(" ");

        ui.label(format!(
            "Mouse pressed: L-{:?} R-{:?} M-{:?} {}",
            mouse.button_pressed[1],
            mouse.button_pressed[2],
            mouse.button_pressed[3],
            &extra_buttons_pressed
        ));
    }

    pub fn show_autoclicker(&mut self, ui: &mut egui::Ui) {
        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
            // Allocate a fixed-width region for the button pair so the centered
            // parent layout centers the whole group (a plain `horizontal` would
            // fill the full width and left-align its content).
            let button_size = egui::vec2(120.0f32, 38.0f32);
            let group_width = button_size.x * 2.0 + ui.spacing().item_spacing.x;

            ui.allocate_ui_with_layout(
                egui::vec2(group_width, button_size.y),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    // Autoclick start/stop: disabled while holding or the hotkeys window is open
                    let autoclick_enabled = !self.hotkey_window_open && !self.is_holding();
                    if ui
                        .add_enabled(
                            autoclick_enabled,
                            egui::widgets::Button::new(self.autoclick_button_label())
                                .min_size(button_size),
                        )
                        .clicked()
                    {
                        if self.is_autoclicking() {
                            self.mode = InteractionMode::Idle;
                        } else {
                            // Start autoclick, first click is delayed
                            self.start_autoclick(0u64);
                        }
                    }

                    // Click & hold: disabled while autoclicking or the hotkeys window is open
                    let hold_enabled = !self.hotkey_window_open && !self.is_autoclicking();
                    if ui
                        .add_enabled(
                            hold_enabled,
                            egui::widgets::Button::new(self.hold_button_label())
                                .min_size(button_size),
                        )
                        .clicked()
                    {
                        if self.is_holding() {
                            self.stop_hold();
                        } else {
                            self.start_hold();
                        }
                    }
                },
            );
        });
    }
}
