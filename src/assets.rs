use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use rand::Rng;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(AssetCollection, Resource)]
pub struct GameAssets {
    #[asset(path = "sprites/rogues.png")]
    pub rogues: Handle<Image>,
    #[asset(path = "sprites/tiles.png")]
    pub tiles: Handle<Image>,
    #[asset(path = "fonts/akkurat.otf")]
    pub akkurat_font: Handle<Font>,
}

// Sprite configuration structures
#[derive(Debug, Deserialize, Clone)]
pub struct SpriteConfig {
    pub sheet: String,
    pub position: [u32; 2],
    pub category: String,
}

#[derive(Debug, Deserialize)]
pub struct SpritesJson {
    pub sprite_size: [u32; 2],
    pub sprites: HashMap<String, SpriteConfig>,
}

#[derive(Resource)]
pub struct SpriteDatabase {
    // Maps sprite names to their texture indices
    pub sprite_indices: HashMap<String, u32>,
    // Maps categories to lists of available sprite names
    pub categories: HashMap<String, Vec<String>>,
}

impl SpriteDatabase {
    pub fn new() -> Self {
        let mut sprite_indices = HashMap::new();
        let mut categories = HashMap::new();
        
        // Define some default sprites based on sprites.json structure
        // Wall sprites
        sprite_indices.insert("dirt_wall_top".to_string(), sprite_position_to_index(0, 0));
        sprite_indices.insert("dirt_wall_side".to_string(), sprite_position_to_index(1, 0));
        sprite_indices.insert("rough_stone_wall_top".to_string(), sprite_position_to_index(0, 1));
        sprite_indices.insert("rough_stone_wall_side".to_string(), sprite_position_to_index(1, 1));
        
        // Floor sprites
        sprite_indices.insert("floor_stone1".to_string(), sprite_position_to_index(1, 6));
        sprite_indices.insert("floor_stone2".to_string(), sprite_position_to_index(2, 6));
        sprite_indices.insert("floor_stone3".to_string(), sprite_position_to_index(3, 6));
        sprite_indices.insert("dirt1".to_string(), sprite_position_to_index(1, 8));
        sprite_indices.insert("dirt2".to_string(), sprite_position_to_index(2, 8));
        sprite_indices.insert("dirt3".to_string(), sprite_position_to_index(3, 8));
        
        // Organize by categories
        categories.insert("wall_top".to_string(), vec![
            "dirt_wall_top".to_string(),
            "rough_stone_wall_top".to_string(),
        ]);
        categories.insert("wall_side".to_string(), vec![
            "dirt_wall_side".to_string(),
            "rough_stone_wall_side".to_string(),
        ]);
        categories.insert("floors".to_string(), vec![
            "floor_stone1".to_string(),
            "floor_stone2".to_string(),
            "floor_stone3".to_string(),
            "dirt1".to_string(),
            "dirt2".to_string(),
            "dirt3".to_string(),
        ]);
        
        Self { sprite_indices, categories }
    }
    
    pub fn get_sprite_index(&self, sprite_name: &str) -> Option<u32> {
        self.sprite_indices.get(sprite_name).copied()
    }
    
    pub fn get_random_sprite_from_category(&self, category: &str, rng: &mut impl Rng) -> Option<u32> {
        if let Some(sprites) = self.categories.get(category) {
            if !sprites.is_empty() {
                let sprite_name = &sprites[rng.random_range(0..sprites.len())];
                return self.get_sprite_index(sprite_name);
            }
        }
        None
    }
}

// Helper function to convert 2D sprite position to 1D texture index
// The sprite sheet is arranged in a grid, so index = y * sheet_width + x
pub fn sprite_position_to_index(x: u32, y: u32) -> u32 {
    // Based on tiles.png dimensions (544x832) with 32x32 tiles: 17 tiles wide, 26 tiles high
    y * 17 + x
}
