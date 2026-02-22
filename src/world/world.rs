use cgmath::{InnerSpace, Point3, Vector3};

use crate::world::chunk::{Block, Chunk, CHUNK_SIZE};

const MAX_MOBS: usize = 15;

#[derive(Clone, Copy)]
pub struct Bullet {
    pub position: Point3<f32>,
    pub velocity: Vector3<f32>,
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
    pub velocity_y: f32,
    pub grounded: bool,
    pub jump_cooldown: f32,
    pub attack_cooldown: f32,
}

pub struct World {
    pub chunk: Chunk,
    pub bullets: Vec<Bullet>,
    pub mobs: Vec<Mob>,
    spawn_timer: f32,
}

impl World {
    pub fn new() -> Self {
        let mut chunk = Chunk::new();
        chunk.generate();
        let mut world = Self {
            chunk,
            bullets: Vec::new(),
            mobs: Vec::new(),
            spawn_timer: 0.0,
        };
        world.spawn_mob_at(18.5, 15.5);
        world.spawn_mob_at(26.5, 20.5);
        world.spawn_mob_at(34.5, 28.5);
        world
    }

    fn spawn_mob_at(&mut self, x: f32, z: f32) {
        if self.mobs.len() >= MAX_MOBS {
            return;
        }

        if let Some(ground_y) = self.highest_solid_below(x, z, CHUNK_SIZE as f32 - 1.0) {
            self.mobs.push(Mob {
                position: Point3::new(x, ground_y, z),
                health: 100.0,
                radius: 0.4,
                height: 1.8,
                velocity_y: 0.0,
                grounded: true,
                jump_cooldown: 0.0,
                attack_cooldown: 0.0,
            });
        }
    }

    fn try_spawn_mob_near(&mut self, player: Point3<f32>) -> bool {
        if self.mobs.len() >= MAX_MOBS {
            return false;
        }

        let mut preferred = [
            (player.x + 10.0, player.z),
            (player.x - 10.0, player.z),
            (player.x, player.z + 10.0),
            (player.x, player.z - 10.0),
            (player.x + 7.5, player.z + 7.5),
        ];

        for (x, z) in &mut preferred {
            *x = x.clamp(1.5, CHUNK_SIZE as f32 - 2.5);
            *z = z.clamp(1.5, CHUNK_SIZE as f32 - 2.5);
            if self
                .highest_solid_below(*x, *z, CHUNK_SIZE as f32 - 1.0)
                .is_some()
            {
                self.spawn_mob_at(*x, *z);
                return true;
            }
        }

        false
    }

    pub fn spawn_bullet(&mut self, origin: Point3<f32>, direction: Vector3<f32>) {
        if direction.magnitude2() <= 0.0 {
            return;
        }

        let dir = direction.normalize();
        self.bullets.push(Bullet {
            position: origin + dir * 0.5,
            velocity: dir * 0.2,
            damage: 34.0,
            max_distance: 100.0,
            traveled: 0.0,
        });
    }

    pub fn update_bullets(&mut self) -> bool {
        let mut world_changed = false;

        for i in (0..self.bullets.len()).rev() {
            let bullet = &mut self.bullets[i];
            let previous_position = bullet.position;
            bullet.velocity.y -= 0.002;
            bullet.position += bullet.velocity;
            bullet.traveled += bullet.velocity.magnitude();

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
                if bullet_hits_mob(previous_position, bullet.position, mob) {
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

    pub fn update_mobs(&mut self, player: Point3<f32>) -> (bool, f32) {
        let mut world_changed = false;
        let mut player_damage = 0.0;

        self.spawn_timer += 1.0;
        if self.spawn_timer >= 120.0 {
            self.spawn_timer = 0.0;
            if self.try_spawn_mob_near(player) {
                world_changed = true;
            }
        }

        for i in 0..self.mobs.len() {
            let mut moved = false;
            let (head, tail) = self.mobs.split_at_mut(i);
            let (mob, rest) = tail.split_first_mut().expect("mob exists");

            let toward_player =
                Vector3::new(player.x - mob.position.x, 0.0, player.z - mob.position.z);
            let distance = toward_player.magnitude();

            if distance > 0.01 {
                let speed = if distance > 1.4 { 0.04 } else { 0.02 };
                let dir = toward_player / distance;
                let proposed_x = mob.position.x + dir.x * speed;
                let can_move_x =
                    !mob_collides(
                        &self.chunk,
                        proposed_x,
                        mob.position.y,
                        mob.position.z,
                        mob.radius,
                        mob.height,
                    ) && !mob_collides_with_others(proposed_x, mob.position.z, mob.radius, head)
                        && !mob_collides_with_others(proposed_x, mob.position.z, mob.radius, rest);
                if can_move_x {
                    mob.position.x = proposed_x;
                    moved = true;
                }

                let proposed_z = mob.position.z + dir.z * speed;
                let can_move_z =
                    !mob_collides(
                        &self.chunk,
                        mob.position.x,
                        mob.position.y,
                        proposed_z,
                        mob.radius,
                        mob.height,
                    ) && !mob_collides_with_others(mob.position.x, proposed_z, mob.radius, head)
                        && !mob_collides_with_others(mob.position.x, proposed_z, mob.radius, rest);
                if can_move_z {
                    mob.position.z = proposed_z;
                    moved = true;
                }

                if mob.grounded && !can_move_x && !can_move_z && mob.jump_cooldown <= 0.0 {
                    mob.velocity_y = 0.06;
                    mob.grounded = false;
                    mob.jump_cooldown = 20.0;
                    moved = true;
                }
                if moved {
                    world_changed = true;
                }
            }

            mob.velocity_y -= 0.003;
            let proposed_y = mob.position.y + mob.velocity_y;
            if !mob_collides(
                &self.chunk,
                mob.position.x,
                proposed_y,
                mob.position.z,
                mob.radius,
                mob.height,
            ) {
                mob.position.y = proposed_y;
                mob.grounded = false;
            } else {
                if mob.velocity_y < 0.0 {
                    if let Some(ground_y) = highest_solid_below_chunk(
                        &self.chunk,
                        mob.position.x,
                        mob.position.z,
                        mob.position.y + 0.5,
                    ) {
                        mob.position.y = ground_y;
                    }
                    mob.grounded = true;
                }
                mob.velocity_y = 0.0;
            }

            if mob.jump_cooldown > 0.0 {
                mob.jump_cooldown -= 1.0;
            }

            if mob.attack_cooldown > 0.0 {
                mob.attack_cooldown -= 1.0;
            }

            let dx = player.x - mob.position.x;
            let dz = player.z - mob.position.z;
            let horizontal_dist_sq = dx * dx + dz * dz;
            let vertical_close = (player.y - mob.position.y).abs() <= mob.height;
            let touch_dist = mob.radius + 0.35;
            if horizontal_dist_sq <= touch_dist * touch_dist
                && vertical_close
                && mob.attack_cooldown <= 0.0
            {
                player_damage += 8.0;
                mob.attack_cooldown = 45.0;
            }
        }

        (world_changed, player_damage)
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
        highest_solid_below_chunk(&self.chunk, x, z, max_y)
    }
}

fn bullet_hits_mob(start: Point3<f32>, end: Point3<f32>, mob: &Mob) -> bool {
    let seg_dx = end.x - start.x;
    let seg_dz = end.z - start.z;
    let seg_len_sq = seg_dx * seg_dx + seg_dz * seg_dz;

    let t = if seg_len_sq > 0.0001 {
        ((mob.position.x - start.x) * seg_dx + (mob.position.z - start.z) * seg_dz) / seg_len_sq
    } else {
        0.0
    }
    .clamp(0.0, 1.0);

    let closest_x = start.x + seg_dx * t;
    let closest_z = start.z + seg_dz * t;
    let dx = closest_x - mob.position.x;
    let dz = closest_z - mob.position.z;
    let in_radius = dx * dx + dz * dz <= mob.radius * mob.radius;

    let seg_min_y = start.y.min(end.y);
    let seg_max_y = start.y.max(end.y);
    let mob_min_y = mob.position.y;
    let mob_max_y = mob.position.y + mob.height;
    let in_height = seg_max_y >= mob_min_y && seg_min_y <= mob_max_y;

    in_radius && in_height
}

fn mob_collides(chunk: &Chunk, x: f32, y: f32, z: f32, radius: f32, height: f32) -> bool {
    let min_x = (x - radius).floor() as i32;
    let max_x = (x + radius).floor() as i32;
    let min_z = (z - radius).floor() as i32;
    let max_z = (z + radius).floor() as i32;
    let min_y = y.floor() as i32;
    let max_y = (y + height).ceil() as i32;

    for bx in min_x..=max_x {
        for by in min_y..=max_y {
            for bz in min_z..=max_z {
                if chunk.get_i32(bx, by, bz) != Block::Air {
                    return true;
                }
            }
        }
    }

    false
}

fn mob_collides_with_others(x: f32, z: f32, radius: f32, others: &[Mob]) -> bool {
    for other in others {
        let dx = x - other.position.x;
        let dz = z - other.position.z;
        let min_dist = radius + other.radius;
        if dx * dx + dz * dz < min_dist * min_dist {
            return true;
        }
    }

    false
}

fn highest_solid_below_chunk(chunk: &Chunk, x: f32, z: f32, max_y: f32) -> Option<f32> {
    let xi = x.floor() as i32;
    let zi = z.floor() as i32;

    if xi < 0 || zi < 0 || xi >= CHUNK_SIZE as i32 || zi >= CHUNK_SIZE as i32 {
        return None;
    }

    let mut y = max_y.floor() as i32;
    y = y.min(CHUNK_SIZE as i32 - 1);

    while y >= 0 {
        if chunk.get_i32(xi, y, zi) != Block::Air {
            return Some(y as f32 + 1.0);
        }
        y -= 1;
    }

    Some(0.0)
}
