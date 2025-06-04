use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::assets::GameAssets;
use crate::components::*;
use crate::map::GameMap;
use crate::biome::BiomeType;
use crate::level_manager::capture_tile_visibility;
use crate::fov::TileVisibilityState;

pub fn spawn_player(
    mut commands: Commands,
    assets: Res<GameAssets>,
    map: Res<GameMap>,
) {
    // Find a suitable spawn position (preferably near center)
    let center_x = map.width / 2;
    let center_y = map.height / 2;
    
    // Look for a floor tile near the center
    let mut spawn_pos = (center_x, center_y);
    
    // If center is not a floor, search nearby
    if map.tiles[center_y as usize][center_x as usize] != TileType::Floor {
        'search: for radius in 1..10 {
            for dx in -(radius as i32)..=(radius as i32) {
                for dy in -(radius as i32)..=(radius as i32) {
                    let x = center_x as i32 + dx;
                    let y = center_y as i32 + dy;
                    
                    if x >= 0 && x < map.width as i32 && y >= 0 && y < map.height as i32 {
                        let ux = x as u32;
                        let uy = y as u32;
                        if map.tiles[uy as usize][ux as usize] == TileType::Floor {
                            spawn_pos = (ux, uy);
                            break 'search;
                        }
                    }
                }
            }
        }
    }
    
    let grid_x = spawn_pos.0;
    let grid_y = spawn_pos.1;
    
    // Convert grid position to world position with new map centering
    let world_x = (grid_x as f32 - (map.width as f32 / 2.0 - 0.5)) * 32.0;
    let world_y = (grid_y as f32 - (map.height as f32 / 2.0 - 0.5)) * 32.0;
    
    // Player sprite at position 4,4 from rogues.png (32x32 sprites)
    // Extract 31x31 pixels from center to avoid sprite bleeding
    let sprite_x = 4.0 * 32.0;
    let sprite_y = 4.0 * 32.0;
    
    // Extract 31x31 from center (add 0.5 pixel offset on each side)
    let extract_x = sprite_x + 0.5;
    let extract_y = sprite_y + 0.5;
    let extract_size = 31.0;
    
    let player_entity = commands.spawn((
        Player { x: grid_x, y: grid_y },
        GridPosition { x: grid_x, y: grid_y },
        MovementInput {
            move_timer: Timer::from_seconds(0.15, TimerMode::Once), // 150ms for hold-to-move
            is_holding: false,
        },
        Sprite {
            image: assets.rogues.clone(),
            rect: Some(Rect::new(extract_x, extract_y, extract_x + extract_size, extract_y + extract_size)),
            flip_x: false, // Start facing left (natural sprite direction)
            // Custom size to stretch 31x31 extracted pixels to 32x32 render size
            custom_size: Some(Vec2::new(32.0, 32.0)),
            ..default()
        },
        Transform::from_xyz(world_x, world_y, 1.0),
    )).id();
    
    // Store player entity for camera targeting
    commands.insert_resource(PlayerEntity(player_entity));
}

pub fn handle_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut commands: Commands,
    mut player_query: Query<(Entity, &mut Player, &mut MovementInput, &mut Sprite), Without<MovementAnimation>>,
    map: Res<GameMap>,
) {
    if let Ok((entity, mut player, mut movement_input, mut sprite)) = player_query.single_mut() {
        let mut movement_attempted = false;
        let mut new_x = player.x;
        let mut new_y = player.y;
        let mut flip_sprite = false;
        
        // Check for any movement key being pressed
        let up_pressed = keyboard_input.pressed(KeyCode::ArrowUp) || keyboard_input.pressed(KeyCode::KeyW);
        let down_pressed = keyboard_input.pressed(KeyCode::ArrowDown) || keyboard_input.pressed(KeyCode::KeyS);
        let left_pressed = keyboard_input.pressed(KeyCode::ArrowLeft) || keyboard_input.pressed(KeyCode::KeyA);
        let right_pressed = keyboard_input.pressed(KeyCode::ArrowRight) || keyboard_input.pressed(KeyCode::KeyD);
        
        let any_movement_key = up_pressed || down_pressed || left_pressed || right_pressed;
        
        // Handle initial key press or continuous movement
        let should_move = if any_movement_key {
            if !movement_input.is_holding {
                // First press
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
        
        if should_move {
            // Handle movement input with priority for recent presses
            if up_pressed {
                if new_y < map.height - 1 { 
                    new_y += 1; 
                    movement_attempted = true;
                }
            } else if down_pressed {
                if new_y > 0 { 
                    new_y -= 1; 
                    movement_attempted = true;
                }
            }
            
            if left_pressed {
                if new_x > 0 { 
                    new_x -= 1; 
                    movement_attempted = true;
                    flip_sprite = false; // No flip for left (natural direction)
                }
            } else if right_pressed {
                if new_x < map.width - 1 { 
                    new_x += 1; 
                    movement_attempted = true;
                    flip_sprite = true; // Flip for right (face right)
                }
            }
            
            // Check collision with walls and apply movement
            if movement_attempted && map.tiles[new_y as usize][new_x as usize] != TileType::Wall {
                // Calculate start and end positions for animation
                let start_world_x = (player.x as f32 - (map.width as f32 / 2.0 - 0.5)) * 32.0;
                let start_world_y = (player.y as f32 - (map.height as f32 / 2.0 - 0.5)) * 32.0;
                let end_world_x = (new_x as f32 - (map.width as f32 / 2.0 - 0.5)) * 32.0;
                let end_world_y = (new_y as f32 - (map.height as f32 / 2.0 - 0.5)) * 32.0;
                
                // Update player grid position
                player.x = new_x;
                player.y = new_y;
                
                // Handle sprite flipping
                if left_pressed || right_pressed {
                    sprite.flip_x = flip_sprite;
                }
                
                // Add movement animation component
                commands.entity(entity).insert(MovementAnimation {
                    start_pos: Vec3::new(start_world_x, start_world_y, 1.0),
                    end_pos: Vec3::new(end_world_x, end_world_y, 1.0),
                    timer: Timer::from_seconds(0.1, TimerMode::Once), // 100ms hop animation
                });
                
                println!("Player moved to ({}, {})", new_x, new_y);
            } else if movement_attempted {
                println!("Cannot move to ({}, {}) - wall detected", new_x, new_y);
            }
        }
    }
}

pub fn move_player(
    mut player_query: Query<(&Player, &mut Transform), (Changed<Player>, Without<MovementAnimation>)>,
    map: Res<GameMap>,
) {
    // Only update transform for players without active movement animation
    // This system should only run when there's no animation active
    for (player, mut transform) in player_query.iter_mut() {
        // Convert grid position to world position
        let world_x = (player.x as f32 - (map.width as f32 / 2.0 - 0.5)) * 32.0;
        let world_y = (player.y as f32 - (map.height as f32 / 2.0 - 0.5)) * 32.0;
        
        transform.translation.x = world_x;
        transform.translation.y = world_y;
        transform.translation.z = 1.0; // Ensure consistent Z position
    }
}

pub fn animate_movement(
    mut commands: Commands,
    time: Res<Time>,
    mut animation_query: Query<(Entity, &mut MovementAnimation, &mut Transform)>,
) {
    for (entity, mut animation, mut transform) in animation_query.iter_mut() {
        animation.timer.tick(time.delta());
        
        // Linear interpolation between start and end positions
        let progress = animation.timer.elapsed_secs() / animation.timer.duration().as_secs_f32();
        let progress = progress.clamp(0.0, 1.0);
        
        // Add a slight hop effect (parabolic curve)
        let hop_height = 8.0 * (1.0 - (2.0 * progress - 1.0).powi(2));
        
        transform.translation = animation.start_pos.lerp(animation.end_pos, progress);
        transform.translation.z = 1.0 + hop_height; // Add hop to Z coordinate
        
        // Remove animation component when finished
        if animation.timer.finished() {
            transform.translation.z = 1.0; // Reset Z position
            commands.entity(entity).remove::<MovementAnimation>();
        }
    }
}

pub fn handle_continuous_movement(
    time: Res<Time>,
    mut player_query: Query<&mut MovementInput>,
) {
    // This system just ensures the movement timer is updated
    // The actual movement logic is handled in handle_input
    for mut movement_input in player_query.iter_mut() {
        if movement_input.is_holding {
            movement_input.move_timer.tick(time.delta());
        }
    }
}

pub fn handle_stair_interaction(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    player_query: Query<&Player>,
    tile_visibility_query: Query<(&TilePos, &TileVisibilityState)>,
    map: Res<GameMap>,
    current_level: Res<CurrentLevel>,
    mut level_maps: ResMut<LevelMaps>,
    mut level_change_events: EventWriter<LevelChangeEvent>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyE) {
        if let Ok(player) = player_query.single() {
            let tile_type = map.tiles[player.y as usize][player.x as usize];
            
            match tile_type {
                TileType::StairUp if current_level.level > 0 => {
                    println!("Going up to level {}", current_level.level - 1);
                    // Save current map with tile visibility
                    let current_visibility = capture_tile_visibility(&tile_visibility_query, map.width, map.height);
                    level_maps.maps.insert(current_level.level, map.to_saved_data(current_level.biome, current_visibility));
                    // Trigger level change
                    level_change_events.write(LevelChangeEvent {
                        new_level: current_level.level - 1,
                        spawn_position: SpawnPosition::StairDown,
                    });
                },
                TileType::StairDown if current_level.level < 50 => {
                    println!("Going down to level {}", current_level.level + 1);
                    // Save current map with tile visibility
                    let current_visibility = capture_tile_visibility(&tile_visibility_query, map.width, map.height);
                    level_maps.maps.insert(current_level.level, map.to_saved_data(current_level.biome, current_visibility));
                    // Trigger level change
                    level_change_events.write(LevelChangeEvent {
                        new_level: current_level.level + 1,
                        spawn_position: SpawnPosition::StairUp,
                    });
                },
                TileType::StairUp => {
                    println!("Cannot go up from the surface!");
                },
                TileType::StairDown => {
                    println!("Cannot go deeper - you've reached the bottom!");
                },
                _ => {
                    println!("No stairs here to use.");
                }
            }
        }
    }
}

pub fn debug_map_regeneration(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut regenerate_events: EventWriter<RegenerateMapEvent>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyR) && 
       (keyboard_input.pressed(KeyCode::ShiftLeft) || keyboard_input.pressed(KeyCode::ShiftRight)) {
        println!("Regenerating current level map...");
        regenerate_events.write(RegenerateMapEvent);
    }
}

pub fn debug_biome_cycling(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut current_level: ResMut<CurrentLevel>,
    mut regenerate_events: EventWriter<RegenerateMapEvent>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyB) && 
       (keyboard_input.pressed(KeyCode::ShiftLeft) || keyboard_input.pressed(KeyCode::ShiftRight)) {
        
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
