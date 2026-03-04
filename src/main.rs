mod camera;
mod map;
mod units;
mod resources;
mod buildings;
mod ai;
mod ui;
mod audio;
pub mod civilization;

use bevy::prelude::*;
use bevy::window::WindowResolution;
use camera::CameraPlugin;
use map::MapPlugin;
use units::UnitPlugin;
use resources::ResourcePlugin;
use buildings::BuildingPlugin;
use ai::AiPlugin;
use ui::GameUiPlugin;
use audio::GameAudioPlugin;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    Menu,
    InGame,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameMode {
    Conquest,
    WonderRace,
    Regicide,
}

impl Default for GameMode {
    fn default() -> Self { Self::Conquest }
}

impl GameMode {
    pub const ALL: [GameMode; 3] = [
        GameMode::Conquest,
        GameMode::WonderRace,
        GameMode::Regicide,
    ];

    pub fn label(self) -> &'static str {
        match self {
            GameMode::Conquest => "Conquest",
            GameMode::WonderRace => "Wonder Race",
            GameMode::Regicide => "Regicide",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            GameMode::Conquest => "Destroy all enemy buildings and units to win.",
            GameMode::WonderRace => "Build a Wonder and defend it for 200 seconds, or collect all relics for 200 seconds.",
            GameMode::Regicide => "Protect your King! Lose your King and you lose the game.",
        }
    }
}

#[derive(Resource)]
pub struct GameSpeed(pub f32);

impl Default for GameSpeed {
    fn default() -> Self { Self(1.0) }
}

impl GameSpeed {
    pub const MIN: f32 = 0.5;
    pub const MAX: f32 = 3.0;
    pub const STEP: f32 = 0.25;
}

#[derive(Resource)]
pub struct GameSettings {
    pub map_type: map::generation::MapType,
    pub num_players: usize,
    pub game_mode: GameMode,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            map_type: map::generation::MapType::Arabia,
            num_players: 2,
            game_mode: GameMode::Conquest,
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Age of Rust".into(),
                resolution: WindowResolution::new(1280, 720),
                ..default()
            }),
            ..default()
        }))
        .init_state::<GameState>()
        .init_resource::<GameSettings>()
        .init_resource::<GameSpeed>()
        .init_resource::<civilization::PlayerCivilization>()
        .add_plugins(CameraPlugin)
        .add_plugins(MapPlugin)
        .add_plugins(UnitPlugin)
        .add_plugins(ResourcePlugin)
        .add_plugins(BuildingPlugin)
        .add_plugins(AiPlugin)
        .add_plugins(GameUiPlugin)
        .add_plugins(GameAudioPlugin)
        .run();
}
