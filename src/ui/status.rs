use ratatui::layout::{Alignment, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::config::GameConfig;
use crate::game::{Game, GamePhase};
use crate::theme::Theme;

pub fn render_status_bar(
    frame: &mut Frame,
    game: &Game,
    config: &GameConfig,
    area: Rect,
    theme: &Theme,
    os_label: &str,
    notice: Option<&str>,
) {
    let phase_hint = match game.phase {
        GamePhase::Running => "Playing",
        GamePhase::Paused => "Paused",
        GamePhase::GameOver => "Dead",
        GamePhase::Menu => "Press Enter",
    };

    let full = Line::from(vec![
        Span::styled(" NailSnake ", theme.title),
        Span::raw(" | "),
        Span::raw(os_label),
        Span::raw(" | "),
        Span::raw(phase_hint),
        Span::raw(" | "),
        Span::raw(format!("score {}", game.score)),
        Span::raw(" | "),
        Span::raw(format!("best {}", config.stats.high_score)),
        Span::raw(" | "),
        Span::raw(format!("games {}", config.stats.games_played)),
    ]);

    let text = if let Some(notice) = notice {
        Line::from(format!(" NailSnake | {notice}"))
    } else if area.width < 70 {
        Line::from(format!(" NailSnake | {phase_hint} | {}", game.score))
    } else {
        full
    };
    let bar = Paragraph::new(text)
        .style(theme.status_bar)
        .alignment(Alignment::Left);
    frame.render_widget(bar, area);
}
