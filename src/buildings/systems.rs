use bevy::prelude::*;
use crate::map::{GridPosition, TILE_SIZE};
use crate::units::components::*;
use crate::units::types::{UnitKind, UnitSprites, spawn_unit};
use crate::resources::components::PlayerResources;
use super::components::*;

pub fn training_system(
    mut commands: Commands,
    mut buildings: Query<(Entity, &Building, &mut TrainingQueue, &Transform, &Team)>,
    _player_resources: ResMut<PlayerResources>,
    sprites: Res<UnitSprites>,
    time: Res<Time>,
) {
    for (_entity, building, mut queue, transform, team) in &mut buildings {
        if queue.queue.is_empty() {
            continue;
        }

        queue.queue[0].remaining.tick(time.delta());

        if queue.queue[0].remaining.just_finished() {
            let slot = queue.queue.remove(0);
            let kind = slot.kind;

            let rally = building.rally_point.unwrap_or_else(|| {
                transform.translation.truncate() + Vec2::new(0.0, -TILE_SIZE * 3.0)
            });

            let spawn_pos = transform.translation.truncate() + Vec2::new(0.0, -TILE_SIZE * 2.0);
            let grid = GridPosition::from_world(spawn_pos);

            let entity = spawn_unit(&mut commands, sprites.get(kind), kind, *team, grid, spawn_pos);
            commands.entity(entity)
                .insert(MoveTarget(rally))
                .insert(UnitState::Moving);
        }
    }
}

pub fn enqueue_unit(
    _commands: &mut Commands,
    _building_entity: Entity,
    queue: &mut TrainingQueue,
    kind: UnitKind,
    resources: &mut PlayerResources,
    current_age: &CurrentAge,
) -> bool {
    if queue.queue.len() >= 5 {
        return false;
    }

    if current_age.0 < kind.required_age() {
        return false;
    }

    let (food, wood, gold, stone) = kind.train_cost();
    if !resources.spend(food, wood, gold, stone) {
        return false;
    }

    queue.queue.push(TrainingSlot {
        kind,
        remaining: Timer::from_seconds(kind.train_time(), TimerMode::Once),
    });

    true
}

pub fn age_up_system(
    mut age: ResMut<CurrentAge>,
    mut progress: ResMut<AgeUpProgress>,
    time: Res<Time>,
) {
    if !progress.researching {
        return;
    }

    if let Some(ref mut timer) = progress.timer {
        timer.tick(time.delta());
        if timer.just_finished() {
            if let Some(target) = progress.target_age {
                age.0 = target;
            }
            progress.researching = false;
            progress.timer = None;
            progress.target_age = None;
        }
    }
}

pub fn start_age_up(
    resources: &mut PlayerResources,
    current_age: &CurrentAge,
    progress: &mut AgeUpProgress,
    player_buildings: &[(BuildingKind, u8)],
) -> bool {
    if progress.researching {
        return false;
    }

    let Some(next) = current_age.0.next() else {
        return false;
    };

    if next == Age::Feudal {
        let has_barracks = player_buildings.iter().any(|(k, _)| *k == BuildingKind::Barracks);
        let other_military = player_buildings.iter().any(|(k, _)| {
            matches!(k, BuildingKind::ArcheryRange | BuildingKind::Stable)
        });
        if !has_barracks || !other_military {
            return false;
        }
    }

    let Some((food, wood, gold, stone)) = current_age.0.advance_cost() else {
        return false;
    };

    if !resources.spend(food, wood, gold, stone) {
        return false;
    }

    let research_time = match next {
        Age::Feudal => 130.0,
        Age::Castle => 160.0,
        Age::Imperial => 190.0,
        Age::Dark => 0.0,
    };

    progress.researching = true;
    progress.timer = Some(Timer::from_seconds(research_time, TimerMode::Once));
    progress.target_age = Some(next);
    true
}

pub fn building_death_system(
    mut commands: Commands,
    query: Query<(Entity, &Health), With<Building>>,
) {
    for (entity, health) in &query {
        if health.current <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

pub fn building_selection_system(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform), With<crate::camera::MainCamera>>,
    buildings: Query<(Entity, &Transform, &Building, &Team)>,
    selected: Query<Entity, With<Selected>>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    if !mouse.just_released(MouseButton::Left) {
        return;
    }

    let Ok(window) = windows.single() else { return };
    let Ok((camera, cam_transform)) = camera_q.single() else { return };
    let Some(cursor_pos) = window.cursor_position() else { return };
    let Ok(world_pos) = camera.viewport_to_world_2d(cam_transform, cursor_pos) else { return };

    for (entity, transform, building, team) in &buildings {
        if team.0 != 0 { continue; }
        let (tw, th) = building.kind.tile_size();
        let half_w = tw as f32 * TILE_SIZE / 2.0;
        let half_h = th as f32 * TILE_SIZE / 2.0;
        let pos = transform.translation.truncate();

        if world_pos.x >= pos.x - half_w && world_pos.x <= pos.x + half_w
            && world_pos.y >= pos.y - half_h && world_pos.y <= pos.y + half_h
        {
            let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
            if !shift {
                for e in &selected {
                    commands.entity(e).remove::<Selected>();
                }
            }
            commands.entity(entity).insert(Selected);
            return;
        }
    }
}

pub fn keyboard_training_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut buildings_selected: Query<(&Building, &mut TrainingQueue, &Team), With<Selected>>,
    all_player_buildings: Query<(&Building, &Team), Without<Selected>>,
    mut resources: ResMut<PlayerResources>,
    age: Res<CurrentAge>,
    mut age_progress: ResMut<AgeUpProgress>,
    mut commands: Commands,
) {
    for (building, mut queue, team) in &mut buildings_selected {
        if team.0 != 0 { continue; }

        let trainable = building.kind.can_train();

        if keys.just_pressed(KeyCode::KeyQ) {
            if let Some(&kind) = trainable.first() {
                enqueue_unit(&mut commands, Entity::PLACEHOLDER, &mut queue, kind, &mut resources, &age);
            }
        }
        if keys.just_pressed(KeyCode::KeyW) {
            if let Some(&kind) = trainable.get(1) {
                enqueue_unit(&mut commands, Entity::PLACEHOLDER, &mut queue, kind, &mut resources, &age);
            }
        }

        if keys.just_pressed(KeyCode::Escape) && !queue.queue.is_empty() {
            let cancelled = queue.queue.remove(0);
            let (f, w, g, s) = cancelled.kind.train_cost();
            resources.food += f;
            resources.wood += w;
            resources.gold += g;
            resources.stone += s;
        }

        if keys.just_pressed(KeyCode::KeyP) {
            if building.kind == BuildingKind::TownCenter {
                let mut player_buildings: Vec<(BuildingKind, u8)> = Vec::new();
                player_buildings.push((building.kind, team.0));
                for (b, t) in &all_player_buildings {
                    if t.0 == 0 {
                        player_buildings.push((b.kind, t.0));
                    }
                }
                start_age_up(&mut resources, &age, &mut age_progress, &player_buildings);
            }
        }
    }
}

pub fn rally_point_system(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform), With<crate::camera::MainCamera>>,
    mut buildings: Query<(&mut Building, &Team), With<Selected>>,
) {
    if !mouse.just_pressed(MouseButton::Right) {
        return;
    }

    let Ok(window) = windows.single() else { return };
    let Ok((camera, cam_transform)) = camera_q.single() else { return };
    let Some(cursor_pos) = window.cursor_position() else { return };
    let Ok(world_pos) = camera.viewport_to_world_2d(cam_transform, cursor_pos) else { return };

    for (mut building, team) in &mut buildings {
        if team.0 != 0 { continue; }
        building.rally_point = Some(world_pos);
    }
}
