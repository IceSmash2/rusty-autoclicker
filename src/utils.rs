use std::{env, thread, time::Duration};

use eframe::emath::Numeric;
use rand::{RngExt, prelude::ThreadRng};
use rdev::{EventType, SimulateError, simulate};
use sanitizer::prelude::StringSanitizer;

use crate::{
    defines::{
        APP_ICON, DURATION_CLICK_MAX, DURATION_CLICK_MIN, DURATION_DOUBLE_CLICK_MAX,
        DURATION_DOUBLE_CLICK_MIN, MOUSE_TWEEN_CURVE_MAX_PX, MOUSE_TWEEN_CURVE_RATIO_MAX,
        MOUSE_TWEEN_CURVE_RATIO_MIN, MOUSE_TWEEN_DELAY_JITTER_FRAC, MOUSE_TWEEN_MIN_STEPS,
        MOUSE_TWEEN_STEP_PX, MOUSE_TWEEN_TREMOR_DIST_THRESHOLD_PX, MOUSE_TWEEN_TREMOR_MAX_FAR,
        MOUSE_TWEEN_TREMOR_MAX_NEAR, MOUSE_TWEEN_TREMOR_PX,
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

    // Let the OS catchup (at least MacOS); mouse moves are exempt, as the
    // extra delay would cap the humanlike glide speed
    if env::consts::OS == "macos" && !matches!(event_type, EventType::MouseMove { .. }) {
        thread::sleep(Duration::from_millis(20u64));
    }
}

/// Evaluate a cubic Bezier curve at parameter `t` in [0, 1].
fn cubic_bezier(
    p0: (f64, f64),
    p1: (f64, f64),
    p2: (f64, f64),
    p3: (f64, f64),
    t: f64,
) -> (f64, f64) {
    let u = 1.0 - t;
    let w0 = u * u * u;
    let w1 = 3.0 * u * u * t;
    let w2 = 3.0 * u * t * t;
    let w3 = t * t * t;
    (
        w0 * p0.0 + w1 * p1.0 + w2 * p2.0 + w3 * p3.0,
        w0 * p0.1 + w1 * p1.1 + w2 * p2.1 + w3 * p3.1,
    )
}

/// Smoothstep ease-in-out: slow start, fast middle, slow end.
fn ease_in_out(t: f64) -> f64 {
    t * t * (3.0 - 2.0 * t)
}

/// Maximum number of extra mid-tween hand tremors for a glide of `distance` px.
fn mid_tremor_max(distance: f64) -> u64 {
    if distance < MOUSE_TWEEN_TREMOR_DIST_THRESHOLD_PX {
        MOUSE_TWEEN_TREMOR_MAX_NEAR
    } else {
        MOUSE_TWEEN_TREMOR_MAX_FAR
    }
}

/// [Humanlike] Glide the mouse from `start_coords` to `click_coord` along a
/// randomized cubic Bezier curve with ease-in-out timing and occasional hand
/// tremor (at the start and end of the glide, plus up to one or two random
/// mid-tween steps depending on distance), mimicking human movement. The
/// final event lands exactly on `click_coord`.
///
/// # Arguments
///
/// * `click_coord` - The click coordinates
/// * `start_coords` - The starting mouse coordinates
/// * `speed_range` - The (min, max) glide speed in px/s; the actual speed is
///   randomized within it
/// * `rng_thread` - The random number generator
fn move_to(
    click_coord: (f64, f64),
    start_coords: (f64, f64),
    speed_range: (f64, f64),
    rng_thread: &mut ThreadRng,
) {
    let dx = click_coord.0 - start_coords.0;
    let dy = click_coord.1 - start_coords.1;
    let distance = dx.hypot(dy);
    if distance < f64::EPSILON {
        send(&EventType::MouseMove {
            x: click_coord.0,
            y: click_coord.1,
        });
        return;
    }

    let steps = ((distance / MOUSE_TWEEN_STEP_PX).ceil() as u64).max(MOUSE_TWEEN_MIN_STEPS);

    // Randomized control points: bow the curve perpendicular to the straight
    // line, independently per control point (yields C-arcs and S-curves)
    let perp = (-dy / distance, dx / distance);
    let bow = (distance
        * rng_thread.random_range(MOUSE_TWEEN_CURVE_RATIO_MIN..=MOUSE_TWEEN_CURVE_RATIO_MAX))
    .min(MOUSE_TWEEN_CURVE_MAX_PX);
    let offset1 = bow * rng_thread.random_range(-1.0..=1.0);
    let offset2 = bow * rng_thread.random_range(-1.0..=1.0);
    let p1 = (
        start_coords.0 + 0.30 * dx + perp.0 * offset1,
        start_coords.1 + 0.30 * dy + perp.1 * offset1,
    );
    let p2 = (
        start_coords.0 + 0.70 * dx + perp.0 * offset2,
        start_coords.1 + 0.70 * dy + perp.1 * offset2,
    );

    // Hand tremor at the start and end of the glide (the final step stays
    // exact), plus up to one (short move) or two (long move) tremors at
    // random mid-tween steps
    let mut tremor_steps = vec![1, steps - 1];
    for _ in 0..rng_thread.random_range(0..=mid_tremor_max(distance)) {
        tremor_steps.push(rng_thread.random_range(2..=steps - 2));
    }

    // Random glide speed within the configured range
    let speed = rng_thread.random_range(speed_range.0..=speed_range.1.max(speed_range.0));
    let step_delay_s = distance / speed / steps as f64;

    #[cfg(debug_assertions)]
    println!(
        "Tweening from {start_coords:?} to {click_coord:?}: distance {distance:.1}px, speed {speed:.0}px/s, {steps} steps, bow {bow:.1}px, tremors at {tremor_steps:?}"
    );

    for i in 1..=steps {
        let t = ease_in_out(i as f64 / steps as f64);
        let (x, y) = if i == steps {
            // land exactly on the target, a click follows
            click_coord
        } else {
            let (x, y) = cubic_bezier(start_coords, p1, p2, click_coord, t);
            if tremor_steps.contains(&i) {
                (
                    x + rng_thread.random_range(-MOUSE_TWEEN_TREMOR_PX..=MOUSE_TWEEN_TREMOR_PX),
                    y + rng_thread.random_range(-MOUSE_TWEEN_TREMOR_PX..=MOUSE_TWEEN_TREMOR_PX),
                )
            } else {
                (x, y)
            }
        };
        send(&EventType::MouseMove { x, y });

        if i < steps {
            thread::sleep(Duration::from_secs_f64(
                step_delay_s
                    * rng_thread.random_range(
                        (1.0 - MOUSE_TWEEN_DELAY_JITTER_FRAC)
                            ..=(1.0 + MOUSE_TWEEN_DELAY_JITTER_FRAC),
                    ),
            ));
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
/// * `speed_range` - The (min, max) humanlike glide speed in px/s
/// * `rng_thread` - The random number generator (humanlike tweening)
pub fn move_mouse_to(
    app_mode: AppMode,
    click_coord: (f64, f64),
    start_coords: (i32, i32),
    speed_range: (f64, f64),
    rng_thread: &mut ThreadRng,
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
                move_to(click_coord, start, speed_range, rng_thread);
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
/// * `speed_range` - The (min, max) humanlike glide speed in px/s
/// * `rng_thread` - The random number generator thread
pub fn autoclick(
    app_mode: AppMode,
    click_info: ClickInfo,
    mouse_coord: (i32, i32),
    speed_range: (f64, f64),
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
                    speed_range,
                    &mut rng_thread,
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
                    speed_range,
                    &mut rng_thread,
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
    fn cubic_bezier_hits_endpoints() {
        let p0 = (10.0, 20.0);
        let p1 = (50.0, -30.0);
        let p2 = (120.0, 80.0);
        let p3 = (200.0, 40.0);
        assert_eq!(cubic_bezier(p0, p1, p2, p3, 0.0), p0);
        assert_eq!(cubic_bezier(p0, p1, p2, p3, 1.0), p3);
    }

    #[test]
    fn cubic_bezier_midpoint_known_value() {
        // At t = 0.5 the weights are 1/8, 3/8, 3/8, 1/8
        let (x, y) = cubic_bezier((0.0, 0.0), (8.0, 0.0), (0.0, 8.0), (8.0, 8.0), 0.5);
        assert!((x - 4.0).abs() < 1e-9);
        assert!((y - 4.0).abs() < 1e-9);
    }

    #[test]
    fn mid_tremor_max_depends_on_distance() {
        assert_eq!(mid_tremor_max(50.0), 1);
        assert_eq!(mid_tremor_max(479.9), 1);
        assert_eq!(mid_tremor_max(480.0), 2);
        assert_eq!(mid_tremor_max(1500.0), 2);
    }

    #[test]
    fn ease_in_out_boundaries_and_symmetry() {
        assert_eq!(ease_in_out(0.0), 0.0);
        assert_eq!(ease_in_out(1.0), 1.0);
        assert!((ease_in_out(0.5) - 0.5).abs() < 1e-9);
        assert!(ease_in_out(0.25) < ease_in_out(0.5));
        assert!(ease_in_out(0.5) < ease_in_out(0.75));
    }
}
