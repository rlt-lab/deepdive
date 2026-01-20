//! Stair interaction and auto-pathfinding to stairs.

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use std::collections::{HashMap, VecDeque};

use crate::components::{
    Player, MovementAnimation, Autoexplore, AutoMoveToStair,
    TileVisibilityState, TileVisibility, TileType, CurrentLevel, LevelMaps,
};
use crate::constants::AUTOEXPLORE_ANIM_TIMER;
use crate::events::{LevelChangeEvent, SpawnPosition};
use crate::level_manager::capture_tile_visibility;
use crate::map::GameMap;
use crate::player::find_path;

use super::KeyBindings;

// ============================================================================
// STAIR INTERACTION SYSTEM
// ============================================================================

pub fn handle_stair_interaction(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    key_bindings: Res<KeyBindings>,
    player_query: Query<(Entity, &Player, Option<&Autoexplore>, Option<&AutoMoveToStair>)>,
    tile_visibility_query: Query<(&TilePos, &TileVisibilityState)>,
    map: Res<GameMap>,
    current_level: Res<CurrentLevel>,
    mut level_maps: ResMut<LevelMaps>,
    mut level_change_events: EventWriter<LevelChangeEvent>,
) {
    if let Ok((entity, player, autoexplore_opt, auto_move_opt)) = player_query.single() {
        let tile_type = map.get(player.x, player.y);

        // Check for move up (S key)
        if key_bindings.is_just_pressed(&key_bindings.stair_up, &keyboard_input) {
            // If standing on up stairs, use them
            if tile_type == TileType::StairUp {
                if current_level.level > 0 {
                    println!("Going up to level {}", current_level.level - 1);
                    // Save current map with tile visibility
                    let current_visibility = capture_tile_visibility(&tile_visibility_query, map.width, map.height);
                    level_maps.maps.insert(current_level.level, map.to_saved_data(current_level.biome, current_visibility));
                    // Trigger level change
                    level_change_events.write(LevelChangeEvent {
                        new_level: current_level.level - 1,
                        spawn_position: SpawnPosition::StairDown,
                    });
                } else {
                    println!("Cannot go up from the surface!");
                }
            } else {
                // Not on stairs - try to auto-move to nearest discovered up stairwell
                handle_auto_move_to_stair(
                    &mut commands,
                    entity,
                    player,
                    TileType::StairUp,
                    &tile_visibility_query,
                    &map,
                    autoexplore_opt,
                    auto_move_opt,
                );
            }
        }

        // Check for move down (D key)
        if key_bindings.is_just_pressed(&key_bindings.stair_down, &keyboard_input) {
            // If standing on down stairs, use them
            if tile_type == TileType::StairDown {
                if current_level.level < 50 {
                    println!("Going down to level {}", current_level.level + 1);
                    // Save current map with tile visibility
                    let current_visibility = capture_tile_visibility(&tile_visibility_query, map.width, map.height);
                    level_maps.maps.insert(current_level.level, map.to_saved_data(current_level.biome, current_visibility));
                    // Trigger level change
                    level_change_events.write(LevelChangeEvent {
                        new_level: current_level.level + 1,
                        spawn_position: SpawnPosition::StairUp,
                    });
                } else {
                    println!("Cannot go deeper - you've reached the bottom!");
                }
            } else {
                // Not on stairs - try to auto-move to nearest discovered down stairwell
                handle_auto_move_to_stair(
                    &mut commands,
                    entity,
                    player,
                    TileType::StairDown,
                    &tile_visibility_query,
                    &map,
                    autoexplore_opt,
                    auto_move_opt,
                );
            }
        }
    }
}

/// Helper to initiate auto-movement to a discovered stairwell
fn handle_auto_move_to_stair(
    commands: &mut Commands,
    entity: Entity,
    player: &Player,
    stair_type: TileType,
    tile_visibility_query: &Query<(&TilePos, &TileVisibilityState)>,
    map: &GameMap,
    autoexplore_opt: Option<&Autoexplore>,
    auto_move_opt: Option<&AutoMoveToStair>,
) {
    if let Some(nearest_stair) = find_nearest_discovered_stairwell(
        player,
        stair_type,
        tile_visibility_query,
        map,
    ) {
        let path = VecDeque::from(find_path((player.x, player.y), nearest_stair, map));
        if !path.is_empty() {
            // Cancel any existing auto-movement
            if autoexplore_opt.is_some() {
                commands.entity(entity).remove::<Autoexplore>();
            }
            if auto_move_opt.is_some() {
                commands.entity(entity).remove::<AutoMoveToStair>();
            }

            let direction = if stair_type == TileType::StairUp { "up" } else { "down" };
            println!("Auto-moving to discovered {} stairwell at ({}, {})", direction, nearest_stair.0, nearest_stair.1);
            commands.entity(entity).insert(AutoMoveToStair::new(
                nearest_stair,
                path,
                stair_type,
            ));
        } else {
            println!("No path to {} stairwell!", if stair_type == TileType::StairUp { "up" } else { "down" });
        }
    } else {
        println!("No discovered {} stairwell found. Explore to find stairs.", if stair_type == TileType::StairUp { "up" } else { "down" });
    }
}

/// Find nearest discovered stairwell of a specific type.
///
/// Uses O(1) visibility lookups via HashMap instead of linear query scans.
fn find_nearest_discovered_stairwell(
    player: &Player,
    stair_type: TileType,
    tile_visibility_query: &Query<(&TilePos, &TileVisibilityState)>,
    map: &GameMap,
) -> Option<(u32, u32)> {
    // Build visibility HashMap once for O(1) lookups (instead of O(n) per stair)
    let visibility_map: HashMap<(u32, u32), TileVisibility> = tile_visibility_query
        .iter()
        .map(|(pos, state)| ((pos.x, pos.y), state.visibility))
        .collect();

    let mut nearest_stair: Option<(u32, u32)> = None;
    let mut min_distance = f32::INFINITY;

    // Search all tiles for discovered stairs of the requested type
    for y in 0..map.height {
        for x in 0..map.width {
            // Check if this tile is the stair type we're looking for
            if map.get(x, y) != stair_type {
                continue;
            }

            // O(1) visibility lookup - check if stairwell has been discovered
            let is_discovered = visibility_map
                .get(&(x, y))
                .map_or(false, |&v| v == TileVisibility::Visible || v == TileVisibility::Seen);

            if !is_discovered {
                continue;
            }

            // Calculate Manhattan distance
            let distance = ((x as i32 - player.x as i32).abs() + (y as i32 - player.y as i32).abs()) as f32;

            if distance < min_distance {
                min_distance = distance;
                nearest_stair = Some((x, y));
            }
        }
    }

    nearest_stair
}

// ============================================================================
// AUTO-MOVE TO STAIR SYSTEM
// ============================================================================

/// System to handle auto-movement to discovered stairwells
pub fn run_auto_move_to_stair(
    mut commands: Commands,
    time: Res<Time>,
    mut player_query: Query<(Entity, &mut Player, &mut AutoMoveToStair, &mut Sprite), Without<MovementAnimation>>,
    map: Res<GameMap>,
) {
    if let Ok((entity, mut player, mut auto_move, mut sprite)) = player_query.single_mut() {
        // Tick timer
        auto_move.move_timer.tick(time.delta());

        if !auto_move.move_timer.just_finished() {
            return;
        }

        // Get next step in path
        if let Some(next_pos) = auto_move.path.front().copied() {
            // Check if we can move to next position
            if map.get(next_pos.0, next_pos.1) != TileType::Wall {
                // Calculate animation positions
                let start_world = map.grid_to_world(player.x, player.y);
                let end_world = map.grid_to_world(next_pos.0, next_pos.1);

                // Update sprite facing
                if next_pos.0 < player.x {
                    sprite.flip_x = false; // Moving left
                } else if next_pos.0 > player.x {
                    sprite.flip_x = true; // Moving right
                }

                // Move player
                player.x = next_pos.0;
                player.y = next_pos.1;

                // Add animation
                commands.entity(entity).insert(MovementAnimation {
                    start_pos: Vec3::new(start_world.x, start_world.y, 1.0),
                    end_pos: Vec3::new(end_world.x, end_world.y, 1.0),
                    timer: Timer::from_seconds(AUTOEXPLORE_ANIM_TIMER, TimerMode::Once),
                });

                // Remove this step from path (O(1) with VecDeque)
                auto_move.path.pop_front();
            } else {
                // Path blocked, cancel auto-move
                println!("Path to stairwell blocked!");
                commands.entity(entity).remove::<AutoMoveToStair>();
            }
        } else {
            // Reached destination
            let current_tile = map.get(player.x, player.y);
            if current_tile == auto_move.stair_type {
                println!("Reached {} stairwell! Press {} to use it.",
                    if auto_move.stair_type == TileType::StairUp { "up" } else { "down" },
                    if auto_move.stair_type == TileType::StairUp { "S" } else { "D" }
                );
            }
            commands.entity(entity).remove::<AutoMoveToStair>();
        }
    }
}
