use ratatui::style::{Color, Modifier, Style};

use super::model::{ColorMode, Theme};

impl Theme {
    pub fn new(mode: ColorMode) -> Self {
        match mode {
            ColorMode::Basic => Self::basic(),
            ColorMode::Ansi256 => Self::ansi256(),
            ColorMode::TrueColor | ColorMode::Auto => Self::true_color(),
        }
    }

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
                .fg(Color::Rgb(255, 62, 48))
                .bg(Color::Rgb(78, 18, 14))
                .add_modifier(Modifier::BOLD),
            snake_head: Style::default()
                .fg(Color::Rgb(22, 18, 15))
                .bg(Color::Rgb(238, 83, 30))
                .add_modifier(Modifier::BOLD),
            snake_body: Style::default()
                .fg(Color::Rgb(255, 202, 158))
                .bg(Color::Rgb(160, 61, 30)),
            snake_tail: Style::default()
                .fg(Color::Rgb(244, 134, 77))
                .bg(Color::Rgb(86, 39, 27)),
            wall: Style::default()
                .fg(Color::Rgb(117, 82, 60))
                .bg(Color::Rgb(31, 25, 22)),
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
                .fg(Color::LightRed)
                .bg(Color::Indexed(52))
                .add_modifier(Modifier::BOLD),
            snake_head: Style::default()
                .fg(Color::Black)
                .bg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
            snake_body: Style::default().fg(Color::Black).bg(Color::Green),
            snake_tail: Style::default().fg(Color::Black).bg(Color::Indexed(22)),
            wall: Style::default()
                .fg(Color::Indexed(95))
                .bg(Color::Indexed(235)),
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
            wall: Style::default().fg(Color::DarkGray),
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
}
