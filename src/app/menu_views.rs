use super::key_bindings::key_label;
use super::menu_state::MenuScreen;
use super::App;
use nailsnake::game::GamePhase;
use nailsnake::ui;

impl App {
    pub(super) fn active_menu_view(&self) -> Option<ui::MenuView> {
        if self.show_help {
            return Some(ui::MenuView {
                title: "Help".to_string(),
                items: vec![
                    "Move: Arrow keys or WASD".to_string(),
                    "Pause: Space or Enter".to_string(),
                    "Restart: R".to_string(),
                    "Help: H or ?".to_string(),
                    "Quit: Q or Esc".to_string(),
                    "Press Enter, H, ?, or Esc to close".to_string(),
                ],
                selected: None,
                hint: "NailSnake keeps terminal controls simple and fast.".to_string(),
            });
        }

        if let Some(capture) = self.capture_key {
            return Some(ui::MenuView {
                title: "Controls".to_string(),
                items: vec![format!("Press a key for {}", capture.label())],
                selected: None,
                hint: "Esc is allowed, but avoid binding it if you still want quick quit."
                    .to_string(),
            });
        }

        match self.game.phase {
            GamePhase::Menu => Some(self.menu_view()),
            GamePhase::Paused => Some(ui::MenuView {
                title: "Pause Menu".to_string(),
                items: vec![
                    "Resume".to_string(),
                    "Restart".to_string(),
                    "Help".to_string(),
                    "Quit Game".to_string(),
                ],
                selected: Some(self.pause_menu_index),
                hint: "Enter selects. Space resumes instantly.".to_string(),
            }),
            _ => None,
        }
    }

    fn menu_view(&self) -> ui::MenuView {
        match self.menu_screen {
            MenuScreen::Main => ui::MenuView {
                title: "Main Menu".to_string(),
                items: self.main_menu_items(),
                selected: Some(self.main_menu_index),
                hint: "Enter selects. Q/Esc quits. Ctrl+C saves and exits.".to_string(),
            },
            MenuScreen::Settings => ui::MenuView {
                title: "Game Settings".to_string(),
                items: vec![
                    format!("Difficulty     {}", self.config.difficulty.label()),
                    format!("Speed          {} ms", self.config.custom_speed_ms),
                    format!(
                        "Walls          {}",
                        if self.config.wrap_walls {
                            "Teleport"
                        } else {
                            "Solid"
                        }
                    ),
                    format!(
                        "Random Maze    {}",
                        if self.config.random_maze { "On" } else { "Off" }
                    ),
                    format!("Snake Skin     {}", self.config.snake_skin.label()),
                    format!(
                        "Grid           {}",
                        if self.config.show_grid { "On" } else { "Off" }
                    ),
                    "Back".to_string(),
                ],
                selected: Some(self.settings_index),
                hint: "Left/Right changes values. Enter toggles/selects.".to_string(),
            },
            MenuScreen::Controls => ui::MenuView {
                title: "Controls".to_string(),
                items: vec![
                    format!("Key Up      {}", key_label(self.keys.up)),
                    format!("Key Down    {}", key_label(self.keys.down)),
                    format!("Key Left    {}", key_label(self.keys.left)),
                    format!("Key Right   {}", key_label(self.keys.right)),
                    format!("Key Pause   {}", key_label(self.keys.pause)),
                    "Reset to Defaults".to_string(),
                    "Back".to_string(),
                ],
                selected: Some(self.controls_index),
                hint: "Enter changes the selected key. Defaults use arrow keys.".to_string(),
            },
            MenuScreen::Credits => ui::MenuView {
                title: "Credits".to_string(),
                items: vec![
                    "Author: voltsparx (Niyor Kalita)".to_string(),
                    "Contact: voltsparx@gmail.com".to_string(),
                    "Repo: github.com/voltsparx/NailSnake".to_string(),
                    "Version: v1.0".to_string(),
                    "Press Enter or Esc to go back.".to_string(),
                ],
                selected: None,
                hint: "Thanks for playing NailSnake.".to_string(),
            },
        }
    }

    fn main_menu_items(&self) -> Vec<String> {
        let mut items = Vec::with_capacity(if self.saved_game_available { 7 } else { 6 });
        if self.saved_game_available {
            items.push("Continue".to_string());
        }
        items.extend([
            "Arcade Mode".to_string(),
            "Game Settings".to_string(),
            "Help".to_string(),
            "Controls".to_string(),
            "Credits".to_string(),
            "Quit".to_string(),
        ]);
        items
    }
}
