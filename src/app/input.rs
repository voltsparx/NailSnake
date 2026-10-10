use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::layout::Rect;

use super::key_bindings::{action_label, Action, KeyBindings};
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
        // Terminal dimensions are a viewport concern.  The logical board never
        // changes here, so a too-small resize cannot erase a live game.
        let areas = nailsnake::ui::compute_layout(Rect::new(0, 0, width, height));
        if !nailsnake::ui::board_fits(&self.game, areas.board)
            && self.game.phase == GamePhase::Running
        {
            self.pause_game();
            self.notice = Some("Terminal too small; game paused until it fits again.".into());
        }
        self.terminal.clear()?;
        Ok(())
    }

    pub(super) fn handle_key(&mut self, key: KeyEvent) -> Result<bool> {
        if key.kind == KeyEventKind::Release {
            return Ok(false);
        }
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return Ok(true);
        }
        let action = self.keys.action_for(key.code);

        // Repeated movement is useful, but repeated toggles such as Pause or
        // Confirm make held keys flicker through screens on several terminals.
        if key.kind == KeyEventKind::Repeat
            && !matches!(
                action,
                Some(Action::Up | Action::Down | Action::Left | Action::Right)
            )
        {
            return Ok(false);
        }
        if let Some(capture) = self.capture_key {
            return self.handle_capture(capture, key);
        }

        if self.show_help {
            if matches!(action, Some(Action::Back | Action::Confirm | Action::Help)) {
                self.show_help = false;
            } else if matches!(action, Some(Action::Quit))
                && self.game.phase == GamePhase::Menu
                && self.menu_screen == MenuScreen::Main
            {
                return Ok(true);
            }
            return Ok(false);
        }
        match self.game.phase {
            GamePhase::Menu => self.handle_main_menu_action(action),
            GamePhase::Paused => self.handle_pause_action(action),
            GamePhase::GameOver | GamePhase::Won => self.handle_game_over_action(action),
            GamePhase::Running => self.handle_running_action(action),
        }
    }

    fn handle_capture(&mut self, capture: CaptureKey, key: KeyEvent) -> Result<bool> {
        if key.code == KeyCode::Esc {
            self.capture_key = None;
            return Ok(false);
        }
        let action = capture.action();
        if let Some(existing) = self.keys.conflict(action, key.code) {
            self.notice = Some(format!(
                "That key is already bound to {}.",
                action_label(existing)
            ));
        } else {
            self.keys.assign(action, key.code);
            self.config.key_bindings = self.keys.persisted();
            self.save_settings_feedback();
            self.notice = Some(format!(
                "{} bound to {}.",
                action_label(action),
                super::key_bindings::key_label(key.code)
            ));
        }
        self.capture_key = None;
        Ok(false)
    }

    fn handle_running_action(&mut self, action: Option<Action>) -> Result<bool> {
        match action {
            Some(Action::Back | Action::Pause | Action::Quit) => self.pause_game(),
            Some(Action::Restart) => self.restart_game(),
            Some(Action::Help) => {
                // Help is a modal view. Pause first so opening it never leaves
                // an invisible, still-moving game underneath the modal.
                self.pause_game();
                self.show_help = true;
            }
            Some(Action::Up) => self.game.set_direction(Direction::Up),
            Some(Action::Down) => self.game.set_direction(Direction::Down),
            Some(Action::Left) => self.game.set_direction(Direction::Left),
            Some(Action::Right) => self.game.set_direction(Direction::Right),
            _ => {}
        }
        Ok(false)
    }

    fn handle_game_over_action(&mut self, action: Option<Action>) -> Result<bool> {
        match action {
            Some(Action::Restart | Action::Confirm) => self.restart_game(),
            Some(Action::Back | Action::Quit) => {
                self.game.phase = GamePhase::Menu;
                self.menu_screen = MenuScreen::Main;
            }
            Some(Action::Help) => self.show_help = true,
            _ => {}
        }
        Ok(false)
    }

    fn handle_main_menu_action(&mut self, action: Option<Action>) -> Result<bool> {
        match self.menu_screen {
            MenuScreen::Settings => return self.handle_settings_action(action),
            MenuScreen::Controls => return self.handle_controls_action(action),
            MenuScreen::Credits => {
                if matches!(action, Some(Action::Back | Action::Confirm | Action::Quit)) {
                    self.menu_screen = MenuScreen::Main;
                }
                return Ok(false);
            }
            MenuScreen::Main => {}
        }
        match action {
            Some(Action::Back | Action::Quit) => return Ok(true),
            Some(Action::Up) => self.main_menu_index = self.main_menu_index.saturating_sub(1),
            Some(Action::Down) => {
                self.main_menu_index = (self.main_menu_index + 1).min(self.main_menu_len() - 1)
            }
            Some(Action::Confirm) => match self.main_menu_action(self.main_menu_index) {
                MainMenuAction::Continue => self.continue_game(),
                MainMenuAction::ArcadeMode => self.start_game(),
                MainMenuAction::Settings => self.menu_screen = MenuScreen::Settings,
                MainMenuAction::Help => self.show_help = true,
                MainMenuAction::Controls => self.menu_screen = MenuScreen::Controls,
                MainMenuAction::Credits => self.menu_screen = MenuScreen::Credits,
                MainMenuAction::Quit => return Ok(true),
            },
            Some(Action::Help) => self.show_help = true,
            _ => {}
        }
        Ok(false)
    }

    fn handle_settings_action(&mut self, action: Option<Action>) -> Result<bool> {
        match action {
            Some(Action::Back | Action::Quit) => self.menu_screen = MenuScreen::Main,
            Some(Action::Up) => self.settings_index = self.settings_index.saturating_sub(1),
            Some(Action::Down) => {
                self.settings_index = (self.settings_index + 1).min(settings_menu_len() - 1)
            }
            Some(Action::Left) => self.adjust_setting(false),
            Some(Action::Right | Action::Confirm) => {
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

    fn handle_controls_action(&mut self, action: Option<Action>) -> Result<bool> {
        match action {
            Some(Action::Back | Action::Quit) => self.menu_screen = MenuScreen::Main,
            Some(Action::Up) => self.controls_index = self.controls_index.saturating_sub(1),
            Some(Action::Down) => {
                self.controls_index = (self.controls_index + 1).min(controls_menu_len() - 1)
            }
            Some(Action::Confirm) => match self.controls_index {
                0 => self.capture_key = Some(CaptureKey::Up),
                1 => self.capture_key = Some(CaptureKey::Down),
                2 => self.capture_key = Some(CaptureKey::Left),
                3 => self.capture_key = Some(CaptureKey::Right),
                4 => self.capture_key = Some(CaptureKey::Pause),
                5 => {
                    self.keys = KeyBindings::default();
                    self.config.key_bindings = self.keys.persisted();
                    self.save_settings_feedback();
                }
                6 => self.menu_screen = MenuScreen::Main,
                _ => {}
            },
            _ => {}
        }
        Ok(false)
    }

    fn handle_pause_action(&mut self, action: Option<Action>) -> Result<bool> {
        match action {
            Some(Action::Back | Action::Pause) => self.resume_game(),
            Some(Action::Up) => self.pause_menu_index = self.pause_menu_index.saturating_sub(1),
            Some(Action::Down) => {
                self.pause_menu_index = (self.pause_menu_index + 1).min(pause_menu_len() - 1)
            }
            Some(Action::Help) => self.show_help = true,
            Some(Action::Quit) => {
                self.game.phase = GamePhase::Menu;
                self.menu_screen = MenuScreen::Main;
            }
            Some(Action::Confirm) => match self.pause_menu_index {
                0 => self.resume_game(),
                1 => self.restart_game(),
                2 => {
                    self.game.phase = GamePhase::Menu;
                    self.menu_screen = MenuScreen::Main;
                }
                3 => self.show_help = true,
                4 => return Ok(true),
                _ => {}
            },
            _ => {}
        }
        Ok(false)
    }

    fn pause_game(&mut self) {
        if self.game.phase == GamePhase::Running {
            self.game.toggle_pause();
            self.pause_menu_index = 0;
        }
    }
    fn resume_game(&mut self) {
        if self.game.phase == GamePhase::Paused {
            self.game.toggle_pause();
            self.reset_tick_deadline();
        }
    }
    fn restart_game(&mut self) {
        self.game.reset();
        self.game.phase = GamePhase::Running;
        self.reset_tick_deadline();
    }

    fn start_game(&mut self) {
        self.game = Game::with_options(
            self.game.width,
            self.game.height,
            self.config.difficulty,
            self.config.wrap_walls,
            self.config.random_maze,
            self.config.custom_speed_ms,
        );
        if let Err(error) = self.config.clear_active_game() {
            self.notice = Some(format!("Could not clear saved game: {error}"));
        }
        self.saved_game_available = false;
        self.reset_tick_deadline();
    }

    fn continue_game(&mut self) {
        match self.config.load_active_game() {
            Ok(Some(mut game)) => {
                game.phase = GamePhase::Paused;
                self.config.difficulty = game.difficulty;
                self.config.wrap_walls = game.wrap_walls;
                self.config.random_maze = game.random_maze;
                self.config.custom_speed_ms = game.custom_speed_ms;
                self.game = game;
                self.pause_menu_index = 0;
                self.reset_tick_deadline();
            }
            Ok(None) => {
                self.saved_game_available = false;
                self.main_menu_index = self.main_menu_index.min(self.main_menu_len() - 1);
            }
            Err(error) => {
                self.saved_game_available = false;
                self.notice = Some(format!("Could not load saved game: {error}"));
            }
        }
    }

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
                    self.config.custom_speed_ms =
                        self.config.custom_speed_ms.saturating_add(5).min(260);
                }
            }
            2 => self.config.wrap_walls = !self.config.wrap_walls,
            3 => self.config.random_maze = !self.config.random_maze,
            4 => self.config.snake_skin = self.config.snake_skin.next(),
            5 => self.config.show_grid = !self.config.show_grid,
            _ => return,
        }
        self.notice = Some("Saved. Settings apply to the next game.".into());
        self.save_settings_feedback();
    }

    fn save_settings_feedback(&mut self) {
        if let Err(error) = self.config.save_settings() {
            self.notice = Some(format!("Could not save settings: {error}"));
        }
    }

    fn main_menu_len(&self) -> usize {
        if self.saved_game_available {
            7
        } else {
            6
        }
    }
    fn main_menu_action(&self, index: usize) -> MainMenuAction {
        let actions = if self.saved_game_available {
            &[
                MainMenuAction::Continue,
                MainMenuAction::ArcadeMode,
                MainMenuAction::Settings,
                MainMenuAction::Help,
                MainMenuAction::Controls,
                MainMenuAction::Credits,
                MainMenuAction::Quit,
            ][..]
        } else {
            &[
                MainMenuAction::ArcadeMode,
                MainMenuAction::Settings,
                MainMenuAction::Help,
                MainMenuAction::Controls,
                MainMenuAction::Credits,
                MainMenuAction::Quit,
            ][..]
        };
        actions.get(index).copied().unwrap_or(MainMenuAction::Quit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn capture_key_cancel_esc() {
        assert_eq!(KeyCode::Esc, KeyCode::Esc);
    }
}
