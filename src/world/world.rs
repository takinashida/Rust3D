use cgmath::{InnerSpace, Point3, Vector3};

use crate::world::chunk::{Block, Chunk, CHUNK_SIZE};

#[derive(Clone, Copy)]
pub struct Bullet {
    pub position: Point3<f32>,
    pub direction: Vector3<f32>,
    pub speed: f32,
    pub damage: f32,
    pub max_distance: f32,
    pub traveled: f32,
}

#[derive(Clone, Copy)]
pub struct Mob {
    pub position: Point3<f32>,
    pub health: f32,
    pub radius: f32,
    pub height: f32,
}

pub struct World {
    pub chunk: Chunk,
    pub bullets: Vec<Bullet>,
    pub mobs: Vec<Mob>,
}

impl World {
    pub fn new() -> Self {
        let mut chunk = Chunk::new();
        chunk.generate();
        let mut world = Self {
            chunk,
            bullets: Vec::new(),
            mobs: Vec::new(),
        };
        world.spawn_mobs();
        world
    }

    fn spawn_mobs(&mut self) {
        let spawn_points = [(18.5, 15.5), (26.5, 20.5), (34.5, 28.5)];

        for (x, z) in spawn_points {
            let base_y = self.highest_solid_below(x, z, CHUNK_SIZE as f32 - 1.0);
            if let Some(ground_y) = base_y {
                self.mobs.push(Mob {
                    position: Point3::new(x, ground_y, z),
                    health: 100.0,
                    radius: 0.4,
                    height: 1.8,
                });
            }
        }
    }

    pub fn spawn_bullet(&mut self, origin: Point3<f32>, direction: Vector3<f32>) {
        if direction.magnitude2() <= 0.0 {
            return;
        }

        let dir = direction.normalize();
        self.bullets.push(Bullet {
            position: origin + dir * 0.5,
            direction: dir,
            speed: 0.9,
            damage: 34.0,
            max_distance: 35.0,
            traveled: 0.0,
        });
    }

    pub fn update_bullets(&mut self) -> bool {
        let mut world_changed = false;

        for i in (0..self.bullets.len()).rev() {
            let bullet = &mut self.bullets[i];
            bullet.position += bullet.direction * bullet.speed;
            bullet.traveled += bullet.speed;

            let x = bullet.position.x.floor() as i32;
            let y = bullet.position.y.floor() as i32;
            let z = bullet.position.z.floor() as i32;

            let hit_block = x >= 0
                && y >= 0
                && z >= 0
                && x < CHUNK_SIZE as i32
                && y < CHUNK_SIZE as i32
                && z < CHUNK_SIZE as i32
                && self.chunk.get_i32(x, y, z) != Block::Air;

            if hit_block {
                self.bullets.swap_remove(i);
                continue;
            }

            let mut hit_mob = false;
            for mob in &mut self.mobs {
                if bullet_hits_mob(bullet, mob) {
                    mob.health -= bullet.damage;
                    hit_mob = true;
                    world_changed = true;
                    break;
                }
            }

            if hit_mob || bullet.traveled >= bullet.max_distance {
                self.bullets.swap_remove(i);
            }
        }

        let alive_before = self.mobs.len();
        self.mobs.retain(|mob| mob.health > 0.0);
        if self.mobs.len() != alive_before {
            world_changed = true;
        }

        world_changed
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

fn bullet_hits_mob(bullet: &Bullet, mob: &Mob) -> bool {
    let dx = bullet.position.x - mob.position.x;
    let dz = bullet.position.z - mob.position.z;
    let in_radius = dx * dx + dz * dz <= mob.radius * mob.radius;
    let in_height =
        bullet.position.y >= mob.position.y && bullet.position.y <= mob.position.y + mob.height;
    in_radius && in_height
}
