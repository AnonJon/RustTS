pub mod hud;
pub mod minimap;
pub mod game_over;

use bevy::prelude::*;
use hud::*;
use minimap::*;
use game_over::*;

pub struct GameUiPlugin;

impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameResult>()
            .add_systems(Startup, (setup_hud, setup_minimap))
            .add_systems(Update, (
                update_resource_display,
                update_unit_info_panel,
                update_age_display,
                update_building_panel,
                update_minimap,
                minimap_click,
                check_win_lose,
                show_game_over,
            ));
    }
}
