#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum MenuScreen {
    Main,
    Settings,
    Controls,
    Credits,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CaptureKey {
    Up,
    Down,
    Left,
    Right,
    Pause,
}

impl CaptureKey {
    pub(super) fn label(self) -> &'static str {
        match self {
            CaptureKey::Up => "Key Up",
            CaptureKey::Down => "Key Down",
            CaptureKey::Left => "Key Left",
            CaptureKey::Right => "Key Right",
            CaptureKey::Pause => "Key Pause",
        }
    }
}

pub(super) fn settings_menu_len() -> usize {
    7
}

pub(super) fn controls_menu_len() -> usize {
    7
}

pub(super) fn pause_menu_len() -> usize {
    4
}
