use super::generation::TerrainOverride;

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
        let variant = (x.wrapping_mul(17) ^ y.wrapping_mul(31))
            .wrapping_add(x.wrapping_add(y));
        self.base_index() + (variant % 16)
    }

    pub fn is_walkable(self) -> bool {
        !matches!(self, TerrainType::Water)
    }

    pub fn for_position(x: u32, y: u32, _seed: u64, overrides: &[Vec<TerrainOverride>]) -> Self {
        if (x as usize) < overrides.len() && (y as usize) < overrides[0].len() {
            match overrides[x as usize][y as usize] {
                TerrainOverride::ForceGrass | TerrainOverride::None => TerrainType::Grass,
                TerrainOverride::ForceForest => TerrainType::DarkGrass,
                TerrainOverride::ForceDirt => TerrainType::Dirt,
                TerrainOverride::ForceWater => TerrainType::Water,
            }
        } else {
            TerrainType::Grass
        }
    }
}
