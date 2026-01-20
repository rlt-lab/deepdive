# Code Review Task List (TDD Reorganized)

**Project:** Deepdive - Turn-based Roguelike
**Reorganized:** 2026-01-19
**Methodology:** Test-Driven Development (Red → Green → Refactor)
**Source:** docs/CODE_REVIEW_REPORT.md

---

## TDD Workflow

For each phase:
1. **RED** - Write failing tests first
2. **GREEN** - Implement minimal code to pass tests
3. **REFACTOR** - Clean up while tests stay green

**Verification after each task:** `cargo build && cargo test`

---

## Phase 1: Test Infrastructure & Constants

**Goal:** Establish foundation for all subsequent TDD work.

### 1.1 Test Framework Setup (P0)

| # | Task | Status |
|---|------|--------|
| 1.1.1 | Create `tests/` directory at project root | [x] |
| 1.1.2 | Create `tests/common/mod.rs` with test utilities (test map builders, assertions) | [x] |
| 1.1.3 | Add `[dev-dependencies]` to Cargo.toml: `proptest = "1.0"` | [x] |
| 1.1.4 | Verify test infrastructure: `cargo test` runs successfully | [x] |

### 1.2 Extract Constants (P1)

**Why first:** Tests need constants for assertions (MAP_WIDTH, TILE_SIZE, etc.)

| # | File | Task | Status |
|---|------|------|--------|
| 1.2.1 | Create `src/constants.rs` module | [x] |
| 1.2.2 | Define map constants: `MAP_WIDTH: u32 = 80`, `MAP_HEIGHT: u32 = 50`, `MAX_TILE_POOL: usize = 4000` | [x] |
| 1.2.3 | Define tile constants: `TILE_SIZE: f32 = 32.0` | [x] |
| 1.2.4 | Define window constants: `WINDOW_WIDTH: f32 = 1400.0`, `WINDOW_HEIGHT: f32 = 800.0` | [x] |
| 1.2.5 | Define timer constants: `HOLD_TO_MOVE_TIMER: f32 = 0.15`, `AUTOEXPLORE_ANIM_TIMER: f32 = 0.05`, `HOP_ANIM_TIMER: f32 = 0.1` | [x] |
| 1.2.6 | Define camera constants: `CAMERA_PADDING: f32 = 200.0`, `ZOOM_SPEED: f32 = 1.2`, `ZOOM_MIN: f32 = 0.5`, `ZOOM_MAX: f32 = 3.0` | [x] |
| 1.2.7 | Define FOV constants: `FOV_RADIUS: u32 = 20` | [x] |
| 1.2.8 | Add `mod constants;` to main.rs, replace usages across codebase | [x] |

---

## Phase 2: Map Core (TDD)

**Dependency:** Foundation for pathfinding, FOV, particles. Test this first.

### 2.1 Write Map Unit Tests (P0) — RED

| # | Test Category | Tests to Write | Status |
|---|---------------|----------------|--------|
| 2.1.1 | Index calculation | `idx()` - valid coords, edge cases (0,0), boundary (79,49), out of bounds | [ ] |
| 2.1.2 | Tile operations | `get()`, `set()` - round-trip correctness for all TileType variants | [ ] |
| 2.1.3 | Walkability | `is_walkable()` - Floor=true, Wall=false, Water=false, stairs=true | [ ] |
| 2.1.4 | Floor positions | `get_floor_positions()` - returns all and only floor tiles | [ ] |
| 2.1.5 | Grid-to-world | Test `grid_to_world()` conversion (will fail until 2.2.1) | [ ] |
| 2.1.6 | Find nearby floor | Test `find_nearby_floor()` radial search (will fail until 2.2.4) | [ ] |

### 2.2 Implement Map Helpers — GREEN

| # | File | Task | Status |
|---|------|------|--------|
| 2.2.1 | map.rs | Add `impl GameMap { pub fn grid_to_world(&self, x: u32, y: u32) -> Vec2 }` | [ ] |
| 2.2.2 | player.rs | Replace inline grid-to-world at lines 50-51, 85-86, 159-162 | [ ] |
| 2.2.3 | input_handler.rs | Replace inline grid-to-world at lines 222-225, 521-524 | [ ] |
| 2.2.4 | map.rs | Add `pub fn find_nearby_floor(&self, center_x: u32, center_y: u32, max_radius: u32) -> Option<(u32, u32)>` | [ ] |
| 2.2.5 | player.rs | Replace spawn search (lines 26-44) with `map.find_nearby_floor()` | [ ] |
| 2.2.6 | level_manager.rs | Replace spawn search (lines 288-308) with `map.find_nearby_floor()` | [ ] |

### 2.3 Map Cleanup — REFACTOR

| # | File | Task | Status |
|---|------|------|--------|
| 2.3.1 | map.rs | Delete unused `get_tile_texture_index()` function (lines 291-313) | [ ] |
| 2.3.2 | map.rs | Add `.expect()` message to line 96-98 unwrap | [ ] |
| 2.3.3 | map.rs | Add `warn!()` before early return in `place_stairs()` line 208 | [ ] |

---

## Phase 3: Pathfinding (TDD)

**Dependency:** Requires working map. Used by autoexplore, stair navigation.

### 3.1 Write Pathfinding Tests (P0) — RED

| # | Test Category | Tests to Write | Status |
|---|---------------|----------------|--------|
| 3.1.1 | Direct path | Path on empty map, straight line | [ ] |
| 3.1.2 | Path around obstacles | Wall in the way, finds alternate route | [ ] |
| 3.1.3 | No path | Completely blocked, returns empty Vec | [ ] |
| 3.1.4 | Edge cases | Start=goal returns empty, adjacent tiles returns single step | [ ] |
| 3.1.5 | Path validity | All steps are walkable, each step adjacent to previous | [ ] |
| 3.1.6 | VecDeque operations | Test `front()`, `pop_front()` on path (will fail until 3.2) | [ ] |

### 3.2 Implement VecDeque Optimization (P0) — GREEN

| # | File | Task | Status |
|---|------|------|--------|
| 3.2.1 | components.rs | Add `use std::collections::VecDeque;` | [ ] |
| 3.2.2 | components.rs | Change `Autoexplore.path` from `Vec<(u32, u32)>` to `VecDeque<(u32, u32)>` | [ ] |
| 3.2.3 | components.rs | Update `Autoexplore::default()` to use `VecDeque::new()` | [ ] |
| 3.2.4 | components.rs | Change `AutoMoveToStair.path` from `Vec` to `VecDeque` | [ ] |
| 3.2.5 | player.rs | Add `use std::collections::VecDeque;` | [ ] |
| 3.2.6 | player.rs | Replace `path.first().copied()` with `path.front().copied()` (line 155) | [ ] |
| 3.2.7 | player.rs | Replace `path.remove(0)` with `path.pop_front()` (line 183) | [ ] |
| 3.2.8 | player.rs | Convert `find_path()` result to VecDeque at assignment | [ ] |
| 3.2.9 | input_handler.rs | Add VecDeque import, update `path.first()` → `path.front()` (line 517) | [ ] |
| 3.2.10 | input_handler.rs | Replace `path.remove(0)` with `path.pop_front()` (line 545) | [ ] |

### 3.3 Pathfinding Cleanup — REFACTOR

| # | File | Task | Status |
|---|------|------|--------|
| 3.3.1 | player.rs | Replace `.unwrap()` with `.expect("came_from missing predecessor")` (line 291) | [ ] |

---

## Phase 4: FOV System (TDD)

**Dependency:** Requires map. Affects visibility, exploration tracking.

### 4.1 Write FOV Tests (P1) — RED

| # | Test Category | Tests to Write | Status |
|---|---------------|----------------|--------|
| 4.1.1 | Line of sight | Clear LOS on empty map | [ ] |
| 4.1.2 | LOS blocking | Wall blocks LOS | [ ] |
| 4.1.3 | LOS symmetry | If A sees B, B sees A | [ ] |
| 4.1.4 | FOV radius | Tiles within radius visible, outside not | [ ] |
| 4.1.5 | Visibility states | Unseen → Visible transition | [ ] |
| 4.1.6 | Visibility states | Visible → Seen transition when out of FOV | [ ] |

### 4.2 FOV Optimizations — GREEN

| # | File | Task | Status |
|---|------|------|--------|
| 4.2.1 | fov.rs | Add `debug!()` log in early return (line 57) | [ ] |

### 4.3 Decompose FovSettings (P2) — REFACTOR

| # | File | Task | Status |
|---|------|------|--------|
| 4.3.1 | components.rs | Create `FovConfig { radius: u32 }` | [ ] |
| 4.3.2 | components.rs | Create `FovState { debug_reveal_all, needs_recalculation, debug_mode_applied, last_player_pos, dirty_tiles }` | [ ] |
| 4.3.3 | components.rs | Create `LosCache { cache: HashMap<...>, hits: usize, misses: usize }` | [ ] |
| 4.3.4 | fov.rs | Update to use three separate resources | [ ] |
| 4.3.5 | level_manager.rs | Update cache clearing to access `LosCache` | [ ] |
| 4.3.6 | main.rs | Initialize all three FOV resources | [ ] |

---

## Phase 5: TileIndex Optimization (TDD)

**Dependency:** Requires map, FOV. Major performance improvement.

### 5.1 Write TileIndex Usage Tests — RED

| # | Test Category | Tests to Write | Status |
|---|---------------|----------------|--------|
| 5.1.1 | TileIndex lookup | O(1) lookup returns correct entity | [ ] |
| 5.1.2 | find_nearest_unexplored | Returns closest unexplored tile | [ ] |
| 5.1.3 | count_unexplored_tiles | Accurate count with various visibility states | [ ] |

### 5.2 Implement TileIndex Usage (P0) — GREEN

| # | File | Function | Task | Status |
|---|------|----------|------|--------|
| 5.2.1 | player.rs | `find_nearest_unexplored()` | Add `tile_index: Res<TileIndex>`, use O(1) lookup | [ ] |
| 5.2.2 | player.rs | `count_unexplored_tiles()` | Build HashMap from visibility, use O(1) lookups | [ ] |
| 5.2.3 | input_handler.rs | `find_nearest_discovered_stairwell()` | Add TileIndex param, single-pass with O(1) lookups | [ ] |
| 5.2.4 | particles.rs | `is_suitable_for_particles_fast()` | Add `tile_index: &TileIndex`, use `tile_index.tiles.get()` | [ ] |
| 5.2.5 | particles.rs | `is_near_wall_fast()` | Add `tile_index: &TileIndex`, use O(1) lookup | [ ] |
| 5.2.6 | particles.rs | Call sites | Pass tile_index at lines 396 and 532 | [ ] |

---

## Phase 6: Particle System (TDD)

**Dependency:** Requires TileIndex optimization.

### 6.1 Write Particle Tests — RED

| # | Test Category | Tests to Write | Status |
|---|---------------|----------------|--------|
| 6.1.1 | Particle counts | ParticleCounts resource updated correctly | [ ] |
| 6.1.2 | Spawn limits | Respects primary/secondary limits | [ ] |

### 6.2 Implement Particle Optimization (P1) — GREEN

| # | File | Task | Status |
|---|------|------|--------|
| 6.2.1 | components.rs | Create `ParticleCounts { primary_count: usize, secondary_count: usize }` resource | [ ] |
| 6.2.2 | particles.rs | Add `init_resource::<ParticleCounts>()` to plugin | [ ] |
| 6.2.3 | particles.rs | Create `cache_particle_counts()` system | [ ] |
| 6.2.4 | particles.rs | Replace double-iteration with `ParticleCounts` reads (lines 296-301) | [ ] |
| 6.2.5 | particles.rs | Add `cache_particle_counts` to run before `spawn_biome_particles` | [ ] |
| 6.2.6 | particles.rs | Stagger wall checks with per-particle offset (line 526) | [ ] |
| 6.2.7 | particles.rs | Add `debug!()` before early return (line 292) | [ ] |

---

## Phase 7: Architecture Cleanup

**Dependency:** Core systems tested. Safe to restructure.

### 7.1 Resolve Circular Dependency (P2)

| # | File | Task | Status |
|---|------|------|--------|
| 7.1.1 | Create `src/events.rs` module | [ ] |
| 7.1.2 | events.rs | Move `LevelChangeEvent`, `RegenerateMapEvent`, `SpawnPosition` from input_handler.rs | [ ] |
| 7.1.3 | events.rs | Move shared functions if needed by both modules | [ ] |
| 7.1.4 | input_handler.rs | Replace definitions with `use crate::events::*;` | [ ] |
| 7.1.5 | level_manager.rs | Update imports to use events module | [ ] |
| 7.1.6 | main.rs | Add `mod events;` | [ ] |

### 7.2 Remove Unused Code (P1)

| # | File | Task | Status |
|---|------|------|--------|
| 7.2.1 | biome.rs | Remove `allowed_stair_assets` field from `BiomeConfig` | [ ] |
| 7.2.2 | biome.rs | Remove `allowed_stair_assets` from all biome config initializations | [ ] |

### 7.3 Split input_handler.rs (P3) — Optional

| # | Task | Status |
|---|------|--------|
| 7.3.1 | Create `src/input/mod.rs` with re-exports | [ ] |
| 7.3.2 | Create `src/input/movement.rs` | [ ] |
| 7.3.3 | Create `src/input/interaction.rs` | [ ] |
| 7.3.4 | Create `src/input/autoexplore.rs` | [ ] |
| 7.3.5 | Create `src/input/debug.rs` | [ ] |
| 7.3.6 | Update main.rs imports | [ ] |

---

## Phase 8: Integration & Property Tests

**Dependency:** All unit tests passing, architecture stable.

### 8.1 Integration Tests (P1)

| # | File | Test Scenarios | Status |
|---|------|----------------|--------|
| 8.1.1 | `tests/level_transitions.rs` | Map preservation across level changes | [ ] |
| 8.1.2 | `tests/level_transitions.rs` | Stair positioning correct after transition | [ ] |
| 8.1.3 | `tests/autoexplore.rs` | Complete map exploration terminates | [ ] |
| 8.1.4 | `tests/autoexplore.rs` | Path validity during exploration | [ ] |
| 8.1.5 | `tests/fov_movement.rs` | Visibility updates on player movement | [ ] |

### 8.2 Property-Based Tests (P2)

| # | File | Properties | Status |
|---|------|------------|--------|
| 8.2.1 | `tests/property_map_gen.rs` | All generated maps are connected | [ ] |
| 8.2.2 | `tests/property_map_gen.rs` | Stairs always accessible from spawn | [ ] |
| 8.2.3 | `tests/property_pathfinding.rs` | A* finds path iff path exists | [ ] |
| 8.2.4 | `tests/property_pathfinding.rs` | Returned path is always valid | [ ] |
| 8.2.5 | `tests/property_fov.rs` | LOS is symmetric | [ ] |
| 8.2.6 | `tests/property_fov.rs` | Visible tiles within radius bounds | [ ] |

### 8.3 Map Generation Tests (P1)

| # | Test Category | Tests to Write | Status |
|---|---------------|----------------|--------|
| 8.3.1 | Dimensions | Output matches MAP_WIDTH × MAP_HEIGHT | [ ] |
| 8.3.2 | Tile validity | All tiles are valid TileType variants | [ ] |
| 8.3.3 | Connectivity | All floor tiles reachable from spawn | [ ] |
| 8.3.4 | Reproducibility | Same seed produces identical map | [ ] |

---

## Phase 9: Polish

**Dependency:** All tests green. Final cleanup.

### 9.1 Error Handling Improvements (P1)

| # | File | Task | Status |
|---|------|------|--------|
| 9.1.1 | All remaining `.unwrap()` calls | Convert to `.expect()` with context | [ ] |

### 9.2 Naming Consistency (P3)

| # | File | Change | Status |
|---|------|--------|--------|
| 9.2.1 | components.rs | `Autoexplore.active` → `is_active` | [ ] |
| 9.2.2 | components.rs | `FovSettings.debug_mode_applied` → `is_debug_mode_applied` | [ ] |
| 9.2.3 | components.rs | `ParticleSettings.enabled` → `is_enabled` | [ ] |
| 9.2.4 | components.rs | `ParticleSettings.debug_mode` → `is_debug_mode` | [ ] |
| 9.2.5 | components.rs | `ParticleSpawner.initial_spawn_complete` → `has_initial_spawn_completed` | [ ] |
| 9.2.6 | particles.rs | `BiomeParticleConfig.enabled` → `is_enabled` | [ ] |

### 9.3 Long Function Refactoring (P3) — Optional

| # | File | Function | Task | Status |
|---|------|----------|------|--------|
| 9.3.1 | map.rs | `select_biome_asset()` | Extract helper functions | [ ] |
| 9.3.2 | input_handler.rs | `handle_stair_interaction()` | Extract up/down handlers | [ ] |
| 9.3.3 | level_manager.rs | `handle_level_transitions()` | Extract sub-functions | [ ] |

### 9.4 Documentation (P3) — Optional

| # | File | Task | Status |
|---|------|------|--------|
| 9.4.1 | camera.rs | Update WASD reference comment (line 134) | [ ] |
| 9.4.2 | input_handler.rs | Update "X key" comment (line 57) | [ ] |
| 9.4.3 | components.rs | Add rustdoc to public structs | [ ] |
| 9.4.4 | map.rs | Add rustdoc to GameMap, TileType | [ ] |

---

## FOV Spatial Grid (P2) — Deferred

**Note:** Only implement if FOV performance becomes a bottleneck after other optimizations.

| # | Task | Status |
|---|------|--------|
| D.1 | Create `TileSpatialGrid` resource for 10×10 regions | [ ] |
| D.2 | Populate during tile spawning | [ ] |
| D.3 | Use for incremental FOV updates | [ ] |
| D.4 | Clear/rebuild on level change | [ ] |

---

## Task Summary by Phase

| Phase | Description | Task Count | Priority | Status |
|-------|-------------|------------|----------|--------|
| 1 | Test Infrastructure & Constants | 12 | P0-P1 | [x] Complete |
| 2 | Map Core (TDD) | 15 | P0-P1 | [ ] |
| 3 | Pathfinding (TDD) | 17 | P0 | [ ] |
| 4 | FOV System (TDD) | 12 | P1-P2 | [ ] |
| 5 | TileIndex Optimization | 9 | P0 | [ ] |
| 6 | Particle System | 9 | P1 | [ ] |
| 7 | Architecture Cleanup | 14 | P1-P3 | [ ] |
| 8 | Integration & Property Tests | 16 | P1-P2 | [ ] |
| 9 | Polish | 17 | P1-P3 | [ ] |
| D | Deferred (FOV Spatial Grid) | 4 | P2 | [ ] |
| **Total** | | **125** | | **12/125** |

---

## Execution Order

```
Phase 1 ──→ Phase 2 ──→ Phase 3 ──→ Phase 4
   │           │           │           │
   └── Tests   └── Tests   └── Tests   └── Tests
       first       first       first       first

Phase 5 ──→ Phase 6 ──→ Phase 7 ──→ Phase 8 ──→ Phase 9
   │           │           │           │           │
   └── Tests   └── Tests   └── Safe    └── Int.    └── Polish
       first       first       refactor    tests       only
```

**Key Principle:** Never write implementation code until you have a failing test for it.

---

*Reorganized for TDD from CODE_REVIEW_REPORT.md*
