use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

use crate::config::GameConfig;
use crate::game::{Game, GamePhase};
use crate::platform::stats_hint;
use crate::theme::Theme;

pub fn render_sidebar(
    frame: &mut Frame,
    game: &Game,
    config: &GameConfig,
    area: Rect,
    theme: &Theme,
    os_label: &str,
    controls: &[String],
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
            Constraint::Length(7),
            Constraint::Length(9),
            Constraint::Min(5),
        ])
        .split(inner);

    render_score(frame, game, config, chunks[0], theme);
    render_session(frame, game, config, chunks[1], theme, os_label);
    render_controls(frame, chunks[2], theme, controls);
}

fn render_score(frame: &mut Frame, game: &Game, config: &GameConfig, area: Rect, theme: &Theme) {
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
            Span::styled(format!("{}", config.stats.high_score), theme.score_high),
        ]),
        Line::from(vec![
            Span::raw("Len   "),
            Span::styled(format!("{}", game.snake.len()), theme.score),
        ]),
    ];
    frame.render_widget(Paragraph::new(score_lines).style(theme.sidebar), area);
}

fn render_session(
    frame: &mut Frame,
    game: &Game,
    config: &GameConfig,
    area: Rect,
    theme: &Theme,
    os_label: &str,
) {
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
        Line::from(format!("OS     {os_label}")),
        Line::from(format!("Mode   {}", config.difficulty.label())),
        Line::from(format!(
            "Walls  {}",
            if config.wrap_walls { "Wrap" } else { "Solid" }
        )),
        Line::from(format!("Speed  {}ms", game.tick_interval_ms())),
        Line::from(format!(
            "Maze   {}",
            if config.random_maze { "On" } else { "Off" }
        )),
        Line::from(format!("Stats  {}", short_stats_hint())),
    ];
    frame.render_widget(Paragraph::new(info_lines).style(theme.sidebar), area);
}

fn render_controls(frame: &mut Frame, area: Rect, theme: &Theme, controls: &[String]) {
    let mut help = vec![
        Line::from(Span::styled("Controls", theme.sidebar_title)),
        Line::from(""),
    ];
    help.extend(
        controls
            .iter()
            .map(|line| Line::from(Span::styled(line.clone(), theme.help_key))),
    );
    help.push(Line::from(""));
    help.push(Line::from(Span::styled("man nailsnake", theme.help_key)));
    frame.render_widget(
        Paragraph::new(help)
            .style(theme.sidebar)
            .wrap(Wrap { trim: true }),
        area,
    );
}

fn short_stats_hint() -> &'static str {
    let hint = stats_hint();
    if hint.len() > 22 {
        "config stats.json"
    } else {
        hint
    }
}
