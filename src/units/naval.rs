use bevy::prelude::*;
use super::components::*;
use crate::map::TILE_SIZE;
use crate::resources::components::{ResourceNode, ResourceKind, PlayerResources, Carrying};
use crate::buildings::components::{Building, BuildingKind};

#[derive(Component)]
pub struct FishingTarget(pub Entity);

pub fn fishing_ship_system(
    mut commands: Commands,
    mut ships: Query<(Entity, &Transform, &Team, &mut UnitState, Option<&mut Carrying>), (With<NavalUnit>, With<Unit>)>,
    fish: Query<(Entity, &Transform, &ResourceNode)>,
    docks: Query<(Entity, &Transform, &Team, &Building)>,
    mut resources: ResMut<PlayerResources>,
    fishing_targets: Query<&FishingTarget>,
) {
    for (ship_e, ship_tf, team, mut state, mut carrying) in &mut ships {
        if team.0 != 0 { continue; }

        let is_full = carrying.as_ref().map_or(false, |c| c.is_full());
        if is_full {
            if let Some((_dock_e, dock_tf, _, _)) = docks.iter()
                .filter(|(_, _, t, b)| t.0 == team.0 && b.kind == BuildingKind::Dock)
                .min_by(|(_, a, _, _), (_, b, _, _)| {
                    let da = a.translation.truncate().distance_squared(ship_tf.translation.truncate());
                    let db = b.translation.truncate().distance_squared(ship_tf.translation.truncate());
                    da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
                })
            {
                let dock_pos = dock_tf.translation.truncate();
                let dist = ship_tf.translation.truncate().distance(dock_pos);
                if dist < TILE_SIZE * 3.0 {
                    if let Some(ref mut c) = carrying {
                        let (kind, amount) = c.take_all();
                        resources.add(kind, amount);
                    }
                    *state = UnitState::Idle;
                    commands.entity(ship_e).remove::<MoveTarget>();
                } else {
                    commands.entity(ship_e)
                        .insert(MoveTarget(dock_pos))
                        .insert(UnitState::Moving);
                }
            }
            continue;
        }

        if let Ok(target) = fishing_targets.get(ship_e) {
            let Ok((_, fish_tf, fish_node)) = fish.get(target.0) else {
                commands.entity(ship_e).remove::<FishingTarget>();
                *state = UnitState::Idle;
                continue;
            };

            if fish_node.remaining == 0 {
                commands.entity(ship_e).remove::<FishingTarget>();
                *state = UnitState::Idle;
                continue;
            }

            let dist = ship_tf.translation.truncate().distance(fish_tf.translation.truncate());
            if dist < TILE_SIZE * 2.0 {
                if let Some(ref mut c) = carrying {
                    if !c.is_full() {
                        c.kind = Some(ResourceKind::Food);
                        c.amount += 1;
                    }
                }
            }
            continue;
        }

        if *state == UnitState::Idle {
            let ship_pos = ship_tf.translation.truncate();
            if let Some((fish_e, fish_tf, _)) = fish.iter()
                .filter(|(_, _, n)| n.kind == ResourceKind::Food && n.remaining > 0)
                .min_by(|(_, a, _), (_, b, _)| {
                    let da = a.translation.truncate().distance_squared(ship_pos);
                    let db = b.translation.truncate().distance_squared(ship_pos);
                    da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
                })
            {
                commands.entity(ship_e)
                    .insert(FishingTarget(fish_e))
                    .insert(MoveTarget(fish_tf.translation.truncate()))
                    .insert(UnitState::Moving);
            }
        }
    }
}
