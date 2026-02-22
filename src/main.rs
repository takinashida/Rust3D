mod engine;
mod world;

use std::sync::Arc;

use engine::{camera::Camera, input::InputState, renderer::Renderer};
use world::world::World;

use winit::{
    event::{DeviceEvent, ElementState, Event, MouseButton, WindowEvent},
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::{CursorGrabMode, WindowBuilder},
};

fn main() {
    pollster::block_on(run());
}

fn set_mouse_lock(window: &winit::window::Window, locked: bool) -> bool {
    if !locked {
        let _ = window.set_cursor_grab(CursorGrabMode::None);
        window.set_cursor_visible(true);
        return true;
    }

    let locked_ok = window.set_cursor_grab(CursorGrabMode::Locked).is_ok()
        || window.set_cursor_grab(CursorGrabMode::Confined).is_ok();

    window.set_cursor_visible(!locked_ok);
    locked_ok
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
    let mut mouse_look_enabled = set_mouse_lock(&window, true);

    let mut renderer = Renderer::new(window.clone()).await;
    renderer.build_world_mesh(&world);

    let _ = event_loop.run(move |event, target| match event {
        Event::WindowEvent { event, window_id } if window_id == window.id() => match event {
            WindowEvent::CloseRequested => target.exit(),
            WindowEvent::Focused(focused) => {
                mouse_look_enabled = set_mouse_lock(&window, focused);
                if !focused {
                    input.clear();
                }
            }
            WindowEvent::Resized(size) => {
                renderer.resize(size);
                camera.aspect = (size.width.max(1) as f32) / (size.height.max(1) as f32);
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(KeyCode::Escape) = event.physical_key {
                    if event.state == ElementState::Pressed {
                        mouse_look_enabled = false;
                        let _ = set_mouse_lock(&window, false);
                    }
                }
                if let PhysicalKey::Code(KeyCode::Tab) = event.physical_key {
                    if event.state == ElementState::Pressed && !event.repeat {
                        mouse_look_enabled = set_mouse_lock(&window, !mouse_look_enabled);
                    }
                }
                input.update(&event);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if button == MouseButton::Right && state == ElementState::Pressed {
                    mouse_look_enabled = set_mouse_lock(&window, !mouse_look_enabled);
                }
                if button == MouseButton::Left && state == ElementState::Pressed {
                    world.break_block(8, 8, 8);
                    renderer.build_world_mesh(&world);
                }
            }
            WindowEvent::RedrawRequested => renderer.render(),
            _ => {}
        },
        Event::DeviceEvent {
            event: DeviceEvent::MouseMotion { delta },
            ..
        } => {
            if mouse_look_enabled {
                camera.process_mouse(delta.0, delta.1);
            }
        }
        Event::AboutToWait => {
            camera.update(&input);
            renderer.update_camera(&camera);
            window.request_redraw();
        }
        _ => {}
    });
}
