# Roguelike Game Design Document

## Technical Setup
- **Engine**: Bevy (pure Bevy implementation)
- **Window**: 1400x800 resolution, 60 fps
- **Libraries**: 
  - bevy_ecs_tilemap (tile rendering)
  - shadowcasting (FOV implementation)
  - pathfinding (A* and Dijkstra algorithms)
  - rand, noise (procedural generation)
  - bevy_asset_loader (asset management)

## Asset System
- **Sprites**: 32x32 pixels on grid-based spritesheets
- **Configuration**: .ron file for sprite names, coordinates, filepaths, sizes
- **Animations**:
  - Hop animation for grid movement (speed-adjusted)
  - Horizontal flip for directional facing (default: left-facing)
  - Bump animation for attack interactions

## Map System
### Tile Types
- **Floor**: Walkable terrain
- **Wall**: Blocking terrain with depth rendering
  - `wall_top`: When wall tile has wall below it
  - `wall_side`: When wall tile has no wall below it
- **Water**: Environmental tile
- **Stairwells**: Up/down level transitions

### Map Generation
- **Implementation**: Custom algorithms using rand and noise crates
- **Default Biome**: Caverns using Drunkard's Walk + tunneling
- **Algorithms Available**: Cellular Automata, BSP, Recursive Backtracking, Voronoi, Wave Function Collapse
- **Biome System**: Expandable framework for different generation techniques, sprites, spawns

### Level Progression
- **Depth Range**: 0-50 levels
- **Stairwell Rules**:
  - Level 0: Down stairwell only
  - Level 1-49: Both up and down stairwells
  - Level 50: Up stairwell only
- **Map Persistence**: Serialize/deserialize when changing levels
- **Stairwell Placement**: Random but accessible, with guaranteed pathfinding between up/down stairs

## Entity System
### Player Entity
- **Sprite**: rogues.png sheet, position 1,4
- **Movement**: Arrow keys (cardinal), SHIFT+arrows (diagonal)
- **Interaction**: E key (items, doors, switches, NPCs)
- **Movement Behavior**: Single tile per input, rapid press support, hold-to-move with delay

### Debug Controls
- **SHIFT+O**: Toggle FOV reveal
- **SHIFT+R**: Regenerate current map
- **SHIFT+E**: Use stairwells

## Game Mechanics
### Turn System
- **Turn-based**: Player action triggers simultaneous entity actions
- **Actions**: Movement, interactions, combat

### Field of View
- **Implementation**: shadowcasting crate or custom shadowcasting algorithm
- **Behavior**: 
  - Walls block vision
  - Unseen tiles remain black
  - Previously seen tiles show shaded when out of range
  - Entities show last-seen position when out of range

### Camera System
- **Following**: Loosely follows player with map boundary constraints
- **Zoom Levels**:
  - Full zoom out: Show entire map, camera detached from player
  - Zoomed in: Camera centers and follows player
  - Smooth transition between zoom levels

## Deferred Features
The following traditional roguelike features will be implemented in future development phases:

### Combat System
- Attack mechanics and damage calculation
- Weapon types and combat stats
- Turn-based combat resolution

### Entities
- **Enemies**: AI behavior, pathfinding, different enemy types
- **Friendlies**: NPCs, allies, dialogue system
- **Neutrals**: Non-combat entities, merchants, quest givers

### Character Progression
- **Stats**: Strength, dexterity, intelligence, etc.
- **Health/Hit Points**: Damage tracking and healing
- **Action Points**: Turn-based action economy
- **Experience/Leveling**: Character advancement

### Item Systems
- **Inventory**: Item storage and management
- **Equipment**: Weapons, armor, accessories
- **Consumables**: Potions, scrolls, food

### User Interface
- **Menus**: Main menu, options, save/load
- **HUD**: Health bars, minimap, status indicators
- **Inventory Screen**: Item management interface

### Additional Features
- **Magic/Spells**: Casting system and spell effects
- **Quests**: Objective tracking and rewards
- **Shops**: Trading and economy
- **Save System**: Game state persistence