use crate::world::chunk::{Block, Chunk, CHUNK_SIZE};

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
    pub fn from_chunk(chunk: &Chunk) -> Self {
        let mut vertices = Vec::new();

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
                        Block::Air => [0.0, 0.0, 0.0],
                    };

                    for (normal, quad) in cube_faces(x as f32, y as f32, z as f32) {
                        let nx = x + normal[0];
                        let ny = y + normal[1];
                        let nz = z + normal[2];

                        if chunk.get_i32(nx, ny, nz) != Block::Air {
                            continue;
                        }

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
                }
            }
        }

        Self { vertices }
    }
}

fn cube_faces(x: f32, y: f32, z: f32) -> [([i32; 3], [[f32; 3]; 4]); 6] {
    [
        (
            [0, 0, 1],
            [
                [x, y, z + 1.0],
                [x + 1.0, y, z + 1.0],
                [x + 1.0, y + 1.0, z + 1.0],
                [x, y + 1.0, z + 1.0],
            ],
        ),
        (
            [0, 0, -1],
            [
                [x + 1.0, y, z],
                [x, y, z],
                [x, y + 1.0, z],
                [x + 1.0, y + 1.0, z],
            ],
        ),
        (
            [1, 0, 0],
            [
                [x + 1.0, y, z + 1.0],
                [x + 1.0, y, z],
                [x + 1.0, y + 1.0, z],
                [x + 1.0, y + 1.0, z + 1.0],
            ],
        ),
        (
            [-1, 0, 0],
            [
                [x, y, z],
                [x, y, z + 1.0],
                [x, y + 1.0, z + 1.0],
                [x, y + 1.0, z],
            ],
        ),
        (
            [0, 1, 0],
            [
                [x, y + 1.0, z + 1.0],
                [x + 1.0, y + 1.0, z + 1.0],
                [x + 1.0, y + 1.0, z],
                [x, y + 1.0, z],
            ],
        ),
        (
            [0, -1, 0],
            [
                [x, y, z],
                [x + 1.0, y, z],
                [x + 1.0, y, z + 1.0],
                [x, y, z + 1.0],
            ],
        ),
    ]
}
