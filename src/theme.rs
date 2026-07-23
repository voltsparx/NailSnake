use ratatui::style::{Color, Modifier, Style};

/// Colour-capability tiers that the game can target.
///
/// The theme constructor picks the best palette based on the detected or
/// user-forced capability, so the game looks good everywhere from a 1980s
/// VT220 to a modern 24-bit terminal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorMode {
    Auto,
    TrueColor,
    Ansi256,
    Basic,
}

/// Complete set of styles used across the TUI.
///
/// Every visual element - from the border to the snake tail - has a dedicated
/// field so it can be tuned independently without touching the rendering code.
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

impl Theme {
    pub fn new(mode: ColorMode) -> Self {
        match mode {
            ColorMode::Basic => Self::basic(),
            ColorMode::Ansi256 => Self::ansi256(),
            ColorMode::TrueColor | ColorMode::Auto => Self::true_color(),
        }
    }

    /// 24-bit truecolour palette - vibrant, modern-terminal look.
    ///
    /// Uses carefully chosen RGB values for a dark, rich aesthetic. Foreground
    /// and background colours are paired to create depth (e.g., a dark red
    /// background under the food to make it pop).
    fn true_color() -> Self {
        Self {
            border: Style::default().fg(Color::Rgb(183, 91, 45)),
            title: Style::default()
                .fg(Color::Rgb(244, 164, 96))
                .add_modifier(Modifier::BOLD),
            status_bar: Style::default()
                .bg(Color::Rgb(32, 25, 20))
                .fg(Color::Rgb(235, 214, 190)),
            sidebar: Style::default()
                .bg(Color::Rgb(26, 21, 18))
                .fg(Color::Rgb(225, 209, 194)),
            sidebar_title: Style::default()
                .fg(Color::Rgb(255, 184, 77))
                .add_modifier(Modifier::BOLD),
            score: Style::default().fg(Color::Rgb(255, 193, 120)),
            score_high: Style::default()
                .fg(Color::Rgb(255, 222, 140))
                .add_modifier(Modifier::BOLD),
            help: Style::default().fg(Color::Rgb(190, 168, 148)),
            help_key: Style::default()
                .fg(Color::Rgb(255, 135, 67))
                .add_modifier(Modifier::BOLD),
            food: Style::default()
                .fg(Color::Rgb(255, 221, 128))
                .bg(Color::Rgb(92, 41, 22))
                .add_modifier(Modifier::BOLD),
            snake_head: Style::default()
                .fg(Color::Rgb(22, 18, 15))
                .bg(Color::Rgb(255, 130, 64))
                .add_modifier(Modifier::BOLD),
            snake_body: Style::default()
                .fg(Color::Rgb(33, 24, 18))
                .bg(Color::Rgb(202, 82, 39)),
            snake_tail: Style::default()
                .fg(Color::Rgb(44, 32, 24))
                .bg(Color::Rgb(128, 58, 34)),
            grid: Style::default().fg(Color::Rgb(72, 55, 43)),
            overlay: Style::default()
                .bg(Color::Rgb(22, 17, 14))
                .fg(Color::Rgb(245, 229, 210)),
            paused: Style::default()
                .fg(Color::Rgb(255, 199, 95))
                .add_modifier(Modifier::BOLD),
            game_over: Style::default()
                .fg(Color::Rgb(255, 96, 64))
                .add_modifier(Modifier::BOLD),
            message: Style::default().fg(Color::Rgb(205, 184, 164)),
            menu_panel: Style::default()
                .bg(Color::Rgb(24, 18, 15))
                .fg(Color::Rgb(235, 216, 196)),
            menu_border: Style::default().fg(Color::Rgb(220, 99, 46)),
            menu_text: Style::default().fg(Color::Rgb(235, 216, 196)),
            menu_selected: Style::default()
                .fg(Color::Rgb(255, 236, 177))
                .bg(Color::Rgb(122, 52, 25))
                .add_modifier(Modifier::BOLD),
            menu_cursor: Style::default()
                .fg(Color::Rgb(255, 134, 68))
                .add_modifier(Modifier::BOLD),
            rain_head: Style::default()
                .fg(Color::Rgb(255, 218, 155))
                .add_modifier(Modifier::BOLD),
            rain_mid: Style::default().fg(Color::Rgb(194, 91, 45)),
            rain_tail: Style::default().fg(Color::Rgb(92, 48, 31)),
        }
    }

    /// 256-colour ANSI palette - good quality on terminals without 24-bit support.
    ///
    /// Maps the same intent to indexed colours. Where reasonable, named colours
    /// (`Color::Green`, `Color::Cyan`) are used so the terminal can map them
    /// to the user's chosen theme.
    fn ansi256() -> Self {
        Self {
            border: Style::default().fg(Color::Green),
            title: Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
            status_bar: Style::default()
                .bg(Color::Indexed(236))
                .fg(Color::Indexed(252)),
            sidebar: Style::default()
                .bg(Color::Indexed(235))
                .fg(Color::Indexed(252)),
            sidebar_title: Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
            score: Style::default().fg(Color::LightGreen),
            score_high: Style::default()
                .fg(Color::LightYellow)
                .add_modifier(Modifier::BOLD),
            help: Style::default().fg(Color::DarkGray),
            help_key: Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
            food: Style::default()
                .fg(Color::Red)
                .bg(Color::Indexed(52))
                .add_modifier(Modifier::BOLD),
            snake_head: Style::default()
                .fg(Color::Black)
                .bg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
            snake_body: Style::default().fg(Color::Black).bg(Color::Green),
            snake_tail: Style::default().fg(Color::Black).bg(Color::Indexed(22)),
            grid: Style::default().fg(Color::Indexed(238)),
            overlay: Style::default().bg(Color::Indexed(234)).fg(Color::White),
            paused: Style::default()
                .fg(Color::LightYellow)
                .add_modifier(Modifier::BOLD),
            game_over: Style::default()
                .fg(Color::LightRed)
                .add_modifier(Modifier::BOLD),
            message: Style::default().fg(Color::Gray),
            menu_panel: Style::default()
                .bg(Color::Indexed(234))
                .fg(Color::Indexed(223)),
            menu_border: Style::default().fg(Color::Indexed(166)),
            menu_text: Style::default().fg(Color::Indexed(223)),
            menu_selected: Style::default()
                .fg(Color::Indexed(230))
                .bg(Color::Indexed(130))
                .add_modifier(Modifier::BOLD),
            menu_cursor: Style::default()
                .fg(Color::Indexed(208))
                .add_modifier(Modifier::BOLD),
            rain_head: Style::default()
                .fg(Color::Indexed(222))
                .add_modifier(Modifier::BOLD),
            rain_mid: Style::default().fg(Color::Indexed(166)),
            rain_tail: Style::default().fg(Color::Indexed(94)),
        }
    }

    /// Basic 16-colour ANSI - the fallback for truly minimal environments.
    ///
    /// Only standard named colours (no Indexed or Rgb) so it works in any
    /// terminal emulator, even an xterm on a remote server.
    fn basic() -> Self {
        Self {
            border: Style::default().fg(Color::Green),
            title: Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
            status_bar: Style::default().fg(Color::White),
            sidebar: Style::default().fg(Color::White),
            sidebar_title: Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
            score: Style::default().fg(Color::Green),
            score_high: Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
            help: Style::default().fg(Color::DarkGray),
            help_key: Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
            food: Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            snake_head: Style::default()
                .fg(Color::Black)
                .bg(Color::Green)
                .add_modifier(Modifier::BOLD),
            snake_body: Style::default().fg(Color::Green),
            snake_tail: Style::default().fg(Color::Indexed(22)),
            grid: Style::default().fg(Color::DarkGray),
            overlay: Style::default().fg(Color::White),
            paused: Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
            game_over: Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            message: Style::default().fg(Color::Gray),
            menu_panel: Style::default().fg(Color::White),
            menu_border: Style::default().fg(Color::Yellow),
            menu_text: Style::default().fg(Color::White),
            menu_selected: Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
            menu_cursor: Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
            rain_head: Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
            rain_mid: Style::default().fg(Color::Red),
            rain_tail: Style::default().fg(Color::DarkGray),
        }
    }

    /// Select the style for a snake segment based on its position.
    ///
    /// The head is distinct (a different glyph + brighter colour).  The tail
    /// blends into the body for short snakes; for long snakes we transition
    /// gradually across the body length to create a subtle gradient effect.
    pub fn snake_segment(&self, index: usize, total: usize) -> Style {
        if index == 0 {
            return self.snake_head;
        }
        if index + 1 == total {
            return self.snake_tail;
        }
        if total <= 3 {
            return self.snake_body;
        }
        let t = index as f32 / (total - 1).max(1) as f32;
        if t < 0.35 {
            self.snake_body
        } else {
            self.snake_tail
        }
    }
}
