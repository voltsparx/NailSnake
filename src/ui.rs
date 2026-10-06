mod animation;
mod board;
mod layout;
mod menu;
mod model;
mod overlay;
mod sidebar;
mod skins;
mod status;

use ratatui::Frame;

use crate::config::{GameConfig, SnakeSkin};
use crate::game::{Game, GamePhase};
use crate::theme::{ColorMode, Theme};

pub use layout::compute_layout;
pub use model::{LayoutMode, MenuView, SIDEBAR_WIDTH};

#[allow(clippy::too_many_arguments)]
pub fn render(
    frame: &mut Frame,
    game: &Game,
    config: &GameConfig,
    theme: &Theme,
    os_label: &str,
    menu_view: Option<&MenuView>,
    frame_tick: u64,
    notice: Option<&str>,
    controls: &[String],
) {
    let areas = compute_layout(frame.area());

    if areas.mode == LayoutMode::TooSmall {
        overlay::render_overlay(
            frame,
            frame.area(),
            theme,
            " TERMINAL TOO SMALL ",
            "Resize to at least 50 columns by 18 rows. Your game is preserved.",
            theme.paused,
        );
        return;
    }

    let skin = if config.color_mode == ColorMode::Basic {
        SnakeSkin::Blocky
    } else {
        config.snake_skin
    };
    board::render_board(frame, game, areas.board, theme, config.show_grid, skin);
    if areas.mode != LayoutMode::Minimal {
        sidebar::render_sidebar(
            frame,
            game,
            config,
            areas.sidebar,
            theme,
            os_label,
            controls,
        );
    }
    status::render_status_bar(frame, game, config, areas.status, theme, os_label, notice);

    if game.phase == GamePhase::Menu {
        if let Some(menu_view) = menu_view {
            menu::render_main_menu(
                frame,
                frame.area(),
                menu_view,
                game,
                config,
                theme,
                frame_tick,
            );
        }
        return;
    }

    match game.phase {
        GamePhase::Paused => {
            if let Some(menu_view) = menu_view {
                menu::render_menu_panel(frame, areas.board, menu_view, theme, 62, 54);
            }
        }
        GamePhase::GameOver => overlay::render_overlay(
            frame,
            areas.board,
            theme,
            " GAME OVER ",
            "R/Enter restart  |  Esc/Q main menu",
            theme.game_over,
        ),
        GamePhase::Menu | GamePhase::Running => {}
    }
}
