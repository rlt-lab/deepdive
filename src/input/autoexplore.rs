//! Autoexplore toggle input handling.

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::components::{Player, Autoexplore, TileVisibilityState};
use crate::map::GameMap;
use crate::player::count_unexplored_tiles;

use super::KeyBindings;

// ============================================================================
// AUTOEXPLORE INPUT SYSTEM
// ============================================================================

pub fn toggle_autoexplore(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    key_bindings: Res<KeyBindings>,
    mut commands: Commands,
    mut player_query: Query<(Entity, &Player, Option<&Autoexplore>)>,
    tile_visibility_query: Query<(&TilePos, &TileVisibilityState)>,
    map: Res<GameMap>,
) {
    // Check for A to toggle, or ESC/Space to cancel
    let toggle_pressed = key_bindings.is_just_pressed(&key_bindings.toggle_autoexplore, &keyboard_input);
    let cancel_pressed = key_bindings.is_just_pressed(&key_bindings.cancel_autoexplore, &keyboard_input);

    if toggle_pressed || cancel_pressed {
        if let Ok((entity, _player, autoexplore_opt)) = player_query.single_mut() {
            if autoexplore_opt.is_some() {
                // Remove component entirely to stop autoexplore
                commands.entity(entity).remove::<Autoexplore>();
                println!("Autoexplore disabled");
            } else if toggle_pressed {
                // Only enable on A, not on ESC
                // Check if there are unexplored tiles
                let unexplored_count = count_unexplored_tiles(&tile_visibility_query, &map);
                if unexplored_count > 0 {
                    commands.entity(entity).insert(Autoexplore::default());
                    println!("Autoexplore enabled - {} tiles to explore", unexplored_count);
                } else {
                    println!("Map fully explored!");
                }
            }
        }
    }
}
