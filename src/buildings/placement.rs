use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use crate::camera::MainCamera;
use crate::map::{GridPosition, TILE_SIZE, MAP_WIDTH, MAP_HEIGHT};
use crate::units::components::*;
use crate::resources::components::{PlayerResources, Carrying};
use super::components::*;
use super::{spawn_building, load_building_texture, sprite_path};

#[derive(Resource, Default)]
pub struct PlacementMode {
    pub active: bool,
    pub kind: Option<BuildingKind>,
    pub ghost: Option<Entity>,
    pub menu_entity: Option<Entity>,
    pub wall_start: Option<GridPosition>,
}

#[derive(Component)]
pub struct GhostBuilding;

#[derive(Component)]
pub struct BuildMenu;

const BUILDABLE_KINDS: [(KeyCode, BuildingKind); 17] = [
    (KeyCode::Digit1, BuildingKind::Barracks),
    (KeyCode::Digit2, BuildingKind::ArcheryRange),
    (KeyCode::Digit3, BuildingKind::Stable),
    (KeyCode::Digit4, BuildingKind::LumberCamp),
    (KeyCode::Digit5, BuildingKind::MiningCamp),
    (KeyCode::Digit6, BuildingKind::Farm),
    (KeyCode::Digit7, BuildingKind::WatchTower),
    (KeyCode::Digit8, BuildingKind::PalisadeWall),
    (KeyCode::Digit9, BuildingKind::StoneWall),
    (KeyCode::Digit0, BuildingKind::Gate),
    (KeyCode::Minus, BuildingKind::SiegeWorkshop),
    (KeyCode::Equal, BuildingKind::Blacksmith),
    (KeyCode::Backspace, BuildingKind::University),
    (KeyCode::BracketLeft, BuildingKind::Market),
    (KeyCode::BracketRight, BuildingKind::Monastery),
    (KeyCode::Backslash, BuildingKind::Castle),
    (KeyCode::Semicolon, BuildingKind::Dock),
];

pub fn enter_placement_mode(
    keys: Res<ButtonInput<KeyCode>>,
    mut placement: ResMut<PlacementMode>,
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    selected_villagers: Query<&Unit, With<Selected>>,
    age: Res<CurrentAge>,
    resources: Res<PlayerResources>,
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

    if keys.just_pressed(KeyCode::KeyB) {
        if placement.active {
            cancel_placement(&mut placement, &mut commands);
        } else {
            placement.active = true;
            spawn_build_menu(&mut commands, &mut placement, &age, &resources);
        }
        return;
    }

    if !placement.active {
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

            placement.kind = Some(kind);
            let (tw, th) = kind.tile_size();
            let iso_tile_h = TILE_SIZE / 2.0;
            let pixel_w = (tw as f32 * TILE_SIZE) as u32;
            let pixel_h = (th as f32 * iso_tile_h) as u32;

            let texture = load_building_texture(&mut images, kind, pixel_w, pixel_h);
            let display_size = if sprite_path(kind).is_some() {
                kind.sprite_display_size(pixel_w as f32, pixel_h as f32)
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

fn spawn_build_menu(
    commands: &mut Commands,
    placement: &mut PlacementMode,
    age: &CurrentAge,
    resources: &PlayerResources,
) {
    let menu = commands.spawn((
        BuildMenu,
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(110.0),
            left: Val::Px(10.0),
            padding: UiRect::all(Val::Px(12.0)),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(4.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.08, 0.08, 0.12, 0.92)),
    )).with_children(|parent| {
        parent.spawn((
            Text::new("-- BUILD (Esc to cancel) --"),
            TextFont { font_size: 15.0, ..default() },
            TextColor(Color::srgb(1.0, 0.85, 0.3)),
        ));

        let labels = [
            ("1", "Barracks"),
            ("2", "Archery Range"),
            ("3", "Stable"),
            ("4", "Lumber Camp"),
            ("5", "Mining Camp"),
            ("6", "Farm"),
            ("7", "Watch Tower"),
            ("8", "Palisade Wall"),
            ("9", "Stone Wall"),
            ("0", "Gate"),
            ("-", "Siege Workshop"),
            ("=", "Blacksmith"),
            ("Bksp", "University"),
            ("[", "Market"),
            ("]", "Monastery"),
            ("\\", "Castle"),
            (";", "Dock"),
        ];

        for (i, &(key_label, name)) in labels.iter().enumerate() {
            let kind = BUILDABLE_KINDS[i].1;
            let (f, w, g, s) = kind.build_cost();
            let locked = kind.required_age() > age.0;
            let too_expensive = !resources.can_afford(f, w, g, s);

            let mut cost_parts: Vec<String> = Vec::new();
            if f > 0 { cost_parts.push(format!("{f}F")); }
            if w > 0 { cost_parts.push(format!("{w}W")); }
            if g > 0 { cost_parts.push(format!("{g}G")); }
            if s > 0 { cost_parts.push(format!("{s}S")); }
            let cost_str = cost_parts.join(" ");

            let label = if locked {
                format!("[{key_label}] {name} -- requires {:?} Age", kind.required_age())
            } else {
                format!("[{key_label}] {name}  ({cost_str})")
            };

            let color = if locked {
                Color::srgb(0.4, 0.4, 0.4)
            } else if too_expensive {
                Color::srgb(0.9, 0.3, 0.3)
            } else {
                Color::srgb(0.85, 0.85, 0.85)
            };

            parent.spawn((
                Text::new(label),
                TextFont { font_size: 14.0, ..default() },
                TextColor(color),
            ));
        }
    }).id();

    placement.menu_entity = Some(menu);
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
) {
    if !placement.active || placement.kind.is_none() {
        return;
    }

    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

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
