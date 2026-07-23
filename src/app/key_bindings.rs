use crossterm::event::KeyCode;

#[derive(Debug, Clone)]
pub(super) struct KeyBindings {
    pub(super) up: KeyCode,
    pub(super) down: KeyCode,
    pub(super) left: KeyCode,
    pub(super) right: KeyCode,
    pub(super) pause: KeyCode,
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            up: KeyCode::Up,
            down: KeyCode::Down,
            left: KeyCode::Left,
            right: KeyCode::Right,
            pause: KeyCode::Char(' '),
        }
    }
}

pub(super) fn key_label(code: KeyCode) -> String {
    match code {
        KeyCode::Up => "Up".to_string(),
        KeyCode::Down => "Down".to_string(),
        KeyCode::Left => "Left".to_string(),
        KeyCode::Right => "Right".to_string(),
        KeyCode::Char(' ') => "Space".to_string(),
        KeyCode::Char(c) => c.to_ascii_uppercase().to_string(),
        KeyCode::Enter => "Enter".to_string(),
        KeyCode::Esc => "Esc".to_string(),
        other => format!("{other:?}"),
    }
}
