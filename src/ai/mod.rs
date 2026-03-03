pub mod behaviors;
pub mod opponent;

use bevy::prelude::*;
use behaviors::*;
use opponent::*;

pub struct AiPlugin;

impl Plugin for AiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AiState>()
            .add_systems(Startup, ai_startup)
            .add_systems(Update, (
                ai_economy_system,
                ai_build_system,
                ai_train_system,
                ai_military_system,
                ai_detection_system,
                ai_patrol_system,
                ai_flee_system,
            ));
    }
}
