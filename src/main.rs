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
mod player;
mod camera;

use assets::{GameAssets, SpriteDatabase};
use states::GameState;
use components::*;
use map::spawn_map;
use player::*;
use camera::*;

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
        .insert_resource(SpriteDatabase::new()) // Add sprite database resource
        .add_loading_state(
            LoadingState::new(GameState::AssetLoading)
                .continue_to_state(GameState::Playing)
                .load_collection::<GameAssets>()
        )
        .add_systems(Startup, setup_camera)
        .add_systems(OnEnter(GameState::Playing), (
            spawn_map, 
            spawn_player.before(setup_camera_follow),
            setup_camera_follow
        ))
        .add_systems(Update, (
            handle_input, 
            move_player, 
            animate_movement,
            handle_continuous_movement,
            camera_follow_system,
            camera_zoom_system,
            camera_debug_system
        ).run_if(in_state(GameState::Playing)))
        .run();
}
