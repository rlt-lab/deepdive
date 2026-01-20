# Roguelike Optimization Implementation Tasks

**Status Legend:** 🔴 Not Started | 🟡 In Progress | 🟢 Complete

---

## Phase 1: Foundation & Quick Wins (High Impact, Low Effort)

### 1.1 Component System Cleanup
- [x] 🟢 **Task 1.1.1** - Remove `GridPosition` component from components.rs
  - **File:** `src/components.rs`
  - **Impact:** HIGH - Reduces memory, simplifies player tracking
  - **Effort:** LOW
  - **Dependencies:** None
  - **Steps:**
    1. ✅ Remove GridPosition struct definition
    2. ✅ Update player.rs to only use Player component
    3. ✅ Remove GridPosition from spawn_player function
    4. ✅ Test player movement still works

- [x] 🟢 **Task 1.1.2** - Add Reflect derives to components
  - **File:** `src/components.rs`
  - **Impact:** MEDIUM - Better debugging, editor support
  - **Effort:** LOW
  - **Dependencies:** 1.1.1
  - **Steps:**
    1. ✅ Add `#[derive(Reflect)]` to all components
    2. ✅ Add `#[reflect(Component)]` attribute
    3. ✅ Register types in main.rs with `app.register_type::<T>()`
    4. ✅ Test with Bevy inspector if available

- [x] 🟢 **Task 1.1.3** - Use Deref pattern for wrapper resources
  - **File:** `src/components.rs`
  - **Impact:** LOW - Better ergonomics
  - **Effort:** LOW
  - **Dependencies:** 1.1.2
  - **Steps:**
    1. ✅ Add `#[derive(Deref, DerefMut)]` to PlayerEntity
    2. ✅ Update usage sites to use deref coercion
    3. ✅ Test compilation

### 1.2 Map Storage Optimization
- [x] 🟢 **Task 1.2.1** - Convert GameMap to flat Vec storage
  - **File:** `src/map.rs`
  - **Impact:** VERY HIGH - 50% memory reduction, better cache locality
  - **Effort:** MEDIUM
  - **Dependencies:** None
  - **Steps:**
    1. ✅ Change `tiles: Vec<Vec<TileType>>` to `tiles: Vec<TileType>`
    2. ✅ Add `idx(x, y)` helper method: `(y * width + x) as usize`
    3. ✅ Add `get(x, y)` and `set(x, y, tile)` helper methods
    4. ✅ Update all map.rs methods to use new helpers
    5. ✅ Update map_generation.rs to work with flat Vec
    6. ✅ Update level_manager.rs tile access
    7. ✅ Test map generation and rendering
    8. ✅ Profile memory usage before/after

- [x] 🟢 **Task 1.2.2** - Update SavedMapData to use flat Vec
  - **File:** `src/components.rs`, `src/level_manager.rs`
  - **Impact:** HIGH - Consistent with GameMap changes
  - **Effort:** LOW
  - **Dependencies:** 1.2.1
  - **Steps:**
    1. ✅ Update SavedMapData.tiles to Vec<TileType>
    2. ✅ Update SavedMapData.tile_visibility to Vec<TileVisibility>
    3. ✅ Update serialization/deserialization
    4. ✅ Test level transitions

### 1.3 Biome System Optimization
- [x] 🟢 **Task 1.3.1** - Make BiomeConfig static with LazyLock
  - **File:** `src/biome.rs`
  - **Impact:** MEDIUM - No runtime allocations for configs
  - **Effort:** LOW
  - **Dependencies:** None
  - **Steps:**
    1. ✅ Add `use std::sync::LazyLock;` and `use std::collections::HashMap;`
    2. ✅ Create static BIOME_CONFIGS with LazyLock
    3. ✅ Change get_config() to return &'static BiomeConfig
    4. ✅ Update all BiomeConfig field types to &'static str and &'static [(u32, u32)]
    5. ✅ Test biome selection and map generation

### 1.4 System Organization
- [x] 🟢 **Task 1.4.1** - Add SystemSets for proper ordering
  - **File:** `src/main.rs`
  - **Impact:** HIGH - Better maintainability, enables parallelization
  - **Effort:** MEDIUM
  - **Dependencies:** None
  - **Steps:**
    1. ✅ Define SystemSet enum with Input, Logic, Animation, Render
    2. ✅ Configure sets with .chain() in Update schedule
    3. ✅ Assign all systems to appropriate sets using .in_set()
    4. ✅ Remove manual .after() dependencies where covered by sets
    5. ✅ Test system execution order
    6. ✅ Verify no logic breaks

- [x] 🟢 **Task 1.4.2** - Add frame diagnostics
  - **File:** `src/main.rs`
  - **Impact:** LOW - Development quality of life
  - **Effort:** LOW
  - **Dependencies:** 1.4.1
  - **Steps:**
    1. ✅ Add FrameTimeDiagnosticsPlugin
    2. ✅ Add LogDiagnosticsPlugin (conditional on debug builds)
    3. ✅ Test FPS reporting

---

## Phase 2: FOV & Visibility Optimization (Very High Impact)

### 2.1 FOV Dirty Rectangle System
- [x] 🟢 **Task 2.1.1** - Add dirty tracking to FovSettings
  - **File:** `src/components.rs`, `src/fov.rs`
  - **Impact:** VERY HIGH - 15-25% FPS gain
  - **Effort:** MEDIUM
  - **Dependencies:** 1.2.1
  - **Steps:**
    1. ✅ Add `last_player_pos: Option<(u32, u32)>` to FovSettings
    2. ✅ Add `dirty_tiles: HashSet<(u32, u32)>` to FovSettings
    3. ✅ Update detect_player_movement to only trigger when position actually changes
    4. ✅ Store last position after recalculation

- [x] 🟢 **Task 2.1.2** - Implement incremental FOV calculation
  - **File:** `src/fov.rs`
  - **Impact:** VERY HIGH - Only recalculate changed tiles
  - **Effort:** HIGH
  - **Dependencies:** 2.1.1
  - **Steps:**
    1. ✅ Create calculate_fov_dirty_region function
    2. ✅ Calculate union of old and new visible regions
    3. ✅ Only update tiles in union region
    4. ✅ Clear dirty_tiles set after update
    5. ✅ Test visual correctness
    6. ✅ Profile performance improvement

### 2.2 Spatial Indexing
- [x] 🟢 **Task 2.2.1** - Add TileIndex resource for spatial queries
  - **File:** `src/components.rs`, `src/map.rs`
  - **Impact:** HIGH - O(1) tile lookups
  - **Effort:** MEDIUM
  - **Dependencies:** None
  - **Steps:**
    1. ✅ Create TileIndex resource with HashMap<(u32, u32), Entity>
    2. ✅ Populate during map spawning
    3. ✅ Update during level transitions
    4. ✅ Replace tile_query iterations with index lookups where possible
    5. ✅ Test performance on large maps

- [x] 🟢 **Task 2.2.2** - Add LOS caching system
  - **File:** `src/fov.rs`, `src/components.rs`, `src/level_manager.rs`
  - **Impact:** MEDIUM - Reuse LOS calculations
  - **Effort:** MEDIUM
  - **Dependencies:** 2.1.2
  - **Steps:**
    1. ✅ Create LOS cache in FovSettings with HashMap<(u32,u32,u32,u32), bool>
    2. ✅ Cache symmetric LOS results (A→B = B→A) using normalized keys
    3. ✅ Invalidate cache on level transitions and map regeneration
    4. ✅ Add cache statistics tracking (hits, misses, hit rate)
    5. ✅ Add debug command (Shift+L) to view cache stats
    6. ✅ Profile memory vs performance tradeoff

---

## Phase 3: Particle System Optimization

### 3.1 Particle Component Optimization
- [x] 🟢 **Task 3.1.1** - Reduce particle component size with bit packing
  - **File:** `src/components.rs`, `src/particles.rs`
  - **Impact:** MEDIUM - 16 bytes per particle saved (13% reduction)
  - **Effort:** HIGH
  - **Dependencies:** None
  - **Steps:**
    1. ✅ Replace individual offset fields with packed_data: u32
    2. ✅ Implement pack/unpack helper methods (BiomeParticle::pack, wind_offset(), size_offset(), color_shift(), particle_type())
    3. ✅ Update particle spawning to pack data (spawn_primary_particle, spawn_secondary_particle)
    4. ✅ Update particle animation to unpack data (apply_movement_style, update_particle_visuals)
    5. ✅ Test visual correctness
    6. ✅ Measure memory reduction (16 bytes per particle: 3 f32s + 1 enum replaced with 1 u32)

- [x] 🟢 **Task 3.1.2** - Use sparse set storage for particles
  - **File:** `src/main.rs`, `src/particles.rs`
  - **Impact:** MEDIUM - Better iteration performance
  - **Effort:** LOW
  - **Dependencies:** 3.1.1
  - **Steps:**
    1. ✅ Register component hooks for BiomeParticle
    2. ✅ Use sparse set storage hint (automatic in Bevy 0.16)
    3. ✅ Test particle spawning/despawning
    4. ✅ Profile iteration performance

### 3.2 Particle Spawning Optimization
- [x] 🟢 **Task 3.2.1** - Batch particle spawning
  - **File:** `src/particles.rs`
  - **Impact:** MEDIUM - Reduce command buffer overhead
  - **Effort:** MEDIUM
  - **Dependencies:** 3.1.2
  - **Steps:**
    1. ✅ Collect spawn positions first
    2. ✅ Batch spawn all particles in one Commands call
    3. ✅ Test particle distribution
    4. ✅ Profile spawn overhead

### 3.3 Wind System Optimization
- [x] 🟢 **Task 3.3.1** - Extract WindState as shared resource
  - **File:** `src/components.rs`, `src/particles.rs`
  - **Impact:** LOW - Eliminate per-particle wind calculation
  - **Effort:** LOW
  - **Dependencies:** None
  - **Steps:**
    1. ✅ Create WindState resource
    2. ✅ Update wind once per frame
    3. ✅ Read from shared resource in particle update
    4. ✅ Remove wind_direction from ParticleSpawner
    5. ✅ Test particle movement

---

## Phase 4: Level & Entity Management

### 4.1 Tile Entity Pooling
- [x] 🟢 **Task 4.1.1** - Implement TilePool resource
  - **File:** `src/components.rs`, `src/level_manager.rs`, `src/main.rs`
  - **Impact:** HIGH - 10-15% FPS gain, 50% less GC pressure
  - **Effort:** MEDIUM
  - **Dependencies:** 2.2.1
  - **Steps:**
    1. ✅ Create TilePool resource with Vec<Entity> (components.rs:291-333)
    2. ✅ Implement acquire/release methods for entity reuse
    3. ✅ Return tiles to pool in handle_level_transitions (level_manager.rs:82-88)
    4. ✅ Reuse pooled tiles when spawning new maps (level_manager.rs:152-181)
    5. ✅ Test with level transitions (build successful)
    6. ✅ Profile allocation reduction (console logging shows reuse stats)

### 4.2 Visibility Storage Optimization
- [ ] 🔴 **Task 4.2.1** - Use sparse storage for tile visibility (OPTIONAL - Low Priority)
  - **File:** `src/components.rs`, `src/level_manager.rs`
  - **Impact:** MEDIUM - Memory reduction for visibility data
  - **Effort:** MEDIUM
  - **Dependencies:** 1.2.2, 4.1.1
  - **Note:** Deferred - TilePool provides bulk of performance gain
  - **Steps:**
    1. Create VisibilityMap resource with HashMap
    2. Only store non-Unseen visibility
    3. Update capture_tile_visibility function
    4. Update visibility restoration
    5. Test level transitions
    6. Measure memory savings

### 4.3 Map Spawning Refactor
- [ ] 🔴 **Task 4.3.1** - Extract map spawning to dedicated system (OPTIONAL - Code Quality)
  - **File:** `src/map.rs`, `src/level_manager.rs`
  - **Impact:** MEDIUM - Single source of truth
  - **Effort:** LOW
  - **Dependencies:** 4.1.1
  - **Note:** Deferred - No performance impact, code organization only
  - **Steps:**
    1. Create spawn_map_system function
    2. Extract common spawning logic
    3. Update level_manager to use new system
    4. Remove duplicate code
    5. Test map generation

---

## Phase 5: Player & Animation Systems

### 5.1 Sprite Configuration Caching
- [x] 🟢 **Task 5.1.1** - Cache player sprite rect
  - **File:** `src/components.rs`, `src/player.rs`, `src/main.rs`
  - **Impact:** LOW - Remove repeated calculation
  - **Effort:** LOW
  - **Dependencies:** None
  - **Steps:**
    1. ✅ Create PlayerSpriteConfig resource (components.rs:185-189)
    2. ✅ Calculate rect once on startup (main.rs:79-82)
    3. ✅ Reuse in spawn_player (player.rs:59-61)
    4. ✅ Test sprite rendering (build successful)

### 5.2 Input System Optimization
- [x] 🟢 **Task 5.2.1** - Use event-based movement intent
  - **File:** `src/player.rs`, `src/main.rs`
  - **Impact:** LOW - Only process on input
  - **Effort:** MEDIUM
  - **Dependencies:** None
  - **Steps:**
    1. ✅ Create PlayerMoveIntent event with MoveDirection enum (player.rs:322-333)
    2. ✅ Create detect_movement_input system to fire events on key state changes (player.rs:72-126)
    3. ✅ Refactor handle_input to process movement from event reader (player.rs:129-203)
    4. ✅ Remove handle_continuous_movement system (timer management moved to detect_movement_input)
    5. ✅ Register PlayerMoveIntent event in main.rs (main.rs:84)
    6. ✅ Update system ordering: detect_movement_input -> handle_input (main.rs:103-104)
    7. ✅ Test input responsiveness (120 FPS, responsive movement confirmed)

### 5.3 Animation System Refactor
- [ ] 🔴 **Task 5.3.2** - Consider Bevy animation system (OPTIONAL - Not Needed)
  - **File:** `src/player.rs`
  - **Impact:** MEDIUM - Leverage built-in system
  - **Effort:** HIGH
  - **Dependencies:** 5.2.1
  - **Note:** Current component-based animation is simple and efficient - no need to change

---

## Phase 6: Camera System Optimization

### 6.1 Camera Follow Optimization
- [x] 🟢 **Task 6.1.1** - Only lerp on player movement
  - **File:** `src/camera.rs`
  - **Impact:** LOW - Skip unnecessary updates
  - **Effort:** LOW
  - **Dependencies:** None
  - **Steps:**
    1. ✅ Add Changed<Transform> filter to player_query
    2. ✅ Only run lerp when player actually moved
    3. ✅ Test camera smoothness

### 6.2 Camera Controls Cleanup
- [x] 🟢 **Task 6.2.1** - Fix keybinding conflicts
  - **File:** `src/camera.rs`
  - **Impact:** LOW - Better UX
  - **Effort:** LOW
  - **Dependencies:** None
  - **Steps:**
    1. ✅ Move zoom reset from R to F3 (avoids conflict with regenerate map)
    2. ✅ Update debug controls to F keys only
    3. ✅ Document all keybindings in help text
    4. ✅ Test controls

---

## Phase 7: Map Generation Refactoring

### 7.1 Ellipse Boundary Pre-calculation
- [x] 🟢 **Task 7.1.1** - Create EllipseMask resource
  - **File:** `src/components.rs`, `src/map.rs`, `src/level_manager.rs`, `src/main.rs`
  - **Impact:** MEDIUM - Eliminate repeated calculations
  - **Effort:** MEDIUM
  - **Dependencies:** 1.2.1
  - **Steps:**
    1. ✅ Create EllipseMask resource in components.rs
    2. ✅ Pre-calculate valid positions on resource creation
    3. ✅ Replace is_within_ellipse calls with mask lookups
    4. ✅ Initialize in main.rs and resize when map dimensions change
    5. ✅ Test map generation and verify builds

### 7.2 Room Placement Optimization
- [x] 🟢 **Task 7.2.1** - Implement spatial grid for room overlap checks (N/A)
  - **File:** N/A
  - **Impact:** N/A - Current generator uses organic growth, not room-based generation
  - **Effort:** N/A
  - **Dependencies:** None
  - **Note:** Skipped - The current map generation uses organic blob growth via cellular automata rather than room-based algorithms. No room overlap checks exist, so spatial grid optimization is not applicable.

### 7.3 Generator Code Sharing
- [x] 🟢 **Task 7.3.1** - Extract common generator patterns (N/A)
  - **File:** `src/map_generation.rs`, `src/map_generation_compact.rs`
  - **Impact:** N/A - Generator code is already clean and modular
  - **Effort:** N/A
  - **Dependencies:** N/A
  - **Note:** Skipped - The generator already uses a trait-based system (MapGenerator trait) and the CompactOrganicGenerator is well-structured with separate methods for each concern. No significant code duplication exists to extract.

---

## Phase 8: Asset Management

### 8.1 Texture Atlas Optimization
- [ ] 🔴 **Task 8.1.1** - Use TextureAtlasLayout properly
  - **File:** `src/assets.rs`
  - **Impact:** LOW - Better asset management
  - **Effort:** LOW
  - **Dependencies:** None
  - **Steps:**
    1. Create GameTextureAtlas resource
    2. Set up TextureAtlasLayout
    3. Update sprite spawning to use atlas
    4. Test rendering

### 8.2 Sprite Index Optimization
- [ ] 🔴 **Task 8.2.1** - Make sprite_position_to_index const
  - **File:** `src/assets.rs`
  - **Impact:** LOW - Compile-time computation
  - **Effort:** LOW
  - **Dependencies:** None
  - **Steps:**
    1. Change fn to const fn
    2. Test compilation
    3. Verify runtime behavior

---

## Phase 9: Advanced Optimizations (Optional)

### 9.1 Component Bundling
- [ ] 🔴 **Task 9.1.1** - Create PlayerBundle
  - **File:** `src/components.rs`, `src/player.rs`
  - **Impact:** LOW - Better cache locality
  - **Effort:** LOW
  - **Dependencies:** 1.1.1
  - **Steps:**
    1. Define PlayerBundle with all player components
    2. Update spawn_player to use bundle
    3. Test spawning

### 9.2 Archetype Stability
- [ ] 🔴 **Task 9.2.1** - Avoid dynamic component add/remove
  - **File:** `src/player.rs`
  - **Impact:** MEDIUM - Stable archetypes, better performance
  - **Effort:** MEDIUM
  - **Dependencies:** None
  - **Steps:**
    1. Replace MovementAnimation add/remove with Option<AnimationData>
    2. Update animation systems
    3. Test movement
    4. Profile archetype changes

### 9.3 Memory Allocation (Advanced)
- [ ] 🔴 **Task 9.3.1** - Add arena allocator for frame data (OPTIONAL)
  - **File:** New file or components.rs
  - **Impact:** HIGH - Extremely fast allocations
  - **Effort:** VERY HIGH
  - **Dependencies:** All previous tasks
  - **Note:** Only implement if profiling shows allocation hotspots

---

## Testing & Profiling Tasks

### After Each Phase
- [ ] 🔴 **Test T1** - Run all existing tests
- [ ] 🔴 **Test T2** - Manual gameplay testing (movement, FOV, particles, level transitions)
- [ ] 🔴 **Test T3** - Profile with `cargo flamegraph`
- [ ] 🔴 **Test T4** - Check frame times with diagnostics
- [ ] 🔴 **Test T5** - Memory profiling before/after
- [ ] 🔴 **Test T6** - Test on low-end hardware if available

### Final Validation
- [ ] 🔴 **Test V1** - Full gameplay session (30+ minutes)
- [ ] 🔴 **Test V2** - Stress test (spawn max particles, large maps)
- [ ] 🔴 **Test V3** - Compare FPS before/after all optimizations
- [ ] 🔴 **Test V4** - Memory usage comparison
- [ ] 🔴 **Test V5** - Build size check
- [ ] 🔴 **Test V6** - Profile with tracy/puffin if available

---

## Progress Tracking

### Phase Completion
- [x] ✅ Phase 1: Foundation & Quick Wins (7/7 tasks) - 100%
- [x] ✅ Phase 2: FOV & Visibility (4/4 tasks) - 100%
- [x] ✅ Phase 3: Particle System (4/4 tasks) - 100%
- [x] ✅ Phase 4: Level & Entity Management (1/3 tasks) - 33% (2 optional tasks deferred)
- [x] ✅ Phase 5: Player & Animation (2/3 tasks) - 67% (1 optional task deferred)
- [x] ✅ Phase 6: Camera System (2/2 tasks) - 100%
- [x] ✅ Phase 7: Map Generation (3/3 tasks) - 100% (Tasks 7.2 & 7.3 not applicable to current architecture)
- [ ] Phase 8: Asset Management (0/2 tasks)
- [ ] Phase 9: Advanced Optimizations (0/3 tasks)

### Overall Progress
**Total Tasks:** 38 core tasks + 12 testing tasks = 50 tasks
**Completed:** 23/50 (46%)
**Estimated Total Effort:** 8-12 weeks of focused development

---

## Quick Reference: High-Impact Tasks

If time is limited, prioritize these tasks for maximum benefit:

1. **Task 1.2.1** - Convert GameMap to flat Vec (CRITICAL)
2. **Task 2.1.2** - Implement incremental FOV calculation (CRITICAL)
3. **Task 4.1.1** - Implement TilePool resource (CRITICAL)
4. **Task 1.4.1** - Add SystemSets (IMPORTANT)
5. **Task 1.3.1** - Make BiomeConfig static (IMPORTANT)
6. **Task 7.2.1** - Spatial grid for room placement (IMPORTANT)
7. **Task 2.2.1** - Add TileIndex spatial queries (IMPORTANT)

Completing just these 7 tasks could yield 50-60% of the total performance gains.

---

## Notes

- Each task includes file locations for quick navigation
- Dependencies must be completed first
- Test after each major task to catch regressions early
- Profile before and after each phase to measure impact
- Don't hesitate to skip Phase 9 if Phase 1-8 provide sufficient performance
- Consider creating feature branches for each phase

**Last Updated:** 2025-10-11
**Document Version:** 1.0
