use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct GameStats {
    pub units_created: u32,
    pub units_lost: u32,
    pub enemy_units_killed: u32,
    pub buildings_built: u32,
    pub buildings_lost: u32,
    pub enemy_buildings_destroyed: u32,
    pub food_gathered: u32,
    pub wood_gathered: u32,
    pub gold_gathered: u32,
    pub stone_gathered: u32,
    pub conversions: u32,
    pub game_time: f32,
}

impl GameStats {
    pub fn total_resources(&self) -> u32 {
        self.food_gathered + self.wood_gathered + self.gold_gathered + self.stone_gathered
    }

    pub fn military_score(&self) -> u32 {
        self.enemy_units_killed * 20 + self.enemy_buildings_destroyed * 50
    }

    pub fn economy_score(&self) -> u32 {
        self.total_resources() / 10
    }

    pub fn total_score(&self) -> u32 {
        self.military_score() + self.economy_score() + self.buildings_built * 10
    }
}

pub fn track_game_time(
    mut stats: ResMut<GameStats>,
    time: Res<Time>,
) {
    stats.game_time += time.delta_secs();
}

pub fn format_time(seconds: f32) -> String {
    let m = (seconds / 60.0) as u32;
    let s = (seconds % 60.0) as u32;
    format!("{m}:{s:02}")
}
