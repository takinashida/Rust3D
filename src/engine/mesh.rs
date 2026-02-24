use cgmath::Point3;

use crate::world::{
    chunk::{Block, Chunk, CHUNK_DEPTH, CHUNK_HEIGHT, CHUNK_REGION_SIZE, CHUNK_WIDTH},
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
    pub fn dynamic_from_world(world: &World) -> Self {
        let mut vertices = Vec::new();

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

        for explosive in &world.explosives {
            append_cube(
                &mut vertices,
                Point3::new(
                    explosive.position.x - 0.24,
                    explosive.position.y - 0.24,
                    explosive.position.z - 0.24,
                ),
                0.48,
                [0.1, 0.1, 0.1],
            );
        }

        for particle in &world.particles {
            append_cube(
                &mut vertices,
                Point3::new(
                    particle.position.x - particle.size * 0.5,
                    particle.position.y - particle.size * 0.5,
                    particle.position.z - particle.size * 0.5,
                ),
                particle.size,
                particle.color,
            );
        }

        for mob in &world.mobs {
            append_box(
                &mut vertices,
                Point3::new(
                    mob.position.x - mob.radius,
                    mob.position.y,
                    mob.position.z - mob.radius,
                ),
                [mob.radius * 2.0, mob.height, mob.radius * 2.0],
                [0.8, 0.15 + (mob.health / 150.0), 0.15],
            );
            append_cube(
                &mut vertices,
                Point3::new(
                    mob.position.x + mob.radius * 0.4,
                    mob.position.y + mob.height * 0.45,
                    mob.position.z + mob.radius * 0.4,
                ),
                mob.radius * 0.5,
                [0.95, 0.95, 0.95],
            );
        }

        Self { vertices }
    }
}

pub fn mesh_region_dims() -> (usize, usize, usize) {
    (
        CHUNK_WIDTH / CHUNK_REGION_SIZE,
        CHUNK_HEIGHT / CHUNK_REGION_SIZE,
        CHUNK_DEPTH / CHUNK_REGION_SIZE,
    )
}

pub fn mesh_region_count() -> usize {
    let (rx, ry, rz) = mesh_region_dims();
    rx * ry * rz
}

pub fn region_index(rx: usize, ry: usize, rz: usize) -> usize {
    let (regions_x, _, regions_z) = mesh_region_dims();
    rx + rz * regions_x + ry * regions_x * regions_z
}

pub fn append_chunk_region_mesh(vertices: &mut Vec<Vertex>, chunk: &Chunk, region: [usize; 3]) {
    let (regions_x, regions_y, regions_z) = mesh_region_dims();
    let [rx, ry, rz] = region;
    if rx >= regions_x || ry >= regions_y || rz >= regions_z {
        return;
    }

    let x_start = (rx * CHUNK_REGION_SIZE) as i32;
    let y_start = (ry * CHUNK_REGION_SIZE) as i32;
    let z_start = (rz * CHUNK_REGION_SIZE) as i32;
    let x_end = ((rx + 1) * CHUNK_REGION_SIZE).min(CHUNK_WIDTH) as i32;
    let y_end = ((ry + 1) * CHUNK_REGION_SIZE).min(CHUNK_HEIGHT) as i32;
    let z_end = ((rz + 1) * CHUNK_REGION_SIZE).min(CHUNK_DEPTH) as i32;

    for x in x_start..x_end {
        for y in y_start..y_end {
            for z in z_start..z_end {
                let block = chunk.get_i32(x, y, z);
                if block == Block::Air {
                    continue;
                }

                let color = block_color(block);

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

fn block_color(block: Block) -> [f32; 3] {
    match block {
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
        Block::Water => [0.2, 0.45, 0.9],
        Block::Mud => [0.35, 0.27, 0.16],
        Block::Cobblestone => [0.42, 0.42, 0.44],
        Block::CoalOre => [0.25, 0.25, 0.25],
        Block::IronOre => [0.71, 0.56, 0.46],
        Block::GoldOre => [0.9, 0.75, 0.2],
        Block::DiamondOre => [0.35, 0.9, 0.85],
        Block::Gravel => [0.55, 0.53, 0.5],
        Block::Clay => [0.63, 0.67, 0.73],
        Block::Basalt => [0.18, 0.19, 0.22],
        Block::Moss => [0.18, 0.47, 0.2],
        Block::RedSand => [0.75, 0.42, 0.25],
        Block::Ice => [0.75, 0.92, 1.0],
        Block::Cactus => [0.2, 0.58, 0.24],
        Block::Air => [0.0, 0.0, 0.0],
    }
}

fn append_cube(vertices: &mut Vec<Vertex>, origin: Point3<f32>, size: f32, color: [f32; 3]) {
    append_box(vertices, origin, [size, size, size], color);
}

fn append_box(vertices: &mut Vec<Vertex>, origin: Point3<f32>, size: [f32; 3], color: [f32; 3]) {
    for (normal, quad) in cuboid_faces(origin.x, origin.y, origin.z, size[0], size[1], size[2]) {
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
    cuboid_faces(x, y, z, size, size, size)
}

fn cuboid_faces(
    x: f32,
    y: f32,
    z: f32,
    sx: f32,
    sy: f32,
    sz: f32,
) -> [([i32; 3], [[f32; 3]; 4]); 6] {
    [
        (
            [0, 0, 1],
            [
                [x, y, z + sz],
                [x + sx, y, z + sz],
                [x + sx, y + sy, z + sz],
                [x, y + sy, z + sz],
            ],
        ),
        (
            [0, 0, -1],
            [
                [x + sx, y, z],
                [x, y, z],
                [x, y + sy, z],
                [x + sx, y + sy, z],
            ],
        ),
        (
            [1, 0, 0],
            [
                [x + sx, y, z + sz],
                [x + sx, y, z],
                [x + sx, y + sy, z],
                [x + sx, y + sy, z + sz],
            ],
        ),
        (
            [-1, 0, 0],
            [
                [x, y, z],
                [x, y, z + sz],
                [x, y + sy, z + sz],
                [x, y + sy, z],
            ],
        ),
        (
            [0, 1, 0],
            [
                [x, y + sy, z + sz],
                [x + sx, y + sy, z + sz],
                [x + sx, y + sy, z],
                [x, y + sy, z],
            ],
        ),
        (
            [0, -1, 0],
            [
                [x, y, z],
                [x + sx, y, z],
                [x + sx, y, z + sz],
                [x, y, z + sz],
            ],
        ),
    ]
}
