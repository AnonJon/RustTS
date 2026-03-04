pub mod terrain;
pub mod coords;
pub mod generation;
pub mod fog;

use bevy::prelude::*;
use bevy::image::{ImageSampler, ImageFilterMode, ImageSamplerDescriptor};
use bevy::asset::RenderAssetUsages;
use bevy_ecs_tilemap::prelude::*;
use terrain::TerrainType;
use generation::{generate_map_config, MapConfig};
use crate::GameState;

pub struct MapPlugin;

pub const TILE_SIZE: f32 = 128.0;
pub const MAP_WIDTH: u32 = 48;
pub const MAP_HEIGHT: u32 = 48;

const SUB_TILE_W: u32 = 128;
const SUB_TILE_H: u32 = 64;
const VARIANTS_PER_TERRAIN: u32 = 16;
const TERRAIN_COUNT: u32 = 4;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TilemapPlugin)
            .init_resource::<fog::FogOfWar>()
            .add_systems(
                OnEnter(GameState::InGame),
                (generate_map_config, setup_tilemap, fog::setup_fog_overlay).chain(),
            )
            .add_systems(PostUpdate, depth_sort_system.run_if(in_state(GameState::InGame)))
            .add_systems(Update, (
                fog::update_fog_system,
                fog::apply_fog_visibility_system.after(fog::update_fog_system),
                fog::update_fog_texture.after(fog::update_fog_system),
            ).run_if(in_state(GameState::InGame)));
    }
}

fn depth_sort_system(
    mut query: Query<
        &mut Transform,
        (With<Sprite>, Without<bevy_ecs_tilemap::tiles::TilePos>, Without<Node>, Without<fog::FogOverlayMarker>),
    >,
) {
    // In the diamond iso projection world_x = 64*(x+y), so higher world_x
    // means larger (x+y) which is visually "in front" (south).  Sort so that
    // larger world_x → higher z → rendered on top.
    let max_x = (MAP_WIDTH + MAP_HEIGHT) as f32 * (TILE_SIZE / 2.0);
    for mut transform in &mut query {
        let x = transform.translation.x;
        transform.translation.z = 10.0 + x / max_x;
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GridPosition {
    pub x: i32,
    pub y: i32,
}

impl GridPosition {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// Convert grid position to world position.
    ///
    /// CRITICAL: This MUST match bevy_ecs_tilemap's Diamond isometric projection
    /// (`DIAMOND_BASIS` in `helpers/square_grid/diamond.rs`):
    ///
    ///   world_x = grid_size_x * 0.5 * (x + y)   = 64 * (x + y)
    ///   world_y = grid_size_y * 0.5 * (y - x)    = 32 * (y - x)
    ///
    /// If this formula diverges from bevy_ecs_tilemap, entities will not align
    /// with their tiles. The tilemap must use `TilemapAnchor::None` (default)
    /// with `Transform::default()` so tile (0,0) sits at the world origin.
    pub fn to_world(self) -> Vec2 {
        let half_gx = TILE_SIZE / 2.0;   // 64
        let half_gy = TILE_SIZE / 4.0;   // 32
        Vec2::new(
            half_gx * (self.x as f32 + self.y as f32),
            half_gy * (self.y as f32 - self.x as f32),
        )
    }

    /// Inverse of `to_world`. Uses bevy_ecs_tilemap's `INV_DIAMOND_BASIS`.
    pub fn from_world(world: Vec2) -> Self {
        let gx = TILE_SIZE;              // 128 (grid_size.x)
        let gy = TILE_SIZE / 2.0;        // 64  (grid_size.y)
        let nx = world.x / gx;
        let ny = world.y / gy;
        Self {
            x: (nx - ny + 0.5).floor() as i32,
            y: (nx + ny + 0.5).floor() as i32,
        }
    }

    pub fn distance_to(self, other: GridPosition) -> f32 {
        let dx = (self.x - other.x) as f32;
        let dy = (self.y - other.y) as f32;
        (dx * dx + dy * dy).sqrt()
    }
}

fn ground_tile_paths(season: &str, types: &[&str], rows: &[u32]) -> Vec<String> {
    let mut paths = Vec::new();
    for t in types {
        for &r in rows {
            paths.push(format!("assets/textures/ground/{season}_{t}_r{r}.png"));
        }
    }
    paths
}

fn make_solid_tile(color: [u8; 3]) -> image::RgbaImage {
    let mut tile = image::RgbaImage::new(SUB_TILE_W, SUB_TILE_H);
    let cx = SUB_TILE_W as f32 / 2.0;
    let cy = SUB_TILE_H as f32 / 2.0;
    for y in 0..SUB_TILE_H {
        for x in 0..SUB_TILE_W {
            let dx = (x as f32 - cx).abs() / cx;
            let dy = (y as f32 - cy).abs() / cy;
            if dx + dy <= 1.0 {
                tile.put_pixel(x, y, image::Rgba([color[0], color[1], color[2], 255]));
            }
        }
    }
    tile
}

fn create_terrain_atlas(images: &mut Assets<Image>) -> Handle<Image> {
    let atlas_w = VARIANTS_PER_TERRAIN * SUB_TILE_W;
    let atlas_h = TERRAIN_COUNT * SUB_TILE_H;
    let mut atlas_data = vec![0u8; (atlas_w * atlas_h * 4) as usize];

    let fallback_colors: [[u8; 3]; 4] = [
        [34, 139, 34],   // grass
        [0, 100, 0],     // forest
        [139, 119, 101], // dirt
        [30, 80, 160],   // water
    ];

    // Grass: r0 + r3 only (7-19% brown); r1/r2 are transition tiles (40-60% brown)
    // Forest: autumn r0 + r3 for consistent dark floor
    // Dirt: actual dirt tiles + the brown transition rows (r1/r2) from grass
    // Water: solid blue fallback
    let tile_sets: [Vec<String>; 4] = [
        ground_tile_paths("summer", &["grass_a", "grass_b", "grass_c"], &[0, 3]),
        ground_tile_paths("autumn", &["grass_a", "grass_b", "grass_c"], &[0, 3]),
        {
            let mut d = ground_tile_paths("summer", &["dirt_a", "dirt_b"], &[0, 1, 2, 3]);
            d.extend(ground_tile_paths("summer", &["grass_a", "grass_b"], &[1, 2]));
            d
        },
        vec![],
    ];

    for terrain_row in 0..TERRAIN_COUNT as usize {
        let paths = &tile_sets[terrain_row];
        let dst_y0 = terrain_row as u32 * SUB_TILE_H;

        for variant_idx in 0..VARIANTS_PER_TERRAIN {
            let dst_x0 = variant_idx * SUB_TILE_W;

            let tile = if paths.is_empty() {
                make_solid_tile(fallback_colors[terrain_row])
            } else {
                let path = &paths[variant_idx as usize % paths.len()];
                match image::open(path) {
                    Ok(img) => img.to_rgba8(),
                    Err(e) => {
                        warn!("Failed to load tile {path}: {e}");
                        make_solid_tile(fallback_colors[terrain_row])
                    }
                }
            };

            for py in 0..SUB_TILE_H.min(tile.height()) {
                for px in 0..SUB_TILE_W.min(tile.width()) {
                    let src_pixel = tile.get_pixel(px, py);
                    if src_pixel[3] == 0 {
                        continue;
                    }
                    let dst_idx = (((dst_y0 + py) * atlas_w + dst_x0 + px) * 4) as usize;
                    atlas_data[dst_idx] = src_pixel[0];
                    atlas_data[dst_idx + 1] = src_pixel[1];
                    atlas_data[dst_idx + 2] = src_pixel[2];
                    atlas_data[dst_idx + 3] = src_pixel[3];
                }
            }
        }
    }

    let mut img = Image::new(
        bevy::render::render_resource::Extent3d {
            width: atlas_w,
            height: atlas_h,
            depth_or_array_layers: 1,
        },
        bevy::render::render_resource::TextureDimension::D2,
        atlas_data,
        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
    img.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        mag_filter: ImageFilterMode::Linear,
        min_filter: ImageFilterMode::Linear,
        ..default()
    });

    images.add(img)
}

fn setup_tilemap(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    config: Res<MapConfig>,
) {
    let texture_handle = create_terrain_atlas(&mut images);

    let map_size = TilemapSize {
        x: MAP_WIDTH,
        y: MAP_HEIGHT,
    };
    let tile_size = TilemapTileSize {
        x: SUB_TILE_W as f32,
        y: SUB_TILE_H as f32,
    };
    let grid_size: TilemapGridSize = tile_size.into();

    let mut tile_storage = TileStorage::empty(map_size);
    let tilemap_entity = commands.spawn_empty().id();

    for x in 0..map_size.x {
        for y in 0..map_size.y {
            let tile_pos = TilePos { x, y };
            let terrain = TerrainType::for_position(x, y, &config.terrain_grid);
            let tile_entity = commands
                .spawn(TileBundle {
                    position: tile_pos,
                    tilemap_id: TilemapId(tilemap_entity),
                    texture_index: TileTextureIndex(terrain.atlas_index(x, y)),
                    ..default()
                })
                .id();
            tile_storage.set(&tile_pos, tile_entity);
        }
    }

    let map_type = TilemapType::Isometric(IsoCoordSystem::Diamond);

    // TilemapAnchor::None (the default) places tile (0,0) at the tilemap
    // transform origin, matching GridPosition::to_world() exactly.
    commands.entity(tilemap_entity).insert(TilemapBundle {
        grid_size,
        map_type,
        size: map_size,
        storage: tile_storage,
        texture: TilemapTexture::Single(texture_handle),
        tile_size,
        transform: Transform::default(),
        ..default()
    });
}
