// Compact Organic Map Generator - separate file due to size
use std::collections::HashSet;
use rand::Rng;
use crate::components::TileType;
use crate::map_generation::{MapGenerator, MapGenParams, flatten_tiles};

pub struct CompactOrganicGenerator;

impl MapGenerator for CompactOrganicGenerator {
    fn generate(&mut self, width: u32, height: u32, params: &MapGenParams) -> Vec<TileType> {
        let mut tiles = vec![vec![TileType::Wall; width as usize]; height as usize];
        let mut rng = rand::rng();

        // Step 1: Generate organic outer boundary (20x20 constraint)
        let boundary = self.generate_organic_boundary(width, height, &mut rng);

        // Step 2: Fill boundary with floors
        self.fill_boundary(&mut tiles, &boundary, width, height);

        // Step 3: Create interior wall divisions
        let divisions = self.create_interior_divisions(&boundary, params, &mut rng);
        self.apply_wall_divisions(&mut tiles, &divisions, width, height);

        // Step 4: Punch doorways through walls
        self.create_doorways(&mut tiles, &divisions, width, height, &mut rng);

        // Step 5: Ensure connectivity
        self.ensure_all_rooms_connected(&mut tiles, width, height);

        flatten_tiles(tiles, width, height)
    }
}

impl CompactOrganicGenerator {
    // Generate an organic blob shape using cellular automata growth
    fn generate_organic_boundary(&self, width: u32, height: u32, rng: &mut impl Rng) -> Vec<(u32, u32)> {
        let center_x = width / 2;
        let center_y = height / 2;

        // Start with a seed point
        let mut active = HashSet::new();
        active.insert((center_x, center_y));

        // Target size (20x20 = 400 tiles, but organic so ~300-350)
        let target_size = rng.random_range(300..400);

        // Grow the blob organically
        while active.len() < target_size {
            let mut candidates = Vec::new();

            // Find all positions adjacent to active tiles
            for &(x, y) in &active {
                for (dx, dy) in &[(0, 1), (1, 0), (0, -1), (-1, 0)] {
                    let nx = (x as i32 + dx) as u32;
                    let ny = (y as i32 + dy) as u32;

                    if nx > 0 && nx < width - 1 && ny > 0 && ny < height - 1 {
                        if !active.contains(&(nx, ny)) {
                            candidates.push((nx, ny));
                        }
                    }
                }
            }

            if candidates.is_empty() {
                break;
            }

            // Add random candidate with bias toward keeping shape compact
            let pick_idx = rng.random_range(0..candidates.len().min(8));
            let new_pos = candidates[pick_idx];

            // Bias toward circular/organic shapes
            let dist_from_center = ((new_pos.0 as f32 - center_x as f32).powi(2) +
                                   (new_pos.1 as f32 - center_y as f32).powi(2)).sqrt();

            if dist_from_center < 12.0 || rng.random::<f32>() < 0.7 {
                active.insert(new_pos);
            }
        }

        active.into_iter().collect()
    }

    // Fill the boundary with floor tiles
    fn fill_boundary(&self, tiles: &mut Vec<Vec<TileType>>, boundary: &[(u32, u32)], width: u32, height: u32) {
        for &(x, y) in boundary {
            if x < width && y < height {
                tiles[y as usize][x as usize] = TileType::Floor;
            }
        }
    }

    // Create interior wall divisions using recursive slicing
    fn create_interior_divisions(&self, boundary: &[(u32, u32)], params: &MapGenParams, rng: &mut impl Rng) -> Vec<WallDivision> {
        let mut divisions = Vec::new();

        // Find bounding box of boundary
        let min_x = boundary.iter().map(|(x, _)| *x).min().unwrap_or(0);
        let max_x = boundary.iter().map(|(x, _)| *x).max().unwrap_or(0);
        let min_y = boundary.iter().map(|(_, y)| *y).min().unwrap_or(0);
        let max_y = boundary.iter().map(|(_, y)| *y).max().unwrap_or(0);

        let bbox = BoundingBox {
            x: min_x,
            y: min_y,
            width: max_x - min_x,
            height: max_y - min_y,
        };

        // Create 2-4 wall divisions
        let num_divisions = rng.random_range(2..=4).min(params.max_rooms as i32) as usize;

        for _ in 0..num_divisions {
            let is_horizontal = rng.random::<bool>();

            if is_horizontal {
                // Horizontal wall
                let y = rng.random_range(bbox.y + 3..bbox.y + bbox.height - 3);
                let start_x = bbox.x;
                let end_x = bbox.x + bbox.width;

                divisions.push(WallDivision {
                    start: (start_x, y),
                    end: (end_x, y),
                    is_horizontal,
                });
            } else {
                // Vertical wall
                let x = rng.random_range(bbox.x + 3..bbox.x + bbox.width - 3);
                let start_y = bbox.y;
                let end_y = bbox.y + bbox.height;

                divisions.push(WallDivision {
                    start: (x, start_y),
                    end: (x, end_y),
                    is_horizontal,
                });
            }
        }

        divisions
    }

    // Apply wall divisions to the tile map
    fn apply_wall_divisions(&self, tiles: &mut Vec<Vec<TileType>>, divisions: &[WallDivision], width: u32, height: u32) {
        for division in divisions {
            if division.is_horizontal {
                let y = division.start.1;
                for x in division.start.0..=division.end.0 {
                    if x < width && y < height && tiles[y as usize][x as usize] == TileType::Floor {
                        tiles[y as usize][x as usize] = TileType::Wall;
                    }
                }
            } else {
                let x = division.start.0;
                for y in division.start.1..=division.end.1 {
                    if x < width && y < height && tiles[y as usize][x as usize] == TileType::Floor {
                        tiles[y as usize][x as usize] = TileType::Wall;
                    }
                }
            }
        }
    }

    // Punch doorways (1-3 tiles) through each wall division
    fn create_doorways(&self, tiles: &mut Vec<Vec<TileType>>, divisions: &[WallDivision],
                      width: u32, height: u32, rng: &mut impl Rng) {
        for division in divisions {
            // Create 1-2 doorways per division
            let num_doorways = rng.random_range(1..=2);

            for _ in 0..num_doorways {
                let doorway_width = rng.random_range(1..=3);

                if division.is_horizontal {
                    let y = division.start.1;
                    let wall_length = division.end.0 - division.start.0;
                    let doorway_x = division.start.0 + rng.random_range(2..wall_length.saturating_sub(doorway_width + 2));

                    // Punch out the doorway
                    for dx in 0..doorway_width {
                        let x = doorway_x + dx;
                        if x < width && y < height {
                            tiles[y as usize][x as usize] = TileType::Floor;
                        }
                    }
                } else {
                    let x = division.start.0;
                    let wall_length = division.end.1 - division.start.1;
                    let doorway_y = division.start.1 + rng.random_range(2..wall_length.saturating_sub(doorway_width + 2));

                    // Punch out the doorway
                    for dy in 0..doorway_width {
                        let y = doorway_y + dy;
                        if x < width && y < height {
                            tiles[y as usize][x as usize] = TileType::Floor;
                        }
                    }
                }
            }
        }
    }

    // Ensure all floor regions are connected
    fn ensure_all_rooms_connected(&self, tiles: &mut Vec<Vec<TileType>>, width: u32, height: u32) {
        let regions = self.find_disconnected_regions(tiles, width, height);

        if regions.len() <= 1 {
            return;
        }

        // Connect each region to the next
        for i in 0..regions.len() - 1 {
            let pos1 = regions[i][0];
            let pos2 = regions[i + 1][0];

            // Create a simple corridor
            self.create_simple_corridor(tiles, pos1, pos2, width, height);
        }
    }

    fn find_disconnected_regions(&self, tiles: &Vec<Vec<TileType>>, width: u32, height: u32) -> Vec<Vec<(u32, u32)>> {
        let mut visited = vec![vec![false; width as usize]; height as usize];
        let mut regions = Vec::new();

        for y in 0..height {
            for x in 0..width {
                if tiles[y as usize][x as usize] == TileType::Floor && !visited[y as usize][x as usize] {
                    let region = self.flood_fill(tiles, &mut visited, x, y, width, height);
                    if !region.is_empty() {
                        regions.push(region);
                    }
                }
            }
        }

        regions
    }

    fn flood_fill(&self, tiles: &Vec<Vec<TileType>>, visited: &mut Vec<Vec<bool>>,
                 start_x: u32, start_y: u32, width: u32, height: u32) -> Vec<(u32, u32)> {
        let mut region = Vec::new();
        let mut stack = vec![(start_x, start_y)];

        while let Some((x, y)) = stack.pop() {
            if visited[y as usize][x as usize] {
                continue;
            }

            visited[y as usize][x as usize] = true;
            region.push((x, y));

            // Check neighbors
            for (dx, dy) in &[(0, 1), (1, 0), (0, -1), (-1, 0)] {
                let nx = (x as i32 + dx) as u32;
                let ny = (y as i32 + dy) as u32;

                if nx < width && ny < height {
                    if tiles[ny as usize][nx as usize] == TileType::Floor && !visited[ny as usize][nx as usize] {
                        stack.push((nx, ny));
                    }
                }
            }
        }

        region
    }

    fn create_simple_corridor(&self, tiles: &mut Vec<Vec<TileType>>,
                             start: (u32, u32), end: (u32, u32), width: u32, height: u32) {
        let mut x = start.0 as i32;
        let mut y = start.1 as i32;
        let target_x = end.0 as i32;
        let target_y = end.1 as i32;

        // Move horizontally first
        while x != target_x {
            if x < target_x {
                x += 1;
            } else {
                x -= 1;
            }

            if x >= 0 && x < width as i32 && y >= 0 && y < height as i32 {
                tiles[y as usize][x as usize] = TileType::Floor;
            }
        }

        // Then vertically
        while y != target_y {
            if y < target_y {
                y += 1;
            } else {
                y -= 1;
            }

            if x >= 0 && x < width as i32 && y >= 0 && y < height as i32 {
                tiles[y as usize][x as usize] = TileType::Floor;
            }
        }
    }
}

#[derive(Clone, Debug)]
struct WallDivision {
    start: (u32, u32),
    end: (u32, u32),
    is_horizontal: bool,
}

#[derive(Clone, Debug)]
struct BoundingBox {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}
