use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use rand::Rng;
use std::collections::HashSet;

use crate::assets::{GameAssets, SpriteDatabase, sprite_position_to_index};
use crate::components::{TileType, MapTile, SavedMapData, CurrentLevel, LevelMaps};
use crate::fov::{TileVisibilityState, TileVisibility};
use crate::biome::{BiomeType, BiomeConfig};

#[derive(Resource)]
pub struct GameMap {
    pub width: u32,
    pub height: u32,
    pub tiles: Vec<Vec<TileType>>,
    pub stair_up_pos: Option<(u32, u32)>,
    pub stair_down_pos: Option<(u32, u32)>,
}

impl GameMap {
    pub fn new(width: u32, height: u32) -> Self {
        let tiles = vec![vec![TileType::Wall; width as usize]; height as usize];
        Self { 
            width, 
            height, 
            tiles,
            stair_up_pos: None,
            stair_down_pos: None,
        }
    }
    
    // Helper function to check if a position is within the oblong circle boundary
    fn is_within_ellipse(&self, x: u32, y: u32) -> bool {
        let center_x = self.width as f32 / 2.0;
        let center_y = self.height as f32 / 2.0;
        
        // Create an ellipse that fits within the map bounds with some padding
        let a = (self.width as f32 / 2.0) - 2.0; // Semi-major axis (horizontal)
        let b = (self.height as f32 / 2.0) - 2.0; // Semi-minor axis (vertical)
        
        let dx = x as f32 - center_x;
        let dy = y as f32 - center_y;
        
        // Ellipse equation: (x-h)²/a² + (y-k)²/b² <= 1
        (dx * dx) / (a * a) + (dy * dy) / (b * b) <= 1.0
    }
    
    pub fn generate_simple_room(&mut self) {
        // Fill with walls
        for y in 0..self.height {
            for x in 0..self.width {
                self.tiles[y as usize][x as usize] = TileType::Wall;
            }
        }
        
        // Create room interior (leave 1-tile border)
        for y in 1..self.height-1 {
            for x in 1..self.width-1 {
                self.tiles[y as usize][x as usize] = TileType::Floor;
            }
        }
    }
    
    pub fn generate_drunkard_walk(&mut self, steps: u32, num_walkers: u32) {
        let mut rng = rand::rng();
        
        // Start with all walls
        for y in 0..self.height {
            for x in 0..self.width {
                self.tiles[y as usize][x as usize] = TileType::Wall;
            }
        }
        
        let mut carved_positions = HashSet::new();
        
        // Multiple walkers for more interesting caves
        for _ in 0..num_walkers {
            let mut x = self.width / 2;
            let mut y = self.height / 2;
            
            // Ensure starting position is within ellipse (it should be since center is always inside)
            if !self.is_within_ellipse(x, y) {
                // Fallback to exact center if needed
                x = self.width / 2;
                y = self.height / 2;
            }
            
            // Carve the starting position
            self.tiles[y as usize][x as usize] = TileType::Floor;
            carved_positions.insert((x, y));
            
            // Perform drunkard walk
            for _ in 0..steps {
                // Choose random direction (0=up, 1=right, 2=down, 3=left)
                let direction = rng.random_range(0..4);
                
                let (new_x, new_y) = match direction {
                    0 if y > 0 => (x, y - 1),
                    1 if x < self.width - 1 => (x + 1, y),
                    2 if y < self.height - 1 => (x, y + 1),
                    3 if x > 0 => (x - 1, y),
                    _ => (x, y), // Stay in place if at boundary
                };
                
                // Only move if the new position is within the elliptical boundary
                if self.is_within_ellipse(new_x, new_y) {
                    x = new_x;
                    y = new_y;
                    
                    // Carve the floor
                    self.tiles[y as usize][x as usize] = TileType::Floor;
                    carved_positions.insert((x, y));
                }
                // If outside ellipse, stay at current position but don't carve
            }
        }
        
        // Ensure all carved areas are connected
        self.connect_disconnected_areas(&carved_positions);
        
        // Final cleanup: ensure all tiles outside the ellipse are walls
        for y in 0..self.height {
            for x in 0..self.width {
                if !self.is_within_ellipse(x, y) {
                    self.tiles[y as usize][x as usize] = TileType::Wall;
                }
            }
        }
    }
    
    fn connect_disconnected_areas(&mut self, carved_positions: &HashSet<(u32, u32)>) {
        let _rng = rand::rng();
        let groups = self.find_disconnected_groups(carved_positions);
        
        // Connect all groups to the largest one
        if groups.len() > 1 {
            let largest_group_idx = groups.iter()
                .enumerate()
                .max_by_key(|(_, group)| group.len())
                .map(|(idx, _)| idx)
                .unwrap();
            
            let largest_group = &groups[largest_group_idx];
            
            for (i, group) in groups.iter().enumerate() {
                if i != largest_group_idx {
                    // Find closest points between groups
                    let (start, end) = self.find_closest_points(group, largest_group);
                    self.carve_tunnel(start, end);
                }
            }
        }
    }
    
    fn find_disconnected_groups(&self, carved_positions: &HashSet<(u32, u32)>) -> Vec<Vec<(u32, u32)>> {
        let mut visited = HashSet::new();
        let mut groups = Vec::new();
        
        for &pos in carved_positions {
            if !visited.contains(&pos) {
                let mut group = Vec::new();
                let mut stack = vec![pos];
                
                while let Some(current) = stack.pop() {
                    if visited.contains(&current) {
                        continue;
                    }
                    
                    visited.insert(current);
                    group.push(current);
                    
                    // Check adjacent positions
                    let (x, y) = current;
                    for &(dx, dy) in &[(0, 1), (1, 0), (0, -1), (-1, 0)] {
                        let new_x = x as i32 + dx;
                        let new_y = y as i32 + dy;
                        
                        if new_x >= 0 && new_x < self.width as i32 && 
                           new_y >= 0 && new_y < self.height as i32 {
                            let new_pos = (new_x as u32, new_y as u32);
                            if carved_positions.contains(&new_pos) && !visited.contains(&new_pos) {
                                stack.push(new_pos);
                            }
                        }
                    }
                }
                
                groups.push(group);
            }
        }
        
        groups
    }
    
    fn find_closest_points(&self, group1: &[(u32, u32)], group2: &[(u32, u32)]) -> ((u32, u32), (u32, u32)) {
        let mut min_distance = f32::INFINITY;
        let mut closest_pair = ((0, 0), (0, 0));
        
        for &pos1 in group1 {
            for &pos2 in group2 {
                let distance = ((pos1.0 as f32 - pos2.0 as f32).powi(2) + 
                               (pos1.1 as f32 - pos2.1 as f32).powi(2)).sqrt();
                
                if distance < min_distance {
                    min_distance = distance;
                    closest_pair = (pos1, pos2);
                }
            }
        }
        
        closest_pair
    }
    
    fn carve_tunnel(&mut self, start: (u32, u32), end: (u32, u32)) {
        let mut x = start.0 as i32;
        let mut y = start.1 as i32;
        let target_x = end.0 as i32;
        let target_y = end.1 as i32;
        
        // Simple L-shaped tunnel
        while x != target_x {
            if x < target_x {
                x += 1;
            } else {
                x -= 1;
            }
            if x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32 {
                // Only carve if within ellipse boundary
                if self.is_within_ellipse(x as u32, y as u32) {
                    self.tiles[y as usize][x as usize] = TileType::Floor;
                }
            }
        }
        
        while y != target_y {
            if y < target_y {
                y += 1;
            } else {
                y -= 1;
            }
            if x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32 {
                // Only carve if within ellipse boundary
                if self.is_within_ellipse(x as u32, y as u32) {
                    self.tiles[y as usize][x as usize] = TileType::Floor;
                }
            }
        }
    }
    
    pub fn place_stairs(&mut self, level: u32) {
        let mut rng = rand::rng();
        let floor_positions: Vec<(u32, u32)> = self.get_floor_positions();
        
        if floor_positions.is_empty() {
            return;
        }
        
        // Place stairs up (except on level 0)
        if level > 0 {
            let pos_idx = rng.random_range(0..floor_positions.len());
            let (x, y) = floor_positions[pos_idx];
            self.tiles[y as usize][x as usize] = TileType::StairUp;
            self.stair_up_pos = Some((x, y));
        }
        
        // Place stairs down (except on level 50)
        if level < 50 {
            let mut attempts = 0;
            loop {
                let pos_idx = rng.random_range(0..floor_positions.len());
                let (x, y) = floor_positions[pos_idx];
                
                // Make sure stairs aren't too close to each other
                if self.stair_up_pos.map_or(true, |(ux, uy)| {
                    ((x as i32 - ux as i32).abs() + (y as i32 - uy as i32).abs()) > 5
                }) {
                    self.tiles[y as usize][x as usize] = TileType::StairDown;
                    self.stair_down_pos = Some((x, y));
                    break;
                }
                
                attempts += 1;
                if attempts > 100 {
                    // Fallback: place anywhere
                    self.tiles[y as usize][x as usize] = TileType::StairDown;
                    self.stair_down_pos = Some((x, y));
                    break;
                }
            }
        }
    }
    
    fn get_floor_positions(&self) -> Vec<(u32, u32)> {
        let mut positions = Vec::new();
        for y in 0..self.height {
            for x in 0..self.width {
                if self.tiles[y as usize][x as usize] == TileType::Floor {
                    positions.push((x, y));
                }
            }
        }
        positions
    }
    
    pub fn from_saved_data(saved: &SavedMapData) -> Self {
        let mut map = GameMap::new(saved.width, saved.height);
        map.tiles = saved.tiles.clone();
        map.stair_up_pos = saved.stair_up_pos;
        map.stair_down_pos = saved.stair_down_pos;
        map
    }

    pub fn to_saved_data(&self, biome: BiomeType, tile_visibility: Vec<Vec<TileVisibility>>) -> SavedMapData {
        SavedMapData {
            width: self.width,
            height: self.height,
            tiles: self.tiles.clone(),
            stair_up_pos: self.stair_up_pos,
            stair_down_pos: self.stair_down_pos,
            biome,
            tile_visibility,
        }
    }
    
    // Helper function to check if there's a wall tile below the current position
    pub fn has_wall_below(&self, x: u32, y: u32) -> bool {
        if y == 0 {
            return false; // Bottom edge, no tile below
        }
        self.tiles[(y - 1) as usize][x as usize] == TileType::Wall
    }
}

// Helper function to get the correct tile texture index based on tile type and context
pub fn get_tile_texture_index(tile_type: TileType, map: &GameMap, x: u32, y: u32, sprite_db: &SpriteDatabase) -> u32 {
    let mut rng = rand::rng();
    
    match tile_type {
        TileType::Floor => {
            // Get a random floor sprite from the floors category
            sprite_db.get_random_sprite_from_category("floors", &mut rng)
                .unwrap_or(sprite_position_to_index(1, 6)) // fallback to floor_stone1
        },
        TileType::Wall => {
            if map.has_wall_below(x, y) {
                // Use wall_top sprites
                sprite_db.get_random_sprite_from_category("wall_top", &mut rng)
                    .unwrap_or(sprite_position_to_index(0, 0)) // fallback to dirt_wall_top
            } else {
                // Use wall_side sprites  
                sprite_db.get_random_sprite_from_category("wall_side", &mut rng)
                    .unwrap_or(sprite_position_to_index(1, 0)) // fallback to dirt_wall_side
            }
        },
        TileType::StairUp => sprite_position_to_index(8, 16), // stair_up at 8,16
        TileType::StairDown => sprite_position_to_index(7, 16), // stair_down at 7,16
        _ => sprite_position_to_index(0, 6), // Default to blank_floor_dark_grey
    }
}

// Add biome asset selection function with context-aware wall selection
pub fn select_biome_asset(biome_config: &BiomeConfig, tile_type: TileType, map: &GameMap, x: u32, y: u32, rng: &mut impl Rng) -> (u32, u32) {
    match tile_type {
        TileType::Floor => {
            let assets = &biome_config.allowed_floor_assets;
            if assets.is_empty() {
                return (1, 6); // fallback to floor_stone1
            }
            assets[rng.random_range(0..assets.len())]
        },
        TileType::Wall => {
            let wall_assets = &biome_config.allowed_wall_assets;
            if wall_assets.is_empty() {
                // Fallback logic with proper wall type selection
                if map.has_wall_below(x, y) {
                    return (0, 0); // dirt_wall_top
                } else {
                    return (1, 0); // dirt_wall_side
                }
            }
            
            // For biomes like Caverns with organized pairs: (top, side), (top, side)
            // Check if we have pairs by looking at asset coordinates
            let has_organized_pairs = wall_assets.len() >= 2 && 
                wall_assets.len() % 2 == 0 &&
                wall_assets.iter().step_by(2).all(|(x, y)| {
                    // Check if this looks like a wall_top position (typically at x=0)
                    *x == 0 || (*x, *y) == (0, 0) || (*x, *y) == (0, 1) || (*x, *y) == (0, 2) ||
                    (*x, *y) == (0, 3) || (*x, *y) == (0, 4) || (*x, *y) == (0, 5)
                });
            
            if has_organized_pairs {
                // Use proper wall type selection for organized assets
                if map.has_wall_below(x, y) {
                    // Use wall_top sprites (even indices)
                    let wall_top_indices: Vec<_> = (0..wall_assets.len()).step_by(2).collect();
                    let selected_pair_idx = wall_top_indices[rng.random_range(0..wall_top_indices.len())];
                    wall_assets[selected_pair_idx]
                } else {
                    // Use wall_side sprites (odd indices)
                    let wall_side_indices: Vec<_> = (1..wall_assets.len()).step_by(2).collect();
                    let selected_pair_idx = wall_side_indices[rng.random_range(0..wall_side_indices.len())];
                    wall_assets[selected_pair_idx]
                }
            } else {
                // For biomes without organized pairs, just use any wall asset
                wall_assets[rng.random_range(0..wall_assets.len())]
            }
        },
        TileType::Water => {
            let assets = &biome_config.allowed_water_assets;
            if assets.is_empty() {
                return (0, 12); // fallback to blank_blue_floor for water
            }
            assets[rng.random_range(0..assets.len())]
        },
        TileType::StairUp => {
            // Always use the specific staircase_up sprite
            (8, 16)
        },
        TileType::StairDown => {
            // Always use the specific staircase_down sprite  
            (7, 16)
        },
    }
}

pub fn spawn_map(
    mut commands: Commands,
    assets: Res<GameAssets>,
    _sprite_db: Res<SpriteDatabase>,
    level_maps: Res<LevelMaps>,
    current_level: Res<CurrentLevel>,
) {
    // Biome-aware config and RNG
    let biome_config = current_level.biome.get_config();
    let mut rng = rand::rng();

    let map = if let Some(saved_data) = level_maps.maps.get(&current_level.level) {
        // Load existing map
        GameMap::from_saved_data(saved_data)
    } else {
        // Generate new map
        let mut map = GameMap::new(80, 50); // Larger maps for drunkard walk
        
        if current_level.level == 0 {
            // Use a basic drunkard walk for level 0 for testing - scaled for 80x50 map
            map.generate_drunkard_walk(2000, 3);
        } else {
            // Use drunkard walk for deeper levels - scaled for 80x50 map
            let steps = 2500 + (current_level.level * 50); // More complex deeper levels
            let walkers = 4 + (current_level.level / 3); // More walkers for deeper levels
            map.generate_drunkard_walk(steps, walkers);
        }
        
        map.place_stairs(current_level.level);
        map
    };
    
    let tilemap_entity = commands.spawn_empty().id();
    let mut tile_storage = TileStorage::empty(TilemapSize { x: map.width, y: map.height });

    // Spawn tiles with biome-aware asset selection
    for y in 0..map.height {
        for x in 0..map.width {
            let tile_type = map.tiles[y as usize][x as usize];
            // Select sprite position based on biome configuration
            let (sprite_x, sprite_y) = select_biome_asset(&biome_config, tile_type, &map, x, y, &mut rng);
            let texture_index = sprite_position_to_index(sprite_x, sprite_y);

            let tile_pos = TilePos { x, y };
            let tile_entity = commands
                .spawn((
                    TileBundle {
                        position: tile_pos,
                        tilemap_id: TilemapId(tilemap_entity),
                        texture_index: TileTextureIndex(texture_index),
                        ..Default::default()
                    },
                    MapTile { tile_type },
                    TileVisibilityState { visibility: TileVisibility::Unseen },
                ))
                .id();
            tile_storage.set(&tile_pos, tile_entity);
        }
    }
    
    let tile_size = TilemapTileSize { x: 32.0, y: 32.0 };
    let grid_size = tile_size.into();
    let map_type = TilemapType::default();
    
    commands.entity(tilemap_entity).insert(TilemapBundle {
        grid_size,
        map_type,
        size: TilemapSize { x: map.width, y: map.height },
        storage: tile_storage,
        texture: TilemapTexture::Single(assets.tiles.clone()),
        tile_size,
        anchor: TilemapAnchor::Center,
        ..Default::default()
    });
    
    commands.insert_resource(map);
}
