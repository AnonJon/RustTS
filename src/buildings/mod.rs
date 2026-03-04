pub mod components;
pub mod systems;
pub mod placement;
pub mod research;

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

/// Set of grid tiles occupied by buildings, rebuilt each frame.
/// Used by movement systems to prevent units walking through buildings.
#[derive(Resource, Default)]
pub struct BuildingOccupancy(pub std::collections::HashSet<(i32, i32)>);

fn update_building_occupancy(
    buildings: Query<(&GridPosition, &Building)>,
    mut occupancy: ResMut<BuildingOccupancy>,
) {
    occupancy.0.clear();
    for (grid, building) in &buildings {
        if building.kind == BuildingKind::Gate {
            continue;
        }
        let (tw, th) = building.kind.tile_size();
        for dx in 0..tw as i32 {
            for dy in 0..th as i32 {
                occupancy.0.insert((grid.x + dx, grid.y + dy));
            }
        }
    }
}

pub struct BuildingPlugin;

impl Plugin for BuildingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentAge>()
            .init_resource::<AgeUpProgress>()
            .init_resource::<PlacementMode>()
            .init_resource::<BuildingOccupancy>()
            .init_resource::<research::ResearchedTechnologies>()
            .init_resource::<research::UnitLineUpgrades>()
            .add_systems(OnEnter(GameState::InGame), spawn_starting_buildings.after(generate_map_config))
            .add_systems(Update, (
                update_building_occupancy,
                training_system,
                construction_system,
                age_up_system,
                building_death_system,
                tower_attack_system,
                garrison_command_system,
                ungarrison_system,
                garrison_eject_on_death_system,
                garrison_arrow_bonus_system,
                building_selection_system,
                keyboard_training_system,
            ).run_if(in_state(GameState::InGame)))
            .add_systems(Update, (
                rally_point_system,
                repair_command_system,
                repair_system,
                research::research_system,
                research::keyboard_research_system,
                research::apply_villager_tech_bonuses,
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
        false,
    );
}

pub fn spawn_building(
    commands: &mut Commands,
    images: &mut Assets<Image>,
    kind: BuildingKind,
    team: Team,
    grid: GridPosition,
    under_construction: bool,
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

    let initial_alpha = if under_construction { 0.3 } else { 1.0 };
    let initial_hp = if under_construction {
        kind.max_hp() * 0.1
    } else {
        kind.max_hp()
    };

    let (m_arm, p_arm) = kind.armor();
    let mut entity_cmds = commands.spawn((
        Building {
            kind,
            rally_point: Some(world + Vec2::new(0.0, -TILE_SIZE * 1.5)),
        },
        team,
        grid,
        Health {
            current: initial_hp,
            max: kind.max_hp(),
        },
        Armor::new(m_arm, p_arm),
        Sprite {
            image: texture,
            custom_size: sprite_size,
            color: Color::srgba(1.0, 1.0, 1.0, initial_alpha),
            ..default()
        },
        Transform::from_xyz(world.x, world.y, 5.0),
    ));

    if under_construction {
        entity_cmds.insert(UnderConstruction::new(kind));
    }

    if kind == BuildingKind::WatchTower && !under_construction {
        entity_cmds.insert(TowerAttack::watch_tower());
    }

    if kind == BuildingKind::TownCenter && !under_construction {
        entity_cmds.insert(TowerAttack::town_center());
    }

    if kind == BuildingKind::Castle && !under_construction {
        entity_cmds.insert(TowerAttack {
            range: 10.0,
            base_pierce_damage: 11.0,
            pierce_damage: 11.0,
            cooldown: Timer::from_seconds(1.5, TimerMode::Repeating),
        });
    }

    if kind == BuildingKind::Gate {
        entity_cmds.insert(GatePassable { owner_team: team.0 });
    }

    if kind == BuildingKind::Farm && !under_construction {
        entity_cmds.insert(crate::resources::components::FarmFood::new());
    }

    let garrison_capacity = match kind {
        BuildingKind::TownCenter => Some(15),
        BuildingKind::WatchTower => Some(5),
        BuildingKind::Castle => Some(20),
        _ => None,
    };
    if let Some(cap) = garrison_capacity {
        if !under_construction {
            entity_cmds.insert(GarrisonSlots::new(cap));
        }
    }

    if !under_construction && !kind.can_train().is_empty() {
        entity_cmds.insert(TrainingQueue {
            queue: Vec::new(),
        });
    }

    if matches!(kind, BuildingKind::Mill | BuildingKind::TownCenter) && !under_construction {
        entity_cmds.insert(crate::resources::components::AutoReseed(true));
    }

    let is_research_bld = matches!(kind,
        BuildingKind::TownCenter | BuildingKind::Blacksmith | BuildingKind::University
        | BuildingKind::LumberCamp | BuildingKind::MiningCamp | BuildingKind::Mill
        | BuildingKind::Barracks | BuildingKind::ArcheryRange | BuildingKind::Stable
        | BuildingKind::Castle | BuildingKind::Monastery | BuildingKind::Market
    );
    if is_research_bld && !under_construction {
        entity_cmds.insert(research::ResearchQueue { queue: Vec::new() });
    }

    if kind == BuildingKind::Monastery && !under_construction {
        entity_cmds.insert(components::RelicStorage::new());
    }

    if !under_construction {
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
    }

    entity_cmds.id()
}

pub fn sprite_path(kind: BuildingKind) -> Option<&'static str> {
    match kind {
        BuildingKind::TownCenter => Some("assets/sprites/buildings/castlekeep_14.png"),
        BuildingKind::LumberCamp => Some("assets/textures/lumber_camp.png"),
        BuildingKind::MiningCamp => Some("assets/textures/mining_camp.png"),
        _ => None,
    }
}

pub fn load_building_texture(
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
