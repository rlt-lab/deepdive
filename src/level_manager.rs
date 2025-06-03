use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::assets::{GameAssets, SpriteDatabase};
use crate::components::*;
use crate::map::{GameMap, get_tile_texture_index};
use crate::player::{LevelChangeEvent, RegenerateMapEvent, SpawnPosition};
use crate::states::GameState;
use crate::fov::{FovSettings, TileVisibilityState, TileVisibility};

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
        Self { level: 0 }
    }
}

pub fn handle_level_transitions(
    mut commands: Commands,
    mut level_change_events: EventReader<LevelChangeEvent>,
    mut current_level: ResMut<CurrentLevel>,
    level_maps: Res<LevelMaps>,
    assets: Res<GameAssets>,
    sprite_db: Res<SpriteDatabase>,
    mut player_query: Query<&mut Player>,
    tilemap_query: Query<Entity, With<TileStorage>>,
    tile_visibility_query: Query<Entity, With<TileVisibilityState>>,
    mut fov_settings: ResMut<FovSettings>,
) {
    for event in level_change_events.read() {
        println!("Transitioning to level {}", event.new_level);
        
        // Update current level
        current_level.level = event.new_level;
        
        // Clear existing tilemap and all tile entities recursively
        for entity in tilemap_query.iter() {
            commands.entity(entity).despawn();
        }
        
        // Also clear any remaining tile visibility entities
        for entity in tile_visibility_query.iter() {
            commands.entity(entity).despawn();
        }
        
        // Load or generate map for new level
        let map = if let Some(saved_data) = level_maps.maps.get(&event.new_level) {
            GameMap::from_saved_data(saved_data)
        } else {
            let mut map = GameMap::new(30, 20);
            
            if event.new_level == 0 {
                // Use a basic drunkard walk for level 0 for testing
                map.generate_drunkard_walk(300, 2);
            } else {
                let steps = 400 + (event.new_level * 10);
                let walkers = 3 + (event.new_level / 5);
                map.generate_drunkard_walk(steps, walkers);
            }
            
            map.place_stairs(event.new_level);
            map
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
        
        // Spawn tiles with proper texture selection
        for y in 0..map.height {
            for x in 0..map.width {
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
        let mut map = GameMap::new(30, 20);
        
        if current_level.level == 0 {
            // Use a basic drunkard walk for level 0 for testing
            map.generate_drunkard_walk(300, 2);
        } else {
            let steps = 400 + (current_level.level * 10);
            let walkers = 3 + (current_level.level / 5);
            map.generate_drunkard_walk(steps, walkers);
        }
        
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
        level_maps.maps.insert(current_level.level, map.to_saved_data());
        
        // Spawn the new map inline
        let tilemap_entity = commands.spawn_empty().id();
        let mut tile_storage = TileStorage::empty(TilemapSize { x: map.width, y: map.height });
        
        // Spawn tiles with proper texture selection
        for y in 0..map.height {
            for x in 0..map.width {
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
        
        // Trigger FOV recalculation for regenerated map
        fov_settings.needs_recalculation = true;
    }
}
