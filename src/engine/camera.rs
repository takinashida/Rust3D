use cgmath::{perspective, vec3, Deg, InnerSpace, Matrix4, Point3, Vector3};
use winit::keyboard::KeyCode;

use crate::world::{chunk::Block, world::World};

use super::input::InputState;

pub struct Camera {
    pub position: Point3<f32>,
    pub yaw: f32,
    pub pitch: f32,
    pub aspect: f32,
    pub fov_y: f32,
    pub z_near: f32,
    pub z_far: f32,
    pub speed: f32,
    pub mouse_sensitivity: f32,
    pub velocity_y: f32,
    pub grounded: bool,
    pub eye_height: f32,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            position: Point3::new(8.0, 16.0, 10.0),
            yaw: -90.0,
            pitch: -15.0,
            aspect: 16.0 / 9.0,
            fov_y: 60.0,
            z_near: 0.1,
            z_far: 300.0,
            speed: 0.08,
            mouse_sensitivity: 0.2,
            velocity_y: 0.0,
            grounded: false,
            eye_height: 1.8,
        }
    }

    pub fn process_mouse(&mut self, delta_x: f64, delta_y: f64) {
        self.yaw += delta_x as f32 * self.mouse_sensitivity;
        self.pitch = (self.pitch - delta_y as f32 * self.mouse_sensitivity).clamp(-89.0, 89.0);
    }

    pub fn update(&mut self, input: &InputState, world: &World) {
        let front = self.front();
        let planar_front = {
            let v = vec3(front.x, 0.0, front.z);
            if v.magnitude2() > 0.0 {
                v.normalize()
            } else {
                vec3(0.0, 0.0, -1.0)
            }
        };
        let right = planar_front.cross(Vector3::unit_y()).normalize();

        let mut move_delta = vec3(0.0, 0.0, 0.0);
        if input.is_pressed(KeyCode::KeyW) {
            move_delta += planar_front * self.speed;
        }
        if input.is_pressed(KeyCode::KeyS) {
            move_delta -= planar_front * self.speed;
        }
        if input.is_pressed(KeyCode::KeyD) {
            move_delta += right * self.speed;
        }
        if input.is_pressed(KeyCode::KeyA) {
            move_delta -= right * self.speed;
        }

        self.apply_horizontal_collision(move_delta, world);

        let jump_velocity = 0.30;
        let gravity = 0.012;

        if input.is_pressed(KeyCode::Space) && self.grounded {
            self.velocity_y = jump_velocity;
            self.grounded = false;
        }

        self.velocity_y -= gravity;
        self.position.y += self.velocity_y;

        let feet_y = self.position.y - self.eye_height;
        if let Some(ground_y) =
            world.highest_solid_below(self.position.x, self.position.z, feet_y + 0.1)
        {
            if feet_y <= ground_y {
                self.position.y = ground_y + self.eye_height;
                self.velocity_y = 0.0;
                self.grounded = true;
            } else {
                self.grounded = false;
            }
        } else {
            self.grounded = false;
        }

        let head_x = self.position.x.floor() as i32;
        let head_y = self.position.y.floor() as i32;
        let head_z = self.position.z.floor() as i32;
        if world.block_at(head_x, head_y, head_z) != Block::Air && self.velocity_y > 0.0 {
            self.position.y = head_y as f32 - 0.01;
            self.velocity_y = 0.0;
        }

        if input.is_pressed(KeyCode::ArrowLeft) {
            self.yaw -= 1.5;
        }
        if input.is_pressed(KeyCode::ArrowRight) {
            self.yaw += 1.5;
        }
        if input.is_pressed(KeyCode::ArrowUp) {
            self.pitch = (self.pitch + 1.0).clamp(-89.0, 89.0);
        }
        if input.is_pressed(KeyCode::ArrowDown) {
            self.pitch = (self.pitch - 1.0).clamp(-89.0, 89.0);
        }
    }

    pub fn view_proj_matrix(&self) -> Matrix4<f32> {
        let view = Matrix4::look_to_rh(self.position, self.front(), Vector3::unit_y());
        let proj = perspective(
            Deg(self.fov_y),
            self.aspect.max(0.01),
            self.z_near,
            self.z_far,
        );

        OPENGL_TO_WGPU_MATRIX * proj * view
    }

    fn front(&self) -> Vector3<f32> {
        vec3(
            self.yaw.to_radians().cos() * self.pitch.to_radians().cos(),
            self.pitch.to_radians().sin(),
            self.yaw.to_radians().sin() * self.pitch.to_radians().cos(),
        )
        .normalize()
    }

    pub fn look_direction(&self) -> Vector3<f32> {
        self.front()
    }
}

impl Camera {
    fn apply_horizontal_collision(&mut self, move_delta: Vector3<f32>, world: &World) {
        if move_delta.x == 0.0 && move_delta.z == 0.0 {
            return;
        }

        let player_radius = 0.3;
        let mut candidate = self.position;

        candidate.x += move_delta.x;
        if !self.collides_at(candidate, player_radius, world) {
            self.position.x = candidate.x;
        }

        candidate = self.position;
        candidate.z += move_delta.z;
        if !self.collides_at(candidate, player_radius, world) {
            self.position.z = candidate.z;
        }
    }

    fn collides_at(&self, position: Point3<f32>, radius: f32, world: &World) -> bool {
        let feet = position.y - self.eye_height;
        let head = position.y - 0.1;
        let skin = 0.02;
        // Avoid side-colliding with the top face of a block when stepping/falling off an edge.
        // Keeping a slightly thicker clearance near the feet prevents the player from
        // "sticking" while dropping down from higher blocks.
        let feet_clearance = 0.12;
        let min_x = (position.x - radius).floor() as i32;
        let max_x = (position.x + radius).floor() as i32;
        let min_z = (position.z - radius).floor() as i32;
        let max_z = (position.z + radius).floor() as i32;
        let min_y = (feet + feet_clearance).floor() as i32;
        let max_y = (head - skin).floor() as i32;

        if min_y > max_y {
            return false;
        }

        for x in min_x..=max_x {
            for y in min_y..=max_y {
                for z in min_z..=max_z {
                    if world.block_at(x, y, z) != Block::Air {
                        return true;
                    }
                }
            }
        }

        false
    }
}

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);
