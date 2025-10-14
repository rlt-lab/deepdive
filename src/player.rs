use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::assets::GameAssets;
use crate::components::*;
use crate::map::GameMap;

// ============================================================================
// PLAYER SPAWNING
// ============================================================================

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

// ============================================================================
// PLAYER MOVEMENT & ANIMATION
// ============================================================================

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

// ============================================================================
// AUTOEXPLORE SYSTEMS
// ============================================================================

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

// ============================================================================
// AUTOEXPLORE HELPER FUNCTIONS (Public for input_handler)
// ============================================================================

/// Find nearest unexplored tile using breadth-first search
pub fn find_nearest_unexplored(
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

/// Simple A* pathfinding
pub fn find_path(start: (u32, u32), goal: (u32, u32), map: &GameMap) -> Vec<(u32, u32)> {
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

/// Count unexplored tiles on the map
pub fn count_unexplored_tiles(
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
