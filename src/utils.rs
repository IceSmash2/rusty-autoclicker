use std::{env, thread, time::Duration};

use eframe::emath::Numeric;
use rand::{RngExt, prelude::ThreadRng};
use rdev::{EventType, SimulateError, simulate};
use sanitizer::prelude::StringSanitizer;

use crate::{
    defines::{
        APP_ICON, DURATION_CLICK_MAX, DURATION_CLICK_MIN, DURATION_DOUBLE_CLICK_MAX,
        DURATION_DOUBLE_CLICK_MIN, MOUSE_STEP_NEG_X, MOUSE_STEP_NEG_Y, MOUSE_STEP_POS_X,
        MOUSE_STEP_POS_Y,
    },
    types::{AppMode, ClickButton, ClickInfo, ClickPosition},
};

/// Load icon from memory and return it
pub fn load_icon() -> eframe::egui::IconData {
    let (icon_rgba, icon_width, icon_height) = {
        let image = image::load_from_memory(APP_ICON)
            .expect("Failed to open icon path")
            .into_rgba8();

        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };

    eframe::egui::IconData {
        rgba: icon_rgba,
        width: icon_width,
        height: icon_height,
    }
}

/// Sanitize string
///
/// # Arguments
///
/// * `string` - String to sanitize
/// * `max_length` - Maximum length of string
pub fn sanitize_string(string: &mut String, max_length: usize) {
    // Accept numeric only
    let s_slice = string.as_str();
    let mut sanitizer = StringSanitizer::from(s_slice);
    sanitizer.numeric();
    *string = sanitizer.get();

    // Remove leading 0
    while string.len() > 1 && string.starts_with('0') {
        string.remove(0);
    }

    truncate_string(string, max_length);
}

/// Sanitize string of expected i64 type
///
/// # Arguments
///
/// * `string` - String to sanitize
/// * `max_length` - Maximum length of string
pub fn sanitize_i64_string(string: &mut String, max_length: usize) {
    // Remove leading & trailing whitespaces
    // Parse to i64 or return default of 0
    *string = string.trim().parse::<i64>().unwrap_or_default().to_string();

    truncate_string(string, max_length);
}

/// Truncate string to specified length
///
/// # Arguments
///
/// * `string` - String to be truncated
/// * `max_length` - Maximum length of string
fn truncate_string(string: &mut String, max_length: usize) {
    // Allow max size of `max_length` characters
    if string.len() >= max_length {
        string.truncate(max_length)
    };
}

/// Click interval in milliseconds from the hour/minute/second/millisecond values.
pub const fn interval_ms(hr: u64, min: u64, sec: u64, ms: u64) -> u64 {
    (hr * 3_600_000) + (min * 60_000) + (sec * 1_000) + ms
}

/// Mouse-movement delay in milliseconds from the second/millisecond values.
pub const fn movement_delay_ms(sec: u64, ms: u64) -> u64 {
    (sec * 1_000) + ms
}

/// Send the simulated event (`rdev` crate)
///
/// # Arguments
///
/// * `event_type` - The event type to simulate
fn send(event_type: &EventType) {
    match simulate(event_type) {
        Ok(()) => (),
        Err(SimulateError) => {
            println!("We could not send {event_type:?}");
        }
    }

    // Let the OS catchup (at least MacOS)
    if env::consts::OS == "macos" {
        thread::sleep(Duration::from_millis(20u64));
    }
}

/// [Humanlike] Move the mouse step-by-step from `start_coords` toward `click_coord`,
/// pausing between steps.
///
/// # Arguments
///
/// * `click_coord` - The click coordinates
/// * `start_coords` - The starting mouse coordinates
/// * `movement_delay_in_ms` - The delay between mouse movements in milliseconds
fn move_to(click_coord: (f64, f64), start_coords: (f64, f64), movement_delay_in_ms: u64) {
    // Move mouse slowly to saved coordinates if requested
    let mut current_x = start_coords.0;
    let mut current_y = start_coords.1;
    loop {
        // horizontal movement: determine whether we need to move left, right or not at all
        let delta_x: f64 = if current_x < click_coord.0 {
            MOUSE_STEP_POS_X.min(click_coord.0 - current_x)
        } else if current_x > click_coord.0 {
            MOUSE_STEP_NEG_X.max(click_coord.0 - current_x)
        } else {
            0.0
        };

        // vertical movement: determine whether we need to move up, down or not at all
        let delta_y: f64 = if current_y < click_coord.1 {
            MOUSE_STEP_POS_Y.min(click_coord.1 - current_y)
        } else if current_y > click_coord.1 {
            MOUSE_STEP_NEG_Y.max(click_coord.1 - current_y)
        } else {
            0.0
        };

        current_x += delta_x;
        current_y += delta_y;

        #[cfg(debug_assertions)]
        println!("Moving by {delta_x:?} / {delta_y:?}, new pos: {current_x:?} / {current_y:?}");
        send(&EventType::MouseMove {
            x: current_x,
            y: current_y,
        });

        thread::sleep(Duration::from_millis(movement_delay_in_ms));
        if current_x == click_coord.0 && current_y == click_coord.1 {
            return;
        }
    }
}

/// Move the mouse to `click_coord`, honoring the app mode: an instant jump in
/// [`Bot`](AppMode::Bot), or a step-wise humanlike glide in
/// [`Humanlike`](AppMode::Humanlike) (skipped when already at the target).
/// Shared by autoclick and click-and-hold.
///
/// # Arguments
///
/// * `app_mode` - The app mode
/// * `click_coord` - The target coordinates
/// * `start_coords` - The current mouse coordinates (humanlike start point)
/// * `movement_delay_in_ms` - The delay between mouse movements in milliseconds
pub fn move_mouse_to(
    app_mode: AppMode,
    click_coord: (f64, f64),
    start_coords: (i32, i32),
    movement_delay_in_ms: u64,
) {
    match app_mode {
        AppMode::Bot => send(&EventType::MouseMove {
            x: click_coord.0,
            y: click_coord.1,
        }),
        AppMode::Humanlike => {
            let start = (start_coords.0.to_f64(), start_coords.1.to_f64());
            // only move if start pos and click pos are not identical
            if click_coord.0 != start.0 || click_coord.1 != start.1 {
                move_to(click_coord, start, movement_delay_in_ms);
            }
        }
    }
}

/// Press the button/key down without releasing it (used by click-and-hold).
pub fn press_button(button: ClickButton) {
    match button {
        ClickButton::Mouse(button) => send(&EventType::ButtonPress(button)),
        ClickButton::Key(key) => send(&EventType::KeyPress(key)),
    }
}

/// Release a previously pressed button/key (used by click-and-hold).
pub fn release_button(button: ClickButton) {
    match button {
        ClickButton::Mouse(button) => send(&EventType::ButtonRelease(button)),
        ClickButton::Key(key) => send(&EventType::KeyRelease(key)),
    }
}

fn click_once(button: ClickButton, hold: Option<Duration>) {
    press_button(button);
    if let Some(hold) = hold {
        thread::sleep(hold);
    }
    release_button(button);
}

/// Autoclick the mouse
///
/// # Arguments
///
/// * `app_mode` - The app mode
/// * `click_info` - The click information
/// * `mouse_coord` - The mouse coordinates
/// * `movement_delay_in_ms` - The delay between mouse movements in milliseconds
/// * `rng_thread` - The random number generator thread
pub fn autoclick(
    app_mode: AppMode,
    click_info: ClickInfo,
    mouse_coord: (i32, i32),
    movement_delay_in_ms: u64,
    mut rng_thread: ThreadRng,
) {
    // Number of press/release cycles required
    let run_amount = click_info.click_type.run_count();

    // Autoclick as fast as possible
    if app_mode == AppMode::Bot {
        for _n in 1..=run_amount {
            // Move mouse to saved coordinates if requested
            if click_info.click_position == ClickPosition::Coord {
                move_mouse_to(
                    app_mode,
                    click_info.click_coord,
                    mouse_coord,
                    movement_delay_in_ms,
                );
            }
            click_once(click_info.click_btn, None);
        }
    // Autoclick to emulate a humanlike clicks
    } else if app_mode == AppMode::Humanlike {
        // move to target
        #[cfg(debug_assertions)]
        println!(
            "Moving from {:?}/{:?} towards: {:?}/{:?}",
            mouse_coord.0.to_f64(),
            mouse_coord.1.to_f64(),
            click_info.click_coord.0,
            click_info.click_coord.1
        );

        // perform clicks
        for n in 1..=run_amount {
            // Sleep between clicks
            if n % 2 == 0 {
                thread::sleep(Duration::from_millis(
                    rng_thread.random_range(DURATION_DOUBLE_CLICK_MIN..DURATION_DOUBLE_CLICK_MAX),
                ));
            }

            // Move mouse to saved coordinates if requested
            if click_info.click_position == ClickPosition::Coord {
                move_mouse_to(
                    app_mode,
                    click_info.click_coord,
                    mouse_coord,
                    movement_delay_in_ms,
                );
            }

            // Press, hold for a randomized human-like duration, then release
            let hold = Duration::from_millis(
                rng_thread.random_range(DURATION_CLICK_MIN..DURATION_CLICK_MAX),
            );
            click_once(click_info.click_btn, Some(hold));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_string_keeps_digits_only() {
        let mut s = String::from("1a2b3");
        sanitize_string(&mut s, 10);
        assert_eq!(s, "123");
    }

    #[test]
    fn sanitize_string_strips_leading_zeros() {
        let mut s = String::from("007");
        sanitize_string(&mut s, 10);
        assert_eq!(s, "7");
    }

    #[test]
    fn sanitize_string_keeps_a_single_zero() {
        let mut s = String::from("0");
        sanitize_string(&mut s, 10);
        assert_eq!(s, "0");
    }

    #[test]
    fn sanitize_string_truncates_to_max_length() {
        let mut s = String::from("123456789");
        sanitize_string(&mut s, 5);
        assert_eq!(s, "12345");
    }

    #[test]
    fn sanitize_i64_string_trims_and_keeps_sign() {
        let mut s = String::from("  -5 ");
        sanitize_i64_string(&mut s, 7);
        assert_eq!(s, "-5");
    }

    #[test]
    fn sanitize_i64_string_falls_back_to_zero_on_garbage() {
        let mut s = String::from("abc");
        sanitize_i64_string(&mut s, 7);
        assert_eq!(s, "0");
    }

    #[test]
    fn truncate_string_trims_when_too_long() {
        let mut s = String::from("123456");
        truncate_string(&mut s, 5);
        assert_eq!(s, "12345");
    }

    #[test]
    fn truncate_string_leaves_short_strings_untouched() {
        let mut s = String::from("12");
        truncate_string(&mut s, 5);
        assert_eq!(s, "12");
    }

    #[test]
    fn interval_ms_combines_units() {
        assert_eq!(interval_ms(0, 0, 0, 100), 100);
        assert_eq!(interval_ms(1, 0, 0, 0), 3_600_000);
        assert_eq!(interval_ms(0, 1, 1, 1), 61_001);
    }

    #[test]
    fn movement_delay_ms_combines_units() {
        assert_eq!(movement_delay_ms(0, 20), 20);
        assert_eq!(movement_delay_ms(1, 0), 1_000);
        assert_eq!(movement_delay_ms(2, 5), 2_005);
    }
}
