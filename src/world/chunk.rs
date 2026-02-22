pub const CHUNK_SIZE: usize = 48;

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
}

pub struct Chunk {
    blocks: [[[Block; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            blocks: [[[Block::Air; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
        }
    }

    pub fn generate(&mut self) {
        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let n = ((x as f32 * 0.19).sin() + (z as f32 * 0.23).cos()) * 2.0
                    + ((x as f32 * 0.07 + z as f32 * 0.11).sin()) * 3.0;
                let base_height = (12.0 + n).clamp(7.0, 18.0) as usize;

                for y in 0..base_height {
                    self.blocks[x][y][z] = if y + 4 < base_height {
                        Block::Stone
                    } else if y == base_height - 1 {
                        if base_height < 10 {
                            Block::Sand
                        } else if base_height > 16 {
                            Block::Snow
                        } else {
                            Block::Grass
                        }
                    } else {
                        Block::Dirt
                    };
                }
            }
        }

        self.add_house(8, 8, 8, 6, 8, 6);
        self.add_house(22, 10, 18, 7, 7, 7);
        self.add_house(34, 11, 30, 8, 8, 6);

        self.add_tree(14, 17);
        self.add_tree(28, 12);
        self.add_tree(38, 36);
    }

    fn add_house(&mut self, x0: usize, y0: usize, z0: usize, w: usize, h: usize, d: usize) {
        for x in x0..(x0 + w).min(CHUNK_SIZE - 1) {
            for z in z0..(z0 + d).min(CHUNK_SIZE - 1) {
                for y in y0..(y0 + h).min(CHUNK_SIZE - 1) {
                    let wall = x == x0 || x == x0 + w - 1 || z == z0 || z == z0 + d - 1;
                    let roof = y == y0 + h - 1;
                    let floor = y == y0;

                    if roof {
                        self.blocks[x][y][z] = Block::Planks;
                    } else if floor {
                        self.blocks[x][y][z] = Block::Wood;
                    } else if wall {
                        let is_window = y > y0 + 1
                            && y < y0 + h - 2
                            && ((x == x0 || x == x0 + w - 1) && z == z0 + d / 2
                                || (z == z0 || z == z0 + d - 1) && x == x0 + w / 2);
                        self.blocks[x][y][z] = if is_window {
                            Block::Glass
                        } else {
                            Block::Bricks
                        };
                    } else {
                        self.blocks[x][y][z] = Block::Air;
                    }
                }
            }
        }

        let door_x = x0 + w / 2;
        self.set(door_x, y0 + 1, z0, Block::Air);
        self.set(door_x, y0 + 2, z0, Block::Air);
    }

    fn add_tree(&mut self, x: usize, z: usize) {
        if x >= CHUNK_SIZE || z >= CHUNK_SIZE {
            return;
        }

        let mut ground = 0;
        for y in (0..CHUNK_SIZE).rev() {
            if self.blocks[x][y][z] != Block::Air {
                ground = y + 1;
                break;
            }
        }

        let trunk_h = 4;
        for y in ground..(ground + trunk_h).min(CHUNK_SIZE) {
            self.blocks[x][y][z] = Block::Wood;
        }

        let top = (ground + trunk_h).min(CHUNK_SIZE - 1);
        for lx in x.saturating_sub(2)..=(x + 2).min(CHUNK_SIZE - 1) {
            for lz in z.saturating_sub(2)..=(z + 2).min(CHUNK_SIZE - 1) {
                for ly in top.saturating_sub(1)..=(top + 1).min(CHUNK_SIZE - 1) {
                    if self.blocks[lx][ly][lz] == Block::Air {
                        self.blocks[lx][ly][lz] = Block::Leaf;
                    }
                }
            }
        }
    }

    pub fn get_i32(&self, x: i32, y: i32, z: i32) -> Block {
        if x < 0 || y < 0 || z < 0 {
            return Block::Air;
        }

        let (x, y, z) = (x as usize, y as usize, z as usize);
        if x >= CHUNK_SIZE || y >= CHUNK_SIZE || z >= CHUNK_SIZE {
            Block::Air
        } else {
            self.blocks[x][y][z]
        }
    }

    pub fn set(&mut self, x: usize, y: usize, z: usize, block: Block) {
        if x < CHUNK_SIZE && y < CHUNK_SIZE && z < CHUNK_SIZE {
            self.blocks[x][y][z] = block;
        }
    }
}
