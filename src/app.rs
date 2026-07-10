use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use device_query::{DeviceState, Keycode, MouseState};
use eframe::{egui, epaint::FontId};
use rand::{prelude::ThreadRng, rng};
use rdev::Button;

use crate::{
    defines::*,
    settings::{self, Settings},
    types::{AppMode, ClickButton, ClickPosition, ClickType, InteractionMode},
    utils::{
        interval_ms, move_mouse_to, press_button, release_button, sanitize_i64_string,
        sanitize_string,
    },
};

/// Live data the coordinate-picker's independent (deferred) viewport reads
/// every frame to know where to draw itself. Shared via `Arc<Mutex<..>>`
/// because `show_viewport_deferred` requires a `Send + Sync + 'static`
/// callback — unlike `show_viewport_immediate`, it cannot borrow `&mut
/// self` directly, since it is designed to run independently of the
/// call site's frame lifecycle (which is exactly what makes it safe to
/// drive from `logic()`, and safe under rapid F10 spam).
#[derive(Default, Clone, Copy)]
pub struct CoordPickerShared {
    pub pos: (i32, i32),
    pub confirm_key: Option<Keycode>,
}

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
    pub key_open_set_coord: Option<Keycode>,
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

    /// True only when THIS app auto-minimized the main window itself upon
    /// entering coordinate-setting mode (because it was visible at the
    /// time). Used so `exit_coordinate_setting` only un-minimizes the
    /// window if we're the ones who minimized it — if the window was
    /// already minimized before F10 was pressed, it correctly stays
    /// minimized after confirming coordinates too.
    pub auto_minimized_for_picker: bool,

    /// Last known inner window size [w, h] in logical pixels, for the MAIN
    /// window only. Updated every frame. The coordinate picker lives in its
    /// own viewport and never writes to this field, so it can never leak
    /// its own (much smaller) size into the persisted/main geometry.
    pub last_window_size: [f32; 2],

    /// Last known outer window position [x, y] in logical pixels, for the
    /// MAIN window only. Same "never touched by the picker" guarantee as
    /// `last_window_size` above.
    pub last_window_pos: egui::Pos2,

    // Key-down edge-detection flags
    pub key_pressed_autoclick: bool,
    pub key_pressed_open_set_coord: bool,
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

    /// Cached device-state poller — created once and reused every frame
    /// instead of being constructed anew in every `logic()` call.
    pub device_state: DeviceState,

    /// Shared with the coordinate-picker's deferred viewport (see
    /// `gui/windows.rs::show_coord_picker_viewport`).
    pub coord_picker_shared: Arc<Mutex<CoordPickerShared>>,
}

impl Default for RustyAutoClickerApp {
    fn default() -> Self {
        Self {
            hr_str: DEFAULT_HR_STR.to_owned(),
            min_str: DEFAULT_MIN_STR.to_owned(),
            sec_str: DEFAULT_SEC_STR.to_owned(),
            ms_str: DEFAULT_MS_STR.to_owned(),
            click_amount_str: DEFAULT_CLICK_AMOUNT_STR.to_owned(),
            click_x_str: DEFAULT_CLICK_X_STR.to_owned(),
            click_y_str: DEFAULT_CLICK_Y_STR.to_owned(),
            speed_min_str: MOUSE_TWEEN_SPEED_MIN_PX_S.to_string(),
            speed_max_str: MOUSE_TWEEN_SPEED_MAX_PX_S.to_string(),

            last_now: Instant::now(),
            frame_start: Instant::now(),

            click_counter: 0u64,

            key_autoclick: HOTKEY_AUTOCLICK,
            key_open_set_coord: HOTKEY_OPEN_SET_COORD,
            key_set_coord: HOTKEY_SET_COORD,
            key_hold: HOTKEY_HOLD,

            mode: InteractionMode::Idle,
            held_button: None,
            app_mode: AppMode::Bot,

            hotkey_window_open: false,
            auto_minimized_for_picker: false,
            last_window_size: [WINDOW_WIDTH, WINDOW_HEIGHT],
            last_window_pos: egui::pos2(WINDOW_DEFAULT_X, WINDOW_DEFAULT_Y),

            key_pressed_autoclick: false,
            key_pressed_open_set_coord: false,
            key_pressed_esc: false,
            key_pressed_hold: false,
            keys_pressed: None,

            mouse: MouseState::default(),

            click_btn: ClickButton::Mouse(Button::Left),
            click_type: ClickType::Single,
            click_position: ClickPosition::Mouse,

            rng_thread: rng(),
            device_state: DeviceState::new(),
            coord_picker_shared: Arc::new(Mutex::new(CoordPickerShared::default())),
        }
    }
}

impl RustyAutoClickerApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let ctx = &cc.egui_ctx;

        let mut style = (*ctx.global_style()).clone();
        style.override_font_id = Some(FontId { size: FONT_SIZE, family: FONT_FAMILY });
        ctx.set_global_style(style);

        let mut app = Self::default();

        let mut had_saved_position = false;

        if let Some(loaded) = settings::load_settings() {
            loaded.apply_to(&mut app);

            // Restore window size — sent as a viewport command here; the OS
            // will apply it before the first frame is painted. This is a
            // one-time startup restore, not a "resize" in the runtime sense
            // the user cares about, so it's exempt from the
            // "only user/Reset may resize" rule.
            if let Some([w, h]) = loaded.window_size {
                app.last_window_size = [w, h];
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(w, h)));
            }

            // Restore window position. We send this command here so the
            // OS can position the window before the first paint, eliminating
            // the startup flicker where the window briefly appears at the
            // default position.
            if let Some([x, y]) = loaded.window_position {
                let pos = egui::pos2(x, y);
                app.last_window_pos = pos;
                ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(pos));
                had_saved_position = true;
            }
        }

        // Fresh install / no saved position yet: center on the actual
        // screen resolution, same as the Reset button does, instead of the
        // old hardcoded WINDOW_DEFAULT_X/Y fallback. Still sent before the
        // first frame paints, so there's no flicker on first launch either.
        if !had_saved_position {
            if let Some(cmd) = egui::ViewportCommand::center_on_screen(ctx) {
                if let egui::ViewportCommand::OuterPosition(pos) = cmd {
                    app.last_window_pos = pos;
                }
                ctx.send_viewport_cmd(cmd);
            }
        }

        app
    }

    // -----------------------------------------------------------------------
    // Settings
    // -----------------------------------------------------------------------

    /// Write the current settings to disk immediately (Save button).
    pub fn save_settings_now(&self) {
        settings::save_settings(&Settings::from_app(self));
    }

    /// Reset every user-configurable setting to its default value,
    /// resize the window back to the default size, and move it to the
    /// default position. Persists immediately so the reset survives the next launch.
    ///
    /// This is one of exactly two places in the whole app allowed to resize
    /// the main window at runtime (the other being a manual drag by the
    /// user, which egui/the OS handles directly and never goes through this
    /// code at all).
    pub fn reset_to_defaults(&mut self, ctx: &egui::Context) {
        self.hr_str = DEFAULT_HR_STR.to_owned();
        self.min_str = DEFAULT_MIN_STR.to_owned();
        self.sec_str = DEFAULT_SEC_STR.to_owned();
        self.ms_str = DEFAULT_MS_STR.to_owned();
        self.click_amount_str = DEFAULT_CLICK_AMOUNT_STR.to_owned();
        self.click_x_str = DEFAULT_CLICK_X_STR.to_owned();
        self.click_y_str = DEFAULT_CLICK_Y_STR.to_owned();
        self.speed_min_str = MOUSE_TWEEN_SPEED_MIN_PX_S.to_string();
        self.speed_max_str = MOUSE_TWEEN_SPEED_MAX_PX_S.to_string();

        self.key_autoclick = HOTKEY_AUTOCLICK;
        self.key_open_set_coord = HOTKEY_OPEN_SET_COORD;
        self.key_set_coord = HOTKEY_SET_COORD;
        self.key_hold = HOTKEY_HOLD;

        self.click_btn = ClickButton::Mouse(Button::Left);
        self.click_type = ClickType::Single;
        self.click_position = ClickPosition::Mouse;
        self.app_mode = AppMode::Bot;

        // Resize back to the original window size and move it back to the
        // true center of the screen it's currently on. We use egui's own
        // `ViewportCommand::center_on_screen` helper rather than manually
        // reading `viewport().monitor_size` — that field is documented as
        // commonly `None`/unreliable depending on platform and timing, which
        // is the most likely reason centering silently fell back to
        // WINDOW_DEFAULT_X/Y in practice. `center_on_screen` is egui's own
        // built-in, more robust implementation of exactly this.
        // WINDOW_DEFAULT_X/Y remains as the last-resort fallback for the
        // rare case egui itself can't determine placement.
        self.last_window_size = [WINDOW_WIDTH, WINDOW_HEIGHT];
        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(
            WINDOW_WIDTH,
            WINDOW_HEIGHT,
        )));
        match egui::ViewportCommand::center_on_screen(ctx) {
            Some(cmd) => {
                if let egui::ViewportCommand::OuterPosition(pos) = cmd {
                    self.last_window_pos = pos;
                }
                ctx.send_viewport_cmd(cmd);
            }
            None => {
                let fallback = egui::pos2(WINDOW_DEFAULT_X, WINDOW_DEFAULT_Y);
                self.last_window_pos = fallback;
                ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(fallback));
            }
        }

        self.save_settings_now();
    }

    // -----------------------------------------------------------------------
    // State queries
    // -----------------------------------------------------------------------

    pub fn is_autoclicking(&self) -> bool { matches!(self.mode, InteractionMode::Autoclicking) }
    pub fn is_holding(&self) -> bool { matches!(self.mode, InteractionMode::Holding) }
    pub fn is_setting_coord(&self) -> bool { matches!(self.mode, InteractionMode::SettingCoord) }
    pub fn is_busy(&self) -> bool {
        self.is_autoclicking() || self.is_holding() || self.hotkey_window_open || self.is_setting_coord()
    }
    pub fn is_idle(&self) -> bool { matches!(self.mode, InteractionMode::Idle) }

    pub fn disable_if_busy(&self, ui: &mut egui::Ui) {
        if self.is_busy() { ui.disable(); }
    }

    // -----------------------------------------------------------------------
    // Labels
    // -----------------------------------------------------------------------

    pub fn autoclick_button_label(&self) -> String {
        let verb = if self.is_autoclicking() { "STOP" } else { "START" };
        match self.key_autoclick {
            Some(k) => format!("🖱 {verb} ({k})"),
            None => format!("🖱 {verb}"),
        }
    }

    pub fn hold_button_label(&self) -> String {
        let verb = if self.is_holding() { "RELEASE" } else { "HOLD" };
        match self.key_hold {
            Some(k) => format!("{verb} ({k})"),
            None => verb.to_string(),
        }
    }

    // -----------------------------------------------------------------------
    // Input sanitization / parsing
    // -----------------------------------------------------------------------

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

    pub fn parsed_interval_ms(&self) -> u64 {
        interval_ms(
            self.hr_str.parse().unwrap_or_default(),
            self.min_str.parse().unwrap_or_default(),
            self.sec_str.parse().unwrap_or_default(),
            self.ms_str.parse().unwrap_or_default(),
        )
    }

    pub fn parsed_speed_range(&self) -> (f64, f64) {
        let min = self.speed_min_str.parse().unwrap_or(MOUSE_TWEEN_SPEED_MIN_PX_S).max(1.0);
        let max = self.speed_max_str.parse().unwrap_or(MOUSE_TWEEN_SPEED_MAX_PX_S).max(min);
        (min, max)
    }

    pub fn parsed_click_amount(&self) -> u64 { self.click_amount_str.parse().unwrap_or_default() }

    pub fn parsed_click_coord(&self) -> (f64, f64) {
        (
            self.click_x_str.parse().unwrap_or_default(),
            self.click_y_str.parse().unwrap_or_default(),
        )
    }

    // -----------------------------------------------------------------------
    // Coordinate-setting mode
    // -----------------------------------------------------------------------
    //
    // The coordinate picker is ALWAYS rendered in its own independent egui
    // viewport (see `gui/windows.rs::show_coord_picker_viewport`) at a fixed
    // 500x32 size — it is never resized, in any code path, ever.
    //
    // The MAIN window itself is never resized here either. The only thing
    // this code does to the main window is minimize/restore it:
    //   * If the main window is VISIBLE when F10 fires, we minimize it so
    //     only the picker is on screen while picking, then restore it (un-
    //     minimize, back to its exact prior size/position — the OS remembers
    //     that) once coordinates are confirmed.
    //   * If the main window is ALREADY minimized when F10 fires, we leave
    //     it alone entirely — it stays minimized before, during, and after
    //     picking. No restore, no animation, no pop-up.
    // `auto_minimized_for_picker` is what makes this distinction: it's only
    // `true` when THIS code performed the minimize, so only then do we
    // reverse it on exit.

    /// Returns `true` when the OS reports the main window as minimized.
    pub fn is_window_minimized(ctx: &egui::Context) -> bool {
        ctx.input(|i| i.viewport().minimized).unwrap_or(false)
    }

    /// Enter coordinate-setting mode. If the main window is currently
    /// visible, minimize it (so only the fixed-size picker is on screen);
    /// if it's already minimized, leave it exactly as-is.
    pub fn enter_coordinate_setting(&mut self, ctx: &egui::Context) {
        if !self.is_idle() {
            return;
        }
        self.mode = InteractionMode::SettingCoord;

        if Self::is_window_minimized(ctx) {
            self.auto_minimized_for_picker = false;
        } else {
            self.auto_minimized_for_picker = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
        }
    }

    /// Exit coordinate-setting mode. Restores the main window from minimized
    /// only if `enter_coordinate_setting` was the one that minimized it.
    pub fn exit_coordinate_setting(&mut self, ctx: &egui::Context) {
        self.mode = InteractionMode::Idle;
        self.click_position = ClickPosition::Coord;

        if self.auto_minimized_for_picker {
            ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
        }
        self.auto_minimized_for_picker = false;
    }

    // -----------------------------------------------------------------------
    // Autoclick / hold
    // -----------------------------------------------------------------------

    pub fn start_autoclick(&mut self, negative_click_start_offset: u64) {
        self.click_counter = 0u64;
        self.mode = InteractionMode::Autoclicking;
        self.rng_thread = rng();
        self.last_now = Instant::now()
            .checked_sub(Duration::from_millis(negative_click_start_offset))
            .unwrap();
    }

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

    pub fn stop_hold(&mut self) {
        if let Some(button) = self.held_button.take() {
            release_button(button);
        }
        self.mode = InteractionMode::Idle;
    }
}

impl Drop for RustyAutoClickerApp {
    /// On close: release any held button, then write the final window
    /// geometry (size + position) to disk without touching other settings.
    fn drop(&mut self) {
        if let Some(button) = self.held_button.take() {
            release_button(button);
        }
        settings::save_window_geometry(
            self.last_window_size[0],
            self.last_window_size[1],
            self.last_window_pos.x,
            self.last_window_pos.y,
        );
    }
}
