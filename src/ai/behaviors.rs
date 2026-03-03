use bevy::prelude::*;
use crate::units::components::*;
use crate::map::TILE_SIZE;

#[derive(Component, Default)]
pub enum AiBehavior {
    #[default]
    Patrol,
    Chase(Entity),
    Flee,
}

#[derive(Component)]
pub struct PatrolPath {
    pub waypoints: Vec<Vec2>,
    pub current_index: usize,
}

#[derive(Component)]
pub struct DetectionRadius(pub f32);

const DEFAULT_DETECTION: f32 = 8.0;

pub fn ai_detection_system(
    mut commands: Commands,
    mut ai_units: Query<(Entity, &Transform, &mut AiBehavior, Option<&DetectionRadius>, &Team, &Health), With<Unit>>,
    player_units: Query<(Entity, &Transform, &Team), (With<Unit>, Without<AiBehavior>)>,
) {
    let player_positions: Vec<(Entity, Vec2)> = player_units.iter()
        .filter(|(_, _, team)| team.0 == 0)
        .map(|(e, t, _)| (e, t.translation.truncate()))
        .collect();

    for (entity, transform, mut behavior, detection, team, health) in &mut ai_units {
        if team.0 == 0 { continue; }

        let pos = transform.translation.truncate();
        let detect_range = detection.map_or(DEFAULT_DETECTION, |d| d.0) * TILE_SIZE;

        if health.fraction() < 0.2 {
            *behavior = AiBehavior::Flee;
            commands.entity(entity).remove::<AttackTarget>();
            continue;
        }

        match *behavior {
            AiBehavior::Patrol | AiBehavior::Flee => {
                let mut closest: Option<(Entity, f32)> = None;
                for &(player_entity, player_pos) in &player_positions {
                    let dist = pos.distance(player_pos);
                    if dist < detect_range {
                        if closest.is_none() || dist < closest.unwrap().1 {
                            closest = Some((player_entity, dist));
                        }
                    }
                }
                if let Some((target, _)) = closest {
                    if !matches!(*behavior, AiBehavior::Flee) {
                        *behavior = AiBehavior::Chase(target);
                        commands.entity(entity)
                            .insert(AttackTarget(target))
                            .insert(UnitState::Attacking);
                    }
                }
            }
            AiBehavior::Chase(target) => {
                let target_alive = player_positions.iter().any(|(e, _)| *e == target);
                if !target_alive {
                    *behavior = AiBehavior::Patrol;
                    commands.entity(entity).remove::<AttackTarget>();
                }
            }
        }
    }
}

pub fn ai_patrol_system(
    mut query: Query<(&mut Transform, &Speed, &mut PatrolPath, &AiBehavior), With<Unit>>,
    time: Res<Time>,
) {
    for (mut transform, speed, mut patrol, behavior) in &mut query {
        if !matches!(behavior, AiBehavior::Patrol) {
            continue;
        }
        if patrol.waypoints.is_empty() {
            continue;
        }

        let target = patrol.waypoints[patrol.current_index];
        let current = transform.translation.truncate();
        let direction = target - current;
        let distance = direction.length();

        if distance < 5.0 {
            patrol.current_index = (patrol.current_index + 1) % patrol.waypoints.len();
        } else {
            let velocity = direction.normalize() * speed.0 * TILE_SIZE * time.delta_secs();
            transform.translation.x += velocity.x;
            transform.translation.y += velocity.y;
        }
    }
}

pub fn ai_flee_system(
    mut query: Query<(&mut Transform, &Speed, &AiBehavior, &Team), With<Unit>>,
    player_units: Query<&Transform, (With<Unit>, Without<AiBehavior>)>,
    time: Res<Time>,
) {
    for (mut transform, speed, behavior, team) in &mut query {
        if team.0 == 0 { continue; }
        if !matches!(behavior, AiBehavior::Flee) {
            continue;
        }

        let pos = transform.translation.truncate();
        let mut flee_dir = Vec2::ZERO;
        let mut count = 0;

        for player_tf in &player_units {
            let player_pos = player_tf.translation.truncate();
            let dist = pos.distance(player_pos);
            if dist < 10.0 * TILE_SIZE && dist > 0.0 {
                flee_dir += (pos - player_pos).normalize();
                count += 1;
            }
        }

        if count > 0 {
            flee_dir = (flee_dir / count as f32).normalize_or_zero();
            let velocity = flee_dir * speed.0 * TILE_SIZE * time.delta_secs();
            transform.translation.x += velocity.x;
            transform.translation.y += velocity.y;
        }
    }
}
