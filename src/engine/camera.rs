use cgmath::{Matrix4, Point3, Vector3, Deg, perspective};

pub struct Camera {
    pub position: Point3<f32>,
    pub yaw: f32,
    pub pitch: f32,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            position: Point3::new(8.0, 12.0, 20.0),
            yaw: -90.0,
            pitch: 0.0,
        }
    }

    pub fn view_matrix(&self) -> Matrix4<f32> {
        let front = Vector3 {
            x: self.yaw.to_radians().cos() * self.pitch.to_radians().cos(),
            y: self.pitch.to_radians().sin(),
            z: self.yaw.to_radians().sin() * self.pitch.to_radians().cos(),
        };

        Matrix4::look_to_rh(self.position, front, Vector3::unit_y())
    }
}