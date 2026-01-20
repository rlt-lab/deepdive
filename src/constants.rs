//! Game constants extracted from magic numbers across the codebase.
//!
//! Centralizing these values makes them easy to find, modify, and test.

// =============================================================================
// Map Dimensions
// =============================================================================

/// Width of the game map in tiles.
pub const MAP_WIDTH: u32 = 80;

/// Height of the game map in tiles.
pub const MAP_HEIGHT: u32 = 50;

/// Maximum number of tiles in the tile pool (MAP_WIDTH * MAP_HEIGHT).
pub const MAX_TILE_POOL: usize = 4000;

// =============================================================================
// Tile & Rendering
// =============================================================================

/// Size of each tile in pixels.
pub const TILE_SIZE: f32 = 32.0;

// =============================================================================
// Window
// =============================================================================

/// Default window width in pixels.
pub const WINDOW_WIDTH: f32 = 1400.0;

/// Default window height in pixels.
pub const WINDOW_HEIGHT: f32 = 800.0;

// =============================================================================
// Camera
// =============================================================================

/// Padding around the map for camera bounds (in pixels).
pub const CAMERA_PADDING: f32 = 200.0;

/// Zoom multiplier for zoom in/out operations.
pub const ZOOM_SPEED: f32 = 1.2;

/// Minimum zoom level (most zoomed out).
pub const ZOOM_MIN: f32 = 0.5;

/// Maximum zoom level (most zoomed in).
pub const ZOOM_MAX: f32 = 3.0;

// =============================================================================
// Field of View
// =============================================================================

/// Radius of player's field of view in tiles.
pub const FOV_RADIUS: u32 = 20;

// =============================================================================
// Timers
// =============================================================================

/// Timer duration for hold-to-move input (seconds).
pub const HOLD_TO_MOVE_TIMER: f32 = 0.15;

/// Timer duration for autoexplore animation steps (seconds).
pub const AUTOEXPLORE_ANIM_TIMER: f32 = 0.05;

/// Timer duration for hop animation (seconds).
pub const HOP_ANIM_TIMER: f32 = 0.1;

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_dimensions_are_valid() {
        assert!(MAP_WIDTH > 0, "MAP_WIDTH must be positive");
        assert!(MAP_HEIGHT > 0, "MAP_HEIGHT must be positive");
    }

    #[test]
    fn max_tile_pool_matches_map_size() {
        assert_eq!(
            MAX_TILE_POOL,
            (MAP_WIDTH * MAP_HEIGHT) as usize,
            "MAX_TILE_POOL should equal MAP_WIDTH * MAP_HEIGHT"
        );
    }

    #[test]
    fn tile_size_is_positive() {
        assert!(TILE_SIZE > 0.0, "TILE_SIZE must be positive");
    }

    #[test]
    fn window_dimensions_are_valid() {
        assert!(WINDOW_WIDTH > 0.0, "WINDOW_WIDTH must be positive");
        assert!(WINDOW_HEIGHT > 0.0, "WINDOW_HEIGHT must be positive");
    }

    #[test]
    fn camera_constants_are_valid() {
        assert!(CAMERA_PADDING >= 0.0, "CAMERA_PADDING must be non-negative");
        assert!(ZOOM_SPEED > 1.0, "ZOOM_SPEED must be > 1.0 to have effect");
        assert!(ZOOM_MIN > 0.0, "ZOOM_MIN must be positive");
        assert!(ZOOM_MAX > ZOOM_MIN, "ZOOM_MAX must be greater than ZOOM_MIN");
    }

    #[test]
    fn fov_radius_is_valid() {
        assert!(FOV_RADIUS > 0, "FOV_RADIUS must be positive");
        assert!(
            FOV_RADIUS < MAP_WIDTH && FOV_RADIUS < MAP_HEIGHT,
            "FOV_RADIUS should be smaller than map dimensions"
        );
    }

    #[test]
    fn timer_durations_are_valid() {
        assert!(HOLD_TO_MOVE_TIMER > 0.0, "HOLD_TO_MOVE_TIMER must be positive");
        assert!(AUTOEXPLORE_ANIM_TIMER > 0.0, "AUTOEXPLORE_ANIM_TIMER must be positive");
        assert!(HOP_ANIM_TIMER > 0.0, "HOP_ANIM_TIMER must be positive");
    }

    #[test]
    fn autoexplore_faster_than_manual() {
        assert!(
            AUTOEXPLORE_ANIM_TIMER < HOLD_TO_MOVE_TIMER,
            "Autoexplore should be faster than manual movement"
        );
    }
}
