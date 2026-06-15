use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;

use crate::config::GameConfig;
use crate::game::{Game, GamePhase, Point};
use crate::platform::stats_hint;
use crate::theme::Theme;

/// Width reserved for the right-hand info sidebar.
pub const SIDEBAR_WIDTH: u16 = 28;

/// Pre-computed layout areas refreshed each frame.
pub struct LayoutAreas {
    pub board: Rect,
    pub sidebar: Rect,
    pub status: Rect,
}

/// Split the terminal area into board, sidebar, and status bar regions.
pub fn compute_layout(area: Rect) -> LayoutAreas {
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(1)])
        .split(area);

    let main = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(10),
            Constraint::Length(SIDEBAR_WIDTH.min(area.width.saturating_sub(12))),
        ])
        .split(outer[0]);

    LayoutAreas {
        board: main[0],
        sidebar: main[1],
        status: outer[1],
    }
}

/// Draw one complete frame: the game board, info sidebar, status bar, and any
/// active overlay (pause, game-over, or title screen).
pub fn render(
    frame: &mut Frame,
    game: &Game,
    config: &GameConfig,
    theme: &Theme,
    os_label: &str,
) {
    let areas = compute_layout(frame.area());

    render_board(frame, game, areas.board, theme, config.show_grid);
    render_sidebar(frame, game, config, areas.sidebar, theme, os_label);
    render_status_bar(frame, game, config, areas.status, theme, os_label);

    match game.phase {
        GamePhase::Paused => render_overlay(
            frame,
            areas.board,
            theme,
            " PAUSED ",
            "Press Space to resume",
            theme.paused,
        ),
        GamePhase::GameOver => render_overlay(
            frame,
            areas.board,
            theme,
            " GAME OVER ",
            "Press R to restart · Q to quit",
            theme.game_over,
        ),
        GamePhase::Menu => render_overlay(
            frame,
            areas.board,
            theme,
            " NailSnake ",
            "Press Enter to start",
            theme.title,
        ),
        GamePhase::Running => {}
    }
}

/// Render the playfield: border, optional grid dots, snake segments, food.
///
/// Each logical game cell is drawn as a `cell_w × cell_h` character block so
/// the board always fills the available space, even at different terminal sizes.
fn render_board(
    frame: &mut Frame,
    game: &Game,
    area: Rect,
    theme: &Theme,
    show_grid: bool,
) {
    let inner = inset(area, 1);
    let board_w = inner.width;
    let board_h = inner.height;

    let cell_w = (board_w / game.width.max(1)).max(1);
    let cell_h = (board_h / game.height.max(1)).max(1);

    let used_w = game.width * cell_w;
    let used_h = game.height * cell_h;
    let offset_x = inner.x + (board_w.saturating_sub(used_w)) / 2;
    let offset_y = inner.y + (board_h.saturating_sub(used_h)) / 2;

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.border)
        .title(Span::styled(" NailSnake ", theme.title))
        .title_alignment(Alignment::Center);

    frame.render_widget(block, area);

    if show_grid {
        draw_grid(
            frame,
            offset_x,
            offset_y,
            game.width,
            game.height,
            cell_w,
            cell_h,
            theme,
        );
    }

    for (i, segment) in game.snake.iter().enumerate() {
        let style = theme.snake_segment(i, game.snake.len());
        draw_cell(
            frame,
            offset_x,
            offset_y,
            *segment,
            cell_w,
            cell_h,
            style,
            if i == 0 { "◉" } else { "█" },
        );
    }

    draw_cell(
        frame,
        offset_x,
        offset_y,
        game.food,
        cell_w,
        cell_h,
        theme.food,
        "♦",
    );
}

#[allow(clippy::too_many_arguments)]
fn draw_grid(
    frame: &mut Frame,
    ox: u16,
    oy: u16,
    gw: u16,
    gh: u16,
    cw: u16,
    ch: u16,
    theme: &Theme,
) {
    for y in 0..gh {
        for x in 0..gw {
            let rect = cell_rect(ox, oy, Point { x, y }, cw, ch);
            let dot = Paragraph::new("·").style(theme.grid);
            frame.render_widget(dot, rect);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_cell(
    frame: &mut Frame,
    ox: u16,
    oy: u16,
    point: Point,
    cw: u16,
    ch: u16,
    style: ratatui::style::Style,
    glyph: &str,
) {
    let rect = cell_rect(ox, oy, point, cw, ch);
    let cell = Paragraph::new(glyph).style(style);
    frame.render_widget(cell, rect);
}

/// Map a logical board coordinate to a pixel (character-cell) rectangle.
fn cell_rect(ox: u16, oy: u16, point: Point, cw: u16, ch: u16) -> Rect {
    Rect {
        x: ox + point.x * cw,
        y: oy + point.y * ch,
        width: cw,
        height: ch,
    }
}

/// Shrink a rectangle by `pad` units on each side.
fn inset(area: Rect, pad: u16) -> Rect {
    Rect {
        x: area.x + pad,
        y: area.y + pad,
        width: area.width.saturating_sub(pad * 2),
        height: area.height.saturating_sub(pad * 2),
    }
}

/// Render the right-hand info panel: score, session stats, controls help.
fn render_sidebar(
    frame: &mut Frame,
    game: &Game,
    config: &GameConfig,
    area: Rect,
    theme: &Theme,
    os_label: &str,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.border)
        .title(Span::styled(" Info ", theme.sidebar_title));

    let inner = block.inner(area);
    frame.render_widget(block.style(theme.sidebar), area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),
            Constraint::Length(11),
            Constraint::Min(6),
        ])
        .split(inner);

    let score_lines = vec![
        Line::from(Span::styled("Score", theme.sidebar_title)),
        Line::from(""),
        Line::from(vec![
            Span::styled(format!("{:>6}", game.score), theme.score),
            Span::raw(" pts"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("Best  "),
            Span::styled(
                format!("{}", config.stats.high_score),
                theme.score_high,
            ),
        ]),
        Line::from(vec![
            Span::raw("Len   "),
            Span::styled(format!("{}", game.snake.len()), theme.score),
        ]),
    ];
    frame.render_widget(Paragraph::new(score_lines).style(theme.sidebar), chunks[0]);

    let status = match game.phase {
        GamePhase::Running => "Running",
        GamePhase::Paused => "Paused",
        GamePhase::GameOver => "Game Over",
        GamePhase::Menu => "Ready",
    };

    let info_lines = vec![
        Line::from(Span::styled("Session", theme.sidebar_title)),
        Line::from(""),
        Line::from(format!("Status   {status}")),
        Line::from(format!("OS       {os_label}")),
        Line::from(format!("Mode     {}", config.difficulty.label())),
        Line::from(format!(
            "Walls    {}",
            if config.wrap_walls { "Wrap" } else { "Solid" }
        )),
        Line::from(format!("Speed    {} ms", game.tick_interval_ms())),
        Line::from(format!("Food     {}", game.food_eaten)),
        Line::from(format!("Stats    {}", stats_hint())),
    ];
    frame.render_widget(Paragraph::new(info_lines).style(theme.sidebar), chunks[1]);

    let help = vec![
        Line::from(Span::styled("Controls", theme.sidebar_title)),
        Line::from(""),
        help_line("Move", "↑↓←→ WASD", theme),
        help_line("Pause", "Space", theme),
        help_line("Restart", "R", theme),
        help_line("Quit", "Q Esc", theme),
        Line::from(""),
        Line::from(Span::styled("man nailsnake", theme.help_key)),
    ];
    frame.render_widget(
        Paragraph::new(help).style(theme.sidebar).wrap(Wrap { trim: true }),
        chunks[2],
    );
}

fn help_line<'a>(action: &'a str, keys: &'a str, theme: &Theme) -> Line<'a> {
    Line::from(vec![
        Span::styled(format!("{action:<7}"), theme.help),
        Span::styled(keys, theme.help_key),
    ])
}

/// A one-line status bar at the bottom of the screen showing quick-reference info.
fn render_status_bar(
    frame: &mut Frame,
    game: &Game,
    config: &GameConfig,
    area: Rect,
    theme: &Theme,
    os_label: &str,
) {
    let phase_hint = match game.phase {
        GamePhase::Running => "Playing",
        GamePhase::Paused => "Paused",
        GamePhase::GameOver => "Dead",
        GamePhase::Menu => "Press Enter",
    };

    let text = Line::from(vec![
        Span::styled(" NailSnake ", theme.title),
        Span::raw(" │ "),
        Span::raw(os_label),
        Span::raw(" │ "),
        Span::raw(phase_hint),
        Span::raw(" │ "),
        Span::raw(format!("score {}", game.score)),
        Span::raw(" │ "),
        Span::raw(format!("best {}", config.stats.high_score)),
        Span::raw(" │ "),
        Span::raw(format!("games {}", config.stats.games_played)),
    ]);

    let bar = Paragraph::new(text)
        .style(theme.status_bar)
        .alignment(Alignment::Left);
    frame.render_widget(bar, area);
}

/// Draw a centered popup overlay (title, pause, game-over).
///
/// Uses `Clear` to erase the underlying board area so the overlay text is
/// readable even on a busy playfield.
fn render_overlay(
    frame: &mut Frame,
    area: Rect,
    theme: &Theme,
    title: &str,
    subtitle: &str,
    title_style: ratatui::style::Style,
) {
    let popup_area = centered_rect(60, 40, area);
    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.border)
        .title(Span::styled(title, title_style.add_modifier(Modifier::BOLD)))
        .style(theme.overlay);

    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    let text = vec![
        Line::from(""),
        Line::from(Span::styled(title.trim(), title_style)),
        Line::from(""),
        Line::from(Span::styled(subtitle, theme.message)),
    ];

    frame.render_widget(
        Paragraph::new(text)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true }),
        inner,
    );
}

/// Compute a rectangle centred within `area` at a given percentage size.
fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
