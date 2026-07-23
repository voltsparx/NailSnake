use std::io::{self, stdout, Stdout, Write};
use std::panic;
use std::thread;
use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::{cursor, execute, terminal::ClearType};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use nailsnake::config::GameConfig;
use nailsnake::game::{Direction, Game, GamePhase};
use nailsnake::platform::{ensure_terminal_size, os_label};
use nailsnake::theme::Theme;
use nailsnake::ui;

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
    pause_menu_index: usize,
    show_help: bool,
    frame_tick: u64,
}

impl App {
    pub fn new(config: GameConfig) -> Result<Self> {
        install_panic_hook();
        enable_raw_mode()?;
        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen, cursor::Hide)?;

        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        let size = terminal.size()?;
        ensure_terminal_size(size.width, size.height)?;

        let (board_w, board_h) = board_dimensions(size.width, size.height);
        let theme = Theme::new(config.color_mode);
        let mut game = Game::new(board_w, board_h, config.difficulty, config.wrap_walls);
        game.phase = GamePhase::Menu;

        Ok(Self {
            terminal,
            game,
            config,
            theme,
            last_tick: Instant::now(),
            restored: false,
            main_menu_index: 0,
            pause_menu_index: 0,
            show_help: false,
            frame_tick: 0,
        })
    }

    /// Main event loop: render, poll input, tick game, then cap frame rate.
    ///
    /// The polling interval adapts to the current game phase. While playing, we
    /// poll at a finer grain (min of tick_ms and 50ms) so key presses feel
    /// responsive even at slow difficulty speeds.
    pub fn run(&mut self) -> Result<()> {
        const TARGET_FPS: u64 = 120;
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
                tick_ms.min(50)
            } else {
                100
            };

            if event::poll(Duration::from_millis(poll_ms))? {
                match event::read()? {
                    Event::Key(key) => {
                        if self.handle_key(key)? {
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
                }
            }

            let elapsed = frame_start.elapsed();
            if elapsed < FRAME_TIME {
                thread::sleep(FRAME_TIME - elapsed);
            }
        }

        Ok(())
    }

    /// On terminal resize, remap game coordinates proportionally.
    ///
    /// If the remap results in a self-intersecting snake or food-on-snake, we
    /// fall back to a fresh game (preserving the current phase).
    fn handle_resize(&mut self, width: u16, height: u16) -> Result<()> {
        if width < nailsnake::platform::MIN_TERM_WIDTH
            || height < nailsnake::platform::MIN_TERM_HEIGHT
        {
            return Ok(());
        }

        self.terminal.clear()?;
        let (board_w, board_h) = board_dimensions(width, height);
        let phase = self.game.phase;

        if !self.game.resize(board_w, board_h) {
            self.game = Game::new(
                board_w,
                board_h,
                self.config.difficulty,
                self.config.wrap_walls,
            );
            self.game.phase = phase;
        } else {
            self.game.phase = phase;
        }
        Ok(())
    }

    /// Translate keyboard events into game actions.
    ///
    /// Returns `true` if the caller should break out of the event loop (quit).
    fn handle_key(&mut self, key: KeyEvent) -> Result<bool> {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return Ok(true);
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
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.game.toggle_pause();
                self.pause_menu_index = 0;
            }
            KeyCode::Char('h') | KeyCode::Char('H') | KeyCode::Char('?') => {
                self.show_help = true;
            }
            KeyCode::Char('r') | KeyCode::Char('R') => {
                self.game.reset();
                self.last_tick = Instant::now();
            }
            KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('W') => {
                self.game.set_direction(Direction::Up);
            }
            KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('S') => {
                self.game.set_direction(Direction::Down);
            }
            KeyCode::Left | KeyCode::Char('a') | KeyCode::Char('A') => {
                self.game.set_direction(Direction::Left);
            }
            KeyCode::Right | KeyCode::Char('d') | KeyCode::Char('D') => {
                self.game.set_direction(Direction::Right);
            }
            _ => {}
        }

        Ok(false)
    }

    fn handle_main_menu_key(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => return Ok(true),
            KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('W') => {
                self.main_menu_index = self.main_menu_index.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('S') => {
                self.main_menu_index = (self.main_menu_index + 1).min(main_menu_len() - 1);
            }
            KeyCode::Left
            | KeyCode::Right
            | KeyCode::Char('a')
            | KeyCode::Char('A')
            | KeyCode::Char('d')
            | KeyCode::Char('D') => {
                self.adjust_main_menu_option();
            }
            KeyCode::Enter => match self.main_menu_index {
                0 => self.start_game(),
                1..=3 => self.adjust_main_menu_option(),
                4 => self.show_help = true,
                5 => return Ok(true),
                _ => {}
            },
            KeyCode::Char('h') | KeyCode::Char('H') | KeyCode::Char('?') => self.show_help = true,
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
                    self.last_tick = Instant::now();
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
        self.game.reset();
        self.game.phase = GamePhase::Running;
        self.last_tick = Instant::now();
    }

    fn adjust_main_menu_option(&mut self) {
        match self.main_menu_index {
            1 => {
                self.config.difficulty = self.config.difficulty.next();
                self.game.difficulty = self.config.difficulty;
            }
            2 => {
                self.config.wrap_walls = !self.config.wrap_walls;
                self.game.wrap_walls = self.config.wrap_walls;
            }
            3 => self.config.show_grid = !self.config.show_grid,
            _ => {}
        }
    }

    fn active_menu_view(&self) -> Option<ui::MenuView> {
        if self.show_help {
            return Some(ui::MenuView {
                title: "Help".to_string(),
                items: vec![
                    "Move: Arrow keys or WASD".to_string(),
                    "Pause: Space or Enter".to_string(),
                    "Restart: R".to_string(),
                    "Help: H or ?".to_string(),
                    "Quit: Q or Esc".to_string(),
                    "Press Enter, H, ?, or Esc to close".to_string(),
                ],
                selected: None,
                hint: "NailSnake keeps terminal controls simple and fast.".to_string(),
            });
        }

        match self.game.phase {
            GamePhase::Menu => Some(ui::MenuView {
                title: "Main Menu".to_string(),
                items: vec![
                    "Arcade Mode".to_string(),
                    format!("Difficulty: {}", self.config.difficulty.label()),
                    format!(
                        "Walls: {}",
                        if self.config.wrap_walls {
                            "Teleport"
                        } else {
                            "Solid"
                        }
                    ),
                    format!("Grid: {}", if self.config.show_grid { "On" } else { "Off" }),
                    "Help".to_string(),
                    "Quit".to_string(),
                ],
                selected: Some(self.main_menu_index),
                hint: "Use Up/Down, Enter to choose, Left/Right to change settings.".to_string(),
            }),
            GamePhase::Paused => Some(ui::MenuView {
                title: "Pause Menu".to_string(),
                items: vec![
                    "Resume".to_string(),
                    "Restart".to_string(),
                    "Help".to_string(),
                    "Quit Game".to_string(),
                ],
                selected: Some(self.pause_menu_index),
                hint: "Enter selects. Space resumes instantly.".to_string(),
            }),
            _ => None,
        }
    }
}

fn main_menu_len() -> usize {
    6
}

fn pause_menu_len() -> usize {
    4
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

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> io::Result<()> {
    let _ = disable_raw_mode();
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        cursor::Show,
        crossterm::terminal::Clear(ClearType::All),
    )?;
    let _ = terminal.show_cursor();
    Ok(())
}

/// Override the default panic hook so a crash doesn't leave the terminal broken.
///
/// The original hook is preserved and called afterward so the backtrace still
/// prints, giving us the best of both worlds: a clean terminal *and* a useful
/// crash report.
fn install_panic_hook() {
    let original = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let mut stdout = stdout();
        let _ = execute!(
            stdout,
            LeaveAlternateScreen,
            cursor::Show,
            crossterm::terminal::Clear(ClearType::All)
        );
        let _ = stdout.flush();
        original(info);
    }));
}

/// Given the terminal dimensions, compute a usable board size.
///
/// We reserve space for the sidebar, status bar, borders, and padding, then
/// clamp the result to reasonable bounds. Games on tiny terminals are no fun.
fn board_dimensions(term_w: u16, term_h: u16) -> (u16, u16) {
    let usable_w = term_w.saturating_sub(ui::SIDEBAR_WIDTH + 6);
    let usable_h = term_h.saturating_sub(4);

    let width = usable_w.clamp(20, 48);
    let height = usable_h.clamp(12, 28);
    (width, height)
}
