use bevy::prelude::*;
use bevy::input::mouse::MouseWheel;
use bevy::window::PrimaryWindow;
use crate::map::{MAP_WIDTH, MAP_HEIGHT, TILE_SIZE};
use crate::map::generation::{MapConfig, generate_map_config};
use crate::GameState;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ZoomState>()
            .add_systems(Startup, spawn_menu_camera)
            .add_systems(OnEnter(GameState::InGame), reposition_camera.after(generate_map_config))
            .add_systems(Update, (
                camera_edge_pan,
                camera_zoom,
                camera_zoom_smooth,
                camera_keyboard_pan,
                camera_clamp,
            ).run_if(in_state(GameState::InGame)));
    }
}

#[derive(Component)]
pub struct MainCamera;

#[derive(Resource)]
pub struct ZoomState {
    pub target_scale: f32,
}

impl Default for ZoomState {
    fn default() -> Self {
        Self { target_scale: DEFAULT_ZOOM }
    }
}

const EDGE_PAN_SPEED: f32 = 900.0;
const EDGE_PAN_MARGIN: f32 = 20.0;
const KEYBOARD_PAN_SPEED: f32 = 600.0;
const ZOOM_SPEED: f32 = 0.08;
const MIN_ZOOM: f32 = 0.35;
const MAX_ZOOM: f32 = 1.2;
const DEFAULT_ZOOM: f32 = 1.2;
const ZOOM_LERP_SPEED: f32 = 8.0;

fn spawn_menu_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        MainCamera,
        Transform::from_xyz(0.0, 0.0, 999.9),
    ));
}

fn reposition_camera(
    config: Res<MapConfig>,
    mut camera_q: Query<(&mut Transform, &mut Projection), With<MainCamera>>,
) {
    let start = config.player_base().to_world();
    for (mut transform, mut projection) in &mut camera_q {
        transform.translation.x = start.x;
        transform.translation.y = start.y;
        if let Projection::Orthographic(ref mut ortho) = *projection {
            ortho.scale = DEFAULT_ZOOM;
        }
    }
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
    mut zoom_state: ResMut<ZoomState>,
) {
    for ev in scroll_events.read() {
        let zoom_delta = -ev.y * ZOOM_SPEED;
        zoom_state.target_scale = (zoom_state.target_scale + zoom_delta).clamp(MIN_ZOOM, MAX_ZOOM);
    }
}

fn camera_zoom_smooth(
    zoom_state: Res<ZoomState>,
    mut camera_q: Query<(&mut Projection, &mut Transform, &Camera), With<MainCamera>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    time: Res<Time>,
) {
    let Ok((mut projection, mut transform, camera)) = camera_q.single_mut() else {
        return;
    };
    let Projection::Orthographic(ref mut ortho) = *projection else {
        return;
    };

    let old_scale = ortho.scale;
    let diff = zoom_state.target_scale - old_scale;
    if diff.abs() < 0.001 {
        ortho.scale = zoom_state.target_scale;
        return;
    }

    let new_scale = old_scale + diff * (ZOOM_LERP_SPEED * time.delta_secs()).min(1.0);
    ortho.scale = new_scale;

    // Zoom toward cursor position
    let Ok(window) = windows.single() else { return };
    let Some(cursor_pos) = window.cursor_position() else { return };
    let Ok(cursor_world) = camera.viewport_to_world_2d(&GlobalTransform::from(*transform), cursor_pos) else { return };

    let scale_ratio = new_scale / old_scale;
    let cam_pos = transform.translation.truncate();
    let new_cam = cursor_world + (cam_pos - cursor_world) * scale_ratio;
    transform.translation.x = new_cam.x;
    transform.translation.y = new_cam.y;
}

fn camera_clamp(
    mut camera_q: Query<&mut Transform, With<MainCamera>>,
) {
    let Ok(mut transform) = camera_q.single_mut() else { return };

    // Diamond iso: x ∈ [0, 64*(W-1+H-1)], y ∈ [-32*(W-1), 32*(H-1)]
    let max_x = (MAP_WIDTH as f32 - 1.0 + MAP_HEIGHT as f32 - 1.0) * (TILE_SIZE / 2.0);
    let min_y = -((MAP_WIDTH as f32 - 1.0) * (TILE_SIZE / 4.0));
    let max_y = (MAP_HEIGHT as f32 - 1.0) * (TILE_SIZE / 4.0);

    transform.translation.x = transform.translation.x.clamp(0.0, max_x);
    transform.translation.y = transform.translation.y.clamp(min_y, max_y);
}
