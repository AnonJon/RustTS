use bevy::prelude::*;
use super::{TILE_SIZE, MAP_WIDTH, MAP_HEIGHT};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjectionMode {
    TopDown,
    Isometric,
}

impl Default for ProjectionMode {
    fn default() -> Self {
        ProjectionMode::TopDown
    }
}

pub fn grid_to_world(x: i32, y: i32, mode: ProjectionMode) -> Vec2 {
    match mode {
        ProjectionMode::TopDown => Vec2::new(
            x as f32 * TILE_SIZE + TILE_SIZE / 2.0,
            y as f32 * TILE_SIZE + TILE_SIZE / 2.0,
        ),
        ProjectionMode::Isometric => {
            let iso_x = (x - y) as f32 * (TILE_SIZE / 2.0);
            let iso_y = (x + y) as f32 * (TILE_SIZE / 4.0);
            Vec2::new(
                iso_x + (MAP_WIDTH as f32 * TILE_SIZE / 2.0),
                iso_y,
            )
        }
    }
}

pub fn world_to_grid(world: Vec2, mode: ProjectionMode) -> (i32, i32) {
    match mode {
        ProjectionMode::TopDown => (
            (world.x / TILE_SIZE).floor() as i32,
            (world.y / TILE_SIZE).floor() as i32,
        ),
        ProjectionMode::Isometric => {
            let adjusted_x = world.x - (MAP_WIDTH as f32 * TILE_SIZE / 2.0);
            let adjusted_y = world.y;
            let half = TILE_SIZE / 2.0;
            let quarter = TILE_SIZE / 4.0;
            let grid_x = ((adjusted_x / half) + (adjusted_y / quarter)) / 2.0;
            let grid_y = ((adjusted_y / quarter) - (adjusted_x / half)) / 2.0;
            (grid_x.floor() as i32, grid_y.floor() as i32)
        }
    }
}

pub fn depth_sort_z(y: f32, mode: ProjectionMode) -> f32 {
    match mode {
        ProjectionMode::TopDown => 10.0,
        ProjectionMode::Isometric => {
            let max_y = MAP_HEIGHT as f32 * TILE_SIZE;
            10.0 + (max_y - y) / max_y
        }
    }
}
