use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

use bytemuck::Zeroable;
use instant::Instant;
use wgpu::Texture;
use winit::dpi::PhysicalSize;
use winit::event::{Event, MouseButton, WindowEvent};
use winit::event::ElementState::Pressed;
use winit::event_loop::{ControlFlow, EventLoop, EventLoopBuilder};
use winit::keyboard::KeyCode;

use crate::application::AppCommand;
use crate::wgpu_runtime::wgpu_context::WgpuContext;
use crate::wgpu_runtime::wgpu_math::{Vec2f, Vec2i};

pub mod wgpu_context;
pub mod wgpu_math;

pub struct RuntimeContext {
    pub gfx: WgpuContext,
}

pub struct WgpuRuntime<AppData, RuntimeCommand> {
    pub data: Option<AppData>,
    context: RuntimeContext,
    event_loop: EventLoop<()>,
    callback: RuntimeCallbackFunctions<AppData, RuntimeCommand>,
    logic_update_frame: f32,
    command_sender: Sender<RuntimeCommand>,
    command_receiver: Receiver<RuntimeCommand>,
}

pub struct RuntimeCallbackFunctions<AppData, RuntimeCommand> {
    pub init: fn(&mut RuntimeContext) -> AppData,
    pub update: fn(&mut RuntimeContext, &mut AppData, f32),
    pub render: fn(&mut RuntimeContext, &mut AppData, &Texture),
    pub resize: fn(&mut RuntimeContext, &mut AppData, Vec2i),
    pub key_input: fn(&mut RuntimeContext, &mut AppData, KeyCode, bool),
    pub runtime_command: fn(&mut RuntimeContext, &mut AppData, RuntimeCommand),
}

impl<AppData: 'static, RuntimeCommand: 'static> WgpuRuntime<AppData, RuntimeCommand> {
    pub fn new(
        title: &str,
        window_size: Vec2i,
        init_callback: fn(&mut RuntimeContext) -> AppData,
    ) -> Self {
        WgpuRuntime::<AppData, RuntimeCommand>::init_logger();
        let event_loop = EventLoopBuilder::new().build().expect("Failed to create event loop");
        let gfx = pollster::block_on(WgpuContext::new(&event_loop, title, window_size));

        let (sender, receiver) = mpsc::channel();

        let mut runtime = WgpuRuntime {
            context: RuntimeContext {
                gfx,
            },
            data: None,
            event_loop,
            callback: RuntimeCallbackFunctions::new(init_callback),
            logic_update_frame: 1000.0 / 60.0,
            command_sender: sender,
            command_receiver: receiver,
        };

        runtime.data = Some((runtime.callback.init)(&mut runtime.context));

        runtime
    }

    pub fn start(mut self) {
        self.run();
    }

    pub fn get_command_sender(&self) -> Sender<RuntimeCommand> {
        self.command_sender.clone()
    }

    fn run(mut self) {
        let context = &mut self.context;
        let callback = self.callback;
        let data = self.data.as_mut().unwrap();

        let mut last_update_time = Instant::now();

        self.event_loop.run(|event, _, control_flow| {
            while let Ok(message) = self.command_receiver.try_recv() {
                (callback.runtime_command)(context, data, message);
            }

            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                    }
                    WindowEvent::Resized(size) => {
                        if size.width >= 4294967295 || size.height >= 4294967295{
                            return;
                        }
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

    pub fn on_runtime_command(&mut self,
                              callback: fn(&mut RuntimeContext, &mut AppData, RuntimeCommand),
    ) {
        self.callback.runtime_command = callback;
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

impl<AppData, RuntimeCommand> RuntimeCallbackFunctions<AppData, RuntimeCommand> {
    pub fn new(init: fn(&mut RuntimeContext) -> AppData) -> Self {
        Self {
            init,
            update: |_, _, _| {},
            render: |_, _, _| {},
            resize: |_, _, _| {},
            key_input: |_, _, _, _| {},
            runtime_command: |_, _, _| {},
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
