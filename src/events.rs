//! Game events for level transitions and map regeneration.
//!
//! These events are used by both input_handler.rs (sender) and level_manager.rs (receiver).

use bevy::prelude::*;

/// Event to trigger a level change (e.g., using stairs).
#[derive(Event)]
pub struct LevelChangeEvent {
    pub new_level: u32,
    pub spawn_position: SpawnPosition,
}

/// Event to regenerate the current map (debug feature).
#[derive(Event)]
pub struct RegenerateMapEvent;

/// Where to spawn the player after a level change.
#[derive(Clone, Copy)]
pub enum SpawnPosition {
    /// Spawn at the upward staircase
    StairUp,
    /// Spawn at the downward staircase
    StairDown,
    /// Spawn at map center (fallback)
    Center,
}
