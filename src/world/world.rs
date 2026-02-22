use cgmath::{InnerSpace, Point3, Vector3};

use crate::world::chunk::{Block, Chunk, CHUNK_DEPTH, CHUNK_HEIGHT, CHUNK_WIDTH};

const MAX_MOBS: usize = 20;

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

#[derive(Clone, Copy)]
pub struct Particle {
    pub position: Point3<f32>,
    pub velocity: Vector3<f32>,
    pub life: f32,
    pub color: [f32; 3],
    pub size: f32,
}

#[derive(Clone, Copy)]
pub struct Explosive {
    pub position: Point3<f32>,
    pub velocity: Vector3<f32>,
    pub timer: f32,
    pub radius: f32,
}

pub struct World {
    pub chunk: Chunk,
    pub bullets: Vec<Bullet>,
    pub mobs: Vec<Mob>,
    pub particles: Vec<Particle>,
    pub explosives: Vec<Explosive>,
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
            particles: Vec::new(),
            explosives: Vec::new(),
            spawn_timer: 0.0,
        };
        world.spawn_mob_at(160.5, 140.5);
        world.spawn_mob_at(210.5, 200.5);
        world.spawn_mob_at(280.5, 260.5);
        world
    }

    fn spawn_mob_at(&mut self, x: f32, z: f32) {
        if self.mobs.len() >= MAX_MOBS {
            return;
        }

        if let Some(ground_y) = self.highest_solid_below(x, z, CHUNK_HEIGHT as f32 - 1.0) {
            self.mobs.push(Mob {
                position: Point3::new(x, ground_y + 2.0, z),
                health: 100.0,
                radius: 0.4,
                height: 1.8,
                velocity_y: 0.0,
                grounded: false,
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
            (player.x + 22.0, player.z),
            (player.x - 22.0, player.z),
            (player.x, player.z + 22.0),
            (player.x, player.z - 22.0),
            (player.x + 16.0, player.z + 16.0),
        ];

        for (x, z) in &mut preferred {
            *x = x.clamp(2.5, CHUNK_WIDTH as f32 - 2.5);
            *z = z.clamp(2.5, CHUNK_DEPTH as f32 - 2.5);
            if self
                .highest_solid_below(*x, *z, CHUNK_HEIGHT as f32 - 1.0)
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

    pub fn spawn_explosive(&mut self, origin: Point3<f32>, direction: Vector3<f32>) {
        if direction.magnitude2() <= 0.0 {
            return;
        }
        let dir = direction.normalize();
        self.explosives.push(Explosive {
            position: origin + dir * 0.8,
            velocity: dir * 0.34 + Vector3::new(0.0, 0.14, 0.0),
            timer: 120.0,
            radius: 30.0,
        });
    }

    pub fn update_bullets(&mut self) -> bool {
        let mut world_changed = false;

        for i in (0..self.bullets.len()).rev() {
            let previous_position;
            let bullet_position;
            let bullet_traveled;
            let bullet_max_distance;
            let bullet_damage;
            {
                let bullet = &mut self.bullets[i];
                previous_position = bullet.position;
                bullet.velocity.y -= 0.002;
                bullet.position += bullet.velocity;
                bullet.traveled += bullet.velocity.magnitude();
                bullet_position = bullet.position;
                bullet_traveled = bullet.traveled;
                bullet_max_distance = bullet.max_distance;
                bullet_damage = bullet.damage;
            }

            let x = bullet_position.x.floor() as i32;
            let y = bullet_position.y.floor() as i32;
            let z = bullet_position.z.floor() as i32;

            let hit_block = x >= 0
                && y >= 0
                && z >= 0
                && x < CHUNK_WIDTH as i32
                && y < CHUNK_HEIGHT as i32
                && z < CHUNK_DEPTH as i32
                && self.chunk.get_i32(x, y, z) != Block::Air;

            if hit_block {
                self.bullets.swap_remove(i);
                continue;
            }

            let mut hit_mob = false;
            let mut hit_particle_pos = None;
            let mut death_particle_pos = None;
            for mob in &mut self.mobs {
                if bullet_hits_mob(previous_position, bullet_position, mob) {
                    mob.health -= bullet_damage;
                    hit_mob = true;
                    world_changed = true;
                    hit_particle_pos = Some(bullet_position);
                    if mob.health <= 0.0 {
                        death_particle_pos = Some(mob.position);
                    }
                    break;
                }
            }
            if let Some(pos) = hit_particle_pos {
                self.spawn_particles(pos, 10, [1.0, 0.2, 0.2], 0.12, 30.0, 0.18);
            }
            if let Some(pos) = death_particle_pos {
                self.spawn_particles(pos, 26, [1.0, 0.55, 0.15], 0.22, 55.0, 0.28);
            }

            if hit_mob || bullet_traveled >= bullet_max_distance {
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

    pub fn update_explosives(&mut self) -> bool {
        let mut world_changed = false;
        for i in (0..self.explosives.len()).rev() {
            let explosive = &mut self.explosives[i];
            explosive.velocity.y -= 0.004;
            explosive.position += explosive.velocity;
            explosive.timer -= 1.0;

            let x = explosive.position.x.floor() as i32;
            let y = explosive.position.y.floor() as i32;
            let z = explosive.position.z.floor() as i32;

            let hit_block = self.chunk.get_i32(x, y, z) != Block::Air;
            if hit_block || explosive.timer <= 0.0 {
                let pos = explosive.position;
                let radius = explosive.radius;
                self.explosives.swap_remove(i);
                if self.explode_at(pos, radius) {
                    world_changed = true;
                }
            }
        }

        world_changed
    }

    fn explode_at(&mut self, center: Point3<f32>, radius: f32) -> bool {
        let mut changed = false;
        let r2 = radius * radius;
        let min_x = (center.x - radius).floor() as i32;
        let max_x = (center.x + radius).ceil() as i32;
        let min_y = (center.y - radius).floor() as i32;
        let max_y = (center.y + radius).ceil() as i32;
        let min_z = (center.z - radius).floor() as i32;
        let max_z = (center.z + radius).ceil() as i32;

        for x in min_x..=max_x {
            for y in min_y..=max_y {
                for z in min_z..=max_z {
                    let dx = x as f32 + 0.5 - center.x;
                    let dy = y as f32 + 0.5 - center.y;
                    let dz = z as f32 + 0.5 - center.z;
                    if dx * dx + dy * dy + dz * dz <= r2
                        && self.chunk.get_i32(x, y, z) != Block::Air
                    {
                        self.break_block(x as usize, y as usize, z as usize);
                        changed = true;
                    }
                }
            }
        }

        self.spawn_particles(center, 45, [1.0, 0.6, 0.2], 0.36, 70.0, 0.34);
        changed
    }

    pub fn update_particles(&mut self) {
        for p in &mut self.particles {
            p.velocity.y -= 0.0015;
            p.position += p.velocity;
            p.life -= 1.0;
        }
        self.particles.retain(|p| p.life > 0.0);
    }

    fn spawn_particles(
        &mut self,
        origin: Point3<f32>,
        count: usize,
        color: [f32; 3],
        speed: f32,
        life: f32,
        size: f32,
    ) {
        for i in 0..count {
            let a = i as f32 * 0.618_033_95;
            let b = i as f32 * 0.414_213_57;
            let dir = Vector3::new(a.cos() * b.sin(), (b * 1.3).cos().abs(), a.sin() * b.cos())
                .normalize();
            self.particles.push(Particle {
                position: origin,
                velocity: dir * speed,
                life,
                color,
                size,
            });
        }
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
        if x < CHUNK_WIDTH && y < CHUNK_HEIGHT && z < CHUNK_DEPTH {
            self.chunk.set(x, y, z, Block::Air);
        }
    }

    pub fn set_block(&mut self, x: usize, y: usize, z: usize, block: Block) {
        if x < CHUNK_WIDTH && y < CHUNK_HEIGHT && z < CHUNK_DEPTH {
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
                && x < CHUNK_WIDTH as i32
                && y < CHUNK_HEIGHT as i32
                && z < CHUNK_DEPTH as i32
            {
                let block = self.chunk.get_i32(x, y, z);
                if block != Block::Air {
                    self.break_block(x as usize, y as usize, z as usize);
                    self.spawn_particles(p, 12, [0.8, 0.75, 0.65], 0.1, 25.0, 0.12);
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
                || x >= CHUNK_WIDTH as i32
                || y >= CHUNK_HEIGHT as i32
                || z >= CHUNK_DEPTH as i32
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

    if xi < 0 || zi < 0 || xi >= CHUNK_WIDTH as i32 || zi >= CHUNK_DEPTH as i32 {
        return None;
    }

    let mut y = max_y.floor() as i32;
    y = y.min(CHUNK_HEIGHT as i32 - 1);

    while y >= 0 {
        if chunk.get_i32(xi, y, zi) != Block::Air {
            return Some(y as f32 + 1.0);
        }
        y -= 1;
    }

    Some(0.0)
}
