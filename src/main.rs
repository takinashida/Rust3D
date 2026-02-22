mod engine;
mod world;

use std::sync::Arc;

use engine::{camera::Camera, input::InputState, renderer::Renderer};
use world::world::World;

use winit::{
    event::{ElementState, Event, MouseButton, WindowEvent},
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::WindowBuilder,
};

fn main() {
    pollster::block_on(run());
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
    let mut input = InputState::new();

    let mut renderer = Renderer::new(window.clone()).await;
    renderer.build_world_mesh(&world);

    let _ = event_loop.run(move |event, target| match event {
        Event::WindowEvent { event, window_id } if window_id == window.id() => match event {
            WindowEvent::CloseRequested => target.exit(),
            WindowEvent::Resized(size) => {
                renderer.resize(size);
                camera.aspect = (size.width.max(1) as f32) / (size.height.max(1) as f32);
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(KeyCode::Escape) = event.physical_key {
                    if event.state == ElementState::Pressed {
                        target.exit();
                    }
                }
                input.update(&event);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if button == MouseButton::Left && state == ElementState::Pressed {
                    world.break_block(8, 8, 8);
                    renderer.build_world_mesh(&world);
                }
            }
            WindowEvent::RedrawRequested => renderer.render(),
            _ => {}
        },
        Event::AboutToWait => {
            camera.update(&input);
            renderer.update_camera(&camera);
            window.request_redraw();
        }
        _ => {}
    });
}
