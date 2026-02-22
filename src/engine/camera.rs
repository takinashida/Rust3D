use cgmath::{perspective, vec3, Deg, InnerSpace, Matrix4, Point3, Vector3};
use winit::keyboard::KeyCode;

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
}

impl Camera {
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
        }
        if input.is_pressed(KeyCode::Space) {
            self.position.y += self.speed;
        }
        if input.is_pressed(KeyCode::ShiftLeft) {
            self.position.y -= self.speed;
        }
        if input.is_pressed(KeyCode::Space) {
            self.position.y += self.speed;
        }
        if input.is_pressed(KeyCode::ShiftLeft) {
            self.position.y -= self.speed;
        }
    }

    pub fn process_mouse(&mut self, delta_x: f64, delta_y: f64) {
        self.yaw += delta_x as f32 * self.mouse_sensitivity;
        self.pitch = (self.pitch - delta_y as f32 * self.mouse_sensitivity).clamp(-89.0, 89.0);
    }

    pub fn process_mouse(&mut self, delta_x: f64, delta_y: f64) {
        self.yaw += delta_x as f32 * self.mouse_sensitivity;
        self.pitch = (self.pitch - delta_y as f32 * self.mouse_sensitivity).clamp(-89.0, 89.0);
    }

    pub fn update(&mut self, input: &InputState) {
        let front = self.front();
        let right = front.cross(Vector3::unit_y()).normalize();

        if input.is_pressed(KeyCode::KeyW) {
            self.position += front * self.speed;
        }
        if input.is_pressed(KeyCode::KeyS) {
            self.position -= front * self.speed;
        }
        if input.is_pressed(KeyCode::KeyA) {
            self.position -= right * self.speed;
        }
        if input.is_pressed(KeyCode::KeyD) {
            self.position += right * self.speed;
        }
        if input.is_pressed(KeyCode::Space) {
            self.position.y += self.speed;
        }
        if input.is_pressed(KeyCode::ShiftLeft) {
            self.position.y -= self.speed;
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
}

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);
