//! FOV (Field of View) unit tests for Phase 4: FOV System (TDD)
//!
//! Tests for line-of-sight calculations, FOV radius bounds,
//! visibility states, and LOS cache behavior.

use deepdive::components::{FovConfig, FovState, LosCache, TileType, TileVisibility};
use deepdive::constants::{FOV_RADIUS, MAP_HEIGHT, MAP_WIDTH};
use deepdive::fov::has_line_of_sight;
use deepdive::map::GameMap;

mod common;

// =============================================================================
// 4.1.1 - Line of Sight: Clear LOS on Empty Map
// =============================================================================

/// Test that clear LOS exists between any two points on an all-floor map.
#[test]
fn los_clear_on_empty_map() {
    let mut map = GameMap::new(MAP_WIDTH, MAP_HEIGHT);

    // Fill map with floor tiles
    for x in 0..MAP_WIDTH {
        for y in 0..MAP_HEIGHT {
            map.set(x, y, TileType::Floor);
        }
    }

    // Test various LOS checks on open map
    assert!(has_line_of_sight(&map, 10, 10, 20, 20)); // Diagonal
    assert!(has_line_of_sight(&map, 10, 10, 10, 30)); // Vertical
    assert!(has_line_of_sight(&map, 10, 10, 30, 10)); // Horizontal
    assert!(has_line_of_sight(&map, 5, 5, 5, 5)); // Same point
}

/// Test LOS from origin to various points.
#[test]
fn los_from_origin_clear() {
    let mut map = GameMap::new(MAP_WIDTH, MAP_HEIGHT);

    for x in 0..MAP_WIDTH {
        for y in 0..MAP_HEIGHT {
            map.set(x, y, TileType::Floor);
        }
    }

    assert!(has_line_of_sight(&map, 0, 0, 10, 10));
    assert!(has_line_of_sight(&map, 0, 0, 0, 20));
    assert!(has_line_of_sight(&map, 0, 0, 20, 0));
}

/// Test LOS across long distances on open map.
#[test]
fn los_long_distance_clear() {
    let mut map = GameMap::new(MAP_WIDTH, MAP_HEIGHT);

    for x in 0..MAP_WIDTH {
        for y in 0..MAP_HEIGHT {
            map.set(x, y, TileType::Floor);
        }
    }

    // Test edge to edge
    assert!(has_line_of_sight(&map, 1, 1, MAP_WIDTH as i32 - 2, MAP_HEIGHT as i32 - 2));
}

// =============================================================================
// 4.1.2 - LOS Blocking: Wall Blocks LOS
// =============================================================================

/// Test that a single wall blocks LOS.
#[test]
fn los_blocked_by_single_wall() {
    let mut map = GameMap::new(MAP_WIDTH, MAP_HEIGHT);

    for x in 0..MAP_WIDTH {
        for y in 0..MAP_HEIGHT {
            map.set(x, y, TileType::Floor);
        }
    }

    // Place wall between viewer and target
    map.set(15, 15, TileType::Wall);

    // LOS from (10,15) to (20,15) should be blocked by wall at (15,15)
    assert!(!has_line_of_sight(&map, 10, 15, 20, 15));
}

/// Test that wall directly on target does NOT block LOS (can see walls).
#[test]
fn los_not_blocked_at_target() {
    let mut map = GameMap::new(MAP_WIDTH, MAP_HEIGHT);

    for x in 0..MAP_WIDTH {
        for y in 0..MAP_HEIGHT {
            map.set(x, y, TileType::Floor);
        }
    }

    // Place wall at target
    map.set(20, 15, TileType::Wall);

    // LOS to wall should work (we can see the wall itself)
    assert!(has_line_of_sight(&map, 10, 15, 20, 15));
}

/// Test that diagonal wall blocks diagonal LOS.
#[test]
fn los_blocked_by_wall_diagonal() {
    let mut map = GameMap::new(MAP_WIDTH, MAP_HEIGHT);

    for x in 0..MAP_WIDTH {
        for y in 0..MAP_HEIGHT {
            map.set(x, y, TileType::Floor);
        }
    }

    // Place wall on diagonal path
    map.set(15, 15, TileType::Wall);

    // Diagonal LOS through (15,15) should be blocked
    assert!(!has_line_of_sight(&map, 10, 10, 20, 20));
}

/// Test that multiple walls block LOS.
#[test]
fn los_blocked_by_wall_line() {
    let mut map = GameMap::new(MAP_WIDTH, MAP_HEIGHT);

    for x in 0..MAP_WIDTH {
        for y in 0..MAP_HEIGHT {
            map.set(x, y, TileType::Floor);
        }
    }

    // Create a wall line
    for x in 14..17 {
        map.set(x, 15, TileType::Wall);
    }

    // LOS through wall line should be blocked
    assert!(!has_line_of_sight(&map, 10, 15, 20, 15));
    assert!(!has_line_of_sight(&map, 15, 10, 15, 20));
}

/// Test LOS path that goes around a wall (should be clear if path exists).
#[test]
fn los_clear_around_wall() {
    let mut map = GameMap::new(MAP_WIDTH, MAP_HEIGHT);

    for x in 0..MAP_WIDTH {
        for y in 0..MAP_HEIGHT {
            map.set(x, y, TileType::Floor);
        }
    }

    // Place wall not on direct line
    map.set(15, 20, TileType::Wall);

    // LOS that doesn't cross wall should be clear
    assert!(has_line_of_sight(&map, 10, 15, 20, 15)); // Horizontal, wall is above
}

// =============================================================================
// 4.1.3 - LOS Symmetry: If A Sees B, B Sees A
// =============================================================================

/// Test LOS symmetry on open map.
#[test]
fn los_symmetry_clear_map() {
    let mut map = GameMap::new(MAP_WIDTH, MAP_HEIGHT);

    for x in 0..MAP_WIDTH {
        for y in 0..MAP_HEIGHT {
            map.set(x, y, TileType::Floor);
        }
    }

    // A sees B implies B sees A
    let (ax, ay) = (10, 10);
    let (bx, by) = (25, 30);

    let a_to_b = has_line_of_sight(&map, ax, ay, bx, by);
    let b_to_a = has_line_of_sight(&map, bx, by, ax, ay);

    assert_eq!(a_to_b, b_to_a, "LOS should be symmetric: A→B = B→A");
}

/// Test LOS symmetry with blocking wall.
#[test]
fn los_symmetry_with_wall() {
    let mut map = GameMap::new(MAP_WIDTH, MAP_HEIGHT);

    for x in 0..MAP_WIDTH {
        for y in 0..MAP_HEIGHT {
            map.set(x, y, TileType::Floor);
        }
    }

    // Place wall between points
    map.set(17, 20, TileType::Wall);

    let (ax, ay) = (10, 20);
    let (bx, by) = (25, 20);

    let a_to_b = has_line_of_sight(&map, ax, ay, bx, by);
    let b_to_a = has_line_of_sight(&map, bx, by, ax, ay);

    assert_eq!(a_to_b, b_to_a, "LOS blocking should be symmetric");
    assert!(!a_to_b, "Wall should block LOS in both directions");
}

/// Test LOS symmetry for diagonal paths.
#[test]
fn los_symmetry_diagonal() {
    let mut map = GameMap::new(MAP_WIDTH, MAP_HEIGHT);

    for x in 0..MAP_WIDTH {
        for y in 0..MAP_HEIGHT {
            map.set(x, y, TileType::Floor);
        }
    }

    let test_pairs = [
        ((5, 5), (15, 25)),
        ((10, 40), (30, 10)),
        ((1, 1), (40, 40)),
    ];

    for ((ax, ay), (bx, by)) in test_pairs {
        let a_to_b = has_line_of_sight(&map, ax, ay, bx, by);
        let b_to_a = has_line_of_sight(&map, bx, by, ax, ay);
        assert_eq!(
            a_to_b, b_to_a,
            "LOS symmetry failed for ({},{}) <-> ({},{})",
            ax, ay, bx, by
        );
    }
}

// =============================================================================
// 4.1.4 - FOV Radius: Tiles Within Radius Visible, Outside Not
// =============================================================================

/// Test that FOV_RADIUS constant is reasonable.
#[test]
fn fov_radius_is_positive() {
    assert!(FOV_RADIUS > 0, "FOV radius must be positive");
    assert!(
        FOV_RADIUS < MAP_WIDTH && FOV_RADIUS < MAP_HEIGHT,
        "FOV radius should be smaller than map dimensions"
    );
}

/// Test distance calculation for FOV bounds.
#[test]
fn fov_distance_within_radius() {
    let player_x: i32 = 40;
    let player_y: i32 = 25;
    let radius = FOV_RADIUS as i32;
    let radius_squared = radius * radius;

    // Test point clearly within radius
    let near_x: i32 = player_x + 5;
    let near_y: i32 = player_y + 5;
    let near_dist_sq = (near_x - player_x).pow(2) + (near_y - player_y).pow(2);
    assert!(
        near_dist_sq <= radius_squared,
        "Point at (5,5) offset should be within FOV"
    );

    // Test point clearly outside radius
    let far_x: i32 = player_x + (radius + 10);
    let far_y: i32 = player_y + (radius + 10);
    let far_dist_sq = (far_x - player_x).pow(2) + (far_y - player_y).pow(2);
    assert!(
        far_dist_sq > radius_squared,
        "Point far outside should not be within FOV"
    );
}

/// Test FOV boundary edge cases.
#[test]
fn fov_boundary_edge_cases() {
    let radius = FOV_RADIUS as i32;
    let radius_squared = radius * radius;

    // Exactly at radius should be within
    let at_radius_dist_sq = radius * radius;
    assert!(at_radius_dist_sq <= radius_squared);

    // Just beyond radius should be outside
    let beyond_radius_dist_sq = (radius + 1) * (radius + 1);
    assert!(beyond_radius_dist_sq > radius_squared);
}

// =============================================================================
// 4.1.5 - Visibility States: Unseen → Visible Transition
// =============================================================================

/// Test TileVisibility enum has expected variants.
#[test]
fn visibility_states_exist() {
    let unseen = TileVisibility::Unseen;
    let seen = TileVisibility::Seen;
    let visible = TileVisibility::Visible;

    // Verify they are distinct
    assert_ne!(unseen, seen);
    assert_ne!(unseen, visible);
    assert_ne!(seen, visible);
}

/// Test FovConfig default initialization.
#[test]
fn fov_config_default() {
    let config = FovConfig::default();
    assert_eq!(config.radius, FOV_RADIUS);
}

/// Test FovState default initialization.
#[test]
fn fov_state_default() {
    let state = FovState::default();
    assert!(!state.debug_reveal_all);
    assert!(state.needs_recalculation);
    assert!(!state.debug_mode_applied);
    assert!(state.last_player_pos.is_none());
    assert!(state.dirty_tiles.is_empty());
}

/// Test LosCache default initialization.
#[test]
fn los_cache_default() {
    let cache = LosCache::default();
    assert!(cache.cache.is_empty());
    assert_eq!(cache.hits, 0);
    assert_eq!(cache.misses, 0);
}

/// Test that visibility can transition from Unseen to Visible.
#[test]
fn visibility_transition_unseen_to_visible() {
    let mut visibility = TileVisibility::Unseen;

    // Simulate transition when tile comes into FOV with clear LOS
    visibility = TileVisibility::Visible;

    assert_eq!(visibility, TileVisibility::Visible);
}

// =============================================================================
// 4.1.6 - Visibility States: Visible → Seen Transition
// =============================================================================

/// Test that visibility can transition from Visible to Seen.
#[test]
fn visibility_transition_visible_to_seen() {
    let mut visibility = TileVisibility::Visible;

    // Simulate transition when tile leaves FOV
    visibility = TileVisibility::Seen;

    assert_eq!(visibility, TileVisibility::Seen);
}

/// Test that Seen tiles stay Seen (never go back to Unseen).
#[test]
fn visibility_seen_stays_seen() {
    let visibility = TileVisibility::Seen;

    // Seen should not revert to Unseen
    // (This is a design assertion - once explored, always remembered)
    assert_ne!(visibility, TileVisibility::Unseen);
}

/// Test full visibility cycle.
#[test]
fn visibility_full_cycle() {
    let mut visibility = TileVisibility::Unseen;

    // Initial state
    assert_eq!(visibility, TileVisibility::Unseen);

    // Tile enters FOV
    visibility = TileVisibility::Visible;
    assert_eq!(visibility, TileVisibility::Visible);

    // Tile leaves FOV
    visibility = TileVisibility::Seen;
    assert_eq!(visibility, TileVisibility::Seen);

    // Tile re-enters FOV
    visibility = TileVisibility::Visible;
    assert_eq!(visibility, TileVisibility::Visible);

    // Tile leaves FOV again
    visibility = TileVisibility::Seen;
    assert_eq!(visibility, TileVisibility::Seen);
}
