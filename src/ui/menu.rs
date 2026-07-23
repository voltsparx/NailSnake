use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;

use crate::config::GameConfig;
use crate::game::Game;
use crate::theme::Theme;

use super::animation::draw_oxide_rain;
use super::layout::{centered_rect, inset};
use super::model::MenuView;
use super::skins::animated_snake_preview;

pub fn render_main_menu(
    frame: &mut Frame,
    area: Rect,
    menu: &MenuView,
    game: &Game,
    config: &GameConfig,
    theme: &Theme,
    frame_tick: u64,
) {
    frame.render_widget(Clear, area);

    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7),
            Constraint::Min(8),
            Constraint::Length(4),
        ])
        .split(inset(area, 1));

    let main = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(34), Constraint::Length(30)])
        .split(outer[1]);

    render_logo(frame, outer[0], theme);
    render_menu_panel(frame, main[1], menu, theme, 100, 100);
    render_attract_panel(frame, main[0], game, config, theme, frame_tick);
    render_menu_footer(frame, outer[2], theme);
}

pub fn render_menu_panel(
    frame: &mut Frame,
    area: Rect,
    menu: &MenuView,
    theme: &Theme,
    percent_x: u16,
    percent_y: u16,
) {
    let panel = if percent_x < 100 || percent_y < 100 {
        centered_rect(percent_x, percent_y, area)
    } else {
        area
    };
    frame.render_widget(Clear, panel);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.menu_border)
        .title(Span::styled(
            format!(" {} ", menu.title),
            theme.sidebar_title,
        ))
        .style(theme.menu_panel);
    let inner = inset(block.inner(panel), 1);
    frame.render_widget(block, panel);

    let mut lines = Vec::new();
    for (index, item) in menu.items.iter().enumerate() {
        let selected = menu.selected == Some(index);
        let marker = if selected { "> " } else { "  " };
        let style = if selected {
            theme.menu_selected
        } else {
            theme.menu_text
        };
        lines.push(Line::from(vec![
            Span::styled(marker, theme.menu_cursor),
            Span::styled(item.clone(), style),
        ]));
        lines.push(Line::from(""));
    }
    lines.push(Line::from(Span::styled(menu.hint.clone(), theme.message)));

    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: true }), inner);
}

fn render_logo(frame: &mut Frame, area: Rect, theme: &Theme) {
    let logo = vec![
        Line::from(Span::styled(
            " _   _       _ _ ____              _        ",
            theme.title,
        )),
        Line::from(Span::styled(
            "| \\ | | __ _(_) / ___| _ __   __ _| | _____ ",
            theme.title,
        )),
        Line::from(Span::styled(
            "|  \\| |/ _` | | \\___ \\| '_ \\ / _` | |/ / _ \\",
            theme.title,
        )),
        Line::from(Span::styled(
            "| |\\  | (_| | | |___) | | | | (_| |   <  __/",
            theme.title,
        )),
        Line::from(Span::styled(
            "|_| \\_|\\__,_|_|_|____/|_| |_|\\__,_|_|\\_\\___|",
            theme.title,
        )),
    ];

    frame.render_widget(
        Paragraph::new(logo)
            .alignment(Alignment::Center)
            .style(theme.menu_text),
        area,
    );
}

fn render_attract_panel(
    frame: &mut Frame,
    area: Rect,
    game: &Game,
    config: &GameConfig,
    theme: &Theme,
    frame_tick: u64,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.border)
        .title(Span::styled(" Arcade Preview ", theme.sidebar_title))
        .style(theme.sidebar);
    let inner = inset(block.inner(area), 1);
    frame.render_widget(block, area);
    draw_oxide_rain(frame, inner, theme, frame_tick);

    let mut lines = vec![
        Line::from(Span::styled(
            "Rust-powered terminal Snake",
            theme.score_high,
        )),
        Line::from(""),
        Line::from(format!("Difficulty   {}", config.difficulty.label())),
        Line::from(format!(
            "Walls        {}",
            if config.wrap_walls {
                "Teleport"
            } else {
                "Solid"
            }
        )),
        Line::from(format!(
            "Grid         {}",
            if config.show_grid { "On" } else { "Off" }
        )),
        Line::from(format!("Best score   {}", config.stats.high_score)),
        Line::from(format!("Games played {}", config.stats.games_played)),
        Line::from(""),
        Line::from(Span::styled(
            "Eat fast. Turn clean. Never reverse.",
            theme.help_key,
        )),
        Line::from(""),
    ];

    lines.push(Line::from(format!(
        "Skin         {}",
        config.snake_skin.label()
    )));
    let snake_row = animated_snake_preview(frame_tick, config.snake_skin);
    lines.push(Line::from(Span::styled(snake_row, theme.snake_body)));
    lines.push(Line::from(format!(
        "Tick speed   {} ms",
        game.tick_interval_ms()
    )));

    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: true }), inner);
}

fn render_menu_footer(frame: &mut Frame, area: Rect, theme: &Theme) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.menu_border)
        .style(theme.menu_panel);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let text = Line::from(vec![
        Span::styled("Enter", theme.help_key),
        Span::raw(" select  "),
        Span::styled("Up/Down", theme.help_key),
        Span::raw(" move  "),
        Span::styled("Left/Right", theme.help_key),
        Span::raw(" change  "),
        Span::styled("Q/Esc", theme.help_key),
        Span::raw(" quit"),
    ]);
    frame.render_widget(
        Paragraph::new(text)
            .alignment(Alignment::Center)
            .style(theme.menu_panel),
        inner,
    );
}
