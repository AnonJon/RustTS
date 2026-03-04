use bevy::prelude::*;
use crate::map::{GridPosition, TILE_SIZE};
use crate::units::components::*;
use crate::units::types::{UnitKind, UnitSprites, spawn_unit};
use crate::resources::components::{PlayerResources, DropOff, ResourceNode, ResourceKind};
use super::components::*;

const BUILD_RANGE: f32 = TILE_SIZE * 2.0;

pub fn construction_system(
    mut commands: Commands,
    time: Res<Time>,
    constructors: Query<
        (Entity, &Transform, &UnitState, &ConstructTarget),
        With<Unit>,
    >,
    mut buildings: Query<
        (Entity, &Transform, &mut UnderConstruction, &mut Health, &mut Sprite, &Building, &Team),
        Without<Unit>,
    >,
    resource_nodes: Query<(Entity, &Transform, &ResourceNode), (Without<Unit>, Without<Building>)>,
    mut stats: ResMut<crate::ui::stats::GameStats>,
) {
    let dt = time.delta_secs();
    if dt == 0.0 {
        return;
    }

    let mut progress_per_building: std::collections::HashMap<Entity, (f32, Vec<Entity>)> =
        std::collections::HashMap::new();

    for (villager_e, villager_tf, state, construct_target) in &constructors {
        let UnitState::Constructing { building } = state else {
            continue;
        };
        if *building != construct_target.0 {
            continue;
        }

        let Ok((_, bld_tf, ref uc, _, _, _, _)) = buildings.get(construct_target.0) else {
            commands.entity(villager_e)
                .remove::<ConstructTarget>()
                .insert(UnitState::Idle);
            continue;
        };

        let dist = villager_tf.translation.truncate()
            .distance(bld_tf.translation.truncate());

        if dist > BUILD_RANGE {
            continue;
        }

        let tick = dt / uc.build_time;
        let entry = progress_per_building
            .entry(construct_target.0)
            .or_insert((0.0, Vec::new()));
        entry.0 += tick;
        entry.1.push(villager_e);
    }

    for (building_e, total_tick, builders) in progress_per_building
        .into_iter()
        .map(|(e, (t, b))| (e, t, b))
    {
        let Ok((_, bld_tf, mut uc, mut health, mut sprite, building, bld_team)) =
            buildings.get_mut(building_e) else { continue };

        uc.progress = (uc.progress + total_tick).min(1.0);

        sprite.color = Color::srgba(1.0, 1.0, 1.0, 0.3 + 0.7 * uc.progress);
        health.current = building.kind.max_hp() * uc.progress.max(0.1);

        if uc.progress < 1.0 {
            continue;
        }

        if bld_team.0 == 0 {
            stats.buildings_built += 1;
        }

        let bld_pos = bld_tf.translation.truncate();
        let bld_kind = building.kind;

        commands.entity(building_e).remove::<UnderConstruction>();
        sprite.color = Color::srgba(1.0, 1.0, 1.0, 1.0);
        health.current = bld_kind.max_hp();

        if !bld_kind.can_train().is_empty() {
            commands.entity(building_e).insert(TrainingQueue {
                queue: Vec::new(),
            });
        }

        if bld_kind == BuildingKind::WatchTower {
            commands.entity(building_e).insert(TowerAttack::watch_tower());
        }

        if bld_kind == BuildingKind::Castle {
            commands.entity(building_e).insert(TowerAttack {
                range: 10.0,
                pierce_damage: 11.0,
                cooldown: Timer::from_seconds(1.5, TimerMode::Repeating),
            });
        }

        let garrison_cap = match bld_kind {
            BuildingKind::TownCenter => Some(15),
            BuildingKind::WatchTower => Some(5),
            BuildingKind::Castle => Some(20),
            _ => None,
        };
        if let Some(cap) = garrison_cap {
            commands.entity(building_e).insert(GarrisonSlots::new(cap));
        }

        let drop_off = match bld_kind {
            BuildingKind::TownCenter => Some(DropOff::all()),
            BuildingKind::LumberCamp => Some(DropOff::wood()),
            BuildingKind::MiningCamp => Some(DropOff::mining()),
            BuildingKind::Mill | BuildingKind::Farm => Some(DropOff::food()),
            _ => None,
        };
        if let Some(d) = drop_off {
            commands.entity(building_e).insert(d);
        }

        if bld_kind == BuildingKind::Farm {
            commands.entity(building_e).insert(crate::resources::components::FarmFood::new());
        }

        if matches!(bld_kind, BuildingKind::Mill | BuildingKind::TownCenter) {
            commands.entity(building_e).insert(crate::resources::components::AutoReseed(true));
        }

        let is_research_bld = matches!(bld_kind,
            BuildingKind::Blacksmith | BuildingKind::University
            | BuildingKind::LumberCamp | BuildingKind::MiningCamp | BuildingKind::Mill
        );
        if is_research_bld {
            commands.entity(building_e).insert(super::research::ResearchQueue { queue: Vec::new() });
        }

        if bld_kind == BuildingKind::Monastery {
            commands.entity(building_e).insert(super::components::RelicStorage::new());
        }

        let resource_kind = match bld_kind {
            BuildingKind::LumberCamp => Some(ResourceKind::Wood),
            BuildingKind::MiningCamp => Some(ResourceKind::Gold),
            BuildingKind::Mill => Some(ResourceKind::Food),
            BuildingKind::Farm => Some(ResourceKind::Food),
            _ => None,
        };

        for villager_e in builders {
            commands.entity(villager_e).remove::<ConstructTarget>();

            if let Some(rk) = resource_kind {
                if let Some((res_e, res_tf, _)) = resource_nodes
                    .iter()
                    .filter(|(_, _, rn)| rn.kind == rk && rn.remaining > 0)
                    .min_by(|(_, a_tf, _), (_, b_tf, _)| {
                        let da = a_tf.translation.truncate().distance_squared(bld_pos);
                        let db = b_tf.translation.truncate().distance_squared(bld_pos);
                        da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
                    })
                {
                    commands.entity(villager_e).insert((
                        UnitState::Gathering { resource: res_e },
                        MoveTarget(res_tf.translation.truncate()),
                    ));
                } else {
                    commands.entity(villager_e).insert(UnitState::Idle);
                }
            } else {
                commands.entity(villager_e).insert(UnitState::Idle);
            }
        }
    }
}


pub fn training_system(
    mut commands: Commands,
    mut buildings: Query<(Entity, &Building, &mut TrainingQueue, &Transform, &Team)>,
    _player_resources: ResMut<PlayerResources>,
    sprites: Res<UnitSprites>,
    time: Res<Time>,
    mut stats: ResMut<crate::ui::stats::GameStats>,
    player_civ: Res<crate::civilization::PlayerCivilization>,
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

            let unit_e = spawn_unit(&mut commands, sprites.get(kind), kind, *team, grid, spawn_pos);
            if team.0 == 0 {
                stats.units_created += 1;
                commands.entity(unit_e).insert(NeedsCivBonus);
            }
            commands.entity(unit_e)
                .insert(MoveTarget(rally))
                .insert(UnitState::Moving);

            if kind == UnitKind::TradeCart {
                commands.entity(unit_e).insert(crate::resources::market::TradeRoute {
                    home_market: _entity,
                    target_market: None,
                    going_to_target: false,
                    gold_earned: 0,
                });
            }
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
    query: Query<(Entity, &Health, &Team), With<Building>>,
    mut stats: ResMut<crate::ui::stats::GameStats>,
) {
    for (entity, health, team) in &query {
        if health.current <= 0.0 {
            if team.0 == 0 {
                stats.buildings_lost += 1;
            } else {
                stats.enemy_buildings_destroyed += 1;
            }
            commands.entity(entity).despawn();
        }
    }
}

pub fn tower_attack_system(
    mut towers: Query<(&Transform, &Team, &mut TowerAttack), (With<Building>, Without<UnderConstruction>)>,
    mut targets: Query<(Entity, &Transform, &Team, &mut Health, &Armor), With<Unit>>,
    time: Res<Time>,
) {
    let mut damage_events: Vec<(Entity, f32)> = Vec::new();

    for (tower_tf, tower_team, mut attack) in &mut towers {
        attack.cooldown.tick(time.delta());
        if !attack.cooldown.just_finished() {
            continue;
        }

        let tower_pos = tower_tf.translation.truncate();
        let range_px = attack.range * TILE_SIZE;
        let mut best_target: Option<(Entity, f32)> = None;

        for (entity, target_tf, target_team, health, _) in &targets {
            if target_team.0 == tower_team.0 || health.current <= 0.0 {
                continue;
            }
            let dist = tower_pos.distance(target_tf.translation.truncate());
            if dist < range_px {
                if best_target.is_none() || dist < best_target.unwrap().1 {
                    best_target = Some((entity, dist));
                }
            }
        }

        if let Some((target_entity, _)) = best_target {
            if let Ok((_, _, _, _, armor)) = targets.get(target_entity) {
                let pierce = (attack.pierce_damage - armor.pierce).max(0.0);
                let dmg = pierce.max(1.0);
                damage_events.push((target_entity, dmg));
            }
        }
    }

    for (entity, dmg) in damage_events {
        if let Ok((_, _, _, mut health, _)) = targets.get_mut(entity) {
            health.current -= dmg;
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
    resource_nodes: Query<&Transform, With<crate::resources::components::ResourceNode>>,
    units: Query<&Transform, With<Unit>>,
) {
    if !mouse.just_released(MouseButton::Left) {
        return;
    }

    let Ok(window) = windows.single() else { return };
    let Ok((camera, cam_transform)) = camera_q.single() else { return };
    let Some(cursor_pos) = window.cursor_position() else { return };
    let Ok(world_pos) = camera.viewport_to_world_2d(cam_transform, cursor_pos) else { return };

    let resource_nearby = resource_nodes.iter().any(|tf| {
        tf.translation.truncate().distance(world_pos) < 40.0
    });
    let unit_nearby = units.iter().any(|tf| {
        tf.translation.truncate().distance(world_pos) < 50.0
    });
    if resource_nearby || unit_nearby {
        return;
    }

    for (entity, transform, building, _team) in &buildings {
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
    player_civ: Res<crate::civilization::PlayerCivilization>,
) {
    for (building, mut queue, team) in &mut buildings_selected {
        if team.0 != 0 { continue; }

        let all_trainable = building.kind.can_train();
        let trainable: Vec<&crate::units::types::UnitKind> = if building.kind == BuildingKind::Castle {
            let uu = player_civ.0.unique_unit();
            all_trainable.iter().filter(|&&k| k == uu).collect()
        } else {
            all_trainable.iter().collect()
        };

        if keys.just_pressed(KeyCode::KeyQ) {
            if let Some(&&kind) = trainable.first() {
                enqueue_unit(&mut commands, Entity::PLACEHOLDER, &mut queue, kind, &mut resources, &age);
            }
        }
        if keys.just_pressed(KeyCode::KeyW) {
            if let Some(&&kind) = trainable.get(1) {
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

pub fn garrison_command_system(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform), With<crate::camera::MainCamera>>,
    selected_units: Query<(Entity, &Team), (With<Unit>, With<Selected>)>,
    mut garrison_buildings: Query<(Entity, &Transform, &Team, &mut GarrisonSlots), With<Building>>,
) {
    if !mouse.just_pressed(MouseButton::Right) {
        return;
    }

    let Ok(window) = windows.single() else { return };
    let Ok((camera, cam_transform)) = camera_q.single() else { return };
    let Some(cursor_pos) = window.cursor_position() else { return };
    let Ok(world_pos) = camera.viewport_to_world_2d(cam_transform, cursor_pos) else { return };

    for (bld_entity, bld_tf, bld_team, mut slots) in &mut garrison_buildings {
        let dist = bld_tf.translation.truncate().distance(world_pos);
        if dist > 80.0 {
            continue;
        }

        for (unit_entity, unit_team) in &selected_units {
            if unit_team.0 != bld_team.0 {
                continue;
            }
            if !slots.has_space() {
                break;
            }
            slots.units.push(unit_entity);
            commands.entity(unit_entity)
                .remove::<Selected>()
                .insert(Visibility::Hidden)
                .insert(UnitState::Idle);
            commands.entity(unit_entity).remove::<MoveTarget>();
            commands.entity(unit_entity).remove::<AttackTarget>();
        }
        return;
    }
}

pub fn ungarrison_system(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    mut buildings: Query<(&Transform, &Team, &mut GarrisonSlots), (With<Building>, With<Selected>)>,
) {
    if !keys.just_pressed(KeyCode::KeyG) {
        return;
    }

    for (bld_tf, team, mut slots) in &mut buildings {
        if team.0 != 0 {
            continue;
        }
        let bld_pos = bld_tf.translation.truncate();
        for (i, unit_entity) in slots.units.drain(..).enumerate() {
            let offset = Vec2::new(
                ((i % 4) as f32 - 1.5) * 30.0,
                -((i / 4) as f32 + 1.0) * 30.0,
            );
            commands.entity(unit_entity)
                .insert(Visibility::Visible)
                .insert(Transform::from_xyz(
                    bld_pos.x + offset.x,
                    bld_pos.y + offset.y,
                    10.0,
                ))
                .insert(UnitState::Idle);
        }
    }
}

pub fn garrison_eject_on_death_system(
    mut commands: Commands,
    query: Query<(&Transform, &Health, &GarrisonSlots), With<Building>>,
) {
    for (bld_tf, health, slots) in &query {
        if health.current > 0.0 || slots.units.is_empty() {
            continue;
        }
        let bld_pos = bld_tf.translation.truncate();
        for (i, &unit_entity) in slots.units.iter().enumerate() {
            let offset = Vec2::new(
                ((i % 4) as f32 - 1.5) * 30.0,
                -((i / 4) as f32 + 1.0) * 30.0,
            );
            commands.entity(unit_entity)
                .insert(Visibility::Visible)
                .insert(Transform::from_xyz(
                    bld_pos.x + offset.x,
                    bld_pos.y + offset.y,
                    10.0,
                ))
                .insert(UnitState::Idle);
        }
    }
}

pub fn garrison_arrow_bonus_system(
    mut garrison_buildings: Query<(&GarrisonSlots, &mut TowerAttack), With<Building>>,
    archers: Query<(&AttackStats, &UnitClass), With<Unit>>,
) {
    for (slots, mut tower_attack) in &mut garrison_buildings {
        let mut bonus_arrows = 0.0f32;
        for &unit_e in &slots.units {
            if let Ok((stats, &class)) = archers.get(unit_e) {
                if class == UnitClass::Archer {
                    bonus_arrows += stats.pierce_damage * 0.5;
                }
            }
        }
        tower_attack.pierce_damage = 5.0 + bonus_arrows;
    }
}
