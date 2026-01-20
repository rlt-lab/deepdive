//! Integration Tests for Level Transitions (Phase 8.1.1-8.1.2)
//!
//! Tests for map preservation across level changes and stair positioning.

mod common;

use std::collections::HashMap;
use deepdive::biome::BiomeType;
use deepdive::components::{EllipseMask, LevelMaps, SavedMapData, TileType, TileVisibility};
use deepdive::constants::{MAP_HEIGHT, MAP_WIDTH};
use deepdive::map::GameMap;
use rand::rngs::StdRng;
use rand::SeedableRng;

// =============================================================================
// 8.1.1 - Map Preservation Across Level Changes
// =============================================================================

/// Test that SavedMapData correctly stores all map properties.
#[test]
fn saved_map_data_preserves_tiles() {
    let mut map = GameMap::new(MAP_WIDTH, MAP_HEIGHT);
    let ellipse_mask = EllipseMask::new(MAP_WIDTH, MAP_HEIGHT);
    let mut rng = StdRng::seed_from_u64(12345);

    map.generate_with_biome(BiomeType::Caverns, 1, &mut rng, &ellipse_mask);

    // Create SavedMapData
    let saved = SavedMapData {
        width: map.width,
        height: map.height,
        tiles: map.tiles.clone(),
        stair_up_pos: map.stair_up_pos,
        stair_down_pos: map.stair_down_pos,
        biome: BiomeType::Caverns,
        tile_visibility: HashMap::new(),
    };

    // Verify preservation
    assert_eq!(saved.width, MAP_WIDTH);
    assert_eq!(saved.height, MAP_HEIGHT);
    assert_eq!(saved.tiles.len(), map.tiles.len());
    assert_eq!(saved.tiles, map.tiles);
}

/// Test that SavedMapData preserves visibility state.
#[test]
fn saved_map_data_preserves_visibility() {
    let mut visibility = HashMap::new();
    visibility.insert((10, 10), TileVisibility::Visible);
    visibility.insert((20, 20), TileVisibility::Seen);
    visibility.insert((30, 30), TileVisibility::Unseen);

    let saved = SavedMapData {
        width: MAP_WIDTH,
        height: MAP_HEIGHT,
        tiles: vec![TileType::Floor; (MAP_WIDTH * MAP_HEIGHT) as usize],
        stair_up_pos: Some((5, 5)),
        stair_down_pos: Some((75, 45)),
        biome: BiomeType::Caverns,
        tile_visibility: visibility.clone(),
    };

    assert_eq!(saved.tile_visibility.len(), 3);
    assert_eq!(saved.tile_visibility.get(&(10, 10)), Some(&TileVisibility::Visible));
    assert_eq!(saved.tile_visibility.get(&(20, 20)), Some(&TileVisibility::Seen));
    assert_eq!(saved.tile_visibility.get(&(30, 30)), Some(&TileVisibility::Unseen));
}

/// Test that LevelMaps can store and retrieve multiple levels.
#[test]
fn level_maps_stores_multiple_levels() {
    let mut level_maps = LevelMaps {
        maps: HashMap::new(),
    };

    // Create maps for levels 0, 1, 2
    for level in 0..3 {
        let saved = SavedMapData {
            width: MAP_WIDTH,
            height: MAP_HEIGHT,
            tiles: vec![TileType::Floor; (MAP_WIDTH * MAP_HEIGHT) as usize],
            stair_up_pos: Some((5 + level, 5 + level)),
            stair_down_pos: Some((70 - level, 40 - level)),
            biome: BiomeType::Caverns,
            tile_visibility: HashMap::new(),
        };
        level_maps.maps.insert(level, saved);
    }

    assert_eq!(level_maps.maps.len(), 3);
    assert!(level_maps.maps.contains_key(&0));
    assert!(level_maps.maps.contains_key(&1));
    assert!(level_maps.maps.contains_key(&2));
}

/// Test that returning to a previous level retrieves the correct map.
#[test]
fn level_maps_retrieves_correct_level() {
    let mut level_maps = LevelMaps {
        maps: HashMap::new(),
    };

    // Store level 0 with specific stair position
    let level0_stair = (10, 15);
    level_maps.maps.insert(0, SavedMapData {
        width: MAP_WIDTH,
        height: MAP_HEIGHT,
        tiles: vec![TileType::Floor; (MAP_WIDTH * MAP_HEIGHT) as usize],
        stair_up_pos: Some(level0_stair),
        stair_down_pos: Some((60, 40)),
        biome: BiomeType::Caverns,
        tile_visibility: HashMap::new(),
    });

    // Store level 1 with different stair position
    let level1_stair = (25, 30);
    level_maps.maps.insert(1, SavedMapData {
        width: MAP_WIDTH,
        height: MAP_HEIGHT,
        tiles: vec![TileType::Wall; (MAP_WIDTH * MAP_HEIGHT) as usize], // Different tiles
        stair_up_pos: Some(level1_stair),
        stair_down_pos: Some((50, 35)),
        biome: BiomeType::Underglade, // Different biome
        tile_visibility: HashMap::new(),
    });

    // Retrieve and verify level 0
    let retrieved_0 = level_maps.maps.get(&0).unwrap();
    assert_eq!(retrieved_0.stair_up_pos, Some(level0_stair));
    assert_eq!(retrieved_0.biome, BiomeType::Caverns);
    assert_eq!(retrieved_0.tiles[0], TileType::Floor);

    // Retrieve and verify level 1
    let retrieved_1 = level_maps.maps.get(&1).unwrap();
    assert_eq!(retrieved_1.stair_up_pos, Some(level1_stair));
    assert_eq!(retrieved_1.biome, BiomeType::Underglade);
    assert_eq!(retrieved_1.tiles[0], TileType::Wall);
}

/// Test map dimensions are preserved after save/load cycle.
#[test]
fn map_dimensions_preserved_after_save_load() {
    let mut map = GameMap::new(MAP_WIDTH, MAP_HEIGHT);
    let ellipse_mask = EllipseMask::new(MAP_WIDTH, MAP_HEIGHT);
    let mut rng = StdRng::seed_from_u64(99999);

    map.generate_with_biome(BiomeType::FungalDeep, 3, &mut rng, &ellipse_mask);

    // Save
    let saved = SavedMapData {
        width: map.width,
        height: map.height,
        tiles: map.tiles.clone(),
        stair_up_pos: map.stair_up_pos,
        stair_down_pos: map.stair_down_pos,
        biome: BiomeType::FungalDeep,
        tile_visibility: HashMap::new(),
    };

    // Simulate "load" by creating new map from saved data
    let mut restored_map = GameMap::new(saved.width, saved.height);
    restored_map.tiles = saved.tiles.clone();
    restored_map.stair_up_pos = saved.stair_up_pos;
    restored_map.stair_down_pos = saved.stair_down_pos;

    assert_eq!(restored_map.width, map.width);
    assert_eq!(restored_map.height, map.height);
    assert_eq!(restored_map.tiles.len(), map.tiles.len());
}

// =============================================================================
// 8.1.2 - Stair Positioning Correct After Transition
// =============================================================================

/// Test that stair positions are correctly stored and retrieved.
#[test]
fn stair_positions_preserved_in_saved_data() {
    let up_pos = (15, 20);
    let down_pos = (65, 35);

    let saved = SavedMapData {
        width: MAP_WIDTH,
        height: MAP_HEIGHT,
        tiles: vec![TileType::Floor; (MAP_WIDTH * MAP_HEIGHT) as usize],
        stair_up_pos: Some(up_pos),
        stair_down_pos: Some(down_pos),
        biome: BiomeType::Caverns,
        tile_visibility: HashMap::new(),
    };

    assert_eq!(saved.stair_up_pos, Some(up_pos));
    assert_eq!(saved.stair_down_pos, Some(down_pos));
}

/// Test that stair positions match tile types in the saved map.
#[test]
fn stair_positions_match_tile_types() {
    let mut map = GameMap::new(MAP_WIDTH, MAP_HEIGHT);
    let ellipse_mask = EllipseMask::new(MAP_WIDTH, MAP_HEIGHT);
    let mut rng = StdRng::seed_from_u64(54321);

    map.generate_with_biome(BiomeType::Caverns, 1, &mut rng, &ellipse_mask);

    // Place stairs manually for testing
    let up_pos = (10, 10);
    let down_pos = (70, 40);
    map.set(up_pos.0, up_pos.1, TileType::StairUp);
    map.set(down_pos.0, down_pos.1, TileType::StairDown);
    map.stair_up_pos = Some(up_pos);
    map.stair_down_pos = Some(down_pos);

    // Save and verify
    let saved = SavedMapData {
        width: map.width,
        height: map.height,
        tiles: map.tiles.clone(),
        stair_up_pos: map.stair_up_pos,
        stair_down_pos: map.stair_down_pos,
        biome: BiomeType::Caverns,
        tile_visibility: HashMap::new(),
    };

    // Verify stair positions correspond to correct tile types
    if let Some((x, y)) = saved.stair_up_pos {
        let idx = (y * saved.width + x) as usize;
        assert_eq!(saved.tiles[idx], TileType::StairUp);
    }

    if let Some((x, y)) = saved.stair_down_pos {
        let idx = (y * saved.width + x) as usize;
        assert_eq!(saved.tiles[idx], TileType::StairDown);
    }
}

/// Test that stairs are within map bounds.
#[test]
fn stair_positions_within_bounds() {
    let saved = SavedMapData {
        width: MAP_WIDTH,
        height: MAP_HEIGHT,
        tiles: vec![TileType::Floor; (MAP_WIDTH * MAP_HEIGHT) as usize],
        stair_up_pos: Some((10, 10)),
        stair_down_pos: Some((70, 40)),
        biome: BiomeType::Caverns,
        tile_visibility: HashMap::new(),
    };

    if let Some((x, y)) = saved.stair_up_pos {
        assert!(x < MAP_WIDTH, "Stair up x out of bounds");
        assert!(y < MAP_HEIGHT, "Stair up y out of bounds");
    }

    if let Some((x, y)) = saved.stair_down_pos {
        assert!(x < MAP_WIDTH, "Stair down x out of bounds");
        assert!(y < MAP_HEIGHT, "Stair down y out of bounds");
    }
}

/// Test level transition preserves both stair positions for bidirectional travel.
#[test]
fn level_transition_preserves_both_stairs() {
    let mut level_maps = LevelMaps {
        maps: HashMap::new(),
    };

    // Level 0: down stairs at (50, 30)
    let level0_down = (50, 30);
    level_maps.maps.insert(0, SavedMapData {
        width: MAP_WIDTH,
        height: MAP_HEIGHT,
        tiles: vec![TileType::Floor; (MAP_WIDTH * MAP_HEIGHT) as usize],
        stair_up_pos: None, // No up stairs on level 0
        stair_down_pos: Some(level0_down),
        biome: BiomeType::Caverns,
        tile_visibility: HashMap::new(),
    });

    // Level 1: up stairs should be near where player came from
    let level1_up = (50, 30); // Same position as level 0 down stairs
    let level1_down = (20, 40);
    level_maps.maps.insert(1, SavedMapData {
        width: MAP_WIDTH,
        height: MAP_HEIGHT,
        tiles: vec![TileType::Floor; (MAP_WIDTH * MAP_HEIGHT) as usize],
        stair_up_pos: Some(level1_up),
        stair_down_pos: Some(level1_down),
        biome: BiomeType::Underglade,
        tile_visibility: HashMap::new(),
    });

    // Verify level 0 down stairs position
    let level0 = level_maps.maps.get(&0).unwrap();
    assert_eq!(level0.stair_down_pos, Some(level0_down));
    assert_eq!(level0.stair_up_pos, None);

    // Verify level 1 has both stairs
    let level1 = level_maps.maps.get(&1).unwrap();
    assert_eq!(level1.stair_up_pos, Some(level1_up));
    assert_eq!(level1.stair_down_pos, Some(level1_down));
}
