use crate::world::chunk::{Chunk, Block};

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
        self.chunk.blocks[x][y][z] = Block::Air;
    }
}