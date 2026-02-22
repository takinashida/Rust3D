pub const CHUNK_SIZE: usize = 16;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Block {
    Air,
    Dirt,
    Grass,
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
                let dx = x as i32 - (CHUNK_SIZE as i32 / 2);
                let dz = z as i32 - (CHUNK_SIZE as i32 / 2);
                let hill = ((dx * dx + dz * dz) as f32).sqrt() * 0.2;
                let height = (10.0 - hill).clamp(4.0, 12.0) as usize;

                for y in 0..height {
                    self.blocks[x][y][z] = if y == height - 1 {
                        Block::Grass
                    } else {
                        Block::Dirt
                    };
                }
            }
        }
    }

    pub fn get(&self, x: usize, y: usize, z: usize) -> Block {
        self.blocks[x][y][z]
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
