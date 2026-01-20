use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

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
    pub name: &'static str,
    pub description: &'static str,
    pub allowed_floor_assets: Vec<(u32, u32)>,
    pub allowed_wall_assets: Vec<(u32, u32)>,
    pub allowed_water_assets: Vec<(u32, u32)>,
}

// Static biome configurations initialized once
static CAVERNS_CONFIG: LazyLock<BiomeConfig> = LazyLock::new(|| BiomeConfig {
    name: "Caverns",
    description: "Natural underground caves with rough stone walls, frequent water features, and occasional crystal formations.",
    allowed_floor_assets: vec![(0,6), (1,6), (2,6), (3,6)],
    allowed_wall_assets: vec![(0,0), (1,0), (0,1), (1,1)],
    allowed_water_assets: vec![(0,6), (1,6), (2,6), (3,6)],
});

static UNDERGLADE_CONFIG: LazyLock<BiomeConfig> = LazyLock::new(|| BiomeConfig {
    name: "Underglade",
    description: "Subterranean forest space with mossy ground, glowing flora, and primitive plant life.",
    allowed_floor_assets: vec![
        (1,7), (2,7), (3,7),        // Grass floors
        (0,13), (1,13), (2,13), (3,13), // Green dirt floors
        (1,14), (2,14), (3,14),    // Green grass floors
    ],
    allowed_wall_assets: vec![(0,0), (1,0), (0,1), (1,1)], // Dirt and rough stone walls
    allowed_water_assets: vec![(0,6)], // Dark grey water/blank floor for water areas
});

static FUNGAL_DEEP_CONFIG: LazyLock<BiomeConfig> = LazyLock::new(|| BiomeConfig {
    name: "Fungal Deep",
    description: "An eerie, spore-filled biome dominated by giant mushrooms and fungal growths, with a damp, earthy floor.",
    allowed_floor_assets: vec![(4,0), (5,0), (4,1), (5,1)],
    allowed_wall_assets: vec![(1,7), (2,7), (3,7)],
    allowed_water_assets: vec![(0,6), (1,6), (2,6), (3,6)],
});

static CINDER_GAOL_CONFIG: LazyLock<BiomeConfig> = LazyLock::new(|| BiomeConfig {
    name: "Cinder Gaol",
    description: "Ancient prison complex with charred stone walls and abandoned cells, now home to malevolent spirits.",
    allowed_floor_assets: vec![
        (0,15), (1,15), (2,15), (3,15), // Dark brown and bone floors for prison
        (0,11), (1,11), (2,11), (3,11), // Red floors for fire/brimstone theme
    ],
    allowed_wall_assets: vec![(0,3), (1,3), (0,5), (1,5)], // Igneous and catacombs walls only
    allowed_water_assets: vec![], // No water in prison
});

static ABYSSAL_HOLD_CONFIG: LazyLock<BiomeConfig> = LazyLock::new(|| BiomeConfig {
    name: "Abyssal Hold",
    description: "A mysterious and foreboding biome, with a floor of smooth, dark stone, and walls that seem to absorb light, making the area dim and shadowy.",
    allowed_floor_assets: vec![(8,0), (9,0), (8,1), (9,1)],
    allowed_wall_assets: vec![(1,7), (2,7), (3,7)],
    allowed_water_assets: vec![(0,6), (1,6), (2,6), (3,6)],
});

static NETHER_GRANGE_CONFIG: LazyLock<BiomeConfig> = LazyLock::new(|| BiomeConfig {
    name: "Nether Grange",
    description: "A hellish landscape of fire and brimstone, with a floor of cracked, blackened earth, and walls of molten rock and flame.",
    allowed_floor_assets: vec![(10,0), (11,0), (10,1), (11,1)],
    allowed_wall_assets: vec![(1,7), (2,7), (3,7)],
    allowed_water_assets: vec![(0,6), (1,6), (2,6), (3,6)],
});

static CHTHONIC_CRYPTS_CONFIG: LazyLock<BiomeConfig> = LazyLock::new(|| BiomeConfig {
    name: "Chthonic Crypts",
    description: "Ancient, underground burial grounds, with a floor of packed dirt and stone, and walls lined with tombs and sarcophagi.",
    allowed_floor_assets: vec![(12,0), (13,0), (12,1), (13,1)],
    allowed_wall_assets: vec![(1,7), (2,7), (3,7)],
    allowed_water_assets: vec![(0,6), (1,6), (2,6), (3,6)],
});

static HYPOGEAL_KNOT_CONFIG: LazyLock<BiomeConfig> = LazyLock::new(|| BiomeConfig {
    name: "Hypogeal Knot",
    description: "A complex and confusing network of underground tunnels and chambers, with a floor of rough stone and dirt, and walls that seem to shift and change.",
    allowed_floor_assets: vec![(14,0), (15,0), (14,1), (15,1)],
    allowed_wall_assets: vec![(1,7), (2,7), (3,7)],
    allowed_water_assets: vec![(0,6), (1,6), (2,6), (3,6)],
});

static STYGIAN_POOL_CONFIG: LazyLock<BiomeConfig> = LazyLock::new(|| BiomeConfig {
    name: "Stygian Pool",
    description: "A dark and ominous underground lake, with a floor of smooth stone and walls that are slick with moisture, reflecting the faintest light.",
    allowed_floor_assets: vec![(16,0), (17,0), (16,1), (17,1)],
    allowed_wall_assets: vec![(1,7), (2,7), (3,7)],
    allowed_water_assets: vec![(0,6), (1,6), (2,6), (3,6)],
});

impl BiomeType {
    pub fn get_config(&self) -> &'static BiomeConfig {
        match self {
            BiomeType::Caverns => &CAVERNS_CONFIG,
            BiomeType::Underglade => &UNDERGLADE_CONFIG,
            BiomeType::FungalDeep => &FUNGAL_DEEP_CONFIG,
            BiomeType::CinderGaol => &CINDER_GAOL_CONFIG,
            BiomeType::AbyssalHold => &ABYSSAL_HOLD_CONFIG,
            BiomeType::NetherGrange => &NETHER_GRANGE_CONFIG,
            BiomeType::ChthronicCrypts => &CHTHONIC_CRYPTS_CONFIG,
            BiomeType::HypogealKnot => &HYPOGEAL_KNOT_CONFIG,
            BiomeType::StygianPool => &STYGIAN_POOL_CONFIG,
        }
    }
}