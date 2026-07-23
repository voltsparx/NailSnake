use ratatui::layout::{Alignment, Rect};
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;

use crate::theme::Theme;

use super::layout::centered_rect;

pub fn render_overlay(
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
        .title(Span::styled(
            title,
            title_style.add_modifier(Modifier::BOLD),
        ))
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
