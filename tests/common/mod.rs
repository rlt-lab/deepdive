//! Common test utilities for Deepdive integration tests.
//!
//! Provides test map builders, assertions, and helper functions
//! used across multiple test modules.

use deepdive::constants::*;

/// Creates a simple test map filled with floor tiles.
/// Useful for pathfinding and FOV tests that need open space.
pub fn create_floor_map() -> Vec<u8> {
    vec![0; (MAP_WIDTH * MAP_HEIGHT) as usize] // 0 = Floor
}

/// Creates a test map with walls around the perimeter.
pub fn create_bordered_map() -> Vec<u8> {
    let mut tiles = vec![0u8; (MAP_WIDTH * MAP_HEIGHT) as usize];

    for x in 0..MAP_WIDTH {
        for y in 0..MAP_HEIGHT {
            let idx = (y * MAP_WIDTH + x) as usize;
            // Wall on edges
            if x == 0 || x == MAP_WIDTH - 1 || y == 0 || y == MAP_HEIGHT - 1 {
                tiles[idx] = 1; // 1 = Wall
            }
        }
    }

    tiles
}

/// Calculates the linear index for a 2D coordinate.
/// Matches the indexing used in GameMap.
pub fn idx(x: u32, y: u32) -> usize {
    (y * MAP_WIDTH + x) as usize
}

/// Asserts that a coordinate is within map bounds.
pub fn assert_in_bounds(x: u32, y: u32) {
    assert!(x < MAP_WIDTH, "x={} out of bounds (max {})", x, MAP_WIDTH - 1);
    assert!(y < MAP_HEIGHT, "y={} out of bounds (max {})", y, MAP_HEIGHT - 1);
}

/// Asserts that two positions are adjacent (including diagonals).
pub fn assert_adjacent(pos1: (u32, u32), pos2: (u32, u32)) {
    let dx = (pos1.0 as i32 - pos2.0 as i32).abs();
    let dy = (pos1.1 as i32 - pos2.1 as i32).abs();
    assert!(
        dx <= 1 && dy <= 1 && (dx + dy) > 0,
        "Positions {:?} and {:?} are not adjacent",
        pos1, pos2
    );
}
