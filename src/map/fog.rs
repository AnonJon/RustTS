use bevy::prelude::*;
use bevy::asset::RenderAssetUsages;
use bevy::image::{ImageSampler, ImageFilterMode, ImageSamplerDescriptor};
use super::{GridPosition, MAP_WIDTH, MAP_HEIGHT};
use crate::units::components::{Unit, Team};
use crate::buildings::components::Building;

/// Line of sight radius in tiles.
#[derive(Component)]
pub struct LineOfSight(pub u32);

/// Marker to exclude the fog sprite from depth sorting.
#[derive(Component)]
pub struct FogOverlayMarker;

const FOG_TEX_W: u32 = 192;
const FOG_TEX_H: u32 = 96;
const FOG_WORLD_X_MIN: f32 = -64.0;
const FOG_WORLD_Y_MAX: f32 = 1536.0;
const FOG_WORLD_W: f32 = 6144.0;
const FOG_WORLD_H: f32 = 3072.0;
const FOG_PX_SIZE: f32 = 32.0; // world units per texture pixel

#[derive(Resource)]
pub struct FogOverlay {
    pub texture: Handle<Image>,
}

#[derive(Resource)]
pub struct FogOfWar {
    /// Whether a tile has ever been seen by the player.
    pub explored: Vec<Vec<bool>>,
    /// How many player entities currently have vision on this tile.
    /// Tile is visible when visible[x][y] > 0.
    pub visible: Vec<Vec<u8>>,
}

impl Default for FogOfWar {
    fn default() -> Self {
        let w = MAP_WIDTH as usize;
        let h = MAP_HEIGHT as usize;
        Self {
            explored: vec![vec![false; h]; w],
            visible: vec![vec![0u8; h]; w],
        }
    }
}

impl FogOfWar {
    pub fn is_visible(&self, x: i32, y: i32) -> bool {
        if x < 0 || y < 0 || x >= MAP_WIDTH as i32 || y >= MAP_HEIGHT as i32 {
            return false;
        }
        self.visible[x as usize][y as usize] > 0
    }

    pub fn is_explored(&self, x: i32, y: i32) -> bool {
        if x < 0 || y < 0 || x >= MAP_WIDTH as i32 || y >= MAP_HEIGHT as i32 {
            return false;
        }
        self.explored[x as usize][y as usize]
    }
}

/// Recalculate the visibility grid each frame based on player entities.
pub fn update_fog_system(
    mut fog: ResMut<FogOfWar>,
    units: Query<(&Transform, &Team, &LineOfSight), With<Unit>>,
    buildings: Query<(&GridPosition, &Team, &Building, Option<&LineOfSight>)>,
) {
    let w = MAP_WIDTH as usize;
    let h = MAP_HEIGHT as usize;

    // Clear visibility (but keep explored)
    for x in 0..w {
        for y in 0..h {
            fog.visible[x][y] = 0;
        }
    }

    // Mark tiles visible from player units
    for (transform, team, los) in &units {
        if team.0 != 0 { continue; }
        let grid = GridPosition::from_world(transform.translation.truncate());
        mark_visible(&mut fog, grid.x, grid.y, los.0 as i32);
    }

    // Mark tiles visible from player buildings
    for (grid, team, building, los) in &buildings {
        if team.0 != 0 { continue; }
        let (tw, th) = building.kind.tile_size();
        let center_x = grid.x + tw as i32 / 2;
        let center_y = grid.y + th as i32 / 2;
        let radius = los.map_or(
            match building.kind {
                crate::buildings::components::BuildingKind::TownCenter => 8,
                _ => 4,
            },
            |l| l.0 as i32,
        );
        mark_visible(&mut fog, center_x, center_y, radius);
    }
}

fn mark_visible(fog: &mut FogOfWar, cx: i32, cy: i32, radius: i32) {
    let w = MAP_WIDTH as i32;
    let h = MAP_HEIGHT as i32;
    let r2 = radius * radius;

    for dx in -radius..=radius {
        for dy in -radius..=radius {
            if dx * dx + dy * dy > r2 { continue; }
            let x = cx + dx;
            let y = cy + dy;
            if x < 0 || y < 0 || x >= w || y >= h { continue; }
            let ux = x as usize;
            let uy = y as usize;
            fog.visible[ux][uy] = fog.visible[ux][uy].saturating_add(1);
            fog.explored[ux][uy] = true;
        }
    }
}

/// Hide/show enemy entities based on fog of war visibility.
pub fn apply_fog_visibility_system(
    mut units: Query<(&Transform, &Team, &mut Visibility), With<Unit>>,
    mut buildings: Query<(&GridPosition, &Team, &mut Visibility), (With<Building>, Without<Unit>)>,
    fog: Res<FogOfWar>,
) {
    for (transform, team, mut vis) in &mut units {
        if team.0 == 0 { continue; } // always show own units
        let grid = GridPosition::from_world(transform.translation.truncate());
        *vis = if fog.is_visible(grid.x, grid.y) {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }

    for (grid, team, mut vis) in &mut buildings {
        if team.0 == 0 { continue; }
        *vis = if fog.is_visible(grid.x, grid.y) {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

pub fn setup_fog_overlay(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
) {
    let pixel_count = (FOG_TEX_W * FOG_TEX_H * 4) as usize;
    let mut data = vec![0u8; pixel_count];
    // Start fully opaque black
    for i in 0..(FOG_TEX_W * FOG_TEX_H) as usize {
        data[i * 4 + 3] = 255;
    }

    let mut img = Image::new(
        bevy::render::render_resource::Extent3d {
            width: FOG_TEX_W,
            height: FOG_TEX_H,
            depth_or_array_layers: 1,
        },
        bevy::render::render_resource::TextureDimension::D2,
        data,
        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    );
    img.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        mag_filter: ImageFilterMode::Linear,
        min_filter: ImageFilterMode::Linear,
        ..default()
    });

    let texture = images.add(img);
    let center_x = FOG_WORLD_X_MIN + FOG_WORLD_W / 2.0;
    let center_y = FOG_WORLD_Y_MAX - FOG_WORLD_H / 2.0;

    commands.spawn((
        FogOverlayMarker,
        Sprite {
            image: texture.clone(),
            custom_size: Some(Vec2::new(FOG_WORLD_W, FOG_WORLD_H)),
            ..default()
        },
        Transform::from_xyz(center_x, center_y, 100.0),
    ));

    commands.insert_resource(FogOverlay { texture });
}

pub fn update_fog_texture(
    fog: Res<FogOfWar>,
    overlay: Res<FogOverlay>,
    mut images: ResMut<Assets<Image>>,
) {
    let Some(img) = images.get_mut(&overlay.texture) else { return };

    let w = FOG_TEX_W as usize;
    let h = FOG_TEX_H as usize;
    let map_w = MAP_WIDTH as i32;
    let map_h = MAP_HEIGHT as i32;

    let mut alpha_buf = vec![0u8; w * h];

    for py in 0..h {
        for px in 0..w {
            let world_x = FOG_WORLD_X_MIN + (px as f32 + 0.5) * FOG_PX_SIZE;
            let world_y = FOG_WORLD_Y_MAX - (py as f32 + 0.5) * FOG_PX_SIZE;
            let grid = GridPosition::from_world(Vec2::new(world_x, world_y));

            let a = if grid.x < 0 || grid.y < 0 || grid.x >= map_w || grid.y >= map_h {
                255
            } else {
                let gx = grid.x as usize;
                let gy = grid.y as usize;
                if fog.visible[gx][gy] > 0 {
                    0
                } else if fog.explored[gx][gy] {
                    140
                } else {
                    255
                }
            };
            alpha_buf[py * w + px] = a;
        }
    }

    // 3-pass box blur on the alpha channel
    let mut tmp = vec![0u8; w * h];
    for _ in 0..3 {
        for y in 0..h {
            for x in 0..w {
                let mut sum: u32 = 0;
                let mut count: u32 = 0;
                for dy in -1i32..=1 {
                    for dx in -1i32..=1 {
                        let nx = x as i32 + dx;
                        let ny = y as i32 + dy;
                        if nx >= 0 && nx < w as i32 && ny >= 0 && ny < h as i32 {
                            sum += alpha_buf[ny as usize * w + nx as usize] as u32;
                            count += 1;
                        }
                    }
                }
                tmp[y * w + x] = (sum / count) as u8;
            }
        }
        alpha_buf.copy_from_slice(&tmp);
    }

    let data = img.data.as_mut().expect("fog texture should be accessible");
    for i in 0..(w * h) {
        data[i * 4] = 0;
        data[i * 4 + 1] = 0;
        data[i * 4 + 2] = 0;
        data[i * 4 + 3] = alpha_buf[i];
    }
}
