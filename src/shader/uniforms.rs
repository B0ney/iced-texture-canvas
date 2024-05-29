use std::sync::Arc;

use glam::Vec2;
use iced::widget::shader::wgpu::{self, util::DeviceExt as _};

pub struct Uniform {
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    pub layout: Arc<wgpu::BindGroupLayout>,
}

impl Uniform {
    pub fn new(device: &wgpu::Device) -> Self {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("controls uniform"),
            contents: bytemuck::cast_slice(&[Uniforms::default()]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Uniform Bind Group"),
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

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("camera bind group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });

        Self {
            buffer,
            bind_group,
            layout: Arc::new(bind_group_layout),
        }
    }

    /// Upload uniform buffer
    pub fn upload(&self, queue: &wgpu::Queue, uniforms: Uniforms) {
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[uniforms]))
    }
}

/// camera uniforms
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, Default)]
#[repr(C)]
pub struct Uniforms {
    pub center: Vec2,
    pub scale: f32,
}

impl Uniforms {
    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Uniforms>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<Vec2>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32,
                },
            ],
        }
    }
}
