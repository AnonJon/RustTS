use bevy::prelude::*;
use super::components::*;
use super::pathfinding::Path;

const ARRIVAL_THRESHOLD: f32 = 8.0;

pub fn movement_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &Speed, &MoveTarget, &mut UnitState), (With<Unit>, Without<Path>)>,
    time: Res<Time>,
) {
    for (entity, mut transform, speed, target, mut state) in &mut query {
        let current = transform.translation.truncate();
        let direction = target.0 - current;
        let distance = direction.length();

        if distance < ARRIVAL_THRESHOLD {
            commands.entity(entity).remove::<MoveTarget>();
            if *state == UnitState::Moving {
                *state = UnitState::Idle;
            }
            continue;
        }

        let velocity = direction.normalize() * speed.0 * crate::map::TILE_SIZE * time.delta_secs();
        if velocity.length() > distance {
            transform.translation.x = target.0.x;
            transform.translation.y = target.0.y;
        } else {
            transform.translation.x += velocity.x;
            transform.translation.y += velocity.y;
        }
    }
}
