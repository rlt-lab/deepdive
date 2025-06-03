use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::biome::BiomeType;
use crate::fov::TileVisibility;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum TileType {
    Floor,
    Wall,
    Water,
    StairUp,
    StairDown,
}

#[derive(Component)]
pub struct MapTile {
    pub tile_type: TileType,
}

#[derive(Component)]
pub struct Player {
    pub x: u32,
    pub y: u32,
}

#[derive(Component)]
pub struct GridPosition {
    pub x: u32,
    pub y: u32,
}

#[derive(Component)]
pub struct MovementAnimation {
    pub start_pos: Vec3,
    pub end_pos: Vec3,
    pub timer: Timer,
}

#[derive(Component)]
pub struct MovementInput {
    pub move_timer: Timer,
    pub is_holding: bool,
}

#[derive(Component)]
pub struct CameraFollow {
    pub target: Entity,
    pub lerp_speed: f32,
    pub zoom_level: f32,
    pub target_zoom: f32,
}

#[derive(Component)]
pub struct GameCamera;

#[derive(Resource)]
pub struct PlayerEntity(pub Entity);

#[derive(Resource)]
pub struct CurrentLevel {
    pub level: u32,
    pub biome: BiomeType, // Add biome field
}

#[derive(Resource, Default)]
pub struct LevelMaps {
    pub maps: std::collections::HashMap<u32, SavedMapData>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SavedMapData {
    pub width: u32,
    pub height: u32,
    pub tiles: Vec<Vec<TileType>>,
    pub stair_up_pos: Option<(u32, u32)>,
    pub stair_down_pos: Option<(u32, u32)>,
    pub biome: BiomeType, // Add biome field
    pub tile_visibility: Vec<Vec<TileVisibility>>, // Add tile visibility data
}

#[derive(Clone, Copy, PartialEq)]
pub enum MapGenerationType {
    SimpleRoom,
    DrunkardWalk,
}
