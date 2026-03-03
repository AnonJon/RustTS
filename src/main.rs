mod camera;
mod map;
mod units;
mod resources;
mod buildings;
mod ai;
mod ui;
mod audio;

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
