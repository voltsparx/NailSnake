use crossterm::event::KeyCode;
use nailsnake::config::PersistedKeyBindings;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Action {
    Up,
    Down,
    Left,
    Right,
    Pause,
    Restart,
    Confirm,
    Back,
    Help,
    Quit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct KeyBindings {
    pub(super) up: KeyCode,
    pub(super) down: KeyCode,
    pub(super) left: KeyCode,
    pub(super) right: KeyCode,
    pub(super) pause: KeyCode,
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self::from_persisted(&PersistedKeyBindings::default())
    }
}

impl KeyBindings {
    pub(super) fn from_persisted(bindings: &PersistedKeyBindings) -> Self {
        let defaults = PersistedKeyBindings::default();
        Self {
            up: parse_key(&bindings.up).unwrap_or_else(|| parse_key(&defaults.up).unwrap()),
            down: parse_key(&bindings.down).unwrap_or_else(|| parse_key(&defaults.down).unwrap()),
            left: parse_key(&bindings.left).unwrap_or_else(|| parse_key(&defaults.left).unwrap()),
            right: parse_key(&bindings.right)
                .unwrap_or_else(|| parse_key(&defaults.right).unwrap()),
            pause: parse_key(&bindings.pause)
                .unwrap_or_else(|| parse_key(&defaults.pause).unwrap()),
        }
    }

    pub(super) fn persisted(&self) -> PersistedKeyBindings {
        PersistedKeyBindings {
            up: key_label(self.up),
            down: key_label(self.down),
            left: key_label(self.left),
            right: key_label(self.right),
            pause: key_label(self.pause),
        }
    }

    pub(super) fn action_for(&self, code: KeyCode) -> Option<Action> {
        let code = normalize(code);
        if code == normalize(self.up) {
            Some(Action::Up)
        } else if code == normalize(self.down) {
            Some(Action::Down)
        } else if code == normalize(self.left) {
            Some(Action::Left)
        } else if code == normalize(self.right) {
            Some(Action::Right)
        } else if code == normalize(self.pause) {
            Some(Action::Pause)
        } else {
            match code {
                KeyCode::Enter => Some(Action::Confirm),
                KeyCode::Esc => Some(Action::Back),
                KeyCode::Char('r') => Some(Action::Restart),
                KeyCode::Char('h') | KeyCode::Char('?') => Some(Action::Help),
                KeyCode::Char('q') => Some(Action::Quit),
                _ => None,
            }
        }
    }

    pub(super) fn conflict(&self, action: Action, code: KeyCode) -> Option<Action> {
        self.bindings()
            .into_iter()
            .find_map(|(candidate, existing)| {
                (candidate != action && normalize(existing) == normalize(code)).then_some(candidate)
            })
    }

    pub(super) fn assign(&mut self, action: Action, code: KeyCode) {
        match action {
            Action::Up => self.up = normalize(code),
            Action::Down => self.down = normalize(code),
            Action::Left => self.left = normalize(code),
            Action::Right => self.right = normalize(code),
            Action::Pause => self.pause = normalize(code),
            _ => {}
        }
    }

    fn bindings(&self) -> [(Action, KeyCode); 5] {
        [
            (Action::Up, self.up),
            (Action::Down, self.down),
            (Action::Left, self.left),
            (Action::Right, self.right),
            (Action::Pause, self.pause),
        ]
    }
}

pub(super) fn key_label(code: KeyCode) -> String {
    match normalize(code) {
        KeyCode::Up => "Up".into(),
        KeyCode::Down => "Down".into(),
        KeyCode::Left => "Left".into(),
        KeyCode::Right => "Right".into(),
        KeyCode::Char(' ') => "Space".into(),
        KeyCode::Char(c) => c.to_ascii_uppercase().to_string(),
        KeyCode::Enter => "Enter".into(),
        KeyCode::Esc => "Esc".into(),
        other => format!("{other:?}"),
    }
}

fn parse_key(value: &str) -> Option<KeyCode> {
    match value.trim().to_ascii_lowercase().as_str() {
        "up" => Some(KeyCode::Up),
        "down" => Some(KeyCode::Down),
        "left" => Some(KeyCode::Left),
        "right" => Some(KeyCode::Right),
        "space" => Some(KeyCode::Char(' ')),
        "enter" => Some(KeyCode::Enter),
        "esc" | "escape" => Some(KeyCode::Esc),
        value if value.chars().count() == 1 => value.chars().next().map(KeyCode::Char),
        _ => None,
    }
}

fn normalize(code: KeyCode) -> KeyCode {
    match code {
        KeyCode::Char(c) => KeyCode::Char(c.to_ascii_lowercase()),
        other => other,
    }
}

pub(super) fn action_label(action: Action) -> &'static str {
    match action {
        Action::Up => "Up",
        Action::Down => "Down",
        Action::Left => "Left",
        Action::Right => "Right",
        Action::Pause => "Pause",
        Action::Restart => "Restart",
        Action::Confirm => "Confirm",
        Action::Back => "Back",
        Action::Help => "Help",
        Action::Quit => "Quit",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bindings_round_trip_through_persistence_format() {
        let keys = KeyBindings::default();
        assert_eq!(KeyBindings::from_persisted(&keys.persisted()), keys);
    }
    #[test]
    fn duplicate_binding_is_reported() {
        let keys = KeyBindings::default();
        assert_eq!(keys.conflict(Action::Left, KeyCode::Up), Some(Action::Up));
    }
}
