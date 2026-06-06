use std::fmt;

use rdev::{Button, Key};

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum AppMode {
    Bot,
    Humanlike,
}

/// Mutually-exclusive interaction states; exactly one is active at a time,
/// which makes "two modes at once" unrepresentable.
#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum InteractionMode {
    Idle,
    Autoclicking,
    SettingCoord,
    SettingAutoclickKey,
    SettingSetCoordKey,
}

#[derive(PartialEq, Copy, Clone)]
pub struct ClickInfo {
    pub click_btn: ClickButton,
    pub click_coord: (f64, f64),
    pub click_position: ClickPosition,
    pub click_type: ClickType,
}

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum ClickPosition {
    Mouse,
    Coord,
}

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum ClickType {
    Single,
    Double,
}

impl ClickType {
    /// Number of press/release cycles performed per autoclick tick.
    pub const fn run_count(self) -> u8 {
        match self {
            ClickType::Single => 1,
            ClickType::Double => 2,
        }
    }
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum ClickButton {
    Mouse(Button),
    Key(Key),
}

impl fmt::Display for ClickButton {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClickButton::Mouse(button) => write!(f, "{button:?}"),
            ClickButton::Key(key) => write!(f, "{key:?}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_count_matches_click_type() {
        assert_eq!(ClickType::Single.run_count(), 1);
        assert_eq!(ClickType::Double.run_count(), 2);
    }
}
