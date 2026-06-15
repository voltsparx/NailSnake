use std::io::stdout;

use anyhow::{bail, Result};
use crossterm::tty::IsTty;

use crate::theme::ColorMode;

/// Target operating system, determined at compile time via `cfg!`.
///
/// We use this for platform-specific colour detection and stats path hints.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Os {
    Windows,
    Linux,
    Macos,
    Other,
}

pub fn current_os() -> Os {
    if cfg!(target_os = "windows") {
        Os::Windows
    } else if cfg!(target_os = "linux") {
        Os::Linux
    } else if cfg!(target_os = "macos") {
        Os::Macos
    } else {
        Os::Other
    }
}

pub fn os_label() -> &'static str {
    match current_os() {
        Os::Windows => "Windows",
        Os::Linux => "Linux",
        Os::Macos => "macOS",
        Os::Other => "Unix",
    }
}

/// Minimum terminal size for a usable layout (board + sidebar + status bar).
pub const MIN_TERM_WIDTH: u16 = 60;
pub const MIN_TERM_HEIGHT: u16 = 22;

/// Refuse to run if stdout is piped, redirected, or otherwise non-interactive.
///
/// The game needs raw mode and an alternate screen — neither works in a CI
/// log or a file redirect.
pub fn ensure_interactive_terminal() -> Result<()> {
    if !stdout().is_tty() {
        bail!(
            "NailSnake requires an interactive terminal (TTY).\n\
             Run it from Windows Terminal, PowerShell, cmd, or a Linux/macOS shell — not from a pipe or CI log."
        );
    }
    Ok(())
}

pub fn ensure_terminal_size(width: u16, height: u16) -> Result<()> {
    if width < MIN_TERM_WIDTH || height < MIN_TERM_HEIGHT {
        bail!(
            "Terminal too small ({width}x{height}). NailSnake needs at least \
             {MIN_TERM_WIDTH}x{MIN_TERM_HEIGHT} columns×rows.\n\
             Resize your terminal window and try again."
        );
    }
    Ok(())
}

/// Detect the terminal's colour capability.
///
/// When `requested` is `Auto`, we probe environment variables (`COLORTERM`,
/// `TERM`, `TRUECOLOR`) and fall back to platform defaults.  Modern Windows
/// Terminal advertises `WT_SESSION`, which we treat as a truecolor signal.
pub fn detect_color_mode(requested: ColorMode) -> ColorMode {
    if requested != ColorMode::Auto {
        return requested;
    }

    if env_has_truecolor() {
        return ColorMode::TrueColor;
    }

    if env_has_256_color() {
        return ColorMode::Ansi256;
    }

    match current_os() {
        Os::Windows => {
            if std::env::var("WT_SESSION").is_ok()
                || std::env::var("TERM_PROGRAM")
                    .map(|v| v.eq_ignore_ascii_case("WindowsTerminal"))
                    .unwrap_or(false)
            {
                ColorMode::TrueColor
            } else {
                ColorMode::Ansi256
            }
        }
        Os::Macos | Os::Linux | Os::Other => ColorMode::Ansi256,
    }
}

fn env_has_truecolor() -> bool {
    std::env::var("COLORTERM")
        .map(|v| {
            let lower = v.to_lowercase();
            lower.contains("truecolor") || lower.contains("24bit")
        })
        .unwrap_or(false)
        || std::env::var("TRUECOLOR").is_ok()
}

fn env_has_256_color() -> bool {
    std::env::var("TERM")
        .map(|t| t.contains("256") || t.contains("xterm"))
        .unwrap_or(false)
        || std::env::var("COLORTERM").is_ok()
}

pub fn stats_hint() -> &'static str {
    match current_os() {
        Os::Windows => r"%APPDATA%\NailSnake\stats.json",
        Os::Macos => "~/Library/Application Support/NailSnake/stats.json",
        Os::Linux | Os::Other => "~/.config/NailSnake/stats.json",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn os_label_is_non_empty() {
        assert!(!os_label().is_empty());
    }

    #[test]
    fn auto_color_respects_explicit_request() {
        assert_eq!(detect_color_mode(ColorMode::Basic), ColorMode::Basic);
    }
}
