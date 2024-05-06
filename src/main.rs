mod input;
mod sprite;

use std::mem::size_of;
use std::sync::Arc;

use bytemuck::{Pod, Zeroable};
use sprite::Sprite;
use wgpu::include_wgsl;
use wgpu::{util::DeviceExt, Instance};
use winit::dpi::PhysicalSize;
use winit::event::*;
use winit::event_loop::EventLoop;
use winit::keyboard::{Key, KeyCode, NamedKey, PhysicalKey};
use winit::window::Window;

use image::io::Reader as ImageReader;

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Pod, Zeroable)]
struct Vertex {
    pos: [f32; 2],
    tex_coord: [f32; 2],
}

macro_rules! vert {
    ( $( [$x:expr, $y:expr] ),* ) => {
        [
        $(
            Vertex {
                pos: $x,
                tex_coord: $y,
            },
        )*
        ]
    };
}

const VERTICES: [Vertex; 6] = vert!(
    [[0.0, 1.0], [0.0, 1.0]], // top left
    [[1.0, 1.0], [1.0, 1.0]], // top right
    [[0.0, 0.0], [0.0, 0.0]], // bottom left
    [[0.0, 0.0], [0.0, 0.0]], // bottom left
    [[1.0, 0.0], [1.0, 0.0]], // bottom right
    [[1.0, 1.0], [1.0, 1.0]]  // top right
);

struct Context {
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
}
impl Context {
    async fn init() -> Result<Self, Box<dyn std::error::Error>> {
        let instance = Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let adapter_options = wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            ..Default::default()
        };
        let adapter = instance
            .request_adapter(&adapter_options)
            .await
            .ok_or("Could not aquire an adapter")?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    ..Default::default()
                },
                None,
            )
            .await?;

        Ok(Context {
            instance,
            adapter,
            device,
            queue,
        })
    }
}

struct Renderer {
    vertex_buf: wgpu::Buffer,
    pipeline: wgpu::RenderPipeline,
    proj_bind_group: wgpu::BindGroup,
    proj_buf: wgpu::Buffer,
}

impl Renderer {
    fn init(context: &Context, surface_config: &wgpu::SurfaceConfiguration) -> Self {
        let device = &context.device;

        let vertex_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let proj_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                // Projection Matrix
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let sprite_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    // Model Matrix
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // Texture
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    // Sampler
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&proj_group_layout, &sprite_group_layout],
            push_constant_ranges: &[],
        });

        let shader = device.create_shader_module(include_wgsl!("shader.wgsl"));

        let vertex_buffers = [wgpu::VertexBufferLayout {
            array_stride: 4 * size_of::<f32>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 2 * size_of::<f32>() as u64,
                    shader_location: 1,
                },
            ],
        }];

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &vertex_buffers,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_config.view_formats[0],
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let proj_mat = nalgebra_glm::ortho::<f32>(0.0, 229.0, 0.0, 190.0, -1.0, 1.0);

        let proj_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice((&proj_mat).into()),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
        });

        let proj_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &proj_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: proj_buf.as_entire_binding(),
            }],
        });

        Self {
            vertex_buf,
            pipeline,
            proj_bind_group,
            proj_buf,
        }
    }

    fn render(&mut self, target: &wgpu::TextureView, context: &Context, sprites: Vec<&Sprite>) {
        let device = &context.device;
        let queue = &context.queue;

        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.2,
                            g: 0.2,
                            b: 0.2,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            rpass.set_pipeline(&self.pipeline);
            rpass.set_vertex_buffer(0, self.vertex_buf.slice(..));
            rpass.set_bind_group(0, &self.proj_bind_group, &[]);
            for sprite in sprites {
                rpass.set_bind_group(1, sprite.get_bind_group(), &[]);
                rpass.draw(0..VERTICES.len() as u32, 0..1);
            }
        }

        queue.submit(Some(encoder.finish()));
    }
}

struct Surface {
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
}
impl Surface {
    fn new(context: &Context, window: Arc<Window>) -> Self {
        let window_size = window.inner_size();
        let width = window_size.width.max(1);
        let height = window_size.height.max(1);

        let surface = context.instance.create_surface(window).unwrap();

        let mut config = surface
            .get_default_config(&context.adapter, width, height)
            .unwrap();
        let format = config.format.remove_srgb_suffix();
        config.format = format;
        config.view_formats.push(format);

        surface.configure(&context.device, &config);

        Self { surface, config }
    }

    fn next_texture(&mut self) -> wgpu::SurfaceTexture {
        self.surface.get_current_texture().unwrap()
    }

    fn resize(&mut self, context: &Context, size: PhysicalSize<u32>) {
        let config = &mut self.config;
        config.width = size.width.max(1);
        config.height = size.height.max(1);

        let surface = &mut self.surface;
        surface.configure(&context.device, config);
    }

    fn config(&self) -> &wgpu::SurfaceConfiguration {
        &self.config
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    let window = Arc::new(winit::window::Window::new(&event_loop)?);
    let context = Context::init().await?;
    let mut surface = None;
    let mut renderer = None;

    let img = ImageReader::open("player.png")
        .unwrap()
        .decode()
        .unwrap()
        .flipv()
        .to_rgba8();
    let texels = bytemuck::cast_slice(img.as_raw());

    let texture = context.device.create_texture_with_data(
        &context.queue,
        &wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: img.width(),
                height: img.height(),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        },
        wgpu::util::TextureDataOrder::MipMajor,
        texels,
    );

    let sampler = context.device.create_sampler(&wgpu::SamplerDescriptor {
        ..Default::default()
    });

    let mut player = None;

    let _ = event_loop.run(move |event, target| match event {
        Event::NewEvents(StartCause::Init) => {
            surface = Some(Surface::new(&context, window.clone()));

            renderer = Some(Renderer::init(&context, surface.as_ref().unwrap().config()));

            player = Some(Sprite::new(
                nalgebra_glm::vec2(30.0, 30.0),
                nalgebra_glm::vec2(13.0, 8.0),
                &texture,
                &renderer.as_ref().unwrap().pipeline.get_bind_group_layout(1),
                &sampler,
                &context,
            ));
        }
        Event::WindowEvent { event, .. } => match event {
            WindowEvent::RedrawRequested => {
                let surface = surface.as_mut().unwrap();
                let frame = surface.next_texture();
                let view = frame.texture.create_view(&wgpu::TextureViewDescriptor {
                    format: Some(surface.config().view_formats[0]),
                    ..Default::default()
                });

                if input::is_key_pressed(KeyCode::ArrowRight) {
                    player
                        .as_mut()
                        .unwrap()
                        .move_by(&nalgebra_glm::vec2(0.07, 0.0), &context);
                }
                if input::is_key_pressed(KeyCode::ArrowLeft) {
                    player
                        .as_mut()
                        .unwrap()
                        .move_by(&nalgebra_glm::vec2(-0.07, 0.0), &context);
                }

                let sprites = vec![player.as_ref().unwrap()];

                renderer.as_mut().unwrap().render(&view, &context, sprites);

                frame.present();

                window.request_redraw();
            }

            WindowEvent::Resized(size) => {
                let surface = surface.as_mut().unwrap();
                surface.resize(&context, size);
            }

            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key: Key::Named(NamedKey::Escape),
                        ..
                    },
                ..
            }
            | WindowEvent::CloseRequested => {
                target.exit();
            }

            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(key),
                        state,
                        repeat: false,
                        ..
                    },
                ..
            } => {
                input::register_key_state(key, state);
            }

            _ => {}
        },
        _ => {}
    });

    Ok(())
}
