pub mod components;
pub mod gathering;

use bevy::prelude::*;
use components::*;
use gathering::*;
use crate::map::generation::MapConfig;

pub struct ResourcePlugin;

impl Plugin for ResourcePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PlayerResources {
                food: 200,
                wood: 200,
                gold: 100,
                stone: 50,
            })
            .add_systems(Startup, spawn_resource_nodes)
            .add_systems(Update, (
                gathering_system,
                returning_system,
                farm_system,
                resource_visual_system,
                resource_depletion_system,
                gather_move_recovery_system,
                floating_text_system,
                auto_reseek_system,
            ));
    }
}

const TREE_SPRITES: &[&str] = &[
    "textures/trees/spr_Tree_01_01_0.png",
    "textures/trees/spr_Tree_01_02_0.png",
    "textures/trees/spr_Tree_01_03_0.png",
    "textures/trees/spr_Tree_02_01_0.png",
    "textures/trees/spr_Tree_02_02_0.png",
];

fn spawn_resource_nodes(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    config: Res<MapConfig>,
) {
    let tree_handles: Vec<Handle<Image>> = TREE_SPRITES
        .iter()
        .map(|path| asset_server.load(*path))
        .collect();

    let gold_texture = create_resource_texture(&mut images, [255, 215, 0, 255]);
    let stone_texture = create_resource_texture(&mut images, [140, 140, 140, 255]);
    let berry_texture = create_resource_texture(&mut images, [180, 40, 40, 255]);

    let mut tree_idx = 0usize;

    for cluster in &config.resource_clusters {
        for &(x, y) in &cluster.positions {
            let grid = crate::map::GridPosition::new(x, y);
            let world = grid.to_world();

            let (image, size) = match cluster.kind {
                ResourceKind::Wood => {
                    let handle = tree_handles[tree_idx % tree_handles.len()].clone();
                    tree_idx = tree_idx.wrapping_add(
                        (x as usize).wrapping_mul(7) ^ (y as usize).wrapping_mul(13) | 1
                    );
                    (handle, Vec2::new(90.0, 103.0))
                }
                ResourceKind::Gold => (gold_texture.clone(), Vec2::new(64.0, 48.0)),
                ResourceKind::Stone => (stone_texture.clone(), Vec2::new(64.0, 48.0)),
                ResourceKind::Food => (berry_texture.clone(), Vec2::new(56.0, 42.0)),
            };

            commands.spawn((
                ResourceNode {
                    kind: cluster.kind,
                    remaining: cluster.amount,
                    max_amount: cluster.amount,
                },
                grid,
                Sprite {
                    image,
                    custom_size: Some(size),
                    ..default()
                },
                Transform::from_xyz(world.x, world.y, 5.0),
            ));
        }
    }
}

fn create_resource_texture(images: &mut Assets<Image>, color: [u8; 4]) -> Handle<Image> {
    let size = 28u32;
    let mut data = vec![0u8; (size * size * 4) as usize];

    for y in 0..size {
        for x in 0..size {
            let idx = ((y * size + x) * 4) as usize;
            let dx = (x as f32 - size as f32 / 2.0).abs();
            let dy = (y as f32 - size as f32 / 2.0).abs();
            if dx + dy < size as f32 / 2.0 {
                let variation = ((x.wrapping_mul(7) ^ y.wrapping_mul(11)) % 30) as i16 - 15;
                data[idx] = (color[0] as i16 + variation).clamp(0, 255) as u8;
                data[idx + 1] = (color[1] as i16 + variation).clamp(0, 255) as u8;
                data[idx + 2] = (color[2] as i16 + variation).clamp(0, 255) as u8;
                data[idx + 3] = 255;
            }
        }
    }

    let mut image = Image::new(
        bevy::render::render_resource::Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
        },
        bevy::render::render_resource::TextureDimension::D2,
        data,
        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
        bevy::asset::RenderAssetUsages::RENDER_WORLD,
    );
    image.sampler = bevy::image::ImageSampler::Descriptor(bevy::image::ImageSamplerDescriptor {
        mag_filter: bevy::image::ImageFilterMode::Nearest,
        min_filter: bevy::image::ImageFilterMode::Nearest,
        ..default()
    });

    images.add(image)
}
