use bevy::prelude::*;
use rand::prelude::*;
use rand::rngs::StdRng;
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashSet, VecDeque};
use super::{GridPosition, MAP_HEIGHT, MAP_WIDTH};
use super::terrain::TerrainType;
use crate::resources::components::ResourceKind;

const EDGE_MARGIN: i32 = 5;
const NEIGHBORS_4: [(i32, i32); 4] = [(0, 1), (0, -1), (1, 0), (-1, 0)];

// ── Core data structures ───────────────────────────────

#[derive(Clone, Copy, Debug)]
pub struct Tile {
    pub terrain: TerrainType,
    pub elevation: u8,
    pub zone_id: u8,
    pub cliff: bool,
    pub object_id: Option<u32>,
}

impl Tile {
    pub fn new(terrain: TerrainType) -> Self {
        Self { terrain, elevation: 0, zone_id: 0, cliff: false, object_id: None }
    }

    pub fn is_water(&self) -> bool {
        self.terrain == TerrainType::Water
    }

    pub fn is_walkable(&self) -> bool {
        self.terrain.is_walkable() && !self.cliff
    }
}

impl Default for Tile {
    fn default() -> Self {
        Self::new(TerrainType::Grass)
    }
}

#[derive(Clone, Debug)]
pub struct PlayerStart {
    pub position: GridPosition,
    pub is_human: bool,
}

#[derive(Clone, Debug)]
pub struct ResourceCluster {
    pub positions: Vec<(i32, i32)>,
    pub kind: ResourceKind,
    pub amount: u32,
}

#[derive(Resource)]
pub struct MapConfig {
    pub seed: u64,
    pub players: Vec<PlayerStart>,
    pub terrain_grid: Vec<Vec<Tile>>,
    pub resource_clusters: Vec<ResourceCluster>,
}

impl MapConfig {
    pub fn player_base(&self) -> GridPosition {
        self.players
            .iter()
            .find(|p| p.is_human)
            .map(|p| p.position)
            .unwrap_or(self.players[0].position)
    }

    pub fn ai_base(&self) -> GridPosition {
        self.players
            .iter()
            .find(|p| !p.is_human)
            .map(|p| p.position)
            .unwrap_or_else(|| self.players.last().unwrap().position)
    }
}

// ── Map type selection ──────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum MapType {
    #[default]
    Arabia,
    BlackForest,
    Islands,
    Coastal,
}

impl MapType {
    pub const ALL: [MapType; 4] = [
        MapType::Arabia,
        MapType::BlackForest,
        MapType::Islands,
        MapType::Coastal,
    ];

    pub fn label(self) -> &'static str {
        match self {
            MapType::Arabia => "Arabia",
            MapType::BlackForest => "Black Forest",
            MapType::Islands => "Islands",
            MapType::Coastal => "Coastal",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            MapType::Arabia => "Open grasslands with moderate forests and light water",
            MapType::BlackForest => "Dense forest walls between players with narrow passages",
            MapType::Islands => "Each player on a separate island surrounded by water",
            MapType::Coastal => "Land-based map with a large water body along one edge",
        }
    }

    pub fn params(self, num_players: usize) -> MapParams {
        let mut p = MapParams::default();
        p.num_players = num_players;

        match self {
            MapType::Arabia => {
                p.num_neutral_lands = (1, 3);
                p.neutral_land_size = (20, 50);
            }
            MapType::BlackForest => {
                p.num_forests = (20, 30);
                p.forest_size = (20, 50);
                p.num_water = (0, 1);
                p.water_size = (5, 15);
                p.forest_wall_between_players = true;
                p.connection_width = 2;
            }
            MapType::Islands => {
                p.base_terrain = TerrainType::Water;
                p.player_land_percent = 0.12;
                p.zone_avoidance = 8;
                p.clumping_factor = 0.8;
                p.base_size = 4;
                p.connect_players = false;
                p.num_water = (0, 0);
                p.num_forests = (2, 5);
                p.forest_size = (4, 10);
                p.num_dirt = (0, 1);
                p.dirt_size = (6, 12);
                p.max_resource_distance = 8.0;
                p.num_neutral_lands = (2, 4);
                p.neutral_land_size = (15, 40);
            }
            MapType::Coastal => {
                p.coastal_sides = 1;
                p.num_water = (0, 1);
                p.water_size = (5, 15);
                p.num_forests = (6, 10);
                p.forest_size = (10, 25);
            }
        }
        p
    }
}

// ── Generation parameters (precursor to RMS scripting) ─

pub struct MapParams {
    pub num_players: usize,
    pub base_size: i32,
    pub player_land_percent: f32,
    pub clumping_factor: f32,
    pub zone_avoidance: i32,
    pub num_hills: (usize, usize),
    pub hill_size: (i32, i32),
    pub elevation_avoid_bases: i32,
    pub num_forests: (usize, usize),
    pub forest_size: (i32, i32),
    pub num_dirt: (usize, usize),
    pub dirt_size: (i32, i32),
    pub num_water: (usize, usize),
    pub water_size: (i32, i32),
    pub base_terrain: TerrainType,
    pub connect_players: bool,
    pub connection_width: i32,
    pub coastal_sides: u8,
    pub forest_wall_between_players: bool,
    pub max_resource_distance: f32,
    pub num_neutral_lands: (usize, usize),
    pub neutral_land_size: (i32, i32),
}

impl Default for MapParams {
    fn default() -> Self {
        Self {
            num_players: 2,
            base_size: 3,
            player_land_percent: 0.15,
            clumping_factor: 0.5,
            zone_avoidance: 3,
            num_hills: (3, 6),
            hill_size: (8, 20),
            elevation_avoid_bases: 9,
            num_forests: (6, 10),
            forest_size: (10, 30),
            num_dirt: (1, 3),
            dirt_size: (30, 60),
            num_water: (1, 3),
            water_size: (15, 40),
            base_terrain: TerrainType::Grass,
            connect_players: true,
            connection_width: 3,
            coastal_sides: 0,
            forest_wall_between_players: false,
            max_resource_distance: 21.0,
            num_neutral_lands: (0, 0),
            neutral_land_size: (20, 60),
        }
    }
}

// ── Internal land struct for simultaneous growth ───────

struct Land {
    zone: u8,
    origin: (i32, i32),
    target_tiles: i32,
    terrain: TerrainType,
    clumping_factor: f32,
    frontier: Vec<(i32, i32)>,
    placed_count: i32,
}

// ── Public API ─────────────────────────────────────────

pub fn generate_map_config(mut commands: Commands, settings: Res<crate::GameSettings>) {
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;
    info!("Map seed: {seed}, type: {:?}, players: {}", settings.map_type, settings.num_players);
    let params = settings.map_type.params(settings.num_players);
    let config = build_map_with_params(seed, &params);
    commands.insert_resource(config);
}

pub fn build_map(seed: u64) -> MapConfig {
    build_map_with_params(seed, &MapParams::default())
}

/// AoE2-style generation pipeline executed in fixed section order:
///   1. Player Setup      -- place N players on a circle
///   2. Land Generation   -- base squares + simultaneous growth with zones
///   3. Elevation          -- random hills avoiding player bases
///   4. Cliff Generation   -- (stub)
///   5. Terrain Generation -- forest / dirt / water clumps overlaid on lands
///   6. Connection Gen     -- ensure walkable paths between all players
///   7. Object Generation  -- per-base and central resources
pub fn build_map_with_params(seed: u64, params: &MapParams) -> MapConfig {
    let mut rng = StdRng::seed_from_u64(seed);
    let w = MAP_WIDTH as i32;
    let h = MAP_HEIGHT as i32;

    // 1. Initialize grid with base terrain (Grass for most maps, Water for Islands)
    let base_tile = Tile::new(params.base_terrain);
    let mut grid: Vec<Vec<Tile>> = (0..w as usize)
        .map(|_| vec![base_tile; h as usize])
        .collect();

    // 2. Player Setup
    let players = place_players(&mut rng, params, w, h);
    info!(
        "Players: {:?}",
        players.iter().map(|p| p.position).collect::<Vec<_>>()
    );

    // 3. Land Generation
    generate_lands(&mut rng, params, &mut grid, &players, w, h);

    // 4. Elevation Generation
    generate_elevation(&mut rng, params, &mut grid, &players, w, h);

    // 5. (Cliff Generation -- stub, reserved for future)

    // 5b. Coastal water strip along map edges
    if params.coastal_sides > 0 {
        generate_coastal_water(&mut rng, &mut grid, params.coastal_sides, &players, w, h);
    }

    // 5c. Forest walls between players (Black Forest)
    if params.forest_wall_between_players {
        generate_forest_walls(&mut rng, &mut grid, &players, w, h);
    }

    // 6. Terrain Generation (clumps on top of lands)
    generate_terrain_clumps(&mut rng, params, &mut grid, &players, w, h);

    // 6b. Safety net: clear any water that ended up near bases.
    // Skip for Islands -- the land growth already creates grass islands around bases,
    // and clearing water here would destroy the island separation.
    if params.base_terrain != TerrainType::Water {
        let clear_radius = params.base_size + 10;
        clear_water_near_bases(&mut grid, &players, clear_radius, w, h);
    }

    // 7. Connection Generation (skip for Islands)
    if params.connect_players {
        generate_connections(&mut grid, &players, w, h, params.connection_width);
    }

    // 8. Object Generation (resources)
    let mut resource_clusters = generate_objects(&mut rng, &grid, &players, w, h, params.max_resource_distance);

    // Safety net: discard any cluster that ended up with a tile on water
    resource_clusters.retain(|c| {
        c.positions.iter().all(|&(x, y)| {
            let xu = x as usize;
            let yu = y as usize;
            xu < grid.len() && yu < grid[0].len() && !grid[xu][yu].is_water()
        })
    });

    MapConfig {
        seed,
        players,
        terrain_grid: grid,
        resource_clusters,
    }
}

// ── Stage 2: Player Setup ──────────────────────────────

fn place_players(
    rng: &mut StdRng,
    params: &MapParams,
    w: i32,
    h: i32,
) -> Vec<PlayerStart> {
    let cx = w as f32 / 2.0;
    let cy = h as f32 / 2.0;
    let n = params.num_players;
    let max_radius = (w.min(h) as f32 / 2.0) - EDGE_MARGIN as f32 - params.base_size as f32;
    let radius = max_radius * rng.random_range(0.65..0.90);

    let base_angle: f32 = rng.random_range(0.0..std::f32::consts::TAU);
    let angle_step = std::f32::consts::TAU / n as f32;

    (0..n)
        .map(|i| {
            let angle = base_angle + i as f32 * angle_step;
            let px = (cx + radius * angle.cos()).round() as i32;
            let py = (cy + radius * angle.sin()).round() as i32;
            PlayerStart {
                position: GridPosition::new(
                    px.clamp(EDGE_MARGIN, w - EDGE_MARGIN - 1),
                    py.clamp(EDGE_MARGIN, h - EDGE_MARGIN - 1),
                ),
                is_human: i == 0,
            }
        })
        .collect()
}

// ── Stage 3: Land Generation ───────────────────────────

fn generate_lands(
    rng: &mut StdRng,
    params: &MapParams,
    grid: &mut Vec<Vec<Tile>>,
    players: &[PlayerStart],
    w: i32,
    h: i32,
) {
    let total_tiles = w * h;
    let player_target = (total_tiles as f32 * params.player_land_percent) as i32;
    let base_half = params.base_size;

    let mut lands: Vec<Land> = Vec::new();

    // Create one land per player, place base squares, seed frontiers
    for (i, player) in players.iter().enumerate() {
        let zone = (i + 1) as u8;
        let origin = (player.position.x, player.position.y);

        let mut land = Land {
            zone,
            origin,
            target_tiles: player_target,
            terrain: TerrainType::Grass,
            clumping_factor: params.clumping_factor,
            frontier: Vec::new(),
            placed_count: 0,
        };

        place_base_square(grid, &mut land, base_half, w, h);
        lands.push(land);
    }

    // All lands grow simultaneously from their origins
    grow_lands_simultaneously(grid, &mut lands, rng, w, h, params.zone_avoidance);

    // Neutral (non-player) lands for map variety (bonus islands, decorative landmasses)
    let num_neutral = if params.num_neutral_lands.0 < params.num_neutral_lands.1 {
        rng.random_range(params.num_neutral_lands.0..=params.num_neutral_lands.1)
    } else {
        params.num_neutral_lands.0
    };

    if num_neutral > 0 {
        let next_zone = (players.len() + 1) as u8;
        let mut neutrals: Vec<Land> = Vec::new();

        for i in 0..num_neutral {
            let neutral_target = if params.neutral_land_size.0 < params.neutral_land_size.1 {
                rng.random_range(params.neutral_land_size.0..=params.neutral_land_size.1)
            } else {
                params.neutral_land_size.0
            };

            // Try to find an origin that's far from existing lands
            let mut best = (rng.random_range(EDGE_MARGIN..w - EDGE_MARGIN),
                           rng.random_range(EDGE_MARGIN..h - EDGE_MARGIN));
            let mut best_dist = 0i32;

            for _ in 0..30 {
                let cx = rng.random_range(EDGE_MARGIN..w - EDGE_MARGIN);
                let cy = rng.random_range(EDGE_MARGIN..h - EDGE_MARGIN);
                let min_dist = players.iter()
                    .map(|p| (p.position.x - cx).abs() + (p.position.y - cy).abs())
                    .min()
                    .unwrap_or(0);
                if min_dist > best_dist {
                    best_dist = min_dist;
                    best = (cx, cy);
                }
            }

            let zone = next_zone + i as u8;
            let mut land = Land {
                zone,
                origin: best,
                target_tiles: neutral_target,
                terrain: TerrainType::Grass,
                clumping_factor: params.clumping_factor.max(0.6),
                frontier: Vec::new(),
                placed_count: 0,
            };

            let neutral_half = 2;
            place_base_square(grid, &mut land, neutral_half, w, h);
            neutrals.push(land);
        }

        grow_lands_simultaneously(grid, &mut neutrals, rng, w, h, params.zone_avoidance);
    }
}

fn place_base_square(
    grid: &mut [Vec<Tile>],
    land: &mut Land,
    half_size: i32,
    w: i32,
    h: i32,
) {
    let (ox, oy) = land.origin;

    for dx in -half_size..=half_size {
        for dy in -half_size..=half_size {
            let x = ox + dx;
            let y = oy + dy;
            if x >= 0 && x < w && y >= 0 && y < h {
                grid[x as usize][y as usize].terrain = land.terrain;
                grid[x as usize][y as usize].zone_id = land.zone;
                land.placed_count += 1;
            }
        }
    }

    // Seed frontier with ring just outside the base square
    for dx in -(half_size + 1)..=(half_size + 1) {
        for dy in -(half_size + 1)..=(half_size + 1) {
            if dx.abs() <= half_size && dy.abs() <= half_size {
                continue;
            }
            let x = ox + dx;
            let y = oy + dy;
            if x >= 1 && x < w - 1 && y >= 1 && y < h - 1
                && grid[x as usize][y as usize].zone_id == 0
            {
                land.frontier.push((x, y));
            }
        }
    }
}

fn grow_lands_simultaneously(
    grid: &mut [Vec<Tile>],
    lands: &mut [Land],
    rng: &mut StdRng,
    w: i32,
    h: i32,
    zone_avoidance: i32,
) {
    let mut any_grew = true;
    while any_grew {
        any_grew = false;
        for land_idx in 0..lands.len() {
            if lands[land_idx].placed_count >= lands[land_idx].target_tiles
                || lands[land_idx].frontier.is_empty()
            {
                continue;
            }
            if try_grow_land_one_tile(grid, &mut lands[land_idx], rng, w, h, zone_avoidance) {
                any_grew = true;
            }
        }
    }
}

fn try_grow_land_one_tile(
    grid: &mut [Vec<Tile>],
    land: &mut Land,
    rng: &mut StdRng,
    w: i32,
    h: i32,
    zone_avoidance: i32,
) -> bool {
    let max_attempts = land.frontier.len().min(20);

    for _ in 0..max_attempts {
        if land.frontier.is_empty() {
            return false;
        }

        let idx = pick_frontier_tile(grid, &land.frontier, land.zone, land.clumping_factor, rng);
        let (fx, fy) = land.frontier.swap_remove(idx);

        if grid[fx as usize][fy as usize].zone_id != 0 {
            continue;
        }

        if zone_avoidance > 0
            && has_foreign_zone_nearby(grid, fx, fy, w, h, land.zone, zone_avoidance)
        {
            continue;
        }

        grid[fx as usize][fy as usize].terrain = land.terrain;
        grid[fx as usize][fy as usize].zone_id = land.zone;
        land.placed_count += 1;

        for &(dx, dy) in &NEIGHBORS_4 {
            let nx = fx + dx;
            let ny = fy + dy;
            if nx >= 1 && nx < w - 1 && ny >= 1 && ny < h - 1
                && grid[nx as usize][ny as usize].zone_id == 0
            {
                land.frontier.push((nx, ny));
            }
        }

        return true;
    }
    false
}

fn pick_frontier_tile(
    grid: &[Vec<Tile>],
    frontier: &[(i32, i32)],
    zone: u8,
    clumping: f32,
    rng: &mut StdRng,
) -> usize {
    if frontier.len() <= 1 || clumping < 0.01 {
        return if frontier.is_empty() { 0 } else { rng.random_range(0..frontier.len()) };
    }

    let sample_size = frontier.len().min(16);
    let mut best_idx = 0;
    let mut best_score = f32::NEG_INFINITY;

    for _ in 0..sample_size {
        let i = rng.random_range(0..frontier.len());
        let (x, y) = frontier[i];
        let neighbor_score = count_same_zone_neighbors(grid, x, y, zone) as f32 / 4.0;
        let random_score: f32 = rng.random_range(0.0..1.0);
        let score = clumping * neighbor_score + (1.0 - clumping) * random_score;
        if score > best_score {
            best_score = score;
            best_idx = i;
        }
    }
    best_idx
}

fn count_same_zone_neighbors(grid: &[Vec<Tile>], x: i32, y: i32, zone: u8) -> i32 {
    let mut count = 0;
    for &(dx, dy) in &NEIGHBORS_4 {
        let nx = x + dx;
        let ny = y + dy;
        if nx >= 0
            && (nx as usize) < grid.len()
            && ny >= 0
            && (ny as usize) < grid[0].len()
            && grid[nx as usize][ny as usize].zone_id == zone
        {
            count += 1;
        }
    }
    count
}

fn has_foreign_zone_nearby(
    grid: &[Vec<Tile>],
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    own_zone: u8,
    distance: i32,
) -> bool {
    let dist_sq = distance * distance;
    for dx in -distance..=distance {
        for dy in -distance..=distance {
            if dx * dx + dy * dy > dist_sq {
                continue;
            }
            let nx = x + dx;
            let ny = y + dy;
            if nx >= 0 && nx < w && ny >= 0 && ny < h {
                let z = grid[nx as usize][ny as usize].zone_id;
                if z != 0 && z != own_zone {
                    return true;
                }
            }
        }
    }
    false
}

// ── Stage 4: Elevation Generation ──────────────────────

fn generate_elevation(
    rng: &mut StdRng,
    params: &MapParams,
    grid: &mut [Vec<Tile>],
    players: &[PlayerStart],
    w: i32,
    h: i32,
) {
    let num_hills = rng.random_range(params.num_hills.0..=params.num_hills.1);
    let avoid_sq = params.elevation_avoid_bases * params.elevation_avoid_bases;

    for _ in 0..num_hills {
        for _ in 0..50 {
            let hx = rng.random_range(3..w - 3);
            let hy = rng.random_range(3..h - 3);

            let too_close = players.iter().any(|p| {
                let dx = hx - p.position.x;
                let dy = hy - p.position.y;
                dx * dx + dy * dy < avoid_sq
            });
            if too_close {
                continue;
            }
            if grid[hx as usize][hy as usize].is_water() {
                continue;
            }

            let size = rng.random_range(params.hill_size.0..=params.hill_size.1);
            let max_height: u8 = rng.random_range(2..=5);
            grow_elevation_blob(grid, rng, hx, hy, size, max_height, w, h);
            break;
        }
    }
}

fn grow_elevation_blob(
    grid: &mut [Vec<Tile>],
    rng: &mut StdRng,
    cx: i32,
    cy: i32,
    target_size: i32,
    max_height: u8,
    w: i32,
    h: i32,
) {
    if cx < 1 || cx >= w - 1 || cy < 1 || cy >= h - 1 {
        return;
    }

    grid[cx as usize][cy as usize].elevation = max_height.min(7);
    let mut placed = 1;
    let mut frontier: VecDeque<(i32, i32, u8)> = VecDeque::new();

    for &(dx, dy) in &NEIGHBORS_4 {
        let nx = cx + dx;
        let ny = cy + dy;
        if nx >= 1 && nx < w - 1 && ny >= 1 && ny < h - 1 {
            frontier.push_back((nx, ny, max_height.saturating_sub(1)));
        }
    }

    while placed < target_size && !frontier.is_empty() {
        let idx = rng.random_range(0..frontier.len());
        let (fx, fy, height) = frontier.remove(idx).unwrap();

        if height == 0 {
            continue;
        }
        if grid[fx as usize][fy as usize].is_water() {
            continue;
        }
        if grid[fx as usize][fy as usize].elevation >= height {
            continue;
        }

        grid[fx as usize][fy as usize].elevation = height;
        placed += 1;

        let next_h = height.saturating_sub(1);
        if next_h > 0 {
            for &(dx, dy) in &NEIGHBORS_4 {
                let nx = fx + dx;
                let ny = fy + dy;
                if nx >= 1 && nx < w - 1 && ny >= 1 && ny < h - 1
                    && grid[nx as usize][ny as usize].elevation < next_h
                {
                    frontier.push_back((nx, ny, next_h));
                }
            }
        }
    }
}

// ── Coastal water generation ────────────────────────────

fn generate_coastal_water(
    rng: &mut StdRng,
    grid: &mut [Vec<Tile>],
    sides: u8,
    players: &[PlayerStart],
    w: i32,
    h: i32,
) {
    let base_clear = 12i32;
    let base_clear_sq = base_clear * base_clear;

    let mut used_edges: Vec<u8> = Vec::new();
    for _ in 0..sides {
        let edge = loop {
            let e = rng.random_range(0u8..4);
            if !used_edges.contains(&e) {
                break e;
            }
        };
        used_edges.push(edge);

        let depth = rng.random_range(6..=12);

        for x in 0..w {
            for y in 0..h {
                let in_strip = match edge {
                    0 => y < depth,           // top
                    1 => y >= h - depth,      // bottom
                    2 => x < depth,           // left
                    _ => x >= w - depth,      // right
                };
                if !in_strip {
                    continue;
                }

                // Noisy boundary: tiles near the strip edge have a chance to stay land
                let dist_from_edge = match edge {
                    0 => depth - y,
                    1 => depth - (h - 1 - y),
                    2 => depth - x,
                    _ => depth - (w - 1 - x),
                };
                if dist_from_edge <= 2 && rng.random_range(0..100) < 40 {
                    continue;
                }

                // Don't place water too close to player bases
                let near_base = players.iter().any(|p| {
                    let dx = x - p.position.x;
                    let dy = y - p.position.y;
                    dx * dx + dy * dy < base_clear_sq
                });
                if near_base {
                    continue;
                }

                grid[x as usize][y as usize].terrain = TerrainType::Water;
            }
        }
    }
}

// ── Forest wall generation (Black Forest) ──────────────

fn generate_forest_walls(
    rng: &mut StdRng,
    grid: &mut [Vec<Tile>],
    players: &[PlayerStart],
    w: i32,
    h: i32,
) {
    if players.len() < 2 {
        return;
    }

    let wall_half_width = rng.random_range(4..=6);
    let base_clear = 8i32;
    let base_clear_sq = base_clear * base_clear;

    // Build a forest wall along the perpendicular bisector between each pair of
    // adjacent players (sorted by angle from center, so "adjacent" means next on
    // the ring).
    let cx = w as f32 / 2.0;
    let cy = h as f32 / 2.0;
    let mut sorted: Vec<usize> = (0..players.len()).collect();
    sorted.sort_by(|&a, &b| {
        let ang_a = (players[a].position.y as f32 - cy).atan2(players[a].position.x as f32 - cx);
        let ang_b = (players[b].position.y as f32 - cy).atan2(players[b].position.x as f32 - cx);
        ang_a.partial_cmp(&ang_b).unwrap()
    });

    for i in 0..sorted.len() {
        let j = (i + 1) % sorted.len();
        let pa = &players[sorted[i]].position;
        let pb = &players[sorted[j]].position;

        let mid_x = (pa.x + pb.x) as f32 / 2.0;
        let mid_y = (pa.y + pb.y) as f32 / 2.0;

        // Direction perpendicular to the line between the two players
        let dx = (pb.x - pa.x) as f32;
        let dy = (pb.y - pa.y) as f32;
        let perp_x = -dy;
        let perp_y = dx;
        let len = (perp_x * perp_x + perp_y * perp_y).sqrt().max(1.0);
        let nx = perp_x / len;
        let ny = perp_y / len;

        // Walk along the perpendicular bisector and place forest in a band
        let max_extent = (w.max(h) as f32) * 0.75;
        let step = 0.5f32;
        let mut t = -max_extent;
        while t <= max_extent {
            let sx = mid_x + nx * t;
            let sy = mid_y + ny * t;

            for offset in (-wall_half_width as i32)..=(wall_half_width as i32) {
                // offset perpendicular to the wall direction (i.e. along player-to-player line)
                let dir_x = dx / len;
                let dir_y = dy / len;
                let fx = (sx + dir_x * offset as f32).round() as i32;
                let fy = (sy + dir_y * offset as f32).round() as i32;

                if fx < 1 || fx >= w - 1 || fy < 1 || fy >= h - 1 {
                    continue;
                }

                let near_base = players.iter().any(|p| {
                    let ddx = fx - p.position.x;
                    let ddy = fy - p.position.y;
                    ddx * ddx + ddy * ddy < base_clear_sq
                });
                if near_base {
                    continue;
                }

                if grid[fx as usize][fy as usize].terrain == TerrainType::Water {
                    continue;
                }

                // Add some noise to the wall edges
                if offset.abs() >= wall_half_width - 1 && rng.random_range(0..100) < 30 {
                    continue;
                }

                grid[fx as usize][fy as usize].terrain = TerrainType::DarkGrass;
            }
            t += step;
        }
    }
}

// ── Stage 6: Terrain Generation (clumps) ───────────────

fn generate_terrain_clumps(
    rng: &mut StdRng,
    params: &MapParams,
    grid: &mut [Vec<Tile>],
    players: &[PlayerStart],
    w: i32,
    h: i32,
) {
    let base_avoid = if params.base_terrain == TerrainType::Water {
        params.base_size + 3
    } else {
        params.base_size + 11
    };
    let base_avoid_sq = base_avoid * base_avoid;
    let water_avoid = params.base_size + 18;
    let water_avoid_sq = water_avoid * water_avoid;

    // Exclusion zones prevent water blobs from growing into base areas
    let water_exclusion_radius = params.base_size + 10;
    let water_exclusion_sq = water_exclusion_radius * water_exclusion_radius;
    let water_exclusions: Vec<(i32, i32, i32)> = players
        .iter()
        .map(|p| (p.position.x, p.position.y, water_exclusion_sq))
        .collect();

    // Dirt patches: only placed for maps that explicitly want standalone dirt blobs
    // (most maps rely on forest-fringe dirt at the bottom of this function)
    if params.num_dirt.1 > 0 && params.base_terrain == TerrainType::Water {
        let num_dirt = rng.random_range(params.num_dirt.0..=params.num_dirt.1);
        for _ in 0..num_dirt {
            for _ in 0..10 {
                let dx = rng.random_range(2..w - 2);
                let dy = rng.random_range(2..h - 2);
                if too_close_to_players(dx, dy, players, base_avoid_sq) {
                    continue;
                }
                let size = rng.random_range(params.dirt_size.0..=params.dirt_size.1);
                grow_terrain_blob(grid, rng, dx, dy, size, TerrainType::Dirt, w, h, false, false, &[]);
                break;
            }
        }
    }

    // Water bodies
    let num_water = rng.random_range(params.num_water.0..=params.num_water.1);
    for _ in 0..num_water {
        for _ in 0..20 {
            let lx = rng.random_range(5..w - 5);
            let ly = rng.random_range(5..h - 5);
            if too_close_to_players(lx, ly, players, water_avoid_sq) {
                continue;
            }
            if grid[lx as usize][ly as usize].is_water() {
                continue;
            }
            let size = rng.random_range(params.water_size.0..=params.water_size.1);
            grow_terrain_blob(grid, rng, lx, ly, size, TerrainType::Water, w, h, true, false, &water_exclusions);
            break;
        }
    }

    // Forest blobs
    let num_forests = rng.random_range(params.num_forests.0..=params.num_forests.1);
    for _ in 0..num_forests {
        for _ in 0..10 {
            let fx = rng.random_range(3..w - 3);
            let fy = rng.random_range(3..h - 3);
            if too_close_to_players(fx, fy, players, base_avoid_sq) {
                continue;
            }
            if grid[fx as usize][fy as usize].is_water() {
                continue;
            }
            if tile_or_neighbor_is_water(fx, fy, w, h, grid) {
                continue;
            }
            let size = rng.random_range(params.forest_size.0..=params.forest_size.1);
            grow_terrain_blob(
                grid,
                rng,
                fx,
                fy,
                size,
                TerrainType::DarkGrass,
                w,
                h,
                false,
                true,
                &[],
            );
            break;
        }
    }

    // Dirt fringe around forests: grass tiles adjacent (including diagonals) to
    // forest get a chance to become dirt, simulating natural forest-edge bare earth.
    // Two passes widen the fringe for a more natural look.
    let neighbors_8: [(i32, i32); 8] = [
        (0, 1), (0, -1), (1, 0), (-1, 0),
        (1, 1), (1, -1), (-1, 1), (-1, -1),
    ];

    // Pass 1: direct forest neighbors (high chance)
    let mut fringe: Vec<(i32, i32)> = Vec::new();
    for x in 1..w - 1 {
        for y in 1..h - 1 {
            if grid[x as usize][y as usize].terrain != TerrainType::Grass {
                continue;
            }
            let has_forest_neighbor = neighbors_8.iter().any(|&(dx, dy)| {
                let nx = x + dx;
                let ny = y + dy;
                nx >= 0
                    && nx < w
                    && ny >= 0
                    && ny < h
                    && grid[nx as usize][ny as usize].terrain == TerrainType::DarkGrass
            });
            if has_forest_neighbor {
                fringe.push((x, y));
            }
        }
    }
    for &(x, y) in &fringe {
        if rng.random_range(0..100) < 65 {
            grid[x as usize][y as usize].terrain = TerrainType::Dirt;
        }
    }

    // Pass 2: grass tiles adjacent to the dirt we just placed (lower chance)
    // to create a softer transition from forest -> dirt -> grass
    let mut outer_fringe: Vec<(i32, i32)> = Vec::new();
    for x in 1..w - 1 {
        for y in 1..h - 1 {
            if grid[x as usize][y as usize].terrain != TerrainType::Grass {
                continue;
            }
            let has_dirt_neighbor = NEIGHBORS_4.iter().any(|&(dx, dy)| {
                let nx = x + dx;
                let ny = y + dy;
                nx >= 0
                    && nx < w
                    && ny >= 0
                    && ny < h
                    && grid[nx as usize][ny as usize].terrain == TerrainType::Dirt
            });
            if has_dirt_neighbor {
                outer_fringe.push((x, y));
            }
        }
    }
    for &(x, y) in &outer_fringe {
        if rng.random_range(0..100) < 25 {
            grid[x as usize][y as usize].terrain = TerrainType::Dirt;
        }
    }
}

fn clear_water_near_bases(
    grid: &mut [Vec<Tile>],
    players: &[PlayerStart],
    radius: i32,
    w: i32,
    h: i32,
) {
    let r_sq = radius * radius;
    for p in players {
        let bx = p.position.x;
        let by = p.position.y;
        for dx in -radius..=radius {
            for dy in -radius..=radius {
                if dx * dx + dy * dy > r_sq {
                    continue;
                }
                let x = bx + dx;
                let y = by + dy;
                if x >= 0 && x < w && y >= 0 && y < h {
                    if grid[x as usize][y as usize].is_water() {
                        grid[x as usize][y as usize].terrain = TerrainType::Grass;
                    }
                }
            }
        }
    }
}

fn too_close_to_players(x: i32, y: i32, players: &[PlayerStart], threshold_sq: i32) -> bool {
    players.iter().any(|p| {
        let dx = x - p.position.x;
        let dy = y - p.position.y;
        dx * dx + dy * dy < threshold_sq
    })
}

fn grow_terrain_blob(
    grid: &mut [Vec<Tile>],
    rng: &mut StdRng,
    cx: i32,
    cy: i32,
    target_size: i32,
    terrain: TerrainType,
    w: i32,
    h: i32,
    overwrite_water: bool,
    avoid_water_adjacent: bool,
    exclusion_zones: &[(i32, i32, i32)], // (center_x, center_y, radius_squared)
) {
    if cx < 1 || cx >= w - 1 || cy < 1 || cy >= h - 1 {
        return;
    }
    if !overwrite_water && grid[cx as usize][cy as usize].is_water() {
        return;
    }
    if avoid_water_adjacent && tile_or_neighbor_is_water(cx, cy, w, h, grid) {
        return;
    }

    let in_exclusion = |x: i32, y: i32| -> bool {
        exclusion_zones.iter().any(|&(ex, ey, r_sq)| {
            let dx = x - ex;
            let dy = y - ey;
            dx * dx + dy * dy < r_sq
        })
    };

    if in_exclusion(cx, cy) {
        return;
    }

    grid[cx as usize][cy as usize].terrain = terrain;
    let mut placed = 1;
    let mut frontier: VecDeque<(i32, i32)> = VecDeque::new();

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

        if grid[fx as usize][fy as usize].terrain == terrain {
            continue;
        }
        if !overwrite_water && grid[fx as usize][fy as usize].is_water() {
            continue;
        }
        if avoid_water_adjacent && tile_or_neighbor_is_water(fx, fy, w, h, grid) {
            continue;
        }
        if in_exclusion(fx, fy) {
            continue;
        }

        grid[fx as usize][fy as usize].terrain = terrain;
        placed += 1;

        for &(dx, dy) in &NEIGHBORS_4 {
            let nx = fx + dx;
            let ny = fy + dy;
            if nx >= 1 && nx < w - 1 && ny >= 1 && ny < h - 1
                && grid[nx as usize][ny as usize].terrain != terrain
            {
                frontier.push_back((nx, ny));
            }
        }
    }
}

fn tile_or_neighbor_is_water(x: i32, y: i32, w: i32, h: i32, grid: &[Vec<Tile>]) -> bool {
    for (dx, dy) in [(0, 0), (-1, 0), (1, 0), (0, -1), (0, 1)] {
        let nx = x + dx;
        let ny = y + dy;
        if nx >= 0 && nx < w && ny >= 0 && ny < h && grid[nx as usize][ny as usize].is_water() {
            return true;
        }
    }
    false
}

// ── Stage 7: Connection Generation ─────────────────────

fn generate_connections(
    grid: &mut [Vec<Tile>],
    players: &[PlayerStart],
    w: i32,
    h: i32,
    corridor_width: i32,
) {
    if players.len() < 2 {
        return;
    }
    let first = players[0].position;
    for player in players.iter().skip(1) {
        if !bases_connected(grid, first, player.position, w, h) {
            if let Some(path) = astar_connection(grid, first, player.position, w, h) {
                carve_along_path(grid, &path, w, h, corridor_width);
            } else {
                carve_corridor_fallback(grid, first, player.position, w, h, corridor_width);
            }
        }
    }
}

fn bases_connected(
    grid: &[Vec<Tile>],
    a: GridPosition,
    b: GridPosition,
    w: i32,
    h: i32,
) -> bool {
    let mut visited: HashSet<(i32, i32)> = HashSet::new();
    let mut queue: VecDeque<(i32, i32)> = VecDeque::new();
    queue.push_back((a.x, a.y));
    visited.insert((a.x, a.y));

    while let Some((x, y)) = queue.pop_front() {
        if x == b.x && y == b.y {
            return true;
        }
        for (dx, dy) in NEIGHBORS_4 {
            let (nx, ny) = (x + dx, y + dy);
            if nx < 0 || nx >= w || ny < 0 || ny >= h {
                continue;
            }
            if visited.contains(&(nx, ny)) {
                continue;
            }
            if grid[nx as usize][ny as usize].is_water() {
                continue;
            }
            visited.insert((nx, ny));
            queue.push_back((nx, ny));
        }
    }
    false
}

fn terrain_cost(tile: &Tile) -> u32 {
    match tile.terrain {
        TerrainType::Grass => 1,
        TerrainType::Dirt => 1,
        TerrainType::DarkGrass => 8,
        TerrainType::Water => 50,
    }
}

fn astar_connection(
    grid: &[Vec<Tile>],
    a: GridPosition,
    b: GridPosition,
    w: i32,
    h: i32,
) -> Option<Vec<(i32, i32)>> {
    let start = (a.x, a.y);
    let goal = (b.x, b.y);

    let heuristic = |x: i32, y: i32| -> u32 {
        (x - goal.0).unsigned_abs() + (y - goal.1).unsigned_abs()
    };

    let mut g_score: Vec<Vec<u32>> = vec![vec![u32::MAX; h as usize]; w as usize];
    let mut came_from: Vec<Vec<(i32, i32)>> = vec![vec![(-1, -1); h as usize]; w as usize];

    // BinaryHeap is a max-heap; wrap in Reverse for min-heap behavior
    // Entries: (Reverse(f_score), g_score, x, y)
    let mut open: BinaryHeap<(Reverse<u32>, u32, i32, i32)> = BinaryHeap::new();

    g_score[start.0 as usize][start.1 as usize] = 0;
    open.push((Reverse(heuristic(start.0, start.1)), 0, start.0, start.1));

    while let Some((_, g, x, y)) = open.pop() {
        if x == goal.0 && y == goal.1 {
            let mut path = Vec::new();
            let (mut cx, mut cy) = goal;
            while (cx, cy) != start {
                path.push((cx, cy));
                let prev = came_from[cx as usize][cy as usize];
                cx = prev.0;
                cy = prev.1;
            }
            path.push(start);
            path.reverse();
            return Some(path);
        }

        if g > g_score[x as usize][y as usize] {
            continue;
        }

        for &(dx, dy) in &NEIGHBORS_4 {
            let (nx, ny) = (x + dx, y + dy);
            if nx < 0 || nx >= w || ny < 0 || ny >= h {
                continue;
            }
            let cost = terrain_cost(&grid[nx as usize][ny as usize]);
            let new_g = g.saturating_add(cost);
            if new_g < g_score[nx as usize][ny as usize] {
                g_score[nx as usize][ny as usize] = new_g;
                came_from[nx as usize][ny as usize] = (x, y);
                let f = new_g + heuristic(nx, ny);
                open.push((Reverse(f), new_g, nx, ny));
            }
        }
    }
    None
}

fn carve_along_path(
    grid: &mut [Vec<Tile>],
    path: &[(i32, i32)],
    w: i32,
    h: i32,
    half_width: i32,
) {
    for &(px, py) in path {
        for dx in -half_width..=half_width {
            for dy in -half_width..=half_width {
                let (nx, ny) = (px + dx, py + dy);
                if nx >= 1 && nx < w - 1 && ny >= 1 && ny < h - 1 {
                    let tile = &mut grid[nx as usize][ny as usize];
                    if tile.terrain == TerrainType::DarkGrass || tile.terrain == TerrainType::Water {
                        tile.terrain = TerrainType::Grass;
                    }
                }
            }
        }
    }
}

fn carve_corridor_fallback(
    grid: &mut [Vec<Tile>],
    a: GridPosition,
    b: GridPosition,
    w: i32,
    h: i32,
    half_width: i32,
) {
    let (mut x, mut y) = (a.x, a.y);
    let (tx, ty) = (b.x, b.y);
    while x != tx || y != ty {
        if x < tx {
            x += 1;
        } else if x > tx {
            x -= 1;
        }
        if y < ty {
            y += 1;
        } else if y > ty {
            y -= 1;
        }
        for dx in -half_width..=half_width {
            for dy in -half_width..=half_width {
                let (nx, ny) = (x + dx, y + dy);
                if nx >= 1 && nx < w - 1 && ny >= 1 && ny < h - 1 {
                    grid[nx as usize][ny as usize].terrain = TerrainType::Grass;
                }
            }
        }
    }
}

// ── Stage 8: Object Generation (resources) ─────────────

fn generate_objects(
    rng: &mut StdRng,
    grid: &[Vec<Tile>],
    players: &[PlayerStart],
    w: i32,
    h: i32,
    max_resource_distance: f32,
) -> Vec<ResourceCluster> {
    let cx = w / 2;
    let cy = h / 2;
    let mut clusters = Vec::new();
    let mut occupied_tiles: HashSet<(i32, i32)> = HashSet::new();

    for player in players {
        place_base_resources(
            rng,
            player.position,
            w,
            h,
            grid,
            &mut clusters,
            &mut occupied_tiles,
            max_resource_distance,
        );
    }

    place_central_resources(rng, cx, cy, w, h, grid, &mut clusters, &mut occupied_tiles);
    clusters
}

fn place_base_resources(
    rng: &mut StdRng,
    base_pos: GridPosition,
    w: i32,
    h: i32,
    grid: &[Vec<Tile>],
    clusters: &mut Vec<ResourceCluster>,
    occupied: &mut HashSet<(i32, i32)>,
    max_dist_cap: f32,
) {
    let base = (base_pos.x, base_pos.y);
    let cap = |d: f32| d.min(max_dist_cap);

    place_resource(
        rng, base, cap(4.0), cap(7.0), 3, 2, false, false, ResourceKind::Food, 125, "berries", w, h, grid,
        clusters, occupied,
    );
    place_resource(
        rng, base, cap(5.0), cap(9.0), 3, 2, true, true, ResourceKind::Wood, 150, "wood-main", w, h, grid,
        clusters, occupied,
    );
    place_resource(
        rng, base, cap(6.0), cap(10.0), 2, 2, true, true, ResourceKind::Wood, 150, "wood-secondary", w, h,
        grid, clusters, occupied,
    );
    place_resource(
        rng, base, cap(9.0).min(cap(13.0) - 1.0), cap(13.0), 2, 2, false, false, ResourceKind::Gold, 800, "near-gold", w, h,
        grid, clusters, occupied,
    );
    place_resource(
        rng, base, cap(10.0).min(cap(15.0) - 1.0), cap(15.0), 2, 2, false, false, ResourceKind::Stone, 400, "stone", w, h, grid,
        clusters, occupied,
    );
    place_resource(
        rng, base, cap(15.0).min(cap(21.0) - 1.0), cap(21.0), 2, 2, false, false, ResourceKind::Gold, 800, "far-gold", w, h,
        grid, clusters, occupied,
    );
}

fn place_central_resources(
    rng: &mut StdRng,
    cx: i32,
    cy: i32,
    w: i32,
    h: i32,
    grid: &[Vec<Tile>],
    clusters: &mut Vec<ResourceCluster>,
    occupied: &mut HashSet<(i32, i32)>,
) {
    // Skip central resources if the map center is water (e.g. Islands)
    if grid[cx as usize][cy as usize].is_water() {
        return;
    }

    if let Some(positions) =
        find_cluster_in_ring(rng, (cx, cy), 0.0, 5.0, 2, 2, false, false, occupied, w, h, grid)
    {
        for &pos in &positions {
            occupied.insert(pos);
        }
        clusters.push(ResourceCluster {
            positions,
            kind: ResourceKind::Gold,
            amount: 800,
        });
    } else {
        warn!("Failed to place central gold");
    }

    if let Some(positions) =
        find_cluster_in_ring(rng, (cx, cy), 0.0, 7.0, 2, 1, false, false, occupied, w, h, grid)
    {
        for &pos in &positions {
            occupied.insert(pos);
        }
        clusters.push(ResourceCluster {
            positions,
            kind: ResourceKind::Stone,
            amount: 400,
        });
    } else {
        warn!("Failed to place central stone");
    }
}

// ── Resource placement helpers ─────────────────────────

fn place_resource(
    rng: &mut StdRng,
    base: (i32, i32),
    min_dist: f32,
    max_dist: f32,
    cols: i32,
    rows: i32,
    allow_forest: bool,
    avoid_water_adj: bool,
    kind: ResourceKind,
    amount: u32,
    label: &str,
    w: i32,
    h: i32,
    grid: &[Vec<Tile>],
    clusters: &mut Vec<ResourceCluster>,
    occupied: &mut HashSet<(i32, i32)>,
) {
    match find_cluster_in_ring(
        rng,
        base,
        min_dist,
        max_dist,
        cols,
        rows,
        allow_forest,
        avoid_water_adj,
        occupied,
        w,
        h,
        grid,
    ) {
        Some(positions) => {
            for &pos in &positions {
                occupied.insert(pos);
            }
            clusters.push(ResourceCluster {
                positions,
                kind,
                amount,
            });
        }
        None => warn!("Failed to place {label} resource cluster"),
    }
}

fn find_cluster_in_ring(
    rng: &mut StdRng,
    base: (i32, i32),
    min_dist: f32,
    max_dist: f32,
    cols: i32,
    rows: i32,
    allow_forest: bool,
    avoid_water_adjacent: bool,
    occupied: &HashSet<(i32, i32)>,
    w: i32,
    h: i32,
    grid: &[Vec<Tile>],
) -> Option<Vec<(i32, i32)>> {
    const N: usize = 24;
    let (bx, by) = base;
    let base_angle: f32 = rng.random_range(0.0..std::f32::consts::TAU);

    for pass in 0..2u32 {
        let hi = max_dist * if pass == 0 { 1.0 } else { 1.25 };
        for i in 0..N {
            let angle = base_angle + (i as f32) * std::f32::consts::TAU / N as f32;
            let dist: f32 = rng.random_range(min_dist..hi);
            let cx = bx + (dist * angle.cos()) as i32;
            let cy = by + (dist * angle.sin()) as i32;

            let positions = cluster_at(cx, cy, cols, rows, w, h, grid, allow_forest, avoid_water_adjacent);
            if positions.is_empty() {
                continue;
            }
            if positions.iter().any(|p| occupied.contains(p)) {
                continue;
            }
            return Some(positions);
        }
    }
    None
}

fn cluster_at(
    cx: i32,
    cy: i32,
    cols: i32,
    rows: i32,
    w: i32,
    h: i32,
    grid: &[Vec<Tile>],
    allow_forest: bool,
    avoid_trunk_on_water: bool,
) -> Vec<(i32, i32)> {
    let mut positions = Vec::new();
    let expected = (cols * rows) as usize;
    for dx in 0..cols {
        for dy in 0..rows {
            let x = (cx + dx).clamp(1, w - 2);
            let y = (cy + dy).clamp(1, h - 2);
            if (x as usize) >= grid.len() || (y as usize) >= grid[0].len() {
                return Vec::new();
            }
            let tile = &grid[x as usize][y as usize];
            if tile.is_water() {
                return Vec::new();
            }
            if !allow_forest && tile.terrain == TerrainType::DarkGrass {
                return Vec::new();
            }
            if avoid_trunk_on_water && trunk_overlaps_water(x, y, w, h, grid) {
                return Vec::new();
            }
            positions.push((x, y));
        }
    }
    if positions.len() != expected {
        return Vec::new();
    }
    positions
}

fn trunk_overlaps_water(x: i32, y: i32, w: i32, h: i32, grid: &[Vec<Tile>]) -> bool {
    for dx in -2..=1 {
        for dy in -2..=1 {
            if dx == 0 && dy == 0 {
            } else if dx + dy >= 2 {
                continue;
            }
            let nx = x + dx;
            let ny = y + dy;
            if nx >= 0 && nx < w && ny >= 0 && ny < h && grid[nx as usize][ny as usize].is_water()
            {
                return true;
            }
        }
    }
    false
}


// ── Public utility ─────────────────────────────────────

pub fn building_footprint_has_water(
    anchor: GridPosition,
    tw: u32,
    th: u32,
    terrain_grid: &[Vec<Tile>],
) -> bool {
    let w = terrain_grid.len() as i32;
    let h = if terrain_grid.is_empty() {
        0
    } else {
        terrain_grid[0].len() as i32
    };
    let hw = tw as i32 / 2;
    let hh = th as i32 / 2;
    for dx in -hw..(tw as i32 - hw) {
        for dy in -hh..(th as i32 - hh) {
            let x = anchor.x + dx;
            let y = anchor.y + dy;
            if x >= 0 && x < w && y >= 0 && y < h {
                if terrain_grid[x as usize][y as usize].is_water() {
                    return true;
                }
            }
        }
    }
    false
}

/// Nudge a building anchor position onto land if its footprint overlaps water.
pub fn nudge_building_onto_land(
    anchor: super::GridPosition,
    tw: u32,
    th: u32,
    terrain_grid: &[Vec<Tile>],
) -> super::GridPosition {
    if !building_footprint_has_water(anchor, tw, th, terrain_grid) {
        return anchor;
    }
    for radius in 1_i32..=6 {
        for dx in -radius..=radius {
            for dy in -radius..=radius {
                if dx.abs() != radius && dy.abs() != radius {
                    continue;
                }
                let candidate = super::GridPosition::new(anchor.x + dx, anchor.y + dy);
                if !building_footprint_has_water(candidate, tw, th, terrain_grid) {
                    return candidate;
                }
            }
        }
    }
    anchor
}

/// Spiral outward from (x, y) to find the nearest non-water tile.
/// Returns the original position if it's already on land.
pub fn find_nearest_land(
    terrain_grid: &[Vec<Tile>],
    x: i32,
    y: i32,
) -> (i32, i32) {
    let w = terrain_grid.len() as i32;
    let h = if terrain_grid.is_empty() { 0 } else { terrain_grid[0].len() as i32 };
    if x >= 0 && x < w && y >= 0 && y < h && !terrain_grid[x as usize][y as usize].is_water() {
        return (x, y);
    }
    for radius in 1_i32..=20 {
        for dx in -radius..=radius {
            for dy in -radius..=radius {
                if dx.abs() != radius && dy.abs() != radius {
                    continue;
                }
                let nx = x + dx;
                let ny = y + dy;
                if nx >= 0 && nx < w && ny >= 0 && ny < h
                    && !terrain_grid[nx as usize][ny as usize].is_water()
                {
                    return (nx, ny);
                }
            }
        }
    }
    (x, y)
}
