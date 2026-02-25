use std::collections::HashMap;

use bevy::{
    input::mouse::MouseMotion,
    math::primitives::Cuboid,
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow},
};
use noise::{NoiseFn, Perlin};

const WORLD_HALF_EXTENT: i32 = 20;
const MIN_HEIGHT: i32 = 2;
const MAX_HEIGHT: i32 = 9;
const REACH_DISTANCE: f32 = 6.0;
const PLAYER_SPEED: f32 = 9.0;
const MOUSE_SENSITIVITY: f32 = 0.003;

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
                block_interaction,
            ),
        )
        .run();
}

#[derive(Resource, Default)]
struct WorldBlocks {
    map: HashMap<IVec3, (Entity, BlockType)>,
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
    mut world: ResMut<WorldBlocks>,
) {
    let block_mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));

    let block_materials = BlockMaterials {
        grass: materials.add(Color::srgb(0.3, 0.7, 0.25)),
        dirt: materials.add(Color::srgb(0.45, 0.3, 0.16)),
        stone: materials.add(Color::srgb(0.5, 0.5, 0.55)),
    };

    generate_world(&mut commands, &mut world, &block_mesh, &block_materials);

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

    // Tiny crosshair
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

fn generate_world(
    commands: &mut Commands,
    world: &mut WorldBlocks,
    mesh: &Handle<Mesh>,
    materials: &BlockMaterials,
) {
    let noise = Perlin::new(1337);

    for x in -WORLD_HALF_EXTENT..=WORLD_HALF_EXTENT {
        for z in -WORLD_HALF_EXTENT..=WORLD_HALF_EXTENT {
            let sample = noise.get([x as f64 * 0.08, z as f64 * 0.08]) as f32;
            let normalized = (sample + 1.0) * 0.5;
            let height =
                MIN_HEIGHT + ((MAX_HEIGHT - MIN_HEIGHT) as f32 * normalized).round() as i32;

            for y in 0..=height {
                let position = IVec3::new(x, y, z);
                let block_type = if y == height {
                    BlockType::Grass
                } else if y > height - 3 {
                    BlockType::Dirt
                } else {
                    BlockType::Stone
                };

                spawn_block(commands, world, mesh, materials, position, block_type);
            }
        }
    }
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
    mut meshes: ResMut<Assets<Mesh>>,
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
            }
        }
    }

    if mouse.just_pressed(MouseButton::Right) {
        if hit_cell.is_some() {
            if let Some(place_pos) = previous_cell {
                if !world.map.contains_key(&place_pos) {
                    let mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
                    spawn_block(
                        &mut commands,
                        &mut world,
                        &mesh,
                        &block_materials,
                        place_pos,
                        BlockType::Grass,
                    );
                }
            }
        }
    }
}
