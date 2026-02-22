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

    pub fn set_block(&mut self, x: usize, y: usize, z: usize, block: Block) {
        if x < CHUNK_SIZE && y < CHUNK_SIZE && z < CHUNK_SIZE {
            self.chunk.set(x, y, z, block);
        }
    }

    pub fn break_block_from_ray(
        &mut self,
        origin: Point3<f32>,
        direction: Vector3<f32>,
        max_distance: f32,
    ) -> bool {
        let dir = if direction.magnitude2() > 0.0 {
            direction.normalize()
        } else {
            return false;
        };

        let step = 0.05;
        let mut distance = 0.0;

        while distance <= max_distance {
            let p = origin + dir * distance;
            let x = p.x.floor() as i32;
            let y = p.y.floor() as i32;
            let z = p.z.floor() as i32;

            if x >= 0
                && y >= 0
                && z >= 0
                && x < CHUNK_SIZE as i32
                && y < CHUNK_SIZE as i32
                && z < CHUNK_SIZE as i32
            {
                let block = self.chunk.get_i32(x, y, z);
                if block != Block::Air {
                    self.break_block(x as usize, y as usize, z as usize);
                    return true;
                }
            }

            distance += step;
        }

        false
    }

    pub fn shoot_target_from_ray(
        &mut self,
        origin: Point3<f32>,
        direction: Vector3<f32>,
        max_distance: f32,
    ) -> bool {
        let dir = if direction.magnitude2() > 0.0 {
            direction.normalize()
        } else {
            return false;
        };

        let step = 0.05;
        let mut distance = 0.0;

        while distance <= max_distance {
            let p = origin + dir * distance;
            let x = p.x.floor() as i32;
            let y = p.y.floor() as i32;
            let z = p.z.floor() as i32;

            if x >= 0
                && y >= 0
                && z >= 0
                && x < CHUNK_SIZE as i32
                && y < CHUNK_SIZE as i32
                && z < CHUNK_SIZE as i32
            {
                let block = self.chunk.get_i32(x, y, z);
                if block == Block::Target {
                    self.break_block(x as usize, y as usize, z as usize);
                    return true;
                }

                if block != Block::Air {
                    return false;
                }
            }

            distance += step;
        }

        false
    }

    pub fn place_block_from_ray(
        &mut self,
        origin: Point3<f32>,
        direction: Vector3<f32>,
        max_distance: f32,
        block: Block,
    ) -> bool {
        if block == Block::Air {
            return false;
        }

        let dir = if direction.magnitude2() > 0.0 {
            direction.normalize()
        } else {
            return false;
        };

        let step = 0.05;
        let mut distance = 0.0;
        let mut last_air = None;

        while distance <= max_distance {
            let p = origin + dir * distance;
            let x = p.x.floor() as i32;
            let y = p.y.floor() as i32;
            let z = p.z.floor() as i32;

            if x < 0
                || y < 0
                || z < 0
                || x >= CHUNK_SIZE as i32
                || y >= CHUNK_SIZE as i32
                || z >= CHUNK_SIZE as i32
            {
                distance += step;
                continue;
            }

            let current = self.chunk.get_i32(x, y, z);
            if current == Block::Air {
                last_air = Some((x as usize, y as usize, z as usize));
            } else if let Some((ax, ay, az)) = last_air {
                self.set_block(ax, ay, az, block);
                return true;
            } else {
                return false;
            }

            distance += step;
        }

        false
    }

    pub fn block_at(&self, x: i32, y: i32, z: i32) -> Block {
        self.chunk.get_i32(x, y, z)
    }

    pub fn highest_solid_below(&self, x: f32, z: f32, max_y: f32) -> Option<f32> {
        let xi = x.floor() as i32;
        let zi = z.floor() as i32;

        if xi < 0 || zi < 0 || xi >= CHUNK_SIZE as i32 || zi >= CHUNK_SIZE as i32 {
            return None;
        }

        let mut y = max_y.floor() as i32;
        y = y.min(CHUNK_SIZE as i32 - 1);

        while y >= 0 {
            if self.chunk.get_i32(xi, y, zi) != Block::Air {
                return Some(y as f32 + 1.0);
            }
            y -= 1;
        }

        Some(0.0)
    }
}
