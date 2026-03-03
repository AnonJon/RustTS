use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use super::components::*;
use crate::camera::MainCamera;

#[derive(Resource, Default)]
pub(crate) struct DragState {
    start: Option<Vec2>,
    current: Option<Vec2>,
    dragging: bool,
}

#[derive(Resource, Default)]
pub(crate) struct SelectionBoxVisual(Option<Entity>);

const DRAG_THRESHOLD: f32 = 5.0;

pub fn handle_selection_click(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    units: Query<(Entity, &Transform, &Team), With<Unit>>,
    selected: Query<Entity, With<Selected>>,
    mut drag_state: Local<DragState>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    let Ok(window) = windows.single() else { return };
    let Ok((camera, cam_transform)) = camera_q.single() else { return };
    let Some(cursor_pos) = window.cursor_position() else { return };

    if mouse.just_pressed(MouseButton::Left) {
        if let Ok(world_pos) = camera.viewport_to_world_2d(cam_transform, cursor_pos) {
            drag_state.start = Some(world_pos);
            drag_state.current = Some(world_pos);
            drag_state.dragging = false;
        }
    }

    if mouse.pressed(MouseButton::Left) {
        if let Ok(world_pos) = camera.viewport_to_world_2d(cam_transform, cursor_pos) {
            drag_state.current = Some(world_pos);
            if let Some(start) = drag_state.start {
                if start.distance(world_pos) > DRAG_THRESHOLD {
                    drag_state.dragging = true;
                }
            }
        }
    }

    if mouse.just_released(MouseButton::Left) {
        if let Ok(world_pos) = camera.viewport_to_world_2d(cam_transform, cursor_pos) {
            let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);

            if !drag_state.dragging {
                if !shift {
                    for entity in &selected {
                        commands.entity(entity).remove::<Selected>();
                    }
                }

                let mut closest: Option<(Entity, f32)> = None;
                for (entity, transform, team) in &units {
                    if team.0 != 0 { continue; }
                    let dist = transform.translation.truncate().distance(world_pos);
                    if dist < 50.0 {
                        if closest.is_none() || dist < closest.unwrap().1 {
                            closest = Some((entity, dist));
                        }
                    }
                }
                if let Some((entity, _)) = closest {
                    commands.entity(entity).insert(Selected);
                }
            }
        }

        drag_state.start = None;
        drag_state.current = None;
        drag_state.dragging = false;
    }
}

pub fn handle_drag_selection(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    units: Query<(Entity, &Transform, &Team), With<Unit>>,
    selected: Query<Entity, With<Selected>>,
    mut drag_state: Local<DragState>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    let Ok(window) = windows.single() else { return };
    let Ok((camera, cam_transform)) = camera_q.single() else { return };
    let Some(cursor_pos) = window.cursor_position() else { return };

    if mouse.just_pressed(MouseButton::Left) {
        if let Ok(world_pos) = camera.viewport_to_world_2d(cam_transform, cursor_pos) {
            drag_state.start = Some(world_pos);
            drag_state.current = Some(world_pos);
            drag_state.dragging = false;
        }
    }

    if mouse.pressed(MouseButton::Left) {
        if let Ok(world_pos) = camera.viewport_to_world_2d(cam_transform, cursor_pos) {
            drag_state.current = Some(world_pos);
            if let Some(start) = drag_state.start {
                if start.distance(world_pos) > DRAG_THRESHOLD {
                    drag_state.dragging = true;
                }
            }
        }
    }

    if mouse.just_released(MouseButton::Left) && drag_state.dragging {
        if let (Some(start), Some(end)) = (drag_state.start, drag_state.current) {
            let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);

            if !shift {
                for entity in &selected {
                    commands.entity(entity).remove::<Selected>();
                }
            }

            let min_x = start.x.min(end.x);
            let max_x = start.x.max(end.x);
            let min_y = start.y.min(end.y);
            let max_y = start.y.max(end.y);

            for (entity, transform, team) in &units {
                if team.0 != 0 { continue; }
                let pos = transform.translation.truncate();
                if pos.x >= min_x && pos.x <= max_x && pos.y >= min_y && pos.y <= max_y {
                    commands.entity(entity).insert(Selected);
                }
            }
        }

        drag_state.start = None;
        drag_state.current = None;
        drag_state.dragging = false;
    }

    if mouse.just_released(MouseButton::Left) {
        drag_state.start = None;
        drag_state.current = None;
        drag_state.dragging = false;
    }
}

pub fn draw_selection_box(
    mut gizmos: Gizmos,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut drag_state: Local<DragState>,
) {
    let Ok(window) = windows.single() else { return };
    let Ok((camera, cam_transform)) = camera_q.single() else { return };
    let Some(cursor_pos) = window.cursor_position() else { return };

    if mouse.just_pressed(MouseButton::Left) {
        if let Ok(world_pos) = camera.viewport_to_world_2d(cam_transform, cursor_pos) {
            drag_state.start = Some(world_pos);
        }
    }

    if mouse.pressed(MouseButton::Left) {
        if let Ok(world_pos) = camera.viewport_to_world_2d(cam_transform, cursor_pos) {
            drag_state.current = Some(world_pos);
            if let Some(start) = drag_state.start {
                if start.distance(world_pos) > DRAG_THRESHOLD {
                    drag_state.dragging = true;
                }
            }
        }
    }

    if drag_state.dragging {
        if let (Some(start), Some(current)) = (drag_state.start, drag_state.current) {
            let center = (start + current) / 2.0;
            let size = (current - start).abs();
            gizmos.rect_2d(
                Isometry2d::from_translation(center),
                size,
                Color::srgba(0.0, 1.0, 0.0, 0.8),
            );
        }
    }

    if mouse.just_released(MouseButton::Left) {
        drag_state.start = None;
        drag_state.current = None;
        drag_state.dragging = false;
    }
}

pub fn draw_selection_indicators(
    mut gizmos: Gizmos,
    selected_units: Query<&Transform, (With<Unit>, With<Selected>)>,
) {
    for transform in &selected_units {
        let pos = transform.translation.truncate();
        gizmos.circle_2d(
            Isometry2d::from_translation(pos - Vec2::new(0.0, 30.0)),
            44.0,
            Color::srgba(0.0, 1.0, 0.0, 0.6),
        );
    }
}

pub fn handle_right_click_command(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    selected_units: Query<(Entity, Option<&crate::resources::components::Carrying>), (With<Unit>, With<Selected>)>,
    enemy_units: Query<(Entity, &Transform, &Team), With<Unit>>,
    resource_nodes: Query<(Entity, &Transform), With<crate::resources::components::ResourceNode>>,
    farm_buildings: Query<(Entity, &Transform, &crate::buildings::components::Building)>,
) {
    if !mouse.just_pressed(MouseButton::Right) {
        return;
    }

    let Ok(window) = windows.single() else { return };
    let Ok((camera, cam_transform)) = camera_q.single() else { return };
    let Some(cursor_pos) = window.cursor_position() else { return };
    let Ok(world_pos) = camera.viewport_to_world_2d(cam_transform, cursor_pos) else { return };

    // Priority 1: right-click on resource node → only villagers gather
    for (res_entity, res_transform) in &resource_nodes {
        let dist = res_transform.translation.truncate().distance(world_pos);
        if dist < 60.0 {
            for (unit_entity, carrying) in &selected_units {
                if carrying.is_some() {
                    commands.entity(unit_entity)
                        .remove::<AttackTarget>()
                        .insert(MoveTarget(res_transform.translation.truncate()))
                        .insert(UnitState::Gathering { resource: res_entity });
                }
            }
            return;
        }
    }

    // Priority 2: right-click on farm → only villagers farm
    for (farm_entity, farm_tf, building) in &farm_buildings {
        if building.kind != crate::buildings::components::BuildingKind::Farm {
            continue;
        }
        let dist = farm_tf.translation.truncate().distance(world_pos);
        if dist < 60.0 {
            for (unit_entity, carrying) in &selected_units {
                if carrying.is_some() {
                    commands.entity(unit_entity)
                        .remove::<AttackTarget>()
                        .insert(MoveTarget(farm_tf.translation.truncate()))
                        .insert(UnitState::FarmingAt { farm: farm_entity });
                }
            }
            return;
        }
    }

    // Priority 3: right-click on enemy → attack
    let mut target_enemy: Option<Entity> = None;
    for (entity, transform, team) in &enemy_units {
        if team.0 == 0 { continue; }
        let dist = transform.translation.truncate().distance(world_pos);
        if dist < 50.0 {
            target_enemy = Some(entity);
            break;
        }
    }

    // Priority 4: move to location
    let selected: Vec<Entity> = selected_units.iter().map(|(e, _)| e).collect();
    let count = selected.len().max(1) as f32;

    for (i, entity) in selected.iter().enumerate() {
        commands.entity(*entity).remove::<AttackTarget>();

        if let Some(enemy) = target_enemy {
            commands.entity(*entity)
                .insert(AttackTarget(enemy))
                .insert(UnitState::Attacking);
        } else {
            let offset = formation_offset(i, count as usize);
            commands.entity(*entity)
                .insert(MoveTarget(world_pos + offset))
                .insert(UnitState::Moving);
        }
    }
}

fn formation_offset(index: usize, total: usize) -> Vec2 {
    if total <= 1 {
        return Vec2::ZERO;
    }
    let cols = (total as f32).sqrt().ceil() as usize;
    let row = index / cols;
    let col = index % cols;
    let spacing = 50.0;
    let offset_x = (col as f32 - (cols as f32 - 1.0) / 2.0) * spacing;
    let offset_y = (row as f32 - ((total / cols) as f32 - 1.0) / 2.0) * spacing;
    Vec2::new(offset_x, -offset_y)
}
