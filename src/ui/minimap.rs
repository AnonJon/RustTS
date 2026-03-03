use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy::asset::RenderAssetUsages;
use crate::units::components::*;
use crate::buildings::components::Building;
use crate::map::{MAP_WIDTH, MAP_HEIGHT, TILE_SIZE};
use crate::camera::MainCamera;

const MINIMAP_PX: u32 = 180;
const MINIMAP_PADDING: f32 = 10.0;
const BG_COLOR: [u8; 4] = [15, 40, 15, 230];

#[derive(Component)]
pub struct Minimap;

#[derive(Resource)]
pub struct MinimapImage(pub Handle<Image>);

pub fn setup_minimap(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let data = vec![BG_COLOR[0]; (MINIMAP_PX * MINIMAP_PX * 4) as usize];
    let mut img = Image::new(
        bevy::render::render_resource::Extent3d {
            width: MINIMAP_PX,
            height: MINIMAP_PX,
            depth_or_array_layers: 1,
        },
        bevy::render::render_resource::TextureDimension::D2,
        data,
        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    clear_minimap(&mut img);

    let handle = images.add(img);

    commands.insert_resource(MinimapImage(handle.clone()));

    commands.spawn((
        Minimap,
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(MINIMAP_PADDING),
            right: Val::Px(MINIMAP_PADDING),
            width: Val::Px(MINIMAP_PX as f32),
            height: Val::Px(MINIMAP_PX as f32),
            ..default()
        },
        ImageNode {
            image: handle,
            ..default()
        },
    ));
}

fn clear_minimap(img: &mut Image) {
    let Some(ref mut data) = img.data else { return };
    for chunk in data.chunks_exact_mut(4) {
        chunk[0] = BG_COLOR[0];
        chunk[1] = BG_COLOR[1];
        chunk[2] = BG_COLOR[2];
        chunk[3] = BG_COLOR[3];
    }
}

fn set_pixel(img: &mut Image, x: i32, y: i32, color: [u8; 4]) {
    if x < 0 || y < 0 || x >= MINIMAP_PX as i32 || y >= MINIMAP_PX as i32 {
        return;
    }
    let Some(ref mut data) = img.data else { return };
    let idx = ((y as u32 * MINIMAP_PX + x as u32) * 4) as usize;
    if idx + 3 < data.len() {
        data[idx] = color[0];
        data[idx + 1] = color[1];
        data[idx + 2] = color[2];
        data[idx + 3] = color[3];
    }
}

fn draw_rect(img: &mut Image, cx: i32, cy: i32, half_w: i32, half_h: i32, color: [u8; 4]) {
    for dy in -half_h..=half_h {
        for dx in -half_w..=half_w {
            set_pixel(img, cx + dx, cy + dy, color);
        }
    }
}

fn draw_rect_outline(img: &mut Image, cx: i32, cy: i32, half_w: i32, half_h: i32, color: [u8; 4]) {
    for dx in -half_w..=half_w {
        set_pixel(img, cx + dx, cy - half_h, color);
        set_pixel(img, cx + dx, cy + half_h, color);
    }
    for dy in -half_h..=half_h {
        set_pixel(img, cx - half_w, cy + dy, color);
        set_pixel(img, cx + half_w, cy + dy, color);
    }
}

fn world_to_minimap(world_pos: Vec2) -> (i32, i32) {
    let iso_w = MAP_WIDTH as f32 * TILE_SIZE;
    let iso_h = (MAP_WIDTH + MAP_HEIGHT) as f32 * TILE_SIZE / 4.0;
    let x = ((world_pos.x / iso_w) * MINIMAP_PX as f32) as i32;
    let y = ((1.0 - world_pos.y / iso_h) * MINIMAP_PX as f32) as i32;
    (x.clamp(0, MINIMAP_PX as i32 - 1), y.clamp(0, MINIMAP_PX as i32 - 1))
}

pub fn update_minimap(
    minimap_img: Res<MinimapImage>,
    mut images: ResMut<Assets<Image>>,
    units: Query<(&Transform, &Team), With<Unit>>,
    buildings: Query<(&Transform, &Team), With<Building>>,
    camera_q: Query<(&Transform, &Projection), With<MainCamera>>,
    windows: Query<&Window>,
) {
    let Some(img) = images.get_mut(&minimap_img.0) else { return };
    clear_minimap(img);

    for (transform, team) in &buildings {
        let pos = transform.translation.truncate();
        let (mx, my) = world_to_minimap(pos);
        let color = if team.0 == 0 {
            [60, 100, 220, 255]
        } else {
            [220, 60, 60, 255]
        };
        draw_rect(img, mx, my, 2, 2, color);
    }

    for (transform, team) in &units {
        let pos = transform.translation.truncate();
        let (mx, my) = world_to_minimap(pos);
        let color = if team.0 == 0 {
            [100, 160, 255, 255]
        } else {
            [255, 100, 100, 255]
        };
        draw_rect(img, mx, my, 1, 1, color);
    }

    if let Ok((cam_tf, cam_proj)) = camera_q.single() {
        let cam_pos = cam_tf.translation.truncate();
        let scale = if let Projection::Orthographic(ref ortho) = *cam_proj {
            ortho.scale
        } else {
            1.0
        };

        let Ok(window) = windows.single() else { return };
        let view_w = window.width() * scale;
        let view_h = window.height() * scale;

        let iso_w = MAP_WIDTH as f32 * TILE_SIZE;
        let iso_h = (MAP_WIDTH + MAP_HEIGHT) as f32 * TILE_SIZE / 4.0;

        let half_w = ((view_w / iso_w) * MINIMAP_PX as f32 / 2.0) as i32;
        let half_h = ((view_h / iso_h) * MINIMAP_PX as f32 / 2.0) as i32;

        let (cx, cy) = world_to_minimap(cam_pos);
        draw_rect_outline(img, cx, cy, half_w, half_h, [255, 255, 255, 200]);
    }
}

pub fn minimap_click(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    minimap_q: Query<&GlobalTransform, With<Minimap>>,
    mut camera_q: Query<&mut Transform, With<MainCamera>>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok(window) = windows.single() else { return };
    let Some(cursor) = window.cursor_position() else { return };
    let Ok(minimap_gt) = minimap_q.single() else { return };

    let mm_center = minimap_gt.translation().truncate();
    let half = MINIMAP_PX as f32 / 2.0;

    let local_x = cursor.x - (mm_center.x - half);
    let local_y = cursor.y - (mm_center.y - half);

    if local_x < 0.0 || local_x > MINIMAP_PX as f32
        || local_y < 0.0 || local_y > MINIMAP_PX as f32
    {
        return;
    }

    let iso_w = MAP_WIDTH as f32 * TILE_SIZE;
    let iso_h = (MAP_WIDTH + MAP_HEIGHT) as f32 * TILE_SIZE / 4.0;
    let world_x = (local_x / MINIMAP_PX as f32) * iso_w;
    let world_y = (1.0 - local_y / MINIMAP_PX as f32) * iso_h;

    let Ok(mut cam_tf) = camera_q.single_mut() else { return };
    cam_tf.translation.x = world_x;
    cam_tf.translation.y = world_y;
}
