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

    pub fn break_block_i32(&mut self, x: i32, y: i32, z: i32) {
        if x < 0 || y < 0 || z < 0 {
            return;
        }
        self.break_block(x as usize, y as usize, z as usize);
    }

    pub fn is_solid_at(&self, x: i32, y: i32, z: i32) -> bool {
        self.chunk.get_i32(x, y, z) != Block::Air
    }

    pub fn is_walkable_at(&self, x: f32, feet_y: f32, z: f32, player_height: f32) -> bool {
        let xi = x.floor() as i32;
        let zi = z.floor() as i32;
        let feet_block = feet_y.floor() as i32;
        let head_block = (feet_y + player_height - 0.001).floor() as i32;

        !self.is_solid_at(xi, feet_block, zi) && !self.is_solid_at(xi, head_block, zi)
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

    pub fn raycast_first_solid(
        &self,
        origin: Point3<f32>,
        direction: Vector3<f32>,
        max_distance: f32,
        step: f32,
    ) -> Option<(i32, i32, i32)> {
        let dir = if direction.magnitude2() > 0.0 {
            direction.normalize()
        } else {
            return None;
        };

        let mut distance = 0.0;
        while distance <= max_distance {
            let p = origin + dir * distance;
            let bx = p.x.floor() as i32;
            let by = p.y.floor() as i32;
            let bz = p.z.floor() as i32;

            if self.is_solid_at(bx, by, bz) {
                return Some((bx, by, bz));
            }

            distance += step;
        }

        None
    }
}
