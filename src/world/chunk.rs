pub const CHUNK_WIDTH: usize = 512;
pub const CHUNK_DEPTH: usize = 512;
pub const CHUNK_HEIGHT: usize = 64;

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
}

pub struct Chunk {
    blocks: Vec<Block>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            blocks: vec![Block::Air; CHUNK_WIDTH * CHUNK_HEIGHT * CHUNK_DEPTH],
        }
    }

    fn index(x: usize, y: usize, z: usize) -> usize {
        x + z * CHUNK_WIDTH + y * CHUNK_WIDTH * CHUNK_DEPTH
    }

    pub fn generate(&mut self) {
        for x in 0..CHUNK_WIDTH {
            for z in 0..CHUNK_DEPTH {
                let n = ((x as f32 * 0.03).sin() + (z as f32 * 0.04).cos()) * 5.5
                    + ((x as f32 * 0.011 + z as f32 * 0.017).sin()) * 6.5;
                let base_height = (18.0 + n).clamp(8.0, 42.0) as usize;

                for y in 0..base_height {
                    let block = if y + 5 < base_height {
                        Block::Stone
                    } else if y == base_height - 1 {
                        if base_height < 14 {
                            Block::Sand
                        } else if base_height > 36 {
                            Block::Snow
                        } else {
                            Block::Grass
                        }
                    } else {
                        Block::Dirt
                    };
                    self.set(x, y, z, block);
                }
            }
        }

        self.add_house(40, 20, 42, 12, 9, 10);
        self.add_house(180, 24, 120, 14, 10, 12);
        self.add_house(340, 19, 300, 11, 8, 11);
        self.add_house(420, 22, 410, 16, 9, 13);

        self.add_tree(64, 64);
        self.add_tree(128, 220);
        self.add_tree(258, 144);
        self.add_tree(380, 333);
        self.add_tree(470, 180);

        self.add_target(250, 26, 256);
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
                        self.set(x, y, z, Block::Wood);
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

        let mut ground = 0;
        for y in (0..CHUNK_HEIGHT).rev() {
            if self.get_i32(x as i32, y as i32, z as i32) != Block::Air {
                ground = y + 1;
                break;
            }
        }

        let trunk_h = 6;
        for y in ground..(ground + trunk_h).min(CHUNK_HEIGHT) {
            self.set(x, y, z, Block::Wood);
        }

        let top = (ground + trunk_h).min(CHUNK_HEIGHT - 1);
        for lx in x.saturating_sub(2)..=(x + 2).min(CHUNK_WIDTH - 1) {
            for lz in z.saturating_sub(2)..=(z + 2).min(CHUNK_DEPTH - 1) {
                for ly in top.saturating_sub(1)..=(top + 1).min(CHUNK_HEIGHT - 1) {
                    if self.get_i32(lx as i32, ly as i32, lz as i32) == Block::Air {
                        self.set(lx, ly, lz, Block::Leaf);
                    }
                }
            }
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
            self.blocks[idx] = block;
        }
    }
}
