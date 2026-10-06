use std::io::{self, stdout, Stdout, Write};
use std::panic;
use std::sync::Once;

use anyhow::Result;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::{cursor, execute, terminal::ClearType};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

pub(super) fn initialize_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, cursor::Hide)?;

    let backend = CrosstermBackend::new(stdout);
    Ok(Terminal::new(backend)?)
}

pub(super) fn restore_terminal(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
) -> io::Result<()> {
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

pub(super) fn install_panic_hook() {
    static INSTALLED: Once = Once::new();
    INSTALLED.call_once(|| {
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
    });
}

pub(super) fn board_dimensions(term_w: u16, term_h: u16) -> (u16, u16) {
    // Board coordinates are independent of the terminal viewport.  Select a
    // conservative logical board once at startup and preserve it on resize.
    let width = term_w.saturating_sub(8).clamp(20, 40);
    let height = term_h.saturating_sub(4).clamp(12, 20);
    (width, height)
}
