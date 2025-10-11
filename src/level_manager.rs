use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::assets::{GameAssets, SpriteDatabase, sprite_position_to_index};
use crate::components::*;
use crate::map::{GameMap, select_biome_asset};
use crate::player::{LevelChangeEvent, RegenerateMapEvent, SpawnPosition};
use crate::states::GameState;
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

// Helper function to spawn map tiles with biome-aware texture selection and tile pooling
fn spawn_map_tiles(
    commands: &mut Commands,
    map: &GameMap,
    biome: BiomeType,
    saved_visibility: &std::collections::HashMap<(u32, u32), TileVisibility>,
    tile_pool: &mut TilePool,
    tile_index: &mut TileIndex,
    assets: &GameAssets,
) -> (Entity, TileStorage) {
    let tilemap_entity = commands.spawn_empty().id();
    let mut tile_storage = TileStorage::empty(TilemapSize { x: map.width, y: map.height });

    let biome_config = biome.get_config();
    let mut rng = rand::rng();
    let mut reused_tiles = 0;
    let mut new_tiles = 0;

    for y in 0..map.height {
        for x in 0..map.width {
            let tile_type = map.get(x, y);
            let (sprite_x, sprite_y) = select_biome_asset(&biome_config, tile_type, map, x, y, &mut rng);
            let texture_index = sprite_position_to_index(sprite_x, sprite_y);

            let tile_pos = TilePos { x, y };
            let visibility = saved_visibility.get(&(x, y)).copied().unwrap_or(TileVisibility::Unseen);

            // Try to reuse a tile from the pool
            let tile_entity = if let Some(pooled_entity) = tile_pool.acquire() {
                reused_tiles += 1;
                commands.entity(pooled_entity).insert((
                    TileBundle {
                        position: tile_pos,
                        tilemap_id: TilemapId(tilemap_entity),
                        texture_index: TileTextureIndex(texture_index),
                        ..Default::default()
                    },
                    MapTile { tile_type },
                    TileVisibilityState { visibility },
                ));
                pooled_entity
            } else {
                new_tiles += 1;
                commands
                    .spawn((
                        TileBundle {
                            position: tile_pos,
                            tilemap_id: TilemapId(tilemap_entity),
                            texture_index: TileTextureIndex(texture_index),
                            ..Default::default()
                        },
                        MapTile { tile_type },
                        TileVisibilityState { visibility },
                    ))
                    .id()
            };

            tile_storage.set(&tile_pos, tile_entity);
            tile_index.insert(x, y, tile_entity);
        }
    }

    println!("Tile spawning: {} reused from pool, {} newly spawned", reused_tiles, new_tiles);

    let tile_size = TilemapTileSize { x: 32.0, y: 32.0 };
    let grid_size = tile_size.into();
    let map_type = TilemapType::default();

    commands.entity(tilemap_entity).insert(TilemapBundle {
        grid_size,
        map_type,
        size: TilemapSize { x: map.width, y: map.height },
        storage: tile_storage.clone(),
        texture: TilemapTexture::Single(assets.tiles.clone()),
        tile_size,
        anchor: TilemapAnchor::Center,
        ..Default::default()
    });

    (tilemap_entity, tile_storage)
}

// Helper function to capture current tile visibility states (sparse storage)
pub fn capture_tile_visibility(
    tile_query: &Query<(&TilePos, &TileVisibilityState)>,
    _map_width: u32,
    _map_height: u32,
) -> std::collections::HashMap<(u32, u32), TileVisibility> {
    let mut visibility_data = std::collections::HashMap::new();

    // Only store non-Unseen tiles for sparse storage
    for (tile_pos, visibility_state) in tile_query.iter() {
        if visibility_state.visibility != TileVisibility::Unseen {
            visibility_data.insert((tile_pos.x, tile_pos.y), visibility_state.visibility);
        }
    }

    visibility_data
}

pub fn handle_level_transitions(
    mut commands: Commands,
    mut level_change_events: EventReader<LevelChangeEvent>,
    mut current_level: ResMut<CurrentLevel>,
    mut level_maps: ResMut<LevelMaps>,
    assets: Res<GameAssets>,
    _sprite_db: Res<SpriteDatabase>,
    mut player_query: Query<&mut Player>,
    tilemap_query: Query<Entity, With<TileStorage>>,
    tile_visibility_query: Query<Entity, With<TileVisibilityState>>,
    tile_pos_visibility_query: Query<(&TilePos, &TileVisibilityState)>,
    map: Option<Res<GameMap>>,
    mut fov_settings: ResMut<FovSettings>,
    mut tile_index: ResMut<TileIndex>,
    mut tile_pool: ResMut<TilePool>,
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

        // Clear existing tilemap
        for entity in tilemap_query.iter() {
            commands.entity(entity).despawn();
        }

        // Return tile entities to pool after removing tilemap components
        let mut returned_tiles = 0;
        for entity in tile_visibility_query.iter() {
            // Remove all tilemap-specific components to prevent stale references
            commands.entity(entity).remove::<(TilePos, TilemapId, TileTextureIndex, TileVisible, TileFlip)>();
            tile_pool.release(entity);
            returned_tiles += 1;
        }
        println!("Returned {} tiles to pool (pool size: {})", returned_tiles, tile_pool.len());
        
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
            // Create new visibility data for new map (empty HashMap = all Unseen)
            let new_visibility = std::collections::HashMap::new();
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
        
        // Clear and rebuild tile index
        tile_index.clear();

        // Spawn the new map using the helper function
        spawn_map_tiles(&mut commands, &map, current_level.biome, &saved_visibility, &mut tile_pool, &mut tile_index, &assets);

        commands.insert_resource(map);
        
        // Trigger FOV recalculation for new level and invalidate LOS cache
        fov_settings.needs_recalculation = true;
        fov_settings.los_cache.clear();
        if fov_settings.cache_hits > 0 || fov_settings.cache_misses > 0 {
            let hit_rate = fov_settings.cache_hits as f32 / (fov_settings.cache_hits + fov_settings.cache_misses) as f32 * 100.0;
            println!("LOS cache stats - Hits: {}, Misses: {}, Hit rate: {:.1}%",
                    fov_settings.cache_hits, fov_settings.cache_misses, hit_rate);
        }
        fov_settings.cache_hits = 0;
        fov_settings.cache_misses = 0;
    }
}

pub fn handle_map_regeneration(
    mut commands: Commands,
    mut regenerate_events: EventReader<RegenerateMapEvent>,
    current_level: Res<CurrentLevel>,
    mut level_maps: ResMut<LevelMaps>,
    assets: Res<GameAssets>,
    _sprite_db: Res<SpriteDatabase>,
    mut player_query: Query<&mut Player>,
    tilemap_query: Query<Entity, With<TileStorage>>,
    tile_visibility_query: Query<Entity, With<TileVisibilityState>>,
    mut fov_settings: ResMut<FovSettings>,
    mut tile_index: ResMut<TileIndex>,
    mut tile_pool: ResMut<TilePool>,
) {
    for _event in regenerate_events.read() {
        println!("Regenerating level {}", current_level.level);

        // Clear existing tilemap
        for entity in tilemap_query.iter() {
            commands.entity(entity).despawn();
        }

        // Return tile entities to pool after removing tilemap components
        for entity in tile_visibility_query.iter() {
            // Remove all tilemap-specific components to prevent stale references
            commands.entity(entity).remove::<(TilePos, TilemapId, TileTextureIndex, TileVisible, TileFlip)>();
            tile_pool.release(entity);
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
                            if map.get(ux, uy) == TileType::Floor {
                                spawn_pos = (ux, uy);
                                break;
                            }
                        }
                    }
                }
                if map.get(spawn_pos.0, spawn_pos.1) == TileType::Floor {
                    break;
                }
            }
            
            player.x = spawn_pos.0;
            player.y = spawn_pos.1;
            
            println!("Player repositioned at ({}, {})", player.x, player.y);
        }
        
        // Save the new map (empty HashMap = all Unseen)
        let new_visibility = std::collections::HashMap::new();
        level_maps.maps.insert(current_level.level, map.to_saved_data(current_level.biome, new_visibility.clone()));

        // Clear and rebuild tile index
        tile_index.clear();

        // Spawn the new map using the helper function
        spawn_map_tiles(&mut commands, &map, current_level.biome, &new_visibility, &mut tile_pool, &mut tile_index, &assets);

        commands.insert_resource(map);

        // Trigger FOV recalculation for regenerated map and invalidate LOS cache
        fov_settings.needs_recalculation = true;
        fov_settings.los_cache.clear();
        if fov_settings.cache_hits > 0 || fov_settings.cache_misses > 0 {
            let hit_rate = fov_settings.cache_hits as f32 / (fov_settings.cache_hits + fov_settings.cache_misses) as f32 * 100.0;
            println!("LOS cache stats - Hits: {}, Misses: {}, Hit rate: {:.1}%",
                    fov_settings.cache_hits, fov_settings.cache_misses, hit_rate);
        }
        fov_settings.cache_hits = 0;
        fov_settings.cache_misses = 0;
    }
}
