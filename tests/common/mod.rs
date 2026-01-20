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

/// Independent flood-fill to verify map connectivity.
/// Returns all positions reachable from start using 4-directional movement.
pub fn flood_fill_reachable(
    tiles: &[deepdive::components::TileType],
    width: u32,
    height: u32,
    start: (u32, u32),
) -> std::collections::HashSet<(u32, u32)> {
    use deepdive::components::TileType;
    use std::collections::HashSet;

    let mut visited = HashSet::new();
    let mut stack = vec![start];

    while let Some((x, y)) = stack.pop() {
        if visited.contains(&(x, y)) {
            continue;
        }

        let idx = (y * width + x) as usize;
        if idx >= tiles.len() {
            continue;
        }

        // Only traverse walkable tiles
        if !tiles[idx].is_walkable() {
            continue;
        }

        visited.insert((x, y));

        // Check 4 cardinal directions
        if x > 0 {
            stack.push((x - 1, y));
        }
        if x < width - 1 {
            stack.push((x + 1, y));
        }
        if y > 0 {
            stack.push((x, y - 1));
        }
        if y < height - 1 {
            stack.push((x, y + 1));
        }
    }

    visited
}

/// Checks if all walkable tiles in the map are connected.
/// Returns true if there's only one connected region of walkable tiles.
pub fn is_fully_connected(
    tiles: &[deepdive::components::TileType],
    width: u32,
    height: u32,
) -> bool {
    use deepdive::components::TileType;

    // Find first walkable tile
    let mut start = None;
    let mut total_walkable = 0;

    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) as usize;
            if tiles[idx].is_walkable() {
                total_walkable += 1;
                if start.is_none() {
                    start = Some((x, y));
                }
            }
        }
    }

    // If no walkable tiles, consider it connected (vacuously true)
    let Some(start_pos) = start else {
        return true;
    };

    // Flood fill from start and check if we reach all walkable tiles
    let reachable = flood_fill_reachable(tiles, width, height, start_pos);
    reachable.len() == total_walkable
}
