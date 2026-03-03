use bevy::prelude::*;
use super::components::*;
use crate::resources::components::{Carrying, ResourceKind, ResourceNode};

pub fn attack_damage_system(
    mut commands: Commands,
    mut attackers: Query<(Entity, &Transform, &mut AttackStats, &AttackTarget, &mut UnitState), With<Unit>>,
    mut targets: Query<(Entity, &Transform, &mut Health), With<Unit>>,
    time: Res<Time>,
) {
    let mut damage_events: Vec<(Entity, f32)> = Vec::new();

    for (attacker_entity, attacker_transform, mut attack_stats, attack_target, mut state) in &mut attackers {
        let target_entity = attack_target.0;

        let Ok((_, target_transform, _)) = targets.get(target_entity) else {
            commands.entity(attacker_entity).remove::<AttackTarget>();
            *state = UnitState::Idle;
            continue;
        };

        let distance = attacker_transform.translation.truncate()
            .distance(target_transform.translation.truncate());

        let range_px = attack_stats.range * crate::map::TILE_SIZE;

        if distance <= range_px {
            *state = UnitState::Attacking;
            attack_stats.cooldown.tick(time.delta());
            if attack_stats.cooldown.just_finished() {
                damage_events.push((target_entity, attack_stats.damage));
            }
        } else {
            *state = UnitState::Moving;
        }
    }

    for (target_entity, damage) in damage_events {
        if let Ok((_, _, mut health)) = targets.get_mut(target_entity) {
            health.current -= damage;
        }
    }
}

pub fn chase_system(
    mut query: Query<(&mut Transform, &Speed, &AttackStats, &AttackTarget, &UnitState), With<Unit>>,
    target_transforms: Query<&Transform, Without<Speed>>,
    time: Res<Time>,
) {
    for (mut transform, speed, attack_stats, attack_target, state) in &mut query {
        if *state != UnitState::Moving && *state != UnitState::Attacking {
            continue;
        }

        let Ok(target_tf) = target_transforms.get(attack_target.0) else {
            continue;
        };

        let current = transform.translation.truncate();
        let target_pos = target_tf.translation.truncate();
        let distance = current.distance(target_pos);
        let range_px = attack_stats.range * crate::map::TILE_SIZE;

        if distance > range_px {
            let direction = (target_pos - current).normalize_or_zero();
            let velocity = direction * speed.0 * crate::map::TILE_SIZE * time.delta_secs();
            transform.translation.x += velocity.x;
            transform.translation.y += velocity.y;
        }
    }
}

pub fn death_system(
    mut commands: Commands,
    query: Query<(Entity, &Health), With<Unit>>,
) {
    for (entity, health) in &query {
        if health.current <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

pub fn health_bar_system(
    mut gizmos: Gizmos,
    query: Query<(&Transform, &Health), With<Unit>>,
) {
    for (transform, health) in &query {
        if health.fraction() >= 1.0 {
            continue;
        }

        let pos = transform.translation.truncate() + Vec2::new(0.0, 30.0);
        let bar_width = 40.0;

        gizmos.rect_2d(
            Isometry2d::from_translation(pos),
            Vec2::new(bar_width, 6.0),
            Color::srgba(0.2, 0.2, 0.2, 0.8),
        );

        let fill_width = bar_width * health.fraction();
        let fill_center = Vec2::new(
            pos.x - (bar_width - fill_width) / 2.0,
            pos.y,
        );
        let color = if health.fraction() > 0.5 {
            Color::srgba(0.0, 0.8, 0.0, 0.9)
        } else if health.fraction() > 0.25 {
            Color::srgba(0.9, 0.7, 0.0, 0.9)
        } else {
            Color::srgba(0.9, 0.1, 0.0, 0.9)
        };
        gizmos.rect_2d(
            Isometry2d::from_translation(fill_center),
            Vec2::new(fill_width, 6.0),
            color,
        );
    }
}

pub fn carry_indicator_system(
    mut gizmos: Gizmos,
    villagers: Query<(&Transform, &Carrying, &UnitState), With<Unit>>,
) {
    for (transform, carrying, state) in &villagers {
        let is_working = matches!(
            state,
            UnitState::Gathering { .. } | UnitState::Returning { .. } | UnitState::FarmingAt { .. }
        );
        if !is_working && !carrying.has_resources() {
            continue;
        }

        let pos = transform.translation.truncate() + Vec2::new(0.0, -28.0);
        let bar_width = 30.0;
        let bar_height = 4.0;

        gizmos.rect_2d(
            Isometry2d::from_translation(pos),
            Vec2::new(bar_width, bar_height),
            Color::srgba(0.15, 0.15, 0.15, 0.7),
        );

        if carrying.amount > 0 {
            let fraction = carrying.amount as f32 / Carrying::MAX_CARRY as f32;
            let fill_width = bar_width * fraction;
            let fill_center = Vec2::new(
                pos.x - (bar_width - fill_width) / 2.0,
                pos.y,
            );
            let color = match carrying.kind {
                Some(ResourceKind::Food) => Color::srgba(0.9, 0.2, 0.2, 0.9),
                Some(ResourceKind::Wood) => Color::srgba(0.2, 0.7, 0.2, 0.9),
                Some(ResourceKind::Gold) => Color::srgba(1.0, 0.84, 0.0, 0.9),
                Some(ResourceKind::Stone) => Color::srgba(0.6, 0.6, 0.6, 0.9),
                None => Color::srgba(0.5, 0.5, 0.5, 0.9),
            };
            gizmos.rect_2d(
                Isometry2d::from_translation(fill_center),
                Vec2::new(fill_width, bar_height),
                color,
            );
        }
    }
}

pub fn gather_visual_system(
    mut gizmos: Gizmos,
    gatherers: Query<(&Transform, &UnitState), With<Unit>>,
    resource_nodes: Query<&Transform, With<ResourceNode>>,
    time: Res<Time>,
) {
    let pulse = (time.elapsed_secs() * 4.0).sin() * 0.3 + 0.7;

    for (unit_tf, state) in &gatherers {
        let res_entity = match state {
            UnitState::Gathering { resource } => *resource,
            _ => continue,
        };

        let Ok(res_tf) = resource_nodes.get(res_entity) else { continue; };
        let dist = unit_tf.translation.truncate().distance(res_tf.translation.truncate());
        if dist > crate::map::TILE_SIZE * 1.5 { continue; }

        let mid = (unit_tf.translation.truncate() + res_tf.translation.truncate()) / 2.0;
        let color = Color::srgba(1.0, 0.9, 0.3, pulse * 0.6);
        gizmos.circle_2d(
            Isometry2d::from_translation(mid),
            12.0,
            color,
        );
        gizmos.circle_2d(
            Isometry2d::from_translation(res_tf.translation.truncate()),
            50.0 * pulse,
            Color::srgba(1.0, 1.0, 1.0, 0.15),
        );
    }
}
