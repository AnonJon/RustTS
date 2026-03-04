use super::generation::Tile;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TerrainType {
    Grass,
    DarkGrass,
    Dirt,
    Water,
}

impl TerrainType {
    fn base_index(self) -> u32 {
        match self {
            TerrainType::Grass => 0,
            TerrainType::DarkGrass => 16,
            TerrainType::Dirt => 32,
            TerrainType::Water => 48,
        }
    }

    pub fn atlas_index(self, x: u32, y: u32) -> u32 {
        // Squirrel3-style hash for uniform spatial distribution (no visible stripes)
        let mut h = x.wrapping_mul(0xB5297A4D);
        h ^= y.wrapping_mul(0x68E31DA4);
        h = h.wrapping_mul(0x1B56C4E9);
        h ^= h >> 8;
        self.base_index() + (h % 16)
    }

    pub fn is_walkable(self) -> bool {
        !matches!(self, TerrainType::Water)
    }

    pub fn for_position(x: u32, y: u32, grid: &[Vec<Tile>]) -> Self {
        if (x as usize) < grid.len() && (y as usize) < grid[0].len() {
            grid[x as usize][y as usize].terrain
        } else {
            TerrainType::Grass
        }
    }
}
