use cgmath::{perspective, vec3, Deg, InnerSpace, Matrix4, Point3, Vector3};
use winit::keyboard::KeyCode;

use crate::world::world::World;

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
}

impl Camera {
    pub const PLAYER_HEIGHT: f32 = 2.0;
    pub const EYE_HEIGHT: f32 = 1.62;

    pub fn new() -> Self {
        Self {
            position: Point3::new(8.0, 10.0, 20.0),
            yaw: -90.0,
            pitch: -15.0,
            aspect: 16.0 / 9.0,
            fov_y: 60.0,
            z_near: 0.1,
            z_far: 200.0,
            speed: 0.25,
            mouse_sensitivity: 0.1,
            velocity_y: 0.0,
            grounded: false,
        }
    }

    pub fn process_mouse(&mut self, delta_x: f64, delta_y: f64) {
        self.yaw += delta_x as f32 * self.mouse_sensitivity;
        self.pitch = (self.pitch - delta_y as f32 * self.mouse_sensitivity).clamp(-89.0, 89.0);
    }

    pub fn eye_position(&self) -> Point3<f32> {
        Point3::new(
            self.position.x,
            self.position.y + Self::EYE_HEIGHT,
            self.position.z,
        )
    }

    pub fn front(&self) -> Vector3<f32> {
        vec3(
            self.yaw.to_radians().cos() * self.pitch.to_radians().cos(),
            self.pitch.to_radians().sin(),
            self.yaw.to_radians().sin() * self.pitch.to_radians().cos(),
        )
        .normalize()
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

        let mut desired = vec3(0.0, 0.0, 0.0);
        if input.is_pressed(KeyCode::KeyW) {
            desired += planar_front;
        }
        if input.is_pressed(KeyCode::KeyS) {
            desired -= planar_front;
        }
        if input.is_pressed(KeyCode::KeyA) {
            desired += right;
        }
        if input.is_pressed(KeyCode::KeyD) {
            desired -= right;
        }

        if desired.magnitude2() > 0.0 {
            let movement = desired.normalize() * self.speed;

            let target_x = self.position.x + movement.x;
            if world.is_walkable_at(
                target_x,
                self.position.y,
                self.position.z,
                Self::PLAYER_HEIGHT,
            ) {
                self.position.x = target_x;
            }

            let target_z = self.position.z + movement.z;
            if world.is_walkable_at(
                self.position.x,
                self.position.y,
                target_z,
                Self::PLAYER_HEIGHT,
            ) {
                self.position.z = target_z;
            }
        }

        let jump_velocity = 0.35;
        let gravity = 0.02;

        if input.is_pressed(KeyCode::Space) && self.grounded {
            self.velocity_y = jump_velocity;
            self.grounded = false;
        }

        self.velocity_y -= gravity;

        if self.velocity_y > 0.0 {
            let next_top = self.position.y + Self::PLAYER_HEIGHT + self.velocity_y;
            let head_block = next_top.floor() as i32;
            let bx = self.position.x.floor() as i32;
            let bz = self.position.z.floor() as i32;
            if world.is_solid_at(bx, head_block, bz) {
                self.velocity_y = 0.0;
            }
        }

        self.position.y += self.velocity_y;

        if let Some(surface_height) = world.surface_height(self.position.x, self.position.z) {
            if self.position.y <= surface_height {
                self.position.y = surface_height;
                self.velocity_y = 0.0;
                self.grounded = true;
            }
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
        let view = Matrix4::look_to_rh(self.eye_position(), self.front(), Vector3::unit_y());
        let proj = perspective(
            Deg(self.fov_y),
            self.aspect.max(0.01),
            self.z_near,
            self.z_far,
        );

        OPENGL_TO_WGPU_MATRIX * proj * view
    }
}

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);
