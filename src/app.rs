use std::borrow::Cow;

use bytemuck;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use wgpu::{Adapter, BindGroup, Buffer, Device, Instance, Queue, RenderPipeline, ShaderModule, Surface, SurfaceCapabilities, TextureFormat};
use wgpu::util::DeviceExt;
use winit::dpi::{PhysicalSize, Size};
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};

use crate::chip8::{Chip8, mainChips8};
use crate::utils::init_logger;
use crate::vertex::Vertex;

pub fn start() {
    // mainChips8();
    init_logger();
    init_wpgu();
}

fn init_wpgu() {
    // Create event loop & window
    let event_loop = EventLoop::new().unwrap();
    let mut builder = WindowBuilder::new()
        .with_title("Desktop & WASM test")
        .with_inner_size(PhysicalSize::new(1280, 640));

    #[cfg(target_arch = "wasm32")]{
        builder = init_canvas(builder);
    }

    let window: Window = builder.build(&event_loop).unwrap();

    #[cfg(target_arch = "wasm32")]{
        wasm_bindgen_futures::spawn_local(run(event_loop, window));
    }

    #[cfg(not(target_arch = "wasm32"))]{
        pollster::block_on(run(event_loop, window));
    }
}


#[cfg(target_arch = "wasm32")]
fn init_canvas(builder: WindowBuilder) -> WindowBuilder {
    use winit::platform::web::WindowExtWebSys;
    use winit::platform::web::WindowBuilderExtWebSys;

    let canvas = web_sys::window()
        .and_then(|win| win.document())
        .and_then(|doc| doc.get_element_by_id("canvas"))
        .and_then(|el| el.dyn_into::<web_sys::HtmlCanvasElement>().ok())
        .expect("couldn't append canvas to document body");

    return builder.with_canvas(Some(canvas));
}

async fn run(event_loop: EventLoop<()>, window: Window) {
    let size = PhysicalSize::new(1280, 640);

    let (instance, surface, adapter, device, queue) = create_wpgu(&window).await;

    let shader = create_shader(&device);
    let (swapchain_capabilities, swapchain_format) = create_chain_config(&surface, &adapter);

    let vertices: &[Vertex] = &[
        Vertex { position: [-1.0, 1.0, 0.0] },
        Vertex { position: [1.0, 1.0, 0.0] },
        Vertex { position: [-1.0, -1.0, 0.0] },
        Vertex { position: [1.0, -1.0, 0.0] },
    ];

    const INDICES: &[u16] = &[
        0, 1, 2,
        2, 1, 3,
    ];

    let vertex_buffer = device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        }
    );
    let index_buffer = device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        }
    );

    let (render_pipeline, uniform_buffer, display_bind_group) = create_pipeline(&device, &shader,
                                                                                swapchain_format);

    let mut config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: swapchain_format,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Fifo,
        alpha_mode: swapchain_capabilities.alpha_modes[0],
        view_formats: vec![],
    };

    surface.configure(&device, &config);


    let _ = (&instance, &adapter, &shader);
    let mut chip8 = Chip8::new();
    let rom = crate::chip8::load_rom();
    chip8.set_rom(rom);

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                log::info!("Window resized to: {}x{}", size.width, size.height);
                config.width = size.width;
                config.height = size.height;
                surface.configure(&device, &config);

                window.request_redraw();
            }
            Event::AboutToWait => {
                window.request_redraw();
            }
            Event::RedrawRequested(_) => {
                chip8.cycle();

                let frame = surface
                    .get_current_texture()
                    .expect("Failed to acquire next swap chain texture");
                let view = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                let mut encoder =
                    device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
                {
                    let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: None,
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                                store: true,
                            },
                        })],
                        depth_stencil_attachment: None,
                    });
                    queue.write_buffer(&uniform_buffer, 0, bytemuck::cast_slice
                        (&[ShaderUniform::from_display(chip8.display)]));
                    rpass.set_bind_group(0, &display_bind_group, &[]);
                    rpass.set_pipeline(&render_pipeline);
                    rpass.set_vertex_buffer(0, vertex_buffer.slice(..));
                    rpass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);

                    rpass.draw_indexed(0..6, 0, 0..1);
                }

                queue.submit(Some(encoder.finish()));
                frame.present();
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => {}
        }
    }).expect("Failed to run event");
}

fn create_pipeline(device: &Device, shader: &ShaderModule, format: TextureFormat) ->
(RenderPipeline, Buffer, BindGroup) {
    let uniform = ShaderUniform::new();

    let uniform_buffer = device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some("Display Buffer"),
            contents: bytemuck::cast_slice(&[uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        }
    );

    let display_bind_group_layout = device.create_bind_group_layout
    (&wgpu::BindGroupLayoutDescriptor {
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }
        ],
        label: Some("display_bind_group_layout"),
    });

    let display_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &display_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }
        ],
        label: Some("display_bind_group"),
    });


    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[&display_bind_group_layout],
        push_constant_ranges: &[],
    });

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[
                Vertex::get_layout(),
            ],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(format.into())],
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
    });

    (render_pipeline, uniform_buffer, display_bind_group)
}

fn create_chain_config(surface: &Surface, adapter: &Adapter) -> (SurfaceCapabilities, TextureFormat) {
    let swapchain_capabilities = surface.get_capabilities(&adapter);
    let swapchain_format = swapchain_capabilities.formats[0];
    (swapchain_capabilities, swapchain_format)
}

fn create_shader(device: &Device) -> ShaderModule {
    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
    })
}

async fn create_wpgu(window: &Window) -> (Instance, Surface, Adapter, Device, Queue) {
    let instance = wgpu::Instance::default();

    let surface = unsafe { instance.create_surface(&window) }.unwrap();
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        })
        .await
        .expect("Failed to find an appropriate adapter");

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::downlevel_webgl2_defaults()
                    .using_resolution(adapter.limits()),
            },
            None,
        )
        .await
        .expect("Failed to create device");
    return (instance, surface, adapter, device, queue);
}

#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct ShaderUniform {
    value: [u32; 2048],
}

impl ShaderUniform {
    fn new() -> Self {
        let mut n = ShaderUniform {
            value: [0; 2048]
        };

        // n.value[0] = 1;
        // n.value[1] = 1;
        // n.value[2047] = 1;

        return n;
    }

    fn from_display(display: [[bool; 64]; 32]) -> Self {
        let mut n = ShaderUniform {
            value: [0; 2048],
        };

        for (row_index, row) in display.iter().enumerate() {
            for (col_index, &col_value) in row.iter().enumerate() {
                if (col_value) {
                    n.value[row_index * 64 + col_index] = 1;
                }
            }
        }

        return n;
    }
}
