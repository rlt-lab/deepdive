//! Property-Based Pathfinding Tests (Phase 8.2)
//!
//! Uses proptest to verify A* pathfinding invariants:
//! - Finds path if and only if one exists
//! - Returned paths are always valid (walkable, adjacent steps)

mod common;

use deepdive::components::TileType;
use deepdive::constants::{MAP_HEIGHT, MAP_WIDTH};
use deepdive::map::GameMap;
use deepdive::player::find_path;
use proptest::prelude::*;
use std::collections::HashSet;

// =============================================================================
// Test Map Generators for Proptest
// =============================================================================

/// Strategy for generating valid map coordinates.
fn coord_strategy() -> impl Strategy<Value = (u32, u32)> {
    (0..MAP_WIDTH, 0..MAP_HEIGHT)
}

/// Strategy for generating two distinct coordinates.
fn two_distinct_coords() -> impl Strategy<Value = ((u32, u32), (u32, u32))> {
    (coord_strategy(), coord_strategy()).prop_filter(
        "coordinates must be distinct",
        |(a, b)| a != b
    )
}

/// Creates a map with random walls based on seed.
fn create_random_map(seed: u64, wall_density: f64) -> GameMap {
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};

    let mut map = GameMap::new(MAP_WIDTH, MAP_HEIGHT);
    let mut rng = StdRng::seed_from_u64(seed);

    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            if rng.random::<f64>() < wall_density {
                map.set(x, y, TileType::Wall);
            } else {
                map.set(x, y, TileType::Floor);
            }
        }
    }
    map
}

/// Creates an open map (all floor tiles).
fn create_open_map() -> GameMap {
    let mut map = GameMap::new(MAP_WIDTH, MAP_HEIGHT);
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            map.set(x, y, TileType::Floor);
        }
    }
    map
}

// =============================================================================
// 8.2.3 - A* Finds Path If And Only If Path Exists
// =============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property: On an open map, A* always finds a path between any two points.
    #[test]
    fn prop_astar_always_finds_path_on_open_map(
        start in coord_strategy(),
        goal in coord_strategy()
    ) {
        let map = create_open_map();
        let path = find_path(start, goal, &map);

        if start == goal {
            // Same position: path should be empty (already there)
            prop_assert!(
                path.is_empty(),
                "Path from {:?} to itself should be empty",
                start
            );
        } else {
            // Different positions: path must exist
            prop_assert!(
                !path.is_empty(),
                "Path should exist from {:?} to {:?} on open map",
                start,
                goal
            );
        }
    }

    /// Property: A* finds path iff flood-fill confirms connectivity.
    /// This validates that A* is consistent with graph connectivity.
    #[test]
    fn prop_astar_consistent_with_connectivity(
        seed in any::<u64>(),
        (start, goal) in two_distinct_coords()
    ) {
        // Use moderate wall density to create interesting maps
        let map = create_random_map(seed, 0.3);

        // Skip if start or goal is on a wall
        if map.get(start.0, start.1) == TileType::Wall ||
           map.get(goal.0, goal.1) == TileType::Wall {
            return Ok(()); // Skip this case
        }

        let path = find_path(start, goal, &map);

        // Use flood fill to check if goal is reachable from start
        let reachable = common::flood_fill_reachable(&map.tiles, MAP_WIDTH, MAP_HEIGHT, start);
        let goal_reachable = reachable.contains(&goal);

        if goal_reachable {
            prop_assert!(
                !path.is_empty(),
                "A* should find path when flood-fill confirms connectivity: seed={}, {:?} -> {:?}",
                seed,
                start,
                goal
            );
        } else {
            prop_assert!(
                path.is_empty(),
                "A* should return empty when no path exists: seed={}, {:?} -> {:?}",
                seed,
                start,
                goal
            );
        }
    }

    /// Property: If A* finds no path, flood-fill confirms isolation.
    #[test]
    fn prop_no_path_means_isolated(
        seed in any::<u64>(),
        (start, goal) in two_distinct_coords()
    ) {
        let map = create_random_map(seed, 0.4); // Higher density for more isolated regions

        // Skip if start or goal is on a wall
        if map.get(start.0, start.1) == TileType::Wall ||
           map.get(goal.0, goal.1) == TileType::Wall {
            return Ok(());
        }

        let path = find_path(start, goal, &map);

        if path.is_empty() {
            let reachable = common::flood_fill_reachable(&map.tiles, MAP_WIDTH, MAP_HEIGHT, start);
            prop_assert!(
                !reachable.contains(&goal),
                "Empty path should mean goal is unreachable: seed={}, {:?} -> {:?}",
                seed,
                start,
                goal
            );
        }
    }
}

// =============================================================================
// 8.2.4 - Returned Path Is Always Valid
// =============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property: All steps in a returned path are on walkable tiles.
    #[test]
    fn prop_path_all_steps_walkable(
        seed in any::<u64>(),
        (start, goal) in two_distinct_coords()
    ) {
        let map = create_random_map(seed, 0.25);

        // Skip if start or goal is on a wall
        if map.get(start.0, start.1) == TileType::Wall ||
           map.get(goal.0, goal.1) == TileType::Wall {
            return Ok(());
        }

        let path = find_path(start, goal, &map);

        for step in &path {
            let tile = map.get(step.0, step.1);
            prop_assert!(
                tile.is_walkable(),
                "Path step {:?} should be walkable, found {:?}",
                step,
                tile
            );
        }
    }

    /// Property: Each step in path is adjacent (4-connected) to the previous.
    #[test]
    fn prop_path_steps_adjacent(
        seed in any::<u64>(),
        (start, goal) in two_distinct_coords()
    ) {
        let map = create_random_map(seed, 0.25);

        // Skip if start or goal is on a wall
        if map.get(start.0, start.1) == TileType::Wall ||
           map.get(goal.0, goal.1) == TileType::Wall {
            return Ok(());
        }

        let path = find_path(start, goal, &map);

        if path.is_empty() {
            return Ok(());
        }

        // First step adjacent to start
        let first = path[0];
        let dx = (first.0 as i32 - start.0 as i32).abs();
        let dy = (first.1 as i32 - start.1 as i32).abs();
        prop_assert!(
            dx + dy == 1,
            "First step {:?} not adjacent to start {:?}",
            first,
            start
        );

        // Each step adjacent to previous
        for i in 1..path.len() {
            let prev = path[i - 1];
            let curr = path[i];
            let dx = (curr.0 as i32 - prev.0 as i32).abs();
            let dy = (curr.1 as i32 - prev.1 as i32).abs();
            prop_assert!(
                dx + dy == 1,
                "Step {} {:?} not adjacent to step {} {:?}",
                i,
                curr,
                i - 1,
                prev
            );
        }
    }

    /// Property: Path ends at goal (when path exists).
    #[test]
    fn prop_path_ends_at_goal(
        seed in any::<u64>(),
        (start, goal) in two_distinct_coords()
    ) {
        let map = create_random_map(seed, 0.25);

        // Skip if start or goal is on a wall
        if map.get(start.0, start.1) == TileType::Wall ||
           map.get(goal.0, goal.1) == TileType::Wall {
            return Ok(());
        }

        let path = find_path(start, goal, &map);

        if !path.is_empty() {
            prop_assert_eq!(
                path.last().copied(),
                Some(goal),
                "Path should end at goal"
            );
        }
    }

    /// Property: Path contains no duplicate positions.
    #[test]
    fn prop_path_no_duplicates(
        seed in any::<u64>(),
        (start, goal) in two_distinct_coords()
    ) {
        let map = create_random_map(seed, 0.25);

        // Skip if start or goal is on a wall
        if map.get(start.0, start.1) == TileType::Wall ||
           map.get(goal.0, goal.1) == TileType::Wall {
            return Ok(());
        }

        let path = find_path(start, goal, &map);

        let mut seen: HashSet<(u32, u32)> = HashSet::new();
        for step in &path {
            prop_assert!(
                seen.insert(*step),
                "Path contains duplicate: {:?}",
                step
            );
        }
    }

    /// Property: Path does not include start position.
    #[test]
    fn prop_path_excludes_start(
        seed in any::<u64>(),
        (start, goal) in two_distinct_coords()
    ) {
        let map = create_random_map(seed, 0.25);

        // Skip if start or goal is on a wall
        if map.get(start.0, start.1) == TileType::Wall ||
           map.get(goal.0, goal.1) == TileType::Wall {
            return Ok(());
        }

        let path = find_path(start, goal, &map);

        prop_assert!(
            !path.contains(&start),
            "Path should not include start position"
        );
    }

    /// Property: Path length is at least Manhattan distance (optimal lower bound).
    #[test]
    fn prop_path_length_reasonable(
        start in coord_strategy(),
        goal in coord_strategy()
    ) {
        let map = create_open_map();
        let path = find_path(start, goal, &map);

        if start == goal {
            prop_assert!(path.is_empty());
        } else {
            let manhattan = (start.0 as i32 - goal.0 as i32).abs() +
                           (start.1 as i32 - goal.1 as i32).abs();

            // On an open map, path length should equal Manhattan distance
            prop_assert_eq!(
                path.len() as i32,
                manhattan,
                "Path length should equal Manhattan distance on open map"
            );
        }
    }
}
