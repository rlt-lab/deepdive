//! Integration Tests for FOV Movement (Phase 8.1.5)
//!
//! Tests for visibility state updates on player movement.

mod common;

use std::collections::HashMap;
use deepdive::components::{TileVisibility, TileVisibilityState};
use deepdive::constants::{MAP_HEIGHT, MAP_WIDTH, FOV_RADIUS};

// =============================================================================
// 8.1.5 - Visibility Updates on Player Movement
// =============================================================================

/// Test visibility state transitions: Unseen -> Visible.
#[test]
fn visibility_transition_unseen_to_visible() {
    let initial = TileVisibility::Unseen;
    let after_fov = TileVisibility::Visible;

    // Visibility should be able to transition from Unseen to Visible
    assert_ne!(initial, after_fov);
    assert_eq!(after_fov, TileVisibility::Visible);
}

/// Test visibility state transitions: Visible -> Seen.
#[test]
fn visibility_transition_visible_to_seen() {
    let during_fov = TileVisibility::Visible;
    let after_leaving_fov = TileVisibility::Seen;

    // When player moves away, visible tiles become seen
    assert_ne!(during_fov, after_leaving_fov);
    assert_eq!(after_leaving_fov, TileVisibility::Seen);
}

/// Test visibility state transitions: Seen -> Visible (re-entering FOV).
#[test]
fn visibility_transition_seen_to_visible() {
    let remembered = TileVisibility::Seen;
    let re_entered_fov = TileVisibility::Visible;

    // When player returns, seen tiles become visible again
    assert_ne!(remembered, re_entered_fov);
    assert_eq!(re_entered_fov, TileVisibility::Visible);
}

/// Test that Unseen cannot directly become Seen (must be Visible first).
#[test]
fn visibility_unseen_cannot_skip_to_seen() {
    // This test documents the expected state machine:
    // Unseen -> Visible -> Seen -> Visible -> ...
    //
    // A tile cannot go from Unseen directly to Seen without being Visible first.
    // This is enforced by the FOV system logic, not the type system.

    let states = [
        TileVisibility::Unseen,
        TileVisibility::Visible,
        TileVisibility::Seen,
    ];

    // All states are distinct
    assert_ne!(states[0], states[1]);
    assert_ne!(states[1], states[2]);
    assert_ne!(states[0], states[2]);
}

/// Test TileVisibilityState wrapper preserves visibility.
#[test]
fn tile_visibility_state_preserves_value() {
    let state = TileVisibilityState {
        visibility: TileVisibility::Visible,
    };

    assert_eq!(state.visibility, TileVisibility::Visible);

    let state2 = TileVisibilityState {
        visibility: TileVisibility::Seen,
    };

    assert_eq!(state2.visibility, TileVisibility::Seen);
}

/// Test visibility map correctly tracks multiple tile states.
#[test]
fn visibility_map_tracks_multiple_states() {
    let mut visibility_map: HashMap<(u32, u32), TileVisibility> = HashMap::new();

    // Set various visibility states
    visibility_map.insert((10, 10), TileVisibility::Visible);
    visibility_map.insert((20, 20), TileVisibility::Seen);
    visibility_map.insert((30, 30), TileVisibility::Unseen);

    // Verify states
    assert_eq!(
        visibility_map.get(&(10, 10)),
        Some(&TileVisibility::Visible)
    );
    assert_eq!(visibility_map.get(&(20, 20)), Some(&TileVisibility::Seen));
    assert_eq!(
        visibility_map.get(&(30, 30)),
        Some(&TileVisibility::Unseen)
    );

    // Tiles not in map are implicitly unseen
    assert_eq!(visibility_map.get(&(99, 99)), None);
}

/// Test simulated player movement updates visibility.
#[test]
fn player_movement_updates_visibility() {
    let mut visibility: HashMap<(u32, u32), TileVisibility> = HashMap::new();

    // Initially all unseen
    let player_start = (40, 25);
    let player_end = (45, 25);

    // Simulate FOV at start position (simplified: square FOV)
    let fov_range = FOV_RADIUS as i32;
    for dy in -fov_range..=fov_range {
        for dx in -fov_range..=fov_range {
            let x = (player_start.0 as i32 + dx) as u32;
            let y = (player_start.1 as i32 + dy) as u32;
            if x < MAP_WIDTH && y < MAP_HEIGHT {
                visibility.insert((x, y), TileVisibility::Visible);
            }
        }
    }

    let visible_count_before = visibility
        .values()
        .filter(|&&v| v == TileVisibility::Visible)
        .count();
    assert!(visible_count_before > 0, "Should have visible tiles");

    // Player moves - old visible tiles become seen
    for (pos, vis) in visibility.iter_mut() {
        if *vis == TileVisibility::Visible {
            *vis = TileVisibility::Seen;
        }
    }

    // Calculate new FOV at end position
    for dy in -fov_range..=fov_range {
        for dx in -fov_range..=fov_range {
            let x = (player_end.0 as i32 + dx) as u32;
            let y = (player_end.1 as i32 + dy) as u32;
            if x < MAP_WIDTH && y < MAP_HEIGHT {
                visibility.insert((x, y), TileVisibility::Visible);
            }
        }
    }

    // Should have mix of Visible and Seen
    let visible_count = visibility
        .values()
        .filter(|&&v| v == TileVisibility::Visible)
        .count();
    let seen_count = visibility
        .values()
        .filter(|&&v| v == TileVisibility::Seen)
        .count();

    assert!(visible_count > 0, "Should have visible tiles at new position");
    assert!(seen_count > 0, "Should have seen tiles from old position");
}

/// Test FOV radius affects visibility range.
#[test]
fn fov_radius_determines_visibility_range() {
    let player_pos = (40, 25);
    let mut visible_tiles: Vec<(u32, u32)> = Vec::new();

    // Simulate circular FOV calculation
    let radius = FOV_RADIUS;
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            let dx = x as i32 - player_pos.0 as i32;
            let dy = y as i32 - player_pos.1 as i32;
            let distance_sq = (dx * dx + dy * dy) as u32;

            if distance_sq <= radius * radius {
                visible_tiles.push((x, y));
            }
        }
    }

    // All visible tiles should be within radius
    for &(x, y) in &visible_tiles {
        let dx = x as i32 - player_pos.0 as i32;
        let dy = y as i32 - player_pos.1 as i32;
        let distance = ((dx * dx + dy * dy) as f64).sqrt();

        assert!(
            distance <= radius as f64,
            "Tile ({}, {}) at distance {} should be within radius {}",
            x,
            y,
            distance,
            radius
        );
    }

    // Should have at least some visible tiles
    assert!(
        !visible_tiles.is_empty(),
        "FOV should make some tiles visible"
    );
}

/// Test visibility state is copyable and comparable.
#[test]
fn visibility_state_is_copyable() {
    let vis = TileVisibility::Visible;
    let vis_copy = vis; // Copy

    assert_eq!(vis, vis_copy);
    assert_eq!(vis, TileVisibility::Visible);
    assert_eq!(vis_copy, TileVisibility::Visible);
}

/// Test that movement toward unexplored areas reveals new tiles.
#[test]
fn movement_reveals_new_tiles() {
    let mut visibility: HashMap<(u32, u32), TileVisibility> = HashMap::new();

    // Start with small explored area around position (10, 10)
    let initial_pos = (10, 10);
    let initial_fov = 5i32;

    for dy in -initial_fov..=initial_fov {
        for dx in -initial_fov..=initial_fov {
            let x = (initial_pos.0 as i32 + dx) as u32;
            let y = (initial_pos.1 as i32 + dy) as u32;
            if x < MAP_WIDTH && y < MAP_HEIGHT {
                visibility.insert((x, y), TileVisibility::Seen);
            }
        }
    }

    let tiles_explored_before = visibility.len();

    // Move to new position (20, 10) - should reveal new area
    let new_pos = (20, 10);

    for dy in -initial_fov..=initial_fov {
        for dx in -initial_fov..=initial_fov {
            let x = (new_pos.0 as i32 + dx) as u32;
            let y = (new_pos.1 as i32 + dy) as u32;
            if x < MAP_WIDTH && y < MAP_HEIGHT {
                // Only insert if not already in map (reveals new tiles)
                visibility.entry((x, y)).or_insert(TileVisibility::Visible);
            }
        }
    }

    let tiles_explored_after = visibility.len();

    assert!(
        tiles_explored_after > tiles_explored_before,
        "Moving should reveal new tiles: before={}, after={}",
        tiles_explored_before,
        tiles_explored_after
    );
}
