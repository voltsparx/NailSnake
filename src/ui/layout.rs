use ratatui::layout::{Constraint, Direction, Layout, Rect};

use super::model::{LayoutAreas, SIDEBAR_WIDTH};

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

pub fn inset(area: Rect, pad: u16) -> Rect {
    Rect {
        x: area.x + pad,
        y: area.y + pad,
        width: area.width.saturating_sub(pad * 2),
        height: area.height.saturating_sub(pad * 2),
    }
}

pub fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
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
