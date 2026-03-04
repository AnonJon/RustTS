pub mod components;
pub mod gathering;
pub mod market;
pub mod animals;

use bevy::prelude::*;
use components::*;
use gathering::*;
use crate::map::generation::MapConfig;
use crate::map::terrain::TerrainType;
use crate::GameState;
use crate::map::generation::generate_map_config;

pub struct ResourcePlugin;

impl Plugin for ResourcePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<market::MarketPrices>()
            .init_resource::<Population>()
            .insert_resource(PlayerResources {
                food: 200,
                wood: 200,
                gold: 100,
                stone: 50,
            })
            .add_systems(OnEnter(GameState::InGame), (
                spawn_resource_nodes.after(generate_map_config),
                spawn_animals_system.after(generate_map_config),
                spawn_relics_system.after(generate_map_config),
                spawn_fish_system.after(generate_map_config),
            ))
            .add_systems(Update, (
                gathering_system,
                returning_system,
                farm_system,
                resource_visual_system,
                resource_depletion_system,
                gather_move_recovery_system,
                floating_text_system,
                auto_reseek_system,
                farm_auto_reseed_system,
                market::trade_cart_system,
                animals::animal_flee_system,
                animals::animal_movement_system,
                animals::animal_death_system,
                update_population_system,
            ).run_if(in_state(GameState::InGame)));
    }
}

const TREE_SPRITES: &[&str] = &[
    "textures/trees/spr_Tree_01_01_0.png",
    "textures/trees/spr_Tree_01_02_0.png",
    "textures/trees/spr_Tree_01_03_0.png",
    "textures/trees/spr_Tree_02_01_0.png",
    "textures/trees/spr_Tree_02_02_0.png",
];

fn spawn_fish_system(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    config: Res<MapConfig>,
) {
    use crate::map::terrain::TerrainType;

    let fish_texture = create_resource_texture(&mut images, [60, 120, 200, 255]);
    let map_w = config.terrain_grid.len();
    let map_h = if map_w > 0 { config.terrain_grid[0].len() } else { 0 };
    let mut count = 0u32;

    for gx in (2..map_w.saturating_sub(2)).step_by(6) {
        for gy in (2..map_h.saturating_sub(2)).step_by(6) {
            if config.terrain_grid[gx][gy].terrain != TerrainType::Water {
                continue;
            }
            let has_adjacent_water = [(0,1),(0,-1),(1,0),(-1,0)].iter().all(|&(dx,dy)| {
                let nx = gx as i32 + dx;
                let ny = gy as i32 + dy;
                nx >= 0 && ny >= 0 && (nx as usize) < map_w && (ny as usize) < map_h
                    && config.terrain_grid[nx as usize][ny as usize].terrain == TerrainType::Water
            });
            if !has_adjacent_water { continue; }

            let grid = crate::map::GridPosition::new(gx as i32, gy as i32);
            let world = grid.to_world();

            commands.spawn((
                ResourceNode {
                    kind: ResourceKind::Food,
                    remaining: 250,
                    max_amount: 250,
                },
                grid,
                Sprite {
                    image: fish_texture.clone(),
                    custom_size: Some(bevy::math::Vec2::new(40.0, 30.0)),
                    ..default()
                },
                Transform::from_xyz(world.x, world.y, 4.0),
            ));
            count += 1;
            if count >= 12 { return; }
        }
    }
}

fn spawn_relics_system(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    config: Res<MapConfig>,
) {
    use crate::units::components::Relic;
    use crate::map::terrain::TerrainType;

    let relic_texture = create_resource_texture(&mut images, [255, 215, 0, 255]);
    let map_w = config.terrain_grid.len() as i32;
    let map_h = if map_w > 0 { config.terrain_grid[0].len() as i32 } else { 0 };
    let cx = map_w / 2;
    let cy = map_h / 2;

    let relic_offsets = [
        (0, 0),
        (8, 6),
        (-8, 6),
        (6, -8),
        (-6, -8),
    ];

    for (dx, dy) in relic_offsets {
        let mut rx = (cx + dx).clamp(1, map_w - 2);
        let mut ry = (cy + dy).clamp(1, map_h - 2);

        for r in 0..5 {
            let mut found = false;
            for ddx in -r..=r {
                for ddy in -r..=r {
                    let nx = (rx + ddx).clamp(0, map_w - 1) as usize;
                    let ny = (ry + ddy).clamp(0, map_h - 1) as usize;
                    if config.terrain_grid[nx][ny].terrain != TerrainType::Water
                        && config.terrain_grid[nx][ny].terrain != TerrainType::DarkGrass
                    {
                        rx = nx as i32;
                        ry = ny as i32;
                        found = true;
                        break;
                    }
                }
                if found { break; }
            }
            if found { break; }
        }

        let grid = crate::map::GridPosition::new(rx, ry);
        let world = grid.to_world();

        commands.spawn((
            Relic,
            grid,
            Sprite {
                image: relic_texture.clone(),
                custom_size: Some(bevy::math::Vec2::new(32.0, 32.0)),
                ..default()
            },
            Transform::from_xyz(world.x, world.y, 8.0),
        ));
    }
}

fn spawn_animals_system(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    config: Res<MapConfig>,
) {
    animals::spawn_animals(&mut commands, &mut images, &config);
}

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

    // Track which tiles already have a resource from clusters
    let mut occupied: std::collections::HashSet<(i32, i32)> = std::collections::HashSet::new();

    for cluster in &config.resource_clusters {
        for &(x, y) in &cluster.positions {
            occupied.insert((x, y));
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

    // Every DarkGrass (forest) terrain tile gets a harvestable tree entity
    for (gx, col) in config.terrain_grid.iter().enumerate() {
        for (gy, tile) in col.iter().enumerate() {
            if tile.terrain != TerrainType::DarkGrass {
                continue;
            }
            let (x, y) = (gx as i32, gy as i32);
            if occupied.contains(&(x, y)) {
                continue;
            }
            let grid = crate::map::GridPosition::new(x, y);
            let world = grid.to_world();
            let handle = tree_handles[tree_idx % tree_handles.len()].clone();
            tree_idx = tree_idx.wrapping_add(
                (gx).wrapping_mul(7) ^ (gy).wrapping_mul(13) | 1
            );
            commands.spawn((
                ResourceNode {
                    kind: ResourceKind::Wood,
                    remaining: 150,
                    max_amount: 150,
                },
                grid,
                Sprite {
                    image: handle,
                    custom_size: Some(Vec2::new(90.0, 103.0)),
                    ..default()
                },
                Transform::from_xyz(world.x, world.y, 5.0),
            ));
        }
    }
}

fn update_population_system(
    mut pop: ResMut<Population>,
    units: Query<(&crate::units::components::Team, &crate::units::types::UnitKind), With<crate::units::components::Unit>>,
    buildings: Query<(&crate::units::components::Team, &crate::buildings::components::Building), Without<crate::buildings::components::UnderConstruction>>,
) {
    let mut current = 0u32;
    let mut cap = 0u32;

    for (team, kind) in &units {
        if team.0 != 0 { continue; }
        current += kind.population_cost();
    }

    for (team, building) in &buildings {
        if team.0 != 0 { continue; }
        cap += building.kind.population_support();
    }

    pop.current = current;
    pop.cap = cap.min(Population::MAX_POP);
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
