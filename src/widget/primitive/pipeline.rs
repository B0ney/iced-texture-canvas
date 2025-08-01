use super::texture;
use super::uniforms::{self, Uniform};
use crate::widget::Surface;

use iced_core::Rectangle;
use iced_wgpu::wgpu;

pub(crate) struct Pipeline {
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
                    blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
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

    pub fn render(
        &self,
        target: &wgpu::TextureView,
        viewport: &Rectangle<u32>,
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
                depth_slice: None,
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

        pass.draw(0..6, 0..1)
    }
}
