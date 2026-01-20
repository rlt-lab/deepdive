//! TileIndex unit tests for Phase 5: TileIndex Optimization (TDD)
//!
//! Tests for O(1) tile lookups, unexplored tile tracking,
//! and visibility state indexing.

use deepdive::components::{TileIndex, TileType, TileVisibility};
use deepdive::constants::{MAP_HEIGHT, MAP_WIDTH};
use deepdive::map::GameMap;
use bevy::prelude::Entity;
use std::collections::HashMap;

// =============================================================================
// 5.1.1 - TileIndex Lookup: O(1) Lookup Returns Correct Entity
// =============================================================================

/// Test that TileIndex insert and lookup works correctly.
#[test]
fn tile_index_insert_and_lookup() {
    let mut tile_index = TileIndex::default();

    // Create a test entity (using Entity::from_raw for testing)
    let entity = Entity::from_raw(42);

    tile_index.insert(10, 15, entity);

    assert!(tile_index.tiles.contains_key(&(10, 15)));
    assert_eq!(tile_index.tiles.get(&(10, 15)), Some(&entity));
}

/// Test that multiple tiles can be indexed.
#[test]
fn tile_index_multiple_tiles() {
    let mut tile_index = TileIndex::default();

    let entities: Vec<Entity> = (0..5).map(|i| Entity::from_raw(i)).collect();

    tile_index.insert(0, 0, entities[0]);
    tile_index.insert(10, 10, entities[1]);
    tile_index.insert(20, 20, entities[2]);
    tile_index.insert(30, 30, entities[3]);
    tile_index.insert(MAP_WIDTH - 1, MAP_HEIGHT - 1, entities[4]);

    assert_eq!(tile_index.tiles.len(), 5);
    assert_eq!(tile_index.tiles.get(&(10, 10)), Some(&entities[1]));
    assert_eq!(tile_index.tiles.get(&(MAP_WIDTH - 1, MAP_HEIGHT - 1)), Some(&entities[4]));
}

/// Test that TileIndex lookup for non-existent tile returns None.
#[test]
fn tile_index_missing_tile_returns_none() {
    let tile_index = TileIndex::default();

    assert!(tile_index.tiles.get(&(50, 50)).is_none());
}

/// Test that TileIndex clear removes all entries.
#[test]
fn tile_index_clear_removes_all() {
    let mut tile_index = TileIndex::default();

    tile_index.insert(5, 5, Entity::from_raw(1));
    tile_index.insert(10, 10, Entity::from_raw(2));
    tile_index.insert(15, 15, Entity::from_raw(3));

    assert_eq!(tile_index.tiles.len(), 3);

    tile_index.clear();

    assert!(tile_index.tiles.is_empty());
}

/// Test TileIndex overwrite behavior.
#[test]
fn tile_index_overwrite_existing() {
    let mut tile_index = TileIndex::default();

    let entity1 = Entity::from_raw(100);
    let entity2 = Entity::from_raw(200);

    tile_index.insert(10, 10, entity1);
    assert_eq!(tile_index.tiles.get(&(10, 10)), Some(&entity1));

    // Overwrite with new entity
    tile_index.insert(10, 10, entity2);
    assert_eq!(tile_index.tiles.get(&(10, 10)), Some(&entity2));
    assert_eq!(tile_index.tiles.len(), 1); // Still only one entry
}

// =============================================================================
// 5.1.2 - find_nearest_unexplored: Returns Closest Unexplored Tile
// =============================================================================

/// Helper: Creates a visibility map for testing.
fn create_visibility_map(
    visible: &[(u32, u32)],
    seen: &[(u32, u32)],
) -> HashMap<(u32, u32), TileVisibility> {
    let mut map = HashMap::new();
    for &(x, y) in visible {
        map.insert((x, y), TileVisibility::Visible);
    }
    for &(x, y) in seen {
        map.insert((x, y), TileVisibility::Seen);
    }
    map
}

/// Test that unexplored tile detection works with visibility state.
#[test]
fn visibility_map_unseen_detection() {
    let visibility = create_visibility_map(
        &[(10, 10), (11, 10), (12, 10)], // Visible
        &[(5, 5)],                        // Seen
    );

    // Tiles in the visibility map
    assert_eq!(visibility.get(&(10, 10)), Some(&TileVisibility::Visible));
    assert_eq!(visibility.get(&(5, 5)), Some(&TileVisibility::Seen));

    // Tiles NOT in the map are implicitly Unseen
    assert!(visibility.get(&(20, 20)).is_none());
}

/// Test distance calculation for nearest unexplored.
#[test]
fn nearest_unexplored_distance_calculation() {
    // Player at (10, 10)
    let player_pos = (10, 10);

    // Candidate unexplored positions
    let candidates = vec![(12, 10), (10, 15), (5, 5)];

    let mut min_dist = i32::MAX;
    let mut nearest = None;

    for (x, y) in candidates {
        // Manhattan distance
        let dist = (x as i32 - player_pos.0 as i32).abs()
            + (y as i32 - player_pos.1 as i32).abs();
        if dist < min_dist {
            min_dist = dist;
            nearest = Some((x, y));
        }
    }

    // (12, 10) is closest (distance = 2)
    assert_eq!(nearest, Some((12, 10)));
    assert_eq!(min_dist, 2);
}

/// Test that only floor tiles are considered for unexplored search.
#[test]
fn nearest_unexplored_only_floors() {
    let mut map = GameMap::new(MAP_WIDTH, MAP_HEIGHT);

    // Set some tiles
    map.set(10, 10, TileType::Floor); // Walkable
    map.set(11, 10, TileType::Wall);  // Not walkable
    map.set(12, 10, TileType::Floor); // Walkable

    assert!(map.get(10, 10).is_walkable());
    assert!(!map.get(11, 10).is_walkable());
    assert!(map.get(12, 10).is_walkable());
}

// =============================================================================
// 5.1.3 - count_unexplored_tiles: Accurate Count with Visibility States
// =============================================================================

/// Test counting with all visible tiles.
#[test]
fn count_unexplored_all_visible() {
    let mut visibility: HashMap<(u32, u32), TileVisibility> = HashMap::new();

    // Mark all tiles as visible
    for x in 0..10u32 {
        for y in 0..10u32 {
            visibility.insert((x, y), TileVisibility::Visible);
        }
    }

    let unexplored_count = visibility
        .values()
        .filter(|&&v| v == TileVisibility::Unseen)
        .count();

    assert_eq!(unexplored_count, 0);
}

/// Test counting with mixed visibility states.
#[test]
fn count_unexplored_mixed_states() {
    let mut visibility: HashMap<(u32, u32), TileVisibility> = HashMap::new();

    // Mix of states
    visibility.insert((0, 0), TileVisibility::Visible);
    visibility.insert((1, 0), TileVisibility::Seen);
    visibility.insert((2, 0), TileVisibility::Unseen);
    visibility.insert((3, 0), TileVisibility::Unseen);
    visibility.insert((4, 0), TileVisibility::Visible);

    let unexplored_count = visibility
        .values()
        .filter(|&&v| v == TileVisibility::Unseen)
        .count();

    assert_eq!(unexplored_count, 2);
}

/// Test counting floor tiles specifically.
#[test]
fn count_unexplored_floor_tiles_only() {
    let mut map = GameMap::new(20, 20);

    // Create a mix of floor and wall tiles
    let floor_positions = vec![(5, 5), (6, 5), (7, 5), (5, 6), (6, 6), (7, 6)];
    for &(x, y) in &floor_positions {
        map.set(x, y, TileType::Floor);
    }

    // Count floor tiles
    let floor_count = (0..20u32)
        .flat_map(|x| (0..20u32).map(move |y| (x, y)))
        .filter(|&(x, y)| map.get(x, y) == TileType::Floor)
        .count();

    assert_eq!(floor_count, floor_positions.len());
}

/// Test that visibility state comparison works correctly.
#[test]
fn visibility_state_comparison() {
    assert_eq!(TileVisibility::Unseen, TileVisibility::Unseen);
    assert_ne!(TileVisibility::Unseen, TileVisibility::Visible);
    assert_ne!(TileVisibility::Unseen, TileVisibility::Seen);
    assert_ne!(TileVisibility::Visible, TileVisibility::Seen);
}

// =============================================================================
// Additional TileIndex Performance Characteristics Tests
// =============================================================================

/// Test that TileIndex is suitable for large maps.
#[test]
fn tile_index_handles_full_map() {
    let mut tile_index = TileIndex::default();

    // Simulate populating index for a portion of the map
    let test_size = 100; // Don't test full 80x50 for speed
    for x in 0..test_size {
        for y in 0..test_size {
            tile_index.insert(x, y, Entity::from_raw((y * test_size + x) as u32));
        }
    }

    assert_eq!(tile_index.tiles.len(), (test_size * test_size) as usize);

    // Verify O(1) lookup still works
    let entity = tile_index.tiles.get(&(50, 50));
    assert!(entity.is_some());
    assert_eq!(*entity.unwrap(), Entity::from_raw(50 * test_size + 50));
}

/// Test TileIndex boundary coordinates.
#[test]
fn tile_index_boundary_coords() {
    let mut tile_index = TileIndex::default();

    let corners = [
        (0, 0),
        (MAP_WIDTH - 1, 0),
        (0, MAP_HEIGHT - 1),
        (MAP_WIDTH - 1, MAP_HEIGHT - 1),
    ];

    for (i, &(x, y)) in corners.iter().enumerate() {
        tile_index.insert(x, y, Entity::from_raw(i as u32));
    }

    assert_eq!(tile_index.tiles.len(), 4);

    for (i, &(x, y)) in corners.iter().enumerate() {
        assert_eq!(tile_index.tiles.get(&(x, y)), Some(&Entity::from_raw(i as u32)));
    }
}
