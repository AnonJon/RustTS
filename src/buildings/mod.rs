pub mod components;
pub mod systems;
pub mod placement;

use bevy::prelude::*;
use components::*;
use systems::*;
use placement::*;
use crate::units::components::*;
use crate::map::{GridPosition, TILE_SIZE};
use crate::map::generation::MapConfig;
use crate::resources::components::DropOff;
use crate::GameState;
use crate::map::generation::generate_map_config;

pub struct BuildingPlugin;

impl Plugin for BuildingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentAge>()
            .init_resource::<AgeUpProgress>()
            .init_resource::<PlacementMode>()
            .add_systems(OnEnter(GameState::InGame), spawn_starting_buildings.after(generate_map_config))
            .add_systems(Update, (
                training_system,
                age_up_system,
                building_death_system,
                building_selection_system,
                keyboard_training_system,
                rally_point_system,
                enter_placement_mode,
                update_ghost_position,
                place_building_system,
                show_placement_ui,
            ).run_if(in_state(GameState::InGame)));
    }
}

fn spawn_starting_buildings(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    config: Res<MapConfig>,
) {
    let base = config.player_base();
    let (tw, th) = BuildingKind::TownCenter.tile_size();
    let pos = crate::map::generation::nudge_building_onto_land(base, tw, th, &config.terrain_grid);
    spawn_building(
        &mut commands,
        &mut images,
        BuildingKind::TownCenter,
        Team(0),
        pos,
    );
}

pub fn spawn_building(
    commands: &mut Commands,
    images: &mut Assets<Image>,
    kind: BuildingKind,
    team: Team,
    grid: GridPosition,
) -> Entity {
    let (tw, th) = kind.tile_size();
    let iso_tile_h = TILE_SIZE / 2.0;
    let pixel_w = (tw as f32 * TILE_SIZE) as u32;
    let pixel_h = (th as f32 * iso_tile_h) as u32;
    let texture = load_building_texture(images, kind, pixel_w, pixel_h);
    let world = grid.to_world();

    let sprite_size = if sprite_path(kind).is_some() {
        Some(kind.sprite_display_size(pixel_w as f32, pixel_h as f32))
    } else {
        None
    };

    let mut entity_cmds = commands.spawn((
        Building {
            kind,
            rally_point: Some(world + Vec2::new(0.0, -TILE_SIZE * 1.5)),
        },
        team,
        grid,
        Health::new(kind.max_hp()),
        Sprite {
            image: texture,
            custom_size: sprite_size,
            ..default()
        },
        Transform::from_xyz(world.x, world.y, 5.0),
    ));

    if !kind.can_train().is_empty() {
        entity_cmds.insert(TrainingQueue {
            queue: Vec::new(),
        });
    }

    let drop_off = match kind {
        BuildingKind::TownCenter => Some(DropOff::all()),
        BuildingKind::LumberCamp => Some(DropOff::wood()),
        BuildingKind::MiningCamp => Some(DropOff::mining()),
        BuildingKind::Mill | BuildingKind::Farm => Some(DropOff::food()),
        _ => None,
    };
    if let Some(d) = drop_off {
        entity_cmds.insert(d);
    }

    entity_cmds.id()
}

fn sprite_path(kind: BuildingKind) -> Option<&'static str> {
    match kind {
        BuildingKind::TownCenter => Some("assets/sprites/buildings/castlekeep_14.png"),
        BuildingKind::LumberCamp => Some("assets/textures/lumber_camp.png"),
        BuildingKind::MiningCamp => Some("assets/textures/mining_camp.png"),
        _ => None,
    }
}

fn load_building_texture(
    images: &mut Assets<Image>,
    kind: BuildingKind,
    width: u32,
    height: u32,
) -> Handle<Image> {
    if let Some(path) = sprite_path(kind) {
        if let Ok(src) = image::open(path) {
            let rgba = src.to_rgba8();
            let (sw, sh) = rgba.dimensions();
            let mut img = Image::new(
                bevy::render::render_resource::Extent3d {
                    width: sw,
                    height: sh,
                    depth_or_array_layers: 1,
                },
                bevy::render::render_resource::TextureDimension::D2,
                rgba.into_raw(),
                bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
                bevy::asset::RenderAssetUsages::RENDER_WORLD,
            );
            img.sampler = bevy::image::ImageSampler::Descriptor(
                bevy::image::ImageSamplerDescriptor {
                    mag_filter: bevy::image::ImageFilterMode::Linear,
                    min_filter: bevy::image::ImageFilterMode::Linear,
                    ..default()
                },
            );
            return images.add(img);
        }
    }
    create_building_texture(images, kind.color(), width, height)
}

fn create_building_texture(
    images: &mut Assets<Image>,
    color: [u8; 4],
    width: u32,
    height: u32,
) -> Handle<Image> {
    let mut data = vec![0u8; (width * height * 4) as usize];

    for y in 0..height {
        for x in 0..width {
            let idx = ((y * width + x) * 4) as usize;
            let is_border = x == 0 || x == width - 1 || y == 0 || y == height - 1
                || x == 1 || x == width - 2 || y == 1 || y == height - 2;

            if is_border {
                data[idx] = (color[0] as f32 * 0.5) as u8;
                data[idx + 1] = (color[1] as f32 * 0.5) as u8;
                data[idx + 2] = (color[2] as f32 * 0.5) as u8;
                data[idx + 3] = 255;
            } else {
                data[idx] = color[0];
                data[idx + 1] = color[1];
                data[idx + 2] = color[2];
                data[idx + 3] = color[3];
            }
        }
    }

    let mut image = Image::new(
        bevy::render::render_resource::Extent3d {
            width,
            height,
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
