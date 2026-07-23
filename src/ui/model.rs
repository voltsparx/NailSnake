use ratatui::layout::Rect;

pub const SIDEBAR_WIDTH: u16 = 28;

pub struct LayoutAreas {
    pub board: Rect,
    pub sidebar: Rect,
    pub status: Rect,
}

pub struct MenuView {
    pub title: String,
    pub items: Vec<String>,
    pub selected: Option<usize>,
    pub hint: String,
}
