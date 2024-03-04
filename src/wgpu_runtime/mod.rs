use bytemuck::Zeroable;
use instant::Instant;
use wgpu::Texture;
use winit::dpi::PhysicalSize;
use winit::event::{Event, MouseButton, WindowEvent};
use winit::event::ElementState::Pressed;
use winit::event_loop::{ControlFlow, EventLoop, EventLoopBuilder};
use winit::keyboard::KeyCode;

use crate::wgpu_runtime::wgpu_context::WgpuContext;
use crate::wgpu_runtime::wgpu_math::{Vec2f, Vec2i};

pub mod wgpu_context;
pub mod wgpu_math;

pub struct RuntimeContext {
    pub gfx: WgpuContext,
    pub mouse_position: Vec2f,
}

pub struct WgpuRuntime<AppData> {
    data: Option<AppData>,
    context: RuntimeContext,
    event_loop: EventLoop<()>,
    callback: RuntimeCallbackFunctions<AppData>,
    logic_update_frame: f32,
}

pub struct RuntimeCallbackFunctions<AppData> {
    pub init: fn(&mut RuntimeContext) -> AppData,
    pub update: fn(&mut RuntimeContext, &mut AppData, f32),
    pub render: fn(&mut RuntimeContext, &mut AppData, &Texture),
    pub resize: fn(&mut RuntimeContext, &mut AppData, Vec2i),
    pub key_input: fn(&mut RuntimeContext, &mut AppData, KeyCode, bool),
    pub mouse_move: fn(&mut RuntimeContext, &mut AppData, Vec2f),
    pub mouse_click: fn(&mut RuntimeContext, &mut AppData, Vec2i, MouseButton, bool),
}

impl<AppData: 'static> WgpuRuntime<AppData> {
    pub fn new(
        title: &str,
        window_size: Vec2i,
        init_callback: fn(&mut RuntimeContext) -> AppData,
    ) -> Self {
        WgpuRuntime::<AppData>::init_logger();
        let event_loop = EventLoopBuilder::new().build().expect("Failed to create event loop");
        let gfx = pollster::block_on(WgpuContext::new(&event_loop, title, window_size));
        WgpuRuntime {
            context: RuntimeContext {
                gfx,
                mouse_position: Vec2f::zero(),
            },
            data: None,
            event_loop,
            callback: RuntimeCallbackFunctions::new(init_callback),
            logic_update_frame: 1000.0 / 60.0,
        }
    }

    pub fn start(mut self) {
        self.data = Some((self.callback.init)(&mut self.context));
        self.run();
    }

    fn run(mut self) {
        let context = &mut self.context;
        let callback = self.callback;
        let data = self.data.as_mut().unwrap();

        let mut last_update_time = Instant::now();

        self.event_loop.run(move |event, _, control_flow| {
            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                    }
                    WindowEvent::Resized(size) => {
                        log::trace!("Window resized to: {}x{}", size.width, size.height);
                        context.gfx.surface_config.width = size.width;
                        context.gfx.surface_config.height = size.height;
                        context.gfx.surface.configure(&context.gfx.device, &context.gfx.surface_config);
                        let size = Vec2i::new(size.width as i32, size.height as i32);
                        (callback.resize)(
                            context,
                            data,
                            size,
                        );
                        context.gfx.window.request_redraw();
                    }
                    WindowEvent::KeyboardInput {
                        event,
                        ..
                    } => {
                        (callback.key_input)(
                            context,
                            data,
                            event.physical_key,
                            event.state == Pressed,
                        );
                    }
                    WindowEvent::CursorMoved { position, .. } => {
                        context.mouse_position = Vec2f::new(position.x as f32, position.y as f32);
                        let position = context.mouse_position.clone();
                        (callback.mouse_move)(context, data, position);
                    }
                    WindowEvent::MouseInput {
                        button,
                        state,
                        ..
                    }
                    => {
                        (callback.mouse_click)(context, data, context.mouse_position.round_2i(), button, state == Pressed)
                    }
                    _ => {}
                }
                Event::AboutToWait => {
                    context.gfx.window.request_redraw();
                }
                Event::RedrawRequested(_) => {
                    let now = Instant::now();
                    context.gfx.elapsed_frame_time += now.duration_since
                    (last_update_time).as_millis() as f32;
                    last_update_time = now;

                    while context.gfx.elapsed_frame_time > self.logic_update_frame {
                        context.gfx.elapsed_frame_time -= self.logic_update_frame;
                        (callback.update)(context, data, self.logic_update_frame);
                    }

                    let frame = context.gfx.surface
                        .get_current_texture()
                        .expect("Failed to acquire next swap chain texture");


                    (callback.render)(context, data, &frame.texture);
                    frame.present();
                }
                _ => {}
            }
        }).expect("Failed to run event");
    }

    pub fn on_key_event(&mut self, callback: fn(&mut RuntimeContext, &mut AppData, KeyCode, bool)) {
        self.callback.key_input = callback;
    }

    pub fn on_render(&mut self, callback: fn(&mut RuntimeContext, &mut AppData, &Texture)) {
        self.callback.render = callback;
    }

    pub fn on_resize(&mut self, callback: fn(&mut RuntimeContext, &mut AppData, Vec2i)) {
        self.callback.resize = callback;
    }

    pub fn on_update(&mut self, callback: fn(&mut RuntimeContext, &mut AppData, f32)) {
        self.callback.update = callback;
    }

    pub fn on_mouse_move(&mut self, callback: fn(&mut RuntimeContext, &mut AppData, Vec2f)) {
        self.callback.mouse_move = callback;
    }

    pub fn on_mouse_click(
        &mut self,
        callback: fn(
            &mut RuntimeContext,
            &mut AppData,
            Vec2i,
            MouseButton,
            bool)) {
        self.callback.mouse_click = callback;
    }

    fn init_logger() {
        #[cfg(not(target_arch = "wasm32"))]
        {
            simple_logger::SimpleLogger::new().with_colors(true)
                .with_module_level("wgpu", log::LevelFilter::Warn)
                .with_module_level("naga", log::LevelFilter::Info)
                .with_level(log::LevelFilter::Debug).init().unwrap();
        }
        #[cfg(target_arch = "wasm32")]
        {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init().expect("could not initialize logger");
        }
        log::info!("Enabled logging");
    }
}

impl<AppData> RuntimeCallbackFunctions<AppData> {
    pub fn new(init: fn(&mut RuntimeContext) -> AppData) -> Self {
        Self {
            init,
            update: |_, _, _| {},
            render: |_, _, _| {},
            resize: |_, _, _| {},
            key_input: |_, _, _, _| {},
            mouse_move: |_, _, _| {},
            mouse_click: |_, _, _, _, _| {},
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Zeroable, bytemuck::Pod)]
pub struct Vertex {
    pub(crate) position: [f32; 3],
}

impl Vertex {
    pub fn get_layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                }
            ],
        }
    }
}
