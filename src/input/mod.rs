//! Input handling module for player controls.
//!
//! Split into focused submodules:
//! - `movement`: Arrow key movement and animation
//! - `interaction`: Stair usage and auto-pathfinding to stairs
//! - `autoexplore`: Toggle exploration mode
//! - `debug`: Development/debug key bindings

use bevy::prelude::*;

mod movement;
mod interaction;
mod autoexplore;
mod debug;

// Re-export all public items for backwards compatibility
pub use movement::{detect_movement_input, handle_movement_input, PlayerMoveIntent};
pub use interaction::{handle_stair_interaction, run_auto_move_to_stair};
pub use autoexplore::toggle_autoexplore;
pub use debug::{debug_map_regeneration, debug_biome_cycling};

// ============================================================================
// KEY BINDINGS RESOURCE (shared across all input submodules)
// ============================================================================

#[derive(Resource)]
pub struct KeyBindings {
    // Movement keys
    pub move_up: Vec<KeyCode>,
    pub move_down: Vec<KeyCode>,
    pub move_left: Vec<KeyCode>,
    pub move_right: Vec<KeyCode>,

    // Level transition keys
    pub stair_up: Vec<KeyCode>,
    pub stair_down: Vec<KeyCode>,

    // Autoexplore keys
    pub toggle_autoexplore: Vec<KeyCode>,
    pub cancel_autoexplore: Vec<KeyCode>,

    // Debug keys
    pub regenerate_map: Vec<KeyCode>,
    pub cycle_biome: Vec<KeyCode>,
    pub toggle_fov: Vec<KeyCode>,
    pub show_los_cache: Vec<KeyCode>,
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            // Arrow keys only for movement
            move_up: vec![KeyCode::ArrowUp],
            move_down: vec![KeyCode::ArrowDown],
            move_left: vec![KeyCode::ArrowLeft],
            move_right: vec![KeyCode::ArrowRight],

            // Level transitions
            stair_up: vec![KeyCode::KeyS],
            stair_down: vec![KeyCode::KeyD],

            // Autoexplore
            toggle_autoexplore: vec![KeyCode::KeyA],
            cancel_autoexplore: vec![KeyCode::Escape, KeyCode::Space],

            // Debug
            regenerate_map: vec![KeyCode::KeyR],
            cycle_biome: vec![KeyCode::KeyB],
            toggle_fov: vec![KeyCode::KeyO],
            show_los_cache: vec![KeyCode::KeyL],
        }
    }
}

impl KeyBindings {
    pub(crate) fn is_pressed(&self, keys: &[KeyCode], input: &ButtonInput<KeyCode>) -> bool {
        keys.iter().any(|key| input.pressed(*key))
    }

    pub(crate) fn is_just_pressed(&self, keys: &[KeyCode], input: &ButtonInput<KeyCode>) -> bool {
        keys.iter().any(|key| input.just_pressed(*key))
    }
}
