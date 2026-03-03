use bevy::prelude::*;
use bevy::input::mouse::MouseWheel;
use crate::map::{MAP_WIDTH, MAP_HEIGHT, TILE_SIZE};
use crate::map::generation::MapConfig;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera)
            .add_systems(Update, (
                camera_edge_pan,
                camera_zoom,
                camera_keyboard_pan,
                camera_clamp,
            ));
    }
}

#[derive(Component)]
pub struct MainCamera;

const EDGE_PAN_SPEED: f32 = 500.0;
const EDGE_PAN_MARGIN: f32 = 20.0;
const KEYBOARD_PAN_SPEED: f32 = 600.0;
const ZOOM_SPEED: f32 = 0.1;
const MIN_ZOOM: f32 = 0.25;
const MAX_ZOOM: f32 = 4.0;

fn spawn_camera(mut commands: Commands, config: Res<MapConfig>) {
    let start = config.player_base.to_world();
    commands.spawn((
        Camera2d,
        MainCamera,
        Transform::from_xyz(start.x, start.y, 999.9),
    ));
}

fn camera_edge_pan(
    windows: Query<&Window>,
    mut camera_q: Query<&mut Transform, With<MainCamera>>,
    time: Res<Time>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let Ok(mut transform) = camera_q.single_mut() else {
        return;
    };

    let w = window.width();
    let h = window.height();
    let speed = EDGE_PAN_SPEED * time.delta_secs();

    let mut delta = Vec2::ZERO;
    if cursor.x < EDGE_PAN_MARGIN {
        delta.x -= speed;
    }
    if cursor.x > w - EDGE_PAN_MARGIN {
        delta.x += speed;
    }
    if cursor.y < EDGE_PAN_MARGIN {
        delta.y += speed;
    }
    if cursor.y > h - EDGE_PAN_MARGIN {
        delta.y -= speed;
    }

    transform.translation.x += delta.x;
    transform.translation.y += delta.y;
}

fn camera_keyboard_pan(
    keys: Res<ButtonInput<KeyCode>>,
    mut camera_q: Query<&mut Transform, With<MainCamera>>,
    time: Res<Time>,
) {
    let Ok(mut transform) = camera_q.single_mut() else {
        return;
    };
    let speed = KEYBOARD_PAN_SPEED * time.delta_secs();
    let mut delta = Vec2::ZERO;

    if keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp) {
        delta.y += speed;
    }
    if keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown) {
        delta.y -= speed;
    }
    if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) {
        delta.x -= speed;
    }
    if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) {
        delta.x += speed;
    }

    transform.translation.x += delta.x;
    transform.translation.y += delta.y;
}

fn camera_zoom(
    mut scroll_events: MessageReader<MouseWheel>,
    mut camera_q: Query<&mut Projection, With<MainCamera>>,
) {
    let Ok(mut projection) = camera_q.single_mut() else {
        return;
    };
    let Projection::Orthographic(ref mut ortho) = *projection else {
        return;
    };
    for ev in scroll_events.read() {
        let zoom_delta = -ev.y * ZOOM_SPEED;
        ortho.scale = (ortho.scale + zoom_delta).clamp(MIN_ZOOM, MAX_ZOOM);
    }
}

fn camera_clamp(
    mut camera_q: Query<&mut Transform, With<MainCamera>>,
) {
    let Ok(mut transform) = camera_q.single_mut() else { return };

    let iso_w = MAP_WIDTH as f32 * TILE_SIZE;
    let iso_h = (MAP_WIDTH + MAP_HEIGHT) as f32 * TILE_SIZE / 4.0;

    transform.translation.x = transform.translation.x.clamp(0.0, iso_w);
    transform.translation.y = transform.translation.y.clamp(0.0, iso_h);
}
