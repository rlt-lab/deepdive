//! Pathfinding unit tests for Phase 3: Pathfinding (TDD)
//!
//! Tests for A* pathfinding functionality including direct paths,
//! obstacle avoidance, blocked paths, edge cases, and path validity.

mod common;

use deepdive::components::TileType;
use deepdive::constants::{MAP_HEIGHT, MAP_WIDTH};
use deepdive::map::GameMap;
use deepdive::player::find_path;

// =============================================================================
// Test Utilities
// =============================================================================

/// Creates a map filled entirely with floor tiles (open space).
fn create_open_map() -> GameMap {
    let mut map = GameMap::new(MAP_WIDTH, MAP_HEIGHT);
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            map.set(x, y, TileType::Floor);
        }
    }
    map
}

/// Creates a map with a vertical wall blocking the middle.
fn create_wall_barrier_map() -> GameMap {
    let mut map = create_open_map();
    // Create vertical wall at x=40, from y=10 to y=40
    for y in 10..40 {
        map.set(40, y, TileType::Wall);
    }
    map
}

/// Creates a map where a position is completely surrounded by walls.
fn create_isolated_map() -> GameMap {
    let mut map = create_open_map();
    // Surround position (25, 25) with walls
    for dx in -1i32..=1 {
        for dy in -1i32..=1 {
            if dx == 0 && dy == 0 {
                continue; // Keep center as floor
            }
            let x = (25i32 + dx) as u32;
            let y = (25i32 + dy) as u32;
            map.set(x, y, TileType::Wall);
        }
    }
    map
}

// =============================================================================
// 3.1.1 - Direct Path Tests
// =============================================================================

/// Test that a straight horizontal path is found on an open map.
#[test]
fn direct_path_horizontal() {
    let map = create_open_map();
    let start = (10, 25);
    let goal = (20, 25);

    let path = find_path(start, goal, &map);

    // Path should exist and have correct length (10 steps for 10 tiles)
    assert!(!path.is_empty(), "Path should exist for open map");
    assert_eq!(path.len(), 10, "Horizontal path should have 10 steps");

    // Path should end at goal
    assert_eq!(path.last(), Some(&goal), "Path should end at goal");
}

/// Test that a straight vertical path is found on an open map.
#[test]
fn direct_path_vertical() {
    let map = create_open_map();
    let start = (25, 10);
    let goal = (25, 20);

    let path = find_path(start, goal, &map);

    assert!(!path.is_empty(), "Path should exist for open map");
    assert_eq!(path.len(), 10, "Vertical path should have 10 steps");
    assert_eq!(path.last(), Some(&goal), "Path should end at goal");
}

/// Test path across diagonal (Manhattan distance).
#[test]
fn direct_path_diagonal_manhattan() {
    let map = create_open_map();
    let start = (10, 10);
    let goal = (15, 15);

    let path = find_path(start, goal, &map);

    // A* with 4-directional movement: dx=5, dy=5, so 10 steps total
    assert!(!path.is_empty(), "Path should exist");
    assert_eq!(path.len(), 10, "Diagonal path should have 10 steps (Manhattan)");
}

// =============================================================================
// 3.1.2 - Path Around Obstacles Tests
// =============================================================================

/// Test that pathfinding navigates around a wall obstacle.
#[test]
fn path_around_wall_finds_alternate_route() {
    let map = create_wall_barrier_map();
    let start = (35, 25); // Left of wall
    let goal = (45, 25); // Right of wall

    let path = find_path(start, goal, &map);

    // Path should exist (goes around the wall)
    assert!(!path.is_empty(), "Path should exist by going around wall");

    // Path should be longer than direct distance (10)
    // Direct would be 10, but wall forces detour
    assert!(
        path.len() > 10,
        "Path around wall should be longer than direct: got {}",
        path.len()
    );

    // Verify no step is on the wall
    for step in &path {
        assert_ne!(
            map.get(step.0, step.1),
            TileType::Wall,
            "Path should not pass through wall at {:?}",
            step
        );
    }
}

/// Test path around wall from above.
#[test]
fn path_around_wall_from_above() {
    let map = create_wall_barrier_map();
    let start = (35, 5); // Top-left of wall
    let goal = (45, 5); // Top-right of wall (above wall which starts at y=10)

    let path = find_path(start, goal, &map);

    // Path should be direct (wall doesn't block at y=5)
    assert!(!path.is_empty(), "Path should exist above wall");
    assert_eq!(path.len(), 10, "Path above wall should be direct");
}

// =============================================================================
// 3.1.3 - No Path Tests
// =============================================================================

/// Test that no path is returned when goal is completely blocked.
#[test]
fn no_path_when_completely_blocked() {
    let map = create_isolated_map();
    let start = (10, 10); // Outside isolation
    let goal = (25, 25); // Inside isolation (surrounded by walls)

    let path = find_path(start, goal, &map);

    assert!(path.is_empty(), "No path should exist to isolated position");
}

/// Test that no path is returned when start is isolated.
#[test]
fn no_path_when_start_isolated() {
    let map = create_isolated_map();
    let start = (25, 25); // Inside isolation
    let goal = (10, 10); // Outside isolation

    let path = find_path(start, goal, &map);

    assert!(path.is_empty(), "No path should exist from isolated position");
}

/// Test no path through solid wall.
#[test]
fn no_path_through_wall() {
    let mut map = create_open_map();
    // Create complete horizontal wall
    for x in 0..MAP_WIDTH {
        map.set(x, 25, TileType::Wall);
    }

    let start = (40, 10); // Above wall
    let goal = (40, 40); // Below wall

    let path = find_path(start, goal, &map);

    assert!(path.is_empty(), "No path should exist through complete wall");
}

// =============================================================================
// 3.1.4 - Edge Case Tests
// =============================================================================

/// Test that start equals goal returns empty path.
#[test]
fn start_equals_goal_returns_empty() {
    let map = create_open_map();
    let pos = (25, 25);

    let path = find_path(pos, pos, &map);

    assert!(path.is_empty(), "Path from position to itself should be empty");
}

/// Test path to adjacent tile returns single step.
#[test]
fn adjacent_tile_returns_single_step() {
    let map = create_open_map();
    let start = (25, 25);

    // Test all 4 adjacent directions
    let adjacent_goals = [
        (24, 25), // Left
        (26, 25), // Right
        (25, 24), // Up
        (25, 26), // Down
    ];

    for goal in adjacent_goals {
        let path = find_path(start, goal, &map);
        assert_eq!(
            path.len(),
            1,
            "Path to adjacent tile {:?} should be single step",
            goal
        );
        assert_eq!(path[0], goal, "Single step should be the goal");
    }
}

/// Test path at map boundaries.
#[test]
fn path_at_boundaries() {
    let map = create_open_map();

    // Corner to corner paths
    let start = (1, 1);
    let goal = (MAP_WIDTH - 2, MAP_HEIGHT - 2);

    let path = find_path(start, goal, &map);

    assert!(!path.is_empty(), "Path should exist between corners");

    // All steps should be within bounds
    for step in &path {
        assert!(step.0 < MAP_WIDTH, "x should be within bounds");
        assert!(step.1 < MAP_HEIGHT, "y should be within bounds");
    }
}

// =============================================================================
// 3.1.5 - Path Validity Tests
// =============================================================================

/// Test that all path steps are walkable.
#[test]
fn all_path_steps_are_walkable() {
    let map = create_wall_barrier_map();
    let start = (35, 25);
    let goal = (45, 25);

    let path = find_path(start, goal, &map);

    for step in &path {
        let tile = map.get(step.0, step.1);
        assert!(
            tile.is_walkable(),
            "Path step {:?} should be walkable, found {:?}",
            step,
            tile
        );
    }
}

/// Test that each step is adjacent to the previous.
#[test]
fn each_step_adjacent_to_previous() {
    let map = create_wall_barrier_map();
    let start = (35, 25);
    let goal = (45, 25);

    let path = find_path(start, goal, &map);
    assert!(!path.is_empty(), "Path should exist for this test");

    // First step should be adjacent to start
    let first_step = path[0];
    let dx = (first_step.0 as i32 - start.0 as i32).abs();
    let dy = (first_step.1 as i32 - start.1 as i32).abs();
    assert!(
        dx + dy == 1,
        "First step {:?} should be adjacent to start {:?}",
        first_step,
        start
    );

    // Each subsequent step should be adjacent to previous
    for i in 1..path.len() {
        let prev = path[i - 1];
        let curr = path[i];
        let dx = (curr.0 as i32 - prev.0 as i32).abs();
        let dy = (curr.1 as i32 - prev.1 as i32).abs();
        assert!(
            dx + dy == 1,
            "Step {} {:?} should be adjacent to step {} {:?}",
            i,
            curr,
            i - 1,
            prev
        );
    }
}

/// Test that path doesn't contain duplicates.
#[test]
fn path_has_no_duplicates() {
    let map = create_wall_barrier_map();
    let start = (35, 25);
    let goal = (45, 25);

    let path = find_path(start, goal, &map);

    let mut seen = std::collections::HashSet::new();
    for step in &path {
        assert!(
            seen.insert(*step),
            "Path should not contain duplicate step {:?}",
            step
        );
    }
}

/// Test that path doesn't include start position.
#[test]
fn path_does_not_include_start() {
    let map = create_open_map();
    let start = (10, 10);
    let goal = (15, 15);

    let path = find_path(start, goal, &map);

    assert!(
        !path.contains(&start),
        "Path should not include start position"
    );
}

// =============================================================================
// 3.1.6 - VecDeque Operations Tests
// These tests verify the VecDeque-based path consumption pattern.
// VecDeque enables O(1) front() and pop_front() operations vs O(n) with Vec.
// =============================================================================

use deepdive::components::{Autoexplore, AutoMoveToStair};
use std::collections::VecDeque;

/// Test that Autoexplore path uses VecDeque for efficient front operations.
#[test]
fn autoexplore_path_uses_vecdeque() {
    // Create autoexplore with a path
    let mut autoexplore = Autoexplore::default();

    // VecDeque enables efficient O(1) front access and removal
    let test_path: VecDeque<(u32, u32)> = VecDeque::from(vec![(1, 1), (2, 2), (3, 3)]);
    autoexplore.path = test_path;

    // Test front() - O(1) operation
    assert_eq!(
        autoexplore.path.front().copied(),
        Some((1, 1)),
        "front() should return first element"
    );

    // Test pop_front() - O(1) operation
    let popped = autoexplore.path.pop_front();
    assert_eq!(popped, Some((1, 1)), "pop_front() should remove first element");
    assert_eq!(
        autoexplore.path.front().copied(),
        Some((2, 2)),
        "After pop_front(), front should be second element"
    );
}

/// Test that AutoMoveToStair path uses VecDeque.
#[test]
fn auto_move_to_stair_path_uses_vecdeque() {
    // Create AutoMoveToStair with a VecDeque path
    let path: VecDeque<(u32, u32)> = VecDeque::from(vec![(5, 5), (6, 6), (7, 7)]);
    let auto_move = AutoMoveToStair::new((10, 10), path, TileType::StairUp);

    // Test front() operations work with VecDeque
    assert_eq!(
        auto_move.path.front().copied(),
        Some((5, 5)),
        "VecDeque front() should work"
    );
}
