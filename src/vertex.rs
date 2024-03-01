use bytemuck::Zeroable;

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

