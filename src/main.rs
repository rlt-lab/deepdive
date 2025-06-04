use bevy::{
    prelude::*,
    window::{PresentMode, WindowResolution},
};
use bevy_asset_loader::prelude::*;
use bevy_ecs_tilemap::prelude::*;

mod components;
mod states;
mod assets;
mod map;
mod map_generation;
mod player;
mod camera;
mod level_manager;
mod fov;
mod biome;
mod ui;

use assets::{GameAssets, SpriteDatabase};
use states::GameState;
use map::spawn_map;
use player::*;
use camera::*;
use level_manager::LevelManagerPlugin;
use fov::FovPlugin;
use ui::UiPlugin;

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
        .init_state::<GameState>()
        .insert_resource(ClearColor(Color::BLACK)) // Set background to black
        .insert_resource(SpriteDatabase::new()) // Add sprite database resource
        .add_loading_state(
            LoadingState::new(GameState::AssetLoading)
                .continue_to_state(GameState::Playing)
                .load_collection::<GameAssets>()
        )
        .add_systems(Startup, setup_camera)
        .add_systems(OnEnter(GameState::Playing), (
            spawn_map, 
            spawn_player.after(spawn_map),
            setup_camera_follow.after(spawn_player)
        ))
        .add_systems(Update, (
            handle_input, 
            animate_movement,
            move_player.after(animate_movement), // Ensure move_player runs after animation
            handle_continuous_movement,
            handle_stair_interaction,
            debug_map_regeneration,
            debug_biome_cycling,
            camera_follow_system,
            camera_zoom_system,
            camera_debug_system
        ).run_if(in_state(GameState::Playing)))
        .run();
}
