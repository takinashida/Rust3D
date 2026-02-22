use std::sync::Arc;

use wgpu::util::DeviceExt;
use winit::{dpi::PhysicalSize, window::Window};

use crate::{
    engine::mesh::{Mesh, Vertex},
    world::world::World,
};

use super::camera::Camera;

pub struct Renderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    sky_pipeline: wgpu::RenderPipeline,
    crosshair_pipeline: wgpu::RenderPipeline,
    hotbar_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    vertex_count: u32,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    hotbar_vertex_buffer: wgpu::Buffer,
    hotbar_vertex_count: u32,
    depth_texture: DepthTexture,
}

impl Renderer {
    pub async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(window).expect("surface");

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("gpu adapter");

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .expect("device");

        let caps = surface.get_capabilities(&adapter);
        let format = caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let camera_uniform = CameraUniform::new();
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("camera buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("camera layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("camera bind group"),
            layout: &camera_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("voxel shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/voxel.wgsl").into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pipeline layout"),
            bind_group_layouts: &[&camera_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("render pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: Some(wgpu::Face::Back),
                front_face: wgpu::FrontFace::Ccw,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: DepthTexture::FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let sky_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("sky shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/sky.wgsl").into()),
        });

        let sky_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("sky pipeline layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let sky_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("sky pipeline"),
            layout: Some(&sky_layout),
            vertex: wgpu::VertexState {
                module: &sky_shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &sky_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: Some(wgpu::DepthStencilState {
                format: DepthTexture::FORMAT,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Always,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let crosshair_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("crosshair shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/crosshair.wgsl").into()),
        });

        let crosshair_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("crosshair pipeline layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let crosshair_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("crosshair pipeline"),
            layout: Some(&crosshair_layout),
            vertex: wgpu::VertexState {
                module: &crosshair_shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &crosshair_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineList,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: DepthTexture::FORMAT,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Always,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let hotbar_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("hotbar shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/hotbar.wgsl").into()),
        });

        let hotbar_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("hotbar pipeline layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let hotbar_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("hotbar pipeline"),
            layout: Some(&hotbar_layout),
            vertex: wgpu::VertexState {
                module: &hotbar_shader,
                entry_point: "vs_main",
                buffers: &[UiVertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &hotbar_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: DepthTexture::FORMAT,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Always,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let hotbar_vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("empty hotbar vertices"),
            size: 4,
            usage: wgpu::BufferUsages::VERTEX,
            mapped_at_creation: false,
        });

        let depth_texture = DepthTexture::new(&device, &config, "depth texture");

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("empty vertices"),
            size: 4,
            usage: wgpu::BufferUsages::VERTEX,
            mapped_at_creation: false,
        });

        Self {
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            sky_pipeline,
            crosshair_pipeline,
            hotbar_pipeline,
            vertex_buffer,
            vertex_count: 0,
            camera_buffer,
            camera_bind_group,
            hotbar_vertex_buffer,
            hotbar_vertex_count: 0,
            depth_texture,
        }
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width == 0 || new_size.height == 0 {
            return;
        }

        self.size = new_size;
        self.config.width = new_size.width;
        self.config.height = new_size.height;
        self.surface.configure(&self.device, &self.config);
        self.depth_texture = DepthTexture::new(&self.device, &self.config, "depth texture");
    }

    pub fn build_world_mesh(&mut self, world: &World) {
        let mesh = Mesh::from_world(world);
        self.vertex_count = mesh.vertices.len() as u32;

        self.vertex_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("voxel mesh"),
                contents: bytemuck::cast_slice(&mesh.vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
    }

    pub fn update_camera(&mut self, camera: &Camera) {
        let mut uniform = CameraUniform::new();
        uniform.view_proj = camera.view_proj_matrix().into();
        self.queue
            .write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[uniform]));
    }

    pub fn update_hotbar(&mut self, selected: usize, palette: &[[f32; 3]], health: f32, fps: u32) {
        let width = 0.11;
        let height = 0.1;
        let gap = 0.012;
        let total = palette.len() as f32 * width + (palette.len().saturating_sub(1)) as f32 * gap;
        let start_x = -total * 0.5;
        let y = -0.92;

        let mut vertices = Vec::new();

        for (i, color) in palette.iter().enumerate() {
            let x0 = start_x + i as f32 * (width + gap);
            let x1 = x0 + width;
            let y0 = y;
            let y1 = y + height;

            append_rect(&mut vertices, x0, y0, x1, y1, *color);

            let border_color = if i == selected {
                [1.0, 1.0, 1.0]
            } else {
                [0.1, 0.1, 0.1]
            };
            let b = 0.008;
            append_rect(&mut vertices, x0 - b, y0 - b, x1 + b, y0, border_color);
            append_rect(&mut vertices, x0 - b, y1, x1 + b, y1 + b, border_color);
            append_rect(&mut vertices, x0 - b, y0, x0, y1, border_color);
            append_rect(&mut vertices, x1, y0, x1 + b, y1, border_color);
        }

        let hp = health.clamp(0.0, 1.0);
        let bar_w = 0.45;
        let bar_h = 0.04;
        let bx0 = -bar_w * 0.5;
        let by0 = 0.88;
        let bx1 = bx0 + bar_w;
        let by1 = by0 + bar_h;
        append_rect(
            &mut vertices,
            bx0 - 0.01,
            by0 - 0.01,
            bx1 + 0.01,
            by1 + 0.01,
            [0.06, 0.06, 0.06],
        );
        let fill_x1 = bx0 + bar_w * hp;
        append_rect(
            &mut vertices,
            bx0,
            by0,
            fill_x1,
            by1,
            [0.82, 0.16 + 0.55 * hp, 0.16],
        );
        draw_fps_counter(&mut vertices, fps);

        self.hotbar_vertex_count = vertices.len() as u32;
        self.hotbar_vertex_buffer =
            self.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("hotbar mesh"),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                });
    }

    pub fn render(&mut self) {
        let frame = match self.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                self.surface.configure(&self.device, &self.config);
                return;
            }
            Err(wgpu::SurfaceError::OutOfMemory) => {
                panic!("Out of memory");
            }
            Err(wgpu::SurfaceError::Timeout) => return,
        };

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("encoder"),
            });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            pass.set_pipeline(&self.sky_pipeline);
            pass.draw(0..3, 0..1);

            pass.set_pipeline(&self.render_pipeline);
            pass.set_bind_group(0, &self.camera_bind_group, &[]);
            pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            pass.draw(0..self.vertex_count, 0..1);

            pass.set_pipeline(&self.crosshair_pipeline);
            pass.draw(0..4, 0..1);

            pass.set_pipeline(&self.hotbar_pipeline);
            pass.set_vertex_buffer(0, self.hotbar_vertex_buffer.slice(..));
            pass.draw(0..self.hotbar_vertex_count, 0..1);
        }

        self.queue.submit(Some(encoder.finish()));
        frame.present();
    }
}

struct DepthTexture {
    view: wgpu::TextureView,
}

impl DepthTexture {
    const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth24Plus;

    fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration, label: &str) -> Self {
        let size = wgpu::Extent3d {
            width: config.width.max(1),
            height: config.height.max(1),
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        Self { view }
    }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct UiVertex {
    position: [f32; 2],
    color: [f32; 3],
}

impl UiVertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<UiVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

fn append_rect(vertices: &mut Vec<UiVertex>, x0: f32, y0: f32, x1: f32, y1: f32, color: [f32; 3]) {
    vertices.extend_from_slice(&[
        UiVertex {
            position: [x0, y0],
            color,
        },
        UiVertex {
            position: [x1, y0],
            color,
        },
        UiVertex {
            position: [x1, y1],
            color,
        },
        UiVertex {
            position: [x1, y1],
            color,
        },
        UiVertex {
            position: [x0, y1],
            color,
        },
        UiVertex {
            position: [x0, y0],
            color,
        },
    ]);
}

fn draw_fps_counter(vertices: &mut Vec<UiVertex>, fps: u32) {
    let clamped = fps.min(999);
    let hundreds = (clamped / 100) % 10;
    let tens = (clamped / 10) % 10;
    let ones = clamped % 10;

    let mut x = -0.97;
    let y = -0.97;
    let w = 0.034;
    let h = 0.06;

    if hundreds > 0 {
        append_digit(vertices, hundreds as usize, x, y, w, h, [0.95, 0.95, 0.25]);
        x += w + 0.01;
    }
    if hundreds > 0 || tens > 0 {
        append_digit(vertices, tens as usize, x, y, w, h, [0.95, 0.95, 0.25]);
        x += w + 0.01;
    }
    append_digit(vertices, ones as usize, x, y, w, h, [0.95, 0.95, 0.25]);
}

fn append_digit(
    vertices: &mut Vec<UiVertex>,
    digit: usize,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    color: [f32; 3],
) {
    let t = 0.008;
    let half_h = h * 0.5;
    let seg = [
        [x + t, y, x + w - t, y + t],
        [x + w - t, y + t, x + w, y + half_h - t * 0.5],
        [x + w - t, y + half_h + t * 0.5, x + w, y + h - t],
        [x + t, y + h - t, x + w - t, y + h],
        [x, y + half_h + t * 0.5, x + t, y + h - t],
        [x, y + t, x + t, y + half_h - t * 0.5],
        [x + t, y + half_h - t * 0.5, x + w - t, y + half_h + t * 0.5],
    ];
    const MASKS: [u8; 10] = [
        0b0111111, 0b0000110, 0b1011011, 0b1001111, 0b1100110, 0b1101101, 0b1111101, 0b0000111,
        0b1111111, 0b1101111,
    ];
    let mask = MASKS[digit.min(9)];
    for (i, r) in seg.iter().enumerate() {
        if mask & (1 << i) != 0 {
            append_rect(vertices, r[0], r[1], r[2], r[3], color);
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    fn new() -> Self {
        Self {
            view_proj: cgmath::Matrix4::from_scale(1.0).into(),
        }
    }
}
