use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::game::{Game, GamePhase};
use crate::theme::ColorMode;

/// Difficulty presets.  Each variant maps to a base tick interval (in ms) that
/// determines how fast the snake moves.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Difficulty {
    Chill,
    #[default]
    Normal,
    Hard,
    Insane,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SnakeSkin {
    #[default]
    Blocky,
    Rhombus,
}

impl SnakeSkin {
    pub fn label(&self) -> &'static str {
        match self {
            SnakeSkin::Blocky => "Blocky",
            SnakeSkin::Rhombus => "Rhombus",
        }
    }

    pub fn next(self) -> Self {
        match self {
            SnakeSkin::Blocky => SnakeSkin::Rhombus,
            SnakeSkin::Rhombus => SnakeSkin::Blocky,
        }
    }
}

impl Difficulty {
    /// Base interval between game ticks, in milliseconds.
    ///
    /// Chill = 180 ms (~5.5 tps), Normal = 130 ms (~7.7 tps),
    /// Hard = 90 ms (~11 tps), Insane = 55 ms (~18 tps).
    /// The actual interval shrinks as the player eats food (see `Game::tick_interval_ms`).
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

    pub fn next(self) -> Self {
        match self {
            Difficulty::Chill => Difficulty::Normal,
            Difficulty::Normal => Difficulty::Hard,
            Difficulty::Hard => Difficulty::Insane,
            Difficulty::Insane => Difficulty::Chill,
        }
    }
}

/// On-disk statistics, serialized as JSON.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct PersistedStats {
    pub high_score: u32,
    pub games_played: u32,
}

pub const SETTINGS_VERSION: u32 = 1;
const MAX_ACTIVE_GAME_BYTES: u64 = 1_000_000;

/// Stable, human-readable representation of the five configurable bindings.
/// Crossterm's `KeyCode` is intentionally not serialized directly.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct PersistedKeyBindings {
    pub up: String,
    pub down: String,
    pub left: String,
    pub right: String,
    pub pause: String,
}

impl Default for PersistedKeyBindings {
    fn default() -> Self {
        Self {
            up: "Up".into(),
            down: "Down".into(),
            left: "Left".into(),
            right: "Right".into(),
            pause: "Space".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PersistedSettings {
    pub version: u32,
    pub difficulty: Difficulty,
    pub wrap_walls: bool,
    pub color_mode: ColorMode,
    pub show_grid: bool,
    pub random_maze: bool,
    pub snake_skin: SnakeSkin,
    pub custom_speed_ms: u64,
    pub key_bindings: PersistedKeyBindings,
}

impl Default for PersistedSettings {
    fn default() -> Self {
        Self {
            version: SETTINGS_VERSION,
            difficulty: Difficulty::Normal,
            wrap_walls: false,
            color_mode: ColorMode::Auto,
            show_grid: false,
            random_maze: false,
            snake_skin: SnakeSkin::Blocky,
            custom_speed_ms: Difficulty::Normal.tick_ms(),
            key_bindings: PersistedKeyBindings::default(),
        }
    }
}

/// Runtime configuration assembled from CLI flags, persisted stats, and
/// the detected terminal colour capability.
#[derive(Debug, Clone)]
pub struct GameConfig {
    pub difficulty: Difficulty,
    pub wrap_walls: bool,
    pub color_mode: ColorMode,
    pub show_grid: bool,
    pub random_maze: bool,
    pub snake_skin: SnakeSkin,
    pub custom_speed_ms: u64,
    pub stats: PersistedStats,
    pub key_bindings: PersistedKeyBindings,
    stats_path: PathBuf,
    session_path: PathBuf,
    settings_path: PathBuf,
    preferred_color_mode: ColorMode,
}

impl GameConfig {
    pub fn load(
        difficulty: Option<Difficulty>,
        wrap_walls: Option<bool>,
        color_mode: Option<ColorMode>,
        show_grid: Option<bool>,
    ) -> Result<Self> {
        let stats_path = stats_file_path()?;
        let session_path = session_file_path()?;
        let settings_path = settings_file_path()?;
        let stats = load_or_default(&stats_path, load_stats);
        let settings = load_or_default(&settings_path, load_settings);
        let preferred_color_mode = settings.color_mode;
        let requested_color = color_mode.unwrap_or(settings.color_mode);
        Ok(Self {
            difficulty: difficulty.unwrap_or(settings.difficulty),
            wrap_walls: wrap_walls.unwrap_or(settings.wrap_walls),
            color_mode: requested_color,
            show_grid: show_grid.unwrap_or(settings.show_grid),
            random_maze: settings.random_maze,
            snake_skin: settings.snake_skin,
            custom_speed_ms: settings.custom_speed_ms.clamp(45, 260),
            stats,
            key_bindings: settings.key_bindings,
            stats_path,
            session_path,
            settings_path,
            preferred_color_mode,
        })
    }

    pub fn save_settings(&self) -> Result<()> {
        let settings = PersistedSettings {
            version: SETTINGS_VERSION,
            difficulty: self.difficulty,
            wrap_walls: self.wrap_walls,
            color_mode: self.preferred_color_mode,
            show_grid: self.show_grid,
            random_maze: self.random_maze,
            snake_skin: self.snake_skin,
            custom_speed_ms: self.custom_speed_ms.clamp(45, 260),
            key_bindings: self.key_bindings.clone(),
        };
        atomic_write_json(&self.settings_path, &settings, "settings")
    }

    /// Persist current stats to disk as JSON.
    fn save_stats(&self) -> Result<()> {
        if let Some(parent) = self.stats_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("creating stats dir {}", parent.display()))?;
        }
        atomic_write_json(&self.stats_path, &self.stats, "stats")
    }

    /// Record a finished game.  Updates high score if applicable and persists.
    /// Returns `true` if a new high score was set.
    pub fn record_game(&mut self, score: u32) -> Result<bool> {
        self.stats.games_played = self.stats.games_played.saturating_add(1);
        let new_record = score > self.stats.high_score;
        if new_record {
            self.stats.high_score = score;
        }
        self.save_stats()?;
        Ok(new_record)
    }

    pub fn has_saved_game(&self) -> bool {
        self.session_path.is_file() && self.load_active_game().ok().flatten().is_some()
    }

    pub fn load_active_game(&self) -> Result<Option<Game>> {
        if !self.session_path.is_file() {
            return Ok(None);
        }

        let size = fs::metadata(&self.session_path)
            .with_context(|| {
                format!(
                    "reading saved game metadata from {}",
                    self.session_path.display()
                )
            })?
            .len();
        if size > MAX_ACTIVE_GAME_BYTES {
            anyhow::bail!(
                "saved game at {} is too large ({size} bytes; limit is {MAX_ACTIVE_GAME_BYTES})",
                self.session_path.display()
            );
        }

        let data = fs::read_to_string(&self.session_path)
            .with_context(|| format!("reading saved game from {}", self.session_path.display()))?;
        let mut game: Game = match serde_json::from_str(&data) {
            Ok(game) => game,
            Err(_) => {
                let _ = self.clear_active_game();
                return Ok(None);
            }
        };

        if !game.is_valid_for_resume() {
            let _ = self.clear_active_game();
            return Ok(None);
        }

        if game.phase == GamePhase::Running {
            game.phase = GamePhase::Paused;
        }
        Ok(Some(game))
    }

    pub fn save_active_game(&self, game: &Game) -> Result<()> {
        if !game.is_valid_for_resume() {
            return self.clear_active_game();
        }

        if let Some(parent) = self.session_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("creating session dir {}", parent.display()))?;
        }

        atomic_write_json(&self.session_path, game, "saved game")
    }

    pub fn clear_active_game(&self) -> Result<()> {
        if self.session_path.is_file() {
            fs::remove_file(&self.session_path)
                .with_context(|| format!("removing saved game {}", self.session_path.display()))?;
        }
        Ok(())
    }
}

/// Resolve the platform-appropriate stats file path.
///
/// Uses the `directories` crate which follows the XDG spec on Linux, the
/// macOS convention, and the `%APPDATA%` pattern on Windows - all without
/// conditional compilation.
fn stats_file_path() -> Result<PathBuf> {
    let dirs = ProjectDirs::from("", "", "NailSnake")
        .context("could not resolve config directory for NailSnake")?;
    Ok(dirs.config_dir().join("stats.json"))
}

fn session_file_path() -> Result<PathBuf> {
    let dirs = ProjectDirs::from("", "", "NailSnake")
        .context("could not resolve config directory for NailSnake")?;
    Ok(dirs.data_local_dir().join("active-game.json"))
}

fn settings_file_path() -> Result<PathBuf> {
    let dirs = ProjectDirs::from("", "", "NailSnake")
        .context("could not resolve config directory for NailSnake")?;
    Ok(dirs.config_dir().join("settings.json"))
}

fn load_stats(path: &PathBuf) -> Result<PersistedStats> {
    let data = fs::read_to_string(path)
        .with_context(|| format!("reading stats from {}", path.display()))?;
    Ok(serde_json::from_str(&data)?)
}

fn load_settings(path: &PathBuf) -> Result<PersistedSettings> {
    let data = fs::read_to_string(path)
        .with_context(|| format!("reading settings from {}", path.display()))?;
    let settings: PersistedSettings = serde_json::from_str(&data)
        .with_context(|| format!("parsing settings from {}", path.display()))?;
    if settings.version > SETTINGS_VERSION {
        anyhow::bail!(
            "settings file uses unsupported version {}",
            settings.version
        );
    }
    Ok(settings)
}

fn load_or_default<T: Default>(path: &PathBuf, loader: fn(&PathBuf) -> Result<T>) -> T {
    match loader(path) {
        Ok(value) => value,
        Err(_) => {
            // Preserve malformed user data for recovery instead of silently
            // overwriting it with defaults on the next save.
            if path.is_file() {
                let suffix = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|duration| duration.as_secs())
                    .unwrap_or(0);
                let backup = path.with_extension(format!("corrupt-{suffix}.json"));
                let _ = fs::rename(path, backup);
            }
            T::default()
        }
    }
}

fn atomic_write_json<T: Serialize>(path: &PathBuf, value: &T, label: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("creating {label} directory {}", parent.display()))?;
    }
    let temporary = path.with_extension("tmp");
    let json = serde_json::to_vec_pretty(value)?;
    let mut file = File::create(&temporary)
        .with_context(|| format!("writing temporary {label} {}", temporary.display()))?;
    file.write_all(&json)?;
    file.write_all(b"\n")?;
    file.sync_all()?;
    drop(file);

    #[cfg(windows)]
    if path.exists() {
        // Windows does not permit rename-overwrite. The old file remains intact
        // until the temporary replacement has been fully written and synced.
        fs::remove_file(path).with_context(|| format!("replacing {label} {}", path.display()))?;
    }
    fs::rename(&temporary, path)
        .with_context(|| format!("moving {label} into {}", path.display()))?;
    Ok(())
}
