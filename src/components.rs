use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::biome::BiomeType;

// ============================================================================
// TILE TYPES & MAP ENUMS
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, Reflect)]
pub enum TileType {
    Floor,
    Wall,
    Water,
    StairUp,
    StairDown,
}

// ============================================================================
// PLAYER COMPONENTS
// ============================================================================

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Player {
    pub x: u32,
    pub y: u32,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct MovementAnimation {
    pub start_pos: Vec3,
    pub end_pos: Vec3,
    pub timer: Timer,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct MovementInput {
    pub move_timer: Timer,
    pub is_holding: bool,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Autoexplore {
    pub active: bool,
    pub path: Vec<(u32, u32)>,
    pub target: Option<(u32, u32)>,
    pub move_timer: Timer,
}

impl Default for Autoexplore {
    fn default() -> Self {
        Self {
            active: false,
            path: Vec::new(),
            target: None,
            move_timer: Timer::from_seconds(0.001, TimerMode::Repeating), // Blazing fast autoexplore
        }
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct AutoMoveToStair {
    pub path: Vec<(u32, u32)>,
    pub target: (u32, u32),
    pub stair_type: TileType,
    pub move_timer: Timer,
}

impl AutoMoveToStair {
    pub fn new(target: (u32, u32), path: Vec<(u32, u32)>, stair_type: TileType) -> Self {
        Self {
            path,
            target,
            stair_type,
            move_timer: Timer::from_seconds(0.05, TimerMode::Repeating), // Fast movement to stairs
        }
    }
}

// ============================================================================
// MAP COMPONENTS
// ============================================================================

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct MapTile {
    pub tile_type: TileType,
}

// ============================================================================
// FOV COMPONENTS
// ============================================================================

#[derive(Component, Clone, Copy, Debug, PartialEq, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub enum TileVisibility {
    Unseen,
    Seen,
    Visible,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct TileVisibilityState {
    pub visibility: TileVisibility,
}

// ============================================================================
// PARTICLE COMPONENTS
// ============================================================================

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct BiomeParticle {
    pub lifetime: Timer,
    pub velocity: Vec2,
    pub glow_timer: Timer,
    pub layer_speed: f32,
    pub original_alpha: f32,
    // Bit-packed data: wind_offset (10 bits) | size_offset (10 bits) | color_shift (10 bits) | particle_type (1 bit)
    pub packed_data: u32,
}

#[derive(Clone, Copy, PartialEq, Reflect)]
pub enum ParticleType {
    Primary,
    Secondary,
}

impl BiomeParticle {
    // Constants for bit packing
    const OFFSET_BITS: u32 = 10;
    const OFFSET_MASK: u32 = (1 << Self::OFFSET_BITS) - 1; // 0x3FF (1023)
    const MAX_OFFSET_VALUE: f32 = std::f32::consts::TAU; // 2π ≈ 6.28

    // Pack three f32 offsets (0.0..6.28) and particle type into u32
    #[inline]
    pub fn pack(wind_offset: f32, size_offset: f32, color_shift: f32, particle_type: ParticleType) -> u32 {
        let wind = ((wind_offset / Self::MAX_OFFSET_VALUE * Self::OFFSET_MASK as f32) as u32) & Self::OFFSET_MASK;
        let size = ((size_offset / Self::MAX_OFFSET_VALUE * Self::OFFSET_MASK as f32) as u32) & Self::OFFSET_MASK;
        let color = ((color_shift / Self::MAX_OFFSET_VALUE * Self::OFFSET_MASK as f32) as u32) & Self::OFFSET_MASK;
        let ptype = match particle_type {
            ParticleType::Primary => 0,
            ParticleType::Secondary => 1,
        };

        wind | (size << Self::OFFSET_BITS) | (color << (Self::OFFSET_BITS * 2)) | (ptype << 30)
    }

    // Unpack wind_offset from packed_data
    #[inline]
    pub fn wind_offset(&self) -> f32 {
        let value = self.packed_data & Self::OFFSET_MASK;
        (value as f32 / Self::OFFSET_MASK as f32) * Self::MAX_OFFSET_VALUE
    }

    // Unpack size_offset from packed_data
    #[inline]
    pub fn size_offset(&self) -> f32 {
        let value = (self.packed_data >> Self::OFFSET_BITS) & Self::OFFSET_MASK;
        (value as f32 / Self::OFFSET_MASK as f32) * Self::MAX_OFFSET_VALUE
    }

    // Unpack color_shift from packed_data
    #[inline]
    pub fn color_shift(&self) -> f32 {
        let value = (self.packed_data >> (Self::OFFSET_BITS * 2)) & Self::OFFSET_MASK;
        (value as f32 / Self::OFFSET_MASK as f32) * Self::MAX_OFFSET_VALUE
    }

    // Unpack particle_type from packed_data
    #[inline]
    pub fn particle_type(&self) -> ParticleType {
        if (self.packed_data >> 30) & 1 == 0 {
            ParticleType::Primary
        } else {
            ParticleType::Secondary
        }
    }
}

// ============================================================================
// CAMERA COMPONENTS
// ============================================================================

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct CameraFollow {
    pub target: Entity,
    pub lerp_speed: f32,
    pub zoom_level: f32,
    pub target_zoom: f32,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct GameCamera;

// ============================================================================
// UI COMPONENTS
// ============================================================================

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct DepthIndicator;

// ============================================================================
// RESOURCES
// ============================================================================

#[derive(Resource, Deref, DerefMut)]
pub struct PlayerEntity(pub Entity);

#[derive(Resource)]
pub struct PlayerSpriteConfig {
    pub sprite_rect: Rect,
    pub custom_size: Vec2,
}

#[derive(Resource)]
pub struct CurrentLevel {
    pub level: u32,
    pub biome: BiomeType,
}

#[derive(Resource, Default)]
pub struct LevelMaps {
    pub maps: std::collections::HashMap<u32, SavedMapData>,
}

#[derive(Resource)]
pub struct FovSettings {
    pub radius: u32,
    pub debug_reveal_all: bool,
    pub needs_recalculation: bool,
    pub debug_mode_applied: bool,
    // Dirty tracking for incremental FOV updates
    pub last_player_pos: Option<(u32, u32)>,
    pub dirty_tiles: std::collections::HashSet<(u32, u32)>,
    // LOS caching for symmetric line-of-sight calculations
    pub los_cache: std::collections::HashMap<(u32, u32, u32, u32), bool>,
    pub cache_hits: usize,
    pub cache_misses: usize,
}

impl Default for FovSettings {
    fn default() -> Self {
        Self {
            radius: 20, // 2.5x the original radius of 8
            debug_reveal_all: false,
            needs_recalculation: true,
            debug_mode_applied: false,
            last_player_pos: None,
            dirty_tiles: std::collections::HashSet::new(),
            los_cache: std::collections::HashMap::new(),
            cache_hits: 0,
            cache_misses: 0,
        }
    }
}

#[derive(Resource)]
pub struct ParticleSpawner {
    pub primary_timer: Timer,
    pub secondary_timer: Timer,
    pub current_biome: BiomeType,
    pub config: crate::particles::BiomeParticleConfig,
    pub initial_spawn_complete: bool,
}

#[derive(Resource)]
pub struct WindState {
    pub timer: Timer,
    pub strength: f32,
    pub direction: Vec2,
    pub base_multiplier: f32,
}

#[derive(Resource)]
pub struct ParticleSettings {
    pub density_multiplier: f32,
    pub enabled: bool,
    pub debug_mode: bool,
}

impl Default for ParticleSettings {
    fn default() -> Self {
        Self {
            density_multiplier: 6.0, // 6x density for very rich atmospheric effects
            enabled: true,
            debug_mode: false,
        }
    }
}

// ============================================================================
// SPATIAL INDEXING
// ============================================================================

#[derive(Resource, Default)]
pub struct TileIndex {
    // Maps (x, y) position to tile entity for O(1) lookups
    pub tiles: std::collections::HashMap<(u32, u32), Entity>,
}

impl TileIndex {
    pub fn insert(&mut self, x: u32, y: u32, entity: Entity) {
        self.tiles.insert((x, y), entity);
    }

    pub fn clear(&mut self) {
        self.tiles.clear();
    }
}

#[derive(Resource)]
pub struct TilePool {
    // Pool of available tile entities for reuse
    pub available: Vec<Entity>,
    pub max_pool_size: usize,
}

impl Default for TilePool {
    fn default() -> Self {
        Self {
            available: Vec::new(),
            max_pool_size: 4000, // 80x50 map = 4000 tiles
        }
    }
}

impl TilePool {
    // Get a tile entity from pool or indicate need to spawn new
    pub fn acquire(&mut self) -> Option<Entity> {
        self.available.pop()
    }

    // Return a tile entity to the pool for reuse
    pub fn release(&mut self, entity: Entity) {
        if self.available.len() < self.max_pool_size {
            self.available.push(entity);
        }
        // If pool is full, entity will be despawned by caller
    }

    pub fn len(&self) -> usize {
        self.available.len()
    }
}

// ============================================================================
// DATA STRUCTURES
// ============================================================================

#[derive(Clone, Serialize, Deserialize)]
pub struct SavedMapData {
    pub width: u32,
    pub height: u32,
    pub tiles: Vec<TileType>,
    pub stair_up_pos: Option<(u32, u32)>,
    pub stair_down_pos: Option<(u32, u32)>,
    pub biome: BiomeType,
    // Sparse storage: only store non-Unseen tiles (HashMap: position -> visibility state)
    pub tile_visibility: std::collections::HashMap<(u32, u32), TileVisibility>,
}
