pub mod behaviors;
pub mod opponent;

use bevy::prelude::*;
use behaviors::*;
use opponent::*;
use crate::GameState;
use crate::map::generation::generate_map_config;

pub struct AiPlugin;

impl Plugin for AiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AiState>()
            .add_systems(OnEnter(GameState::InGame), ai_startup.after(generate_map_config))
            .add_systems(Update, (
                ai_economy_system,
                ai_build_system,
                ai_train_system,
                ai_military_system,
                ai_scout_system,
                ai_defense_system,
                ai_rebuild_system,
                ai_analyze_player_system,
                ai_detection_system,
                ai_patrol_system,
                ai_flee_system,
            ).run_if(in_state(GameState::InGame)));
    }
}
