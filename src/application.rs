use std::borrow::Cow;
use std::collections::HashMap;

use bytemuck::cast_slice;
use wgpu::{BindGroup, Buffer, Device, RenderPipeline, ShaderModule, Texture, TextureFormat};
use wgpu::util::DeviceExt;
use winit::keyboard::KeyCode;

use crate::application::AppCommand::{LOAD_ROM, RESET};
use crate::chip8::{Chip8, Chip8Rom, QuirkConfig};
use crate::wgpu_runtime::{RuntimeContext, Vertex, WgpuRuntime};
use crate::wgpu_runtime::wgpu_math::Vec2i;

pub enum AppCommand {
    RESET,
    LOAD_ROM(u8),
}

pub struct RuntimeData {
    chip8: Chip8,
    render_pipeline: RenderPipeline,
    uniform_buffer: Buffer,
    bind_group: BindGroup,
    clockspeed: f32,
    elapsed_time: f32,
    key_map: HashMap<KeyCode, u8>,
    current_rom: u8,
    roms: Vec<Chip8Rom>,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct ShaderUniform {
    value: [u32; 2048],
    width: f32,
    height: f32,
    padding: [u8; 8],
}

pub fn start_application() -> WgpuRuntime<RuntimeData, AppCommand> {
    println!("Chip 8 Emulator by Bitechular Innovations");

    let mut runtime = WgpuRuntime::<_, AppCommand>::new(
        "Chip 8 Emulator - Bitechular Innovations",
        Vec2i::new(640, 320),
        |context| {
            let roms = create_rom_list();
            let mut device = roms[0].to_device();

            let shader = create_shader(&context.gfx.device);
            let (render_pipeline, uniform_buffer, bind_group) = create_pipeline
                (&context.gfx.device, &shader, context.gfx.texture_format);

            let key_map = create_key_map();

            RuntimeData {
                chip8: device,
                render_pipeline,
                uniform_buffer,
                bind_group,
                elapsed_time: 0.0,
                clockspeed: 1000.0 / 700.0,
                key_map,
                current_rom: 0,
                roms,
            }
        },
    );

    runtime.on_runtime_command(on_message);
    runtime.on_render(render);
    runtime.on_update(update);
    runtime.on_key_event(input);

    return runtime;
}

fn create_key_map() -> HashMap<KeyCode, u8> {
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
    key_map
}

fn create_rom_list() -> Vec<Chip8Rom> {
    vec![
        Chip8Rom::new("Test: IBM Logo", include_bytes!("roms/tests/ibm.ch8").to_vec()),
        Chip8Rom::new("Test: Corax Plus", include_bytes!("roms/tests/corax.plus.ch8").to_vec()),
        Chip8Rom::new_quirks("Test: Quirks", include_bytes!("roms/tests/quirks.ch8").to_vec(), QuirkConfig::create(true, true)),
        Chip8Rom::new_quirks("Test: Flags", include_bytes!("roms/tests/flags.ch8").to_vec(),
                             QuirkConfig::create(false, false)),
        Chip8Rom::new("Test: Keypad", include_bytes!("roms/tests/keypad.ch8").to_vec()),
        Chip8Rom::new("15 Puzzle", include_bytes!("roms/games/15puzzle.ch8").to_vec()),
        Chip8Rom::new_quirks("Blinky", include_bytes!("roms/games/blinky.ch8").to_vec(), QuirkConfig::create(true, false)),
        Chip8Rom::new("Blitz", include_bytes!("roms/games/blitz.ch8").to_vec()),
        Chip8Rom::new("Brix", include_bytes!("roms/games/brix.ch8").to_vec()),
        Chip8Rom::new("Guess", include_bytes!("roms/games/guess.ch8").to_vec()),
        Chip8Rom::new("Hidden", include_bytes!("roms/games/hidden.ch8").to_vec()),
        Chip8Rom::new("Invaders", include_bytes!("roms/games/invaders.ch8").to_vec()),
        Chip8Rom::new("Maze", include_bytes!("roms/games/maze.ch8").to_vec()),
        Chip8Rom::new("Merlin", include_bytes!("roms/games/merlin.ch8").to_vec()),
        Chip8Rom::new("Missile", include_bytes!("roms/games/missile.ch8").to_vec()),
        Chip8Rom::new("Pong", include_bytes!("roms/games/pong.ch8").to_vec()),
        Chip8Rom::new("Pong2", include_bytes!("roms/games/pong2.ch8").to_vec()),
        Chip8Rom::new("Puzzle", include_bytes!("roms/games/puzzle.ch8").to_vec()),
        Chip8Rom::new("Syzygy", include_bytes!("roms/games/syzygy.ch8").to_vec()),
        Chip8Rom::new("Tank", include_bytes!("roms/games/tank.ch8").to_vec()),
        Chip8Rom::new("Tetris", include_bytes!("roms/games/tetris.ch8").to_vec()),
        Chip8Rom::new("Tictac", include_bytes!("roms/games/tictac.ch8").to_vec()),
        Chip8Rom::new("UFO", include_bytes!("roms/games/ufo.ch8").to_vec()),
        Chip8Rom::new("Vbrix", include_bytes!("roms/games/vbrix.ch8").to_vec()),
        Chip8Rom::new("Vers", include_bytes!("roms/games/vers.ch8").to_vec()),
        Chip8Rom::new("Wipeoff", include_bytes!("roms/games/wipeoff.ch8").to_vec()),
    ]
}

fn on_message(_app: &mut RuntimeContext, data: &mut RuntimeData, command: AppCommand) {
    match command {
        RESET => {
            data.reset_device();
        }
        LOAD_ROM(id) => {
            data.set_rom(id);
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

        let mut disp = [[false; 64]; 32];
        disp[0][0] = true;
        // disp[0][1] = true;
        // disp[0][2] = true;
        // disp[0][3] = true;
        disp[31][63] = true;
        disp[31][62] = true;
        disp[31][61] = true;
        disp[31][60] = true;
        disp[31][59] = true;

        context.gfx.queue.write_buffer(
            &data.uniform_buffer,
            0,
            cast_slice(&[ShaderUniform::from_display(data.chip8.display, context.gfx.surface_config.width, context.gfx.surface_config.height)]),
            // cast_slice(&[ShaderUniform::from_display(disp, context.gfx.surface_config.width, context
            //     .gfx.surface_config.height)]),
        );
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
            contents: cast_slice(&[uniform]),
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

impl RuntimeData {
    pub fn reset_device(&mut self) {
        self.chip8 = self.roms[self.current_rom as usize].to_device();
    }

    pub fn set_rom(&mut self, id: u8) {
        self.current_rom = id;
        self.reset_device();
    }

    pub fn rom_list(&self) -> Vec<String> {
        self.roms.iter().map(|rom| rom.name.clone()).collect()
    }
}

impl ShaderUniform {
    pub fn new() -> Self {
        ShaderUniform {
            width: 320.0,
            height: 160.0,
            value: [0; 2048],
            padding: [0; 8],
        }
    }

    pub fn from_display(display: [[bool; 64]; 32], width: u32, height: u32) -> Self {
        // log::warn!("Display for size: {}x{}", width, height);
        let mut n = ShaderUniform {
            value: [0; 2048],
            width: width as f32,
            height: height as f32,
            padding: [0; 8],
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

