use std::time::{Duration, Instant};

use device_query::{Keycode, MouseState};
use eframe::{egui, epaint::FontId};
use rand::{prelude::ThreadRng, rng};
use rdev::Button;

use crate::{
    defines::*,
    types::{AppMode, ClickButton, ClickPosition, ClickType, InteractionMode},
    utils::{
        interval_ms, move_mouse_to, press_button, release_button, sanitize_i64_string,
        sanitize_string,
    },
};

pub struct RustyAutoClickerApp {
    // Text input strings
    pub hr_str: String,
    pub min_str: String,
    pub sec_str: String,
    pub ms_str: String,
    pub click_amount_str: String,
    pub click_x_str: String,
    pub click_y_str: String,
    pub speed_min_str: String,
    pub speed_max_str: String,

    // Time
    pub last_now: Instant,
    pub frame_start: Instant,

    // Counter
    pub click_counter: u64,

    // Hotkeys
    pub key_autoclick: Option<Keycode>,
    pub key_set_coord: Option<Keycode>,
    pub key_hold: Option<Keycode>,

    // Interaction state (mutually exclusive)
    pub mode: InteractionMode,

    // The button currently held down (click-and-hold), if any
    pub held_button: Option<ClickButton>,

    // App mode
    pub app_mode: AppMode,

    // Window state
    pub hotkey_window_open: bool,
    pub window_position: egui::Pos2,

    // Key states
    pub key_pressed_autoclick: bool,
    pub key_pressed_set_coord: bool,
    pub key_pressed_esc: bool,
    pub key_pressed_hold: bool,
    pub keys_pressed: Option<Vec<Keycode>>,

    // Mouse snapshot (polled in `logic`, displayed in `ui`)
    pub mouse: MouseState,

    // Enums
    pub click_btn: ClickButton,
    pub click_type: ClickType,
    pub click_position: ClickPosition,
    // RNG
    pub rng_thread: ThreadRng,
}

impl Default for RustyAutoClickerApp {
    fn default() -> Self {
        Self {
            // Text input strings
            hr_str: DEFAULT_HR_STR.to_owned(),
            min_str: DEFAULT_MIN_STR.to_owned(),
            sec_str: DEFAULT_SEC_STR.to_owned(),
            ms_str: DEFAULT_MS_STR.to_owned(),
            click_amount_str: DEFAULT_CLICK_AMOUNT_STR.to_owned(),
            click_x_str: DEFAULT_CLICK_X_STR.to_owned(),
            click_y_str: DEFAULT_CLICK_Y_STR.to_owned(),
            speed_min_str: MOUSE_TWEEN_SPEED_MIN_PX_S.to_string(),
            speed_max_str: MOUSE_TWEEN_SPEED_MAX_PX_S.to_string(),

            // Time
            last_now: Instant::now(),
            frame_start: Instant::now(),

            // Counter
            click_counter: 0u64,

            // Hotkeys
            key_autoclick: HOTKEY_AUTOCLICK,
            key_set_coord: HOTKEY_SET_COORD,
            key_hold: HOTKEY_HOLD,

            // Interaction state
            mode: InteractionMode::Idle,

            // No button held initially
            held_button: None,

            // App mode
            app_mode: AppMode::Bot,

            // Window state
            hotkey_window_open: false,
            window_position: egui::Pos2 { x: 0f32, y: 0f32 },

            // Key states
            key_pressed_autoclick: false,
            key_pressed_set_coord: false,
            key_pressed_esc: false,
            key_pressed_hold: false,
            keys_pressed: None,

            // Mouse snapshot
            mouse: MouseState::default(),

            // Enums
            click_btn: ClickButton::Mouse(Button::Left),
            click_type: ClickType::Single,
            click_position: ClickPosition::Mouse,

            // RNG
            rng_thread: rng(),
        }
    }
}

impl RustyAutoClickerApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let ctx = &cc.egui_ctx;

        let mut style = (*ctx.global_style()).clone();
        let font = FontId {
            size: FONT_SIZE,
            family: FONT_FAMILY,
        };
        style.override_font_id = Some(font);
        ctx.set_global_style(style);

        Default::default()
    }

    /// Whether an autoclick run is currently in progress.
    pub fn is_autoclicking(&self) -> bool {
        matches!(self.mode, InteractionMode::Autoclicking)
    }

    /// Whether a button is currently held down via click-and-hold.
    pub fn is_holding(&self) -> bool {
        matches!(self.mode, InteractionMode::Holding)
    }

    /// Whether interactive widgets should be disabled: an autoclick run or a
    /// click-and-hold is in progress, or the hotkeys window is open.
    pub fn is_busy(&self) -> bool {
        self.is_autoclicking() || self.is_holding() || self.hotkey_window_open
    }

    /// Whether the app is idle: not autoclicking and not in any key-capture or
    /// coordinate-setting mode.
    pub fn is_idle(&self) -> bool {
        matches!(self.mode, InteractionMode::Idle)
    }

    /// Disable the remaining widgets in `ui`'s current container while the app
    /// [`is_busy`](Self::is_busy). egui's `disable()` is container-scoped and
    /// permanent, so this mirrors the inline guard it replaces.
    pub fn disable_if_busy(&self, ui: &mut egui::Ui) {
        if self.is_busy() {
            ui.disable();
        }
    }

    /// Label for the start/stop autoclick button, e.g. `🖱 START (F6)`,
    /// `🖱 STOP (F6)`, or `🖱 START` when no hotkey is set.
    pub fn autoclick_button_label(&self) -> String {
        let verb = if self.is_autoclicking() {
            "STOP"
        } else {
            "START"
        };
        match self.key_autoclick {
            Some(hotkey) => format!("🖱 {verb} ({hotkey})"),
            None => format!("🖱 {verb}"),
        }
    }

    /// Label for the click-and-hold button, e.g. `HOLD (F7)`,
    /// `RELEASE (F7)`, or `HOLD` when no hotkey is set.
    pub fn hold_button_label(&self) -> String {
        let verb = if self.is_holding() { "RELEASE" } else { "HOLD" };
        match self.key_hold {
            Some(hotkey) => format!("{verb} ({hotkey})"),
            None => verb.to_string(),
        }
    }

    /// Sanitize all numeric input strings in place: the time, amount, and
    /// speed fields accept digits only; the X/Y coordinate fields accept a
    /// signed integer.
    pub fn sanitize_inputs(&mut self) {
        sanitize_string(&mut self.hr_str, INPUT_LEN_TIME);
        sanitize_string(&mut self.min_str, INPUT_LEN_TIME);
        sanitize_string(&mut self.sec_str, INPUT_LEN_TIME);
        sanitize_string(&mut self.ms_str, INPUT_LEN_TIME);
        sanitize_string(&mut self.click_amount_str, INPUT_LEN_TIME);
        sanitize_i64_string(&mut self.click_x_str, INPUT_LEN_COORD);
        sanitize_i64_string(&mut self.click_y_str, INPUT_LEN_COORD);
        sanitize_string(&mut self.speed_min_str, INPUT_LEN_TIME);
        sanitize_string(&mut self.speed_max_str, INPUT_LEN_TIME);
    }

    /// Total click interval in milliseconds, parsed from the hr/min/sec/ms fields.
    pub fn parsed_interval_ms(&self) -> u64 {
        interval_ms(
            self.hr_str.parse().unwrap_or_default(),
            self.min_str.parse().unwrap_or_default(),
            self.sec_str.parse().unwrap_or_default(),
            self.ms_str.parse().unwrap_or_default(),
        )
    }

    /// Humanlike glide speed range in px/s, parsed from the speed fields.
    /// Empty input falls back to the defaults; the minimum is kept at least
    /// 1 px/s and the maximum at least the minimum, so the range is never
    /// empty or zero.
    pub fn parsed_speed_range(&self) -> (f64, f64) {
        let min = self
            .speed_min_str
            .parse()
            .unwrap_or(MOUSE_TWEEN_SPEED_MIN_PX_S)
            .max(1.0);
        let max = self
            .speed_max_str
            .parse()
            .unwrap_or(MOUSE_TWEEN_SPEED_MAX_PX_S)
            .max(min);
        (min, max)
    }

    /// Number of clicks to perform; `0` means "click forever".
    pub fn parsed_click_amount(&self) -> u64 {
        self.click_amount_str.parse().unwrap_or_default()
    }

    /// Target click coordinates, parsed from the X/Y fields.
    pub fn parsed_click_coord(&self) -> (f64, f64) {
        (
            self.click_x_str.parse().unwrap_or_default(),
            self.click_y_str.parse().unwrap_or_default(),
        )
    }

    /// Enter the coordinate setting mode
    ///
    /// # Arguments
    ///
    /// * `ctx` - The ctx to manipulate
    pub fn enter_coordinate_setting(&mut self, ctx: &egui::Context) {
        self.mode = InteractionMode::SettingCoord;
        self.window_position =
            ctx.input(|input_state| input_state.viewport().outer_rect.unwrap().min);
        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(400f32, 30f32)));
        ctx.send_viewport_cmd(egui::ViewportCommand::Decorations(false));
    }

    /// Make frame follow cursor with an offset
    ///
    /// # Arguments
    ///
    /// * `ctx` - The ctx to set the window position on
    pub fn follow_cursor(&mut self, ctx: &egui::Context) {
        let offset = egui::Vec2 { x: 15f32, y: 15f32 };
        let (click_x, click_y) = self.parsed_click_coord();
        ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(
            egui::pos2(click_x as f32, click_y as f32) + offset,
        ));
    }

    /// Exit the coordinate setting mode
    ///
    /// # Arguments
    ///
    /// * `ctx` - The ctx to manipulate
    pub fn exit_coordinate_setting(&mut self, ctx: &egui::Context) {
        ctx.send_viewport_cmd(egui::ViewportCommand::Decorations(true));
        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(
            WINDOW_WIDTH,
            WINDOW_HEIGHT,
        )));
        ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(self.window_position));

        self.mode = InteractionMode::Idle;
        self.click_position = ClickPosition::Coord;
    }

    /// Start the autoclicking process
    ///
    /// # Arguments
    ///
    /// * `negative_click_start_offset` - The offset to start the click counter at
    pub fn start_autoclick(&mut self, negative_click_start_offset: u64) {
        self.click_counter = 0u64;
        self.mode = InteractionMode::Autoclicking;
        self.rng_thread = rng();

        self.last_now = Instant::now()
            .checked_sub(Duration::from_millis(negative_click_start_offset))
            .unwrap();
    }

    /// Press the currently selected button down and hold it.
    pub fn start_hold(&mut self) {
        if self.click_position == ClickPosition::Coord {
            move_mouse_to(
                self.app_mode,
                self.parsed_click_coord(),
                self.mouse.coords,
                self.parsed_speed_range(),
                &mut self.rng_thread,
            );
        }
        press_button(self.click_btn);
        self.held_button = Some(self.click_btn);
        self.mode = InteractionMode::Holding;
    }

    /// Release the held button (if any) and return to idle.
    pub fn stop_hold(&mut self) {
        if let Some(button) = self.held_button.take() {
            release_button(button);
        }
        self.mode = InteractionMode::Idle;
    }
}

impl Drop for RustyAutoClickerApp {
    /// Release any button still held when the app shuts down, so quitting
    /// mid-hold never leaves a button stuck pressed at the OS level. (Does not
    /// run on a release-build panic, since `panic = "abort"`.)
    fn drop(&mut self) {
        if let Some(button) = self.held_button.take() {
            release_button(button);
        }
    }
}
