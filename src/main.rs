pub mod app;

use anyhow::Result;
use clap::{Parser, ValueEnum};
use nailsnake::config::{Difficulty, GameConfig};
use nailsnake::platform::{detect_color_mode, ensure_interactive_terminal};
use nailsnake::theme::ColorMode;

use crate::app::App;

#[derive(Parser)]
#[command(
    name = "nailsnake",
    version,
    about = "NailSnake — cross-platform terminal Snake (Windows, Linux, macOS)",
    long_about = "NailSnake is a full-screen TUI snake game inspired by nsnake, \
                  built in Rust for safety and smooth terminal rendering on Windows, \
                  Linux, and macOS. See `man nailsnake` for the full manual.",
    after_help = "Full manual: man nailsnake\nAuthor: Voltsparx <voltsparx@gmail.com>"
)]
struct Cli {
    #[arg(short, long, default_value = "normal")]
    difficulty: DifficultyArg,

    #[arg(short, long)]
    wrap: bool,

    #[arg(short, long, default_value = "auto")]
    color: ColorArg,

    #[arg(short, long)]
    grid: bool,
}

#[derive(Clone, ValueEnum)]
enum DifficultyArg {
    Chill,
    Normal,
    Hard,
    Insane,
}

impl From<DifficultyArg> for Difficulty {
    fn from(value: DifficultyArg) -> Self {
        match value {
            DifficultyArg::Chill => Difficulty::Chill,
            DifficultyArg::Normal => Difficulty::Normal,
            DifficultyArg::Hard => Difficulty::Hard,
            DifficultyArg::Insane => Difficulty::Insane,
        }
    }
}

#[derive(Clone, ValueEnum)]
enum ColorArg {
    Auto,
    Truecolor,
    Ansi256,
    Basic,
}

impl From<ColorArg> for ColorMode {
    fn from(value: ColorArg) -> Self {
        match value {
            ColorArg::Auto => ColorMode::Auto,
            ColorArg::Truecolor => ColorMode::TrueColor,
            ColorArg::Ansi256 => ColorMode::Ansi256,
            ColorArg::Basic => ColorMode::Basic,
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    ensure_interactive_terminal()?;

    let color_mode = detect_color_mode(cli.color.into());

    let config = GameConfig::load(cli.difficulty.into(), cli.wrap, color_mode, cli.grid)?;
    let mut app = App::new(config)?;
    app.run()
}
