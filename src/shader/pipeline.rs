use iced::widget::shader::wgpu::{self, util::DeviceExt};

use super::texture;
use super::uniforms::{self, Uniform, UniformsRaw};

pub struct Pipeline {
    pipeline: wgpu::RenderPipeline,
    uniform: uniforms::Uniform,
    texture: texture::Texture,
    quad: Quad,
}

impl Pipeline {
    pub fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        pixmap: &texture::Pixmap,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Pipeline shader"),
            ..wgpu::include_wgsl!("shader.wgsl")
        });

        let texture = texture::Texture::new(device, pixmap.width(), pixmap.height());
        let uniform = Uniform::new(device);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline layout"),
            bind_group_layouts: &[&texture.bind_group_layout, &uniform.bind_group_layout], // order matters
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: 4 * 4,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    // 0: vec2 position
                    // 1: vec2 texture coordinates
                    attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::REPLACE,
                        alpha: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        Self {
            pipeline,
            uniform,
            texture,
            quad: Quad::new(device),
        }
    }

    pub fn update(&mut self, queue: &wgpu::Queue, pixmap: &texture::Pixmap, uniforms: UniformsRaw) {
        self.uniform.upload(queue, uniforms);
        self.texture.upload(queue, pixmap);
    }

    pub fn render(
        &self,
        target: &wgpu::TextureView,
        viewport: &iced::Rectangle<u32>,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Color"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        pass.set_pipeline(&self.pipeline);
        pass.set_viewport(
            viewport.x as f32,
            viewport.y as f32,
            viewport.width as f32,
            viewport.height as f32,
            0.0,
            1.0,
        );

        pass.set_bind_group(0, &self.texture.bind_group, &[]);
        pass.set_bind_group(1, &self.uniform.bind_group, &[]);

        pass.set_vertex_buffer(0, self.quad.slice());
        pass.draw(0..self.quad.vertices(), 0..1)
    }
}

/// Quad to draw our texture onto.
struct Quad(wgpu::Buffer);

impl Quad {
    const VERTEX: [[f32; 4]; 6] = [
        // Top-right triangle
        [-1.0, 1.0, 0.0, 0.0], // tl
        [1.0, 1.0, 1.0, 0.0],  // tr
        [1.0, -1.0, 1.0, 1.0], // br
        // Bottom-left triangle
        [1.0, -1.0, 1.0, 1.0],  // br
        [-1.0, -1.0, 0.0, 1.0], // bl
        [-1.0, 1.0, 0.0, 0.0],  // tl
    ];

    const VERTICES: u32 = Self::VERTEX.len() as u32;

    fn new(device: &wgpu::Device) -> Self {
        Self(
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Quad Vertex buffer"),
                contents: bytemuck::cast_slice(&Self::VERTEX),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            }),
        )
    }

    fn slice(&self) -> wgpu::BufferSlice {
        self.0.slice(..)
    }

    fn vertices(&self) -> u32 {
        Self::VERTICES
    }
}
