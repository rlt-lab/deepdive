use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use std::collections::HashMap;

use crate::components::{Player, CurrentLevel, TileType, MapTile};
use crate::biome::BiomeType;
use crate::states::GameState;
use crate::map::GameMap;

// Base particle system constants
const PARTICLE_FADE_DISTANCE: f32 = 900.0;
const CAMERA_VIEW_RADIUS: f32 = 30.0;

// Biome-specific particle configuration
#[derive(Clone, Debug)]
pub struct BiomeParticleConfig {
    pub primary_max_particles: usize,
    pub secondary_max_particles: usize,
    pub primary_spawn_rate: f32,
    pub secondary_spawn_rate: f32,
    pub secondary_spawn_chance: f32,
    pub primary_colors: Vec<Color>,
    pub secondary_colors: Vec<Color>,
    pub primary_size_range: (f32, f32),
    pub secondary_size_range: (f32, f32),
    pub primary_lifetime_range: (f32, f32),
    pub secondary_lifetime_range: (f32, f32),
    pub primary_velocity_range: (Vec2, Vec2),
    pub secondary_velocity_range: (Vec2, Vec2),
    pub wind_strength_multiplier: f32,
    pub movement_style: MovementStyle,
    pub enabled: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum MovementStyle {
    Gentle,      // Slow, smooth movement (Underglade)
    Erratic,     // Sharp, unpredictable movement (Fungal Deep spores)
    Floating,    // Slow vertical drift (Caverns mist)
    Swirling,    // Circular patterns (Cinder Gaol embers)
    Flowing,     // Water-like movement (Stygian Pool)
}

#[derive(Component)]
pub struct ForestParticle {
    pub particle_type: ParticleType,
    pub lifetime: Timer,
    pub velocity: Vec2,
    pub wind_offset: f32,
    pub size_offset: f32,
    pub color_shift: f32,
    pub glow_timer: Timer,
    pub layer_speed: f32,
    pub original_alpha: f32,
}

#[derive(Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub enum ParticleType {
    Pollen,
    Firefly,
    MagicShimmer,
}

#[derive(Resource)]
pub struct ParticleSpawner {
    pub pollen_timer: Timer,
    pub firefly_timer: Timer,
    pub wind_timer: Timer,
    pub wind_strength: f32,
    pub wind_direction: Vec2,
    pub is_active: bool,
    pub initial_spawn_complete: bool,
}

impl Default for ParticleSpawner {
    fn default() -> Self {
        Self {
            pollen_timer: Timer::from_seconds(1.0 / POLLEN_SPAWN_RATE, TimerMode::Repeating),
            firefly_timer: Timer::from_seconds(1.0 / FIREFLY_SPAWN_RATE, TimerMode::Repeating),
            wind_timer: Timer::from_seconds(6.0, TimerMode::Repeating), // More frequent wind changes
            wind_strength: 1.2, // Increased base wind strength
            wind_direction: Vec2::new(1.0, 0.0),
            is_active: false,
            initial_spawn_complete: false,
        }
    }
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
            density_multiplier: 1.0,
            enabled: true,
            debug_mode: false,
        }
    }
}

pub struct ParticlePlugin;

impl Plugin for ParticlePlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<ParticleSpawner>()
            .init_resource::<ParticleSettings>()
            .add_systems(Update, (
                update_particle_spawner,
                spawn_forest_particles,
                update_forest_particles,
                update_wind_system,
                cleanup_particles,
                handle_particle_debug
            ).run_if(in_state(GameState::Playing)));
    }
}

fn update_particle_spawner(
    time: Res<Time>,
    mut spawner: ResMut<ParticleSpawner>,
    current_level: Res<CurrentLevel>,
    settings: Res<ParticleSettings>,
) {
    let was_active = spawner.is_active;
    
    // Only activate particle system in Underglade biome
    spawner.is_active = settings.enabled && current_level.biome == BiomeType::Underglade;
    
    // Reset initial spawn flag when biome changes or system becomes inactive
    if !spawner.is_active || (!was_active && spawner.is_active) {
        spawner.initial_spawn_complete = false;
    }
    
    if !spawner.is_active {
        return;
    }

    spawner.pollen_timer.tick(time.delta());
    spawner.firefly_timer.tick(time.delta());
    spawner.wind_timer.tick(time.delta());
}

fn spawn_forest_particles(
    mut commands: Commands,
    mut spawner: ResMut<ParticleSpawner>,
    settings: Res<ParticleSettings>,
    player_query: Query<&Transform, With<Player>>,
    tile_query: Query<(&TilePos, &MapTile)>,
    existing_particles: Query<&ForestParticle>,
    map: Res<GameMap>,
) {
    if !spawner.is_active {
        return;
    }

    let Ok(player_transform) = player_query.single() else {
        return;
    };

    // Use world coordinates instead of tile coordinates for better distribution
    let camera_center = Vec2::new(
        player_transform.translation.x,
        player_transform.translation.y,
    );

    // Count existing particles
    let pollen_count = existing_particles.iter()
        .filter(|p| p.particle_type == ParticleType::Pollen)
        .count();
    let firefly_count = existing_particles.iter()
        .filter(|p| p.particle_type == ParticleType::Firefly)
        .count();

    // Initial spawn - fill visible area with better distribution
    if !spawner.initial_spawn_complete {
        // Spawn more particles in a wider grid pattern for better coverage
        for _ in 0..(MAX_POLLEN_PARTICLES * 2 / 3) {
            if let Some(spawn_pos) = find_camera_spawn_position(&camera_center, &tile_query, &map) {
                spawn_pollen_particle(&mut commands, spawn_pos);
            }
        }
        
        // Spawn initial fireflies
        for _ in 0..(MAX_FIREFLY_PARTICLES * 2 / 3) {
            if let Some(spawn_pos) = find_camera_spawn_position(&camera_center, &tile_query, &map) {
                spawn_firefly_particle(&mut commands, spawn_pos);
            }
        }
        
        spawner.initial_spawn_complete = true;
        println!("Initial particle spawn complete for Underglade biome (increased density)");
    }

    // Continuous spawning with increased spawn rate
    if spawner.pollen_timer.just_finished() && pollen_count < MAX_POLLEN_PARTICLES {
        let spawn_count = (3.0 * settings.density_multiplier).max(2.0) as usize;
        for _ in 0..spawn_count {
            if let Some(spawn_pos) = find_camera_spawn_position(&camera_center, &tile_query, &map) {
                spawn_pollen_particle(&mut commands, spawn_pos);
            }
        }
    }

    // Spawn firefly particles
    if spawner.firefly_timer.just_finished() && firefly_count < MAX_FIREFLY_PARTICLES {
        use rand::random;
        if random::<f32>() < FIREFLY_SPAWN_CHANCE {
            if let Some(spawn_pos) = find_camera_spawn_position(&camera_center, &tile_query, &map) {
                spawn_firefly_particle(&mut commands, spawn_pos);
            }
        }
    }
}

fn find_camera_spawn_position(
    camera_center: &Vec2,
    tile_query: &Query<(&TilePos, &MapTile)>,
    map: &GameMap,
) -> Option<Vec2> {
    use rand::Rng;
    let mut rng = rand::rng();
    
    // Convert camera center from world coordinates to tile coordinates
    // Reverse the world position calculation: world_pos = (tile_pos - (map_size/2 - 0.5)) * 32
    // So: tile_pos = (world_pos / 32) + (map_size/2 - 0.5)
    let camera_tile_x = (camera_center.x / 32.0) + (map.width as f32 / 2.0 - 0.5);
    let camera_tile_y = (camera_center.y / 32.0) + (map.height as f32 / 2.0 - 0.5);
    
    // Reduced attempts for better performance
    for _ in 0..8 {
        // Spawn in a wide area around the camera center in world coordinates
        let offset_tiles = Vec2::new(
            rng.random_range(-CAMERA_VIEW_RADIUS..CAMERA_VIEW_RADIUS),
            rng.random_range(-CAMERA_VIEW_RADIUS..CAMERA_VIEW_RADIUS),
        );
        
        let spawn_tile_x = camera_tile_x + offset_tiles.x;
        let spawn_tile_y = camera_tile_y + offset_tiles.y;
        
        // Check if position is within map bounds
        if spawn_tile_x < 0.0 || spawn_tile_x >= map.width as f32 ||
           spawn_tile_y < 0.0 || spawn_tile_y >= map.height as f32 {
            continue;
        }
        
        // Check if position is within the elliptical boundary
        if !is_within_ellipse(spawn_tile_x as u32, spawn_tile_y as u32, map.width, map.height) {
            continue;
        }
        
        let spawn_tile_pos = Vec2::new(spawn_tile_x, spawn_tile_y);
        
        // Use faster, less strict tile checking
        if is_suitable_for_particles_fast(spawn_tile_pos, tile_query) {
            // Convert back to world coordinates with proper map centering
            let world_x = (spawn_tile_x - (map.width as f32 / 2.0 - 0.5)) * 32.0 + rng.random_range(-16.0..16.0);
            let world_y = (spawn_tile_y - (map.height as f32 / 2.0 - 0.5)) * 32.0 + rng.random_range(-16.0..16.0);
            return Some(Vec2::new(world_x, world_y));
        }
    }
    
    None
}

fn is_suitable_for_particles_fast(pos: Vec2, tile_query: &Query<(&TilePos, &MapTile)>) -> bool {
    let tile_x = pos.x as u32;
    let tile_y = pos.y as u32;
    
    // Fast check - just look for floor tiles, don't check walls extensively
    for (tile_pos, map_tile) in tile_query.iter() {
        if tile_pos.x == tile_x && tile_pos.y == tile_y {
            return map_tile.tile_type == TileType::Floor;
        }
    }
    
    // If no exact tile found, assume it's valid (empty space)
    true
}

// Helper function to check if a position is within the elliptical boundary
// This mirrors the logic from GameMap::is_within_ellipse
fn is_within_ellipse(x: u32, y: u32, map_width: u32, map_height: u32) -> bool {
    let center_x = map_width as f32 / 2.0;
    let center_y = map_height as f32 / 2.0;
    
    // Create an ellipse that fits within the map bounds with some padding
    let a = (map_width as f32 / 2.0) - 2.0; // Semi-major axis (horizontal)
    let b = (map_height as f32 / 2.0) - 2.0; // Semi-minor axis (vertical)
    
    let dx = x as f32 - center_x;
    let dy = y as f32 - center_y;
    
    // Ellipse equation: (x-h)²/a² + (y-k)²/b² <= 1
    (dx * dx) / (a * a) + (dy * dy) / (b * b) <= 1.0
}

fn spawn_pollen_particle(commands: &mut Commands, spawn_pos: Vec2) {
    use rand::Rng;
    let mut rng = rand::rng();
    
    let lifetime = rng.random_range(12.0..20.0); // Longer lifetime for more visible wafting
    let velocity = Vec2::new(
        rng.random_range(-8.0..8.0), // Increased movement for more wafting
        rng.random_range(-6.0..4.0),
    );
    
    let color = Color::srgb(
        rng.random_range(0.7..1.0),   // R: 180-255
        rng.random_range(0.78..0.94), // G: 200-240
        rng.random_range(0.47..0.71), // B: 120-180
    );
    
    commands.spawn((
        Sprite {
            color,
            custom_size: Some(Vec2::new(2.5, 2.5)), // Slightly larger
            ..default()
        },
        Transform::from_translation(spawn_pos.extend(1.0)),
        ForestParticle {
            particle_type: ParticleType::Pollen,
            lifetime: Timer::from_seconds(lifetime, TimerMode::Once),
            velocity,
            wind_offset: rng.random_range(0.0..6.28),
            size_offset: rng.random_range(0.0..6.28),
            color_shift: rng.random_range(0.0..6.28),
            glow_timer: Timer::from_seconds(1.0, TimerMode::Repeating),
            layer_speed: rng.random_range(0.6..1.4), // More variable speed
            original_alpha: color.alpha(),
        },
    ));
}

fn spawn_firefly_particle(commands: &mut Commands, spawn_pos: Vec2) {
    use rand::Rng;
    let mut rng = rand::rng();
    
    let lifetime = rng.random_range(10.0..16.0); // Longer lifetime for more visible movement
    let velocity = Vec2::new(
        rng.random_range(-5.0..5.0), // Increased movement for more dynamic flying
        rng.random_range(-4.0..4.0),
    );
    
    let color = if rng.random_bool(0.5) {
        Color::srgb(0.78, 1.0, 0.78)  // Soft green-white
    } else {
        Color::srgb(0.71, 0.86, 1.0)  // Soft blue-white
    };
    
    commands.spawn((
        Sprite {
            color,
            custom_size: Some(Vec2::new(3.5, 3.5)), // Slightly larger
            ..default()
        },
        Transform::from_translation(spawn_pos.extend(2.0)), // Higher Z for fireflies
        ForestParticle {
            particle_type: ParticleType::Firefly,
            lifetime: Timer::from_seconds(lifetime, TimerMode::Once),
            velocity,
            wind_offset: rng.random_range(0.0..6.28),
            size_offset: rng.random_range(0.0..6.28),
            color_shift: rng.random_range(0.0..6.28),
            glow_timer: Timer::from_seconds(rng.random_range(1.5..3.0), TimerMode::Repeating),
            layer_speed: rng.random_range(0.4..1.2), // More variable speed
            original_alpha: color.alpha(),
        },
    ));
}

fn update_forest_particles(
    time: Res<Time>,
    spawner: Res<ParticleSpawner>,
    mut particle_query: Query<(Entity, &mut ForestParticle, &mut Transform, &mut Sprite)>,
    player_query: Query<&Transform, (With<Player>, Without<ForestParticle>)>,
    tile_query: Query<(&TilePos, &MapTile)>,
) {
    if !spawner.is_active {
        return;
    }

    let Ok(player_transform) = player_query.single() else {
        return;
    };

    let delta = time.delta_secs();
    let current_time = time.elapsed_secs();

    for (_entity, mut particle, mut transform, mut sprite) in particle_query.iter_mut() {
        // Update lifetime
        particle.lifetime.tick(time.delta());
        particle.glow_timer.tick(time.delta());

        // Enhanced movement calculation for more dynamic wafting
        let mut movement = particle.velocity * delta * particle.layer_speed;

        // Enhanced wind effect with layered motion
        let wind_sway = (current_time * 0.8 + particle.wind_offset).sin() * 4.0 * spawner.wind_strength;
        let wind_drift = (current_time * 0.3 + particle.wind_offset * 1.5).cos() * 2.0 * spawner.wind_strength;
        movement.x += wind_sway * delta;
        movement.y += wind_drift * delta;

        // Enhanced brownian motion for more natural movement
        use rand::Rng;
        let mut rng = rand::rng();
        let brownian_scale = match particle.particle_type {
            ParticleType::Pollen => 0.5,
            ParticleType::Firefly => 0.8,
            _ => 0.3,
        };
        movement.x += rng.random_range(-brownian_scale..brownian_scale) * delta * 40.0;
        movement.y += rng.random_range(-brownian_scale..brownian_scale) * delta * 40.0;

        // Add swirling motion for fireflies
        if particle.particle_type == ParticleType::Firefly {
            let swirl_x = (current_time * 2.0 + particle.wind_offset).sin() * 3.0;
            let swirl_y = (current_time * 1.5 + particle.wind_offset).cos() * 2.0;
            movement.x += swirl_x * delta;
            movement.y += swirl_y * delta;
        }

        // Simplified tree interaction - only check occasionally
        if (current_time * 4.0) as i32 % 10 == 0 { // Check every ~0.25 seconds
            let tile_pos = Vec2::new(
                (transform.translation.x / 32.0).round(),
                (transform.translation.y / 32.0).round(),
            );
            
            if is_near_wall_fast(tile_pos, &tile_query) {
                movement *= 0.5; // Slow down near trees/walls
            }
        }

        // Apply movement
        transform.translation.x += movement.x;
        transform.translation.y += movement.y;

        // Simplified visual effects for performance
        update_particle_visuals_optimized(&mut particle, &mut sprite, &mut transform, current_time);

        // Distance-based fading (simplified calculation)
        let distance_to_player = transform.translation.distance(player_transform.translation);
        if distance_to_player > PARTICLE_FADE_DISTANCE {
            sprite.color.set_alpha(0.0);
        } else {
            let fade_factor = (1.0 - (distance_to_player / PARTICLE_FADE_DISTANCE)).max(0.0);
            sprite.color.set_alpha(particle.original_alpha * fade_factor);
        }
    }
}

fn is_near_wall_fast(pos: Vec2, tile_query: &Query<(&TilePos, &MapTile)>) -> bool {
    // Simplified wall check - only check immediate position
    let tile_x = pos.x as u32;
    let tile_y = pos.y as u32;

    for (tile_pos, map_tile) in tile_query.iter() {
        if tile_pos.x == tile_x && tile_pos.y == tile_y {
            return map_tile.tile_type == TileType::Wall;
        }
    }
    false
}

fn update_particle_visuals_optimized(
    particle: &mut ForestParticle,
    sprite: &mut Sprite,
    transform: &mut Transform,
    current_time: f32,
) {
    match particle.particle_type {
        ParticleType::Pollen => {
            // Simplified alpha pulsing
            let age_factor = 1.0 - particle.lifetime.fraction();
            let alpha_pulse = (current_time * 1.0 + particle.color_shift).sin() * 0.2 + 0.7;
            sprite.color.set_alpha(particle.original_alpha * alpha_pulse * age_factor);

            // Simplified size variation
            let size_breath = (current_time * 1.5 + particle.size_offset).sin() * 0.05 + 1.0;
            transform.scale = Vec3::new(size_breath, size_breath, 1.0);
        },
        ParticleType::Firefly => {
            // Simplified firefly glow
            let age_factor = 1.0 - particle.lifetime.fraction();
            let glow_pulse = if particle.glow_timer.just_finished() {
                1.0
            } else {
                0.7
            };
            sprite.color.set_alpha(particle.original_alpha * glow_pulse * age_factor);

            // Minimal size variation
            transform.scale = Vec3::new(1.0, 1.0, 1.0);
        },
        ParticleType::MagicShimmer => {
            // Quick fade only
            let age_factor = 1.0 - particle.lifetime.fraction();
            sprite.color.set_alpha(particle.original_alpha * age_factor);
        },
    }
}

fn update_wind_system(
    _time: Res<Time>,
    mut spawner: ResMut<ParticleSpawner>,
) {
    if !spawner.is_active {
        return;
    }

    if spawner.wind_timer.just_finished() {
        use rand::Rng;
        let mut rng = rand::rng();
        
        // Generate new wind direction and strength with more variation
        let angle: f32 = rng.random_range(0.0..6.28);
        spawner.wind_direction = Vec2::new(angle.cos(), angle.sin());
        spawner.wind_strength = rng.random_range(0.8..2.0); // Increased wind variation
        
        // More frequent wind gusts
        if rng.random_bool(0.5) {
            spawner.wind_strength *= rng.random_range(1.5..2.5);
        }
        
        println!("Wind change: strength {:.1}, direction ({:.1}, {:.1})", 
                spawner.wind_strength, spawner.wind_direction.x, spawner.wind_direction.y);
    }
}

fn cleanup_particles(
    mut commands: Commands,
    particle_query: Query<(Entity, &ForestParticle)>,
    spawner: Res<ParticleSpawner>,
) {
    for (entity, particle) in particle_query.iter() {
        if particle.lifetime.finished() || !spawner.is_active {
            commands.entity(entity).despawn();
        }
    }
}

fn handle_particle_debug(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut settings: ResMut<ParticleSettings>,
) {
    // Toggle particles with F1
    if keyboard_input.just_pressed(KeyCode::F1) {
        settings.enabled = !settings.enabled;
        println!("Forest particles: {}", if settings.enabled { "enabled" } else { "disabled" });
    }
    
    // Adjust density with F2/F3
    if keyboard_input.just_pressed(KeyCode::F2) {
        settings.density_multiplier = (settings.density_multiplier - 0.25).max(0.25);
        println!("Particle density: {:.2}", settings.density_multiplier);
    }
    
    if keyboard_input.just_pressed(KeyCode::F3) {
        settings.density_multiplier = (settings.density_multiplier + 0.25).min(3.0);
        println!("Particle density: {:.2}", settings.density_multiplier);
    }
    
    // Toggle debug mode with F4
    if keyboard_input.just_pressed(KeyCode::F4) {
        settings.debug_mode = !settings.debug_mode;
        println!("Particle debug mode: {}", if settings.debug_mode { "enabled" } else { "disabled" });
    }
}
