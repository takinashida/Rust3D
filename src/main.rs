mod engine;
mod world;

use engine::renderer::Renderer;
use engine::camera::Camera;
use engine::input::InputState;

use world::world::World;

use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() {
    pollster::block_on(run());
}

async fn run() {
    // ------------------------
    // WINDOW + EVENT LOOP
    // ------------------------
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Voxel Game")
        .build(&event_loop)
        .unwrap();

    // ------------------------
    // GAME STATE
    // ------------------------
    let mut world = World::new();
    let mut camera = Camera::new();
    let mut input = InputState::new();

    let mut renderer = Renderer::new(&window).await;
    renderer.build_world_mesh(&world);

    // ------------------------
    // MAIN LOOP
    // ------------------------
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent { event, .. } => {
                match event {
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit
                    }

                    WindowEvent::KeyboardInput { input: key, .. } => {
                        input.update(key);
                    }

                    WindowEvent::MouseInput { state, button, .. } => {
                        if button == MouseButton::Left && state == ElementState::Pressed {
                            // Упрощённо ломаем блок перед игроком
                            world.break_block(8, 8, 8);

                            renderer.build_world_mesh(&world);
                        }
                    }

                    _ => {}
                }
            }

            Event::MainEventsCleared => {
                camera.update(&input);
                renderer.update_camera(&camera);

                window.request_redraw();
            }

            Event::RedrawRequested(_) => {
                renderer.render();
            }

            _ => {}
        }
    });
}