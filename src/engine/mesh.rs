use crate::world::chunk::{Chunk, Block, CHUNK_SIZE};

pub struct Mesh {
    pub vertices: Vec<[f32; 3]>,
}

impl Mesh {
    pub fn from_chunk(chunk: &Chunk) -> Self {
        let mut vertices = Vec::new();

        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    if let Block::Air = chunk.get(x,y,z) {
                        continue;
                    }

                    // Добавляем куб (упрощённо)
                    vertices.extend(cube_vertices(
                        x as f32,
                        y as f32,
                        z as f32,
                    ));
                }
            }
        }

        Self { vertices }
    }
}

fn cube_vertices(x: f32, y: f32, z: f32) -> Vec<[f32; 3]> {
    vec![
        [x, y, z],
        [x+1.0, y, z],
        [x+1.0, y+1.0, z],
        // ... остальные вершины
    ]
}