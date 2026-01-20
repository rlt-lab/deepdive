//! Map unit tests for Phase 2: Map Core (TDD)
//!
//! Tests for GameMap functionality including index calculation,
//! tile operations, walkability, and helper methods.

use bevy::math::Vec2;
use deepdive::components::TileType;
use deepdive::constants::{MAP_HEIGHT, MAP_WIDTH, TILE_SIZE};
use deepdive::map::GameMap;

// =============================================================================
// 2.1.1 - Index Calculation Tests
// =============================================================================

/// Test that valid coordinates produce correct indices.
#[test]
fn idx_valid_coords_produces_correct_index() {
    let map = GameMap::new(MAP_WIDTH, MAP_HEIGHT);

    // Test via get/set round-trip (idx is private)
    // Index formula: y * width + x

    // (0, 0) should access index 0
    let mut map = map;
    map.set(0, 0, TileType::Floor);
    assert_eq!(map.get(0, 0), TileType::Floor);

    // (1, 0) should access index 1
    map.set(1, 0, TileType::StairUp);
    assert_eq!(map.get(1, 0), TileType::StairUp);

    // (0, 1) should access index MAP_WIDTH
    map.set(0, 1, TileType::StairDown);
    assert_eq!(map.get(0, 1), TileType::StairDown);
}

/// Test edge case at origin (0, 0).
#[test]
fn idx_origin_edge_case() {
    let mut map = GameMap::new(MAP_WIDTH, MAP_HEIGHT);

    // Origin should be accessible
    map.set(0, 0, TileType::Water);
    assert_eq!(map.get(0, 0), TileType::Water);

    // Verify it doesn't affect adjacent tiles
    assert_eq!(map.get(1, 0), TileType::Wall); // Default is Wall
    assert_eq!(map.get(0, 1), TileType::Wall);
}

/// Test boundary coordinates at map edge.
#[test]
fn idx_boundary_max_coords() {
    let mut map = GameMap::new(MAP_WIDTH, MAP_HEIGHT);
    let max_x = MAP_WIDTH - 1;
    let max_y = MAP_HEIGHT - 1;

    // Maximum valid coordinates
    map.set(max_x, max_y, TileType::Floor);
    assert_eq!(map.get(max_x, max_y), TileType::Floor);

    // Edge boundaries
    map.set(max_x, 0, TileType::StairUp);
    assert_eq!(map.get(max_x, 0), TileType::StairUp);

    map.set(0, max_y, TileType::StairDown);
    assert_eq!(map.get(0, max_y), TileType::StairDown);
}

/// Test that index calculation is consistent with width.
#[test]
fn idx_calculation_matches_expected_formula() {
    let map = GameMap::new(MAP_WIDTH, MAP_HEIGHT);

    // Create a map and verify specific positions
    let mut map = map;

    // Set tiles at calculated positions and verify
    // Position (5, 3) should be at index 3 * MAP_WIDTH + 5
    map.set(5, 3, TileType::Floor);
    assert_eq!(map.get(5, 3), TileType::Floor);

    // Position (MAP_WIDTH - 1, 0) should be at index MAP_WIDTH - 1
    map.set(MAP_WIDTH - 1, 0, TileType::Water);
    assert_eq!(map.get(MAP_WIDTH - 1, 0), TileType::Water);
}

// =============================================================================
// 2.1.2 - Tile Operations Tests (get/set)
// =============================================================================

/// Test round-trip correctness for Floor type.
#[test]
fn tile_operations_floor_round_trip() {
    let mut map = GameMap::new(MAP_WIDTH, MAP_HEIGHT);
    map.set(10, 10, TileType::Floor);
    assert_eq!(map.get(10, 10), TileType::Floor);
}

/// Test round-trip correctness for Wall type.
#[test]
fn tile_operations_wall_round_trip() {
    let mut map = GameMap::new(MAP_WIDTH, MAP_HEIGHT);
    // Wall is default, but test explicit set
    map.set(10, 10, TileType::Floor);
    map.set(10, 10, TileType::Wall);
    assert_eq!(map.get(10, 10), TileType::Wall);
}

/// Test round-trip correctness for Water type.
#[test]
fn tile_operations_water_round_trip() {
    let mut map = GameMap::new(MAP_WIDTH, MAP_HEIGHT);
    map.set(15, 20, TileType::Water);
    assert_eq!(map.get(15, 20), TileType::Water);
}

/// Test round-trip correctness for StairUp type.
#[test]
fn tile_operations_stair_up_round_trip() {
    let mut map = GameMap::new(MAP_WIDTH, MAP_HEIGHT);
    map.set(25, 30, TileType::StairUp);
    assert_eq!(map.get(25, 30), TileType::StairUp);
}

/// Test round-trip correctness for StairDown type.
#[test]
fn tile_operations_stair_down_round_trip() {
    let mut map = GameMap::new(MAP_WIDTH, MAP_HEIGHT);
    map.set(35, 40, TileType::StairDown);
    assert_eq!(map.get(35, 40), TileType::StairDown);
}

/// Test that set overwrites previous value.
#[test]
fn tile_operations_overwrite_previous() {
    let mut map = GameMap::new(MAP_WIDTH, MAP_HEIGHT);

    map.set(5, 5, TileType::Floor);
    assert_eq!(map.get(5, 5), TileType::Floor);

    map.set(5, 5, TileType::Water);
    assert_eq!(map.get(5, 5), TileType::Water);

    map.set(5, 5, TileType::StairUp);
    assert_eq!(map.get(5, 5), TileType::StairUp);
}

/// Test that new map has all walls by default.
#[test]
fn tile_operations_default_is_wall() {
    let map = GameMap::new(MAP_WIDTH, MAP_HEIGHT);

    // Check several random positions
    assert_eq!(map.get(0, 0), TileType::Wall);
    assert_eq!(map.get(40, 25), TileType::Wall);
    assert_eq!(map.get(MAP_WIDTH - 1, MAP_HEIGHT - 1), TileType::Wall);
}

// =============================================================================
// 2.1.3 - Walkability Tests
// =============================================================================

/// Test that Floor is walkable.
#[test]
fn is_walkable_floor_returns_true() {
    assert!(TileType::Floor.is_walkable());
}

/// Test that Wall is not walkable.
#[test]
fn is_walkable_wall_returns_false() {
    assert!(!TileType::Wall.is_walkable());
}

/// Test that Water is not walkable.
#[test]
fn is_walkable_water_returns_false() {
    assert!(!TileType::Water.is_walkable());
}

/// Test that StairUp is walkable.
#[test]
fn is_walkable_stair_up_returns_true() {
    assert!(TileType::StairUp.is_walkable());
}

/// Test that StairDown is walkable.
#[test]
fn is_walkable_stair_down_returns_true() {
    assert!(TileType::StairDown.is_walkable());
}

// =============================================================================
// 2.1.4 - Floor Positions Tests
// =============================================================================

/// Test that get_floor_positions returns empty for all-wall map.
#[test]
fn get_floor_positions_empty_for_all_walls() {
    let map = GameMap::new(MAP_WIDTH, MAP_HEIGHT);
    let positions = map.get_floor_positions();
    assert!(positions.is_empty());
}

/// Test that get_floor_positions returns correct single floor.
#[test]
fn get_floor_positions_single_floor() {
    let mut map = GameMap::new(MAP_WIDTH, MAP_HEIGHT);
    map.set(10, 10, TileType::Floor);

    let positions = map.get_floor_positions();
    assert_eq!(positions.len(), 1);
    assert!(positions.contains(&(10, 10)));
}

/// Test that get_floor_positions returns all floor tiles.
#[test]
fn get_floor_positions_multiple_floors() {
    let mut map = GameMap::new(MAP_WIDTH, MAP_HEIGHT);
    map.set(5, 5, TileType::Floor);
    map.set(10, 10, TileType::Floor);
    map.set(15, 15, TileType::Floor);
    map.set(20, 20, TileType::Floor);

    let positions = map.get_floor_positions();
    assert_eq!(positions.len(), 4);
    assert!(positions.contains(&(5, 5)));
    assert!(positions.contains(&(10, 10)));
    assert!(positions.contains(&(15, 15)));
    assert!(positions.contains(&(20, 20)));
}

/// Test that get_floor_positions excludes non-floor types.
#[test]
fn get_floor_positions_excludes_non_floors() {
    let mut map = GameMap::new(MAP_WIDTH, MAP_HEIGHT);
    map.set(5, 5, TileType::Floor);
    map.set(10, 10, TileType::Water);
    map.set(15, 15, TileType::StairUp);
    map.set(20, 20, TileType::StairDown);
    map.set(25, 25, TileType::Floor);

    let positions = map.get_floor_positions();
    assert_eq!(positions.len(), 2); // Only Floor tiles
    assert!(positions.contains(&(5, 5)));
    assert!(positions.contains(&(25, 25)));
    assert!(!positions.contains(&(10, 10))); // Water
    assert!(!positions.contains(&(15, 15))); // StairUp
    assert!(!positions.contains(&(20, 20))); // StairDown
}

// =============================================================================
// 2.1.5 - Grid-to-World Conversion Tests
// =============================================================================

/// Test grid_to_world at origin.
#[test]
fn grid_to_world_origin() {
    let map = GameMap::new(MAP_WIDTH, MAP_HEIGHT);
    let world_pos = map.grid_to_world(0, 0);

    // Origin grid (0,0) should map to world position considering map centering
    let expected_x = (0.0 - (MAP_WIDTH as f32 / 2.0 - 0.5)) * TILE_SIZE;
    let expected_y = (0.0 - (MAP_HEIGHT as f32 / 2.0 - 0.5)) * TILE_SIZE;

    assert!((world_pos.x - expected_x).abs() < 0.001);
    assert!((world_pos.y - expected_y).abs() < 0.001);
}

/// Test grid_to_world at map center.
#[test]
fn grid_to_world_center() {
    let map = GameMap::new(MAP_WIDTH, MAP_HEIGHT);
    let center_x = MAP_WIDTH / 2;
    let center_y = MAP_HEIGHT / 2;
    let world_pos = map.grid_to_world(center_x, center_y);

    // Center of map should be near world origin
    let expected_x = (center_x as f32 - (MAP_WIDTH as f32 / 2.0 - 0.5)) * TILE_SIZE;
    let expected_y = (center_y as f32 - (MAP_HEIGHT as f32 / 2.0 - 0.5)) * TILE_SIZE;

    assert!((world_pos.x - expected_x).abs() < 0.001);
    assert!((world_pos.y - expected_y).abs() < 0.001);
}

/// Test grid_to_world at map edge.
#[test]
fn grid_to_world_max_boundary() {
    let map = GameMap::new(MAP_WIDTH, MAP_HEIGHT);
    let max_x = MAP_WIDTH - 1;
    let max_y = MAP_HEIGHT - 1;
    let world_pos = map.grid_to_world(max_x, max_y);

    let expected_x = (max_x as f32 - (MAP_WIDTH as f32 / 2.0 - 0.5)) * TILE_SIZE;
    let expected_y = (max_y as f32 - (MAP_HEIGHT as f32 / 2.0 - 0.5)) * TILE_SIZE;

    assert!((world_pos.x - expected_x).abs() < 0.001);
    assert!((world_pos.y - expected_y).abs() < 0.001);
}

/// Test grid_to_world spacing is consistent with TILE_SIZE.
#[test]
fn grid_to_world_tile_spacing() {
    let map = GameMap::new(MAP_WIDTH, MAP_HEIGHT);

    let pos1 = map.grid_to_world(10, 10);
    let pos2 = map.grid_to_world(11, 10);
    let pos3 = map.grid_to_world(10, 11);

    // Horizontal spacing should be TILE_SIZE
    assert!((pos2.x - pos1.x - TILE_SIZE).abs() < 0.001);
    assert!((pos2.y - pos1.y).abs() < 0.001);

    // Vertical spacing should be TILE_SIZE
    assert!((pos3.x - pos1.x).abs() < 0.001);
    assert!((pos3.y - pos1.y - TILE_SIZE).abs() < 0.001);
}

// =============================================================================
// 2.1.6 - Find Nearby Floor Tests
// =============================================================================

/// Test find_nearby_floor returns center if center is floor.
#[test]
fn find_nearby_floor_returns_center_if_floor() {
    let mut map = GameMap::new(MAP_WIDTH, MAP_HEIGHT);
    map.set(10, 10, TileType::Floor);

    let result = map.find_nearby_floor(10, 10, 5);
    assert_eq!(result, Some((10, 10)));
}

/// Test find_nearby_floor searches outward when center is not floor.
#[test]
fn find_nearby_floor_searches_outward() {
    let mut map = GameMap::new(MAP_WIDTH, MAP_HEIGHT);
    // Center is wall, but adjacent tile is floor
    map.set(11, 10, TileType::Floor);

    let result = map.find_nearby_floor(10, 10, 5);
    assert!(result.is_some());
    let (x, y) = result.unwrap();
    // Should find the floor at (11, 10)
    assert_eq!(map.get(x, y), TileType::Floor);
}

/// Test find_nearby_floor returns None if no floor within radius.
#[test]
fn find_nearby_floor_returns_none_when_no_floor() {
    let map = GameMap::new(MAP_WIDTH, MAP_HEIGHT); // All walls

    let result = map.find_nearby_floor(10, 10, 3);
    assert!(result.is_none());
}

/// Test find_nearby_floor respects max_radius.
#[test]
fn find_nearby_floor_respects_radius() {
    let mut map = GameMap::new(MAP_WIDTH, MAP_HEIGHT);
    // Place floor just outside radius
    map.set(20, 10, TileType::Floor); // 10 tiles away from (10, 10)

    let result = map.find_nearby_floor(10, 10, 5);
    assert!(result.is_none()); // Floor is outside radius

    let result = map.find_nearby_floor(10, 10, 15);
    assert!(result.is_some()); // Floor is now within radius
}

/// Test find_nearby_floor finds nearest floor.
#[test]
fn find_nearby_floor_finds_nearest() {
    let mut map = GameMap::new(MAP_WIDTH, MAP_HEIGHT);
    map.set(12, 10, TileType::Floor); // Distance 2
    map.set(15, 10, TileType::Floor); // Distance 5

    let result = map.find_nearby_floor(10, 10, 10);
    assert!(result.is_some());
    let (x, _y) = result.unwrap();
    // Should find the closer floor at distance 2
    assert_eq!(x, 12);
}

/// Test find_nearby_floor handles edge of map.
#[test]
fn find_nearby_floor_handles_map_edge() {
    let mut map = GameMap::new(MAP_WIDTH, MAP_HEIGHT);
    map.set(1, 1, TileType::Floor);

    let result = map.find_nearby_floor(0, 0, 5);
    assert!(result.is_some());
    assert_eq!(result, Some((1, 1)));
}
