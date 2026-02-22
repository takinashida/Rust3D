use cgmath::Point3;

use crate::world::{
    chunk::{Block, Chunk, CHUNK_SIZE},
    world::World,
};

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
    pub normal: [f32; 3],
}

impl Vertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 6]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

pub struct Mesh {
    pub vertices: Vec<Vertex>,
}

impl Mesh {
    pub fn from_world(world: &World) -> Self {
        let mut vertices = Vec::new();

        append_chunk_mesh(&mut vertices, &world.chunk);

        for bullet in &world.bullets {
            append_cube(
                &mut vertices,
                Point3::new(
                    bullet.position.x - 0.08,
                    bullet.position.y - 0.08,
                    bullet.position.z - 0.08,
                ),
                0.16,
                [1.0, 0.95, 0.3],
            );
        }

        for mob in &world.mobs {
            append_cube(
                &mut vertices,
                mob.position,
                0.8,
                [0.8, 0.15 + (mob.health / 150.0), 0.15],
            );
            append_cube(
                &mut vertices,
                Point3::new(
                    mob.position.x + 0.2,
                    mob.position.y + 0.8,
                    mob.position.z + 0.2,
                ),
                0.2,
                [0.95, 0.95, 0.95],
            );
        }

        Self { vertices }
    }
}

fn append_chunk_mesh(vertices: &mut Vec<Vertex>, chunk: &Chunk) {
    for x in 0..CHUNK_SIZE as i32 {
        for y in 0..CHUNK_SIZE as i32 {
            for z in 0..CHUNK_SIZE as i32 {
                let block = chunk.get_i32(x, y, z);
                if block == Block::Air {
                    continue;
                }

                let color = match block {
                    Block::Grass => [0.25, 0.78, 0.22],
                    Block::Dirt => [0.55, 0.35, 0.2],
                    Block::Stone => [0.55, 0.58, 0.62],
                    Block::Sand => [0.88, 0.82, 0.55],
                    Block::Wood => [0.52, 0.34, 0.19],
                    Block::Planks => [0.72, 0.56, 0.34],
                    Block::Bricks => [0.68, 0.27, 0.24],
                    Block::Glass => [0.6, 0.85, 0.95],
                    Block::Leaf => [0.22, 0.56, 0.18],
                    Block::Snow => [0.92, 0.94, 0.98],
                    Block::Target => [0.9, 0.15, 0.15],
                    Block::Air => [0.0, 0.0, 0.0],
                };

                for (normal, quad) in cube_faces(x as f32, y as f32, z as f32, 1.0) {
                    let nx = x + normal[0];
                    let ny = y + normal[1];
                    let nz = z + normal[2];

                    if chunk.get_i32(nx, ny, nz) != Block::Air {
                        continue;
                    }

                    push_quad(vertices, quad, color, normal);
                }
            }
        }
    }
}

fn append_cube(vertices: &mut Vec<Vertex>, origin: Point3<f32>, size: f32, color: [f32; 3]) {
    for (normal, quad) in cube_faces(origin.x, origin.y, origin.z, size) {
        push_quad(vertices, quad, color, normal);
    }
}

fn push_quad(vertices: &mut Vec<Vertex>, quad: [[f32; 3]; 4], color: [f32; 3], normal: [i32; 3]) {
    let normal = [normal[0] as f32, normal[1] as f32, normal[2] as f32];
    vertices.extend_from_slice(&[
        Vertex {
            position: quad[0],
            color,
            normal,
        },
        Vertex {
            position: quad[1],
            color,
            normal,
        },
        Vertex {
            position: quad[2],
            color,
            normal,
        },
        Vertex {
            position: quad[2],
            color,
            normal,
        },
        Vertex {
            position: quad[3],
            color,
            normal,
        },
        Vertex {
            position: quad[0],
            color,
            normal,
        },
    ]);
}

fn cube_faces(x: f32, y: f32, z: f32, size: f32) -> [([i32; 3], [[f32; 3]; 4]); 6] {
    [
        (
            [0, 0, 1],
            [
                [x, y, z + size],
                [x + size, y, z + size],
                [x + size, y + size, z + size],
                [x, y + size, z + size],
            ],
        ),
        (
            [0, 0, -1],
            [
                [x + size, y, z],
                [x, y, z],
                [x, y + size, z],
                [x + size, y + size, z],
            ],
        ),
        (
            [1, 0, 0],
            [
                [x + size, y, z + size],
                [x + size, y, z],
                [x + size, y + size, z],
                [x + size, y + size, z + size],
            ],
        ),
        (
            [-1, 0, 0],
            [
                [x, y, z],
                [x, y, z + size],
                [x, y + size, z + size],
                [x, y + size, z],
            ],
        ),
        (
            [0, 1, 0],
            [
                [x, y + size, z + size],
                [x + size, y + size, z + size],
                [x + size, y + size, z],
                [x, y + size, z],
            ],
        ),
        (
            [0, -1, 0],
            [
                [x, y, z],
                [x + size, y, z],
                [x + size, y, z + size],
                [x, y, z + size],
            ],
        ),
    ]
}
