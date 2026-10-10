use ratatui::layout::{Alignment, Rect};
use ratatui::text::Span;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::config::SnakeSkin;
use crate::game::{Game, Point};
use crate::theme::Theme;

use super::layout::inset;
use super::skins::snake_glyph;

const CELL_WIDTH: u16 = 2;

pub fn board_fits(game: &Game, area: Rect) -> bool {
    area.width >= game.width.saturating_mul(CELL_WIDTH).saturating_add(2)
        && area.height >= game.height.saturating_add(2)
}

pub fn render_board(
    frame: &mut Frame,
    game: &Game,
    area: Rect,
    theme: &Theme,
    show_grid: bool,
    skin: SnakeSkin,
) {
    // Terminal characters are roughly twice as tall as wide. Two columns per
    // logical cell make vertical and horizontal movement look equally fast.
    let cell_w = CELL_WIDTH;
    let cell_h = 1;
    // The frame is the physical game boundary. Keeping it snug around the
    // logical board ensures a solid-wall collision is always visible.
    let frame_width = game.width.saturating_mul(cell_w).saturating_add(2);
    let frame_height = game.height.saturating_mul(cell_h).saturating_add(2);
    let board_area = Rect {
        x: area.x + area.width.saturating_sub(frame_width.min(area.width)) / 2,
        y: area.y + area.height.saturating_sub(frame_height.min(area.height)) / 2,
        width: frame_width.min(area.width),
        height: frame_height.min(area.height),
    };
    let inner = inset(board_area, 1);
    let board_w = inner.width;
    let board_h = inner.height;

    let used_w = game.width * cell_w;
    let used_h = game.height * cell_h;
    let offset_x = inner.x + (board_w.saturating_sub(used_w)) / 2;
    let offset_y = inner.y + (board_h.saturating_sub(used_h)) / 2;

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.border)
        .title(Span::styled(" NailSnake ", theme.title))
        .title_alignment(Alignment::Center);

    frame.render_widget(block, board_area);

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
        if segment.x >= game.width || segment.y >= game.height {
            continue;
        }
        let style = theme.snake_segment(i, game.snake.len());
        draw_cell(
            frame,
            offset_x,
            offset_y,
            *segment,
            cell_w,
            cell_h,
            style,
            snake_glyph(skin, i, game.snake.len()),
        );
    }

    for wall in &game.walls {
        if wall.x >= game.width || wall.y >= game.height {
            continue;
        }
        draw_cell(
            frame, offset_x, offset_y, *wall, cell_w, cell_h, theme.wall, "XX",
        );
    }

    if game.food.x < game.width && game.food.y < game.height {
        draw_cell(
            frame, offset_x, offset_y, game.food, cell_w, cell_h, theme.food, "**",
        );
    }
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
            let dot = Paragraph::new("..").style(theme.grid);
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
    if rect.width == 0
        || rect.height == 0
        || rect.x >= frame.area().right()
        || rect.y >= frame.area().bottom()
    {
        return;
    }
    let cell = Paragraph::new(glyph).style(style);
    frame.render_widget(cell, rect);
}

fn cell_rect(ox: u16, oy: u16, point: Point, cw: u16, ch: u16) -> Rect {
    Rect {
        x: ox + point.x * cw,
        y: oy + point.y * ch,
        width: cw,
        height: ch,
    }
}
