pub mod pipeline;
pub mod texture;
pub mod uniforms;

use crate::shader::surface::Surface;

use pipeline::Pipeline;
use uniforms::UniformsRaw;

use iced_core::Rectangle;
use iced_wgpu::wgpu;
use iced_widget::shader;

use std::sync::Weak;

#[derive(Debug)]
pub struct Primitive<Buffer: Surface> {
    surface: Weak<Buffer>,
    offset: glam::Vec2,
    scale: f32,
}

impl<Buffer: Surface> Primitive<Buffer> {
    pub fn new(pixmap: Weak<Buffer>, offset: glam::Vec2, scale: f32) -> Self {
        Self {
            surface: pixmap,
            offset,
            scale,
        }
    }
}

impl<Buffer: Surface> shader::Primitive for Primitive<Buffer> {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        storage: &mut shader::Storage,
        bounds: &Rectangle,
        _viewport: &shader::Viewport,
    ) {
        let Some(surface) = self.surface.upgrade() else {
            return;
        };

        let mut just_created = false;
        if !storage.has::<Pipeline>() {
            just_created = true;
            storage.store(Pipeline::new(device, format, &surface));
        }

        let pipeline = storage.get_mut::<Pipeline>().unwrap();

        let texture_size = pipeline.texture.size;

        if surface.width() != texture_size.width || surface.height() != texture_size.height {
            *pipeline = Pipeline::new(device, format, &surface);
            just_created = true;
        }

        let scale = self.scale;

        pipeline.uniform.upload(
            queue,
            UniformsRaw::new(self.offset, scale, bounds.size(), surface.size()),
        );

        if just_created {
            pipeline
                .texture
                .upload(queue, surface.width(), surface.height(), surface.data());
        } else {
            surface.run_if_modified(|width, height, buffer| {
                pipeline.texture.upload(queue, width, height, buffer);
            });
        }
    }

    fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        storage: &shader::Storage,
        target: &wgpu::TextureView,
        clip_bounds: &Rectangle<u32>,
    ) {
        if let Some(pipeline) = storage.get::<Pipeline>() {
            pipeline.render(target, clip_bounds, encoder);
        }
    }
}
