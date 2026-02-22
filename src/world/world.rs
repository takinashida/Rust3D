use cgmath::{InnerSpace, Point3, Vector3};

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

    pub fn surface_height(&self, x: f32, z: f32) -> Option<f32> {
        let xi = x.floor() as i32;
        let zi = z.floor() as i32;

        if xi < 0 || zi < 0 || xi >= CHUNK_SIZE as i32 || zi >= CHUNK_SIZE as i32 {
            return None;
        }

        for y in (0..CHUNK_SIZE as i32).rev() {
            if self.chunk.get_i32(xi, y, zi) != Block::Air {
                return Some(y as f32 + 1.0);
            }
        }

        Some(0.0)
    }
}
