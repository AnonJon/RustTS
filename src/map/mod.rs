pub mod terrain;
pub mod coords;
pub mod generation;

use bevy::prelude::*;
use bevy::image::{ImageSampler, ImageFilterMode, ImageSamplerDescriptor};
use bevy::asset::RenderAssetUsages;
use bevy_ecs_tilemap::prelude::*;
use terrain::TerrainType;
use generation::{generate_map_config, MapConfig};

pub struct MapPlugin;

pub const TILE_SIZE: f32 = 128.0;
pub const MAP_WIDTH: u32 = 48;
pub const MAP_HEIGHT: u32 = 48;

const SUB_TILE_W: u32 = 128;
const SUB_TILE_H: u32 = 64;
const VARIANTS_PER_TERRAIN: u32 = 16;
const TERRAIN_COUNT: u32 = 4;

const TERRAIN_FILES: [&str; 4] = [
    "assets/textures/g_grs_m00-m19_r01_00_color.png",
    "assets/textures/g_for_00_color.png",
    "assets/textures/g_rd1_00_color.png",
    "assets/textures/g_wtr_00_color.png",
];

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TilemapPlugin)
            .add_systems(PreStartup, generate_map_config)
            .add_systems(Startup, setup_tilemap)
            .add_systems(PostUpdate, depth_sort_system);
    }
}

fn depth_sort_system(
    mut query: Query<
        &mut Transform,
        (With<Sprite>, Without<bevy_ecs_tilemap::tiles::TilePos>, Without<Node>),
    >,
) {
    let max_y = (MAP_WIDTH + MAP_HEIGHT) as f32 * TILE_SIZE / 4.0;
    for mut transform in &mut query {
        let y = transform.translation.y;
        transform.translation.z = 10.0 + (max_y - y) / max_y;
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

    pub fn to_world(self) -> Vec2 {
        let iso_x = (self.x - self.y) as f32 * (TILE_SIZE / 2.0);
        let iso_y = (self.x + self.y) as f32 * (TILE_SIZE / 4.0);
        Vec2::new(iso_x + (MAP_WIDTH as f32 * TILE_SIZE / 2.0), iso_y)
    }

    pub fn from_world(world: Vec2) -> Self {
        let ax = world.x - (MAP_WIDTH as f32 * TILE_SIZE / 2.0);
        let half = TILE_SIZE / 2.0;
        let quarter = TILE_SIZE / 4.0;
        Self {
            x: ((ax / half + world.y / quarter) / 2.0).floor() as i32,
            y: ((world.y / quarter - ax / half) / 2.0).floor() as i32,
        }
    }

    pub fn distance_to(self, other: GridPosition) -> f32 {
        let dx = (self.x - other.x) as f32;
        let dy = (self.y - other.y) as f32;
        (dx * dx + dy * dy).sqrt()
    }
}

fn is_inside_diamond(px: u32, py: u32, w: u32, h: u32) -> bool {
    let cx = w as f32 / 2.0;
    let cy = h as f32 / 2.0;
    let dx = (px as f32 - cx).abs() / cx;
    let dy = (py as f32 - cy).abs() / cy;
    dx + dy <= 1.0
}

fn create_terrain_atlas(images: &mut Assets<Image>) -> Handle<Image> {
    let atlas_w = VARIANTS_PER_TERRAIN * SUB_TILE_W;
    let atlas_h = TERRAIN_COUNT * SUB_TILE_H;
    let mut atlas_data = vec![0u8; (atlas_w * atlas_h * 4) as usize];

    let src_tile_px = 128u32;
    let crop_y_start = (src_tile_px - SUB_TILE_H) / 2;

    for (terrain_row, file_path) in TERRAIN_FILES.iter().enumerate() {
        let src = match image::open(file_path) {
            Ok(img) => img.to_rgba8(),
            Err(e) => {
                warn!("Failed to load terrain texture {file_path}: {e}, using fallback");
                let fallback_colors: [[u8; 3]; 4] = [
                    [34, 139, 34],
                    [0, 100, 0],
                    [139, 119, 101],
                    [30, 80, 160],
                ];
                let c = fallback_colors[terrain_row];
                let mut fb = image::RgbaImage::new(512, 512);
                for pixel in fb.pixels_mut() {
                    *pixel = image::Rgba([c[0], c[1], c[2], 255]);
                }
                fb
            }
        };

        for sy in 0..4u32 {
            for sx in 0..4u32 {
                let variant_idx = sy * 4 + sx;
                let dst_x0 = variant_idx * SUB_TILE_W;
                let dst_y0 = terrain_row as u32 * SUB_TILE_H;

                for py in 0..SUB_TILE_H {
                    for px in 0..SUB_TILE_W {
                        if !is_inside_diamond(px, py, SUB_TILE_W, SUB_TILE_H) {
                            continue;
                        }
                        let src_x = sx * src_tile_px + px;
                        let src_y = sy * src_tile_px + crop_y_start + py;
                        let src_pixel = src.get_pixel(src_x, src_y);
                        let dst_idx = (((dst_y0 + py) * atlas_w + dst_x0 + px) * 4) as usize;
                        atlas_data[dst_idx] = src_pixel[0];
                        atlas_data[dst_idx + 1] = src_pixel[1];
                        atlas_data[dst_idx + 2] = src_pixel[2];
                        atlas_data[dst_idx + 3] = src_pixel[3];
                    }
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
            let terrain = TerrainType::for_position(x, y, config.seed, &config.terrain_grid);
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

    let center = GridPosition::new(
        MAP_WIDTH as i32 / 2,
        MAP_HEIGHT as i32 / 2,
    ).to_world();

    commands.entity(tilemap_entity).insert(TilemapBundle {
        grid_size,
        map_type,
        size: map_size,
        storage: tile_storage,
        texture: TilemapTexture::Single(texture_handle),
        tile_size,
        transform: Transform::from_xyz(center.x, center.y, 0.0),
        anchor: TilemapAnchor::Center,
        ..default()
    });
}
