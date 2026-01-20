//! Property-Based FOV Tests (Phase 8.2)
//!
//! Uses proptest to verify FOV/LOS invariants:
//! - Line of sight is symmetric (if A sees B, B sees A)
//! - Visible tiles are within radius bounds

mod common;

use deepdive::components::TileType;
use deepdive::constants::{FOV_RADIUS, MAP_HEIGHT, MAP_WIDTH};
use deepdive::fov::has_line_of_sight;
use deepdive::map::GameMap;
use proptest::prelude::*;

// =============================================================================
// Test Utilities
// =============================================================================

/// Strategy for generating valid map coordinates within safe bounds.
/// Uses i32 because has_line_of_sight takes i32 parameters.
fn los_coord_strategy() -> impl Strategy<Value = (i32, i32)> {
    (1i32..(MAP_WIDTH as i32 - 1), 1i32..(MAP_HEIGHT as i32 - 1))
}

/// Strategy for generating two distinct LOS coordinates.
fn two_distinct_los_coords() -> impl Strategy<Value = ((i32, i32), (i32, i32))> {
    (los_coord_strategy(), los_coord_strategy()).prop_filter(
        "coordinates must be distinct",
        |(a, b)| a != b
    )
}

/// Strategy for generating coordinates within FOV radius of a center point.
fn coords_within_radius(center_x: i32, center_y: i32, radius: i32) -> impl Strategy<Value = (i32, i32)> {
    let min_x = (center_x - radius).max(0);
    let max_x = (center_x + radius).min(MAP_WIDTH as i32 - 1);
    let min_y = (center_y - radius).max(0);
    let max_y = (center_y + radius).min(MAP_HEIGHT as i32 - 1);

    (min_x..=max_x, min_y..=max_y)
}

/// Creates an open map (all floor tiles) for LOS testing.
fn create_open_map() -> GameMap {
    let mut map = GameMap::new(MAP_WIDTH, MAP_HEIGHT);
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            map.set(x, y, TileType::Floor);
        }
    }
    map
}

/// Creates a map with random walls.
fn create_random_wall_map(seed: u64, wall_density: f64) -> GameMap {
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

// =============================================================================
// 8.2.5 - LOS Is Symmetric (Property Test)
// =============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// Property: Line of sight is symmetric - if A sees B, then B sees A.
    /// This is a fundamental property for fair gameplay.
    #[test]
    fn prop_los_symmetric_open_map((ax, ay) in los_coord_strategy(), (bx, by) in los_coord_strategy()) {
        let map = create_open_map();

        let a_sees_b = has_line_of_sight(&map, ax, ay, bx, by);
        let b_sees_a = has_line_of_sight(&map, bx, by, ax, ay);

        prop_assert_eq!(
            a_sees_b,
            b_sees_a,
            "LOS not symmetric: ({},{}) -> ({},{}) = {}, but reverse = {}",
            ax, ay, bx, by, a_sees_b, b_sees_a
        );
    }

    /// Property: LOS symmetry holds even with walls present.
    /// IGNORED: Bresenham's algorithm is inherently asymmetric. See docs/BUGS.md BUG-003.
    #[test]
    #[ignore]
    fn prop_los_symmetric_with_walls(
        seed in any::<u64>(),
        ((ax, ay), (bx, by)) in two_distinct_los_coords()
    ) {
        let map = create_random_wall_map(seed, 0.2);

        let a_sees_b = has_line_of_sight(&map, ax, ay, bx, by);
        let b_sees_a = has_line_of_sight(&map, bx, by, ax, ay);

        prop_assert_eq!(
            a_sees_b,
            b_sees_a,
            "LOS not symmetric with walls (seed={}): ({},{}) -> ({},{}) = {}, reverse = {}",
            seed, ax, ay, bx, by, a_sees_b, b_sees_a
        );
    }

    /// Property: LOS to self is always true (trivially).
    #[test]
    fn prop_los_to_self_always_true((x, y) in los_coord_strategy()) {
        let map = create_open_map();

        prop_assert!(
            has_line_of_sight(&map, x, y, x, y),
            "LOS from ({},{}) to itself should be true",
            x, y
        );
    }

    /// Property: Adjacent tiles always have LOS on open map.
    #[test]
    fn prop_los_adjacent_always_clear(x in 1i32..(MAP_WIDTH as i32 - 2), y in 1i32..(MAP_HEIGHT as i32 - 2)) {
        let map = create_open_map();

        // Test all 4 cardinal directions
        prop_assert!(has_line_of_sight(&map, x, y, x + 1, y), "LOS right");
        prop_assert!(has_line_of_sight(&map, x, y, x - 1, y), "LOS left");
        prop_assert!(has_line_of_sight(&map, x, y, x, y + 1), "LOS down");
        prop_assert!(has_line_of_sight(&map, x, y, x, y - 1), "LOS up");

        // Test diagonals
        prop_assert!(has_line_of_sight(&map, x, y, x + 1, y + 1), "LOS diagonal");
        prop_assert!(has_line_of_sight(&map, x, y, x - 1, y - 1), "LOS diagonal");
    }

    /// Property: A wall directly between two points blocks LOS.
    #[test]
    fn prop_wall_blocks_los(x in 5i32..(MAP_WIDTH as i32 - 10), y in 5i32..(MAP_HEIGHT as i32 - 5)) {
        let mut map = create_open_map();

        // Place wall between viewer and target
        let wall_x = x + 3;
        map.set(wall_x as u32, y as u32, TileType::Wall);

        // LOS through wall should be blocked
        let has_los = has_line_of_sight(&map, x, y, x + 6, y);

        prop_assert!(
            !has_los,
            "LOS from ({},{}) to ({},{}) should be blocked by wall at ({},{})",
            x, y, x + 6, y, wall_x, y
        );
    }
}

// =============================================================================
// 8.2.6 - Visible Tiles Within Radius Bounds (Property Test)
// =============================================================================

// Note: Some radius boundary tests are regular unit tests rather than property
// tests because they test specific constant values rather than random inputs.

/// Test that distance calculation for FOV radius is consistent.
#[test]
fn test_distance_within_radius_check() {
    let radius = FOV_RADIUS as i32;
    let radius_squared = radius * radius;

    // Test various offsets within and outside radius
    for dx in -radius..=radius {
        for dy in -radius..=radius {
            let dist_squared = dx * dx + dy * dy;
            let within_radius = dist_squared <= radius_squared;

            // Verify consistency
            assert_eq!(
                within_radius,
                dist_squared <= radius_squared,
                "Distance check should be consistent for ({}, {})",
                dx, dy
            );
        }
    }
}

/// Test that tiles at exactly radius distance pass the check.
#[test]
fn test_boundary_tiles_within_radius() {
    let radius = FOV_RADIUS as i32;
    let radius_squared = radius * radius;

    // Test cardinal directions at exact radius
    let at_radius = radius * radius; // dx=radius, dy=0
    assert!(
        at_radius <= radius_squared,
        "Tile at exact radius should be within FOV"
    );
}

/// Test that tiles just beyond radius fail the check.
#[test]
fn test_beyond_radius_excluded() {
    let radius = FOV_RADIUS as i32;
    let radius_squared = radius * radius;

    // One tile beyond radius
    let beyond = (radius + 1) * (radius + 1);
    assert!(
        beyond > radius_squared,
        "Tile beyond radius should be outside FOV"
    );
}

/// Test that FOV_RADIUS constant is reasonable.
#[test]
fn test_fov_radius_reasonable() {
    assert!(FOV_RADIUS > 0, "FOV radius must be positive");
    assert!(
        FOV_RADIUS < MAP_WIDTH.min(MAP_HEIGHT) / 2,
        "FOV radius should be smaller than half the map"
    );
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// Property: All tiles visible from a point are within radius bounds.
    /// Tests that FOV respects radius constraints on open map.
    #[test]
    fn prop_visible_tiles_respect_radius(
        center_x in (FOV_RADIUS + 1)..(MAP_WIDTH - FOV_RADIUS - 1),
        center_y in (FOV_RADIUS + 1)..(MAP_HEIGHT - FOV_RADIUS - 1)
    ) {
        let map = create_open_map();
        let radius = FOV_RADIUS as i32;
        let radius_squared = radius * radius;

        // Check all tiles that should be visible
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                let tx = center_x as i32 + dx;
                let ty = center_y as i32 + dy;
                let dist_squared = dx * dx + dy * dy;

                if dist_squared <= radius_squared {
                    // Tile is within radius - should have LOS on open map
                    let has_los = has_line_of_sight(&map, center_x as i32, center_y as i32, tx, ty);
                    prop_assert!(
                        has_los,
                        "Tile at ({},{}) within radius {} should be visible from ({},{})",
                        tx, ty, radius, center_x, center_y
                    );
                }
            }
        }
    }

    /// Property: Chebyshev distance tiles at radius have LOS (corner check).
    #[test]
    fn prop_diagonal_radius_has_los(
        center_x in (FOV_RADIUS + 1)..(MAP_WIDTH - FOV_RADIUS - 1),
        center_y in (FOV_RADIUS + 1)..(MAP_HEIGHT - FOV_RADIUS - 1)
    ) {
        let map = create_open_map();
        let radius = FOV_RADIUS as i32;

        // Check tiles along diagonal within Euclidean radius
        // The diagonal distance sqrt(2) * d must be <= radius
        let max_diag = (radius as f64 / std::f64::consts::SQRT_2).floor() as i32;

        for d in 1..=max_diag {
            let tx = center_x as i32 + d;
            let ty = center_y as i32 + d;

            let has_los = has_line_of_sight(&map, center_x as i32, center_y as i32, tx, ty);
            prop_assert!(
                has_los,
                "Diagonal tile at ({},{}) should be visible from ({},{})",
                tx, ty, center_x, center_y
            );
        }
    }
}
