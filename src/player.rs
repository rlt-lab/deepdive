use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::assets::GameAssets;
use crate::components::*;
use crate::map::GameMap;
use crate::biome::BiomeType;
use crate::level_manager::capture_tile_visibility;

pub fn spawn_player(
    mut commands: Commands,
    assets: Res<GameAssets>,
    map: Res<GameMap>,
    sprite_config: Res<PlayerSpriteConfig>,
) {
    // Find a suitable spawn position (preferably near center)
    let center_x = map.width / 2;
    let center_y = map.height / 2;
    
    // Look for a floor tile near the center
    let mut spawn_pos = (center_x, center_y);
    
    // If center is not a floor, search nearby
    if map.get(center_x, center_y) != TileType::Floor {
        'search: for radius in 1..10 {
            for dx in -(radius as i32)..=(radius as i32) {
                for dy in -(radius as i32)..=(radius as i32) {
                    let x = center_x as i32 + dx;
                    let y = center_y as i32 + dy;

                    if x >= 0 && x < map.width as i32 && y >= 0 && y < map.height as i32 {
                        let ux = x as u32;
                        let uy = y as u32;
                        if map.get(ux, uy) == TileType::Floor {
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

    let player_entity = commands.spawn((
        Player { x: grid_x, y: grid_y },
        MovementInput {
            move_timer: Timer::from_seconds(0.15, TimerMode::Once), // 150ms for hold-to-move
            is_holding: false,
        },
        Sprite {
            image: assets.rogues.clone(),
            rect: Some(sprite_config.sprite_rect),
            flip_x: false, // Start facing left (natural sprite direction)
            custom_size: Some(sprite_config.custom_size),
            ..default()
        },
        Transform::from_xyz(world_x, world_y, 1.0),
    )).id();
    
    // Store player entity for camera targeting
    commands.insert_resource(PlayerEntity(player_entity));
}

// Event-based input detection - only fires when key state changes
pub fn detect_movement_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut player_query: Query<&mut MovementInput>,
    mut move_events: EventWriter<PlayerMoveIntent>,
) {
    if let Ok(mut movement_input) = player_query.single_mut() {
        // Check for any movement key being pressed
        let up_pressed = keyboard_input.pressed(KeyCode::ArrowUp) || keyboard_input.pressed(KeyCode::KeyW);
        let down_pressed = keyboard_input.pressed(KeyCode::ArrowDown) || keyboard_input.pressed(KeyCode::KeyS);
        let left_pressed = keyboard_input.pressed(KeyCode::ArrowLeft) || keyboard_input.pressed(KeyCode::KeyA);
        let right_pressed = keyboard_input.pressed(KeyCode::ArrowRight) || keyboard_input.pressed(KeyCode::KeyD);

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

// Process movement intent events
pub fn handle_input(
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
            if movement_attempted && map.get(new_x, new_y) != TileType::Wall {
                // Calculate start and end positions for animation
                let start_world_x = (player.x as f32 - (map.width as f32 / 2.0 - 0.5)) * 32.0;
                let start_world_y = (player.y as f32 - (map.height as f32 / 2.0 - 0.5)) * 32.0;
                let end_world_x = (new_x as f32 - (map.width as f32 / 2.0 - 0.5)) * 32.0;
                let end_world_y = (new_y as f32 - (map.height as f32 / 2.0 - 0.5)) * 32.0;

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
            let tile_type = map.get(player.x, player.y);

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
// AUTOEXPLORE SYSTEMS
// ============================================================================

pub fn toggle_autoexplore(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut player_query: Query<(Entity, &Player, Option<&Autoexplore>)>,
    tile_visibility_query: Query<(&TilePos, &TileVisibilityState)>,
    map: Res<GameMap>,
) {
    // Check for Shift+A to toggle, or ESC to cancel
    let toggle_pressed = keyboard_input.just_pressed(KeyCode::KeyA) &&
                        (keyboard_input.pressed(KeyCode::ShiftLeft) || keyboard_input.pressed(KeyCode::ShiftRight));
    let cancel_pressed = keyboard_input.just_pressed(KeyCode::Escape);

    if toggle_pressed || cancel_pressed {
        if let Ok((entity, _player, autoexplore_opt)) = player_query.single_mut() {
            if autoexplore_opt.is_some() {
                // Remove component entirely to stop autoexplore
                commands.entity(entity).remove::<Autoexplore>();
                println!("Autoexplore disabled");
            } else if toggle_pressed {
                // Only enable on Shift+A, not on ESC
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

pub fn run_autoexplore(
    mut commands: Commands,
    time: Res<Time>,
    mut player_query: Query<(Entity, &mut Player, &mut Autoexplore, &mut Sprite), Without<MovementAnimation>>,
    tile_visibility_query: Query<(&TilePos, &TileVisibilityState)>,
    map: Res<GameMap>,
) {
    if let Ok((entity, mut player, mut autoexplore, mut sprite)) = player_query.single_mut() {
        if !autoexplore.active {
            // Try to activate if component exists
            let unexplored = find_nearest_unexplored(&player, &tile_visibility_query, &map);
            if let Some(target) = unexplored {
                autoexplore.target = Some(target);
                autoexplore.path = find_path((player.x, player.y), target, &map);
                autoexplore.active = true;
            } else {
                // No more unexplored tiles - remove component
                commands.entity(entity).remove::<Autoexplore>();
                println!("Autoexplore complete - map fully explored!");
                return;
            }
        }

        // Tick timer
        autoexplore.move_timer.tick(time.delta());

        if !autoexplore.move_timer.just_finished() {
            return;
        }

        // Get next step in path
        if let Some(next_pos) = autoexplore.path.first().copied() {
            // Check if we can move to next position
            if map.get(next_pos.0, next_pos.1) != TileType::Wall {
                // Calculate animation positions
                let start_world_x = (player.x as f32 - (map.width as f32 / 2.0 - 0.5)) * 32.0;
                let start_world_y = (player.y as f32 - (map.height as f32 / 2.0 - 0.5)) * 32.0;
                let end_world_x = (next_pos.0 as f32 - (map.width as f32 / 2.0 - 0.5)) * 32.0;
                let end_world_y = (next_pos.1 as f32 - (map.height as f32 / 2.0 - 0.5)) * 32.0;

                // Update sprite facing
                if next_pos.0 < player.x {
                    sprite.flip_x = false; // Moving left
                } else if next_pos.0 > player.x {
                    sprite.flip_x = true; // Moving right
                }

                // Move player
                player.x = next_pos.0;
                player.y = next_pos.1;

                // Add fast animation for autoexplore
                commands.entity(entity).insert(MovementAnimation {
                    start_pos: Vec3::new(start_world_x, start_world_y, 1.0),
                    end_pos: Vec3::new(end_world_x, end_world_y, 1.0),
                    timer: Timer::from_seconds(0.05, TimerMode::Once), // 50ms animation - fast but visible
                });

                // Remove this step from path
                autoexplore.path.remove(0);
            } else {
                // Path blocked, recalculate
                autoexplore.path.clear();
                autoexplore.active = false;
            }
        } else {
            // Reached target or path empty, find new target
            autoexplore.active = false;
        }
    }
}

// Find nearest unexplored tile using breadth-first search
fn find_nearest_unexplored(
    player: &Player,
    tile_visibility_query: &Query<(&TilePos, &TileVisibilityState)>,
    map: &GameMap,
) -> Option<(u32, u32)> {
    use std::collections::VecDeque;

    let start = (player.x, player.y);
    let mut visited = vec![vec![false; map.height as usize]; map.width as usize];
    let mut queue = VecDeque::new();
    queue.push_back(start);
    visited[start.0 as usize][start.1 as usize] = true;

    while let Some((x, y)) = queue.pop_front() {
        // Check if this tile is unexplored (Unseen)
        let mut is_unseen = true;
        for (tile_pos, visibility_state) in tile_visibility_query.iter() {
            if tile_pos.x == x && tile_pos.y == y {
                is_unseen = visibility_state.visibility == TileVisibility::Unseen;
                break;
            }
        }

        if is_unseen && map.get(x, y) == TileType::Floor {
            return Some((x, y));
        }

        // Explore neighbors
        let neighbors = [
            (x.wrapping_sub(1), y), (x + 1, y),
            (x, y.wrapping_sub(1)), (x, y + 1),
        ];

        for (nx, ny) in neighbors {
            if nx < map.width && ny < map.height && !visited[nx as usize][ny as usize] {
                if map.get(nx, ny) != TileType::Wall {
                    visited[nx as usize][ny as usize] = true;
                    queue.push_back((nx, ny));
                }
            }
        }
    }

    None
}

// Simple A* pathfinding
fn find_path(start: (u32, u32), goal: (u32, u32), map: &GameMap) -> Vec<(u32, u32)> {
    use std::collections::{BinaryHeap, HashMap};
    use std::cmp::Ordering;

    #[derive(Copy, Clone, Eq, PartialEq)]
    struct State {
        cost: u32,
        position: (u32, u32),
    }

    impl Ord for State {
        fn cmp(&self, other: &Self) -> Ordering {
            other.cost.cmp(&self.cost)
        }
    }

    impl PartialOrd for State {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }

    let heuristic = |a: (u32, u32), b: (u32, u32)| {
        ((a.0 as i32 - b.0 as i32).abs() + (a.1 as i32 - b.1 as i32).abs()) as u32
    };

    let mut open_set = BinaryHeap::new();
    let mut came_from: HashMap<(u32, u32), (u32, u32)> = HashMap::new();
    let mut g_score: HashMap<(u32, u32), u32> = HashMap::new();

    g_score.insert(start, 0);
    open_set.push(State {
        cost: heuristic(start, goal),
        position: start,
    });

    while let Some(State { position, .. }) = open_set.pop() {
        if position == goal {
            // Reconstruct path
            let mut path = Vec::new();
            let mut current = goal;
            while current != start {
                path.push(current);
                current = *came_from.get(&current).unwrap();
            }
            path.reverse();
            return path;
        }

        let neighbors = [
            (position.0.wrapping_sub(1), position.1),
            (position.0 + 1, position.1),
            (position.0, position.1.wrapping_sub(1)),
            (position.0, position.1 + 1),
        ];

        for neighbor in neighbors {
            if neighbor.0 >= map.width || neighbor.1 >= map.height {
                continue;
            }
            if map.get(neighbor.0, neighbor.1) == TileType::Wall {
                continue;
            }

            let tentative_g_score = g_score.get(&position).unwrap_or(&u32::MAX) + 1;
            if tentative_g_score < *g_score.get(&neighbor).unwrap_or(&u32::MAX) {
                came_from.insert(neighbor, position);
                g_score.insert(neighbor, tentative_g_score);
                open_set.push(State {
                    cost: tentative_g_score + heuristic(neighbor, goal),
                    position: neighbor,
                });
            }
        }
    }

    Vec::new() // No path found
}

fn count_unexplored_tiles(
    tile_visibility_query: &Query<(&TilePos, &TileVisibilityState)>,
    map: &GameMap,
) -> usize {
    let mut count = 0;
    for x in 0..map.width {
        for y in 0..map.height {
            if map.get(x, y) == TileType::Floor {
                let mut is_unseen = true;
                for (tile_pos, visibility_state) in tile_visibility_query.iter() {
                    if tile_pos.x == x && tile_pos.y == y {
                        is_unseen = visibility_state.visibility == TileVisibility::Unseen;
                        break;
                    }
                }
                if is_unseen {
                    count += 1;
                }
            }
        }
    }
    count
}
