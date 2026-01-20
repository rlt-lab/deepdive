use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::components::{Player, MovementInput, MovementAnimation, Autoexplore, AutoMoveToStair, TileVisibilityState, TileVisibility, TileType, CurrentLevel, LevelMaps};
use crate::constants::{TILE_SIZE, HOP_ANIM_TIMER, AUTOEXPLORE_ANIM_TIMER};
use crate::map::GameMap;
use crate::biome::BiomeType;
use crate::level_manager::capture_tile_visibility;
use crate::player::{count_unexplored_tiles, find_path};

// ============================================================================
// INPUT EVENTS
// ============================================================================

#[derive(Event)]
pub struct PlayerMoveIntent {
    pub direction: MoveDirection,
}

#[derive(Clone, Copy, PartialEq)]
pub enum MoveDirection {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Event)]
pub struct LevelChangeEvent {
    pub new_level: u32,
    pub spawn_position: SpawnPosition,
}

#[derive(Event)]
pub struct RegenerateMapEvent;

#[derive(Clone, Copy)]
pub enum SpawnPosition {
    StairUp,
    StairDown,
    Center,
}

// ============================================================================
// KEY BINDINGS RESOURCE
// ============================================================================

#[derive(Resource)]
pub struct KeyBindings {
    // Movement keys
    pub move_up: Vec<KeyCode>,
    pub move_down: Vec<KeyCode>,
    pub move_left: Vec<KeyCode>,
    pub move_right: Vec<KeyCode>,
    
    // Level transition keys
    pub stair_up: Vec<KeyCode>,      // S key - go up stairs
    pub stair_down: Vec<KeyCode>,    // X key - go down stairs
    
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
            stair_up: vec![KeyCode::KeyS],      // S - go up stairs
            stair_down: vec![KeyCode::KeyD],    // D - go down stairs
            
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
    fn is_pressed(&self, keys: &[KeyCode], input: &ButtonInput<KeyCode>) -> bool {
        keys.iter().any(|key| input.pressed(*key))
    }
    
    fn is_just_pressed(&self, keys: &[KeyCode], input: &ButtonInput<KeyCode>) -> bool {
        keys.iter().any(|key| input.just_pressed(*key))
    }
}

// ============================================================================
// MOVEMENT INPUT SYSTEMS
// ============================================================================

/// Event-based input detection - only fires when key state changes
pub fn detect_movement_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    key_bindings: Res<KeyBindings>,
    time: Res<Time>,
    mut player_query: Query<&mut MovementInput>,
    mut move_events: EventWriter<PlayerMoveIntent>,
) {
    if let Ok(mut movement_input) = player_query.single_mut() {
        // Check for any movement key being pressed (arrow keys only, no WASD)
        let up_pressed = key_bindings.is_pressed(&key_bindings.move_up, &keyboard_input);
        let down_pressed = key_bindings.is_pressed(&key_bindings.move_down, &keyboard_input);
        let left_pressed = key_bindings.is_pressed(&key_bindings.move_left, &keyboard_input);
        let right_pressed = key_bindings.is_pressed(&key_bindings.move_right, &keyboard_input);

        let any_movement_key = up_pressed || down_pressed || left_pressed || right_pressed;

        // Handle initial key press or continuous movement
        let should_move = if any_movement_key {
            if !movement_input.is_holding {
                // First press - fire event immediately
                movement_input.is_holding = true;
                movement_input.move_timer.reset();
                true
            } else {
                // Continuous movement - check timer
                movement_input.move_timer.tick(time.delta());
                if movement_input.move_timer.finished() {
                    movement_input.move_timer.reset();
                    true
                } else {
                    false
                }
            }
        } else {
            movement_input.is_holding = false;
            false
        };

        // Fire movement intent events based on key priority
        if should_move {
            // Vertical movement (prioritized)
            if up_pressed {
                move_events.write(PlayerMoveIntent { direction: MoveDirection::Up });
            } else if down_pressed {
                move_events.write(PlayerMoveIntent { direction: MoveDirection::Down });
            }

            // Horizontal movement
            if left_pressed {
                move_events.write(PlayerMoveIntent { direction: MoveDirection::Left });
            } else if right_pressed {
                move_events.write(PlayerMoveIntent { direction: MoveDirection::Right });
            }
        }
    }
}

/// Process movement intent events
pub fn handle_movement_input(
    mut commands: Commands,
    mut player_query: Query<(Entity, &mut Player, &mut Sprite, Option<&Autoexplore>), Without<MovementAnimation>>,
    mut move_events: EventReader<PlayerMoveIntent>,
    map: Res<GameMap>,
) {
    if let Ok((entity, mut player, mut sprite, autoexplore_opt)) = player_query.single_mut() {
        // Cancel autoexplore if player manually moves
        if move_events.len() > 0 && autoexplore_opt.is_some() {
            commands.entity(entity).remove::<Autoexplore>();
            println!("Autoexplore cancelled by manual input");
        }

        // Process all movement events for this frame
        for event in move_events.read() {
            let mut movement_attempted = false;
            let mut new_x = player.x;
            let mut new_y = player.y;
            let mut flip_sprite_opt: Option<bool> = None;

            // Apply movement based on direction
            match event.direction {
                MoveDirection::Up => {
                    if new_y < map.height - 1 {
                        new_y += 1;
                        movement_attempted = true;
                    }
                }
                MoveDirection::Down => {
                    if new_y > 0 {
                        new_y -= 1;
                        movement_attempted = true;
                    }
                }
                MoveDirection::Left => {
                    if new_x > 0 {
                        new_x -= 1;
                        movement_attempted = true;
                        flip_sprite_opt = Some(false); // No flip for left (natural direction)
                    }
                }
                MoveDirection::Right => {
                    if new_x < map.width - 1 {
                        new_x += 1;
                        movement_attempted = true;
                        flip_sprite_opt = Some(true); // Flip for right (face right)
                    }
                }
            }

            // Check collision with walls and apply movement
            if movement_attempted && map.get(new_x, new_y) != crate::components::TileType::Wall {
                // Calculate start and end positions for animation
                let start_world_x = (player.x as f32 - (map.width as f32 / 2.0 - 0.5)) * TILE_SIZE;
                let start_world_y = (player.y as f32 - (map.height as f32 / 2.0 - 0.5)) * TILE_SIZE;
                let end_world_x = (new_x as f32 - (map.width as f32 / 2.0 - 0.5)) * TILE_SIZE;
                let end_world_y = (new_y as f32 - (map.height as f32 / 2.0 - 0.5)) * TILE_SIZE;

                // Update player grid position
                player.x = new_x;
                player.y = new_y;

                // Handle sprite flipping
                if let Some(flip) = flip_sprite_opt {
                    sprite.flip_x = flip;
                }

                // Add movement animation component
                commands.entity(entity).insert(MovementAnimation {
                    start_pos: Vec3::new(start_world_x, start_world_y, 1.0),
                    end_pos: Vec3::new(end_world_x, end_world_y, 1.0),
                    timer: Timer::from_seconds(HOP_ANIM_TIMER, TimerMode::Once),
                });

                println!("Player moved to ({}, {})", new_x, new_y);
            } else if movement_attempted {
                println!("Cannot move to ({}, {}) - wall detected", new_x, new_y);
            }
        }
    }
}

// ============================================================================
// INTERACTION INPUT SYSTEMS
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
                if let Some(nearest_stair) = find_nearest_discovered_stairwell(
                    &player,
                    TileType::StairUp,
                    &tile_visibility_query,
                    &map,
                ) {
                    let path = find_path((player.x, player.y), nearest_stair, &map);
                    if !path.is_empty() {
                        // Cancel any existing auto-movement
                        if autoexplore_opt.is_some() {
                            commands.entity(entity).remove::<Autoexplore>();
                        }
                        if auto_move_opt.is_some() {
                            commands.entity(entity).remove::<AutoMoveToStair>();
                        }
                        
                        println!("Auto-moving to discovered up stairwell at ({}, {})", nearest_stair.0, nearest_stair.1);
                        commands.entity(entity).insert(AutoMoveToStair::new(
                            nearest_stair,
                            path,
                            TileType::StairUp,
                        ));
                    } else {
                        println!("No path to up stairwell!");
                    }
                } else {
                    println!("No discovered up stairwell found. Explore to find stairs.");
                }
            }
        }
        
        // Check for move down (X key)
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
                if let Some(nearest_stair) = find_nearest_discovered_stairwell(
                    &player,
                    TileType::StairDown,
                    &tile_visibility_query,
                    &map,
                ) {
                    let path = find_path((player.x, player.y), nearest_stair, &map);
                    if !path.is_empty() {
                        // Cancel any existing auto-movement
                        if autoexplore_opt.is_some() {
                            commands.entity(entity).remove::<Autoexplore>();
                        }
                        if auto_move_opt.is_some() {
                            commands.entity(entity).remove::<AutoMoveToStair>();
                        }
                        
                        println!("Auto-moving to discovered down stairwell at ({}, {})", nearest_stair.0, nearest_stair.1);
                        commands.entity(entity).insert(AutoMoveToStair::new(
                            nearest_stair,
                            path,
                            TileType::StairDown,
                        ));
                    } else {
                        println!("No path to down stairwell!");
                    }
                } else {
                    println!("No discovered down stairwell found. Explore to find stairs.");
                }
            }
        }
    }
}

// ============================================================================
// AUTOEXPLORE INPUT SYSTEMS
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
// Helper function to find nearest discovered stairwell of a specific type
fn find_nearest_discovered_stairwell(
    player: &Player,
    stair_type: TileType,
    tile_visibility_query: &Query<(&TilePos, &TileVisibilityState)>,
    map: &GameMap,
) -> Option<(u32, u32)> {
    let mut nearest_stair: Option<(u32, u32)> = None;
    let mut min_distance = f32::INFINITY;

    // Search all tiles for discovered stairs of the requested type
    for y in 0..map.height {
        for x in 0..map.width {
            // Check if this tile is the stair type we're looking for
            if map.get(x, y) != stair_type {
                continue;
            }

            // Check if this stairwell has been discovered (Visible or Seen)
            let mut is_discovered = false;
            for (tile_pos, visibility_state) in tile_visibility_query.iter() {
                if tile_pos.x == x && tile_pos.y == y {
                    is_discovered = visibility_state.visibility == TileVisibility::Visible
                        || visibility_state.visibility == TileVisibility::Seen;
                    break;
                }
            }

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

// System to handle auto-movement to discovered stairwells
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
        if let Some(next_pos) = auto_move.path.first().copied() {
            // Check if we can move to next position
            if map.get(next_pos.0, next_pos.1) != TileType::Wall {
                // Calculate animation positions
                let start_world_x = (player.x as f32 - (map.width as f32 / 2.0 - 0.5)) * TILE_SIZE;
                let start_world_y = (player.y as f32 - (map.height as f32 / 2.0 - 0.5)) * TILE_SIZE;
                let end_world_x = (next_pos.0 as f32 - (map.width as f32 / 2.0 - 0.5)) * TILE_SIZE;
                let end_world_y = (next_pos.1 as f32 - (map.height as f32 / 2.0 - 0.5)) * TILE_SIZE;

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
                    start_pos: Vec3::new(start_world_x, start_world_y, 1.0),
                    end_pos: Vec3::new(end_world_x, end_world_y, 1.0),
                    timer: Timer::from_seconds(AUTOEXPLORE_ANIM_TIMER, TimerMode::Once),
                });

                // Remove this step from path
                auto_move.path.remove(0);
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
                    if auto_move.stair_type == TileType::StairUp { "S" } else { "X" }
                );
            }
            commands.entity(entity).remove::<AutoMoveToStair>();
        }
    }
}
