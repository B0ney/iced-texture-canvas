use iced::widget::shader::wgpu;

use super::{
    texture,
    uniforms::{self, Uniform, Uniforms},
};

pub struct Pipeline {
    pipeline: wgpu::RenderPipeline,
    uniform: uniforms::Uniform,
    texture: texture::Texture,
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

        let texture = texture::Texture::new(device, pixmap.size(), None);
        let uniform = Uniform::new(device);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline layout"),
            bind_group_layouts: &[&texture.layout, &uniform.layout], // order matters
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
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
        }
    }

    /// TODO: apply uniform buffer to transform, clip texture view.
    pub fn update(&mut self, queue: &wgpu::Queue, pixmap: &texture::Pixmap, uniforms: Uniforms) {
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

        pass.draw(0..3, 0..1)
    }
}
