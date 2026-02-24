mod engine;
mod world;

use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use engine::{camera::Camera, input::InputState, renderer::Renderer};
use world::{chunk::Block, world::World};

use winit::{
    event::{DeviceEvent, ElementState, Event, MouseButton, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{CursorGrabMode, WindowBuilder},
};

#[derive(Clone, Copy)]
enum InventoryItem {
    Block(Block),
    Pistol,
    Explosive,
}

impl InventoryItem {
    fn color(&self) -> [f32; 3] {
        match self {
            InventoryItem::Pistol => [0.15, 0.15, 0.18],
            InventoryItem::Explosive => [0.08, 0.08, 0.08],
            InventoryItem::Block(Block::Grass) => [0.25, 0.78, 0.22],
            InventoryItem::Block(Block::Dirt) => [0.55, 0.35, 0.2],
            InventoryItem::Block(Block::Stone) => [0.55, 0.58, 0.62],
            InventoryItem::Block(Block::Sand) => [0.88, 0.82, 0.55],
            InventoryItem::Block(Block::Wood) => [0.52, 0.34, 0.19],
            InventoryItem::Block(Block::Planks) => [0.72, 0.56, 0.34],
            InventoryItem::Block(Block::Bricks) => [0.68, 0.27, 0.24],
            InventoryItem::Block(Block::Glass) => [0.6, 0.85, 0.95],
            InventoryItem::Block(Block::Leaf) => [0.22, 0.56, 0.18],
            InventoryItem::Block(Block::Snow) => [0.92, 0.94, 0.98],
            InventoryItem::Block(Block::Target) => [0.9, 0.15, 0.15],
            InventoryItem::Block(Block::Water) => [0.2, 0.45, 0.9],
            InventoryItem::Block(Block::Mud) => [0.35, 0.27, 0.16],
            InventoryItem::Block(Block::Cobblestone) => [0.42, 0.42, 0.44],
            InventoryItem::Block(Block::CoalOre) => [0.25, 0.25, 0.25],
            InventoryItem::Block(Block::IronOre) => [0.71, 0.56, 0.46],
            InventoryItem::Block(Block::GoldOre) => [0.9, 0.75, 0.2],
            InventoryItem::Block(Block::DiamondOre) => [0.35, 0.9, 0.85],
            InventoryItem::Block(Block::Gravel) => [0.55, 0.53, 0.5],
            InventoryItem::Block(Block::Clay) => [0.63, 0.67, 0.73],
            InventoryItem::Block(Block::Basalt) => [0.18, 0.19, 0.22],
            InventoryItem::Block(Block::Moss) => [0.18, 0.47, 0.2],
            InventoryItem::Block(Block::RedSand) => [0.75, 0.42, 0.25],
            InventoryItem::Block(Block::Ice) => [0.75, 0.92, 1.0],
            InventoryItem::Block(Block::Cactus) => [0.2, 0.58, 0.24],
            InventoryItem::Block(Block::Air) => [0.0, 0.0, 0.0],
        }
    }
}

const INVENTORY: [InventoryItem; 28] = [
    InventoryItem::Pistol,
    InventoryItem::Explosive,
    InventoryItem::Block(Block::Grass),
    InventoryItem::Block(Block::Dirt),
    InventoryItem::Block(Block::Stone),
    InventoryItem::Block(Block::Sand),
    InventoryItem::Block(Block::Wood),
    InventoryItem::Block(Block::Planks),
    InventoryItem::Block(Block::Bricks),
    InventoryItem::Block(Block::Glass),
    InventoryItem::Block(Block::Leaf),
    InventoryItem::Block(Block::Snow),
    InventoryItem::Block(Block::Water),
    InventoryItem::Block(Block::Mud),
    InventoryItem::Block(Block::Cobblestone),
    InventoryItem::Block(Block::CoalOre),
    InventoryItem::Block(Block::IronOre),
    InventoryItem::Block(Block::GoldOre),
    InventoryItem::Block(Block::DiamondOre),
    InventoryItem::Block(Block::Gravel),
    InventoryItem::Block(Block::Clay),
    InventoryItem::Block(Block::Basalt),
    InventoryItem::Block(Block::Moss),
    InventoryItem::Block(Block::RedSand),
    InventoryItem::Block(Block::Ice),
    InventoryItem::Block(Block::Cactus),
    InventoryItem::Block(Block::Target),
    InventoryItem::Block(Block::Air),
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum AppMode {
    MainMenu,
    Playing,
}

fn main() {
    pollster::block_on(run());
}

fn set_mouse_lock(window: &winit::window::Window, locked: bool) {
    let mode = if locked {
        CursorGrabMode::Locked
    } else {
        CursorGrabMode::None
    };

    if window.set_cursor_grab(mode).is_err() && locked {
        let _ = window.set_cursor_grab(CursorGrabMode::Confined);
    }

    window.set_cursor_visible(!locked);
}

fn inventory_colors(page: usize) -> [[f32; 3]; 10] {
    std::array::from_fn(|i| {
        let idx = page * 10 + i;
        if idx < INVENTORY.len() {
            INVENTORY[idx].color()
        } else {
            [0.08, 0.08, 0.08]
        }
    })
}

async fn run() {
    let event_loop = EventLoop::new().expect("event loop");
    let window = Arc::new(
        WindowBuilder::new()
            .with_title("VoxelCraft: Press Enter to Start")
            .build(&event_loop)
            .expect("window"),
    );

    let mut world = World::new();
    let mut camera = Camera::new();
    let size = window.inner_size();
    camera.aspect = (size.width.max(1) as f32) / (size.height.max(1) as f32);
    let mut input = InputState::new();
    let mut selected_hotbar = 0usize;
    let mut inventory_page = 0usize;
    let mut player_health = 100.0f32;
    let mut game_over = false;
    let mut fps_display = 60u32;
    let mut fps_counter = 0u32;
    let mut fps_last_sample = Instant::now();
    let mut mode = AppMode::MainMenu;

    set_mouse_lock(&window, false);

    let mut renderer = Renderer::new(window.clone()).await;
    renderer.build_chunk_mesh(&world.chunk);
    let _ = world.chunk.take_dirty_regions();
    renderer.build_dynamic_mesh(&world);
    renderer.update_hotbar(
        selected_hotbar,
        &inventory_colors(inventory_page),
        player_health / 100.0,
        fps_display,
    );

    let frame_time = Duration::from_millis(16);

    let _ = event_loop.run(move |event, target| match event {
        Event::DeviceEvent {
            event: DeviceEvent::MouseMotion { delta },
            ..
        } => {
            if mode == AppMode::Playing && !game_over {
                camera.process_mouse(delta.0, delta.1);
            }
        }
        Event::WindowEvent { event, window_id } if window_id == window.id() => match event {
            WindowEvent::CloseRequested => target.exit(),
            WindowEvent::Resized(size) => {
                renderer.resize(size);
                camera.aspect = (size.width.max(1) as f32) / (size.height.max(1) as f32);
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(code) = event.physical_key {
                    if code == KeyCode::Escape && event.state == ElementState::Pressed {
                        if mode == AppMode::Playing {
                            mode = AppMode::MainMenu;
                            window.set_title("VoxelCraft: Press Enter to Start");
                            set_mouse_lock(&window, false);
                        } else {
                            target.exit();
                        }
                    }

                    if mode == AppMode::MainMenu
                        && code == KeyCode::Enter
                        && event.state == ElementState::Pressed
                    {
                        mode = AppMode::Playing;
                        window.set_title("VoxelCraft Sandbox");
                        set_mouse_lock(&window, true);
                    }

                    if mode == AppMode::Playing && event.state == ElementState::Pressed {
                        if code == KeyCode::KeyQ {
                            inventory_page = inventory_page.saturating_sub(1);
                            selected_hotbar = selected_hotbar.min(9);
                        }
                        if code == KeyCode::KeyE {
                            let max_page = (INVENTORY.len() - 1) / 10;
                            inventory_page = (inventory_page + 1).min(max_page);
                            selected_hotbar = selected_hotbar.min(9);
                        }
                        if let Some(digit) = match code {
                            KeyCode::Digit1 => Some(0),
                            KeyCode::Digit2 => Some(1),
                            KeyCode::Digit3 => Some(2),
                            KeyCode::Digit4 => Some(3),
                            KeyCode::Digit5 => Some(4),
                            KeyCode::Digit6 => Some(5),
                            KeyCode::Digit7 => Some(6),
                            KeyCode::Digit8 => Some(7),
                            KeyCode::Digit9 => Some(8),
                            KeyCode::Digit0 => Some(9),
                            _ => None,
                        } {
                            selected_hotbar = digit;
                        }

                        renderer.update_hotbar(
                            selected_hotbar,
                            &inventory_colors(inventory_page),
                            player_health / 100.0,
                            fps_display,
                        );
                    }
                }
                input.update(&event);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if game_over || mode != AppMode::Playing {
                    return;
                }

                if state == ElementState::Pressed {
                    let inventory_index = inventory_page * 10 + selected_hotbar;
                    if inventory_index >= INVENTORY.len() {
                        return;
                    }
                    match button {
                        MouseButton::Left => match INVENTORY[inventory_index] {
                            InventoryItem::Pistol => {
                                world.spawn_bullet(camera.position, camera.look_direction());
                                renderer.build_dynamic_mesh(&world);
                            }
                            InventoryItem::Explosive => {
                                world.spawn_explosive(camera.position, camera.look_direction());
                                renderer.build_dynamic_mesh(&world);
                            }
                            InventoryItem::Block(_) => {
                                if world.break_block_from_ray(
                                    camera.position,
                                    camera.look_direction(),
                                    6.0,
                                ) {
                                    let dirty_regions = world.chunk.take_dirty_regions();
                                    renderer.rebuild_chunk_regions(&world.chunk, &dirty_regions);
                                    renderer.build_dynamic_mesh(&world);
                                }
                            }
                        },
                        MouseButton::Right => {
                            if let InventoryItem::Block(block) = INVENTORY[inventory_index] {
                                if world.place_block_from_ray(
                                    camera.position,
                                    camera.look_direction(),
                                    6.0,
                                    block,
                                ) {
                                    let dirty_regions = world.chunk.take_dirty_regions();
                                    renderer.rebuild_chunk_regions(&world.chunk, &dirty_regions);
                                    renderer.build_dynamic_mesh(&world);
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                renderer.render();
                fps_counter += 1;
                if fps_last_sample.elapsed() >= Duration::from_secs(1) {
                    fps_display = fps_counter;
                    fps_counter = 0;
                    fps_last_sample = Instant::now();
                    renderer.update_hotbar(
                        selected_hotbar,
                        &inventory_colors(inventory_page),
                        player_health / 100.0,
                        fps_display,
                    );
                }
            }
            _ => {}
        },
        Event::AboutToWait => {
            target.set_control_flow(ControlFlow::WaitUntil(Instant::now() + frame_time));

            if mode == AppMode::Playing && !game_over {
                camera.update(&input, &world);
                let had_bullets = !world.bullets.is_empty();
                let had_particles = !world.particles.is_empty();
                let had_explosives = !world.explosives.is_empty();
                let (bullets_changed, bullet_damage) =
                    world.update_bullets(camera.position, camera.eye_height);
                let (explosives_changed, explosive_damage) =
                    world.update_explosives(camera.position, camera.eye_height);
                if bullets_changed || explosives_changed {
                    let dirty_regions = world.chunk.take_dirty_regions();
                    renderer.rebuild_chunk_regions(&world.chunk, &dirty_regions);
                }
                world.update_particles();
                let (mobs_changed, damage) = world.update_mobs(camera.position);
                let total_damage = damage + bullet_damage + explosive_damage;
                if total_damage > 0.0 {
                    player_health = (player_health - total_damage).max(0.0);
                    renderer.update_hotbar(
                        selected_hotbar,
                        &inventory_colors(inventory_page),
                        player_health / 100.0,
                        fps_display,
                    );
                    if player_health <= 0.0 {
                        game_over = true;
                        window.set_title("VoxelCraft - GAME OVER");
                    }
                }
                if had_bullets
                    || bullets_changed
                    || mobs_changed
                    || had_particles
                    || had_explosives
                    || explosives_changed
                    || !world.particles.is_empty()
                {
                    renderer.build_dynamic_mesh(&world);
                }
                renderer.update_camera(&camera);
            }
            window.request_redraw();
        }
        _ => {}
    });
}
