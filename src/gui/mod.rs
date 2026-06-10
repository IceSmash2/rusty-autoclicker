use std::time::{Duration, Instant};

use device_query::{DeviceQuery, DeviceState, Keycode, MouseState};
use eframe::egui;

use crate::{
    RustyAutoClickerApp,
    types::{ClickInfo, InteractionMode},
    utils::autoclick,
};

mod sections;
mod windows;

impl RustyAutoClickerApp {
    /// Poll the current mouse and keyboard state, caching the mouse snapshot so
    /// `ui` can display it while all device polling stays in `logic`.
    fn poll_devices(&mut self) -> (MouseState, Vec<Keycode>) {
        let device_state = DeviceState::new();
        let mouse = device_state.get_mouse();
        let keys = device_state.get_keys();
        self.mouse = mouse.clone();
        (mouse, keys)
    }

    /// Close the hotkeys window once Escape is pressed and then released, but
    /// only while the app is idle.
    fn handle_hotkey_window_escape(&mut self, keys: &[Keycode]) {
        if !self.hotkey_window_open {
            return;
        }
        if keys.contains(&Keycode::Escape) {
            self.key_pressed_esc = true;
        } else if self.key_pressed_esc {
            // Close only if app is not busy
            if self.is_idle() {
                self.hotkey_window_open = false;
            }
            self.key_pressed_esc = false;
        }
    }

    /// Toggle autoclicking when the configured hotkey is pressed and released.
    fn handle_autoclick_toggle(&mut self, keys: &[Keycode], interval: u64) {
        if self.key_autoclick.is_some() && keys.contains(&self.key_autoclick.unwrap()) {
            self.key_pressed_autoclick = true;
        } else if self.key_pressed_autoclick {
            self.key_pressed_autoclick = false;
            if self.is_autoclicking() {
                self.mode = InteractionMode::Idle;
            } else if self.is_idle() && !self.hotkey_window_open {
                // Start autoclick, first click is instantaneous
                self.start_autoclick(interval);
            }
        }
    }

    /// Toggle click-and-hold when the configured hotkey is pressed and released:
    /// press the selected button down, then release it on the next press.
    fn handle_hold_toggle(&mut self, keys: &[Keycode]) {
        if self.key_hold.is_some() && keys.contains(&self.key_hold.unwrap()) {
            self.key_pressed_hold = true;
        } else if self.key_pressed_hold {
            self.key_pressed_hold = false;
            if self.is_holding() {
                self.stop_hold();
            } else if self.is_idle() && !self.hotkey_window_open {
                self.start_hold();
            }
        }
    }

    /// Fire a click if the interval has elapsed, advancing the counter and
    /// stopping when the requested amount is reached. Returns `true` if a click
    /// was dispatched this pass.
    fn dispatch_click(
        &mut self,
        now: Instant,
        interval: u64,
        click_amount: u64,
        click_coord: (f64, f64),
        speed_range: (f64, f64),
        mouse_coord: (i32, i32),
    ) -> bool {
        let since_last = now
            .checked_duration_since(self.last_now)
            .unwrap_or(Duration::ZERO);
        if !self.is_autoclicking() || (since_last.as_millis() as u64) < interval {
            return false;
        }

        #[cfg(debug_assertions)]
        println!(
            "{:?} {:?} Click: {since_last:?}",
            self.click_type, self.click_btn
        );
        self.last_now = Instant::now();

        autoclick(
            self.app_mode,
            ClickInfo {
                click_btn: self.click_btn,
                click_coord,
                click_position: self.click_position,
                click_type: self.click_type,
            },
            mouse_coord,
            speed_range,
            self.rng_thread.clone(),
        );

        // Increment click counter and stop autoclicking if completed
        self.click_counter += 1u64;
        if click_amount != 0u64 && self.click_counter >= click_amount {
            self.mode = InteractionMode::Idle;
        }
        true
    }

    /// Record the next released key into `target_key`, clearing `target_flag`.
    /// An associated function so the two disjoint `&mut` fields can be borrowed
    /// from `self` at the call site.
    fn capture_key(
        target_key: &mut Option<Keycode>,
        mode: &mut InteractionMode,
        last_keys: Option<&[Keycode]>,
        keys: &[Keycode],
    ) {
        let Some(last_keys) = last_keys else {
            return;
        };
        for pressed_key in last_keys {
            if !keys.contains(pressed_key) {
                *target_key = Some(*pressed_key);
                *mode = InteractionMode::Idle;
                break;
            }
        }
    }

    /// Track the cursor while setting coordinates, exiting on a left-click or the
    /// configured confirm key.
    fn update_coordinate_setting(
        &mut self,
        ctx: &egui::Context,
        mouse: &MouseState,
        keys: &[Keycode],
    ) {
        self.click_x_str = mouse.coords.0.to_string();
        self.click_y_str = mouse.coords.1.to_string();

        // Stop if mouse left click
        if mouse.button_pressed[1]
            || (self.key_set_coord.is_some() && keys.contains(&self.key_set_coord.unwrap()))
        {
            self.exit_coordinate_setting(ctx);
        }
    }

    /// Force the hotkeys window open while either hotkey is unset.
    fn ensure_hotkey_window(&mut self) {
        if !self.hotkey_window_open
            && (self.key_autoclick.is_none()
                || self.key_set_coord.is_none()
                || self.key_hold.is_none())
        {
            self.hotkey_window_open = true;
        }
    }
}

impl eframe::App for RustyAutoClickerApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        egui::Rgba::TRANSPARENT.to_array() // Make sure we don't paint anything behind the rounded corners
    }

    /// Runs every pass, including while the window is hidden/minimized, as long as a repaint was requested.
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Print time to between start of old and new frames
        #[cfg(debug_assertions)]
        println!(
            "Frame delta: {:?}",
            Instant::now()
                .checked_duration_since(self.frame_start)
                .unwrap()
        );

        self.frame_start = Instant::now();

        // Poll input devices (mouse snapshot is cached for `ui` to display)
        let (mouse, keys) = self.poll_devices();

        // Sanitize the numeric input strings, then parse the values needed this pass
        self.sanitize_inputs();
        let interval = self.parsed_interval_ms();
        let speed_range = self.parsed_speed_range();
        let click_amount = self.parsed_click_amount();
        let click_coord = self.parsed_click_coord();

        self.handle_hotkey_window_escape(&keys);

        // Sample the clock once, before the toggle, and reuse it for the click timing
        let update_now = Instant::now();
        self.handle_autoclick_toggle(&keys, interval);
        self.handle_hold_toggle(&keys);

        // At most one of these runs per pass (mutually exclusive)
        if !self.dispatch_click(
            update_now,
            interval,
            click_amount,
            click_coord,
            speed_range,
            mouse.coords,
        ) {
            if matches!(self.mode, InteractionMode::SettingAutoclickKey)
                && self.keys_pressed.is_some()
            {
                Self::capture_key(
                    &mut self.key_autoclick,
                    &mut self.mode,
                    self.keys_pressed.as_deref(),
                    &keys,
                );
            } else if matches!(self.mode, InteractionMode::SettingSetCoordKey)
                && self.keys_pressed.is_some()
            {
                Self::capture_key(
                    &mut self.key_set_coord,
                    &mut self.mode,
                    self.keys_pressed.as_deref(),
                    &keys,
                );
            } else if matches!(self.mode, InteractionMode::SettingHoldKey)
                && self.keys_pressed.is_some()
            {
                Self::capture_key(
                    &mut self.key_hold,
                    &mut self.mode,
                    self.keys_pressed.as_deref(),
                    &keys,
                );
            } else if matches!(self.mode, InteractionMode::SettingCoord) {
                self.update_coordinate_setting(ctx, &mouse, &keys);
            }
        }

        // Save state of pressed keys
        self.keys_pressed = Some(keys.clone());

        self.ensure_hotkey_window();

        // Keep updating frame
        ctx.request_repaint();

        // Print time to process frame
        #[cfg(debug_assertions)]
        println!(
            "Frame time: {:?}",
            Instant::now()
                .checked_duration_since(self.frame_start)
                .unwrap()
        );
    }

    /// Called each time the UI needs repainting, but only while the window is visible.
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Cheap `Arc`-handle clone of the context so helpers/windows can use it without
        // conflicting with the `&mut ui` borrows taken by `show_inside`.
        let ctx = ui.ctx().clone();

        if matches!(self.mode, InteractionMode::SettingCoord) {
            egui::Panel::top("top_panel").show_inside(ui, |ui| {
                egui::MenuBar::new().ui(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                            self.disable_if_busy(ui);
                            ui.add(
                                egui::TextEdit::singleline(&mut self.click_y_str)
                                    .desired_width(50.0f32)
                                    .hint_text("0"),
                            );
                            ui.label("Y");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.click_x_str)
                                    .desired_width(50.0f32)
                                    .hint_text("0"),
                            );
                            ui.label("X");
                            ui.separator();
                            ui.label(format!(
                                " Set with \"{:}\" / \"L Click\"",
                                self.key_set_coord.unwrap()
                            ));
                        });
                    });
                })
            });
            Self::follow_cursor(self, &ctx);
        } else {
            // GUI
            // Top and bottom panels first so the central panel fills the remaining space
            self.show_topbar(ui);
            self.show_bottombar(ui);

            let click_amount = self.parsed_click_amount();

            egui::CentralPanel::default().show_inside(ui, |ui| {
                // The central panel the region left after adding TopPanel's and SidePanel's
                self.show_click_interval(ui);
                ui.separator();
                self.show_movement_speed(ui);
                ui.separator();
                self.show_buttons(ui);
                ui.separator();
                self.show_click_type(ui);
                ui.separator();
                self.show_click_amount(ui, click_amount);
                ui.separator();
                self.show_click_position(ui, &ctx);
                ui.separator();
                self.show_infos(ui, &self.mouse, self.keys_pressed.as_deref().unwrap_or(&[]));
                ui.separator();
                self.show_autoclicker(ui);
            });
        }

        // Hotkeys window
        if self.hotkey_window_open {
            self.show_hotkeys_window(&ctx);
        }
    }
}
