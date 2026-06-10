use device_query::Keycode;
use eframe::egui::FontFamily;

pub const APP_NAME: &str = "Rusty AutoClicker";
pub const APP_ICON: &[u8] = include_bytes!("../assets/icon-64x64.ico");

// Font
pub const FONT_SIZE: f32 = 12.0;
pub const FONT_FAMILY: FontFamily = FontFamily::Monospace;

// dimensions of main window
pub const WINDOW_WIDTH: f32 = 550.0;
pub const WINDOW_HEIGHT: f32 = 341.0;

// ranges for click durations
pub const DURATION_CLICK_MIN: u64 = 20;
pub const DURATION_CLICK_MAX: u64 = 40;
pub const DURATION_DOUBLE_CLICK_MIN: u64 = 30;
pub const DURATION_DOUBLE_CLICK_MAX: u64 = 60;

// humanlike mouse tweening
pub const MOUSE_TWEEN_STEP_PX: f64 = 10.0; // approx. path distance per step
pub const MOUSE_TWEEN_MIN_STEPS: u64 = 4; // floor so short moves still glide
pub const MOUSE_TWEEN_CURVE_RATIO_MIN: f64 = 0.05; // curve bow, fraction of distance
pub const MOUSE_TWEEN_CURVE_RATIO_MAX: f64 = 0.18;
pub const MOUSE_TWEEN_CURVE_MAX_PX: f64 = 120.0; // bow cap for long moves
pub const MOUSE_TWEEN_TREMOR_PX: f64 = 1.5; // hand tremor amplitude
pub const MOUSE_TWEEN_TREMOR_DIST_THRESHOLD_PX: f64 = 480.0; // far moves allow more mid-tween tremors
pub const MOUSE_TWEEN_TREMOR_MAX_NEAR: u64 = 1; // max mid-tween tremors below threshold
pub const MOUSE_TWEEN_TREMOR_MAX_FAR: u64 = 2; // max mid-tween tremors at/above threshold
pub const MOUSE_TWEEN_DELAY_JITTER_FRAC: f64 = 0.5; // per-step delay jitter: ±50%
pub const MOUSE_TWEEN_SPEED_MIN_PX_S: f64 = 1500.0; // default glide speed range
pub const MOUSE_TWEEN_SPEED_MAX_PX_S: f64 = 4000.0;

// Default input values
pub const DEFAULT_HR_STR: &str = "0";
pub const DEFAULT_MIN_STR: &str = "0";
pub const DEFAULT_SEC_STR: &str = "0";
pub const DEFAULT_MS_STR: &str = "100";
pub const DEFAULT_CLICK_AMOUNT_STR: &str = "0";
pub const DEFAULT_CLICK_X_STR: &str = "0";
pub const DEFAULT_CLICK_Y_STR: &str = "0";

// Maximum lengths for sanitized numeric inputs
pub const INPUT_LEN_TIME: usize = 5;
pub const INPUT_LEN_COORD: usize = 7;

// Hotkeys
pub const HOTKEY_AUTOCLICK: Option<Keycode> = Some(Keycode::F6);
pub const HOTKEY_SET_COORD: Option<Keycode> = Some(Keycode::Escape);
pub const HOTKEY_HOLD: Option<Keycode> = Some(Keycode::F7);
