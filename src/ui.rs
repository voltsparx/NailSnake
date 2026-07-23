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

use crate::config::GameConfig;
use crate::game::{Game, GamePhase};
use crate::theme::Theme;

pub use layout::compute_layout;
pub use model::{MenuView, SIDEBAR_WIDTH};

pub fn render(
    frame: &mut Frame,
    game: &Game,
    config: &GameConfig,
    theme: &Theme,
    os_label: &str,
    menu_view: Option<&MenuView>,
    frame_tick: u64,
) {
    let areas = compute_layout(frame.area());

    board::render_board(
        frame,
        game,
        areas.board,
        theme,
        config.show_grid,
        config.snake_skin,
    );
    sidebar::render_sidebar(frame, game, config, areas.sidebar, theme, os_label);
    status::render_status_bar(frame, game, config, areas.status, theme, os_label);

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
            "Press R to restart - Q to quit",
            theme.game_over,
        ),
        GamePhase::Menu | GamePhase::Running => {}
    }
}
