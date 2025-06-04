use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum BiomeType {
    Caverns,
    Underglade,
    FungalDeep,
    CinderGaol,
    AbyssalHold,
    NetherGrange,
    ChthronicCrypts,
    HypogealKnot,
    StygianPool,
}

#[derive(Clone, Debug)]
pub struct BiomeConfig {
    pub name: String,
    pub description: String,
    pub allowed_floor_assets: Vec<(u32, u32)>,
    pub allowed_wall_assets: Vec<(u32, u32)>,
    pub allowed_water_assets: Vec<(u32, u32)>,
    pub allowed_stair_assets: Vec<(u32, u32)>,
}

impl BiomeType {
    pub fn get_config(&self) -> BiomeConfig {
        match self {
            BiomeType::Caverns => BiomeConfig {
                name: "Caverns".to_string(),
                description: "Natural underground caves with rough stone walls, frequent water features, and occasional crystal formations.".to_string(),
                allowed_floor_assets: vec![(0,6), (1,6), (2,6), (3,6)],
                allowed_wall_assets: vec![(0,0), (1,0), (0,1), (1,1)],
                allowed_water_assets: vec![(0,6), (1,6), (2,6), (3,6)],
                allowed_stair_assets: vec![(7,16), (8,16)],
            },
            BiomeType::Underglade => BiomeConfig {
                name: "Underglade".to_string(),
                description: "Subterranean forest space with mossy ground, glowing flora, and primitive plant life.".to_string(),
                allowed_floor_assets: vec![
                    (1,7), (2,7), (3,7),        // Grass floors 
                    (0,13), (1,13), (2,13), (3,13), // Green dirt floors
                    (1,14), (2,14), (3,14),    // Green grass floors
                ],
                allowed_wall_assets: vec![(0,0), (1,0), (0,1), (1,1)], // Dirt and rough stone walls
                allowed_water_assets: vec![(0,6)], // Dark grey water/blank floor for water areas
                allowed_stair_assets: vec![(7,16), (8,16)], // Standard staircase assets
            },
            BiomeType::FungalDeep => BiomeConfig {
                name: "Fungal Deep".to_string(),
                description: "An eerie, spore-filled biome dominated by giant mushrooms and fungal growths, with a damp, earthy floor.".to_string(),
                allowed_floor_assets: vec![(4,0), (5,0), (4,1), (5,1)],
                allowed_wall_assets: vec![(1,7), (2,7), (3,7)],
                allowed_water_assets: vec![(0,6), (1,6), (2,6), (3,6)],
                allowed_stair_assets: vec![(1,8), (2,8), (3,8)],
            },
            BiomeType::CinderGaol => BiomeConfig {
                name: "Cinder Gaol".to_string(),
                description: "Ancient prison complex with charred stone walls and abandoned cells, now home to malevolent spirits.".to_string(),
                allowed_floor_assets: vec![
                    (0,15), (1,15), (2,15), (3,15), // Dark brown and bone floors for prison
                    (0,11), (1,11), (2,11), (3,11), // Red floors for fire/brimstone theme
                ], 
                allowed_wall_assets: vec![(0,3), (1,3), (0,5), (1,5)], // Igneous and catacombs walls only
                allowed_water_assets: vec![], // No water in prison
                allowed_stair_assets: vec![(7,16), (8,16)], // Standard staircase assets
            },
            BiomeType::AbyssalHold => BiomeConfig {
                name: "Abyssal Hold".to_string(),
                description: "A mysterious and foreboding biome, with a floor of smooth, dark stone, and walls that seem to absorb light, making the area dim and shadowy.".to_string(),
                allowed_floor_assets: vec![(8,0), (9,0), (8,1), (9,1)],
                allowed_wall_assets: vec![(1,7), (2,7), (3,7)],
                allowed_water_assets: vec![(0,6), (1,6), (2,6), (3,6)],
                allowed_stair_assets: vec![(1,8), (2,8), (3,8)],
            },
            BiomeType::NetherGrange => BiomeConfig {
                name: "Nether Grange".to_string(),
                description: "A hellish landscape of fire and brimstone, with a floor of cracked, blackened earth, and walls of molten rock and flame.".to_string(),
                allowed_floor_assets: vec![(10,0), (11,0), (10,1), (11,1)],
                allowed_wall_assets: vec![(1,7), (2,7), (3,7)],
                allowed_water_assets: vec![(0,6), (1,6), (2,6), (3,6)],
                allowed_stair_assets: vec![(1,8), (2,8), (3,8)],
            },
            BiomeType::ChthronicCrypts => BiomeConfig {
                name: "Chthonic Crypts".to_string(),
                description: "Ancient, underground burial grounds, with a floor of packed dirt and stone, and walls lined with tombs and sarcophagi.".to_string(),
                allowed_floor_assets: vec![(12,0), (13,0), (12,1), (13,1)],
                allowed_wall_assets: vec![(1,7), (2,7), (3,7)],
                allowed_water_assets: vec![(0,6), (1,6), (2,6), (3,6)],
                allowed_stair_assets: vec![(1,8), (2,8), (3,8)],
            },
            BiomeType::HypogealKnot => BiomeConfig {
                name: "Hypogeal Knot".to_string(),
                description: "A complex and confusing network of underground tunnels and chambers, with a floor of rough stone and dirt, and walls that seem to shift and change.".to_string(),
                allowed_floor_assets: vec![(14,0), (15,0), (14,1), (15,1)],
                allowed_wall_assets: vec![(1,7), (2,7), (3,7)],
                allowed_water_assets: vec![(0,6), (1,6), (2,6), (3,6)],
                allowed_stair_assets: vec![(1,8), (2,8), (3,8)],
            },
            BiomeType::StygianPool => BiomeConfig {
                name: "Stygian Pool".to_string(),
                description: "A dark and ominous underground lake, with a floor of smooth stone and walls that are slick with moisture, reflecting the faintest light.".to_string(),
                allowed_floor_assets: vec![(16,0), (17,0), (16,1), (17,1)],
                allowed_wall_assets: vec![(1,7), (2,7), (3,7)],
                allowed_water_assets: vec![(0,6), (1,6), (2,6), (3,6)],
                allowed_stair_assets: vec![(1,8), (2,8), (3,8)],
            },
        }
    }
}