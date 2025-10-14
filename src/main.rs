use bevy::{
    prelude::*,
    window::{PresentMode, WindowResolution},
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
};
use bevy_asset_loader::prelude::*;
use bevy_ecs_tilemap::prelude::*;

mod components;
mod states;
mod assets;
mod map;
mod map_generation;
mod map_generation_compact;
mod player;
mod input_handler;
mod camera;
mod level_manager;
mod fov;
mod biome;
mod ui;
mod particles;

use assets::{GameAssets, SpriteDatabase};
use states::GameState;
use map::spawn_map;
use player::*;
use input_handler::*;
use camera::*;
use level_manager::LevelManagerPlugin;
use fov::FovPlugin;
use ui::UiPlugin;
use particles::ParticlePlugin;
use components::{*, GlobalRng, EllipseMask};

// System sets for organizing update systems
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
enum GameplaySet {
    Input,          // Handle user input
    Movement,       // Process movement and animation
    Camera,         // Camera follow and zoom
    Debug,          // Debug controls
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
        .add_plugins(LevelManagerPlugin)
        .add_plugins(FovPlugin)
        .add_plugins(UiPlugin)
        .add_plugins(ParticlePlugin)
        // Add diagnostics plugins for performance monitoring
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(LogDiagnosticsPlugin::default())
        .init_state::<GameState>()
        .init_resource::<TileIndex>()
        .init_resource::<TilePool>()
        .init_resource::<KeyBindings>()
        .insert_resource(EllipseMask::new(80, 50)) // Pre-calculate ellipse boundary for 80x50 maps
        .insert_resource(GlobalRng::new())
        // Register component types for reflection
        .register_type::<Player>()
        .register_type::<MovementAnimation>()
        .register_type::<MovementInput>()
        .register_type::<Autoexplore>()
        .register_type::<AutoMoveToStair>()
        .register_type::<MapTile>()
        .register_type::<TileVisibility>()
        .register_type::<TileVisibilityState>()
        .register_type::<BiomeParticle>() // Bevy automatically optimizes storage for frequently added/removed components
        .register_type::<ParticleType>()
        .register_type::<CameraFollow>()
        .register_type::<GameCamera>()
        .register_type::<DepthIndicator>()
        .insert_resource(ClearColor(Color::BLACK)) // Set background to black
        .insert_resource(SpriteDatabase::new()) // Add sprite database resource
        // Cache player sprite configuration
        .insert_resource(PlayerSpriteConfig {
            sprite_rect: Rect::new(128.5, 128.5, 159.5, 159.5), // Player at (4,4) with 31x31 extract
            custom_size: Vec2::new(32.0, 32.0),
        })
        // Add player movement event
        .add_event::<PlayerMoveIntent>()
        .add_loading_state(
            LoadingState::new(GameState::AssetLoading)
                .continue_to_state(GameState::Playing)
                .load_collection::<GameAssets>()
        )
        .configure_sets(Update, (
            GameplaySet::Input,
            GameplaySet::Movement,
            GameplaySet::Camera,
            GameplaySet::Debug,
        ).chain().run_if(in_state(GameState::Playing)))
        .add_systems(Startup, setup_camera)
        .add_systems(OnEnter(GameState::Playing), (
            spawn_map,
            spawn_player.after(spawn_map),
            setup_camera_follow.after(spawn_player)
        ))
        .add_systems(Update, (
            detect_movement_input,
            handle_movement_input.after(detect_movement_input),
            handle_stair_interaction,
            toggle_autoexplore,
            run_autoexplore,
            run_auto_move_to_stair,
        ).in_set(GameplaySet::Input))
        .add_systems(Update, (
            animate_movement,
            move_player.after(animate_movement),
        ).in_set(GameplaySet::Movement))
        .add_systems(Update, (
            camera_follow_system,
            camera_zoom_system,
        ).in_set(GameplaySet::Camera))
        .add_systems(Update, (
            debug_map_regeneration,
            debug_biome_cycling,
            camera_debug_system,
        ).in_set(GameplaySet::Debug))
        .run();
}
