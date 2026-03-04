use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use crate::camera::MainCamera;
use crate::map::{GridPosition, TILE_SIZE, MAP_WIDTH, MAP_HEIGHT};
use crate::units::components::*;
use crate::resources::components::{PlayerResources, Carrying};
use super::components::*;
use super::{spawn_building, load_building_texture};

#[derive(Resource, Default)]
pub struct PlacementMode {
    pub active: bool,
    pub kind: Option<BuildingKind>,
    pub ghost: Option<Entity>,
    pub menu_entity: Option<Entity>,
    pub wall_start: Option<GridPosition>,
    pub just_activated: bool,
}

#[derive(Component)]
pub struct GhostBuilding;

pub const BUILDABLE_KINDS: [(KeyCode, BuildingKind); 20] = [
    (KeyCode::Digit1, BuildingKind::House),
    (KeyCode::Digit2, BuildingKind::Barracks),
    (KeyCode::Digit3, BuildingKind::ArcheryRange),
    (KeyCode::Digit4, BuildingKind::Stable),
    (KeyCode::Digit5, BuildingKind::LumberCamp),
    (KeyCode::Digit6, BuildingKind::MiningCamp),
    (KeyCode::Digit7, BuildingKind::Farm),
    (KeyCode::Digit8, BuildingKind::WatchTower),
    (KeyCode::Digit9, BuildingKind::PalisadeWall),
    (KeyCode::Digit0, BuildingKind::StoneWall),
    (KeyCode::Minus, BuildingKind::Gate),
    (KeyCode::Equal, BuildingKind::SiegeWorkshop),
    (KeyCode::Backspace, BuildingKind::Blacksmith),
    (KeyCode::BracketLeft, BuildingKind::University),
    (KeyCode::BracketRight, BuildingKind::Market),
    (KeyCode::Backslash, BuildingKind::Monastery),
    (KeyCode::Semicolon, BuildingKind::Castle),
    (KeyCode::Quote, BuildingKind::Dock),
    (KeyCode::KeyT, BuildingKind::TownCenter),
    (KeyCode::KeyY, BuildingKind::Wonder),
];

pub fn enter_placement_mode(
    keys: Res<ButtonInput<KeyCode>>,
    mut placement: ResMut<PlacementMode>,
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    selected_villagers: Query<&Unit, With<Selected>>,
    age: Res<CurrentAge>,
) {
    if selected_villagers.is_empty() {
        if placement.active {
            cancel_placement(&mut placement, &mut commands);
        }
        return;
    }

    if keys.just_pressed(KeyCode::Escape) && placement.active {
        cancel_placement(&mut placement, &mut commands);
        return;
    }

    for &(key, kind) in &BUILDABLE_KINDS {
        if keys.just_pressed(key) {
            if kind.required_age() > age.0 {
                continue;
            }

            if let Some(old_ghost) = placement.ghost {
                commands.entity(old_ghost).despawn();
            }

            if let Some(menu) = placement.menu_entity {
                commands.entity(menu).despawn();
                placement.menu_entity = None;
            }

            placement.active = true;
            placement.kind = Some(kind);
            let (tw, th) = kind.tile_size();
            let iso_tile_h = TILE_SIZE / 2.0;
            let pixel_w = (tw as f32 * TILE_SIZE) as u32;
            let pixel_h = (th as f32 * iso_tile_h) as u32;

            let (texture, actual_dims) = load_building_texture(&mut images, kind, pixel_w, pixel_h);
            let display_size = if let Some((sw, sh)) = actual_dims {
                let aspect = sh as f32 / sw as f32;
                Vec2::new(pixel_w as f32, pixel_w as f32 * aspect)
            } else {
                Vec2::new(pixel_w as f32, pixel_h as f32)
            };

            let ghost = commands.spawn((
                GhostBuilding,
                Sprite {
                    image: texture,
                    custom_size: Some(display_size),
                    color: Color::srgba(1.0, 1.0, 1.0, 0.5),
                    ..default()
                },
                Transform::from_xyz(0.0, 0.0, 15.0),
            )).id();
            placement.ghost = Some(ghost);
        }
    }
}

pub fn update_ghost_position(
    placement: Res<PlacementMode>,
    mut ghost_q: Query<&mut Transform, With<GhostBuilding>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    if !placement.active || placement.ghost.is_none() {
        return;
    }

    let Ok(window) = windows.single() else { return };
    let Ok((camera, cam_transform)) = camera_q.single() else { return };
    let Some(cursor_pos) = window.cursor_position() else { return };
    let Ok(world_pos) = camera.viewport_to_world_2d(cam_transform, cursor_pos) else { return };

    let grid = GridPosition::from_world(world_pos);
    let snapped = grid.to_world();

    for mut transform in &mut ghost_q {
        transform.translation.x = snapped.x;
        transform.translation.y = snapped.y;
    }
}

pub fn place_building_system(
    mouse: Res<ButtonInput<MouseButton>>,
    mut placement: ResMut<PlacementMode>,
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut resources: ResMut<PlayerResources>,
    config: Res<crate::map::generation::MapConfig>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    existing_buildings: Query<(&Transform, &Building)>,
    age: Res<CurrentAge>,
    selected_villagers: Query<(Entity, &Transform), (With<Selected>, With<Carrying>, With<Unit>)>,
    ui_interactions: Query<&Interaction, With<Node>>,
) {
    if !placement.active || placement.kind.is_none() {
        return;
    }

    if placement.just_activated {
        placement.just_activated = false;
        return;
    }

    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let ui_busy = ui_interactions.iter().any(|i| *i != Interaction::None);
    if ui_busy { return; }

    let Ok(window) = windows.single() else { return };
    let Ok((camera, cam_transform)) = camera_q.single() else { return };
    let Some(cursor_pos) = window.cursor_position() else { return };
    let Ok(world_pos) = camera.viewport_to_world_2d(cam_transform, cursor_pos) else { return };

    let kind = placement.kind.unwrap();

    if kind.required_age() as u8 > age.0 as u8 {
        return;
    }

    let grid = GridPosition::from_world(world_pos);

    if grid.x < 0 || grid.y < 0
        || grid.x >= MAP_WIDTH as i32 || grid.y >= MAP_HEIGHT as i32
    {
        return;
    }

    if kind.is_wall() {
        if let Some(start) = placement.wall_start {
            let tiles = line_between(start.x, start.y, grid.x, grid.y);
            let cost_per = kind.build_cost();
            let mut placed = Vec::new();

            for (tx, ty) in &tiles {
                let g = GridPosition::new(*tx, *ty);
                if *tx < 0 || *ty < 0 || *tx >= MAP_WIDTH as i32 || *ty >= MAP_HEIGHT as i32 {
                    continue;
                }
                if crate::map::generation::building_footprint_has_water(g, 1, 1, &config.terrain_grid) {
                    continue;
                }
                if !resources.can_afford(cost_per.0, cost_per.1, cost_per.2, cost_per.3) {
                    break;
                }
                resources.spend(cost_per.0, cost_per.1, cost_per.2, cost_per.3);
                let e = spawn_building(&mut commands, &mut images, kind, Team(0), g, true);
                placed.push((e, g.to_world()));
            }

            if let Some((first_bld, first_pos)) = placed.first() {
                let mut closest: Option<(Entity, f32)> = None;
                for (ve, vtf) in &selected_villagers {
                    let d = vtf.translation.truncate().distance(*first_pos);
                    if closest.map_or(true, |(_, cd)| d < cd) {
                        closest = Some((ve, d));
                    }
                }
                if let Some((ve, _)) = closest {
                    commands.entity(ve).insert((
                        ConstructTarget(*first_bld),
                        MoveTarget(*first_pos),
                        UnitState::Constructing { building: *first_bld },
                    ));
                }
            }

            cancel_placement(&mut placement, &mut commands);
        } else {
            placement.wall_start = Some(grid);
        }
        return;
    }

    let (tw, th) = kind.tile_size();
    if crate::map::generation::building_footprint_has_water(
        grid,
        tw,
        th,
        &config.terrain_grid,
    ) {
        return;
    }

    let half_w = tw as f32 * TILE_SIZE / 2.0;
    let half_h = th as f32 * TILE_SIZE / 2.0;
    let snapped = grid.to_world();

    for (existing_tf, existing_bld) in &existing_buildings {
        let (ew, eh) = existing_bld.kind.tile_size();
        let e_half_w = ew as f32 * TILE_SIZE / 2.0;
        let e_half_h = eh as f32 * TILE_SIZE / 2.0;
        let epos = existing_tf.translation.truncate();

        let overlap_x = (snapped.x - epos.x).abs() < half_w + e_half_w;
        let overlap_y = (snapped.y - epos.y).abs() < half_h + e_half_h;
        if overlap_x && overlap_y {
            return;
        }
    }

    let (food, wood, gold, stone) = kind.build_cost();
    if !resources.spend(food, wood, gold, stone) {
        return;
    }

    let building_entity = spawn_building(&mut commands, &mut images, kind, Team(0), grid, true);

    let build_site = snapped;
    let mut closest: Option<(Entity, f32)> = None;
    for (villager_e, villager_tf) in &selected_villagers {
        let dist = villager_tf.translation.truncate().distance(build_site);
        if closest.map_or(true, |(_, d)| dist < d) {
            closest = Some((villager_e, dist));
        }
    }

    if let Some((villager_e, _)) = closest {
        commands.entity(villager_e).insert((
            ConstructTarget(building_entity),
            MoveTarget(build_site),
            UnitState::Constructing { building: building_entity },
        ));
    }

    cancel_placement(&mut placement, &mut commands);
}

fn line_between(x0: i32, y0: i32, x1: i32, y1: i32) -> Vec<(i32, i32)> {
    let mut points = Vec::new();
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    let mut cx = x0;
    let mut cy = y0;

    loop {
        points.push((cx, cy));
        if cx == x1 && cy == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            cx += sx;
        }
        if e2 <= dx {
            err += dx;
            cy += sy;
        }
    }
    points
}

fn cancel_placement(placement: &mut PlacementMode, commands: &mut Commands) {
    if let Some(ghost) = placement.ghost {
        commands.entity(ghost).despawn();
    }
    if let Some(menu) = placement.menu_entity {
        commands.entity(menu).despawn();
    }
    placement.active = false;
    placement.kind = None;
    placement.ghost = None;
    placement.menu_entity = None;
    placement.wall_start = None;
    placement.just_activated = false;
}

pub fn show_placement_ui(
    placement: Res<PlacementMode>,
    config: Res<crate::map::generation::MapConfig>,
    mut gizmos: Gizmos,
    ghost_q: Query<&Transform, With<GhostBuilding>>,
    existing_buildings: Query<(&Transform, &Building), Without<GhostBuilding>>,
) {
    if !placement.active {
        return;
    }

    let Some(kind) = placement.kind else { return };

    for ghost_tf in &ghost_q {
        let pos = ghost_tf.translation.truncate();
        let (tw, th) = kind.tile_size();
        let half_w = tw as f32 * TILE_SIZE / 2.0;
        let half_h = th as f32 * TILE_SIZE / 2.0;

        let grid = crate::map::GridPosition::from_world(pos);
        let on_water = crate::map::generation::building_footprint_has_water(
            grid,
            tw,
            th,
            &config.terrain_grid,
        );

        let mut blocked = on_water;
        for (existing_tf, existing_bld) in &existing_buildings {
            let (ew, eh) = existing_bld.kind.tile_size();
            let e_half_w = ew as f32 * TILE_SIZE / 2.0;
            let e_half_h = eh as f32 * TILE_SIZE / 2.0;
            let epos = existing_tf.translation.truncate();

            if (pos.x - epos.x).abs() < half_w + e_half_w
                && (pos.y - epos.y).abs() < half_h + e_half_h
            {
                blocked = true;
                break;
            }
        }

        let color = if blocked {
            Color::srgba(1.0, 0.2, 0.2, 0.6)
        } else {
            Color::srgba(0.2, 1.0, 0.2, 0.6)
        };

        gizmos.rect_2d(
            Isometry2d::from_translation(pos),
            Vec2::new(half_w * 2.0, half_h * 2.0),
            color,
        );
    }
}
