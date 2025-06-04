use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use serde::{Deserialize, Serialize};

use crate::components::{Player, TileType, CurrentLevel};
use crate::map::GameMap;
use crate::biome::BiomeType;

#[derive(Component, Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum TileVisibility {
    Unseen,
    Seen,
    Visible,
}

#[derive(Component)]
pub struct TileVisibilityState {
    pub visibility: TileVisibility,
}

#[derive(Resource)]
pub struct FovSettings {
    pub radius: u32,
    pub debug_reveal_all: bool,
    pub needs_recalculation: bool,
    pub debug_mode_applied: bool, // Track if debug mode has been applied
}

impl Default for FovSettings {
    fn default() -> Self {
        Self {
            radius: 8,
            debug_reveal_all: false,
            needs_recalculation: true,
            debug_mode_applied: false,
        }
    }
}

pub struct FovPlugin;

impl Plugin for FovPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<FovSettings>()
            .add_systems(Update, (
                detect_player_movement,
                calculate_fov.run_if(should_recalculate_fov),
                update_tile_visibility,
                handle_fov_debug_controls,
            ).chain());
    }
}

// System to detect when player has moved to trigger FOV recalculation
pub fn detect_player_movement(
    player_query: Query<&Player, Changed<Player>>,
    mut fov_settings: ResMut<FovSettings>,
) {
    if let Ok(_player) = player_query.single() {
        fov_settings.needs_recalculation = true;
    }
}

// Condition function to check if FOV needs recalculation
pub fn should_recalculate_fov(
    fov_settings: Res<FovSettings>,
    map: Option<Res<GameMap>>,
) -> bool {
    map.is_some() && (
        fov_settings.needs_recalculation || 
        (fov_settings.debug_reveal_all && !fov_settings.debug_mode_applied)
    )
}

// Simple FOV calculation using basic line-of-sight
pub fn calculate_fov(
    player_query: Query<&Player>,
    map: Res<GameMap>,
    mut fov_settings: ResMut<FovSettings>,
    mut tile_query: Query<(&TilePos, &mut TileVisibilityState)>,
) {
    let Ok(player) = player_query.single() else { return; };
    
    // If debug mode is on, reveal all tiles (only once)
    if fov_settings.debug_reveal_all {
        if !fov_settings.debug_mode_applied {
            for (_, mut visibility_state) in tile_query.iter_mut() {
                visibility_state.visibility = TileVisibility::Visible;
            }
            fov_settings.debug_mode_applied = true;
        }
        fov_settings.needs_recalculation = false;
        return;
    }
    
    // Reset debug mode tracking when not in debug mode
    if fov_settings.debug_mode_applied {
        fov_settings.debug_mode_applied = false;
    }
    
    let player_x = player.x as i32;
    let player_y = player.y as i32;
    let radius = fov_settings.radius as i32;
    let radius_squared = radius.pow(2);
    
    // More efficient single-pass approach: iterate through tiles and update visibility
    for (tile_pos, mut visibility_state) in tile_query.iter_mut() {
        let tile_x = tile_pos.x as i32;
        let tile_y = tile_pos.y as i32;
        
        // Calculate distance squared (avoid sqrt for performance)
        let distance_squared = (tile_x - player_x).pow(2) + (tile_y - player_y).pow(2);
        
        if distance_squared <= radius_squared {
            // Check line of sight from player to tile
            if has_line_of_sight(&map, player_x, player_y, tile_x, tile_y) {
                visibility_state.visibility = TileVisibility::Visible;
            } else {
                // If tile was visible, make it seen; don't change unseen tiles
                if visibility_state.visibility == TileVisibility::Visible {
                    visibility_state.visibility = TileVisibility::Seen;
                }
            }
        } else {
            // Outside radius: if tile was visible, make it seen
            if visibility_state.visibility == TileVisibility::Visible {
                visibility_state.visibility = TileVisibility::Seen;
            }
        }
    }
    
    // Mark that recalculation is complete
    fov_settings.needs_recalculation = false;
}

// Simple line-of-sight check using Bresenham's line algorithm
fn has_line_of_sight(map: &GameMap, x0: i32, y0: i32, x1: i32, y1: i32) -> bool {
    let mut x = x0;
    let mut y = y0;
    
    let dx = (x1 - x0).abs();
    let dy = (y1 - y0).abs();
    
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    
    let mut err = dx - dy;
    
    loop {
        // Check if current position is a wall (blocks vision)
        if x >= 0 && x < map.width as i32 && y >= 0 && y < map.height as i32 {
            if map.tiles[y as usize][x as usize] == TileType::Wall {
                // Don't block vision at the target tile itself
                if x != x1 || y != y1 {
                    return false;
                }
            }
        }
        
        if x == x1 && y == y1 { break; }
        
        let e2 = 2 * err;
        if e2 > -dy {
            err -= dy;
            x += sx;
        }
        if e2 < dx {
            err += dx;
            y += sy;
        }
    }
    
    true
}

pub fn update_tile_visibility(
    mut tile_query: Query<(&mut TileColor, &TileVisibilityState), Changed<TileVisibilityState>>,
    current_level: Res<CurrentLevel>,
) {
    // Get biome-specific color tint
    let biome_tint = get_biome_color_tint(current_level.biome);
    
    for (mut tile_color, visibility_state) in tile_query.iter_mut() {
        match visibility_state.visibility {
            TileVisibility::Unseen => {
                // Completely dark/invisible
                tile_color.0 = Color::srgb(0.0, 0.0, 0.0);
            },
            TileVisibility::Seen => {
                // Darkened/grayed out for memory, with biome tint
                let base_color = Color::srgb(0.3, 0.3, 0.4);
                tile_color.0 = apply_color_tint(base_color, biome_tint, 0.4);
            },
            TileVisibility::Visible => {
                // Full visibility with biome tint
                tile_color.0 = apply_color_tint(Color::WHITE, biome_tint, 0.6);
            },
        }
    }
}

// Helper function to get biome-specific color tint
fn get_biome_color_tint(biome: BiomeType) -> Color {
    match biome {
        BiomeType::CinderGaol => Color::srgb(1.3, 0.7, 0.7), // Red tint for fire/prison theme
        BiomeType::NetherGrange => Color::srgb(1.4, 0.6, 0.4), // Orange-red for hellish landscape
        BiomeType::Underglade => Color::srgb(0.8, 1.2, 0.9), // Green tint for lush biome
        BiomeType::FungalDeep => Color::srgb(0.9, 0.8, 1.3), // Purple tint for spores
        BiomeType::AbyssalHold => Color::srgb(0.7, 0.7, 1.2), // Blue tint for dark waters
        BiomeType::StygianPool => Color::srgb(0.6, 0.8, 1.3), // Cyan tint for underground lake
        _ => Color::WHITE, // No tint for other biomes
    }
}

// Helper function to apply color tint with intensity
fn apply_color_tint(base_color: Color, tint: Color, intensity: f32) -> Color {
    let base = base_color.to_linear();
    let tint_linear = tint.to_linear();
    
    // Blend base color with tint based on intensity
    let r = base.red * (1.0 - intensity + intensity * tint_linear.red);
    let g = base.green * (1.0 - intensity + intensity * tint_linear.green);
    let b = base.blue * (1.0 - intensity + intensity * tint_linear.blue);
    
    Color::linear_rgb(r, g, b)
}

pub fn handle_fov_debug_controls(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut fov_settings: ResMut<FovSettings>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyO) && 
       (keyboard_input.pressed(KeyCode::ShiftLeft) || keyboard_input.pressed(KeyCode::ShiftRight)) {
        fov_settings.debug_reveal_all = !fov_settings.debug_reveal_all;
        fov_settings.debug_mode_applied = false; // Reset flag to trigger recalculation
        fov_settings.needs_recalculation = true;
        println!("FOV debug reveal: {}", if fov_settings.debug_reveal_all { "ON" } else { "OFF" });
    }
}
