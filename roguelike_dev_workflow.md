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

## Phase 7: Field of View System (Days 16-18)

### Step 7.1: FOV Implementation
- [ ] Integrate shadowcasting crate
- [ ] Implement basic FOV calculation from player position
- [ ] Test FOV updates with player movement

### Step 7.2: Visibility States
- [ ] Create tile visibility component (unseen, seen, visible)
- [ ] Implement tile rendering based on visibility
- [ ] Add shading for previously seen but not visible tiles
- [ ] Test visibility state transitions

### Step 7.3: Entity Visibility
- [ ] Implement entity visibility based on FOV
- [ ] Show last-seen positions for out-of-range entities
- [ ] Test entity appearance/disappearance

**Milestone 7: Complete FOV system with proper visibility states**

## Phase 8: Turn System (Days 19-20)

### Step 8.1: Turn Management
- [ ] Implement turn-based system structure
- [ ] Create turn manager component
- [ ] Add action queue system
- [ ] Test basic turn progression

### Step 8.2: Action System
- [ ] Define action types (movement, interaction)
- [ ] Implement action validation
- [ ] Add simultaneous entity action processing
- [ ] Test turn system with player actions

**Milestone 8: Functional turn-based system**

## Phase 9: Debug Tools and Polish (Days 21-22)

### Step 9.1: Debug Controls
- [ ] Implement SHIFT+O for FOV reveal toggle
- [ ] Add SHIFT+R for map regeneration
- [ ] Create SHIFT+E for stairwell usage
- [ ] Test all debug controls

### Step 9.2: Interaction System Foundation
- [ ] Add basic E key interaction system
- [ ] Create interaction component for entities
- [ ] Test interaction detection and triggering
- [ ] Prepare foundation for future features

### Step 9.3: Final Polish
- [ ] Add any missing animations
- [ ] Optimize rendering performance
- [ ] Test all systems working together
- [ ] Fix any remaining bugs

**Milestone 9: Complete playable prototype with all core features**

## Phase 10: Testing and Documentation (Days 23-24)

### Step 10.1: Comprehensive Testing
- [ ] Test all movement combinations
- [ ] Verify map generation works consistently
- [ ] Test level transitions thoroughly
- [ ] Validate FOV in various scenarios
- [ ] Check debug controls functionality

### Step 10.2: Code Documentation
- [ ] Add code comments to complex systems
- [ ] Create README with build/run instructions
- [ ] Document any known issues or limitations
- [ ] Prepare for future development phases

**Final Milestone: Complete, tested, and documented roguelike prototype**

## Development Tips for Junior Developers

### Daily Workflow
1. Start each day by reviewing the previous day's work
2. Run the game and test existing functionality
3. Work on one step at a time - don't jump ahead
4. Test frequently (after each sub-step if possible)
5. Commit code at each milestone completion

### Debugging Strategies
- Use `println!` statements liberally for debugging
- Test each system in isolation before integration
- Keep backup versions of working code
- Don't be afraid to rewrite difficult sections

### When Stuck
1. Re-read the design document section
2. Look up Bevy documentation and examples
3. Break the problem into smaller sub-problems
4. Consider simplifying the approach initially

### Code Organization
- Keep each system in its own module/file
- Use descriptive component and system names
- Comment complex algorithms thoroughly
- Maintain consistent code style throughout

## Estimated Timeline: 24 days for complete prototype

This workflow prioritizes getting a playable prototype quickly while building a solid foundation for future features. Each phase builds incrementally on the previous work, ensuring steady progress and early feedback opportunities.