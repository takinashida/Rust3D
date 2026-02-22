mod engine;
mod world;

use std::sync::Arc;

use engine::{camera::Camera, input::InputState, renderer::Renderer};
use world::{chunk::Block, world::World};

use winit::{
    event::{DeviceEvent, ElementState, Event, MouseButton, WindowEvent},
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::{CursorGrabMode, WindowBuilder},
};

const HOTBAR_BLOCKS: [Block; 10] = [
    Block::Grass,
    Block::Dirt,
    Block::Stone,
    Block::Sand,
    Block::Wood,
    Block::Planks,
    Block::Bricks,
    Block::Glass,
    Block::Leaf,
    Block::Snow,
];

const HOTBAR_COLORS: [[f32; 3]; 10] = [
    [0.25, 0.78, 0.22],
    [0.55, 0.35, 0.2],
    [0.55, 0.58, 0.62],
    [0.88, 0.82, 0.55],
    [0.52, 0.34, 0.19],
    [0.72, 0.56, 0.34],
    [0.68, 0.27, 0.24],
    [0.6, 0.85, 0.95],
    [0.22, 0.56, 0.18],
    [0.92, 0.94, 0.98],
];

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

fn hotbar_slot_for_key(code: KeyCode) -> Option<usize> {
    match code {
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
    }
}

async fn run() {
    let event_loop = EventLoop::new().expect("event loop");
    let window = Arc::new(
        WindowBuilder::new()
            .with_title("Voxel Game")
            .build(&event_loop)
            .expect("window"),
    );

    let mut world = World::new();
    let mut camera = Camera::new();
    let size = window.inner_size();
    camera.aspect = (size.width.max(1) as f32) / (size.height.max(1) as f32);
    let mut input = InputState::new();
    let mut selected_hotbar = 0usize;

    set_mouse_lock(&window, true);

    let mut renderer = Renderer::new(window.clone()).await;
    renderer.build_world_mesh(&world);
    renderer.update_hotbar(selected_hotbar, &HOTBAR_COLORS);

    let _ = event_loop.run(move |event, target| match event {
        Event::DeviceEvent {
            event: DeviceEvent::MouseMotion { delta },
            ..
        } => {
            camera.process_mouse(delta.0, delta.1);
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
                        target.exit();
                    }

                    if event.state == ElementState::Pressed {
                        if let Some(slot) = hotbar_slot_for_key(code) {
                            selected_hotbar = slot;
                            renderer.update_hotbar(selected_hotbar, &HOTBAR_COLORS);
                        }
                    }
                }
                input.update(&event);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if state == ElementState::Pressed {
                    match button {
                        MouseButton::Left => {
                            if world.break_block_from_ray(
                                camera.position,
                                camera.look_direction(),
                                6.0,
                            ) {
                                renderer.build_world_mesh(&world);
                            }
                        }
                        MouseButton::Right => {
                            if world.place_block_from_ray(
                                camera.position,
                                camera.look_direction(),
                                6.0,
                                HOTBAR_BLOCKS[selected_hotbar],
                            ) {
                                renderer.build_world_mesh(&world);
                            }
                        }
                        _ => {}
                    }
                }
            }
            WindowEvent::RedrawRequested => renderer.render(),
            _ => {}
        },
        Event::AboutToWait => {
            camera.update(&input, &world);
            renderer.update_camera(&camera);
            window.request_redraw();
        }
        _ => {}
    });
}
