//! Debug input handling for development features.

use bevy::prelude::*;

use crate::biome::BiomeType;
use crate::components::CurrentLevel;
use crate::events::RegenerateMapEvent;

use super::KeyBindings;

// ============================================================================
// DEBUG INPUT SYSTEMS
// ============================================================================

pub fn debug_map_regeneration(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    key_bindings: Res<KeyBindings>,
    mut regenerate_events: EventWriter<RegenerateMapEvent>,
) {
    let shift_held = keyboard_input.pressed(KeyCode::ShiftLeft) || keyboard_input.pressed(KeyCode::ShiftRight);

    if key_bindings.is_just_pressed(&key_bindings.regenerate_map, &keyboard_input) && shift_held {
        println!("Regenerating current level map...");
        regenerate_events.write(RegenerateMapEvent);
    }
}

pub fn debug_biome_cycling(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    key_bindings: Res<KeyBindings>,
    mut current_level: ResMut<CurrentLevel>,
    mut regenerate_events: EventWriter<RegenerateMapEvent>,
) {
    let shift_held = keyboard_input.pressed(KeyCode::ShiftLeft) || keyboard_input.pressed(KeyCode::ShiftRight);

    if key_bindings.is_just_pressed(&key_bindings.cycle_biome, &keyboard_input) && shift_held {
        // Cycle between implemented biomes: Caverns -> Cinder Gaol -> Underglade -> back to Caverns
        current_level.biome = match current_level.biome {
            BiomeType::Caverns => {
                println!("Cycling from Caverns to Cinder Gaol");
                BiomeType::CinderGaol
            },
            BiomeType::CinderGaol => {
                println!("Cycling from Cinder Gaol to Underglade");
                BiomeType::Underglade
            },
            BiomeType::Underglade => {
                println!("Cycling from Underglade back to Caverns");
                BiomeType::Caverns
            },
            _ => BiomeType::Caverns, // Fallback to Caverns for other biomes
        };

        println!("Current biome: {:?}", current_level.biome);
        println!("Regenerating map with new biome...");
        regenerate_events.write(RegenerateMapEvent);
    }
}
