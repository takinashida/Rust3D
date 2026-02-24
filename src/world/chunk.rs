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
    origin_x: i32,
    origin_z: i32,
}

impl Chunk {
    pub fn new() -> Self {
        let regions_x = CHUNK_WIDTH / CHUNK_REGION_SIZE;
        let regions_y = CHUNK_HEIGHT / CHUNK_REGION_SIZE;
        let regions_z = CHUNK_DEPTH / CHUNK_REGION_SIZE;
        Self {
            blocks: vec![Block::Air; CHUNK_WIDTH * CHUNK_HEIGHT * CHUNK_DEPTH],
            dirty_regions: vec![true; regions_x * regions_y * regions_z],
            origin_x: -(CHUNK_WIDTH as i32) / 2,
            origin_z: -(CHUNK_DEPTH as i32) / 2,
        }
    }

    pub fn origin(&self) -> (i32, i32) {
        (self.origin_x, self.origin_z)
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
        self.generate_at(self.origin_x, self.origin_z);
    }

    pub fn generate_at(&mut self, origin_x: i32, origin_z: i32) {
        const SEA_LEVEL: usize = 30;
        self.origin_x = origin_x;
        self.origin_z = origin_z;
        use std::sync::{
            atomic::{AtomicUsize, Ordering},
            Arc,
        };
        use std::thread;

        let worker_count = thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1)
            .min(CHUNK_WIDTH.max(1));
        let next_x = Arc::new(AtomicUsize::new(0));

        let mut slabs = vec![vec![Block::Air; CHUNK_HEIGHT * CHUNK_DEPTH]; CHUNK_WIDTH];
        thread::scope(|scope| {
            let mut handles = Vec::with_capacity(worker_count);
            for _ in 0..worker_count {
                let next_x = Arc::clone(&next_x);
                handles.push(scope.spawn(move || {
                    let mut produced = Vec::new();
                    loop {
                        let x = next_x.fetch_add(1, Ordering::Relaxed);
                        if x >= CHUNK_WIDTH {
                            break;
                        }
                        let wx = origin_x + x as i32;
                        let mut slab = vec![Block::Air; CHUNK_HEIGHT * CHUNK_DEPTH];
                        for z in 0..CHUNK_DEPTH {
                            let wz = origin_z + z as i32;
                            let biome = biome_at(wx, wz);
                            let h = terrain_height(wx, wz, biome)
                                .clamp(8, CHUNK_HEIGHT.saturating_sub(2));
                            for y in 0..CHUNK_HEIGHT {
                                let idx = z * CHUNK_HEIGHT + y;
                                slab[idx] = terrain_block_at(wx, y, wz, biome, h, SEA_LEVEL);
                            }
                        }
                        produced.push((x, slab));
                    }
                    produced
                }));
            }

            for handle in handles {
                for (x, slab) in handle.join().expect("terrain worker panicked") {
                    slabs[x] = slab;
                }
            }
        });

        for (x, slab) in slabs.into_iter().enumerate() {
            let start = x * CHUNK_HEIGHT * CHUNK_DEPTH;
            let end = start + CHUNK_HEIGHT * CHUNK_DEPTH;
            self.blocks[start..end].copy_from_slice(&slab);
        }

        for x in (10..CHUNK_WIDTH - 10).step_by(24) {
            for z in (10..CHUNK_DEPTH - 10).step_by(24) {
                match biome_at(origin_x + x as i32, origin_z + z as i32) {
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
        self.dirty_regions.fill(true);
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

        let wx = self.origin_x + x as i32;
        let wz = self.origin_z + z as i32;
        let mut ground = None;
        for y in (0..CHUNK_HEIGHT).rev() {
            let block = self.get_i32(wx, y as i32, wz);
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
                    if self.get_i32(
                        self.origin_x + lx as i32,
                        ly as i32,
                        self.origin_z + lz as i32,
                    ) == Block::Air
                    {
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
        let wx = self.origin_x + x as i32;
        let wz = self.origin_z + z as i32;
        let mut ground = None;
        for y in (0..CHUNK_HEIGHT).rev() {
            let block = self.get_i32(wx, y as i32, wz);
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
        if y < 0 || y >= CHUNK_HEIGHT as i32 {
            return Block::Air;
        }
        let lx = x - self.origin_x;
        let lz = z - self.origin_z;
        if lx >= 0 && lz >= 0 && lx < CHUNK_WIDTH as i32 && lz < CHUNK_DEPTH as i32 {
            self.blocks[Self::index(lx as usize, y as usize, lz as usize)]
        } else {
            procedural_block_at(x, y as usize, z)
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

    pub fn set_i32(&mut self, x: i32, y: i32, z: i32, block: Block) {
        if y < 0 || y >= CHUNK_HEIGHT as i32 {
            return;
        }
        let lx = x - self.origin_x;
        let lz = z - self.origin_z;
        if lx >= 0 && lz >= 0 && lx < CHUNK_WIDTH as i32 && lz < CHUNK_DEPTH as i32 {
            self.set(lx as usize, y as usize, lz as usize, block);
        }
    }
}

fn noise(x: f32, z: f32, scale: f32, x_mul: f32, z_mul: f32) -> f32 {
    ((x * scale * x_mul).sin() + (z * scale * z_mul).cos()) * 0.5
}

fn biome_at(x: i32, z: i32) -> Biome {
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

fn terrain_height(x: i32, z: i32, biome: Biome) -> usize {
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

fn stone_with_ores(x: i32, y: usize, z: i32) -> Block {
    let h = ((x as i64).wrapping_mul(73_856_093) ^ (z as i64).wrapping_mul(19_349_663)).abs();
    let salt = (h % 65_537) as usize;
    let yi = y as i32;
    if y < 10 && (x + z + yi) % 43 == 0 {
        Block::DiamondOre
    } else if y < 20 && (x * 3 + z * 5 + yi) % 37 == 0 {
        Block::GoldOre
    } else if y < 34 && (x * 7 + z * 11 + yi * 2) % 29 == 0 {
        Block::IronOre
    } else if y < 40 && (x * 13 + z * 17 + yi * 3) % 19 == 0 {
        Block::CoalOre
    } else if y < 14 {
        Block::Basalt
    } else if y < 26 && (salt + y) % 9 == 0 {
        Block::Gravel
    } else if y < 28 && (salt + y * 3) % 21 == 0 {
        Block::Clay
    } else {
        Block::Stone
    }
}

fn terrain_block_at(wx: i32, y: usize, wz: i32, biome: Biome, h: usize, sea_level: usize) -> Block {
    if y > h {
        if y <= sea_level {
            return if biome == Biome::Snow {
                Block::Ice
            } else {
                Block::Water
            };
        }
        return Block::Air;
    }
    let depth = h.saturating_sub(y);
    match biome {
        Biome::Desert => {
            if depth == 0 {
                Block::Sand
            } else if depth < 4 {
                Block::RedSand
            } else {
                stone_with_ores(wx, y, wz)
            }
        }
        Biome::Snow => {
            if depth == 0 {
                Block::Snow
            } else if depth < 3 {
                Block::Dirt
            } else {
                stone_with_ores(wx, y, wz)
            }
        }
        Biome::Swamp => {
            if depth == 0 {
                Block::Moss
            } else if depth < 4 {
                Block::Mud
            } else {
                stone_with_ores(wx, y, wz)
            }
        }
        Biome::Mountains => {
            if depth == 0 && h > 52 {
                Block::Snow
            } else {
                stone_with_ores(wx, y, wz)
            }
        }
        Biome::Forest | Biome::Plains => {
            if depth == 0 {
                Block::Grass
            } else if depth < 4 {
                Block::Dirt
            } else {
                stone_with_ores(wx, y, wz)
            }
        }
    }
}

fn procedural_block_at(x: i32, y: usize, z: i32) -> Block {
    let biome = biome_at(x, z);
    let h = terrain_height(x, z, biome).clamp(8, CHUNK_HEIGHT.saturating_sub(2));
    terrain_block_at(x, y, z, biome, h, 30)
}
