use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use super::components::*;
use crate::camera::MainCamera;

#[derive(Resource, Default)]
pub struct ControlGroups {
    pub groups: [Vec<Entity>; 10],
    last_tap: [Option<f64>; 10],
}

const DOUBLE_TAP_WINDOW: f64 = 0.4;

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
    resource_nodes: Query<(Entity, &Transform), With<crate::resources::components::ResourceNode>>,
    selected: Query<Entity, With<Selected>>,
    mut drag_state: Local<DragState>,
    keys: Res<ButtonInput<KeyCode>>,
    ui_interactions: Query<&Interaction, With<Node>>,
    placement: Res<crate::buildings::placement::PlacementMode>,
) {
    if placement.active { return; }

    let ui_busy = ui_interactions.iter().any(|i| *i != Interaction::None);

    let Ok(window) = windows.single() else { return };
    let Ok((camera, cam_transform)) = camera_q.single() else { return };
    let Some(cursor_pos) = window.cursor_position() else { return };

    if ui_busy { return; }

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

                let mut closest_unit: Option<(Entity, f32)> = None;
                for (entity, transform, _team) in &units {
                    let dist = transform.translation.truncate().distance(world_pos);
                    if dist < 50.0 {
                        if closest_unit.is_none() || dist < closest_unit.unwrap().1 {
                            closest_unit = Some((entity, dist));
                        }
                    }
                }

                if let Some((entity, _)) = closest_unit {
                    commands.entity(entity).insert(Selected);
                } else {
                    let mut closest_res: Option<(Entity, f32)> = None;
                    for (entity, transform) in &resource_nodes {
                        let dist = transform.translation.truncate().distance(world_pos);
                        if dist < 40.0 {
                            if closest_res.is_none() || dist < closest_res.unwrap().1 {
                                closest_res = Some((entity, dist));
                            }
                        }
                    }
                    if let Some((entity, _)) = closest_res {
                        commands.entity(entity).insert(Selected);
                    }
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
    placement: Res<crate::buildings::placement::PlacementMode>,
) {
    if placement.active { return; }
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

            for (entity, transform, _team) in &units {
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

fn team_selection_color(team: Option<&Team>) -> Color {
    match team {
        Some(t) if t.0 != 0 => Color::srgba(1.0, 0.3, 0.3, 0.7),
        _ => Color::srgba(1.0, 1.0, 1.0, 0.7),
    }
}

pub fn draw_selection_indicators(
    mut gizmos: Gizmos,
    selected_units: Query<(&Transform, Option<&Team>), (With<Unit>, With<Selected>)>,
    selected_buildings: Query<(&Transform, &crate::buildings::components::Building, Option<&Team>), (With<Selected>, Without<Unit>)>,
    selected_resources: Query<&Transform, (With<crate::resources::components::ResourceNode>, With<Selected>, Without<Unit>, Without<crate::buildings::components::Building>)>,
) {
    for (transform, team) in &selected_units {
        let pos = transform.translation.truncate() - Vec2::new(0.0, 20.0);
        let color = team_selection_color(team);
        gizmos.ellipse_2d(
            Isometry2d::from_translation(pos),
            Vec2::new(28.0, 14.0),
            color,
        );
    }

    for (transform, building, team) in &selected_buildings {
        let pos = transform.translation.truncate();
        let (tw, th) = building.kind.tile_size();
        let half_w = tw as f32 * crate::map::TILE_SIZE * 0.4;
        let half_h = th as f32 * crate::map::TILE_SIZE * 0.2;
        let color = team_selection_color(team);
        gizmos.ellipse_2d(
            Isometry2d::from_translation(pos),
            Vec2::new(half_w, half_h),
            color,
        );
    }

    for transform in &selected_resources {
        let pos = transform.translation.truncate();
        let color = team_selection_color(None);
        gizmos.ellipse_2d(
            Isometry2d::from_translation(pos),
            Vec2::new(22.0, 11.0),
            color,
        );
    }
}

pub fn handle_right_click_command(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    selected_units: Query<(Entity, &Transform, Option<&crate::resources::components::Carrying>), (With<Unit>, With<Selected>)>,
    enemy_units: Query<(Entity, &Transform, &Team), With<Unit>>,
    resource_nodes: Query<(Entity, &Transform), With<crate::resources::components::ResourceNode>>,
    farm_buildings: Query<(Entity, &Transform, &crate::buildings::components::Building)>,
    keys: Res<ButtonInput<KeyCode>>,
    mut attack_move_pending: Local<bool>,
    monk_units: Query<Entity, (With<Unit>, With<Selected>, With<MonkUnit>)>,
    relic_carriers: Query<Entity, (With<Unit>, With<RelicCarrier>)>,
    relics: Query<(Entity, &Transform), With<Relic>>,
    monastery_buildings: Query<(Entity, &Transform, &Team, &crate::buildings::components::Building)>,
    friendly_units: Query<(Entity, &Transform, &Team), With<Unit>>,
) {
    // A key activates attack-move cursor; next left-click issues the attack-move command
    if keys.just_pressed(KeyCode::KeyA) {
        *attack_move_pending = true;
    }

    // Handle attack-move: A + left-click
    if *attack_move_pending && mouse.just_pressed(MouseButton::Left) {
        *attack_move_pending = false;
        let Ok(window) = windows.single() else { return };
        let Ok((camera, cam_transform)) = camera_q.single() else { return };
        let Some(cursor_pos) = window.cursor_position() else { return };
        let Ok(world_pos) = camera.viewport_to_world_2d(cam_transform, cursor_pos) else { return };

        let selected: Vec<(Entity, Vec2)> = selected_units.iter()
            .map(|(e, tf, _)| (e, tf.translation.truncate()))
            .collect();
        let count = selected.len();

        for (i, (entity, _)) in selected.iter().enumerate() {
            let offset = formation_offset(i, count);
            commands.entity(*entity)
                .remove::<AttackTarget>()
                .insert(MoveTarget(world_pos + offset))
                .insert(MovementIntent::AttackMove)
                .insert(UnitState::Moving);
        }
        return;
    }

    if !mouse.just_pressed(MouseButton::Right) {
        return;
    }

    let Ok(window) = windows.single() else { return };
    let Ok((camera, cam_transform)) = camera_q.single() else { return };
    let Some(cursor_pos) = window.cursor_position() else { return };
    let Ok(world_pos) = camera.viewport_to_world_2d(cam_transform, cursor_pos) else { return };

    *attack_move_pending = false;

    // Ctrl+Shift+right-click = patrol
    let ctrl = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);

    if ctrl && shift {
        let selected: Vec<(Entity, Vec2)> = selected_units.iter()
            .map(|(e, tf, _)| (e, tf.translation.truncate()))
            .collect();
        for (entity, current_pos) in &selected {
            commands.entity(*entity)
                .remove::<AttackTarget>()
                .insert(MoveTarget(world_pos))
                .insert(MovementIntent::Patrol {
                    a: *current_pos,
                    b: world_pos,
                    going_to_b: true,
                })
                .insert(UnitState::Moving);
        }
        return;
    }

    // Priority 0a: monks with relics right-click on own monastery → deposit
    {
        let monk_carriers: Vec<Entity> = selected_units.iter()
            .filter(|(e, _, _)| monk_units.contains(*e))
            .filter(|(e, _, _)| relic_carriers.contains(*e))
            .map(|(e, _, _)| e)
            .collect();
        if !monk_carriers.is_empty() {
            for (mon_e, mon_tf, mon_team, building) in &monastery_buildings {
                if building.kind != crate::buildings::components::BuildingKind::Monastery {
                    continue;
                }
                if mon_team.0 != 0 { continue; }
                let dist = mon_tf.translation.truncate().distance(world_pos);
                if dist < 80.0 {
                    for &monk_e in &monk_carriers {
                        commands.entity(monk_e)
                            .remove::<AttackTarget>()
                            .remove::<HealTarget>()
                            .remove::<ConvertTarget>()
                            .insert(MoveTarget(mon_tf.translation.truncate()))
                            .insert(UnitState::Moving);
                    }
                    return;
                }
            }
        }
    }

    // Priority 0b: monks right-click on relic → pick up
    {
        let monks: Vec<Entity> = selected_units.iter()
            .filter(|(e, _, _)| monk_units.contains(*e))
            .filter(|(e, _, _)| !relic_carriers.contains(*e))
            .map(|(e, _, _)| e)
            .collect();
        if !monks.is_empty() {
            for (relic_e, relic_tf) in &relics {
                let dist = relic_tf.translation.truncate().distance(world_pos);
                if dist < 60.0 {
                    for &monk_e in &monks {
                        commands.entity(monk_e)
                            .remove::<AttackTarget>()
                            .remove::<HealTarget>()
                            .remove::<ConvertTarget>()
                            .insert(RelicCarrier(relic_e))
                            .insert(MoveTarget(relic_tf.translation.truncate()))
                            .insert(UnitState::Moving);
                    }
                    return;
                }
            }
        }
    }

    // Priority 0c: monks right-click on friendly unit → heal
    {
        let monks: Vec<Entity> = selected_units.iter()
            .filter(|(e, _, _)| monk_units.contains(*e))
            .map(|(e, _, _)| e)
            .collect();
        if !monks.is_empty() {
            for (ally_e, ally_tf, ally_team) in &friendly_units {
                if ally_team.0 != 0 { continue; }
                let dist = ally_tf.translation.truncate().distance(world_pos);
                if dist < 50.0 && !monks.contains(&ally_e) {
                    for &monk_e in &monks {
                        commands.entity(monk_e)
                            .remove::<AttackTarget>()
                            .remove::<ConvertTarget>()
                            .insert(HealTarget(ally_e))
                            .insert(MoveTarget(ally_tf.translation.truncate()))
                            .insert(UnitState::Moving);
                    }
                    return;
                }
            }
        }
    }

    // Priority 1: right-click on resource node → only villagers gather
    for (res_entity, res_transform) in &resource_nodes {
        let dist = res_transform.translation.truncate().distance(world_pos);
        if dist < 60.0 {
            for (unit_entity, _, carrying) in &selected_units {
                if carrying.is_some() {
                    commands.entity(unit_entity)
                        .remove::<AttackTarget>()
                        .insert(MoveTarget(res_transform.translation.truncate()))
                        .insert(MovementIntent::Move)
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
            for (unit_entity, _, carrying) in &selected_units {
                if carrying.is_some() {
                    commands.entity(unit_entity)
                        .remove::<AttackTarget>()
                        .insert(MoveTarget(farm_tf.translation.truncate()))
                        .insert(MovementIntent::Move)
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
    let selected: Vec<Entity> = selected_units.iter().map(|(e, _, _)| e).collect();
    let count = selected.len().max(1) as f32;

    for (i, entity) in selected.iter().enumerate() {
        commands.entity(*entity).remove::<AttackTarget>();

        if let Some(enemy) = target_enemy {
            if monk_units.contains(*entity) {
                commands.entity(*entity)
                    .remove::<HealTarget>()
                    .insert(ConvertTarget {
                        entity: enemy,
                        progress: Timer::from_seconds(10.0, TimerMode::Once),
                    })
                    .insert(MoveTarget(world_pos))
                    .insert(UnitState::Moving);
            } else {
                commands.entity(*entity)
                    .insert(AttackTarget(enemy))
                    .insert(MovementIntent::Move)
                    .insert(UnitState::Attacking);
            }
        } else {
            let offset = formation_offset(i, count as usize);
            commands.entity(*entity)
                .insert(MoveTarget(world_pos + offset))
                .insert(MovementIntent::Move)
                .insert(UnitState::Moving);
        }
    }
}

pub fn control_group_system(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    mut groups: ResMut<ControlGroups>,
    selected: Query<Entity, With<Selected>>,
    transforms: Query<&Transform, Without<MainCamera>>,
    mut camera_q: Query<&mut Transform, (With<MainCamera>, Without<Unit>, Without<crate::buildings::components::Building>)>,
    time: Res<Time>,
    placement: Res<crate::buildings::placement::PlacementMode>,
    units: Query<Entity, With<Unit>>,
) {
    if placement.active { return; }

    let ctrl = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);

    let digit_keys = [
        KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3, KeyCode::Digit4, KeyCode::Digit5,
        KeyCode::Digit6, KeyCode::Digit7, KeyCode::Digit8, KeyCode::Digit9, KeyCode::Digit0,
    ];

    for (idx, &key) in digit_keys.iter().enumerate() {
        if !keys.just_pressed(key) { continue; }

        if ctrl {
            let entities: Vec<Entity> = selected.iter().collect();
            groups.groups[idx] = entities;
            groups.last_tap[idx] = None;
        } else {
            let now = time.elapsed_secs_f64();
            let is_double = groups.last_tap[idx]
                .map_or(false, |t| now - t < DOUBLE_TAP_WINDOW);

            groups.last_tap[idx] = Some(now);

            groups.groups[idx].retain(|e| units.contains(*e));

            if groups.groups[idx].is_empty() { continue; }

            let prev_selected: Vec<Entity> = selected.iter().collect();
            for e in prev_selected {
                commands.entity(e).remove::<Selected>();
            }
            for &e in &groups.groups[idx] {
                commands.entity(e).insert(Selected);
            }

            if is_double {
                let mut center = Vec2::ZERO;
                let mut count = 0;
                for &e in &groups.groups[idx] {
                    if let Ok(tf) = transforms.get(e) {
                        center += tf.translation.truncate();
                        count += 1;
                    }
                }
                if count > 0 {
                    center /= count as f32;
                    if let Ok(mut cam_tf) = camera_q.single_mut() {
                        cam_tf.translation.x = center.x;
                        cam_tf.translation.y = center.y;
                    }
                }
            }
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
