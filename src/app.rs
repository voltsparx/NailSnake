use std::io::{self, stdout, Stdout};
use std::panic;
use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::{cursor, execute, terminal::ClearType};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use crate::config::GameConfig;
use crate::game::{Direction, Game, GamePhase};
use crate::platform::{ensure_terminal_size, os_label};
use crate::theme::Theme;
use crate::ui;

pub struct App {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    game: Game,
    config: GameConfig,
    theme: Theme,
    last_tick: Instant,
    restored: bool,
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
        })
    }

    pub fn run(&mut self) -> Result<()> {
        loop {
            self.terminal.draw(|f| {
                ui::render(f, &self.game, &self.config, &self.theme, os_label());
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
        }

        Ok(())
    }

    fn handle_resize(&mut self, width: u16, height: u16) -> Result<()> {
        if width < crate::platform::MIN_TERM_WIDTH || height < crate::platform::MIN_TERM_HEIGHT {
            return Ok(());
        }

        self.terminal.clear()?;
        let (board_w, board_h) = board_dimensions(width, height);
        let phase = self.game.phase;
        self.game.resize(board_w, board_h);
        self.game.phase = phase;
        Ok(())
    }

    fn handle_key(&mut self, key: KeyEvent) -> Result<bool> {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return Ok(true);
        }

        match key.code {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => return Ok(true),
            KeyCode::Enter if self.game.phase == GamePhase::Menu => {
                self.game.phase = GamePhase::Running;
                self.last_tick = Instant::now();
            }
            KeyCode::Char(' ') => self.game.toggle_pause(),
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
}

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

fn install_panic_hook() {
    let original = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = execute!(stdout(), LeaveAlternateScreen, cursor::Show);
        original(info);
    }));
}

fn board_dimensions(term_w: u16, term_h: u16) -> (u16, u16) {
    let usable_w = term_w.saturating_sub(ui::SIDEBAR_WIDTH + 6);
    let usable_h = term_h.saturating_sub(4);

    let width = usable_w.clamp(20, 48);
    let height = usable_h.clamp(12, 28);
    (width, height)
}
