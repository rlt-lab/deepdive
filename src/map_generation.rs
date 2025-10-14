// Map Generation Module - Compact Organic Algorithm
use rand::RngCore;
use crate::components::TileType;
use crate::biome::BiomeType;

/// Map generation parameters that control the generation algorithm
#[derive(Clone, Debug)]
pub struct MapGenParams {
    pub max_rooms: u32,
}

impl MapGenParams {
    /// Get generation parameters for a specific biome
    pub fn for_biome(_biome: BiomeType, level: u32) -> Self {
        // All biomes use the same compact organic generation
        // max_rooms controls number of interior wall divisions (2-4)
        Self {
            max_rooms: 3 + (level / 5).min(2), // 3-5 divisions based on level
        }
    }
}

/// Trait for map generators (using RngCore which is dyn-safe)
pub trait MapGenerator {
    fn generate(&mut self, width: u32, height: u32, params: &MapGenParams, rng: &mut dyn RngCore) -> Vec<TileType>;
}

/// Get the map generator instance
pub fn get_generator() -> Box<dyn MapGenerator> {
    Box::new(crate::map_generation_compact::CompactOrganicGenerator)
}

/// Helper function to convert 2D Vec to flat Vec
pub fn flatten_tiles(tiles_2d: Vec<Vec<TileType>>, width: u32, height: u32) -> Vec<TileType> {
    let mut flat = vec![TileType::Wall; (width * height) as usize];
    for y in 0..height {
        for x in 0..width {
            flat[(y * width + x) as usize] = tiles_2d[y as usize][x as usize];
        }
    }
    flat
}
