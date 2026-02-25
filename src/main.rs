use std::collections::{HashMap, HashSet};

use bevy::{
    asset::RenderAssetUsages,
    input::mouse::MouseMotion,
    prelude::*,
    render::mesh::{Indices, PrimitiveTopology},
    window::{CursorGrabMode, PrimaryWindow},
};
use noise::{NoiseFn, Perlin};

const CHUNK_SIZE: i32 = 16;
const RENDER_DISTANCE_CHUNKS: i32 = 4;
const MAX_CHUNK_GENERATES_PER_FRAME: usize = 2;
const MIN_HEIGHT: i32 = 2;
const MAX_HEIGHT: i32 = 14;
const REACH_DISTANCE: f32 = 6.0;
const PLAYER_SPEED: f32 = 9.0;
const MOUSE_SENSITIVITY: f32 = 0.003;
const PLAYER_AIR_RADIUS: i32 = 1;
const PLAYER_AIR_HEIGHT: i32 = 2;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.55, 0.8, 0.95)))
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 450.0,
        })
        .insert_resource(WorldBlocks::default())
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "RustCraft (Bevy)".to_string(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                lock_cursor_on_click,
                player_look,
                player_movement,
                stream_world_around_player,
                block_interaction,
            ),
        )
        .run();
}

#[derive(Default)]
struct ChunkData {
    entity: Option<Entity>,
    blocks: Vec<IVec3>,
}

#[derive(Resource, Default)]
struct WorldBlocks {
    map: HashMap<IVec3, BlockType>,
    chunks: HashMap<IVec2, ChunkData>,
}

#[derive(Resource)]
struct WorldGenerator {
    noise: Perlin,
    generated_chunks: HashSet<IVec2>,
}

#[derive(Clone, Copy)]
enum BlockType {
    Grass,
    Dirt,
    Stone,
}

#[derive(Component)]
struct BlockChunk;

#[derive(Component)]
struct Player {
    yaw: f32,
    pitch: f32,
}

#[derive(Resource)]
struct BlockRenderResources {
    material: Handle<StandardMaterial>,
}

fn setup(mut commands: Commands, mut materials: ResMut<Assets<StandardMaterial>>) {
    let block_material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        perceptual_roughness: 0.95,
        ..default()
    });

    commands.insert_resource(WorldGenerator {
        noise: Perlin::new(1337),
        generated_chunks: HashSet::new(),
    });

    commands.insert_resource(BlockRenderResources {
        material: block_material,
    });

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            illuminance: 20_000.0,
            ..default()
        },
        transform: Transform::from_xyz(20.0, 40.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    let camera_transform =
        Transform::from_xyz(0.0, 18.0, 24.0).looking_at(Vec3::new(0.0, 5.0, 0.0), Vec3::Y);

    let (yaw, pitch, _) = camera_transform.rotation.to_euler(EulerRot::YXZ);

    commands.spawn((
        Camera3dBundle {
            transform: camera_transform,
            ..default()
        },
        Player { yaw, pitch },
    ));

    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            background_color: Color::NONE.into(),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn(NodeBundle {
                style: Style {
                    width: Val::Px(4.0),
                    height: Val::Px(4.0),
                    ..default()
                },
                background_color: Color::BLACK.with_alpha(0.75).into(),
                ..default()
            });
        });
}

fn stream_world_around_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut world: ResMut<WorldBlocks>,
    mut world_gen: ResMut<WorldGenerator>,
    render: Res<BlockRenderResources>,
    player: Query<&Transform, With<Player>>,
) {
    let player_pos = player.single().translation.round().as_ivec3();
    let center_chunk = world_to_chunk(player_pos);

    let mut required_chunks = HashSet::new();
    for cx in -RENDER_DISTANCE_CHUNKS..=RENDER_DISTANCE_CHUNKS {
        for cz in -RENDER_DISTANCE_CHUNKS..=RENDER_DISTANCE_CHUNKS {
            required_chunks.insert(center_chunk + IVec2::new(cx, cz));
        }
    }

    let mut generated_this_frame = 0;
    for &chunk in &required_chunks {
        if generated_this_frame >= MAX_CHUNK_GENERATES_PER_FRAME {
            break;
        }
        if world_gen.generated_chunks.contains(&chunk) {
            continue;
        }

        generate_chunk(&mut world, &world_gen, chunk, player_pos);
        world_gen.generated_chunks.insert(chunk);
        generated_this_frame += 1;

        rebuild_chunk_and_neighbors(&mut commands, &mut meshes, &mut world, &render, chunk);
    }

    let obsolete_chunks: Vec<IVec2> = world_gen
        .generated_chunks
        .iter()
        .copied()
        .filter(|chunk| !required_chunks.contains(chunk))
        .collect();

    for chunk in obsolete_chunks {
        unload_chunk(&mut commands, &mut world, &mut world_gen, chunk);
    }
}

fn world_to_chunk(position: IVec3) -> IVec2 {
    IVec2::new(
        position.x.div_euclid(CHUNK_SIZE),
        position.z.div_euclid(CHUNK_SIZE),
    )
}

fn chunk_to_world_min(chunk: IVec2) -> IVec2 {
    IVec2::new(chunk.x * CHUNK_SIZE, chunk.y * CHUNK_SIZE)
}

fn generate_chunk(
    world: &mut WorldBlocks,
    world_gen: &WorldGenerator,
    chunk: IVec2,
    player_position: IVec3,
) {
    let min = chunk_to_world_min(chunk);
    let mut positions = Vec::with_capacity((CHUNK_SIZE * CHUNK_SIZE * (MAX_HEIGHT + 1)) as usize);

    for x in min.x..(min.x + CHUNK_SIZE) {
        for z in min.y..(min.y + CHUNK_SIZE) {
            let sample = world_gen.noise.get([x as f64 * 0.08, z as f64 * 0.08]) as f32;
            let normalized = (sample + 1.0) * 0.5;
            let height =
                MIN_HEIGHT + ((MAX_HEIGHT - MIN_HEIGHT) as f32 * normalized).round() as i32;

            for y in 0..=height {
                let position = IVec3::new(x, y, z);
                if is_player_air_cell(position, player_position) {
                    continue;
                }

                let block_type = if y == height {
                    BlockType::Grass
                } else if y > height - 3 {
                    BlockType::Dirt
                } else {
                    BlockType::Stone
                };

                world.map.insert(position, block_type);
                positions.push(position);
            }
        }
    }

    world
        .chunks
        .entry(chunk)
        .and_modify(|data| data.blocks = positions.clone())
        .or_insert(ChunkData {
            entity: None,
            blocks: positions,
        });
}

fn unload_chunk(
    commands: &mut Commands,
    world: &mut WorldBlocks,
    world_gen: &mut WorldGenerator,
    chunk: IVec2,
) {
    if let Some(chunk_data) = world.chunks.remove(&chunk) {
        if let Some(entity) = chunk_data.entity {
            commands.entity(entity).despawn_recursive();
        }

        for position in chunk_data.blocks {
            world.map.remove(&position);
        }
    }

    world_gen.generated_chunks.remove(&chunk);
}

fn chunk_neighbors_inclusive(chunk: IVec2) -> [IVec2; 5] {
    [
        chunk,
        chunk + IVec2::new(1, 0),
        chunk + IVec2::new(-1, 0),
        chunk + IVec2::new(0, 1),
        chunk + IVec2::new(0, -1),
    ]
}

fn rebuild_chunk_and_neighbors(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    world: &mut WorldBlocks,
    render: &BlockRenderResources,
    center: IVec2,
) {
    for chunk in chunk_neighbors_inclusive(center) {
        rebuild_chunk_mesh(commands, meshes, world, render, chunk);
    }
}

fn rebuild_chunk_mesh(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    world: &mut WorldBlocks,
    render: &BlockRenderResources,
    chunk: IVec2,
) {
    let Some(chunk_data) = world.chunks.get_mut(&chunk) else {
        return;
    };

    let mesh = build_chunk_mesh(&world.map, &chunk_data.blocks);

    if let Some(existing_entity) = chunk_data.entity.take() {
        commands.entity(existing_entity).despawn_recursive();
    }

    if let Some(mesh) = mesh {
        let mesh_handle = meshes.add(mesh);
        let entity = commands
            .spawn((
                PbrBundle {
                    mesh: mesh_handle,
                    material: render.material.clone(),
                    ..default()
                },
                BlockChunk,
            ))
            .id();
        chunk_data.entity = Some(entity);
    }
}

fn build_chunk_mesh(map: &HashMap<IVec3, BlockType>, blocks: &[IVec3]) -> Option<Mesh> {
    if blocks.is_empty() {
        return None;
    }

    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut colors: Vec<[f32; 4]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    for &pos in blocks {
        let Some(block_type) = map.get(&pos).copied() else {
            continue;
        };

        for (normal, face) in cube_faces(pos) {
            if map.contains_key(&(pos + normal)) {
                continue;
            }

            let base = positions.len() as u32;
            let n = normal.as_vec3();
            let color = block_color(block_type).to_linear().to_f32_array();

            for vertex in face {
                positions.push(vertex);
                normals.push([n.x, n.y, n.z]);
                colors.push(color);
            }

            indices.extend_from_slice(&[base, base + 2, base + 1, base, base + 3, base + 2]);
        }
    }

    if indices.is_empty() {
        return None;
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(Indices::U32(indices));
    Some(mesh)
}

fn cube_faces(position: IVec3) -> [(IVec3, [[f32; 3]; 4]); 6] {
    let x = position.x as f32;
    let y = position.y as f32;
    let z = position.z as f32;

    [
        (
            IVec3::X,
            [
                [x + 0.5, y - 0.5, z - 0.5],
                [x + 0.5, y - 0.5, z + 0.5],
                [x + 0.5, y + 0.5, z + 0.5],
                [x + 0.5, y + 0.5, z - 0.5],
            ],
        ),
        (
            IVec3::NEG_X,
            [
                [x - 0.5, y - 0.5, z + 0.5],
                [x - 0.5, y - 0.5, z - 0.5],
                [x - 0.5, y + 0.5, z - 0.5],
                [x - 0.5, y + 0.5, z + 0.5],
            ],
        ),
        (
            IVec3::Y,
            [
                [x - 0.5, y + 0.5, z - 0.5],
                [x + 0.5, y + 0.5, z - 0.5],
                [x + 0.5, y + 0.5, z + 0.5],
                [x - 0.5, y + 0.5, z + 0.5],
            ],
        ),
        (
            IVec3::NEG_Y,
            [
                [x - 0.5, y - 0.5, z + 0.5],
                [x + 0.5, y - 0.5, z + 0.5],
                [x + 0.5, y - 0.5, z - 0.5],
                [x - 0.5, y - 0.5, z - 0.5],
            ],
        ),
        (
            IVec3::Z,
            [
                [x + 0.5, y - 0.5, z + 0.5],
                [x - 0.5, y - 0.5, z + 0.5],
                [x - 0.5, y + 0.5, z + 0.5],
                [x + 0.5, y + 0.5, z + 0.5],
            ],
        ),
        (
            IVec3::NEG_Z,
            [
                [x - 0.5, y - 0.5, z - 0.5],
                [x + 0.5, y - 0.5, z - 0.5],
                [x + 0.5, y + 0.5, z - 0.5],
                [x - 0.5, y + 0.5, z - 0.5],
            ],
        ),
    ]
}

fn block_color(block_type: BlockType) -> Color {
    match block_type {
        BlockType::Grass => Color::srgb(0.3, 0.7, 0.25),
        BlockType::Dirt => Color::srgb(0.45, 0.3, 0.16),
        BlockType::Stone => Color::srgb(0.5, 0.5, 0.55),
    }
}

fn is_player_air_cell(position: IVec3, player_position: IVec3) -> bool {
    let dx = (position.x - player_position.x).abs();
    let dz = (position.z - player_position.z).abs();
    let in_horizontal = dx <= PLAYER_AIR_RADIUS && dz <= PLAYER_AIR_RADIUS;
    let in_vertical =
        position.y >= player_position.y - 1 && position.y <= player_position.y + PLAYER_AIR_HEIGHT;

    in_horizontal && in_vertical
}

fn lock_cursor_on_click(
    mouse: Res<ButtonInput<MouseButton>>,
    key: Res<ButtonInput<KeyCode>>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    let mut window = windows.single_mut();

    if mouse.just_pressed(MouseButton::Left) {
        window.cursor.grab_mode = CursorGrabMode::Locked;
        window.cursor.visible = false;
    }

    if key.just_pressed(KeyCode::Escape) {
        window.cursor.grab_mode = CursorGrabMode::None;
        window.cursor.visible = true;
    }
}

fn player_look(
    mut mouse_motion: EventReader<MouseMotion>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut query: Query<(&mut Transform, &mut Player)>,
) {
    let window = windows.single();

    if window.cursor.grab_mode != CursorGrabMode::Locked {
        mouse_motion.clear();
        return;
    }

    let delta = mouse_motion
        .read()
        .fold(Vec2::ZERO, |acc, evt| acc + evt.delta);
    if delta == Vec2::ZERO {
        return;
    }

    let (mut transform, mut player) = query.single_mut();
    player.yaw -= delta.x * MOUSE_SENSITIVITY;
    player.pitch -= delta.y * MOUSE_SENSITIVITY;
    player.pitch = player.pitch.clamp(-1.54, 1.54);

    transform.rotation = Quat::from_euler(EulerRot::YXZ, player.yaw, player.pitch, 0.0);
}

fn player_movement(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut query: Query<&mut Transform, With<Player>>,
) {
    let window = windows.single();
    if window.cursor.grab_mode != CursorGrabMode::Locked {
        return;
    }

    let mut transform = query.single_mut();

    let forward = transform.forward();
    let right = transform.right();

    let mut direction = Vec3::ZERO;

    if keyboard.pressed(KeyCode::KeyW) {
        direction += *forward;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        direction -= *forward;
    }
    if keyboard.pressed(KeyCode::KeyA) {
        direction -= *right;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        direction += *right;
    }
    if keyboard.pressed(KeyCode::Space) {
        direction += Vec3::Y;
    }
    if keyboard.pressed(KeyCode::ShiftLeft) {
        direction -= Vec3::Y;
    }

    if direction.length_squared() > 0.0 {
        transform.translation += direction.normalize() * PLAYER_SPEED * time.delta_seconds();
    }
}

fn block_interaction(
    mouse: Res<ButtonInput<MouseButton>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut world: ResMut<WorldBlocks>,
    render: Res<BlockRenderResources>,
    camera: Query<&Transform, With<Player>>,
) {
    if !mouse.just_pressed(MouseButton::Left) && !mouse.just_pressed(MouseButton::Right) {
        return;
    }

    let camera = camera.single();
    let origin = camera.translation;
    let direction = *camera.forward();

    let mut previous_cell: Option<IVec3> = None;
    let mut hit_cell: Option<IVec3> = None;

    let step = 0.1;
    let steps = (REACH_DISTANCE / step) as i32;

    for i in 0..=steps {
        let point = origin + direction * (i as f32 * step);
        let cell = point.round().as_ivec3();

        if world.map.contains_key(&cell) {
            hit_cell = Some(cell);
            break;
        }

        previous_cell = Some(cell);
    }

    let mut dirty_chunks = HashSet::new();

    if mouse.just_pressed(MouseButton::Left) {
        if let Some(cell) = hit_cell {
            if world.map.remove(&cell).is_some() {
                let chunk = world_to_chunk(cell);
                if let Some(chunk_data) = world.chunks.get_mut(&chunk) {
                    chunk_data.blocks.retain(|&p| p != cell);
                }
                dirty_chunks.insert(chunk);
                dirty_chunks.extend(chunk_neighbors_inclusive(chunk));
            }
        }
    }

    if mouse.just_pressed(MouseButton::Right) && hit_cell.is_some() {
        if let Some(place_pos) = previous_cell {
            if !world.map.contains_key(&place_pos) {
                world.map.insert(place_pos, BlockType::Grass);
                let chunk = world_to_chunk(place_pos);
                world
                    .chunks
                    .entry(chunk)
                    .or_default()
                    .blocks
                    .push(place_pos);
                dirty_chunks.insert(chunk);
                dirty_chunks.extend(chunk_neighbors_inclusive(chunk));
            }
        }
    }

    for chunk in dirty_chunks {
        rebuild_chunk_mesh(&mut commands, &mut meshes, &mut world, &render, chunk);
    }
}
