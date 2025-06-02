use bevy::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq)]
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
