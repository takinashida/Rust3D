pub const CHUNK_WIDTH: usize = 512;
pub const CHUNK_DEPTH: usize = 512;
pub const CHUNK_HEIGHT: usize = 96;
pub const CHUNK_REGION_SIZE: usize = 16;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Block {
    Air,
    Grass,
    Dirt,
    Stone,
    Sand,
    Wood,
    Planks,
    Bricks,
    Glass,
    Leaf,
    Snow,
    Target,
    Water,
    Mud,
    Cobblestone,
    CoalOre,
    IronOre,
    GoldOre,
    DiamondOre,
    Gravel,
    Clay,
    Basalt,
    Moss,
    RedSand,
    Ice,
    Cactus,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Biome {
    Plains,
    Forest,
    Desert,
    Snow,
    Mountains,
    Swamp,
}

pub struct Chunk {
    blocks: Vec<Block>,
    dirty_regions: Vec<bool>,
}

impl Chunk {
    pub fn new() -> Self {
        let regions_x = CHUNK_WIDTH / CHUNK_REGION_SIZE;
        let regions_y = CHUNK_HEIGHT / CHUNK_REGION_SIZE;
        let regions_z = CHUNK_DEPTH / CHUNK_REGION_SIZE;
        Self {
            blocks: vec![Block::Air; CHUNK_WIDTH * CHUNK_HEIGHT * CHUNK_DEPTH],
            dirty_regions: vec![true; regions_x * regions_y * regions_z],
        }
    }

    fn index(x: usize, y: usize, z: usize) -> usize {
        x + z * CHUNK_WIDTH + y * CHUNK_WIDTH * CHUNK_DEPTH
    }

    fn region_dims() -> (usize, usize, usize) {
        (
            CHUNK_WIDTH / CHUNK_REGION_SIZE,
            CHUNK_HEIGHT / CHUNK_REGION_SIZE,
            CHUNK_DEPTH / CHUNK_REGION_SIZE,
        )
    }

    fn region_index(rx: usize, ry: usize, rz: usize) -> usize {
        let (regions_x, _, regions_z) = Self::region_dims();
        rx + rz * regions_x + ry * regions_x * regions_z
    }

    fn mark_region_dirty_by_block(&mut self, x: usize, y: usize, z: usize) {
        let (regions_x, regions_y, regions_z) = Self::region_dims();
        let rx = x / CHUNK_REGION_SIZE;
        let ry = y / CHUNK_REGION_SIZE;
        let rz = z / CHUNK_REGION_SIZE;

        for dx in -1..=1 {
            for dy in -1..=1 {
                for dz in -1..=1 {
                    let nrx = rx as i32 + dx;
                    let nry = ry as i32 + dy;
                    let nrz = rz as i32 + dz;
                    if nrx < 0
                        || nry < 0
                        || nrz < 0
                        || nrx >= regions_x as i32
                        || nry >= regions_y as i32
                        || nrz >= regions_z as i32
                    {
                        continue;
                    }
                    let idx = Self::region_index(nrx as usize, nry as usize, nrz as usize);
                    self.dirty_regions[idx] = true;
                }
            }
        }
    }

    pub fn take_dirty_regions(&mut self) -> Vec<[usize; 3]> {
        let (regions_x, regions_y, regions_z) = Self::region_dims();
        let mut out = Vec::new();
        for ry in 0..regions_y {
            for rz in 0..regions_z {
                for rx in 0..regions_x {
                    let idx = Self::region_index(rx, ry, rz);
                    if self.dirty_regions[idx] {
                        self.dirty_regions[idx] = false;
                        out.push([rx, ry, rz]);
                    }
                }
            }
        }
        out
    }

    pub fn generate(&mut self) {
        const SEA_LEVEL: usize = 30;

        for x in 0..CHUNK_WIDTH {
            for z in 0..CHUNK_DEPTH {
                let biome = biome_at(x, z);
                let h = terrain_height(x, z, biome).clamp(8, CHUNK_HEIGHT.saturating_sub(2));

                for y in 0..=h {
                    let depth = h.saturating_sub(y);
                    let block = match biome {
                        Biome::Desert => {
                            if depth == 0 {
                                Block::Sand
                            } else if depth < 4 {
                                Block::RedSand
                            } else {
                                stone_with_ores(x, y, z)
                            }
                        }
                        Biome::Snow => {
                            if depth == 0 {
                                Block::Snow
                            } else if depth < 3 {
                                Block::Dirt
                            } else {
                                stone_with_ores(x, y, z)
                            }
                        }
                        Biome::Swamp => {
                            if depth == 0 {
                                Block::Moss
                            } else if depth < 4 {
                                Block::Mud
                            } else {
                                stone_with_ores(x, y, z)
                            }
                        }
                        Biome::Mountains => {
                            if depth == 0 && h > 52 {
                                Block::Snow
                            } else {
                                stone_with_ores(x, y, z)
                            }
                        }
                        Biome::Forest => {
                            if depth == 0 {
                                Block::Grass
                            } else if depth < 4 {
                                Block::Dirt
                            } else {
                                stone_with_ores(x, y, z)
                            }
                        }
                        Biome::Plains => {
                            if depth == 0 {
                                Block::Grass
                            } else if depth < 3 {
                                Block::Dirt
                            } else {
                                stone_with_ores(x, y, z)
                            }
                        }
                    };
                    self.set(x, y, z, block);
                }

                if h < SEA_LEVEL {
                    for y in (h + 1)..=SEA_LEVEL.min(CHUNK_HEIGHT - 1) {
                        self.set(
                            x,
                            y,
                            z,
                            if biome == Biome::Snow {
                                Block::Ice
                            } else {
                                Block::Water
                            },
                        );
                    }
                }
            }
        }

        for x in (10..CHUNK_WIDTH - 10).step_by(24) {
            for z in (10..CHUNK_DEPTH - 10).step_by(24) {
                match biome_at(x, z) {
                    Biome::Forest => self.add_tree(x, z),
                    Biome::Desert => self.add_cactus(x, z),
                    _ => {}
                }
            }
        }

        self.add_house(40, 28, 42, 12, 9, 10);
        self.add_house(180, 30, 120, 14, 10, 12);
        self.add_house(340, 32, 300, 11, 8, 11);
        self.add_house(420, 34, 410, 16, 9, 13);
        self.add_target(250, 36, 256);
    }

    fn add_house(&mut self, x0: usize, y0: usize, z0: usize, w: usize, h: usize, d: usize) {
        for x in x0..(x0 + w).min(CHUNK_WIDTH - 1) {
            for z in z0..(z0 + d).min(CHUNK_DEPTH - 1) {
                for y in y0..(y0 + h).min(CHUNK_HEIGHT - 1) {
                    let wall = x == x0 || x == x0 + w - 1 || z == z0 || z == z0 + d - 1;
                    let roof = y == y0 + h - 1;
                    let floor = y == y0;

                    if roof {
                        self.set(x, y, z, Block::Planks);
                    } else if floor {
                        self.set(x, y, z, Block::Cobblestone);
                    } else if wall {
                        let is_window = y > y0 + 1
                            && y < y0 + h - 2
                            && ((x == x0 || x == x0 + w - 1) && z == z0 + d / 2
                                || (z == z0 || z == z0 + d - 1) && x == x0 + w / 2);
                        self.set(
                            x,
                            y,
                            z,
                            if is_window {
                                Block::Glass
                            } else {
                                Block::Bricks
                            },
                        );
                    } else {
                        self.set(x, y, z, Block::Air);
                    }
                }
            }
        }

        let door_x = x0 + w / 2;
        self.set(door_x, y0 + 1, z0, Block::Air);
        self.set(door_x, y0 + 2, z0, Block::Air);
    }

    fn add_tree(&mut self, x: usize, z: usize) {
        if x >= CHUNK_WIDTH || z >= CHUNK_DEPTH {
            return;
        }

        let mut ground = None;
        for y in (0..CHUNK_HEIGHT).rev() {
            let block = self.get_i32(x as i32, y as i32, z as i32);
            if block != Block::Air && block != Block::Water {
                ground = Some(y + 1);
                break;
            }
        }

        let Some(ground) = ground else {
            return;
        };

        let trunk_h = 5 + ((x + z) % 3);
        for y in ground..(ground + trunk_h).min(CHUNK_HEIGHT) {
            self.set(x, y, z, Block::Wood);
        }

        let top = (ground + trunk_h).min(CHUNK_HEIGHT - 1);
        for lx in x.saturating_sub(2)..=(x + 2).min(CHUNK_WIDTH - 1) {
            for lz in z.saturating_sub(2)..=(z + 2).min(CHUNK_DEPTH - 1) {
                for ly in top.saturating_sub(2)..=(top + 1).min(CHUNK_HEIGHT - 1) {
                    if self.get_i32(lx as i32, ly as i32, lz as i32) == Block::Air {
                        self.set(lx, ly, lz, Block::Leaf);
                    }
                }
            }
        }
    }

    fn add_cactus(&mut self, x: usize, z: usize) {
        if x >= CHUNK_WIDTH || z >= CHUNK_DEPTH {
            return;
        }
        let mut ground = None;
        for y in (0..CHUNK_HEIGHT).rev() {
            let block = self.get_i32(x as i32, y as i32, z as i32);
            if block == Block::Sand || block == Block::RedSand {
                ground = Some(y + 1);
                break;
            }
        }
        let Some(ground) = ground else {
            return;
        };

        let h = 2 + ((x * 17 + z * 11) % 3);
        for y in ground..(ground + h).min(CHUNK_HEIGHT) {
            self.set(x, y, z, Block::Cactus);
        }
    }

    fn add_target(&mut self, center_x: usize, center_y: usize, center_z: usize) {
        if center_x < 2 || center_y < 2 || center_z < 2 {
            return;
        }

        let x0 = center_x - 2;
        let y0 = center_y - 2;

        for x in x0..=(center_x + 2).min(CHUNK_WIDTH - 1) {
            for y in y0..=(center_y + 2).min(CHUNK_HEIGHT - 1) {
                self.set(x, y, center_z.min(CHUNK_DEPTH - 1), Block::Target);
            }
        }
    }

    pub fn get_i32(&self, x: i32, y: i32, z: i32) -> Block {
        if x < 0 || y < 0 || z < 0 {
            return Block::Air;
        }
        let (x, y, z) = (x as usize, y as usize, z as usize);
        if x >= CHUNK_WIDTH || y >= CHUNK_HEIGHT || z >= CHUNK_DEPTH {
            Block::Air
        } else {
            self.blocks[Self::index(x, y, z)]
        }
    }

    pub fn set(&mut self, x: usize, y: usize, z: usize, block: Block) {
        if x < CHUNK_WIDTH && y < CHUNK_HEIGHT && z < CHUNK_DEPTH {
            let idx = Self::index(x, y, z);
            if self.blocks[idx] != block {
                self.blocks[idx] = block;
                self.mark_region_dirty_by_block(x, y, z);
            }
        }
    }
}

fn noise(x: f32, z: f32, scale: f32, x_mul: f32, z_mul: f32) -> f32 {
    ((x * scale * x_mul).sin() + (z * scale * z_mul).cos()) * 0.5
}

fn biome_at(x: usize, z: usize) -> Biome {
    let xf = x as f32;
    let zf = z as f32;
    let heat = noise(xf, zf, 0.017, 1.3, 1.1) + noise(xf, zf, 0.041, 1.0, 0.9) * 0.4;
    let moisture = noise(xf, zf, 0.02, 0.7, 1.7) + noise(xf, zf, 0.052, 1.6, 0.8) * 0.5;

    if heat > 0.62 {
        Biome::Desert
    } else if heat < -0.45 {
        Biome::Snow
    } else if moisture > 0.65 {
        Biome::Swamp
    } else if moisture < -0.35 {
        Biome::Mountains
    } else if moisture > 0.2 {
        Biome::Forest
    } else {
        Biome::Plains
    }
}

fn terrain_height(x: usize, z: usize, biome: Biome) -> usize {
    let xf = x as f32;
    let zf = z as f32;
    let macro_shape = noise(xf, zf, 0.011, 1.0, 1.0) * 9.0;
    let detail = noise(xf, zf, 0.042, 1.2, 0.8) * 3.5;
    let ridge = noise(xf, zf, 0.083, 0.9, 1.4).abs() * 5.5;

    let base = match biome {
        Biome::Plains => 28.0 + macro_shape * 0.7 + detail,
        Biome::Forest => 30.0 + macro_shape * 0.8 + detail,
        Biome::Desert => 24.0 + macro_shape * 0.5 + detail * 0.6,
        Biome::Snow => 34.0 + macro_shape * 0.8 + ridge,
        Biome::Mountains => 40.0 + macro_shape * 1.4 + ridge * 1.3,
        Biome::Swamp => 22.0 + macro_shape * 0.4,
    };

    base.clamp(8.0, (CHUNK_HEIGHT - 2) as f32) as usize
}

fn stone_with_ores(x: usize, y: usize, z: usize) -> Block {
    if y < 10 && (x + z + y) % 43 == 0 {
        Block::DiamondOre
    } else if y < 20 && (x * 3 + z * 5 + y) % 37 == 0 {
        Block::GoldOre
    } else if y < 34 && (x * 7 + z * 11 + y * 2) % 29 == 0 {
        Block::IronOre
    } else if y < 40 && (x * 13 + z * 17 + y * 3) % 19 == 0 {
        Block::CoalOre
    } else if y < 14 {
        Block::Basalt
    } else if y < 26 && (x + z) % 9 == 0 {
        Block::Gravel
    } else if y < 28 && (x * 5 + z * 7) % 21 == 0 {
        Block::Clay
    } else {
        Block::Stone
    }
}
