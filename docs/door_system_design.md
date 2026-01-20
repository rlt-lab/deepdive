# Door System Design Document

## Overview

This document outlines the design for implementing a door system in Deepdive, a turn-based roguelike dungeon crawler built with Rust and Bevy 0.16.1. The design is informed by research into classic roguelikes (Nethack, Brogue, Caves of Qud) and analysis of the existing codebase architecture.

---

## Research Summary

### Nethack Door Mechanics
[NetHack Wiki - Door](https://nethackwiki.com/wiki/Door)

Nethack has the most comprehensive door system among classic roguelikes:

- **States**: Doorways can have no door, broken door, open door, or closed door
- **Closed door substates**: locked/unlocked, trapped/untrapped, secret
- **Interaction**: `o` to open, `c` to close
- **Locked doors**: Can be picked (lockpicks, skeleton key), kicked open (`^D`), or magically opened (wands/spells)
- **Secret doors**: Hidden until discovered with `s` (search), then behave as normal doors
- **Monster interaction**: Giants bust doors, ghosts pass through, amoeboids flow under, vampires shapeshift to pass
- **Diagonal restriction**: Open doors cannot be entered diagonally
- **Size restriction**: Tiny creatures cannot pull doors open

### Brogue Door Mechanics
[Brogue Official Site](https://sites.google.com/site/broguegame)

Brogue uses doors for tactical visibility management:

- **FOV blocking**: Cannot see beyond a room until door is opened
- **Turn cost**: Opening a door takes a turn
- **Auto-close**: Doors close behind the player (design choice)
- **Strategic positioning**: Standing at doorway allows visibility into both rooms

### Caves of Qud Door Mechanics
[Caves of Qud Wiki - Security Door](https://wiki.cavesofqud.com/wiki/Security_door)

Caves of Qud adds complexity with special door types:

- **Security doors**: Require security cards or special abilities (Psychometry mutation)
- **Cybernetic unlock**: Security interlock implant allows bypassing locks
- **Persistent lock state**: Doors remain locked for other creatures
- **Destructible walls**: Doors can be bypassed by destroying adjacent walls
- **Living doors**: Special doors that can only be destroyed by fire

### FOV Considerations
[Roguelike Vision Algorithms](https://www.adammil.net/blog/v125_Roguelike_Vision_Algorithms.html)

Key principles from roguelike FOV design:

- **Symmetry**: If A can see B, then B should see A (fairness principle)
- **No blind corners**: Should see at least two tiles around corners to avoid unfair monster encounters
- **Closed doors**: Should block LOS completely (like walls)
- **Open doors**: Should allow vision through (like floor tiles)

---

## Codebase Analysis

### Relevant Existing Systems

#### 1. Tile Type System (`src/components.rs`)
```rust
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
pub enum TileType {
    Floor,
    Wall,
    Water,
    StairUp,
    StairDown,
}

impl TileType {
    pub fn is_walkable(&self) -> bool {
        matches!(self, TileType::Floor | TileType::StairUp | TileType::StairDown)
    }
}
```

#### 2. FOV System (`src/fov.rs`)
- Uses Bresenham's line-of-sight algorithm
- LOS cache for performance: `HashMap<(x0, y0, x1, y1), bool>`
- Walls block LOS (except at target tile - you can "see" walls)
- Incremental updates for performance

#### 3. Tile Visibility (`src/components.rs`)
```rust
#[derive(Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, Reflect)]
pub enum TileVisibility {
    #[default]
    Unseen,
    Seen,
    Visible,
}
```

#### 4. Persistence System (`src/level_manager.rs`)
```rust
#[derive(Serialize, Deserialize, Clone)]
pub struct SavedMapData {
    pub tiles: Vec<TileType>,
    pub stair_up_pos: Option<(u32, u32)>,
    pub stair_down_pos: Option<(u32, u32)>,
    pub biome: BiomeType,
    pub tile_visibility: HashMap<(u32, u32), TileVisibility>,
}
```

#### 5. Input System (`src/input/`)
- Key bindings resource with configurable actions
- Movement handled via `PlayerMoveIntent` events
- Interaction handled in `src/input/interaction.rs`

#### 6. Sprite System (`src/map.rs`)
- `sprite_position_to_index(x, y) -> u32` = y * 17 + x
- Biome-aware asset selection via `select_biome_asset()`
- Tileset: 17x26 grid, 32x32 pixels per tile

---

## Implementation Plan

### Phase 1: Basic Door System (MVP)

#### 1.1 Add Door Component
**File**: `src/components.rs`

```rust
/// Represents a door entity with open/closed state
#[derive(Component, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub struct Door {
    pub is_open: bool,
}

impl Default for Door {
    fn default() -> Self {
        Self { is_open: false }
    }
}
```

#### 1.2 Extend TileType
**File**: `src/components.rs`

```rust
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
pub enum TileType {
    Floor,
    Wall,
    Water,
    StairUp,
    StairDown,
    Door,  // New variant
}
```

#### 1.3 Update Walkability
**File**: `src/components.rs`

Door walkability depends on the Door component state, not the TileType alone. Movement system needs to check:
1. Is tile type Door?
2. If yes, lookup Door component via TileIndex
3. Return `is_open` state

```rust
// In GameMap or movement system
pub fn is_passable(&self, x: u32, y: u32, door_query: &Query<&Door>, tile_index: &TileIndex) -> bool {
    match self.get(x, y) {
        TileType::Door => {
            if let Some(entity) = tile_index.get(x, y) {
                door_query.get(*entity).map(|d| d.is_open).unwrap_or(false)
            } else {
                false
            }
        }
        tile => tile.is_walkable()
    }
}
```

#### 1.4 Add Door Input Handling
**File**: `src/input/interaction.rs` and `src/input/movement.rs`

Three ways to interact with doors:

1. **Move into closed door** → Opens door and moves player onto it
2. **O key** → Opens adjacent closed door (without moving)
3. **C key** → Closes adjacent open door (without moving)

```rust
// Add to KeyBindings struct:
pub open_door: Vec<KeyCode>,   // Default: [KeyCode::KeyO]
pub close_door: Vec<KeyCode>,  // Default: [KeyCode::KeyC]
```

##### Movement-Based Door Opening

Modify `handle_movement_input` to open doors when player walks into them:

```rust
/// In handle_movement_input, when checking if target tile is passable:
pub fn handle_movement_input(
    mut events: EventReader<PlayerMoveIntent>,
    mut player: Query<&mut Player>,
    mut doors: Query<&mut Door>,
    tile_index: Res<TileIndex>,
    map: Res<GameMap>,
    // ... other params
) {
    for event in events.read() {
        let mut player = player.single_mut();
        let (new_x, new_y) = (
            (player.x as i32 + event.dx) as u32,
            (player.y as i32 + event.dy) as u32,
        );

        let tile = map.get(new_x, new_y);

        // Check if movement is allowed
        let can_move = match tile {
            TileType::Door => {
                // If door exists, check state or open it
                if let Some(&entity) = tile_index.get(new_x, new_y) {
                    if let Ok(mut door) = doors.get_mut(entity) {
                        if !door.is_open {
                            // Open the door and allow movement
                            door.is_open = true;
                            // Trigger FOV recalculation
                        }
                        true  // Can move onto door tile (now open)
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            _ => tile.is_walkable(),
        };

        if can_move {
            player.x = new_x;
            player.y = new_y;
            // ... trigger animation, etc.
        }
    }
}
```

##### Explicit Open Door (O Key)

```rust
/// System to explicitly open an adjacent door without moving
pub fn handle_open_door(
    keyboard: Res<ButtonInput<KeyCode>>,
    bindings: Res<KeyBindings>,
    player: Query<&Player>,
    mut doors: Query<&mut Door>,
    tile_index: Res<TileIndex>,
    map: Res<GameMap>,
) {
    if !bindings.open_door.iter().any(|k| keyboard.just_pressed(*k)) {
        return;
    }

    let player = player.single();

    // Check all 4 cardinal directions for adjacent closed doors
    let adjacent = [(0, -1), (0, 1), (-1, 0), (1, 0)];

    for (dx, dy) in adjacent {
        let nx = (player.x as i32 + dx) as u32;
        let ny = (player.y as i32 + dy) as u32;

        if map.get(nx, ny) == TileType::Door {
            if let Some(&entity) = tile_index.get(nx, ny) {
                if let Ok(mut door) = doors.get_mut(entity) {
                    if !door.is_open {
                        door.is_open = true;
                        // Trigger sprite update and FOV recalculation
                        return;  // Only open one door per keypress
                    }
                }
            }
        }
    }
}
```

##### Explicit Close Door (C Key)

```rust
/// System to explicitly close an adjacent door
pub fn handle_close_door(
    keyboard: Res<ButtonInput<KeyCode>>,
    bindings: Res<KeyBindings>,
    player: Query<&Player>,
    mut doors: Query<&mut Door>,
    tile_index: Res<TileIndex>,
    map: Res<GameMap>,
) {
    if !bindings.close_door.iter().any(|k| keyboard.just_pressed(*k)) {
        return;
    }

    let player = player.single();

    // Check all 4 cardinal directions for adjacent open doors
    let adjacent = [(0, -1), (0, 1), (-1, 0), (1, 0)];

    for (dx, dy) in adjacent {
        let nx = (player.x as i32 + dx) as u32;
        let ny = (player.y as i32 + dy) as u32;

        if map.get(nx, ny) == TileType::Door {
            if let Some(&entity) = tile_index.get(nx, ny) {
                if let Ok(mut door) = doors.get_mut(entity) {
                    if door.is_open {
                        door.is_open = false;
                        // Trigger sprite update and FOV recalculation
                        return;  // Only close one door per keypress
                    }
                }
            }
        }
    }
}
```

##### Design Notes

- **Move-to-open**: Walking into a closed door opens it in a single action (player ends up on door tile). This is convenient for exploration.
- **O key**: Useful for opening doors without stepping through (peek into room, tactical positioning).
- **C key**: Essential for closing doors behind you (escape from monsters, stealth).
- **Standing on door**: Player can stand on an open door tile. Closing requires being adjacent, not on the door.

#### 1.5 Update FOV System
**File**: `src/fov.rs`

Modify `has_line_of_sight_cached` to check door state:

```rust
// In LOS calculation, treat closed doors as walls
fn blocks_los(tile_type: TileType, door_query: &Query<&Door>, tile_index: &TileIndex, x: u32, y: u32) -> bool {
    match tile_type {
        TileType::Wall => true,
        TileType::Door => {
            if let Some(&entity) = tile_index.get(x, y) {
                !door_query.get(entity).map(|d| d.is_open).unwrap_or(true)
            } else {
                true  // No entity = treat as blocking
            }
        }
        _ => false,
    }
}
```

#### 1.6 Update Persistence
**File**: `src/components.rs` (SavedMapData)

```rust
#[derive(Serialize, Deserialize, Clone)]
pub struct SavedMapData {
    pub tiles: Vec<TileType>,
    pub stair_up_pos: Option<(u32, u32)>,
    pub stair_down_pos: Option<(u32, u32)>,
    pub biome: BiomeType,
    pub tile_visibility: HashMap<(u32, u32), TileVisibility>,
    pub door_states: HashMap<(u32, u32), bool>,  // New field
}
```

Update `to_saved_data()` and `from_saved_data()` methods to capture/restore door states.

#### 1.7 Sprite Selection
**File**: `src/map.rs`

Door sprites from tiles.png:
- Closed door: position (2, 16) → index = 16 * 17 + 2 = **274**
- Open door: position (3, 16) → index = 16 * 17 + 3 = **275**

```rust
// In sprite selection logic
TileType::Door => {
    if let Some(&entity) = tile_index.get(x, y) {
        if door_query.get(entity).map(|d| d.is_open).unwrap_or(false) {
            275  // Open door sprite
        } else {
            274  // Closed door sprite
        }
    } else {
        274  // Default to closed
    }
}
```

#### 1.8 Door Placement in Map Generation
**File**: `src/map.rs` or new `src/door_placement.rs`

Place doors in logical locations:
1. Identify corridor tiles (floor tiles with walls on opposite sides)
2. Check that door position has floor tiles on at least 2 sides (pathway)
3. Ensure door is not surrounded by floors (would be pointless)

```rust
pub fn find_valid_door_positions(map: &GameMap) -> Vec<(u32, u32)> {
    let mut positions = Vec::new();

    for y in 1..map.height - 1 {
        for x in 1..map.width - 1 {
            if map.get(x, y) == TileType::Floor {
                // Check horizontal corridor pattern: Wall-Floor-Wall vertically
                let is_horizontal_corridor =
                    map.get(x, y - 1) == TileType::Wall &&
                    map.get(x, y + 1) == TileType::Wall &&
                    map.get(x - 1, y) == TileType::Floor &&
                    map.get(x + 1, y) == TileType::Floor;

                // Check vertical corridor pattern: Wall-Floor-Wall horizontally
                let is_vertical_corridor =
                    map.get(x - 1, y) == TileType::Wall &&
                    map.get(x + 1, y) == TileType::Wall &&
                    map.get(x, y - 1) == TileType::Floor &&
                    map.get(x, y + 1) == TileType::Floor;

                if is_horizontal_corridor || is_vertical_corridor {
                    positions.push((x, y));
                }
            }
        }
    }

    positions
}
```

---

### Phase 2: Enhanced Door Features

#### 2.1 Locked Doors
```rust
#[derive(Component, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub struct Door {
    pub is_open: bool,
    pub is_locked: bool,
    pub lock_difficulty: u8,  // 0-255 difficulty for lockpicking
}
```

Requires:
- Key items that can unlock specific doors
- Lockpicking skill/mechanic
- UI feedback for locked door attempts

#### 2.2 Secret Doors
```rust
#[derive(Component, Clone, Copy, Serialize, Deserialize, Reflect)]
pub struct SecretDoor {
    pub is_discovered: bool,
}
```

Requires:
- Search action (s key) to discover
- Initially renders as wall tile
- Integrates with FOV once discovered

#### 2.3 Door Durability
```rust
pub struct Door {
    pub is_open: bool,
    pub health: u32,  // Can be damaged/destroyed
}
```

Requires:
- Combat/interaction system to damage doors
- Broken door state (always passable, different sprite)

#### 2.4 Monster Door Interaction
Based on Nethack patterns:
- Most monsters cannot open doors
- Specific monster types can: humanoids, giants (bash), ghosts (phase through)
- Door closing behavior when monsters pursue

---

### Phase 3: Advanced Features

#### 3.1 Automatic Doors
- Doors that open when player approaches
- Close after player passes

#### 3.2 One-Way Doors
- Can only be opened from one side
- Useful for dungeon progression design

#### 3.3 Trapped Doors
- Trigger traps when opened
- Detectable/disarmable

#### 3.4 Special Door Types (Caves of Qud inspired)
- Security doors requiring special items
- Living doors with unique destruction methods
- Magical doors with spell requirements

---

## System Execution Order

```
GameplaySet::Input
├── detect_movement_input
├── handle_movement_input  ← Opens closed doors on move-into, moves player
├── handle_open_door       ← NEW: O key opens adjacent closed door
├── handle_close_door      ← NEW: C key closes adjacent open door
├── handle_stair_interaction
└── ...

GameplaySet::Rendering
├── update_door_sprites  ← NEW: Update sprite based on Door.is_open
└── ...

GameplaySet::FOV
├── detect_player_movement  ← Trigger on door state change too
├── calculate_fov  ← Check door states in LOS calculation
└── update_tile_visibility
```

---

## Potential Issues at Scale

### 1. Performance Concerns

**LOS Cache Invalidation**
- Current LOS cache is keyed by `(x0, y0, x1, y1)`
- Door state changes require cache invalidation for all rays passing through door tile
- **Mitigation**: Track which cache entries pass through each door position; selective invalidation

**Entity Queries in Hot Paths**
- Checking door states requires entity lookups via TileIndex
- **Mitigation**: Consider caching door states in a parallel data structure (e.g., `HashMap<(u32, u32), bool>` updated on door changes)

### 2. Serialization Complexity

**Save File Growth**
- Each door adds entry to `door_states` HashMap
- **Mitigation**: Only store non-default states (open doors if default is closed)

**Migration**
- Adding new door fields requires save file migration
- **Mitigation**: Use optional fields with defaults, versioned save format

### 3. Map Generation Edge Cases

**Invalid Door Placement**
- Doors at map edges
- Doors blocking only path to areas
- Doors in water or on stairs
- **Mitigation**: Validation pass after placement; pathfinding check to ensure reachability

**Door Density**
- Too many doors = tedious gameplay
- Too few = no tactical value
- **Mitigation**: Configurable spawn rate per biome; minimum spacing rules

### 4. FOV System Complexity

**Incremental FOV + Doors**
- Current system optimizes by only recalculating changed regions
- Door toggle in previously calculated area requires recalculation
- **Mitigation**: Mark door toggles as forcing FOV recalculation; expand dirty region to include door visibility radius

**Symmetry Violations**
- Door state changes mid-turn could cause asymmetric visibility
- **Mitigation**: Process all door changes before FOV calculation in turn order

### 5. AI/Pathfinding Impact

**A* Pathfinding**
- Current autoexplore uses A* that checks `is_walkable()`
- Need to account for doors in pathfinding costs
- **Mitigation**: Closed doors = high cost (not impassable), open doors = normal floor cost

**Monster Pathfinding**
- When monsters exist, they need to handle doors
- **Mitigation**: Monster-type specific door behavior flags

### 6. Multiplayer Considerations (Future)

**State Synchronization**
- Door states need to be synced across clients
- Toggle events need conflict resolution
- **Mitigation**: Server-authoritative door state; optimistic client updates

---

## Testing Strategy

### Unit Tests
- Door component state transitions
- Walkability calculations with doors
- Sprite index selection
- Door placement validation

### Integration Tests
- Door persistence across level transitions
- FOV updates on door toggle
- Moving into closed door opens it and moves player onto tile
- O key opens adjacent closed door without moving player
- C key closes adjacent open door
- Player can walk through already-open doors normally
- Cannot close door while standing on it

### Visual/Manual Tests
- Door sprites render correctly
- Door toggle animation (if added)
- FOV shadow updates visually correct

---

## Implementation Checklist

### MVP (Phase 1)
- [ ] Add `Door` component
- [ ] Add `TileType::Door` variant
- [ ] Modify `handle_movement_input` to open doors on move-into
- [ ] Add O key binding for explicit open door
- [ ] Add C key binding for explicit close door
- [ ] Implement `handle_open_door` system
- [ ] Implement `handle_close_door` system
- [ ] Update FOV to check door states
- [ ] Add `door_states` to `SavedMapData`
- [ ] Implement door state persistence
- [ ] Add door sprite selection (274/275)
- [ ] Implement basic door placement algorithm
- [ ] Add door spawning during map generation
- [ ] System ordering for door systems
- [ ] Basic tests

### Future Phases
- [ ] Locked doors
- [ ] Secret doors
- [ ] Door durability
- [ ] Monster door interaction
- [ ] Advanced door types

---

## References

- [NetHack Wiki - Door](https://nethackwiki.com/wiki/Door)
- [Brogue Official Site](https://sites.google.com/site/broguegame)
- [BrogueCE Source Code](https://github.com/tmewett/BrogueCE/blob/master/src/brogue/Rogue.h)
- [Caves of Qud Wiki - Security Door](https://wiki.cavesofqud.com/wiki/Security_door)
- [Caves of Qud Wiki - Security Interlock](https://wiki.cavesofqud.com/wiki/Security_interlock)
- [Roguelike Vision Algorithms](https://www.adammil.net/blog/v125_Roguelike_Vision_Algorithms.html)
- [RogueBasin - Field of Vision](https://www.roguebasin.com/index.php/Field_of_Vision)
- [What the Hero Sees: FOV for Roguelikes](https://journal.stuffwithstuff.com/2015/09/07/what-the-hero-sees/)
