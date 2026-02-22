pub const CHUNK_SIZE: usize = 16;

#[derive(Clone, Copy)]
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
                let height = 8;
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
}