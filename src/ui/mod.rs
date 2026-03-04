pub mod hud;
pub mod minimap;
pub mod game_over;
pub mod lobby;
pub mod stats;

use bevy::prelude::*;
use hud::*;
use minimap::*;
use game_over::*;
use lobby::*;
use crate::GameState;

pub struct GameUiPlugin;

impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameResult>()
            .init_resource::<stats::GameStats>()
            // Lobby (Menu state)
            .add_systems(OnEnter(GameState::Menu), setup_lobby)
            .add_systems(OnExit(GameState::Menu), teardown_lobby)
            .add_systems(Update, (
                lobby_map_type_buttons,
                lobby_player_count_buttons,
                lobby_start_button,
                lobby_button_hover,
                lobby_civ_buttons,
                lobby_game_mode_buttons,
            ).run_if(in_state(GameState::Menu)))
            // HUD (InGame state)
            .add_systems(OnEnter(GameState::InGame), (setup_hud, setup_minimap))
            .add_systems(Update, (
                update_resource_display,
                update_unit_info_panel,
                update_age_display,
                update_building_panel,
                rebuild_action_panel,
                handle_train_button_clicks,
                handle_research_button_clicks,
                handle_build_button_clicks,
                button_hover_system,
                update_minimap,
                minimap_click,
                check_win_lose,
                show_game_over,
                update_countdown_display,
                stats::track_game_time,
            ).run_if(in_state(GameState::InGame)));
    }
}
