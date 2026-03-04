use bevy::prelude::*;
use super::components::*;
use crate::map::TILE_SIZE;
use crate::buildings::components::{Building, RelicStorage};

const HEAL_RANGE_PX: f32 = 4.0 * TILE_SIZE;
const HEAL_AMOUNT: f32 = 2.0;
const CONVERT_RANGE_PX: f32 = 9.0 * TILE_SIZE;

pub fn monk_heal_system(
    mut commands: Commands,
    mut monks: Query<(Entity, &Transform, &Team, &mut AttackStats, &HealTarget), (With<Unit>, With<MonkUnit>)>,
    mut targets: Query<(Entity, &Transform, &mut Health, &Team), With<Unit>>,
    time: Res<Time>,
) {
    let mut healed: Vec<(Entity, f32)> = Vec::new();

    for (monk_e, monk_tf, monk_team, mut stats, heal_target) in &mut monks {
        let target_e = heal_target.0;

        let Ok((_, target_tf, target_health, target_team)) = targets.get(target_e) else {
            commands.entity(monk_e).remove::<HealTarget>();
            commands.entity(monk_e).insert(UnitState::Idle);
            continue;
        };

        if target_team.0 != monk_team.0 || target_health.current >= target_health.max {
            commands.entity(monk_e).remove::<HealTarget>();
            commands.entity(monk_e).insert(UnitState::Idle);
            continue;
        }

        let dist = monk_tf.translation.truncate()
            .distance(target_tf.translation.truncate());

        if dist <= HEAL_RANGE_PX {
            stats.cooldown.tick(time.delta());
            if stats.cooldown.just_finished() {
                healed.push((target_e, HEAL_AMOUNT));
            }
        } else {
            commands.entity(monk_e)
                .insert(MoveTarget(target_tf.translation.truncate()))
                .insert(UnitState::Moving);
        }
    }

    for (entity, amount) in healed {
        if let Ok((_, _, mut health, _)) = targets.get_mut(entity) {
            health.current = (health.current + amount).min(health.max);
        }
    }
}

pub fn monk_convert_system(
    mut commands: Commands,
    mut converters: Query<(Entity, &Transform, &Team, &mut ConvertTarget), (With<Unit>, With<MonkUnit>)>,
    targets: Query<(Entity, &Transform, &Team), With<Unit>>,
    time: Res<Time>,
    mut stats: ResMut<crate::ui::stats::GameStats>,
) {
    for (monk_e, monk_tf, monk_team, mut convert) in &mut converters {
        let target_e = convert.entity;

        let Ok((_, target_tf, target_team)) = targets.get(target_e) else {
            commands.entity(monk_e).remove::<ConvertTarget>();
            commands.entity(monk_e).insert(UnitState::Idle);
            continue;
        };

        if target_team.0 == monk_team.0 {
            commands.entity(monk_e).remove::<ConvertTarget>();
            commands.entity(monk_e).insert(UnitState::Idle);
            continue;
        }

        let dist = monk_tf.translation.truncate()
            .distance(target_tf.translation.truncate());

        if dist <= CONVERT_RANGE_PX {
            convert.progress.tick(time.delta());
            if convert.progress.just_finished() {
                commands.entity(target_e).insert(*monk_team);
                commands.entity(target_e).remove::<AttackTarget>();
                commands.entity(target_e).insert(UnitState::Idle);
                commands.entity(monk_e).remove::<ConvertTarget>();
                commands.entity(monk_e).insert(UnitState::Idle);
                if monk_team.0 == 0 {
                    stats.conversions += 1;
                }
            }
        } else {
            commands.entity(monk_e)
                .insert(MoveTarget(target_tf.translation.truncate()))
                .insert(UnitState::Moving);
        }
    }
}

pub fn monk_auto_heal_system(
    mut commands: Commands,
    idle_monks: Query<
        (Entity, &Transform, &Team),
        (With<MonkUnit>, Without<HealTarget>, Without<ConvertTarget>, Without<MoveTarget>, Without<RelicCarrier>),
    >,
    injured: Query<(Entity, &Transform, &Health, &Team), With<Unit>>,
) {
    for (monk_e, monk_tf, monk_team) in &idle_monks {
        let monk_pos = monk_tf.translation.truncate();
        let mut best: Option<(Entity, f32)> = None;

        for (target_e, target_tf, health, team) in &injured {
            if target_e == monk_e || team.0 != monk_team.0 {
                continue;
            }
            if health.current >= health.max || health.current <= 0.0 {
                continue;
            }
            let dist = monk_pos.distance(target_tf.translation.truncate());
            if dist < HEAL_RANGE_PX {
                if best.is_none() || dist < best.unwrap().1 {
                    best = Some((target_e, dist));
                }
            }
        }

        if let Some((target_e, _)) = best {
            commands.entity(monk_e).insert(HealTarget(target_e));
        }
    }
}

pub fn relic_pickup_system(
    mut commands: Commands,
    monks: Query<(Entity, &Transform, &RelicCarrier), (With<MonkUnit>, Without<Relic>)>,
    relics: Query<(Entity, &Transform, &Visibility), With<Relic>>,
) {
    for (monk_e, monk_tf, carrier) in &monks {
        let relic_e = carrier.0;
        let Ok((_, relic_tf, vis)) = relics.get(relic_e) else {
            commands.entity(monk_e).remove::<RelicCarrier>();
            continue;
        };
        if *vis == Visibility::Hidden {
            continue;
        }
        let dist = monk_tf.translation.truncate().distance(relic_tf.translation.truncate());
        if dist < TILE_SIZE * 1.5 {
            commands.entity(relic_e).insert(Visibility::Hidden);
            commands.entity(monk_e).remove::<MoveTarget>();
            commands.entity(monk_e).insert(UnitState::Idle);
        }
    }
}

pub fn relic_deposit_system(
    mut commands: Commands,
    monks: Query<(Entity, &Transform, &Team, &RelicCarrier), With<MonkUnit>>,
    relics: Query<&Visibility, With<Relic>>,
    mut monasteries: Query<(&Transform, &Team, &mut RelicStorage), With<Building>>,
) {
    for (monk_e, monk_tf, monk_team, carrier) in &monks {
        if let Ok(vis) = relics.get(carrier.0) {
            if *vis != Visibility::Hidden {
                continue;
            }
        }

        let monk_pos = monk_tf.translation.truncate();

        for (mon_tf, mon_team, mut storage) in &mut monasteries {
            if mon_team.0 != monk_team.0 {
                continue;
            }
            let dist = monk_pos.distance(mon_tf.translation.truncate());
            if dist < TILE_SIZE * 3.0 {
                storage.relics.push(carrier.0);
                commands.entity(monk_e).remove::<RelicCarrier>();
                commands.entity(monk_e).remove::<MoveTarget>();
                commands.entity(monk_e).insert(UnitState::Idle);
                return;
            }
        }
    }
}

pub fn relic_income_system(
    monasteries: Query<(&Team, &RelicStorage)>,
    mut resources: ResMut<crate::resources::components::PlayerResources>,
    time: Res<Time>,
    mut accumulated: Local<f32>,
) {
    let mut total_relics = 0usize;
    for (team, storage) in &monasteries {
        if team.0 == 0 {
            total_relics += storage.relics.len();
        }
    }
    if total_relics == 0 {
        return;
    }
    *accumulated += total_relics as f32 * 0.5 * time.delta_secs();
    let gold = *accumulated as u32;
    if gold > 0 {
        resources.gold += gold;
        *accumulated -= gold as f32;
    }
}

pub fn relic_drop_on_death_system(
    mut commands: Commands,
    dead_monks: Query<(Entity, &Transform, &Health, &RelicCarrier), (With<MonkUnit>, With<Unit>)>,
    mut relics: Query<(&mut Transform, &mut Visibility), (With<Relic>, Without<Unit>)>,
) {
    for (_monk_e, monk_tf, health, carrier) in &dead_monks {
        if health.current > 0.0 {
            continue;
        }
        if let Ok((mut relic_tf, mut vis)) = relics.get_mut(carrier.0) {
            relic_tf.translation = monk_tf.translation;
            *vis = Visibility::Inherited;
        }
    }
}
