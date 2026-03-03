use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use crate::camera::MainCamera;
use crate::map::{GridPosition, TILE_SIZE, MAP_WIDTH, MAP_HEIGHT};
use crate::units::components::*;
use crate::resources::components::PlayerResources;
use super::components::*;
use super::spawn_building;

#[derive(Resource, Default)]
pub struct PlacementMode {
    pub active: bool,
    pub kind: Option<BuildingKind>,
    pub ghost: Option<Entity>,
    pub menu_entity: Option<Entity>,
}

#[derive(Component)]
pub struct GhostBuilding;

#[derive(Component)]
pub struct BuildMenu;

const BUILDABLE_KINDS: [(KeyCode, BuildingKind); 6] = [
    (KeyCode::Digit1, BuildingKind::Barracks),
    (KeyCode::Digit2, BuildingKind::ArcheryRange),
    (KeyCode::Digit3, BuildingKind::Stable),
    (KeyCode::Digit4, BuildingKind::LumberCamp),
    (KeyCode::Digit5, BuildingKind::MiningCamp),
    (KeyCode::Digit6, BuildingKind::Farm),
];

pub fn enter_placement_mode(
    keys: Res<ButtonInput<KeyCode>>,
    mut placement: ResMut<PlacementMode>,
    mut commands: Commands,
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
            let pixel_w = tw as f32 * TILE_SIZE;
            let pixel_h = th as f32 * TILE_SIZE;

            let ghost = commands.spawn((
                GhostBuilding,
                Sprite {
                    color: Color::srgba(0.5, 1.0, 0.5, 0.4),
                    custom_size: Some(Vec2::new(pixel_w, pixel_h)),
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
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    existing_buildings: Query<(&Transform, &Building)>,
    age: Res<CurrentAge>,
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

    let (tw, th) = kind.tile_size();
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

    spawn_building(&mut commands, &mut images, kind, Team(0), grid);

    cancel_placement(&mut placement, &mut commands);
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
}

pub fn show_placement_ui(
    placement: Res<PlacementMode>,
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

        let mut blocked = false;
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
