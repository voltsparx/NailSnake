use ratatui::layout::Rect;

pub const SIDEBAR_WIDTH: u16 = 28;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutMode {
    Full,
    Compact,
    Minimal,
    TooSmall,
}

pub struct LayoutAreas {
    pub board: Rect,
    pub sidebar: Rect,
    pub status: Rect,
    pub mode: LayoutMode,
}

pub struct MenuView {
    pub title: String,
    pub items: Vec<String>,
    pub selected: Option<usize>,
    pub hint: String,
}
