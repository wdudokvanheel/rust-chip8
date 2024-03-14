use std::sync::Arc;

use wgpu::{Adapter, BindGroup, Buffer, Device, Instance, Queue, RenderPipeline, ShaderModule, Surface, SurfaceConfiguration, TextureFormat};
use wgpu::util::DeviceExt;
use winit::dpi::PhysicalSize;
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};
use crate::wgpu_runtime::Vertex;

use crate::wgpu_runtime::wgpu_math::Vec2i;

pub struct WgpuContext {
    pub window: Window,
    pub device: Arc<Device>,
    pub surface: Surface,
    pub surface_config: SurfaceConfiguration,
    pub adapter: Adapter,
    pub queue: Queue,
    pub texture_format: TextureFormat,
    pub elapsed_frame_time: f32,
    pub index_buffer: Buffer,
    pub vertex_buffer: Buffer,
}

impl WgpuContext {
    pub async fn new(event_loop: &EventLoop<()>, title: &str, window_size: Vec2i) -> Self {
        log::debug!("Creating new Wgpu Context");
        let size = PhysicalSize::new(window_size.x as u32, window_size.y as u32);

        let mut builder = WindowBuilder::new().with_title(title);

        #[cfg(target_arch = "wasm32")]{
            builder = WgpuContext::init_canvas(builder);
        }
        #[cfg(not(target_arch = "wasm32"))]{
            builder = builder.with_inner_size(size);
        }

        let window: Window = builder.build(event_loop).unwrap();
        let instance = Instance::default();
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

        let device = Arc::new(device);

        let surface_info = surface.get_capabilities(&adapter);
        let texture_format = surface_info.formats[0];

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

        log::warn!("USING: {:?}", texture_format);

        let surface_config = SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: texture_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_info.alpha_modes[0],
            view_formats: vec![],
        };

        surface.configure(device.as_ref(), &surface_config);

        Self {
            window,
            device,
            surface,
            surface_config,
            adapter,
            queue,
            texture_format,
            elapsed_frame_time: 0.0,
            index_buffer,
            vertex_buffer,
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn init_canvas(builder: WindowBuilder) -> WindowBuilder {
        use winit::platform::web::WindowBuilderExtWebSys;
        use wasm_bindgen::prelude::*;

        let canvas = web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| doc.get_element_by_id("canvas"))
            .and_then(|el| el.dyn_into::<web_sys::HtmlCanvasElement>().ok())
            .expect("Failed to find canvas");

        return builder.with_canvas(Some(canvas));
    }
}
