use crate::world::chunk::{Block, Chunk, CHUNK_SIZE};

pub struct World {
    pub chunk: Chunk,
}

impl World {
    pub fn new() -> Self {
        let mut chunk = Chunk::new();
        chunk.generate();
        Self { chunk }
    }

    pub fn break_block(&mut self, x: usize, y: usize, z: usize) {
        if x < CHUNK_SIZE && y < CHUNK_SIZE && z < CHUNK_SIZE {
            self.chunk.set(x, y, z, Block::Air);
        }
    }
}
