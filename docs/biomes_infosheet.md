Biomes, Asset Assignment, and TCOD Map Gen

## Environment Types and Allowed Assets

### 1. Caverns

**Description:** Natural underground caves with rough stone walls, frequent water features, and occasional crystal formations.

**Assets Allowed:**

```
0,0  1,0  0,1  1,1
0,6  1,6  2,6  3,6
1,7  2,7  3,7
1,8  2,8  3,8
```

### 2. Underglade

**Description:** Subterranean forest space with mossy ground, glowing flora, and primitive plant life.

**Assets Allowed:**

```
0,0  1,0  0,1  1,1
1,7  2,7  3,7
0,6
0,13 1,13 2,13 3,14
1,14 2,14 3,14
```

### 3. Fungal Deep

**Description:** Vast underground caverns dominated by giant mushrooms, spores, and fungal growths.

**Assets Allowed:**

```
1,14 2,14 3,14
1,13 2,13 3,14
0,13 0,15
```

### 4. Cinder Gaol

**Description:** Ancient prison complex with charred stone walls and abandoned cells, now home to malevolent spirits.

**Assets Allowed:**

```
0,2  1,2  2,2
0,3  1,3
0,5  1,5
```

### 5. Abyssal Hold

**Description:** Deep fortress carved into the abyss with alien geometry and occasional lakes of dark water.

**Assets Allowed:**

```
0,1  1,1
0,2  1,2  2,2
0,11 1,11 2,11 3,11
```

### 6. Nether Grange

**Description:** Underground communal farm populated by peaceful farmers, animals, and some docile monsters who have learned to co-habituate.

**Assets Allowed:**

```
0,0  1,0  0,1  1,1
0,4  1,4
0,6  1,6  2,6  3,6
1,7  2,7  3,7
1,8  2,8  3,8
0,13 1,13 2,13 3,13
1,14 2,14 3,14
```

### 7. Chthonic Crypts

**Description:** Ancient burial complex with forgotten tombs, dusty sarcophagi, and undead guardians.

**Assets Allowed:**

```
0,5  1,5
0,2  1,2
0,11 1,11 2,11 3,11
0,15 1,15 2,15 3,15
```

### 8. Hypogeal Knot

**Description:** Twisted labyrinth of impossibly interwoven tunnels that seem to defy spatial logic.

**Assets Allowed:**

```
0,1  1,1
0,2  1,2  2,2
0,4  1,4
0,11 1,11 2,11 3,11
```

### 9. Stygian Pool

**Description:** Flooded chambers surrounding a vast underground lake with strange aquatic creatures and ancient ruins.

**Assets Allowed:**

```
*Use tile filled with the color blue for water placeholder*
0,0  1,0
0,1  1,1
0,3  1,3
0,6  1,6  2,6  3,6
0,7  1,7  2,7  3,7
1,8  2,8  3,8
```

# TCOD MAP GENERATION ALGORITHMS

## Specialized Environment Algorithms

### 1. Labyrinth Mazes

**Algorithm Combination:** Modified Recursive Backtracking + Post-Processing

**Primary Algorithm:** Recursive backtracking or recursive division

**Enhancements:**

- Start with a grid-based structure for perfect mazes
- Use tcod's path functions to ensure connectivity
- Apply post-processing to add loops and alternative paths (breaking walls with specific probability)
- Add dead-ends and wider chambers at strategic points
- Use noise to slightly vary corridor widths for a less uniform appearance

**Implementation Strategy:** Generate a perfect maze first, then modify it by adding larger intersection rooms and occasionally breaking walls to create loops, making the maze navigable but still confusing.

### 2. Prison Blocks

**Algorithm Combination:** Partitioning + Template-Based Generation

**Primary Algorithm:** Grid-based cellular layout

**Enhancements:**

- Create a regular grid structure for cell blocks
- Use BSP for administration areas with larger rooms
- Apply templates for recurring elements (cells, guard posts)
- Use tcod's heightmap for slight floor variations in common areas
- Add connecting corridors using pathfinding algorithms

**Implementation Strategy:** Create a grid-based structure for cell blocks, then place predefined templates for cells. Generate larger BSP-based structures for administrative areas, and connect everything with corridors.

### 3. Forests with Tree Clusters and Winding Paths

**Algorithm Combination:** Noise-Based Density + Random Walk + Voronoi Clustering

**Primary Algorithm:** Noise-based terrain generation

**Enhancements:**

- Generate a noise map for tree density probability
- Use Voronoi diagrams to create distinct regions for tree clusters
- Apply drunkard's walk for natural-looking paths between clearings
- Add individual trees in lower-density areas using noise thresholds
- Use A* pathfinding with terrain weighting to create main paths

**Implementation Strategy:** Create a noise map determining tree density, use Voronoi diagrams for distinct forest regions with different characteristics, and finally use drunkard's walk algorithm with terrain awareness to create natural-looking paths.

### 4. Fortress with Structured Rooms, Corridors, and Bridges

**Algorithm Combination:** BSP + Custom Room Placement + Pathfinding

**Primary Algorithm:** Modified BSP for room layout

**Enhancements:**

- Use BSP for the main fortress structure and rooms
- Generate internal courtyards with noise-based terrain
- Create water features using lower thresholds on heightmaps
- Apply A* pathfinding with penalties to generate bridges over water
- Use prefabricated templates for specialized rooms (throne rooms, etc.)
- Create long corridors by merging BSP nodes strategically

**Implementation Strategy:** Begin with BSP for the main structure, create water features with heightmaps, and connect separate sections with bridges using A* pathfinding with special weights.

### 5. Farm Lands with Village Inside a Cavern

**Algorithm Combination:** Cellular Automata + BSP + River Generation + Grid Overlays

**Primary Algorithm:** Cellular automata for the main cavern

**Enhancements:**

- Use cellular automata for the large open cavern structure
- Generate a river using noise-based path with erosion simulation
- Apply BSP or room-based generation for the village buildings
- Create grid overlays for crop fields in suitable flat areas
- Use pathfinding to create natural walking paths between points of interest
- Add tributaries from the main river using modified random walk

**Implementation Strategy:** Generate the cavern first, then create a river through it using noise-based pathfinding. Place a village using BSP in a suitable area, and overlay grid patterns for crop fields in flat areas away from the river.

### 6. Crypts with Linear Corridors and Open Arenas

**Algorithm Combination:** Linear Corridor Generator + Room Templates + BSP

**Primary Algorithm:** Linear corridor generation with room placement

**Enhancements:**

- Create a primary spine of linear corridors
- Use BSP to generate branching side passages
- Place predefined template rooms for special crypts and tombs
- Generate larger arena spaces at corridor intersections
- Apply noise for floor detail and minor elevation changes
- Use cellular automata for partially collapsed sections

**Implementation Strategy:** Create a backbone of linear corridors first, then place large arena rooms at key junctions. Add smaller tomb chambers along the corridors using templates or BSP subdivision.

### 7. Vast Lake Inside a Cavern with Tributaries and Bridges

**Algorithm Combination:** Heightmap Basin + River Generation + Bridge Placement

**Primary Algorithm:** Heightmap manipulation for basin creation

**Enhancements:**

- Generate a large cavern using cellular automata
- Create a basin in the center using heightmap depressions
- Use noise-based flow algorithms to create tributaries flowing into the lake
- Apply erosion simulation to make natural-looking shorelines
- Generate islands using noise thresholds
- Place bridges at optimal crossing points using pathfinding and visibility analysis
- Add small structures near shorelines using template placement

**Implementation Strategy:** Create a large cavern, then manipulate a heightmap to form a basin for the lake. Generate tributaries flowing into the lake using noise-based path generation, and place bridges at strategic narrow points.

### 8. Vast Caverns with Fortified Structures and Secret Tunnels

**Algorithm Combination:** Cellular Automata + BSP + Pathfinding + Alcove Detection

**Primary Algorithm:** Cellular automata for cavern generation

**Enhancements:**

- Create large, irregular caverns with cellular automata
- Use edge detection to identify suitable alcoves for nestled structures
- Apply BSP or template-based generation for fort/castle structures
- Generate secret tunnels using A* pathfinding with high wall penetration costs
- Add structural details like walls and towers to fort interiors
- Create hidden entrances by manipulating specific wall cells
- Use distance fields to ensure structures are properly positioned relative to cavern walls

**Implementation Strategy:** Generate the cavern system first, identify suitable alcoves and open areas, place fortress structures in these locations, and finally create hidden tunnels between structures using weighted pathfinding that can penetrate through cave walls at strategic points.