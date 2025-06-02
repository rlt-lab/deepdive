use bevy::{
    prelude::*,
    window::{PresentMode, WindowResolution},
};
use bevy_asset_loader::prelude::*;
use bevy_ecs_tilemap::prelude::*;

#[derive(AssetCollection, Resource)]
struct GameAssets {
    #[asset(path = "sprites/rogues.png")]
    rogues: Handle<Image>,
    #[asset(path = "sprites/tiles.png")]
    tiles: Handle<Image>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum TileType {
    Floor,
    Wall,
    Water,
    StairUp,
    StairDown,
}

#[derive(Component)]
struct MapTile {
    tile_type: TileType,
}

#[derive(Resource)]
struct GameMap {
    width: u32,
    height: u32,
    tiles: Vec<Vec<TileType>>,
}

impl GameMap {
    fn new(width: u32, height: u32) -> Self {
        let tiles = vec![vec![TileType::Floor; width as usize]; height as usize];
        Self { width, height, tiles }
    }
    
    fn generate_simple_room(&mut self) {
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
}

#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
    #[default]
    AssetLoading,
    Playing,
}

#[derive(Component)]
struct Player {
    x: u32,
    y: u32,
}

#[derive(Component)]
struct GridPosition {
    x: u32,
    y: u32,
}

#[derive(Component)]
struct MovementAnimation {
    start_pos: Vec3,
    end_pos: Vec3,
    timer: Timer,
}

#[derive(Component)]
struct MovementInput {
    move_timer: Timer,
    is_holding: bool,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Roguelike".into(),
                resolution: WindowResolution::new(1400.0, 800.0),
                present_mode: PresentMode::Fifo, // Vsync
                ..default()
            }),
            ..default()
        }).set(ImagePlugin::default_nearest()))
        .add_plugins(TilemapPlugin)
        .init_state::<GameState>()
        .add_loading_state(
            LoadingState::new(GameState::AssetLoading)
                .continue_to_state(GameState::Playing)
                .load_collection::<GameAssets>()
        )
        .add_systems(Startup, setup_camera)
        .add_systems(OnEnter(GameState::Playing), (spawn_map, spawn_player))
        .add_systems(Update, (
            handle_input, 
            move_player, 
            animate_movement,
            handle_continuous_movement
        ).run_if(in_state(GameState::Playing)))
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn spawn_player(
    mut commands: Commands,
    assets: Res<GameAssets>,
) {
    // Spawn player at grid position (2, 2) - center of the room
    let grid_x = 2;
    let grid_y = 2;
    
    // Convert grid position to world position
    // With TilemapAnchor::Center, the tilemap is centered at (0,0)
    // Each tile is 32x32, so tile (0,0) is at (-144, -144) for a 10x10 map
    // We need to position the player at the center of the tile
    let world_x = (grid_x as f32 - 4.5) * 32.0; // -4.5 centers us on tiles instead of grid lines
    let world_y = (grid_y as f32 - 4.5) * 32.0;
    
    // Player sprite at position 1,4 from rogues.png (32x32 sprites)
    let sprite_x = 1.0 * 32.0;
    let sprite_y = 4.0 * 32.0;
    
    commands.spawn((
        Player { x: grid_x, y: grid_y },
        GridPosition { x: grid_x, y: grid_y },
        MovementInput {
            move_timer: Timer::from_seconds(0.15, TimerMode::Once), // 150ms for hold-to-move
            is_holding: false,
        },
        Sprite {
            image: assets.rogues.clone(),
            rect: Some(Rect::new(sprite_x, sprite_y, sprite_x + 32.0, sprite_y + 32.0)),
            flip_x: false, // Start facing left (natural sprite direction)
            ..default()
        },
        Transform::from_xyz(world_x, world_y, 1.0),
    ));
}

fn spawn_map(
    mut commands: Commands,
    assets: Res<GameAssets>,
) {
    let mut map = GameMap::new(10, 10);
    map.generate_simple_room();
    
    let tilemap_entity = commands.spawn_empty().id();
    let mut tile_storage = TileStorage::empty(TilemapSize { x: 10, y: 10 });
    
    // Spawn tiles
    for y in 0..10 {
        for x in 0..10 {
            let tile_type = map.tiles[y as usize][x as usize];
            let texture_index = match tile_type {
                TileType::Floor => 0, // First tile in tiles.png
                TileType::Wall => 1,  // Second tile in tiles.png
                _ => 0,
            };
            
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

fn handle_input(
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
                let start_world_x = (player.x as f32 - 4.5) * 32.0;
                let start_world_y = (player.y as f32 - 4.5) * 32.0;
                let end_world_x = (new_x as f32 - 4.5) * 32.0;
                let end_world_y = (new_y as f32 - 4.5) * 32.0;
                
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

fn move_player(
    mut player_query: Query<(&Player, &mut Transform), (Changed<Player>, Without<MovementAnimation>)>,
) {
    // Only update transform for players without active movement animation
    for (player, mut transform) in player_query.iter_mut() {
        // Convert grid position to world position
        let world_x = (player.x as f32 - 4.5) * 32.0; // -4.5 centers us on tiles
        let world_y = (player.y as f32 - 4.5) * 32.0;
        
        transform.translation.x = world_x;
        transform.translation.y = world_y;
    }
}

fn animate_movement(
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

fn handle_continuous_movement(
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
