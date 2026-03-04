use bevy::prelude::*;
use super::components::*;
use super::pathfinding::Path;
use crate::map::{GridPosition, MAP_WIDTH, MAP_HEIGHT};
use crate::map::generation::MapConfig;

const ARRIVAL_THRESHOLD: f32 = 8.0;

/// Fallback movement for units with a MoveTarget but no computed Path.
/// Only moves if every step stays on walkable terrain; otherwise waits
/// for the pathfinding system to compute a proper route.
pub fn movement_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &Speed, &MoveTarget, &mut UnitState), (With<Unit>, Without<Path>)>,
    time: Res<Time>,
    config: Res<MapConfig>,
    building_occupancy: Res<crate::buildings::BuildingOccupancy>,
    naval_units: Query<Entity, With<NavalUnit>>,
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
        let new_pos = if velocity.length() > distance {
            target.0
        } else {
            Vec2::new(
                transform.translation.x + velocity.x,
                transform.translation.y + velocity.y,
            )
        };

        let new_grid = GridPosition::from_world(new_pos);
        let in_bounds = new_grid.x >= 0
            && new_grid.x < MAP_WIDTH as i32
            && new_grid.y >= 0
            && new_grid.y < MAP_HEIGHT as i32;

        let is_naval = naval_units.contains(entity);
        let terrain_ok = if is_naval {
            use crate::map::terrain::TerrainType;
            in_bounds && config.terrain_grid[new_grid.x as usize][new_grid.y as usize].terrain == TerrainType::Water
        } else {
            in_bounds
                && config.terrain_grid[new_grid.x as usize][new_grid.y as usize].is_walkable()
                && !building_occupancy.0.contains(&(new_grid.x, new_grid.y))
        };

        if terrain_ok {
            transform.translation.x = new_pos.x;
            transform.translation.y = new_pos.y;
        }
    }
}
