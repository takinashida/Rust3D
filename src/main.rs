use std::collections::{HashMap, HashSet};

use bevy::{
    input::mouse::MouseMotion,
    math::primitives::Cuboid,
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow},
};
use noise::{NoiseFn, Perlin};

const CHUNK_SIZE: i32 = 16;
const RENDER_DISTANCE_CHUNKS: i32 = 4;
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

#[derive(Resource, Default)]
struct WorldBlocks {
    map: HashMap<IVec3, (Entity, BlockType)>,
    chunks: HashMap<IVec2, Vec<IVec3>>,
}

#[derive(Resource)]
struct WorldGenerator {
    noise: Perlin,
    mesh: Handle<Mesh>,
    generated_chunks: HashSet<IVec2>,
}

#[derive(Clone, Copy)]
enum BlockType {
    Grass,
    Dirt,
    Stone,
}

#[derive(Component)]
struct Block;

#[derive(Component)]
struct Player {
    yaw: f32,
    pitch: f32,
}

#[derive(Resource)]
struct BlockMaterials {
    grass: Handle<StandardMaterial>,
    dirt: Handle<StandardMaterial>,
    stone: Handle<StandardMaterial>,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let block_mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));

    let block_materials = BlockMaterials {
        grass: materials.add(Color::srgb(0.3, 0.7, 0.25)),
        dirt: materials.add(Color::srgb(0.45, 0.3, 0.16)),
        stone: materials.add(Color::srgb(0.5, 0.5, 0.55)),
    };

    commands.insert_resource(WorldGenerator {
        noise: Perlin::new(1337),
        mesh: block_mesh,
        generated_chunks: HashSet::new(),
    });

    commands.insert_resource(block_materials);

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
    mut world: ResMut<WorldBlocks>,
    mut world_gen: ResMut<WorldGenerator>,
    materials: Res<BlockMaterials>,
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

    for &chunk in &required_chunks {
        if world_gen.generated_chunks.contains(&chunk) {
            continue;
        }

        generate_chunk(
            &mut commands,
            &mut world,
            &world_gen,
            &materials,
            chunk,
            player_pos,
        );
        world_gen.generated_chunks.insert(chunk);
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
    commands: &mut Commands,
    world: &mut WorldBlocks,
    world_gen: &WorldGenerator,
    materials: &BlockMaterials,
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

                spawn_block(
                    commands,
                    world,
                    &world_gen.mesh,
                    materials,
                    position,
                    block_type,
                );
                positions.push(position);
            }
        }
    }

    world.chunks.insert(chunk, positions);
}

fn unload_chunk(
    commands: &mut Commands,
    world: &mut WorldBlocks,
    world_gen: &mut WorldGenerator,
    chunk: IVec2,
) {
    if let Some(blocks) = world.chunks.remove(&chunk) {
        for position in blocks {
            if let Some((entity, _)) = world.map.remove(&position) {
                commands.entity(entity).despawn_recursive();
            }
        }
    }

    world_gen.generated_chunks.remove(&chunk);
}

fn is_player_air_cell(position: IVec3, player_position: IVec3) -> bool {
    let dx = (position.x - player_position.x).abs();
    let dz = (position.z - player_position.z).abs();
    let in_horizontal = dx <= PLAYER_AIR_RADIUS && dz <= PLAYER_AIR_RADIUS;
    let in_vertical =
        position.y >= player_position.y - 1 && position.y <= player_position.y + PLAYER_AIR_HEIGHT;

    in_horizontal && in_vertical
}

fn material_for(block_type: BlockType, materials: &BlockMaterials) -> Handle<StandardMaterial> {
    match block_type {
        BlockType::Grass => materials.grass.clone(),
        BlockType::Dirt => materials.dirt.clone(),
        BlockType::Stone => materials.stone.clone(),
    }
}

fn spawn_block(
    commands: &mut Commands,
    world: &mut WorldBlocks,
    mesh: &Handle<Mesh>,
    materials: &BlockMaterials,
    position: IVec3,
    block_type: BlockType,
) {
    let entity = commands
        .spawn((
            PbrBundle {
                mesh: mesh.clone(),
                material: material_for(block_type, materials),
                transform: Transform::from_translation(position.as_vec3()),
                ..default()
            },
            Block,
        ))
        .id();

    world.map.insert(position, (entity, block_type));
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
    mut world: ResMut<WorldBlocks>,
    block_materials: Res<BlockMaterials>,
    world_gen: Res<WorldGenerator>,
    camera: Query<&Transform, With<Player>>,
) {
    if !mouse.just_pressed(MouseButton::Left) && !mouse.just_pressed(MouseButton::Right) {
        return;
    }

    let camera = camera.single();
    let origin = camera.translation;
    let direction = camera.forward();

    let mut previous_cell: Option<IVec3> = None;
    let mut hit_cell: Option<IVec3> = None;

    let step = 0.1;
    let steps = (REACH_DISTANCE / step) as i32;

    for i in 0..=steps {
        let point = origin + *direction * (i as f32 * step);
        let cell = point.round().as_ivec3();

        if world.map.contains_key(&cell) {
            hit_cell = Some(cell);
            break;
        }

        previous_cell = Some(cell);
    }

    if mouse.just_pressed(MouseButton::Left) {
        if let Some(cell) = hit_cell {
            if let Some((entity, _)) = world.map.remove(&cell) {
                commands.entity(entity).despawn_recursive();
                if let Some(chunk_blocks) = world.chunks.get_mut(&world_to_chunk(cell)) {
                    chunk_blocks.retain(|&p| p != cell);
                }
            }
        }
    }

    if mouse.just_pressed(MouseButton::Right) && hit_cell.is_some() {
        if let Some(place_pos) = previous_cell {
            if !world.map.contains_key(&place_pos) {
                spawn_block(
                    &mut commands,
                    &mut world,
                    &world_gen.mesh,
                    &block_materials,
                    place_pos,
                    BlockType::Grass,
                );
                world
                    .chunks
                    .entry(world_to_chunk(place_pos))
                    .or_default()
                    .push(place_pos);
            }
        }
    }
}
