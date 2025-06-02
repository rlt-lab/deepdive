use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use rand::Rng;

use crate::assets::{GameAssets, SpriteDatabase, sprite_position_to_index};
use crate::components::{TileType, MapTile};

#[derive(Resource)]
pub struct GameMap {
    pub width: u32,
    pub height: u32,
    pub tiles: Vec<Vec<TileType>>,
}

impl GameMap {
    pub fn new(width: u32, height: u32) -> Self {
        let tiles = vec![vec![TileType::Floor; width as usize]; height as usize];
        Self { width, height, tiles }
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
        _ => sprite_position_to_index(0, 6), // Default to blank_floor_dark_grey
    }
}

pub fn spawn_map(
    mut commands: Commands,
    assets: Res<GameAssets>,
    sprite_db: Res<SpriteDatabase>,
) {
    let mut map = GameMap::new(10, 10);
    map.generate_simple_room();
    
    let tilemap_entity = commands.spawn_empty().id();
    let mut tile_storage = TileStorage::empty(TilemapSize { x: 10, y: 10 });
    
    // Spawn tiles with proper texture selection
    for y in 0..10 {
        for x in 0..10 {
            let tile_type = map.tiles[y as usize][x as usize];
            let texture_index = get_tile_texture_index(tile_type, &map, x, y, &sprite_db);
            
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
        size: TilemapSize { x: 10, y: 10 },
        storage: tile_storage,
        texture: TilemapTexture::Single(assets.tiles.clone()),
        tile_size,
        anchor: TilemapAnchor::Center,
        ..Default::default()
    });
    
    commands.insert_resource(map);
}
