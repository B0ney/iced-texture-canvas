use super::texture;
use super::uniforms::{self, Uniform};
use crate::widget::Surface;

use iced_core::Rectangle;
use iced_wgpu::wgpu;

pub struct Pipeline {
    pipeline: wgpu::RenderPipeline,
    pub uniform: uniforms::Uniform,
    pub texture: texture::Texture,
    pub generation: u64,
    pub surface_ptr: usize,
}

impl Pipeline {
    pub fn new<Buffer: Surface>(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        pixmap: &Buffer,
        generation: u64,
        surface_ptr: usize,
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
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        Self {
            pipeline,
            uniform,
            texture,
            generation,
            surface_ptr,
        }
    }

    pub fn draw(&self, pass: &mut wgpu::RenderPass) {
        pass.set_pipeline(&self.pipeline);

        pass.set_bind_group(0, &self.texture.bind_group, &[]);
        pass.set_bind_group(1, &self.uniform.bind_group, &[]);

        pass.draw(0..6, 0..1)
    }
}
