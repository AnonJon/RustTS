use bevy::prelude::*;
use super::{GridPosition, MAP_WIDTH, MAP_HEIGHT, TILE_SIZE};
use crate::units::components::{Unit, Team};
use crate::buildings::components::Building;

/// Line of sight radius in tiles.
#[derive(Component)]
pub struct LineOfSight(pub u32);

#[derive(Resource)]
pub struct FogOfWar {
    /// Whether a tile has ever been seen by the player.
    pub explored: Vec<Vec<bool>>,
    /// How many player entities currently have vision on this tile.
    /// Tile is visible when visible[x][y] > 0.
    pub visible: Vec<Vec<u8>>,
}

impl Default for FogOfWar {
    fn default() -> Self {
        let w = MAP_WIDTH as usize;
        let h = MAP_HEIGHT as usize;
        Self {
            explored: vec![vec![false; h]; w],
            visible: vec![vec![0u8; h]; w],
        }
    }
}

impl FogOfWar {
    pub fn is_visible(&self, x: i32, y: i32) -> bool {
        if x < 0 || y < 0 || x >= MAP_WIDTH as i32 || y >= MAP_HEIGHT as i32 {
            return false;
        }
        self.visible[x as usize][y as usize] > 0
    }

    pub fn is_explored(&self, x: i32, y: i32) -> bool {
        if x < 0 || y < 0 || x >= MAP_WIDTH as i32 || y >= MAP_HEIGHT as i32 {
            return false;
        }
        self.explored[x as usize][y as usize]
    }
}

/// Recalculate the visibility grid each frame based on player entities.
pub fn update_fog_system(
    mut fog: ResMut<FogOfWar>,
    units: Query<(&Transform, &Team, &LineOfSight), With<Unit>>,
    buildings: Query<(&GridPosition, &Team, &Building, Option<&LineOfSight>)>,
) {
    let w = MAP_WIDTH as usize;
    let h = MAP_HEIGHT as usize;

    // Clear visibility (but keep explored)
    for x in 0..w {
        for y in 0..h {
            fog.visible[x][y] = 0;
        }
    }

    // Mark tiles visible from player units
    for (transform, team, los) in &units {
        if team.0 != 0 { continue; }
        let grid = GridPosition::from_world(transform.translation.truncate());
        mark_visible(&mut fog, grid.x, grid.y, los.0 as i32);
    }

    // Mark tiles visible from player buildings
    for (grid, team, building, los) in &buildings {
        if team.0 != 0 { continue; }
        let (tw, th) = building.kind.tile_size();
        let center_x = grid.x + tw as i32 / 2;
        let center_y = grid.y + th as i32 / 2;
        let radius = los.map_or(
            match building.kind {
                crate::buildings::components::BuildingKind::TownCenter => 8,
                _ => 4,
            },
            |l| l.0 as i32,
        );
        mark_visible(&mut fog, center_x, center_y, radius);
    }
}

fn mark_visible(fog: &mut FogOfWar, cx: i32, cy: i32, radius: i32) {
    let w = MAP_WIDTH as i32;
    let h = MAP_HEIGHT as i32;
    let r2 = radius * radius;

    for dx in -radius..=radius {
        for dy in -radius..=radius {
            if dx * dx + dy * dy > r2 { continue; }
            let x = cx + dx;
            let y = cy + dy;
            if x < 0 || y < 0 || x >= w || y >= h { continue; }
            let ux = x as usize;
            let uy = y as usize;
            fog.visible[ux][uy] = fog.visible[ux][uy].saturating_add(1);
            fog.explored[ux][uy] = true;
        }
    }
}

/// Hide/show enemy entities based on fog of war visibility.
pub fn apply_fog_visibility_system(
    mut units: Query<(&Transform, &Team, &mut Visibility), With<Unit>>,
    mut buildings: Query<(&GridPosition, &Team, &mut Visibility), (With<Building>, Without<Unit>)>,
    fog: Res<FogOfWar>,
) {
    for (transform, team, mut vis) in &mut units {
        if team.0 == 0 { continue; } // always show own units
        let grid = GridPosition::from_world(transform.translation.truncate());
        *vis = if fog.is_visible(grid.x, grid.y) {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }

    for (grid, team, mut vis) in &mut buildings {
        if team.0 == 0 { continue; }
        *vis = if fog.is_visible(grid.x, grid.y) {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

/// Draw fog overlay using gizmos — dark overlay on unexplored/non-visible tiles.
/// This is a lightweight approach; a proper implementation would use a shader or overlay tilemap.
pub fn draw_fog_overlay(
    fog: Res<FogOfWar>,
    mut gizmos: Gizmos,
) {
    let w = MAP_WIDTH as usize;
    let h = MAP_HEIGHT as usize;

    for x in 0..w {
        for y in 0..h {
            let grid = GridPosition::new(x as i32, y as i32);
            let world = grid.to_world();

            if fog.visible[x][y] > 0 {
                // Fully visible — no overlay
                continue;
            }

            let color = if fog.explored[x][y] {
                // Explored but not currently visible — dim overlay
                Color::srgba(0.0, 0.0, 0.0, 0.5)
            } else {
                // Never explored — full black
                Color::srgba(0.0, 0.0, 0.0, 0.85)
            };

            // Draw a diamond shape to match isometric tiles
            let half_w = TILE_SIZE / 2.0; // 64
            let half_h = TILE_SIZE / 4.0; // 32

            // Draw as a filled rect approximation
            gizmos.rect_2d(
                Isometry2d::from_translation(world),
                Vec2::new(half_w * 2.0, half_h * 2.0),
                color,
            );
        }
    }
}
