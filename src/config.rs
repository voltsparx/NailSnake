use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::theme::ColorMode;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Difficulty {
    Chill,
    #[default]
    Normal,
    Hard,
    Insane,
}

impl Difficulty {
    pub fn tick_ms(&self) -> u64 {
        match self {
            Difficulty::Chill => 180,
            Difficulty::Normal => 130,
            Difficulty::Hard => 90,
            Difficulty::Insane => 55,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Difficulty::Chill => "Chill",
            Difficulty::Normal => "Normal",
            Difficulty::Hard => "Hard",
            Difficulty::Insane => "Insane",
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct PersistedStats {
    pub high_score: u32,
    pub games_played: u32,
}

#[derive(Debug, Clone)]
pub struct GameConfig {
    pub difficulty: Difficulty,
    pub wrap_walls: bool,
    pub color_mode: ColorMode,
    pub show_grid: bool,
    pub stats: PersistedStats,
    stats_path: PathBuf,
}

impl GameConfig {
    pub fn load(
        difficulty: Difficulty,
        wrap_walls: bool,
        color_mode: ColorMode,
        show_grid: bool,
    ) -> Result<Self> {
        let stats_path = stats_file_path()?;
        let stats = load_stats(&stats_path).unwrap_or_default();
        Ok(Self {
            difficulty,
            wrap_walls,
            color_mode,
            show_grid,
            stats,
            stats_path,
        })
    }

    pub fn save_stats(&self) -> Result<()> {
        if let Some(parent) = self.stats_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("creating stats dir {}", parent.display()))?;
        }
        let json = serde_json::to_string_pretty(&self.stats)?;
        fs::write(&self.stats_path, json)
            .with_context(|| format!("writing stats to {}", self.stats_path.display()))?;
        Ok(())
    }

    pub fn record_game(&mut self, score: u32) -> Result<bool> {
        self.stats.games_played += 1;
        let new_record = score > self.stats.high_score;
        if new_record {
            self.stats.high_score = score;
        }
        self.save_stats()?;
        Ok(new_record)
    }
}

fn stats_file_path() -> Result<PathBuf> {
    let dirs = ProjectDirs::from("", "", "NailSnake")
        .context("could not resolve config directory for NailSnake")?;
    Ok(dirs.config_dir().join("stats.json"))
}

fn load_stats(path: &PathBuf) -> Result<PersistedStats> {
    let data = fs::read_to_string(path)
        .with_context(|| format!("reading stats from {}", path.display()))?;
    Ok(serde_json::from_str(&data)?)
}
