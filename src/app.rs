mod input;
mod key_bindings;
mod menu_state;
mod menu_views;
mod terminal;

use std::io::Stdout;
use std::thread;
use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::event::{self, Event};
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
    /// Timestamp of the last game tick - used to enforce the tick interval.
    last_tick: Instant,
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
}

impl App {
    pub fn new(config: GameConfig) -> Result<Self> {
        install_panic_hook();
        let (terminal_width, terminal_height) = crossterm::terminal::size()?;
        ensure_terminal_size(terminal_width, terminal_height)?;

        let terminal = initialize_terminal()?;

        let (board_w, board_h) = board_dimensions(terminal_width, terminal_height);
        let theme = Theme::new(config.color_mode);
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
            last_tick: Instant::now(),
            restored: false,
            main_menu_index: 0,
            settings_index: 0,
            controls_index: 0,
            pause_menu_index: 0,
            show_help: false,
            menu_screen: MenuScreen::Main,
            capture_key: None,
            keys: KeyBindings::default(),
            frame_tick: 0,
            saved_game_available,
        })
    }

    /// Main event loop: render, poll input, tick game, then cap frame rate.
    ///
    /// The polling interval adapts to the current game phase. While playing, we
    /// poll at a finer grain (min of tick_ms and 50ms) so key presses feel
    /// responsive even at slow difficulty speeds.
    pub fn run(&mut self) -> Result<()> {
        const TARGET_FPS: u64 = 30;
        const FRAME_TIME: Duration = Duration::from_millis(1000 / TARGET_FPS);

        loop {
            let frame_start = Instant::now();
            self.frame_tick = self.frame_tick.wrapping_add(1);
            let menu = self.active_menu_view();

            self.terminal.draw(|f| {
                ui::render(
                    f,
                    &self.game,
                    &self.config,
                    &self.theme,
                    os_label(),
                    menu.as_ref(),
                    self.frame_tick,
                );
            })?;

            let tick_ms = self.game.tick_interval_ms();
            let poll_ms = if self.game.phase == GamePhase::Running {
                16
            } else {
                80
            };

            if event::poll(Duration::from_millis(poll_ms))? {
                match event::read()? {
                    Event::Key(key) => {
                        if self.handle_key(key)? {
                            self.prepare_exit()?;
                            break;
                        }
                    }
                    Event::Resize(width, height) => {
                        self.handle_resize(width, height)?;
                    }
                    _ => {}
                }
            }

            if self.game.phase == GamePhase::Running
                && self.last_tick.elapsed() >= Duration::from_millis(tick_ms)
            {
                let ended = self.game.tick();
                self.last_tick = Instant::now();
                if ended {
                    let _ = self.config.record_game(self.game.score);
                    let _ = self.config.clear_active_game();
                    self.saved_game_available = false;
                }
            }

            let elapsed = frame_start.elapsed();
            if elapsed < FRAME_TIME {
                thread::sleep(FRAME_TIME - elapsed);
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
            GamePhase::GameOver => {
                self.config.clear_active_game()?;
                self.saved_game_available = false;
            }
            GamePhase::Menu => {}
        }
        Ok(())
    }
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
