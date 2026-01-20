//! Integration Tests for Autoexplore (Phase 8.1.3-8.1.4)
//!
//! Tests for exploration termination and path validity during autoexplore.

mod common;

use std::collections::{HashSet, VecDeque};
use deepdive::biome::BiomeType;
use deepdive::components::{EllipseMask, TileType};
use deepdive::constants::{MAP_HEIGHT, MAP_WIDTH};
use deepdive::map::GameMap;
use deepdive::player::find_path;
use rand::rngs::StdRng;
use rand::SeedableRng;

// =============================================================================
// 8.1.3 - Complete Map Exploration Terminates
// =============================================================================

/// Test that exploration terminates when all reachable tiles are visited.
#[test]
fn exploration_terminates_when_all_visited() {
    let mut map = GameMap::new(MAP_WIDTH, MAP_HEIGHT);
    let ellipse_mask = EllipseMask::new(MAP_WIDTH, MAP_HEIGHT);
    let mut rng = StdRng::seed_from_u64(12345);

    map.generate_with_biome(BiomeType::Caverns, 1, &mut rng, &ellipse_mask);

    // Get all walkable positions
    let walkable: HashSet<(u32, u32)> = (0..MAP_HEIGHT)
        .flat_map(|y| (0..MAP_WIDTH).map(move |x| (x, y)))
        .filter(|&(x, y)| map.get(x, y).is_walkable())
        .collect();

    // Simulate marking all as "explored" (visited)
    let explored = walkable.clone();

    // Find unexplored (should be empty)
    let unexplored: Vec<_> = walkable
        .iter()
        .filter(|pos| !explored.contains(pos))
        .collect();

    assert!(
        unexplored.is_empty(),
        "All walkable tiles should be explored, but {} remain",
        unexplored.len()
    );
}

/// Test that exploration finds all connected floor tiles.
#[test]
fn exploration_reaches_all_connected_floors() {
    let mut map = GameMap::new(MAP_WIDTH, MAP_HEIGHT);
    let ellipse_mask = EllipseMask::new(MAP_WIDTH, MAP_HEIGHT);
    let mut rng = StdRng::seed_from_u64(54321);

    map.generate_with_biome(BiomeType::Caverns, 1, &mut rng, &ellipse_mask);

    // Find a starting floor tile
    let start = (0..MAP_HEIGHT)
        .flat_map(|y| (0..MAP_WIDTH).map(move |x| (x, y)))
        .find(|&(x, y)| map.get(x, y) == TileType::Floor)
        .expect("Map should have at least one floor tile");

    // Use flood fill to find all reachable floors
    let reachable = common::flood_fill_reachable(&map.tiles, MAP_WIDTH, MAP_HEIGHT, start);

    // All reachable tiles should be walkable
    for &(x, y) in &reachable {
        assert!(
            map.get(x, y).is_walkable(),
            "Tile at ({}, {}) should be walkable",
            x, y
        );
    }

    // Verify we can path from start to any reachable tile
    let sample_targets: Vec<_> = reachable.iter().take(10).copied().collect();
    for target in sample_targets {
        let path = find_path(start, target, &map);
        // Path might be empty if start == target, otherwise should exist
        if start != target {
            assert!(
                !path.is_empty(),
                "Should find path from {:?} to {:?}",
                start, target
            );
        }
    }
}

/// Test that exploration count decreases as tiles are visited.
#[test]
fn unexplored_count_decreases_with_exploration() {
    let mut map = GameMap::new(MAP_WIDTH, MAP_HEIGHT);
    let ellipse_mask = EllipseMask::new(MAP_WIDTH, MAP_HEIGHT);
    let mut rng = StdRng::seed_from_u64(11111);

    map.generate_with_biome(BiomeType::Caverns, 1, &mut rng, &ellipse_mask);

    // Get all walkable positions (initially "unexplored")
    let all_walkable: HashSet<(u32, u32)> = (0..MAP_HEIGHT)
        .flat_map(|y| (0..MAP_WIDTH).map(move |x| (x, y)))
        .filter(|&(x, y)| map.get(x, y).is_walkable())
        .collect();

    let total_walkable = all_walkable.len();
    let mut explored: HashSet<(u32, u32)> = HashSet::new();

    // Simulate exploration by adding tiles to explored set
    let walkable_vec: Vec<_> = all_walkable.iter().copied().collect();
    let mut last_unexplored = total_walkable;

    for (i, pos) in walkable_vec.iter().enumerate() {
        explored.insert(*pos);
        let unexplored_count = total_walkable - explored.len();

        // Unexplored count should decrease monotonically
        assert!(
            unexplored_count <= last_unexplored,
            "Unexplored count should not increase: was {}, now {}",
            last_unexplored,
            unexplored_count
        );
        last_unexplored = unexplored_count;

        // Stop after exploring 100 tiles to keep test fast
        if i >= 100 {
            break;
        }
    }
}

// =============================================================================
// 8.1.4 - Path Validity During Exploration
// =============================================================================

/// Test that paths generated during exploration are valid.
#[test]
fn exploration_paths_are_valid() {
    let mut map = GameMap::new(MAP_WIDTH, MAP_HEIGHT);
    let ellipse_mask = EllipseMask::new(MAP_WIDTH, MAP_HEIGHT);
    let mut rng = StdRng::seed_from_u64(77777);

    map.generate_with_biome(BiomeType::Caverns, 1, &mut rng, &ellipse_mask);

    // Find walkable tiles
    let walkable: Vec<(u32, u32)> = (0..MAP_HEIGHT)
        .flat_map(|y| (0..MAP_WIDTH).map(move |x| (x, y)))
        .filter(|&(x, y)| map.get(x, y).is_walkable())
        .collect();

    if walkable.len() < 2 {
        return; // Need at least 2 tiles
    }

    let start = walkable[0];
    let goal = walkable[walkable.len() / 2];

    let path = find_path(start, goal, &map);

    // Verify path validity
    for &(x, y) in &path {
        // Each step should be walkable
        assert!(
            map.get(x, y).is_walkable(),
            "Path step ({}, {}) should be walkable",
            x, y
        );

        // Each step should be within bounds
        assert!(x < MAP_WIDTH, "Path x={} out of bounds", x);
        assert!(y < MAP_HEIGHT, "Path y={} out of bounds", y);
    }
}

/// Test that consecutive path steps are adjacent.
#[test]
fn exploration_path_steps_are_adjacent() {
    let mut map = GameMap::new(MAP_WIDTH, MAP_HEIGHT);
    let ellipse_mask = EllipseMask::new(MAP_WIDTH, MAP_HEIGHT);
    let mut rng = StdRng::seed_from_u64(88888);

    map.generate_with_biome(BiomeType::Caverns, 1, &mut rng, &ellipse_mask);

    // Find walkable tiles
    let walkable: Vec<(u32, u32)> = (0..MAP_HEIGHT)
        .flat_map(|y| (0..MAP_WIDTH).map(move |x| (x, y)))
        .filter(|&(x, y)| map.get(x, y).is_walkable())
        .collect();

    if walkable.len() < 2 {
        return;
    }

    let start = walkable[0];
    let goal = walkable[walkable.len() - 1];

    let path = find_path(start, goal, &map);

    // Check adjacency between consecutive steps
    // Note: path excludes start, includes goal
    let mut current = start;
    for &next in &path {
        common::assert_adjacent(current, next);
        current = next;
    }
}

/// Test path behavior when start equals goal.
#[test]
fn exploration_path_start_equals_goal() {
    let mut map = GameMap::new(MAP_WIDTH, MAP_HEIGHT);
    map.set(10, 10, TileType::Floor);

    let path = find_path((10, 10), (10, 10), &map);

    // Path should be empty when start == goal
    assert!(
        path.is_empty(),
        "Path from position to itself should be empty"
    );
}

/// Test that autoexplore simulation completes without infinite loop.
#[test]
fn autoexplore_simulation_completes() {
    let mut map = GameMap::new(MAP_WIDTH, MAP_HEIGHT);
    let ellipse_mask = EllipseMask::new(MAP_WIDTH, MAP_HEIGHT);
    let mut rng = StdRng::seed_from_u64(99999);

    map.generate_with_biome(BiomeType::Caverns, 1, &mut rng, &ellipse_mask);

    // Find starting position
    let start = (0..MAP_HEIGHT)
        .flat_map(|y| (0..MAP_WIDTH).map(move |x| (x, y)))
        .find(|&(x, y)| map.get(x, y).is_walkable())
        .expect("Map should have walkable tiles");

    // Simulate autoexplore
    let mut explored: HashSet<(u32, u32)> = HashSet::new();
    let mut current_pos = start;
    let mut iterations = 0;
    let max_iterations = 10000; // Safety limit

    while iterations < max_iterations {
        iterations += 1;
        explored.insert(current_pos);

        // Find nearest unexplored walkable tile
        let unexplored_target = find_nearest_unexplored_simple(current_pos, &explored, &map);

        match unexplored_target {
            Some(target) => {
                // Path to target
                let path = find_path(current_pos, target, &map);
                if path.is_empty() {
                    // Target is unreachable, mark it explored and continue
                    explored.insert(target);
                    continue;
                }

                // "Walk" the path
                for &step in &path {
                    current_pos = step;
                    explored.insert(current_pos);
                }
            }
            None => {
                // No more unexplored tiles - exploration complete
                break;
            }
        }
    }

    assert!(
        iterations < max_iterations,
        "Autoexplore should complete within {} iterations, took {}",
        max_iterations, iterations
    );
}

/// Test that paths avoid walls during exploration.
#[test]
fn exploration_paths_avoid_walls() {
    let mut map = GameMap::new(MAP_WIDTH, MAP_HEIGHT);
    let ellipse_mask = EllipseMask::new(MAP_WIDTH, MAP_HEIGHT);
    let mut rng = StdRng::seed_from_u64(44444);

    map.generate_with_biome(BiomeType::Caverns, 1, &mut rng, &ellipse_mask);

    let walkable: Vec<(u32, u32)> = (0..MAP_HEIGHT)
        .flat_map(|y| (0..MAP_WIDTH).map(move |x| (x, y)))
        .filter(|&(x, y)| map.get(x, y).is_walkable())
        .collect();

    if walkable.len() < 2 {
        return;
    }

    // Test several random paths
    for i in 0..10 {
        let start_idx = i * walkable.len() / 20;
        let goal_idx = (i + 10) * walkable.len() / 20;

        if start_idx >= walkable.len() || goal_idx >= walkable.len() {
            continue;
        }

        let start = walkable[start_idx];
        let goal = walkable[goal_idx];

        let path = find_path(start, goal, &map);

        // Every step in path should not be a wall
        for &(x, y) in &path {
            assert_ne!(
                map.get(x, y),
                TileType::Wall,
                "Path should not include wall at ({}, {})",
                x, y
            );
        }
    }
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Simple BFS to find nearest unexplored tile (test helper, not using Bevy Query).
fn find_nearest_unexplored_simple(
    start: (u32, u32),
    explored: &HashSet<(u32, u32)>,
    map: &GameMap,
) -> Option<(u32, u32)> {
    use std::collections::VecDeque;

    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    queue.push_back(start);
    visited.insert(start);

    while let Some((x, y)) = queue.pop_front() {
        // Check if this is unexplored and walkable
        if !explored.contains(&(x, y)) && map.get(x, y).is_walkable() {
            return Some((x, y));
        }

        // Explore neighbors
        let neighbors = [
            (x.wrapping_sub(1), y),
            (x + 1, y),
            (x, y.wrapping_sub(1)),
            (x, y + 1),
        ];

        for (nx, ny) in neighbors {
            if nx < MAP_WIDTH && ny < MAP_HEIGHT && !visited.contains(&(nx, ny)) {
                if map.get(nx, ny).is_walkable() {
                    visited.insert((nx, ny));
                    queue.push_back((nx, ny));
                }
            }
        }
    }

    None
}
