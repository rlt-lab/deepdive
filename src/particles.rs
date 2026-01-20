use bevy::prelude::*;

use crate::components::{Player, CurrentLevel, TileType, BiomeParticle, ParticleType, ParticleSpawner, ParticleSettings, WindState, GlobalRng, ParticleCounts};
use crate::constants::TILE_SIZE;
use crate::biome::BiomeType;
use crate::states::GameState;
use crate::map::GameMap;

// Base particle system constants
const PARTICLE_FADE_DISTANCE: f32 = 900.0;
const SPAWN_BOUNDARY_PADDING: f32 = 10.0; // Extra tiles beyond map edges for spawning

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
    pub is_enabled: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum MovementStyle {
    Gentle,      // Slow, smooth movement (Underglade)
    Erratic,     // Sharp, unpredictable movement (Fungal Deep spores)
    Floating,    // Slow vertical drift (Caverns mist)
    Swirling,    // Circular patterns (Cinder Gaol embers)
    Flowing,     // Water-like movement (Stygian Pool)
}

impl BiomeParticleConfig {
    pub fn for_biome(biome: BiomeType) -> Self {
        match biome {
            BiomeType::Underglade => Self {
                primary_max_particles: 300,
                secondary_max_particles: 75,
                primary_spawn_rate: 4.0,
                secondary_spawn_rate: 1.0,
                secondary_spawn_chance: 0.6,
                primary_colors: vec![
                    Color::srgb(0.85, 0.88, 0.55), // Pollen yellow-green
                    Color::srgb(0.78, 0.85, 0.62), // Soft green
                    Color::srgb(0.92, 0.91, 0.67), // Light yellow
                ],
                secondary_colors: vec![
                    Color::srgb(0.78, 1.0, 0.78),  // Soft green-white fireflies
                    Color::srgb(0.71, 0.86, 1.0),  // Soft blue-white fireflies
                ],
                primary_size_range: (2.0, 3.0),
                secondary_size_range: (3.0, 4.0),
                primary_lifetime_range: (12.0, 20.0),
                secondary_lifetime_range: (10.0, 16.0),
                primary_velocity_range: (Vec2::new(-8.0, -6.0), Vec2::new(8.0, 4.0)),
                secondary_velocity_range: (Vec2::new(-5.0, -4.0), Vec2::new(5.0, 4.0)),
                wind_strength_multiplier: 1.2,
                movement_style: MovementStyle::Gentle,
                is_enabled: true,
            },
            BiomeType::FungalDeep => Self {
                primary_max_particles: 300,
                secondary_max_particles: 75,
                primary_spawn_rate: 4.0,
                secondary_spawn_rate: 1.0,
                secondary_spawn_chance: 0.4,
                primary_colors: vec![
                    Color::srgb(0.6, 0.4, 0.8),   // Purple spores
                    Color::srgb(0.5, 0.6, 0.3),   // Moldy green
                    Color::srgb(0.7, 0.5, 0.4),   // Brown spores
                ],
                secondary_colors: vec![
                    Color::srgb(0.8, 0.6, 1.0),   // Bright purple wisps
                    Color::srgb(0.4, 0.8, 0.4),   // Glowing green
                ],
                primary_size_range: (1.5, 2.5),
                secondary_size_range: (2.0, 3.5),
                primary_lifetime_range: (8.0, 15.0),
                secondary_lifetime_range: (6.0, 12.0),
                primary_velocity_range: (Vec2::new(-12.0, -8.0), Vec2::new(12.0, 6.0)),
                secondary_velocity_range: (Vec2::new(-8.0, -6.0), Vec2::new(8.0, 6.0)),
                wind_strength_multiplier: 0.8,
                movement_style: MovementStyle::Erratic,
                is_enabled: true,
            },
            BiomeType::Caverns => Self {
                primary_max_particles: 300,
                secondary_max_particles: 75,
                primary_spawn_rate: 4.0,
                secondary_spawn_rate: 1.0,
                secondary_spawn_chance: 0.3,
                primary_colors: vec![
                    Color::srgba(0.7, 0.7, 0.8, 0.6),  // Misty blue-gray
                    Color::srgba(0.6, 0.7, 0.7, 0.5),  // Cave mist
                    Color::srgba(0.8, 0.8, 0.9, 0.4),  // Light fog
                ],
                secondary_colors: vec![
                    Color::srgb(0.9, 0.95, 1.0),  // Crystal glints
                    Color::srgb(0.8, 0.9, 1.0),   // Ice blue sparkles
                ],
                primary_size_range: (3.0, 5.0),
                secondary_size_range: (1.0, 2.0),
                primary_lifetime_range: (15.0, 25.0),
                secondary_lifetime_range: (5.0, 10.0),
                primary_velocity_range: (Vec2::new(-3.0, -2.0), Vec2::new(3.0, 8.0)),
                secondary_velocity_range: (Vec2::new(-2.0, -1.0), Vec2::new(2.0, 3.0)),
                wind_strength_multiplier: 0.5,
                movement_style: MovementStyle::Floating,
                is_enabled: true,
            },
            BiomeType::CinderGaol => Self {
                primary_max_particles: 300,
                secondary_max_particles: 75,
                primary_spawn_rate: 4.0,
                secondary_spawn_rate: 1.0,
                secondary_spawn_chance: 0.7,
                primary_colors: vec![
                    Color::srgb(1.0, 0.4, 0.2),   // Bright ember orange
                    Color::srgb(1.0, 0.6, 0.1),   // Fire yellow
                    Color::srgb(0.9, 0.2, 0.1),   // Deep red ember
                ],
                secondary_colors: vec![
                    Color::srgb(0.6, 0.6, 0.7),   // Gray ash
                    Color::srgb(0.4, 0.4, 0.5),   // Dark soot
                ],
                primary_size_range: (2.5, 4.0),
                secondary_size_range: (1.5, 3.0),
                primary_lifetime_range: (6.0, 12.0),
                secondary_lifetime_range: (8.0, 15.0),
                primary_velocity_range: (Vec2::new(-6.0, 2.0), Vec2::new(6.0, 10.0)),
                secondary_velocity_range: (Vec2::new(-4.0, -2.0), Vec2::new(4.0, 2.0)),
                wind_strength_multiplier: 1.5,
                movement_style: MovementStyle::Swirling,
                is_enabled: true,
            },
            BiomeType::StygianPool => Self {
                primary_max_particles: 300,
                secondary_max_particles: 75,
                primary_spawn_rate: 4.0,
                secondary_spawn_rate: 1.0,
                secondary_spawn_chance: 0.5,
                primary_colors: vec![
                    Color::srgba(0.3, 0.6, 0.8, 0.7),  // Water droplets
                    Color::srgba(0.2, 0.5, 0.7, 0.6),  // Deep blue mist
                    Color::srgba(0.4, 0.7, 0.9, 0.5),  // Light blue vapor
                ],
                secondary_colors: vec![
                    Color::srgb(0.6, 0.8, 1.0),   // Bright water sparkles
                    Color::srgb(0.5, 0.9, 0.9),   // Cyan glimmers
                ],
                primary_size_range: (2.0, 4.0),
                secondary_size_range: (1.0, 2.5),
                primary_lifetime_range: (10.0, 18.0),
                secondary_lifetime_range: (4.0, 8.0),
                primary_velocity_range: (Vec2::new(-4.0, -3.0), Vec2::new(4.0, 3.0)),
                secondary_velocity_range: (Vec2::new(-3.0, -2.0), Vec2::new(3.0, 4.0)),
                wind_strength_multiplier: 0.7,
                movement_style: MovementStyle::Flowing,
                is_enabled: true,
            },
            // Biomes with no particle effects
            BiomeType::AbyssalHold | BiomeType::NetherGrange | 
            BiomeType::ChthronicCrypts | BiomeType::HypogealKnot => Self {
                primary_max_particles: 0,
                secondary_max_particles: 0,
                primary_spawn_rate: 0.0,
                secondary_spawn_rate: 0.0,
                secondary_spawn_chance: 0.0,
                primary_colors: vec![],
                secondary_colors: vec![],
                primary_size_range: (0.0, 0.0),
                secondary_size_range: (0.0, 0.0),
                primary_lifetime_range: (0.0, 0.0),
                secondary_lifetime_range: (0.0, 0.0),
                primary_velocity_range: (Vec2::ZERO, Vec2::ZERO),
                secondary_velocity_range: (Vec2::ZERO, Vec2::ZERO),
                wind_strength_multiplier: 0.0,
                movement_style: MovementStyle::Gentle,
                is_enabled: false,
            },
        }
    }
}

// BiomeParticle and ParticleType are now defined in components.rs

// ParticleSpawner and ParticleSettings are now defined in components.rs

impl Default for ParticleSpawner {
    fn default() -> Self {
        let config = BiomeParticleConfig::for_biome(BiomeType::Caverns);
        Self {
            primary_timer: Timer::from_seconds(1.0 / config.primary_spawn_rate.max(0.1), TimerMode::Repeating),
            secondary_timer: Timer::from_seconds(1.0 / config.secondary_spawn_rate.max(0.1), TimerMode::Repeating),
            current_biome: BiomeType::Caverns,
            config,
            has_initial_spawn_completed: false,
        }
    }
}

impl Default for WindState {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(6.0, TimerMode::Repeating),
            strength: 1.0,
            direction: Vec2::new(1.0, 0.0),
            base_multiplier: 1.0,
        }
    }
}

pub struct ParticlePlugin;

impl Plugin for ParticlePlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<ParticleSpawner>()
            .init_resource::<ParticleSettings>()
            .init_resource::<WindState>()
            .init_resource::<ParticleCounts>()
            .add_systems(Update, (
                update_particle_spawner,
                cache_particle_counts,
                spawn_biome_particles.after(cache_particle_counts),
                update_biome_particles,
                update_wind_system,
                cleanup_particles,
                handle_particle_debug
            ).run_if(in_state(GameState::Playing)));
    }
}

fn update_particle_spawner(
    time: Res<Time>,
    mut spawner: ResMut<ParticleSpawner>,
    mut wind_state: ResMut<WindState>,
    current_level: Res<CurrentLevel>,
    settings: Res<ParticleSettings>,
) {
    // Check if biome changed
    if spawner.current_biome != current_level.biome {
        spawner.current_biome = current_level.biome;
        spawner.config = BiomeParticleConfig::for_biome(current_level.biome);
        spawner.has_initial_spawn_completed = false;

        // Update timers with new spawn rates
        if spawner.config.primary_spawn_rate > 0.0 {
            spawner.primary_timer = Timer::from_seconds(1.0 / spawner.config.primary_spawn_rate, TimerMode::Repeating);
        }
        if spawner.config.secondary_spawn_rate > 0.0 {
            spawner.secondary_timer = Timer::from_seconds(1.0 / spawner.config.secondary_spawn_rate, TimerMode::Repeating);
        }

        // Update wind base multiplier for new biome
        wind_state.base_multiplier = spawner.config.wind_strength_multiplier;

        println!("Particle system updated for biome: {:?} (enabled: {})",
                current_level.biome, spawner.config.is_enabled);
    }

    if !settings.is_enabled || !spawner.config.is_enabled {
        return;
    }

    spawner.primary_timer.tick(time.delta());
    spawner.secondary_timer.tick(time.delta());
}

/// Cache particle counts in a single pass to avoid double-iteration in spawn_biome_particles.
fn cache_particle_counts(
    particles: Query<&BiomeParticle>,
    mut counts: ResMut<ParticleCounts>,
) {
    counts.reset();
    for particle in particles.iter() {
        counts.increment(particle.particle_type());
    }
}

fn spawn_biome_particles(
    mut commands: Commands,
    mut spawner: ResMut<ParticleSpawner>,
    settings: Res<ParticleSettings>,
    player_query: Query<&Transform, With<Player>>,
    particle_counts: Res<ParticleCounts>,
    map: Res<GameMap>,
    mut rng: ResMut<GlobalRng>,
) {
    if !settings.is_enabled || !spawner.config.is_enabled {
        return;
    }

    let Ok(_player_transform) = player_query.single() else {
        debug!("spawn_biome_particles: No player found, skipping particle spawn");
        return;
    };

    // Use cached particle counts (updated by cache_particle_counts system)
    let primary_count = particle_counts.primary_count;
    let secondary_count = particle_counts.secondary_count;

    // Initial spawn when entering a new biome
    if !spawner.has_initial_spawn_completed {
        let initial_primary = (spawner.config.primary_max_particles as f32 * 0.67) as usize;
        let initial_secondary = (spawner.config.secondary_max_particles as f32 * 0.67) as usize;

        // Collect primary spawn positions across entire map
        let mut primary_positions = Vec::with_capacity(initial_primary);
        for _ in 0..initial_primary {
            if let Some(spawn_pos) = find_map_spawn_position(&map, rng.as_mut()) {
                primary_positions.push(spawn_pos);
            }
        }

        // Collect secondary spawn positions across entire map
        let mut secondary_positions = Vec::with_capacity(initial_secondary);
        for _ in 0..initial_secondary {
            if let Some(spawn_pos) = find_map_spawn_position(&map, rng.as_mut()) {
                secondary_positions.push(spawn_pos);
            }
        }

        // Batch spawn all primary particles
        for spawn_pos in primary_positions {
            spawn_primary_particle(&mut commands, spawn_pos, &spawner.config, rng.as_mut());
        }

        // Batch spawn all secondary particles
        for spawn_pos in secondary_positions {
            spawn_secondary_particle(&mut commands, spawn_pos, &spawner.config, rng.as_mut());
        }

        spawner.has_initial_spawn_completed = true;
        println!("Initial particle spawn complete for {:?} biome (primary: {}, secondary: {})",
                spawner.current_biome, initial_primary, initial_secondary);
    }

    // Continuous spawning across entire map
    if spawner.primary_timer.just_finished() && primary_count < spawner.config.primary_max_particles {
        // Multiply base spawn count by density multiplier - no cap to allow high density
        let spawn_count = ((3.0 * settings.density_multiplier).max(1.0) as usize).min(30);

        // Collect spawn positions across entire map
        let mut spawn_positions = Vec::with_capacity(spawn_count);
        for _ in 0..spawn_count {
            if let Some(spawn_pos) = find_map_spawn_position(&map, rng.as_mut()) {
                spawn_positions.push(spawn_pos);
            }
        }

        // Batch spawn all particles
        for spawn_pos in spawn_positions {
            spawn_primary_particle(&mut commands, spawn_pos, &spawner.config, rng.as_mut());
        }
    }

    if spawner.secondary_timer.just_finished() && secondary_count < spawner.config.secondary_max_particles {
        use rand::Rng;
        if rng.random::<f32>() < spawner.config.secondary_spawn_chance {
            if let Some(spawn_pos) = find_map_spawn_position(&map, rng.as_mut()) {
                spawn_secondary_particle(&mut commands, spawn_pos, &spawner.config, rng.as_mut());
            }
        }
    }
}

/// Spawn particles across extended area beyond map boundaries for consistent coverage.
///
/// Uses O(1) GameMap lookups for tile type checks.
fn find_map_spawn_position(
    map: &GameMap,
    rng: &mut impl rand::Rng,
) -> Option<Vec2> {

    // Try up to 12 times to find a suitable spawn position
    for _ in 0..12 {
        // Random position with padding beyond map bounds for edge consistency
        let spawn_tile_x = rng.random_range(-SPAWN_BOUNDARY_PADDING..(map.width as f32 + SPAWN_BOUNDARY_PADDING));
        let spawn_tile_y = rng.random_range(-SPAWN_BOUNDARY_PADDING..(map.height as f32 + SPAWN_BOUNDARY_PADDING));

        // Check if within actual map bounds (for tile validation)
        let is_on_map = spawn_tile_x >= 0.0 && spawn_tile_x < map.width as f32 &&
                        spawn_tile_y >= 0.0 && spawn_tile_y < map.height as f32;

        // If on the map, check if it's a floor tile. If outside map, allow spawning
        if is_on_map {
            let spawn_tile_pos = Vec2::new(spawn_tile_x, spawn_tile_y);
            if !is_suitable_for_particles_fast(spawn_tile_pos, &map) {
                continue; // Skip walls on the map
            }
        }
        // If outside the map boundaries, we accept it (no tile check needed)

        // Convert tile coordinates to world coordinates
        let half_tile = TILE_SIZE / 2.0;
        let world_x = (spawn_tile_x - (map.width as f32 / 2.0 - 0.5)) * TILE_SIZE + rng.random_range(-half_tile..half_tile);
        let world_y = (spawn_tile_y - (map.height as f32 / 2.0 - 0.5)) * TILE_SIZE + rng.random_range(-half_tile..half_tile);
        return Some(Vec2::new(world_x, world_y));
    }

    None
}

/// Check if a position is suitable for particles using O(1) GameMap lookup.
fn is_suitable_for_particles_fast(pos: Vec2, map: &GameMap) -> bool {
    let tile_x = pos.x as u32;
    let tile_y = pos.y as u32;

    // O(1) lookup via GameMap instead of iterating ECS query
    if tile_x < map.width && tile_y < map.height {
        map.get(tile_x, tile_y) == TileType::Floor
    } else {
        true // Outside map bounds - allow particle
    }
}

fn spawn_primary_particle(commands: &mut Commands, spawn_pos: Vec2, config: &BiomeParticleConfig, rng: &mut impl rand::Rng) {

    let lifetime = rng.random_range(config.primary_lifetime_range.0..config.primary_lifetime_range.1);
    let velocity = Vec2::new(
        rng.random_range(config.primary_velocity_range.0.x..config.primary_velocity_range.1.x),
        rng.random_range(config.primary_velocity_range.0.y..config.primary_velocity_range.1.y),
    );

    let color = if !config.primary_colors.is_empty() {
        config.primary_colors[rng.random_range(0..config.primary_colors.len())]
    } else {
        Color::WHITE
    };

    let size = rng.random_range(config.primary_size_range.0..config.primary_size_range.1);

    // Generate random offsets
    let wind_offset = rng.random_range(0.0..std::f32::consts::TAU);
    let size_offset = rng.random_range(0.0..std::f32::consts::TAU);
    let color_shift = rng.random_range(0.0..std::f32::consts::TAU);

    commands.spawn((
        Sprite {
            color,
            custom_size: Some(Vec2::new(size, size)),
            ..default()
        },
        Transform::from_translation(spawn_pos.extend(1.0)),
        BiomeParticle {
            lifetime: Timer::from_seconds(lifetime, TimerMode::Once),
            velocity,
            glow_timer: Timer::from_seconds(1.0, TimerMode::Repeating),
            layer_speed: rng.random_range(0.6..1.4),
            original_alpha: color.alpha(),
            packed_data: BiomeParticle::pack(wind_offset, size_offset, color_shift, ParticleType::Primary),
        },
    ));
}

fn spawn_secondary_particle(commands: &mut Commands, spawn_pos: Vec2, config: &BiomeParticleConfig, rng: &mut impl rand::Rng) {

    let lifetime = rng.random_range(config.secondary_lifetime_range.0..config.secondary_lifetime_range.1);
    let velocity = Vec2::new(
        rng.random_range(config.secondary_velocity_range.0.x..config.secondary_velocity_range.1.x),
        rng.random_range(config.secondary_velocity_range.0.y..config.secondary_velocity_range.1.y),
    );

    let color = if !config.secondary_colors.is_empty() {
        config.secondary_colors[rng.random_range(0..config.secondary_colors.len())]
    } else {
        Color::WHITE
    };

    let size = rng.random_range(config.secondary_size_range.0..config.secondary_size_range.1);

    // Generate random offsets
    let wind_offset = rng.random_range(0.0..std::f32::consts::TAU);
    let size_offset = rng.random_range(0.0..std::f32::consts::TAU);
    let color_shift = rng.random_range(0.0..std::f32::consts::TAU);

    commands.spawn((
        Sprite {
            color,
            custom_size: Some(Vec2::new(size, size)),
            ..default()
        },
        Transform::from_translation(spawn_pos.extend(2.0)),
        BiomeParticle {
            lifetime: Timer::from_seconds(lifetime, TimerMode::Once),
            velocity,
            glow_timer: Timer::from_seconds(rng.random_range(1.5..3.0), TimerMode::Repeating),
            layer_speed: rng.random_range(0.4..1.2),
            original_alpha: color.alpha(),
            packed_data: BiomeParticle::pack(wind_offset, size_offset, color_shift, ParticleType::Secondary),
        },
    ));
}

fn update_biome_particles(
    time: Res<Time>,
    spawner: Res<ParticleSpawner>,
    wind_state: Res<WindState>,
    mut particle_query: Query<(Entity, &mut BiomeParticle, &mut Transform, &mut Sprite)>,
    player_query: Query<&Transform, (With<Player>, Without<BiomeParticle>)>,
    map: Res<GameMap>,
    mut rng: ResMut<GlobalRng>,
) {
    if !spawner.config.is_enabled {
        return;
    }

    let Ok(player_transform) = player_query.single() else {
        return;
    };

    let delta = time.delta_secs();
    let current_time = time.elapsed_secs();

    for (entity, mut particle, mut transform, mut sprite) in particle_query.iter_mut() {
        particle.lifetime.tick(time.delta());
        particle.glow_timer.tick(time.delta());

        let mut movement = particle.velocity * delta * particle.layer_speed;

        // Apply movement style based on biome
        apply_movement_style(&mut movement, &spawner.config.movement_style, &particle,
                           current_time, delta, wind_state.strength, rng.as_mut());

        // Staggered wall interaction: use entity index to distribute checks across frames
        // instead of all particles checking on the same frame
        let frame = (current_time * 4.0) as i32;
        let entity_offset = entity.index() as i32;
        if (frame + entity_offset) % 10 == 0 {
            let tile_pos = Vec2::new(
                (transform.translation.x / TILE_SIZE).round(),
                (transform.translation.y / TILE_SIZE).round(),
            );

            if is_near_wall_fast(tile_pos, &map) {
                movement *= 0.5;
            }
        }

        transform.translation.x += movement.x;
        transform.translation.y += movement.y;

        update_particle_visuals(&mut particle, &mut sprite, &mut transform, current_time);

        // Distance-based fading
        let distance_to_player = transform.translation.distance(player_transform.translation);
        if distance_to_player > PARTICLE_FADE_DISTANCE {
            sprite.color.set_alpha(0.0);
        } else {
            let fade_factor = (1.0 - (distance_to_player / PARTICLE_FADE_DISTANCE)).max(0.0);
            sprite.color.set_alpha(particle.original_alpha * fade_factor);
        }
    }
}

fn apply_movement_style(
    movement: &mut Vec2,
    style: &MovementStyle,
    particle: &BiomeParticle,
    current_time: f32,
    delta: f32,
    wind_strength: f32,
    rng: &mut impl rand::Rng,
) {

    // Unpack wind_offset once
    let wind_offset = particle.wind_offset();

    match style {
        MovementStyle::Gentle => {
            // Smooth, soft movement like Underglade
            let wind_sway = (current_time * 0.8 + wind_offset).sin() * 4.0 * wind_strength;
            let wind_drift = (current_time * 0.3 + wind_offset * 1.5).cos() * 2.0 * wind_strength;
            movement.x += wind_sway * delta;
            movement.y += wind_drift * delta;

            let brownian_scale = 0.5;
            movement.x += rng.random_range(-brownian_scale..brownian_scale) * delta * 40.0;
            movement.y += rng.random_range(-brownian_scale..brownian_scale) * delta * 40.0;
        },
        MovementStyle::Erratic => {
            // Sharp, unpredictable movement like spores
            let jitter_scale = 1.5;
            movement.x += rng.random_range(-jitter_scale..jitter_scale) * delta * 60.0;
            movement.y += rng.random_range(-jitter_scale..jitter_scale) * delta * 60.0;

            // Random direction changes
            if rng.random_bool(0.1) {
                movement.x *= -0.5;
                movement.y *= -0.5;
            }
        },
        MovementStyle::Floating => {
            // Slow upward drift like mist
            movement.y += delta * 15.0; // Upward bias
            let gentle_sway = (current_time * 0.5 + wind_offset).sin() * 2.0 * wind_strength;
            movement.x += gentle_sway * delta;
        },
        MovementStyle::Swirling => {
            // Circular, swirling patterns like embers
            let swirl_x = (current_time * 3.0 + wind_offset).sin() * 5.0;
            let swirl_y = (current_time * 2.0 + wind_offset).cos() * 4.0;
            movement.x += swirl_x * delta;
            movement.y += swirl_y * delta;

            // Add heat rise for embers
            movement.y += delta * 8.0;
        },
        MovementStyle::Flowing => {
            // Water-like flowing movement
            let flow_x = (current_time * 1.2 + wind_offset).sin() * 3.0 * wind_strength;
            let flow_y = (current_time * 0.8 + wind_offset * 1.2).cos() * 2.0 * wind_strength;
            movement.x += flow_x * delta;
            movement.y += flow_y * delta;

            // Ripple effect
            let ripple = (current_time * 4.0 + wind_offset * 2.0).sin() * 1.0;
            movement.x += ripple * delta;
        },
    }
}

/// Check if a position is near a wall using O(1) GameMap lookup.
fn is_near_wall_fast(pos: Vec2, map: &GameMap) -> bool {
    let tile_x = pos.x as u32;
    let tile_y = pos.y as u32;

    // O(1) lookup via GameMap instead of iterating ECS query
    if tile_x < map.width && tile_y < map.height {
        map.get(tile_x, tile_y) == TileType::Wall
    } else {
        false // Outside map bounds - no wall
    }
}

fn update_particle_visuals(
    particle: &mut BiomeParticle,
    sprite: &mut Sprite,
    transform: &mut Transform,
    current_time: f32,
) {
    match particle.particle_type() {
        ParticleType::Primary => {
            let age_factor = 1.0 - particle.lifetime.fraction();
            let color_shift = particle.color_shift();
            let alpha_pulse = (current_time * 1.0 + color_shift).sin() * 0.2 + 0.7;
            sprite.color.set_alpha(particle.original_alpha * alpha_pulse * age_factor);

            let size_offset = particle.size_offset();
            let size_breath = (current_time * 1.5 + size_offset).sin() * 0.05 + 1.0;
            transform.scale = Vec3::new(size_breath, size_breath, 1.0);
        },
        ParticleType::Secondary => {
            let age_factor = 1.0 - particle.lifetime.fraction();
            let glow_pulse = if particle.glow_timer.just_finished() { 1.0 } else { 0.7 };
            sprite.color.set_alpha(particle.original_alpha * glow_pulse * age_factor);

            transform.scale = Vec3::new(1.0, 1.0, 1.0);
        },
    }
}

fn update_wind_system(
    time: Res<Time>,
    mut wind_state: ResMut<WindState>,
    spawner: Res<ParticleSpawner>,
    mut rng: ResMut<GlobalRng>,
) {
    if !spawner.config.is_enabled {
        return;
    }

    wind_state.timer.tick(time.delta());

    if wind_state.timer.just_finished() {
        use rand::Rng;

        let angle: f32 = rng.random_range(0.0..6.28);
        wind_state.direction = Vec2::new(angle.cos(), angle.sin());
        wind_state.strength = rng.random_range(0.8..2.0) * wind_state.base_multiplier;

        if rng.random_bool(0.3) {
            wind_state.strength *= rng.random_range(1.5..2.5);
        }

        println!("Wind change in {:?}: strength {:.1}, direction ({:.1}, {:.1})",
                spawner.current_biome, wind_state.strength,
                wind_state.direction.x, wind_state.direction.y);
    }
}

fn cleanup_particles(
    mut commands: Commands,
    particle_query: Query<(Entity, &BiomeParticle)>,
    spawner: Res<ParticleSpawner>,
) {
    for (entity, particle) in particle_query.iter() {
        if particle.lifetime.finished() || !spawner.config.is_enabled {
            commands.entity(entity).despawn();
        }
    }
}

fn handle_particle_debug(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut settings: ResMut<ParticleSettings>,
    spawner: Res<ParticleSpawner>,
) {
    if keyboard_input.just_pressed(KeyCode::F1) {
        settings.is_enabled = !settings.is_enabled;
        println!("Biome particles: {}", if settings.is_enabled { "enabled" } else { "disabled" });
    }
    
    if keyboard_input.just_pressed(KeyCode::F2) {
        settings.density_multiplier = (settings.density_multiplier - 0.25).max(0.25);
        println!("Particle density: {:.2}", settings.density_multiplier);
    }
    
    if keyboard_input.just_pressed(KeyCode::F3) {
        settings.density_multiplier = (settings.density_multiplier + 0.25).min(3.0);
        println!("Particle density: {:.2}", settings.density_multiplier);
    }
    
    if keyboard_input.just_pressed(KeyCode::F4) {
        settings.is_debug_mode = !settings.is_debug_mode;
        println!("Particle debug mode: {} | Current biome: {:?} | Enabled: {}", 
                if settings.is_debug_mode { "enabled" } else { "disabled" },
                spawner.current_biome, spawner.config.is_enabled);
    }
}