use bevy::prelude::*;
use super::components::*;
use crate::units::components::*;
use crate::buildings::components::Building;
use crate::resources::components::ResourceNode;
use crate::map::TILE_SIZE;

const GATHER_RANGE: f32 = 0.6;
const GATHER_RATE: f32 = 1.0;
const DROP_OFF_RANGE: f32 = 1.5;
const FARM_RATE: f32 = 2.0;

#[derive(Component)]
pub struct GatherTimer(pub Timer);

pub fn gathering_system(
    mut commands: Commands,
    mut gatherers: Query<
        (Entity, &Transform, &mut UnitState, &Team, Option<&mut GatherTimer>, Option<&mut Carrying>),
        With<Unit>,
    >,
    mut resource_nodes: Query<(Entity, &Transform, &mut ResourceNode)>,
    drop_offs: Query<(Entity, &Transform, &DropOff, &Team), Without<Unit>>,
    time: Res<Time>,
) {
    for (unit_entity, unit_tf, mut state, team, gather_timer, carrying) in &mut gatherers {
        let resource_entity = match &*state {
            UnitState::Gathering { resource } => *resource,
            _ => continue,
        };

        let Ok((_, res_tf, mut node)) = resource_nodes.get_mut(resource_entity) else {
            *state = UnitState::Idle;
            commands.entity(unit_entity).remove::<GatherTimer>();
            continue;
        };

        let distance = unit_tf.translation.truncate()
            .distance(res_tf.translation.truncate());

        if distance > GATHER_RANGE * TILE_SIZE {
            continue;
        }

        if let Some(ref carrying) = carrying {
            if carrying.is_full() {
                let kind = carrying.kind.unwrap_or(node.kind);
                if let Some((drop_entity, _)) = find_nearest_drop_off(
                    unit_tf.translation.truncate(),
                    kind,
                    team.0,
                    &drop_offs,
                ) {
                    commands.entity(unit_entity)
                        .insert(MoveTarget(
                            drop_offs.get(drop_entity).unwrap().1.translation.truncate(),
                        ));
                    *state = UnitState::Returning {
                        drop_off: drop_entity,
                        then_gather: Some(resource_entity),
                    };
                    commands.entity(unit_entity).remove::<GatherTimer>();
                }
                continue;
            }
        }

        if let Some(mut timer) = gather_timer {
            timer.0.tick(time.delta());
            if timer.0.just_finished() {
                let room = if let Some(ref c) = carrying {
                    c.max_carry.saturating_sub(c.amount).min(5)
                } else {
                    5
                };
                let gathered = room.min(node.remaining);
                if gathered > 0 {
                    let res_pos = res_tf.translation.truncate();
                    let kind = node.kind;
                    node.remaining -= gathered;
                    if let Some(mut c) = carrying {
                        c.kind = Some(kind);
                        c.amount += gathered;
                    }
                    spawn_gather_text(&mut commands, res_pos, gathered, kind);
                }
            }
        } else {
            commands.entity(unit_entity).insert(
                GatherTimer(Timer::from_seconds(GATHER_RATE, TimerMode::Repeating))
            );
        }
    }
}

pub fn returning_system(
    mut commands: Commands,
    mut returners: Query<
        (Entity, &Transform, &mut UnitState, &Team, Option<&mut Carrying>),
        With<Unit>,
    >,
    drop_offs: Query<(Entity, &Transform, &DropOff, &Team), Without<Unit>>,
    resource_nodes: Query<&Transform, With<ResourceNode>>,
    farm_buildings: Query<(&Transform, &Building), Without<ResourceNode>>,
    mut player_resources: ResMut<PlayerResources>,
    mut game_stats: ResMut<crate::ui::stats::GameStats>,
) {
    for (unit_entity, unit_tf, mut state, team, carrying) in &mut returners {
        let (drop_off_entity, then_gather) = match &*state {
            UnitState::Returning { drop_off, then_gather } => (*drop_off, *then_gather),
            _ => continue,
        };

        let Ok((_, drop_tf, _, _)) = drop_offs.get(drop_off_entity) else {
            *state = UnitState::Idle;
            continue;
        };

        let current = unit_tf.translation.truncate();
        let target = drop_tf.translation.truncate();
        let distance = current.distance(target);

        if distance > DROP_OFF_RANGE * TILE_SIZE {
            continue;
        }

        if let Some(mut c) = carrying {
            if c.has_resources() && team.0 == 0 {
                let (kind, amount) = c.take_all();
                match kind {
                    ResourceKind::Food => game_stats.food_gathered += amount,
                    ResourceKind::Wood => game_stats.wood_gathered += amount,
                    ResourceKind::Gold => game_stats.gold_gathered += amount,
                    ResourceKind::Stone => game_stats.stone_gathered += amount,
                }
                player_resources.add(kind, amount);
            } else {
                c.amount = 0;
                c.kind = None;
            }
        }

        if let Some(gather_target) = then_gather {
            if let Ok(res_tf) = resource_nodes.get(gather_target) {
                commands.entity(unit_entity)
                    .insert(MoveTarget(res_tf.translation.truncate()));
                *state = UnitState::Gathering { resource: gather_target };
            } else if let Ok((farm_tf, _)) = farm_buildings.get(gather_target) {
                commands.entity(unit_entity)
                    .insert(MoveTarget(farm_tf.translation.truncate()));
                *state = UnitState::FarmingAt { farm: gather_target };
            } else {
                *state = UnitState::Idle;
            }
        } else {
            *state = UnitState::Idle;
        }
    }
}

pub fn farm_system(
    mut commands: Commands,
    mut farmers: Query<
        (Entity, &Transform, &mut UnitState, &Team, Option<&mut GatherTimer>, Option<&mut Carrying>),
        With<Unit>,
    >,
    mut farms: Query<(Entity, &Transform, &Building, Option<&mut FarmFood>), Without<Unit>>,
    drop_offs: Query<(Entity, &Transform, &DropOff, &Team), Without<Unit>>,
    time: Res<Time>,
) {
    for (unit_entity, unit_tf, mut state, team, gather_timer, carrying) in &mut farmers {
        let farm_entity = match &*state {
            UnitState::FarmingAt { farm } => *farm,
            _ => continue,
        };

        let Ok((_, farm_tf, _, farm_food)) = farms.get_mut(farm_entity) else {
            *state = UnitState::Idle;
            commands.entity(unit_entity).remove::<GatherTimer>();
            continue;
        };

        if let Some(ref ff) = farm_food {
            if ff.remaining == 0 {
                *state = UnitState::Idle;
                commands.entity(unit_entity).remove::<GatherTimer>();
                continue;
            }
        }

        let distance = unit_tf.translation.truncate()
            .distance(farm_tf.translation.truncate());

        if distance > GATHER_RANGE * TILE_SIZE {
            continue;
        }

        if let Some(ref c) = carrying {
            if c.is_full() {
                if let Some((drop_entity, _)) = find_nearest_drop_off(
                    unit_tf.translation.truncate(),
                    ResourceKind::Food,
                    team.0,
                    &drop_offs,
                ) {
                    commands.entity(unit_entity)
                        .insert(MoveTarget(
                            drop_offs.get(drop_entity).unwrap().1.translation.truncate(),
                        ))
                        .remove::<GatherTimer>();
                    *state = UnitState::Returning {
                        drop_off: drop_entity,
                        then_gather: Some(farm_entity),
                    };
                }
                continue;
            }
        }

        if let Some(mut timer) = gather_timer {
            timer.0.tick(time.delta());
            if timer.0.just_finished() {
                if let Some(mut c) = carrying {
                    c.kind = Some(ResourceKind::Food);
                    c.amount += 1;
                }
                if let Some(mut ff) = farm_food {
                    ff.remaining = ff.remaining.saturating_sub(1);
                }
            }
        } else {
            commands.entity(unit_entity).insert(
                GatherTimer(Timer::from_seconds(FARM_RATE, TimerMode::Repeating))
            );
        }
    }
}

pub fn resource_visual_system(
    mut resources: Query<(&ResourceNode, &mut Sprite, &mut Transform), Without<Unit>>,
) {
    for (node, mut sprite, mut transform) in &mut resources {
        let fraction = node.remaining as f32 / node.max_amount.max(1) as f32;
        let scale = 0.4 + 0.6 * fraction;
        transform.scale = Vec3::splat(scale);

        let alpha = 0.3 + 0.7 * fraction;
        sprite.color = Color::srgba(1.0, 1.0, 1.0, alpha);
    }
}

pub fn resource_depletion_system(
    mut commands: Commands,
    query: Query<(Entity, &ResourceNode)>,
) {
    for (entity, node) in &query {
        if node.remaining == 0 {
            commands.entity(entity).despawn();
        }
    }
}

pub fn gather_move_recovery_system(
    mut commands: Commands,
    stuck: Query<
        (Entity, &Transform, &UnitState),
        (With<Unit>, Without<MoveTarget>, Without<crate::units::pathfinding::Path>),
    >,
    resource_nodes: Query<&Transform, With<ResourceNode>>,
    farms: Query<(&Transform, &Building), Without<ResourceNode>>,
) {
    for (entity, unit_tf, state) in &stuck {
        match state {
            UnitState::Gathering { resource } => {
                if let Ok(res_tf) = resource_nodes.get(*resource) {
                    let dist = unit_tf.translation.truncate()
                        .distance(res_tf.translation.truncate());
                    if dist > GATHER_RANGE * TILE_SIZE {
                        commands.entity(entity)
                            .insert(MoveTarget(res_tf.translation.truncate()));
                    }
                }
            }
            UnitState::FarmingAt { farm } => {
                if let Ok((farm_tf, _)) = farms.get(*farm) {
                    let dist = unit_tf.translation.truncate()
                        .distance(farm_tf.translation.truncate());
                    if dist > GATHER_RANGE * TILE_SIZE {
                        commands.entity(entity)
                            .insert(MoveTarget(farm_tf.translation.truncate()));
                    }
                }
            }
            _ => {}
        }
    }
}

pub fn floating_text_system(
    mut commands: Commands,
    mut texts: Query<(Entity, &mut Transform, &mut FloatingText, &mut TextColor)>,
    time: Res<Time>,
) {
    for (entity, mut transform, mut ft, mut color) in &mut texts {
        ft.lifetime.tick(time.delta());
        if ft.lifetime.just_finished() || ft.lifetime.fraction() >= 1.0 {
            commands.entity(entity).despawn();
            continue;
        }

        transform.translation.x += ft.velocity.x * time.delta_secs();
        transform.translation.y += ft.velocity.y * time.delta_secs();

        let alpha = 1.0 - ft.lifetime.fraction();
        *color = TextColor(Color::srgba(1.0, 1.0, 1.0, alpha));
    }
}

pub fn spawn_gather_text(
    commands: &mut Commands,
    pos: Vec2,
    amount: u32,
    kind: ResourceKind,
) {
    let label = match kind {
        ResourceKind::Food => format!("+{amount} Food"),
        ResourceKind::Wood => format!("+{amount} Wood"),
        ResourceKind::Gold => format!("+{amount} Gold"),
        ResourceKind::Stone => format!("+{amount} Stone"),
    };
    let text_color = match kind {
        ResourceKind::Food => Color::srgb(1.0, 0.4, 0.4),
        ResourceKind::Wood => Color::srgb(0.4, 1.0, 0.4),
        ResourceKind::Gold => Color::srgb(1.0, 0.9, 0.2),
        ResourceKind::Stone => Color::srgb(0.8, 0.8, 0.8),
    };
    commands.spawn((
        FloatingText {
            lifetime: Timer::from_seconds(1.2, TimerMode::Once),
            velocity: Vec2::new(0.0, 60.0),
        },
        Text2d::new(label),
        TextFont { font_size: 16.0, ..default() },
        TextColor(text_color),
        Transform::from_xyz(pos.x, pos.y + 50.0, 100.0),
    ));
}

pub fn auto_reseek_system(
    mut commands: Commands,
    idle_villagers: Query<
        (Entity, &Transform, &Team, &UnitState, Option<&Carrying>),
        With<Unit>,
    >,
    drop_offs: Query<(Entity, &Transform, &DropOff, &Team), Without<Unit>>,
    mut player_resources: ResMut<PlayerResources>,
) {
    for (unit_entity, unit_tf, team, state, carrying) in &idle_villagers {
        if team.0 != 0 { continue; }
        if !matches!(state, UnitState::Idle) { continue; }

        let Some(c) = carrying else { continue; };
        if !c.has_resources() { continue; }

        let kind = c.kind.unwrap_or(ResourceKind::Food);
        if let Some((drop_entity, dist)) = find_nearest_drop_off(
            unit_tf.translation.truncate(),
            kind,
            team.0,
            &drop_offs,
        ) {
            if dist < TILE_SIZE * 3.0 {
                player_resources.add(kind, c.amount);
                commands.entity(unit_entity)
                    .insert(Carrying::default());
            } else {
                commands.entity(unit_entity)
                    .insert(MoveTarget(
                        drop_offs.get(drop_entity).unwrap().1.translation.truncate(),
                    ))
                    .insert(UnitState::Returning {
                        drop_off: drop_entity,
                        then_gather: None,
                    });
            }
        }
    }
}

fn find_nearest_drop_off(
    pos: Vec2,
    kind: ResourceKind,
    team_id: u8,
    drop_offs: &Query<(Entity, &Transform, &DropOff, &Team), Without<Unit>>,
) -> Option<(Entity, f32)> {
    let mut nearest: Option<(Entity, f32)> = None;
    for (entity, transform, drop_off, team) in drop_offs.iter() {
        if team.0 != team_id { continue; }
        if !drop_off.accepts(kind) { continue; }
        let dist = pos.distance(transform.translation.truncate());
        if nearest.is_none() || dist < nearest.unwrap().1 {
            nearest = Some((entity, dist));
        }
    }
    nearest
}

pub fn farm_auto_reseed_system(
    mut commands: Commands,
    depleted_farms: Query<(Entity, &Transform, &Team, &FarmFood, &Building), Without<Unit>>,
    reseeders: Query<(&Transform, &Team, &AutoReseed), (With<Building>, Without<FarmFood>)>,
    mut resources: ResMut<PlayerResources>,
    mut images: ResMut<Assets<bevy::prelude::Image>>,
) {
    for (farm_entity, farm_tf, farm_team, farm_food, _building) in &depleted_farms {
        if farm_food.remaining > 0 {
            continue;
        }
        if farm_team.0 != 0 {
            continue;
        }

        let farm_pos = farm_tf.translation.truncate();
        let has_reseeder = reseeders.iter().any(|(bld_tf, bld_team, auto)| {
            bld_team.0 == farm_team.0
                && auto.0
                && bld_tf.translation.truncate().distance(farm_pos) < TILE_SIZE * 12.0
        });

        if !has_reseeder {
            continue;
        }

        let (f, w, g, s) = crate::buildings::components::BuildingKind::Farm.build_cost();
        if !resources.spend(f, w, g, s) {
            continue;
        }

        commands.entity(farm_entity).insert(FarmFood::new());
        commands.entity(farm_entity).insert(Health::new(
            crate::buildings::components::BuildingKind::Farm.max_hp(),
        ));
    }
}
