# Roguelike Prototype Development Workflow

## Phase 1: Project Foundation (Days 1-2)

### Step 1.1: Environment Setup
- [x] Install Rust and set up development environment
- [x] Create new Bevy project with Cargo.toml dependencies:
  ```toml
  bevy = "0.12"
  bevy_ecs_tilemap = "0.12"
  shadowcasting = "0.8"
  pathfinding = "4.0"
  rand = "0.8"
  noise = "0.8"
  bevy_asset_loader = "0.18"
  ron = "0.8"
  ```
- [x] Set up basic main.rs with Bevy app structure
- [x] Configure window size (1400x800) and 60fps target

### Step 1.2: Basic Window and Plugins
- [x] Create minimal Bevy app that opens a window
- [x] Add DefaultPlugins
- [x] Add basic camera setup
- [x] Test that window opens and closes properly

**Milestone 1: Empty game window opens successfully** ✅

## Phase 2: Asset System (Days 3-4)

### Step 2.1: Asset Structure Setup
- [x] Create assets folder structure:
  ```
  assets/
  ├── sprites/
  │   └── rogues.png (32x32 grid spritesheet)
  ├── config/
  │   └── sprites.ron
  └── maps/
  ```
- [x] Create sprite configuration .ron file for tile definitions
- [x] Set up bevy_asset_loader configuration

### Step 2.2: Basic Sprite Rendering
- [x] Create sprite component system
- [x] Load and display a single test sprite
- [x] Implement grid-based positioning (32x32 tiles)
- [x] Test sprite appears on screen at correct position

**Milestone 2: Single sprite renders on grid** ✅

## Phase 3: Map System Foundation (Days 5-7)

### Step 3.1: Tile System
- [x] Define tile type enum (Floor, Wall, Water, Stairwell)
- [x] Create tile entity components
- [x] Implement basic tile rendering with bevy_ecs_tilemap
- [x] Create simple 10x10 test map with different tile types

### Step 3.2: Map Data Structure
- [x] Design map data structure (2D array or HashMap)
- [x] Implement map loading/creation system
- [x] Add wall depth rendering logic (wall_top vs wall_side)
- [x] Test manual map creation and rendering

### Step 3.3: Basic Map Generation
- [x] Implement simple room generation (single rectangular room)
- [x] Add wall placement around room borders
- [x] Test that generated map renders correctly
- [x] Add basic validation (ensure map has floor tiles)

**Milestone 3: Simple generated map displays with walls and floors** ✅

## Phase 4: Player Entity and Movement (Days 8-10) ✅

### Step 4.1: Player Entity Creation
- [x] Create player component and spawn system
- [x] Load player sprite from rogues.png (position 1,4)
- [x] Position player on map
- [x] Ensure player renders above map tiles

### Step 4.2: Input System
- [x] Implement keyboard input detection
- [x] Create movement input handling (arrow keys + WASD)
- [x] Test input registration (log to console)

### Step 4.3: Grid Movement
- [x] Implement tile-based movement system
- [x] Add collision detection with walls
- [x] Ensure player stays within map bounds
- [x] Add movement validation before applying

### Step 4.4: Movement Polish
- [x] Add hop animation for movement (100ms smooth transition with z-hop effect)
- [x] Implement horizontal sprite flipping for direction
- [x] Add rapid press support (immediate response)
- [x] Implement hold-to-move with delay (150ms repeat timer)
- [x] Fix player positioning to center on tiles (not grid lines)
- [x] Correct sprite flipping logic (no flip for left, flip for right)

**Milestone 4: Player moves smoothly on grid with collision detection** ✅

**Phase 4 Implementation Details:**
- Player spawns at grid position (2,2) in the center of the 10x10 room
- Player positioned on tile centers using -4.5 offset for proper alignment
- Smooth hop animation with parabolic z-curve for visual appeal
- Sprite flips horizontally: no flip when moving left (natural direction), flip when moving right
- Hold-to-move system with 150ms delay for continuous movement
- Collision detection prevents movement into walls
- Support for both arrow keys and WASD input
- Immediate response to rapid key presses

## Phase 5: Camera System (Days 11-12) ✅

### Step 5.1: Basic Camera Following ✅
- [x] Implement camera that follows player position
- [x] Add smooth camera interpolation
- [x] Test camera movement with player

### Step 5.2: Camera Constraints and Zoom ✅
- [x] Add map boundary constraints for camera
- [x] Implement zoom levels (full map vs centered)
- [x] Add smooth zoom transitions
- [x] Test camera behavior at map edges

**Milestone 5: Camera follows player smoothly with proper constraints** ✅

**Completed Features:**
- Camera entity with CameraFollow component for tracking
- Smooth camera interpolation with configurable lerp_speed (2.0)
- Zoom system with +/- keys and R for reset (0.5x to 3.0x range)
- Smart boundary constraints (enabled for maps > 15x15)
- Debug system with F1 (camera info) and F2 (controls help)
- Proper system ordering and entity management

**Phase 5 Post-Completion Fix:**
- [x] Fixed tile rendering to use correct sprites from tiles.png
- [x] Implemented sprite database system with proper texture index calculation
- [x] Added support for wall_top/wall_side logic based on tiles below
- [x] Added random floor selection from multiple floor sprite types
- [x] Verified sprite sheet dimensions (17x26 tiles) and proper indexing

## Phase 6: Advanced Map Generation (Days 13-15) ✅

### Step 6.1: Drunkard's Walk Algorithm ✅
- [x] Research and implement Drunkard's Walk algorithm
- [x] Create cavern-style map generation
- [x] Add tunneling between disconnected areas  
- [x] Test various map generations

### Step 6.2: Level System ✅
- [x] Implement multi-level structure (0-50 levels)
- [x] Add stairwell placement system
- [x] Ensure pathfinding between up/down stairs
- [x] Add level transition logic

### Step 6.3: Map Persistence ✅
- [x] Implement map serialization/deserialization
- [x] Save generated maps when changing levels
- [x] Load previously generated maps
- [x] Test level transitions preserve map state

**Milestone 6: Multi-level dungeon with persistent maps** ✅

**Phase 6 Implementation Details:**
- Implemented Drunkard's Walk algorithm with multiple walkers for organic cave-like generation
- Added comprehensive level system (CurrentLevel resource, LevelMaps storage) supporting 0-50 levels
- Created stairwell placement system with StairUp/StairDown tiles and proper positioning
- Implemented map serialization/deserialization using SavedMapData with serde for persistence
- Added level transition system with E key interaction on stairs
- Created tunnel carving system to connect disconnected cave areas
- Updated coordinate system for larger maps (30x20) with proper player spawn positioning
- Added debug map regeneration with Shift+R key combination
- Integrated stair sprites into tile texture system (positions 2,6 and 3,6)
- Implemented comprehensive error handling and state management
- Game successfully runs with all Phase 6 features working correctly

## Phase 7: Field of View System (Days 16-18) ✅ COMPLETED

### Step 7.1: FOV Implementation ✅
- [x] Remove shadowcasting dependency (compatibility issues)
- [x] Implement basic line-of-sight FOV calculation from player position
- [x] Add Bresenham line algorithm for vision blocking
- [x] Test FOV updates with player movement (working with movement detection)
- [x] Optimize FOV performance for larger maps (conditional recalculation system)

### Step 7.2: Visibility States ✅
- [x] Create tile visibility component (unseen, seen, visible)
- [x] Implement tile rendering based on visibility with color tinting
- [x] Add shading for previously seen but not visible tiles
- [x] Test visibility state transitions (working properly)
- [x] Enhanced color scheme for better visibility contrast

### Step 7.3: Entity Visibility (Deferred)
- [ ] Implement entity visibility based on FOV (waiting for entity system)
- [ ] Show last-seen positions for out-of-range entities
- [ ] Test entity appearance/disappearance

### Step 7.4: FOV Polish and Debug ✅
- [x] Add SHIFT+O debug toggle to reveal entire map
- [x] Fine-tune FOV radius and visibility settings
- [x] Add FOV calculation optimization (movement detection system)
- [x] Test FOV system with level transitions (integrated with level manager)

**Milestone 7: Complete FOV system with proper visibility states ✅**

*Phase 7 Completed Features:*
- Performance-optimized FOV system with conditional recalculation
- Movement detection system to trigger FOV only when needed
- Enhanced tile visibility with improved color schemes
- Full integration with level transitions and map regeneration
- Debug controls with SHIFT+O toggle for full map reveal
- Resource existence checking to prevent crashes during initialization

## Phase 8: Biome System Implementation (Days 19-22)

### Step 8.1: Biome Foundation Structure
- [ ] Create biome module and enum system
- [ ] Define BiomeType enum with all 9 biomes from infosheet
- [ ] Create BiomeConfig struct to hold biome parameters
- [ ] Implement biome asset mapping system
- [ ] Test basic biome enum and config creation

### Step 8.2: Asset Coordinate System
- [ ] Update sprite configuration to support biome-specific assets
- [ ] Implement coordinate-to-sprite mapping for biome assets
- [ ] Create asset validation system for biome rules
- [ ] Add fallback sprite handling for missing assets
- [ ] Test asset coordinate system with existing sprites

### Step 8.3: Caverns Biome Implementation
- [ ] Define Caverns biome configuration
- [ ] Implement asset restrictions for Caverns (from infosheet):
  ```
  Floors: 0,0  1,0  0,1  1,1
  Water: 0,6  1,6  2,6  3,6  
  Walls: 1,7  2,7  3,7
  Stairs: 1,8  2,8  3,8
  ```
- [ ] Update existing map generation to use Caverns biome
- [ ] Test Caverns biome renders correctly with restricted assets

### Step 8.4: Biome-Aware Map Generation
- [ ] Modify Drunkard's Walk to accept biome parameters
- [ ] Update tile placement to respect biome asset restrictions
- [ ] Add biome-specific generation parameters (if needed)
- [ ] Integrate biome system with level manager
- [ ] Test map generation uses only allowed assets

### Step 8.5: Biome System Integration
- [ ] Add biome selection to level system
- [ ] Update map serialization to include biome data
- [ ] Ensure level transitions preserve biome information
- [ ] Add debug controls for biome testing (SHIFT+B for biome cycling)
- [ ] Test complete biome system with level persistence

**Milestone 8: Complete Caverns biome with expandable biome system**

## Implementation Details for Junior Developers

### Step 8.1: Biome Foundation Structure

Create a new file `src/biome.rs`:

```rust
// filepath: src/biome.rs
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum BiomeType {
    Caverns,
    Underglade,
    FungalDeep,
    CinderGaol,
    AbyssalHold,
    NetherGrange,
    ChthronicCrypts,
    HypogealKnot,
    StygianPool,
}

#[derive(Clone, Debug)]
pub struct BiomeConfig {
    pub name: String,
    pub description: String,
    pub allowed_floor_assets: Vec<(u32, u32)>,
    pub allowed_wall_assets: Vec<(u32, u32)>,
    pub allowed_water_assets: Vec<(u32, u32)>,
    pub allowed_stair_assets: Vec<(u32, u32)>,
}

impl BiomeType {
    pub fn get_config(&self) -> BiomeConfig {
        match self {
            BiomeType::Caverns => BiomeConfig {
                name: "Caverns".to_string(),
                description: "Natural underground caves with rough stone walls, frequent water features, and occasional crystal formations.".to_string(),
                allowed_floor_assets: vec![(0,0), (1,0), (0,1), (1,1)],
                allowed_wall_assets: vec![(1,7), (2,7), (3,7)],
                allowed_water_assets: vec![(0,6), (1,6), (2,6), (3,6)],
                allowed_stair_assets: vec![(1,8), (2,8), (3,8)],
            },
            // TODO: Add other biomes in future steps
            _ => todo!("Implement other biomes"),
        }
    }
}
```

Add to `src/main.rs`:
```rust
// filepath: src/main.rs
mod biome;
use biome::*;
```

### Step 8.2: Asset Coordinate System

Update `src/components.rs` to include biome information:

```rust
// filepath: src/components.rs
// ...existing code...
use crate::biome::BiomeType;

#[derive(Resource)]
pub struct CurrentLevel {
    pub level: u32,
    pub biome: BiomeType,  // Add this line
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SavedMapData {
    pub tiles: Vec<Vec<TileType>>,
    pub biome: BiomeType,  // Add this line
}
```

### Step 8.3: Caverns Biome Implementation

Update `src/map.rs` to use biome-aware asset selection:

```rust
// filepath: src/map.rs
// ...existing code...
use crate::biome::{BiomeType, BiomeConfig};

// Add this new function
fn select_biome_asset(biome_config: &BiomeConfig, tile_type: TileType, rng: &mut impl Rng) -> (u32, u32) {
    let assets = match tile_type {
        TileType::Floor => &biome_config.allowed_floor_assets,
        TileType::Wall => &biome_config.allowed_wall_assets,
        TileType::Water => &biome_config.allowed_water_assets,
        TileType::StairUp | TileType::StairDown => &biome_config.allowed_stair_assets,
    };
    
    if assets.is_empty() {
        return (0, 0); // Fallback
    }
    
    assets[rng.gen_range(0..assets.len())]
}
```

### Step 8.4: Biome-Aware Map Generation

Update the `spawn_map` function in `src/map.rs`:

```rust
// filepath: src/map.rs
pub fn spawn_map(
    commands: &mut Commands,
    texture_assets: &TextureAssets,
    current_level: &CurrentLevel,
) {
    let biome_config = current_level.biome.get_config();
    let mut rng = rand::thread_rng();
    
    // ...existing map generation code...
    
    // When placing tiles, use biome-aware selection:
    for y in 0..height {
        for x in 0..width {
            let tile_type = map.tiles[y][x];
            let (sprite_x, sprite_y) = select_biome_asset(&biome_config, tile_type, &mut rng);
            
            // ...rest of tile spawning code...
        }
    }
}
```

### Step 8.5: Biome System Integration

Update `src/level_manager.rs` to initialize with Caverns biome:

```rust
// filepath: src/level_manager.rs
// ...existing code...

pub fn setup_level_manager(mut commands: Commands) {
    commands.insert_resource(CurrentLevel { 
        level: 0,
        biome: BiomeType::Caverns,  // Add this line
    });
}
```

### Testing Strategy for Each Step

1. **Step 8.1**: Compile and verify biome enum can be created
2. **Step 8.2**: Test that biome data serializes/deserializes correctly
3. **Step 8.3**: Generate maps and verify only Caverns assets are used
4. **Step 8.4**: Test map regeneration uses biome-restricted assets
5. **Step 8.5**: Test level transitions preserve biome information

### Debug Controls

Add to input handling system:
```rust
// SHIFT+B: Cycle biome for current level (debug)
if keyboard_input.pressed(KeyCode::ShiftLeft) && keyboard_input.just_pressed(KeyCode::KeyB) {
    // Cycle biome and regenerate map
}
```

### Future Expansion

This biome system is designed to easily add the remaining 8 biomes by:
1. Adding new match arms to `BiomeType::get_config()`
2. Implementing biome-specific generation algorithms
3. Adding biome progression logic (which biome appears at which levels)

### Expected Timeline
- **Day 19**: Steps 8.1-8.2 (Foundation and Asset System)
- **Day 20**: Steps 8.3-8.4 (Caverns Implementation) 
- **Day 21**: Step 8.5 (Integration and Testing)
- **Day 22**: Polish, debug controls, and documentation

This approach ensures you have a solid, expandable biome system while completing your current Caverns biome properly according to your infosheet specifications.