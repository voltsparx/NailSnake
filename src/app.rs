mod input;
mod key_bindings;
mod menu_state;
mod menu_views;
mod terminal;

use std::io::Stdout;
use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::event::{self, Event, KeyEventKind};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use key_bindings::KeyBindings;
use menu_state::{CaptureKey, MenuScreen};
use nailsnake::config::GameConfig;
use nailsnake::game::{Game, GamePhase};
use nailsnake::platform::{ensure_terminal_size, os_label};
use nailsnake::theme::Theme;
use nailsnake::ui;
use terminal::{board_dimensions, initialize_terminal, install_panic_hook, restore_terminal};

/// Top-level application: owns the terminal, game state, config, and theme.
///
/// Responsible for the main event loop, input dispatch, resize handling,
/// and ensuring the terminal is always restored on exit (even a panic).
pub struct App {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    game: Game,
    config: GameConfig,
    theme: Theme,
    next_tick: Instant,
    next_render: Instant,
    animation_started: Instant,
    /// Prevents double-restore in Drop (e.g., if `run` errors and we also drop).
    restored: bool,
    main_menu_index: usize,
    settings_index: usize,
    controls_index: usize,
    pause_menu_index: usize,
    show_help: bool,
    menu_screen: MenuScreen,
    capture_key: Option<CaptureKey>,
    keys: KeyBindings,
    frame_tick: u64,
    saved_game_available: bool,
    notice: Option<String>,
}

impl App {
    pub fn new(config: GameConfig) -> Result<Self> {
        install_panic_hook();
        let (terminal_width, terminal_height) = crossterm::terminal::size()?;
        // A too-small terminal is rendered as a recoverable UI state.  Keep a
        // small startup floor only so crossterm can initialize sanely.
        ensure_terminal_size(terminal_width, terminal_height)?;

        let terminal = initialize_terminal()?;

        let (board_w, board_h) = board_dimensions(terminal_width, terminal_height);
        let theme = Theme::new(config.color_mode);
        let keys = KeyBindings::from_persisted(&config.key_bindings);
        let mut game = Game::with_options(
            board_w,
            board_h,
            config.difficulty,
            config.wrap_walls,
            config.random_maze,
            config.custom_speed_ms,
        );
        game.phase = GamePhase::Menu;

        let saved_game_available = config.has_saved_game();

        Ok(Self {
            terminal,
            game,
            config,
            theme,
            next_tick: Instant::now(),
            next_render: Instant::now(),
            animation_started: Instant::now(),
            restored: false,
            main_menu_index: 0,
            settings_index: 0,
            controls_index: 0,
            pause_menu_index: 0,
            show_help: false,
            menu_screen: MenuScreen::Main,
            capture_key: None,
            keys,
            frame_tick: 0,
            saved_game_available,
            notice: None,
        })
    }

    /// Fixed-step simulation, independent render cadence, and bounded event
    /// draining keep input responsive even when drawing takes longer than a
    /// frame.  Catch-up is deliberately capped to avoid a spiral of death.
    pub fn run(&mut self) -> Result<()> {
        const RENDER_INTERVAL: Duration = Duration::from_millis(16);
        const MAX_CATCH_UP_TICKS: u8 = 4;

        loop {
            let mut should_exit = false;
            for _ in 0..64 {
                if !event::poll(Duration::ZERO)? {
                    break;
                }
                match event::read()? {
                    Event::Key(key) if should_dispatch_key(key) => {
                        should_exit |= self.handle_key(key)?
                    }
                    Event::Resize(width, height) => self.handle_resize(width, height)?,
                    _ => {}
                }
            }
            if should_exit {
                self.prepare_exit()?;
                break;
            }

            let now = Instant::now();
            if self.game.phase == GamePhase::Running {
                let mut ticks = 0;
                while Instant::now() >= self.next_tick && ticks < MAX_CATCH_UP_TICKS {
                    let ended = self.game.tick();
                    self.next_tick += Duration::from_millis(self.game.tick_interval_ms());
                    ticks += 1;
                    if ended {
                        if let Err(error) = self.config.record_game(self.game.score) {
                            self.notice = Some(format!("Could not save statistics: {error}"));
                        }
                        if let Err(error) = self.config.clear_active_game() {
                            self.notice = Some(format!("Could not clear saved game: {error}"));
                        }
                        self.saved_game_available = false;
                        break;
                    }
                }
                if ticks == MAX_CATCH_UP_TICKS && Instant::now() >= self.next_tick {
                    self.next_tick =
                        Instant::now() + Duration::from_millis(self.game.tick_interval_ms());
                }
            } else {
                self.next_tick = now + Duration::from_millis(self.game.tick_interval_ms());
            }

            if now >= self.next_render {
                self.frame_tick = self.animation_started.elapsed().as_millis() as u64 / 33;
                let menu = self.active_menu_view();
                let controls = self.control_labels();
                self.terminal.draw(|f| {
                    ui::render(
                        f,
                        &self.game,
                        &self.config,
                        &self.theme,
                        os_label(),
                        menu.as_ref(),
                        self.frame_tick,
                        self.notice.as_deref(),
                        &controls,
                    );
                })?;
                self.next_render = Instant::now() + RENDER_INTERVAL;
            }

            let deadline = if self.game.phase == GamePhase::Running {
                self.next_tick.min(self.next_render)
            } else {
                self.next_render
            };
            let timeout = deadline.saturating_duration_since(Instant::now());
            if !timeout.is_zero() {
                let _ = event::poll(timeout)?;
            }
        }

        Ok(())
    }

    fn prepare_exit(&mut self) -> Result<()> {
        match self.game.phase {
            GamePhase::Running | GamePhase::Paused => {
                self.config.save_active_game(&self.game)?;
                self.saved_game_available = true;
            }
            GamePhase::GameOver | GamePhase::Won => {
                self.config.clear_active_game()?;
                self.saved_game_available = false;
            }
            GamePhase::Menu => {}
        }
        if let Err(error) = self.config.save_settings() {
            self.notice = Some(format!("Could not save settings: {error}"));
            return Err(error);
        }
        Ok(())
    }

    pub(super) fn reset_tick_deadline(&mut self) {
        self.next_tick = Instant::now() + Duration::from_millis(self.game.tick_interval_ms());
    }

    fn control_labels(&self) -> Vec<String> {
        use key_bindings::key_label;
        vec![
            format!(
                "Move  {} {} {} {}",
                key_label(self.keys.up),
                key_label(self.keys.down),
                key_label(self.keys.left),
                key_label(self.keys.right)
            ),
            format!("Pause {} / Esc", key_label(self.keys.pause)),
            "Restart R".into(),
            "Menu Q".into(),
        ]
    }
}

/// Windows terminals emit a key-release event after every key press. Releases
/// are never user actions, so dispatching them makes menu movement and toggles
/// happen twice.
fn should_dispatch_key(key: crossterm::event::KeyEvent) -> bool {
    key.kind != KeyEventKind::Release
}

/// Drop guard restores the terminal even if we exit via `?` or a panic.
///
/// Without this, the user would be left in raw mode with no cursor - a very
/// frustrating experience.
impl Drop for App {
    fn drop(&mut self) {
        if !self.restored {
            let _ = restore_terminal(&mut self.terminal);
            self.restored = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};

    use super::should_dispatch_key;

    #[test]
    fn key_release_is_not_dispatched() {
        let mut key = KeyEvent::new(KeyCode::Enter, crossterm::event::KeyModifiers::NONE);
        key.kind = KeyEventKind::Release;
        assert!(!should_dispatch_key(key));
    }
}
