pub mod app;

use anyhow::Result;
use clap::{ArgAction, Parser, ValueEnum};
use nailsnake::config::{Difficulty, GameConfig};
use nailsnake::platform::{detect_color_mode, ensure_interactive_terminal};
use nailsnake::theme::ColorMode;

use crate::app::App;

#[derive(Parser)]
#[command(
    name = "nailsnake",
    version = env!("CARGO_PKG_VERSION"),
    about = "NailSnake - cross-platform terminal Snake (Windows, Linux, macOS)",
    long_about = "NailSnake is a full-screen TUI snake game inspired by nsnake, \
                  built in Rust for safety and smooth terminal rendering on Windows, \
                  Linux, and macOS. See `man nailsnake` for the full manual.",
    after_help = "Full manual: man nailsnake\nAuthor: Voltsparx <voltsparx@gmail.com>"
)]
struct Cli {
    #[arg(short, long)]
    difficulty: Option<DifficultyArg>,

    #[arg(short, long, action = ArgAction::SetTrue)]
    wrap: Option<bool>,

    #[arg(short, long)]
    color: Option<ColorArg>,

    #[arg(short, long, action = ArgAction::SetTrue)]
    grid: Option<bool>,

    #[arg(long, action = ArgAction::SetTrue)]
    about: bool,
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

fn main() {
    if let Err(error) = run() {
        eprintln!("NailSnake error: {error}");
        for cause in error.chain().skip(1) {
            eprintln!("  caused by: {cause}");
        }
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    if cli.about {
        print_about();
        return Ok(());
    }

    ensure_interactive_terminal()?;

    let color_mode = cli.color.map(Into::into);

    let mut config = GameConfig::load(
        cli.difficulty.map(Into::into),
        cli.wrap,
        color_mode,
        cli.grid,
    )?;
    config.color_mode = detect_color_mode(config.color_mode);
    let mut app = App::new(config)?;
    app.run()
}

fn print_about() {
    println!(
        "NailSnake v{}\n\
         Cross-platform terminal Snake written in Rust.\n\n\
         Author: voltsparx (Niyor Kalita)\n\
         Contact: voltsparx@gmail.com\n\
         Repository: https://github.com/voltsparx/NailSnake\n\
         License: MIT\n\n\
         Features: nsnake-style menus, Rust-themed terminal UI, persistent stats,\n\
         Continue-game recovery, configurable speed/keybinds/skins, and installers\n\
         for Linux, Termux, macOS, and Windows.",
        env!("CARGO_PKG_VERSION")
    );
}
