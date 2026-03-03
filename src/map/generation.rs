use bevy::prelude::*;
use rand::prelude::*;
use rand::rngs::StdRng;
use std::collections::VecDeque;
use super::{GridPosition, MAP_WIDTH, MAP_HEIGHT};
use crate::resources::components::ResourceKind;

const EDGE_MARGIN: i32 = 5;
const PLAYER_LAND_RADIUS: i32 = 5;

const NEIGHBORS_4: [(i32, i32); 4] = [(0, 1), (0, -1), (1, 0), (-1, 0)];

#[derive(Clone, Debug)]
pub struct ResourceCluster {
    pub positions: Vec<(i32, i32)>,
    pub kind: ResourceKind,
    pub amount: u32,
}

#[derive(Resource)]
pub struct MapConfig {
    pub seed: u64,
    pub player_base: GridPosition,
    pub ai_base: GridPosition,
    pub terrain_grid: Vec<Vec<TerrainOverride>>,
    pub resource_clusters: Vec<ResourceCluster>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TerrainOverride {
    None,
    ForceGrass,
    ForceForest,
    ForceDirt,
    ForceWater,
}

pub fn generate_map_config(mut commands: Commands) {
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;
    info!("Map seed: {seed}");

    let config = build_map(seed);
    commands.insert_resource(config);
}

pub fn build_map(seed: u64) -> MapConfig {
    let mut rng = StdRng::seed_from_u64(seed);
    let w = MAP_WIDTH as i32;
    let h = MAP_HEIGHT as i32;
    let cx = w / 2;
    let cy = h / 2;

    // --- Phase 0: Player placement on opposite sides ---
    let radius = (w.min(h) / 2 - EDGE_MARGIN - 2) as f32;
    let angle: f32 = rng.random_range(0.0..std::f32::consts::TAU);
    let px = cx + (radius * angle.cos()) as i32;
    let py = cy + (radius * angle.sin()) as i32;
    let ax = cx - (radius * angle.cos()) as i32;
    let ay = cy - (radius * angle.sin()) as i32;

    let player_base = GridPosition::new(
        px.clamp(EDGE_MARGIN, w - EDGE_MARGIN - 4),
        py.clamp(EDGE_MARGIN, h - EDGE_MARGIN - 4),
    );
    let ai_base = GridPosition::new(
        ax.clamp(EDGE_MARGIN, w - EDGE_MARGIN - 4),
        ay.clamp(EDGE_MARGIN, h - EDGE_MARGIN - 4),
    );

    // --- Phase 1: Fill entire map with grass ---
    let mut grid = vec![vec![TerrainOverride::ForceGrass; h as usize]; w as usize];

    // --- Phase 2: Paint dirt patches (4-8 coherent blobs) ---
    let num_dirt = rng.random_range(4..=8);
    for _ in 0..num_dirt {
        for _ in 0..20 {
            let dx = rng.random_range(2..w - 2);
            let dy = rng.random_range(2..h - 2);
            if too_close_to_base(dx, dy, player_base, ai_base, PLAYER_LAND_RADIUS + 2) {
                continue;
            }
            let size = rng.random_range(8..20);
            grow_blob(&mut grid, &mut rng, dx, dy, size, TerrainOverride::ForceDirt, w, h);
            break;
        }
    }

    // --- Phase 3: Paint water bodies (1-3 connected lakes) ---
    let num_lakes = rng.random_range(1..=3);
    for _ in 0..num_lakes {
        for _ in 0..30 {
            let lx = rng.random_range(5..w - 5);
            let ly = rng.random_range(5..h - 5);
            if too_close_to_base(lx, ly, player_base, ai_base, PLAYER_LAND_RADIUS + 6) {
                continue;
            }
            let size = rng.random_range(15..40);
            grow_blob(&mut grid, &mut rng, lx, ly, size, TerrainOverride::ForceWater, w, h);
            break;
        }
    }

    // --- Phase 4: Paint forest clusters (6-10 organic blobs) ---
    let num_forests = rng.random_range(6..=10);
    for _ in 0..num_forests {
        for _ in 0..20 {
            let fx = rng.random_range(3..w - 3);
            let fy = rng.random_range(3..h - 3);
            if too_close_to_base(fx, fy, player_base, ai_base, PLAYER_LAND_RADIUS + 3) {
                continue;
            }
            let size = rng.random_range(10..30);
            grow_blob(&mut grid, &mut rng, fx, fy, size, TerrainOverride::ForceForest, w, h);
            break;
        }
    }

    // --- Phase 5: Clear player lands (force grass around bases) ---
    clear_player_land(&mut grid, player_base, PLAYER_LAND_RADIUS);
    clear_player_land(&mut grid, ai_base, PLAYER_LAND_RADIUS);

    // --- Phase 6: Resource placement ---
    let mut clusters = Vec::new();

    for &base in &[player_base, ai_base] {
        generate_base_resources(&mut rng, base, w, h, &grid, &mut clusters);
    }

    let mid = GridPosition::new(cx, cy);
    let central_gold = cluster_at(mid.x, mid.y, 2, 2, w, h);
    clusters.push(ResourceCluster {
        positions: central_gold,
        kind: ResourceKind::Gold,
        amount: 800,
    });
    let sx = cx + rng.random_range(-3..=3);
    let sy = cy + rng.random_range(-3..=3);
    let central_stone = cluster_at(sx, sy, 2, 1, w, h);
    clusters.push(ResourceCluster {
        positions: central_stone,
        kind: ResourceKind::Stone,
        amount: 400,
    });

    MapConfig {
        seed,
        player_base,
        ai_base,
        terrain_grid: grid,
        resource_clusters: clusters,
    }
}

fn too_close_to_base(x: i32, y: i32, player: GridPosition, ai: GridPosition, min_dist: i32) -> bool {
    let dp = (x - player.x).pow(2) + (y - player.y).pow(2);
    let da = (x - ai.x).pow(2) + (y - ai.y).pow(2);
    let threshold = min_dist * min_dist;
    dp < threshold || da < threshold
}

/// Grows a connected blob of tiles outward from a center point.
/// Maintains a frontier of candidate tiles adjacent to the blob,
/// picks one at random each step, producing organic irregular shapes.
fn grow_blob(
    grid: &mut [Vec<TerrainOverride>],
    rng: &mut StdRng,
    cx: i32,
    cy: i32,
    target_size: i32,
    override_type: TerrainOverride,
    w: i32,
    h: i32,
) {
    let mut placed = 0;
    let mut frontier: VecDeque<(i32, i32)> = VecDeque::new();

    if cx < 1 || cx >= w - 1 || cy < 1 || cy >= h - 1 {
        return;
    }
    grid[cx as usize][cy as usize] = override_type;
    placed += 1;

    for &(dx, dy) in &NEIGHBORS_4 {
        let nx = cx + dx;
        let ny = cy + dy;
        if nx >= 1 && nx < w - 1 && ny >= 1 && ny < h - 1 {
            frontier.push_back((nx, ny));
        }
    }

    while placed < target_size && !frontier.is_empty() {
        let idx = rng.random_range(0..frontier.len());
        let (fx, fy) = frontier.remove(idx).unwrap();

        if grid[fx as usize][fy as usize] == override_type {
            continue;
        }

        grid[fx as usize][fy as usize] = override_type;
        placed += 1;

        for &(dx, dy) in &NEIGHBORS_4 {
            let nx = fx + dx;
            let ny = fy + dy;
            if nx >= 1 && nx < w - 1 && ny >= 1 && ny < h - 1
                && grid[nx as usize][ny as usize] != override_type
            {
                frontier.push_back((nx, ny));
            }
        }
    }
}

fn generate_base_resources(
    rng: &mut StdRng,
    base: GridPosition,
    w: i32,
    h: i32,
    _grid: &[Vec<TerrainOverride>],
    clusters: &mut Vec<ResourceCluster>,
) {
    let bx = base.x;
    let by = base.y;

    let berry_angle: f32 = rng.random_range(0.0..std::f32::consts::TAU);
    let berry_dist = rng.random_range(4.0..6.0f32);
    let berry_cx = bx + (berry_dist * berry_angle.cos()) as i32;
    let berry_cy = by + (berry_dist * berry_angle.sin()) as i32;
    let berries = cluster_at(berry_cx, berry_cy, 3, 2, w, h);
    clusters.push(ResourceCluster {
        positions: berries,
        kind: ResourceKind::Food,
        amount: 125,
    });

    let tree_angle1 = berry_angle + rng.random_range(1.2..2.5);
    let tree_dist1 = rng.random_range(5.0..7.0f32);
    let tcx1 = bx + (tree_dist1 * tree_angle1.cos()) as i32;
    let tcy1 = by + (tree_dist1 * tree_angle1.sin()) as i32;
    let trees1 = cluster_at(tcx1, tcy1, 3, 3, w, h);
    clusters.push(ResourceCluster {
        positions: trees1,
        kind: ResourceKind::Wood,
        amount: 150,
    });

    let tree_angle2 = berry_angle - rng.random_range(1.2..2.5);
    let tree_dist2 = rng.random_range(5.0..8.0f32);
    let tcx2 = bx + (tree_dist2 * tree_angle2.cos()) as i32;
    let tcy2 = by + (tree_dist2 * tree_angle2.sin()) as i32;
    let trees2 = cluster_at(tcx2, tcy2, 3, 2, w, h);
    clusters.push(ResourceCluster {
        positions: trees2,
        kind: ResourceKind::Wood,
        amount: 150,
    });

    let gold_angle1 = berry_angle + rng.random_range(2.5..4.0);
    let gold_dist1 = rng.random_range(6.0..8.0f32);
    let gcx1 = bx + (gold_dist1 * gold_angle1.cos()) as i32;
    let gcy1 = by + (gold_dist1 * gold_angle1.sin()) as i32;
    let gold1 = cluster_at(gcx1, gcy1, 2, 2, w, h);
    clusters.push(ResourceCluster {
        positions: gold1,
        kind: ResourceKind::Gold,
        amount: 800,
    });

    let gold_angle2 = gold_angle1 + rng.random_range(1.5..3.0);
    let gold_dist2 = rng.random_range(10.0..14.0f32);
    let gcx2 = bx + (gold_dist2 * gold_angle2.cos()) as i32;
    let gcy2 = by + (gold_dist2 * gold_angle2.sin()) as i32;
    let gold2 = cluster_at(gcx2, gcy2, 2, 2, w, h);
    clusters.push(ResourceCluster {
        positions: gold2,
        kind: ResourceKind::Gold,
        amount: 800,
    });

    let stone_angle = berry_angle + rng.random_range(4.0..5.5);
    let stone_dist = rng.random_range(6.0..10.0f32);
    let scx = bx + (stone_dist * stone_angle.cos()) as i32;
    let scy = by + (stone_dist * stone_angle.sin()) as i32;
    let stone = cluster_at(scx, scy, 2, 2, w, h);
    clusters.push(ResourceCluster {
        positions: stone,
        kind: ResourceKind::Stone,
        amount: 400,
    });
}

fn cluster_at(cx: i32, cy: i32, cols: i32, rows: i32, w: i32, h: i32) -> Vec<(i32, i32)> {
    let mut positions = Vec::new();
    for dx in 0..cols {
        for dy in 0..rows {
            let x = (cx + dx).clamp(1, w - 2);
            let y = (cy + dy).clamp(1, h - 2);
            positions.push((x, y));
        }
    }
    positions
}

fn clear_player_land(grid: &mut [Vec<TerrainOverride>], base: GridPosition, radius: i32) {
    let w = grid.len() as i32;
    let h = grid[0].len() as i32;
    for dx in -radius..=radius {
        for dy in -radius..=radius {
            if dx * dx + dy * dy > radius * radius { continue; }
            let x = base.x + dx;
            let y = base.y + dy;
            if x >= 0 && x < w && y >= 0 && y < h {
                grid[x as usize][y as usize] = TerrainOverride::ForceGrass;
            }
        }
    }
}
