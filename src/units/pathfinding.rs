use bevy::prelude::*;
use crate::map::{GridPosition, TILE_SIZE, MAP_WIDTH, MAP_HEIGHT};
use crate::map::generation::{MapConfig, Tile};
use std::collections::{BinaryHeap, HashMap};
use std::cmp::Ordering;

#[derive(Component)]
pub struct Path {
    pub waypoints: Vec<Vec2>,
    pub current_index: usize,
}

#[derive(Clone, Eq, PartialEq)]
struct Node {
    pos: (i32, i32),
    cost: u32,
    heuristic: u32,
}

impl Node {
    fn priority(&self) -> u32 {
        self.cost + self.heuristic
    }
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        other.priority().cmp(&self.priority())
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

const NEIGHBORS: [(i32, i32); 8] = [
    (0, 1), (0, -1), (1, 0), (-1, 0),
    (1, 1), (1, -1), (-1, 1), (-1, -1),
];

pub fn find_path(
    start: GridPosition,
    goal: GridPosition,
    occupied: &HashMap<(i32, i32), Entity>,
    requesting_entity: Entity,
    terrain_grid: &[Vec<Tile>],
) -> Option<Vec<Vec2>> {
    let start_pos = (start.x, start.y);
    let goal_pos = (goal.x, goal.y);

    if start_pos == goal_pos {
        return Some(vec![goal.to_world()]);
    }

    let mut open = BinaryHeap::new();
    let mut came_from: HashMap<(i32, i32), (i32, i32)> = HashMap::new();
    let mut g_score: HashMap<(i32, i32), u32> = HashMap::new();

    g_score.insert(start_pos, 0);
    open.push(Node {
        pos: start_pos,
        cost: 0,
        heuristic: heuristic(start_pos, goal_pos),
    });

    let max_iterations = 1000;
    let mut iterations = 0;

    while let Some(current) = open.pop() {
        iterations += 1;
        if iterations > max_iterations {
            break;
        }

        if current.pos == goal_pos {
            return Some(reconstruct_path(&came_from, goal_pos));
        }

        let current_g = g_score[&current.pos];

        for &(dx, dy) in &NEIGHBORS {
            let next = (current.pos.0 + dx, current.pos.1 + dy);

            if next.0 < 0 || next.0 >= MAP_WIDTH as i32
                || next.1 < 0 || next.1 >= MAP_HEIGHT as i32
            {
                continue;
            }

            if !terrain_grid[next.0 as usize][next.1 as usize].is_walkable() {
                continue;
            }

            if let Some(&occupant) = occupied.get(&next) {
                if occupant != requesting_entity && next != goal_pos {
                    continue;
                }
            }

            let move_cost = if dx != 0 && dy != 0 { 14 } else { 10 };
            let tentative_g = current_g + move_cost;

            if tentative_g < *g_score.get(&next).unwrap_or(&u32::MAX) {
                g_score.insert(next, tentative_g);
                came_from.insert(next, current.pos);
                open.push(Node {
                    pos: next,
                    cost: tentative_g,
                    heuristic: heuristic(next, goal_pos),
                });
            }
        }
    }

    None
}

fn heuristic(a: (i32, i32), b: (i32, i32)) -> u32 {
    let dx = (a.0 - b.0).unsigned_abs();
    let dy = (a.1 - b.1).unsigned_abs();
    let diag = dx.min(dy);
    let straight = dx.max(dy) - diag;
    diag * 14 + straight * 10
}

fn reconstruct_path(
    came_from: &HashMap<(i32, i32), (i32, i32)>,
    goal: (i32, i32),
) -> Vec<Vec2> {
    let mut path = Vec::new();
    let mut current = goal;

    while let Some(&prev) = came_from.get(&current) {
        path.push(GridPosition::new(current.0, current.1).to_world());
        current = prev;
    }

    path.reverse();
    smooth_path(path)
}

fn smooth_path(path: Vec<Vec2>) -> Vec<Vec2> {
    if path.len() <= 2 {
        return path;
    }

    let mut smoothed = vec![path[0]];
    let mut i = 0;

    while i < path.len() - 1 {
        let mut furthest = i + 1;
        for j in (i + 2)..path.len() {
            let dir_to_j = path[j] - path[i];
            let dir_to_next = path[i + 1] - path[i];
            if dir_to_j.normalize_or_zero().dot(dir_to_next.normalize_or_zero()) > 0.95 {
                furthest = j;
            } else {
                break;
            }
        }
        smoothed.push(path[furthest]);
        i = furthest;
    }

    smoothed
}

pub fn pathfinding_system(
    mut commands: Commands,
    query: Query<(Entity, &Transform, &super::components::MoveTarget), (With<super::components::Unit>, Without<Path>)>,
    all_units: Query<(Entity, &Transform), With<super::components::Unit>>,
    config: Res<MapConfig>,
) {
    let mut occupied: HashMap<(i32, i32), Entity> = HashMap::new();
    for (entity, transform) in &all_units {
        let grid = GridPosition::from_world(transform.translation.truncate());
        occupied.insert((grid.x, grid.y), entity);
    }

    for (entity, transform, move_target) in &query {
        let start = GridPosition::from_world(transform.translation.truncate());
        let goal = GridPosition::from_world(move_target.0);

        if let Some(waypoints) = find_path(start, goal, &occupied, entity, &config.terrain_grid) {
            commands.entity(entity).insert(Path {
                waypoints,
                current_index: 0,
            });
        }
    }
}

pub fn path_following_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &super::components::Speed, &mut Path, &mut super::components::UnitState), With<super::components::Unit>>,
    time: Res<Time>,
) {
    for (entity, mut transform, speed, mut path, mut state) in &mut query {
        if path.current_index >= path.waypoints.len() {
            commands.entity(entity).remove::<Path>();
            commands.entity(entity).remove::<super::components::MoveTarget>();
            if *state == super::components::UnitState::Moving {
                *state = super::components::UnitState::Idle;
            }
            continue;
        }

        let target = path.waypoints[path.current_index];
        let current = transform.translation.truncate();
        let direction = target - current;
        let distance = direction.length();

        if distance < 16.0 {
            path.current_index += 1;
            continue;
        }

        let velocity = direction.normalize() * speed.0 * TILE_SIZE * time.delta_secs();
        if velocity.length() > distance {
            transform.translation.x = target.x;
            transform.translation.y = target.y;
        } else {
            transform.translation.x += velocity.x;
            transform.translation.y += velocity.y;
        }
    }
}

pub fn separation_system(
    mut units: Query<(Entity, &mut Transform, &super::components::Team), With<super::components::Unit>>,
) {
    let positions: Vec<(Entity, Vec2, u8)> = units.iter()
        .map(|(e, t, team)| (e, t.translation.truncate(), team.0))
        .collect();

    let separation_radius = 60.0;
    let separation_force = 4.0;

    for (entity, mut transform, _team) in &mut units {
        let pos = transform.translation.truncate();
        let mut push = Vec2::ZERO;

        for &(other_entity, other_pos, _) in &positions {
            if other_entity == entity { continue; }
            let diff = pos - other_pos;
            let dist = diff.length();
            if dist < separation_radius && dist > 0.01 {
                push += diff.normalize() * (separation_radius - dist) / separation_radius;
            }
        }

        if push.length() > 0.01 {
            let nudge = push.normalize() * separation_force;
            transform.translation.x += nudge.x;
            transform.translation.y += nudge.y;
        }
    }
}
