use ratatui::style::Style;

/// Colour-capability tiers that the game can target.
///
/// The theme constructor picks the best palette based on the detected or
/// user-forced capability, so the game looks good on modern and minimal
/// terminals.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorMode {
    Auto,
    TrueColor,
    Ansi256,
    Basic,
}

/// Complete set of styles used across the TUI.
///
/// Every visual element has a dedicated field so it can be tuned independently
/// without touching the rendering code.
#[derive(Debug, Clone)]
pub struct Theme {
    pub border: Style,
    pub title: Style,
    pub status_bar: Style,
    pub sidebar: Style,
    pub sidebar_title: Style,
    pub score: Style,
    pub score_high: Style,
    pub help: Style,
    pub help_key: Style,
    pub food: Style,
    pub snake_head: Style,
    pub snake_body: Style,
    pub snake_tail: Style,
    pub wall: Style,
    pub grid: Style,
    pub overlay: Style,
    pub paused: Style,
    pub game_over: Style,
    pub message: Style,
    pub menu_panel: Style,
    pub menu_border: Style,
    pub menu_text: Style,
    pub menu_selected: Style,
    pub menu_cursor: Style,
    pub rain_head: Style,
    pub rain_mid: Style,
    pub rain_tail: Style,
}
