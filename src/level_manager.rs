use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::assets::{GameAssets, SpriteDatabase, sprite_position_to_index};
use crate::components::*;
use crate::map::{GameMap, select_biome_asset};
use crate::player::{LevelChangeEvent, RegenerateMapEvent, SpawnPosition};
use crate::states::GameState;
use crate::fov::{FovSettings, TileVisibilityState, TileVisibility};
use crate::biome::BiomeType;

pub struct LevelManagerPlugin;

impl Plugin for LevelManagerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<LevelChangeEvent>()
            .add_event::<RegenerateMapEvent>()
            .init_resource::<CurrentLevel>()
            .init_resource::<LevelMaps>()
            .add_systems(Update, (
                handle_level_transitions,
                handle_map_regeneration,
            ).run_if(in_state(GameState::Playing)));
    }
}

impl Default for CurrentLevel {
    fn default() -> Self {
        Self { level: 0, biome: BiomeType::Caverns }
    }
}

// Helper function to capture current tile visibility states
pub fn capture_tile_visibility(
    tile_query: &Query<(&TilePos, &TileVisibilityState)>,
    map_width: u32,
    map_height: u32,
) -> Vec<Vec<TileVisibility>> {
    let mut visibility_data = vec![vec![TileVisibility::Unseen; map_width as usize]; map_height as usize];
    
    for (tile_pos, visibility_state) in tile_query.iter() {
        if tile_pos.x < map_width && tile_pos.y < map_height {
            visibility_data[tile_pos.y as usize][tile_pos.x as usize] = visibility_state.visibility;
        }
    }
    
    visibility_data
}

pub fn handle_level_transitions(
    mut commands: Commands,
    mut level_change_events: EventReader<LevelChangeEvent>,
    mut current_level: ResMut<CurrentLevel>,
    mut level_maps: ResMut<LevelMaps>, // changed to ResMut
    assets: Res<GameAssets>,
    _sprite_db: Res<SpriteDatabase>,
    mut player_query: Query<&mut Player>,
    tilemap_query: Query<Entity, With<TileStorage>>,
    tile_visibility_query: Query<Entity, With<TileVisibilityState>>,
    tile_pos_visibility_query: Query<(&TilePos, &TileVisibilityState)>,
    map: Option<Res<GameMap>>,
    mut fov_settings: ResMut<FovSettings>,
) {
    for event in level_change_events.read() {
        println!("Transitioning to level {}", event.new_level);
        
        // Save current tile visibility states before leaving the current level
        if let Some(current_map) = &map {
            let current_visibility = capture_tile_visibility(&tile_pos_visibility_query, current_map.width, current_map.height);
            if let Some(saved_data) = level_maps.maps.get_mut(&current_level.level) {
                saved_data.tile_visibility = current_visibility;
            }
        }
        
        // Clear existing tilemap and all tile entities
        for entity in tilemap_query.iter() {
            commands.entity(entity).despawn();
        }
        
        // Also clear any remaining tile visibility entities
        for entity in tile_visibility_query.iter() {
            commands.entity(entity).despawn();
        }
        
        // Update current level
        current_level.level = event.new_level;
        
        // Load or generate map for new level
        let (map, saved_visibility) = if let Some(saved_data) = level_maps.maps.get(&event.new_level) {
            // Sync biome from saved data
            current_level.biome = saved_data.biome;
            let saved_visibility = saved_data.tile_visibility.clone();
            (GameMap::from_saved_data(saved_data), saved_visibility)
        } else {
            let mut map = GameMap::new(80, 50);
            // Use biome-aware generation
            map.generate_with_biome(current_level.biome, event.new_level);
            map.place_stairs(event.new_level);
            // Create new visibility data for new map
            let new_visibility = vec![vec![TileVisibility::Unseen; map.width as usize]; map.height as usize];
            // Save new map data with biome
            level_maps.maps.insert(event.new_level, map.to_saved_data(current_level.biome, new_visibility.clone()));
            (map, new_visibility)
        };
        
        // Position player at appropriate spawn point
        if let Ok(mut player) = player_query.single_mut() {
            let spawn_pos = match event.spawn_position {
                SpawnPosition::StairUp => {
                    map.stair_up_pos.unwrap_or((map.width / 2, map.height / 2))
                },
                SpawnPosition::StairDown => {
                    map.stair_down_pos.unwrap_or((map.width / 2, map.height / 2))
                },
                SpawnPosition::Center => (map.width / 2, map.height / 2),
            };
            
            player.x = spawn_pos.0;
            player.y = spawn_pos.1;
            
            println!("Player spawned at ({}, {})", player.x, player.y);
        }
        
        // Spawn the new map inline
        let tilemap_entity = commands.spawn_empty().id();
        let mut tile_storage = TileStorage::empty(TilemapSize { x: map.width, y: map.height });
        
        // Spawn tiles with biome-aware texture selection
        let biome_config = current_level.biome.get_config();
        let mut rng = rand::rng();
        for y in 0..map.height {
            for x in 0..map.width {
                let tile_type = map.tiles[y as usize][x as usize];
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
                        TileVisibilityState { visibility: saved_visibility[y as usize][x as usize] },
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
        
        // Trigger FOV recalculation for new level
        fov_settings.needs_recalculation = true;
    }
}

pub fn handle_map_regeneration(
    mut commands: Commands,
    mut regenerate_events: EventReader<RegenerateMapEvent>,
    current_level: Res<CurrentLevel>,
    mut level_maps: ResMut<LevelMaps>,
    assets: Res<GameAssets>,
    sprite_db: Res<SpriteDatabase>,
    mut player_query: Query<&mut Player>,
    tilemap_query: Query<Entity, With<TileStorage>>,
    tile_visibility_query: Query<Entity, With<TileVisibilityState>>,
    mut fov_settings: ResMut<FovSettings>,
) {
    for _event in regenerate_events.read() {
        println!("Regenerating level {}", current_level.level);
        
        // Clear existing tilemap and all tile entities
        for entity in tilemap_query.iter() {
            commands.entity(entity).despawn();
        }
        
        // Also clear any remaining tile visibility entities
        for entity in tile_visibility_query.iter() {
            commands.entity(entity).despawn();
        }
        
        // Generate new map
        let mut map = GameMap::new(80, 50);
        
        // Use biome-aware generation
        map.generate_with_biome(current_level.biome, current_level.level);
        
        map.place_stairs(current_level.level);
        
        // Position player in center of new map
        if let Ok(mut player) = player_query.single_mut() {
            // Find a suitable floor position near center
            let center_x = map.width / 2;
            let center_y = map.height / 2;
            
            // Look for nearby floor tile
            let mut spawn_pos = (center_x, center_y);
            for radius in 0..10 {
                for dx in -(radius as i32)..=(radius as i32) {
                    for dy in -(radius as i32)..=(radius as i32) {
                        let x = center_x as i32 + dx;
                        let y = center_y as i32 + dy;
                        
                        if x >= 0 && x < map.width as i32 && y >= 0 && y < map.height as i32 {
                            let ux = x as u32;
                            let uy = y as u32;
                            if map.tiles[uy as usize][ux as usize] == TileType::Floor {
                                spawn_pos = (ux, uy);
                                break;
                            }
                        }
                    }
                }
                if map.tiles[spawn_pos.1 as usize][spawn_pos.0 as usize] == TileType::Floor {
                    break;
                }
            }
            
            player.x = spawn_pos.0;
            player.y = spawn_pos.1;
            
            println!("Player repositioned at ({}, {})", player.x, player.y);
        }
        
        // Save the new map
        let new_visibility = vec![vec![TileVisibility::Unseen; map.width as usize]; map.height as usize];
        level_maps.maps.insert(current_level.level, map.to_saved_data(current_level.biome, new_visibility.clone()));
        
        // Spawn the new map inline
        let tilemap_entity = commands.spawn_empty().id();
        let mut tile_storage = TileStorage::empty(TilemapSize { x: map.width, y: map.height });
        
        // Spawn tiles with biome-aware texture selection
        let biome_config = current_level.biome.get_config();
        let mut rng = rand::rng();
        for y in 0..map.height {
            for x in 0..map.width {
                let tile_type = map.tiles[y as usize][x as usize];
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
                        TileVisibilityState { visibility: new_visibility[y as usize][x as usize] },
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
        
        // Trigger FOV recalculation for regenerated map
        fov_settings.needs_recalculation = true;
    }
}
