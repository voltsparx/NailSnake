use ratatui::style::{Color, Modifier, Style};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorMode {
    Auto,
    TrueColor,
    Ansi256,
    Basic,
}

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
}

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
            border: Style::default().fg(Color::Rgb(80, 200, 120)),
            title: Style::default()
                .fg(Color::Rgb(255, 215, 100))
                .add_modifier(Modifier::BOLD),
            status_bar: Style::default()
                .bg(Color::Rgb(30, 35, 45))
                .fg(Color::Rgb(180, 190, 210)),
            sidebar: Style::default()
                .bg(Color::Rgb(24, 28, 36))
                .fg(Color::Rgb(200, 205, 220)),
            sidebar_title: Style::default()
                .fg(Color::Rgb(120, 180, 255))
                .add_modifier(Modifier::BOLD),
            score: Style::default().fg(Color::Rgb(100, 220, 160)),
            score_high: Style::default()
                .fg(Color::Rgb(255, 180, 80))
                .add_modifier(Modifier::BOLD),
            help: Style::default().fg(Color::Rgb(150, 160, 180)),
            help_key: Style::default()
                .fg(Color::Rgb(255, 120, 160))
                .add_modifier(Modifier::BOLD),
            food: Style::default()
                .fg(Color::Rgb(255, 90, 90))
                .bg(Color::Rgb(60, 20, 25))
                .add_modifier(Modifier::BOLD),
            snake_head: Style::default()
                .fg(Color::Rgb(20, 30, 25))
                .bg(Color::Rgb(90, 255, 150))
                .add_modifier(Modifier::BOLD),
            snake_body: Style::default()
                .fg(Color::Rgb(20, 40, 30))
                .bg(Color::Rgb(50, 190, 110)),
            snake_tail: Style::default()
                .fg(Color::Rgb(30, 50, 40))
                .bg(Color::Rgb(35, 130, 80)),
            grid: Style::default().fg(Color::Rgb(40, 48, 58)),
            overlay: Style::default()
                .bg(Color::Rgb(15, 18, 24))
                .fg(Color::Rgb(230, 235, 245)),
            paused: Style::default()
                .fg(Color::Rgb(255, 220, 100))
                .add_modifier(Modifier::BOLD),
            game_over: Style::default()
                .fg(Color::Rgb(255, 100, 100))
                .add_modifier(Modifier::BOLD),
            message: Style::default().fg(Color::Rgb(160, 170, 190)),
        }
    }

    fn ansi256() -> Self {
        Self {
            border: Style::default().fg(Color::Green),
            title: Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            status_bar: Style::default().bg(Color::Indexed(236)).fg(Color::Indexed(252)),
            sidebar: Style::default().bg(Color::Indexed(235)).fg(Color::Indexed(252)),
            sidebar_title: Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            score: Style::default().fg(Color::LightGreen),
            score_high: Style::default()
                .fg(Color::LightYellow)
                .add_modifier(Modifier::BOLD),
            help: Style::default().fg(Color::DarkGray),
            help_key: Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
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
            game_over: Style::default().fg(Color::LightRed).add_modifier(Modifier::BOLD),
            message: Style::default().fg(Color::Gray),
        }
    }

    fn basic() -> Self {
        Self {
            border: Style::default().fg(Color::Green),
            title: Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            status_bar: Style::default().fg(Color::White),
            sidebar: Style::default().fg(Color::White),
            sidebar_title: Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            score: Style::default().fg(Color::Green),
            score_high: Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
            help: Style::default().fg(Color::DarkGray),
            help_key: Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
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
        }
    }

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
