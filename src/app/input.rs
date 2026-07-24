use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::menu_state::{
    controls_menu_len, pause_menu_len, settings_menu_len, CaptureKey, MenuScreen,
};
use super::App;
use nailsnake::game::{Direction, Game, GamePhase};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MainMenuAction {
    Continue,
    ArcadeMode,
    Settings,
    Help,
    Controls,
    Credits,
    Quit,
}

impl App {
    pub(super) fn handle_resize(&mut self, width: u16, height: u16) -> Result<()> {
        if width < nailsnake::platform::MIN_TERM_WIDTH
            || height < nailsnake::platform::MIN_TERM_HEIGHT
        {
            return Ok(());
        }

        self.terminal.clear()?;
        let (board_w, board_h) = super::terminal::board_dimensions(width, height);
        let phase = self.game.phase;

        if !self.game.resize(board_w, board_h) {
            self.game = Game::with_options(
                board_w,
                board_h,
                self.config.difficulty,
                self.config.wrap_walls,
                self.config.random_maze,
                self.config.custom_speed_ms,
            );
            self.game.phase = phase;
        } else {
            self.game.phase = phase;
        }
        Ok(())
    }

    pub(super) fn handle_key(&mut self, key: KeyEvent) -> Result<bool> {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return Ok(true);
        }

        if let Some(capture) = self.capture_key {
            if Self::is_capture_key_cancel(key) {
                self.capture_key = None;
                return Ok(false);
            }
            self.assign_key(capture, key.code);
            self.capture_key = None;
            return Ok(false);
        }

        if self.show_help {
            match key.code {
                KeyCode::Esc
                | KeyCode::Enter
                | KeyCode::Char('h')
                | KeyCode::Char('H')
                | KeyCode::Char('?') => self.show_help = false,
                KeyCode::Char('q') | KeyCode::Char('Q') if self.game.phase == GamePhase::Menu => {
                    return Ok(true);
                }
                _ => {}
            }
            return Ok(false);
        }

        if self.game.phase == GamePhase::Menu {
            return self.handle_main_menu_key(key);
        }

        if self.game.phase == GamePhase::Paused {
            return self.handle_pause_menu_key(key);
        }

        match key.code {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => return Ok(true),
            code if code == self.keys.pause || code == KeyCode::Enter => {
                self.game.toggle_pause();
                self.pause_menu_index = 0;
            }
            KeyCode::Char('h') | KeyCode::Char('H') | KeyCode::Char('?') => {
                self.show_help = true;
            }
            KeyCode::Char('r') | KeyCode::Char('R') => {
                self.game.reset();
                self.last_tick = std::time::Instant::now();
            }
            code if code == self.keys.up
                || code == KeyCode::Char('w')
                || code == KeyCode::Char('W') =>
            {
                self.game.set_direction(Direction::Up);
            }
            code if code == self.keys.down
                || code == KeyCode::Char('s')
                || code == KeyCode::Char('S') =>
            {
                self.game.set_direction(Direction::Down);
            }
            code if code == self.keys.left
                || code == KeyCode::Char('a')
                || code == KeyCode::Char('A') =>
            {
                self.game.set_direction(Direction::Left);
            }
            code if code == self.keys.right
                || code == KeyCode::Char('d')
                || code == KeyCode::Char('D') =>
            {
                self.game.set_direction(Direction::Right);
            }
            _ => {}
        }

        Ok(false)
    }

    fn handle_main_menu_key(&mut self, key: KeyEvent) -> Result<bool> {
        if self.menu_screen == MenuScreen::Settings {
            return self.handle_settings_key(key);
        }
        if self.menu_screen == MenuScreen::Controls {
            return self.handle_controls_key(key);
        }
        if self.menu_screen == MenuScreen::Credits {
            return self.handle_credits_key(key);
        }

        match key.code {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => return Ok(true),
            KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('W') => {
                self.main_menu_index = self.main_menu_index.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('S') => {
                self.main_menu_index = (self.main_menu_index + 1).min(self.main_menu_len() - 1);
            }
            KeyCode::Left
            | KeyCode::Right
            | KeyCode::Char('a')
            | KeyCode::Char('A')
            | KeyCode::Char('d')
            | KeyCode::Char('D') => {
                self.adjust_main_menu_option();
            }
            KeyCode::Enter => match self.main_menu_action(self.main_menu_index) {
                MainMenuAction::Continue => self.continue_game(),
                MainMenuAction::ArcadeMode => self.start_game(),
                MainMenuAction::Settings => self.menu_screen = MenuScreen::Settings,
                MainMenuAction::Help => self.show_help = true,
                MainMenuAction::Controls => self.menu_screen = MenuScreen::Controls,
                MainMenuAction::Credits => self.menu_screen = MenuScreen::Credits,
                MainMenuAction::Quit => return Ok(true),
            },
            KeyCode::Char('h') | KeyCode::Char('H') | KeyCode::Char('?') => self.show_help = true,
            _ => {}
        }

        Ok(false)
    }

    fn handle_settings_key(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => {
                self.menu_screen = MenuScreen::Main
            }
            KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('W') => {
                self.settings_index = self.settings_index.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('S') => {
                self.settings_index = (self.settings_index + 1).min(settings_menu_len() - 1);
            }
            KeyCode::Left | KeyCode::Char('a') | KeyCode::Char('A') => self.adjust_setting(false),
            KeyCode::Right | KeyCode::Char('d') | KeyCode::Char('D') => self.adjust_setting(true),
            KeyCode::Enter => {
                if self.settings_index == settings_menu_len() - 1 {
                    self.menu_screen = MenuScreen::Main;
                } else {
                    self.adjust_setting(true);
                }
            }
            _ => {}
        }
        Ok(false)
    }

    fn handle_controls_key(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => {
                self.menu_screen = MenuScreen::Main
            }
            KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('W') => {
                self.controls_index = self.controls_index.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('S') => {
                self.controls_index = (self.controls_index + 1).min(controls_menu_len() - 1);
            }
            KeyCode::Enter => match self.controls_index {
                0 => self.capture_key = Some(CaptureKey::Up),
                1 => self.capture_key = Some(CaptureKey::Down),
                2 => self.capture_key = Some(CaptureKey::Left),
                3 => self.capture_key = Some(CaptureKey::Right),
                4 => self.capture_key = Some(CaptureKey::Pause),
                5 => self.keys = super::key_bindings::KeyBindings::default(),
                6 => self.menu_screen = MenuScreen::Main,
                _ => {}
            },
            _ => {}
        }
        Ok(false)
    }

    fn handle_credits_key(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q') | KeyCode::Char('Q') => {
                self.menu_screen = MenuScreen::Main;
            }
            _ => {}
        }
        Ok(false)
    }

    fn handle_pause_menu_key(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => return Ok(true),
            KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('W') => {
                self.pause_menu_index = self.pause_menu_index.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('S') => {
                self.pause_menu_index = (self.pause_menu_index + 1).min(pause_menu_len() - 1);
            }
            KeyCode::Char(' ') => self.game.toggle_pause(),
            KeyCode::Enter => match self.pause_menu_index {
                0 => self.game.toggle_pause(),
                1 => {
                    self.game.reset();
                    self.game.phase = GamePhase::Running;
                    self.last_tick = std::time::Instant::now();
                }
                2 => self.show_help = true,
                3 => return Ok(true),
                _ => {}
            },
            KeyCode::Char('h') | KeyCode::Char('H') | KeyCode::Char('?') => self.show_help = true,
            _ => {}
        }

        Ok(false)
    }

    fn start_game(&mut self) {
        self.apply_config_to_game();
        let _ = self.config.clear_active_game();
        self.saved_game_available = false;
        self.game.reset();
        self.game.phase = GamePhase::Running;
        self.last_tick = std::time::Instant::now();
    }

    fn continue_game(&mut self) {
        if let Ok(Some(mut game)) = self.config.load_active_game() {
            game.phase = GamePhase::Paused;
            self.config.difficulty = game.difficulty;
            self.config.wrap_walls = game.wrap_walls;
            self.config.random_maze = game.random_maze;
            self.config.custom_speed_ms = game.custom_speed_ms;
            self.game = game;
            self.pause_menu_index = 0;
            self.last_tick = std::time::Instant::now();
        } else {
            self.saved_game_available = false;
            self.main_menu_index = self.main_menu_index.min(self.main_menu_len() - 1);
        }
    }

    fn adjust_main_menu_option(&mut self) {}

    fn adjust_setting(&mut self, increase: bool) {
        match self.settings_index {
            0 => {
                self.config.difficulty = self.config.difficulty.next();
                self.config.custom_speed_ms = self.config.difficulty.tick_ms();
            }
            1 => {
                if increase {
                    self.config.custom_speed_ms =
                        self.config.custom_speed_ms.saturating_sub(5).max(45);
                } else {
                    self.config.custom_speed_ms = (self.config.custom_speed_ms + 5).min(260);
                }
            }
            2 => self.config.wrap_walls = !self.config.wrap_walls,
            3 => self.config.random_maze = !self.config.random_maze,
            4 => self.config.snake_skin = self.config.snake_skin.next(),
            5 => self.config.show_grid = !self.config.show_grid,
            _ => {}
        }
        self.apply_config_to_game();
    }

    fn apply_config_to_game(&mut self) {
        self.game.difficulty = self.config.difficulty;
        self.game.wrap_walls = self.config.wrap_walls;
        self.game.random_maze = self.config.random_maze;
        self.game.custom_speed_ms = self.config.custom_speed_ms;
    }

    fn assign_key(&mut self, capture: CaptureKey, code: KeyCode) {
        match capture {
            CaptureKey::Up => self.keys.up = code,
            CaptureKey::Down => self.keys.down = code,
            CaptureKey::Left => self.keys.left = code,
            CaptureKey::Right => self.keys.right = code,
            CaptureKey::Pause => self.keys.pause = code,
        }
    }

    fn is_capture_key_cancel(key: KeyEvent) -> bool {
        key.code == KeyCode::Esc
    }

    fn main_menu_len(&self) -> usize {
        if self.saved_game_available {
            7
        } else {
            6
        }
    }

    fn main_menu_action(&self, index: usize) -> MainMenuAction {
        let actions_with_continue = [
            MainMenuAction::Continue,
            MainMenuAction::ArcadeMode,
            MainMenuAction::Settings,
            MainMenuAction::Help,
            MainMenuAction::Controls,
            MainMenuAction::Credits,
            MainMenuAction::Quit,
        ];
        let actions_without_continue = [
            MainMenuAction::ArcadeMode,
            MainMenuAction::Settings,
            MainMenuAction::Help,
            MainMenuAction::Controls,
            MainMenuAction::Credits,
            MainMenuAction::Quit,
        ];

        if self.saved_game_available {
            actions_with_continue
                .get(index)
                .copied()
                .unwrap_or(MainMenuAction::Quit)
        } else {
            actions_without_continue
                .get(index)
                .copied()
                .unwrap_or(MainMenuAction::Quit)
        }
    }
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use super::App;

    #[test]
    fn capture_key_cancel_esc() {
        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        assert!(App::is_capture_key_cancel(key));
    }

    #[test]
    fn capture_key_not_cancel_for_other_keys() {
        let key = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE);
        assert!(!App::is_capture_key_cancel(key));
    }
}
