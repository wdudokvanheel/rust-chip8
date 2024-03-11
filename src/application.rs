use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::mpsc::Sender;
use std::thread;

use wgpu::{BindGroup, Buffer, Device, RenderPipeline, ShaderModule, Texture, TextureFormat};
use wgpu::util::DeviceExt;
use winit::keyboard::KeyCode;

use crate::application::AppCommand::RESET;
use crate::chip8::{Chip8, load_rom};
use crate::wgpu_runtime::{RuntimeContext, Vertex, WgpuRuntime};
use crate::wgpu_runtime::wgpu_math::Vec2i;

#[derive(Debug)]
pub enum AppCommand {
    RESET
}

pub struct RuntimeData {
    chip8: Chip8,
    render_pipeline: RenderPipeline,
    uniform_buffer: Buffer,
    bind_group: BindGroup,
    clockspeed: f32,
    elapsed_time: f32,
    key_map: HashMap<KeyCode, u8>,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct ShaderUniform {
    value: [u32; 2048],
}

pub fn start_application() -> WgpuRuntime<RuntimeData, AppCommand> {
    println!("Chip 8 Emulator by Bitechular Innovations");

    let mut runtime = WgpuRuntime::<_, AppCommand>::new(
        "Chip 8 Emulator - Bitechular Innovations",
        Vec2i::new(1280, 640),
        |context| {
            let rom = load_rom();

            let mut device = Chip8::new();
            device.set_rom(&rom);
            let shader = create_shader(&context.gfx.device);
            let (render_pipeline, uniform_buffer, bind_group) = create_pipeline
                (&context.gfx.device, &shader, context.gfx.texture_format);

            let mut key_map: HashMap<KeyCode, u8> = HashMap::new();

            key_map.insert(KeyCode::Digit1, 0x1);
            key_map.insert(KeyCode::Digit2, 0x2);
            key_map.insert(KeyCode::Digit3, 0x3);
            key_map.insert(KeyCode::Digit4, 0xC);

            key_map.insert(KeyCode::KeyQ, 0x4);
            key_map.insert(KeyCode::KeyW, 0x5);
            key_map.insert(KeyCode::KeyE, 0x6);
            key_map.insert(KeyCode::KeyR, 0xD);

            key_map.insert(KeyCode::KeyA, 0x7);
            key_map.insert(KeyCode::KeyS, 0x8);
            key_map.insert(KeyCode::KeyD, 0x9);
            key_map.insert(KeyCode::KeyF, 0xE);

            key_map.insert(KeyCode::KeyZ, 0xA);
            key_map.insert(KeyCode::KeyX, 0x0);
            key_map.insert(KeyCode::KeyC, 0xB);
            key_map.insert(KeyCode::KeyV, 0xF);

            RuntimeData {
                chip8: device,
                render_pipeline,
                uniform_buffer,
                bind_group,
                elapsed_time: 0.0,
                clockspeed: 1000.0 / 700.0,
                key_map,
            }
        },
    );

    runtime.on_runtime_command(on_message);
    runtime.on_render(render);
    runtime.on_update(update);
    runtime.on_key_event(input);

    return runtime;
}

fn on_message(_app: &mut RuntimeContext, data: &mut RuntimeData, command: AppCommand) {
    match command {
        RESET => {
            log::warn!("Got reset message");
        }
    }
}

fn update(_app: &mut RuntimeContext, data: &mut RuntimeData, elapsed: f32) {
    data.elapsed_time += elapsed;

    data.chip8.update();
    while data.elapsed_time >= data.clockspeed {
        data.elapsed_time -= data.clockspeed;
        data.chip8.cycle();
    }
}


fn input(_app: &mut RuntimeContext, data: &mut RuntimeData, keycode: KeyCode, pressed: bool) {
    if let Some(key) = data.key_map.get(&keycode) {
        data.chip8.set_input(*key, pressed);
    }
}

fn render(context: &mut RuntimeContext, data: &mut RuntimeData, target: &Texture) {
    let mut encoder = context.gfx.device.create_command_encoder
    (&wgpu::CommandEncoderDescriptor { label: None });
    {
        let view = target.create_view(&wgpu::TextureViewDescriptor::default());

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
        //
        context.gfx.queue.write_buffer(&data.uniform_buffer, 0, bytemuck::cast_slice
            (&[ShaderUniform::from_display(data.chip8.display)]));
        rpass.set_bind_group(0, &data.bind_group, &[]);
        rpass.set_pipeline(&data.render_pipeline);
        rpass.set_vertex_buffer(0, context.gfx.vertex_buffer.slice(..));
        rpass.set_index_buffer(context.gfx.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        rpass.draw_indexed(0..6, 0, 0..1);
    }
    context.gfx.queue.submit(Some(encoder.finish()));
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

fn create_shader(device: &Device) -> ShaderModule {
    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
    })
}

impl ShaderUniform {
    pub(crate) fn new() -> Self {
        ShaderUniform {
            value: [0; 2048]
        }
    }

    pub(crate) fn from_display(display: [[bool; 64]; 32]) -> Self {
        let mut n = ShaderUniform {
            value: [0; 2048],
        };

        for (row_index, row) in display.iter().enumerate() {
            for (col_index, &col_value) in row.iter().enumerate() {
                if col_value {
                    n.value[row_index * 64 + col_index] = 1;
                }
            }
        }

        return n;
    }
}
