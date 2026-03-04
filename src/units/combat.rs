use bevy::prelude::*;
use super::components::*;
use crate::resources::components::{Carrying, ResourceKind, ResourceNode};

pub fn attack_damage_system(
    mut commands: Commands,
    mut attackers: Query<(Entity, &Transform, &mut AttackStats, &AttackTarget, &mut UnitState, &Team), (With<Unit>, Without<AreaDamage>)>,
    mut targets: Query<(Entity, &Transform, &mut Health, &Armor, &UnitClass), With<Unit>>,
    time: Res<Time>,
    techs: Res<crate::buildings::research::ResearchedTechnologies>,
) {
    let mut damage_events: Vec<(Entity, f32)> = Vec::new();

    for (attacker_entity, attacker_transform, mut attack_stats, attack_target, mut state, atk_team) in &mut attackers {
        let target_entity = attack_target.0;

        let Ok((_, target_transform, _, target_armor, &target_class)) = targets.get(target_entity) else {
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
                let mut eff_armor = target_armor.clone();
                let mut melee_bonus = 0.0f32;
                let mut pierce_bonus = 0.0f32;

                if atk_team.0 == 0 {
                    melee_bonus = techs.melee_attack_bonus();
                    pierce_bonus = techs.pierce_attack_bonus();
                }
                if target_class != UnitClass::Building {
                    eff_armor.melee += techs.melee_armor_bonus();
                    eff_armor.pierce += techs.pierce_armor_bonus();
                }

                let melee = (attack_stats.melee_damage + melee_bonus - eff_armor.melee).max(0.0);
                let pierce = (attack_stats.pierce_damage + pierce_bonus - eff_armor.pierce).max(0.0);
                let bonus: f32 = attack_stats.bonuses.iter()
                    .filter(|b| b.vs_class == target_class)
                    .map(|b| b.amount)
                    .sum();
                let dmg = (melee + pierce + bonus).max(1.0);
                damage_events.push((target_entity, dmg));
            }
        } else {
            *state = UnitState::Moving;
        }
    }

    for (target_entity, damage) in damage_events {
        if let Ok((_, _, mut health, _, _)) = targets.get_mut(target_entity) {
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
    query: Query<(Entity, &Health, &Team, &UnitState), (With<Unit>, Without<DeathTimer>)>,
    mut dying: Query<(Entity, &mut DeathTimer, &mut Sprite), With<Unit>>,
    time: Res<Time>,
    mut stats: ResMut<crate::ui::stats::GameStats>,
) {
    for (entity, health, team, state) in &query {
        if health.current <= 0.0 && *state != UnitState::Dead {
            if team.0 == 0 {
                stats.units_lost += 1;
            } else {
                stats.enemy_units_killed += 1;
            }
            commands.entity(entity)
                .insert(UnitState::Dead)
                .insert(DeathTimer(Timer::from_seconds(3.0, TimerMode::Once)))
                .remove::<AttackTarget>()
                .remove::<MoveTarget>()
                .remove::<Selected>();
        }
    }

    for (entity, mut timer, mut sprite) in &mut dying {
        timer.0.tick(time.delta());
        let alpha = 1.0 - timer.0.fraction();
        sprite.color = Color::srgba(0.5, 0.5, 0.5, alpha * 0.7);
        if timer.0.just_finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn hp_bar_color(fraction: f32) -> Color {
    if fraction > 0.6 {
        Color::srgba(0.0, 0.8, 0.0, 0.9)
    } else if fraction > 0.3 {
        Color::srgba(0.9, 0.7, 0.0, 0.9)
    } else {
        Color::srgba(0.9, 0.1, 0.0, 0.9)
    }
}

fn draw_hp_bar(gizmos: &mut Gizmos, center: Vec2, bar_width: f32, fraction: f32) {
    gizmos.rect_2d(
        Isometry2d::from_translation(center),
        Vec2::new(bar_width, 6.0),
        Color::srgba(0.2, 0.2, 0.2, 0.8),
    );

    let fill_width = bar_width * fraction.clamp(0.0, 1.0);
    let fill_center = Vec2::new(
        center.x - (bar_width - fill_width) / 2.0,
        center.y,
    );
    gizmos.rect_2d(
        Isometry2d::from_translation(fill_center),
        Vec2::new(fill_width, 6.0),
        hp_bar_color(fraction),
    );
}

pub fn health_bar_system(
    mut gizmos: Gizmos,
    query: Query<(&Transform, &Health), (With<Unit>, Without<Selected>)>,
) {
    for (transform, health) in &query {
        if health.fraction() >= 1.0 {
            continue;
        }
        let pos = transform.translation.truncate() + Vec2::new(0.0, 30.0);
        draw_hp_bar(&mut gizmos, pos, 40.0, health.fraction());
    }
}

pub fn selection_health_bar_system(
    mut gizmos: Gizmos,
    selected_units: Query<(&Transform, &Health), (With<Unit>, With<Selected>)>,
    selected_buildings: Query<
        (&Transform, &Health, &crate::buildings::components::Building),
        (With<Selected>, Without<Unit>, Without<ResourceNode>),
    >,
    selected_resources: Query<
        (&Transform, &ResourceNode),
        (With<Selected>, Without<Unit>, Without<crate::buildings::components::Building>),
    >,
) {
    for (transform, health) in &selected_units {
        let pos = transform.translation.truncate() + Vec2::new(0.0, 30.0);
        draw_hp_bar(&mut gizmos, pos, 40.0, health.fraction());
    }

    for (transform, health, building) in &selected_buildings {
        let (tw, _) = building.kind.tile_size();
        let bar_w = (tw as f32 * crate::map::TILE_SIZE * 0.6).max(50.0);
        let pos = transform.translation.truncate() + Vec2::new(0.0, 40.0);
        draw_hp_bar(&mut gizmos, pos, bar_w, health.fraction());
    }

    for (transform, resource) in &selected_resources {
        let fraction = resource.remaining as f32 / resource.max_amount.max(1) as f32;
        let pos = transform.translation.truncate() + Vec2::new(0.0, 30.0);
        draw_hp_bar(&mut gizmos, pos, 36.0, fraction);
    }
}

pub fn aoe_damage_system(
    mut attackers: Query<(Entity, &Transform, &mut AttackStats, &AttackTarget, &mut UnitState, &Team, &AreaDamage), With<Unit>>,
    mut targets: Query<(Entity, &Transform, &mut Health, &Armor, &Team), (With<Unit>, Without<AreaDamage>)>,
    time: Res<Time>,
) {
    let mut splash_events: Vec<(Vec2, f32, f32, u8)> = Vec::new();

    for (_entity, atk_tf, mut attack, atk_target, mut state, team, aoe) in &mut attackers {
        let Ok((_, target_tf, _, _, _)) = targets.get(atk_target.0) else {
            continue;
        };

        let dist = atk_tf.translation.truncate().distance(target_tf.translation.truncate());
        let range_px = attack.range * crate::map::TILE_SIZE;

        if dist <= range_px {
            *state = UnitState::Attacking;
            attack.cooldown.tick(time.delta());
            if attack.cooldown.just_finished() {
                let impact = target_tf.translation.truncate();
                splash_events.push((impact, aoe.radius * crate::map::TILE_SIZE, attack.pierce_damage, team.0));
            }
        } else {
            *state = UnitState::Moving;
        }
    }

    for (impact, radius, damage, attacker_team) in splash_events {
        for (_, target_tf, mut health, armor, target_team) in &mut targets {
            if target_team.0 == attacker_team {
                continue;
            }
            let dist = target_tf.translation.truncate().distance(impact);
            if dist <= radius {
                let falloff = 1.0 - (dist / radius) * 0.5;
                let effective = ((damage * falloff) - armor.pierce).max(1.0);
                health.current -= effective;
            }
        }
    }
}

pub fn attack_move_scan_system(
    mut commands: Commands,
    movers: Query<(Entity, &Transform, &AttackStats, &Team, &MovementIntent), (With<Unit>, Without<AttackTarget>)>,
    potential_targets: Query<(Entity, &Transform, &Team, &Health), With<Unit>>,
) {
    for (unit_entity, unit_tf, attack, unit_team, intent) in &movers {
        let is_aggressive = matches!(intent, MovementIntent::AttackMove | MovementIntent::Patrol { .. });
        if !is_aggressive {
            continue;
        }

        let scan_range = (attack.range + 4.0) * crate::map::TILE_SIZE;
        let unit_pos = unit_tf.translation.truncate();
        let mut best: Option<(Entity, f32)> = None;

        for (target_entity, target_tf, target_team, target_health) in &potential_targets {
            if target_team.0 == unit_team.0 || target_health.current <= 0.0 {
                continue;
            }
            let dist = unit_pos.distance(target_tf.translation.truncate());
            if dist < scan_range {
                if best.is_none() || dist < best.unwrap().1 {
                    best = Some((target_entity, dist));
                }
            }
        }

        if let Some((enemy, _)) = best {
            commands.entity(unit_entity)
                .insert(AttackTarget(enemy))
                .insert(UnitState::Attacking);
        }
    }
}

pub fn patrol_system(
    mut commands: Commands,
    mut patrollers: Query<(Entity, &Transform, &mut MovementIntent), (With<Unit>, Without<AttackTarget>)>,
) {
    for (entity, transform, mut intent) in &mut patrollers {
        if let MovementIntent::Patrol { a, b, ref mut going_to_b } = *intent {
            let pos = transform.translation.truncate();
            let target = if *going_to_b { b } else { a };
            let dist = pos.distance(target);

            if dist < 20.0 {
                *going_to_b = !*going_to_b;
                let next = if *going_to_b { b } else { a };
                commands.entity(entity).insert(MoveTarget(next));
            }
        }
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
            let fraction = carrying.amount as f32 / carrying.max_carry as f32;
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
