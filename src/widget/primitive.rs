pub mod pipeline;
pub mod texture;
pub mod uniforms;

use crate::widget::surface::Surface;

use pipeline::Pipeline;
use uniforms::UniformsRaw;

use iced_core::Rectangle;
use iced_wgpu::wgpu;
use iced_widget::shader;

use std::sync::{Arc, Weak};

#[derive(Debug)]
pub struct Primitive<Buffer: Surface> {
    surface: Weak<Buffer>,
    offset: glam::Vec2,
    scale: f32,
    generation: u64,
}

impl<Buffer: Surface> Primitive<Buffer> {
    pub fn new(pixmap: Weak<Buffer>, offset: glam::Vec2, scale: f32, generation: u64) -> Self {
        Self {
            surface: pixmap,
            offset,
            scale,
            generation,
        }
    }
}

struct Storage {
    pipeline: Pipeline,
    generation: u64,
    surface_ptr: usize,
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

        let mut force_redraw = false;

        if !storage.has::<Storage>() {
            force_redraw = true;

            storage.store(Storage {
                pipeline: Pipeline::new(device, format, &surface),
                generation: self.generation,
                surface_ptr: get_surface_ptr(&surface),
            });
        }

        let Storage {
            pipeline,
            generation,
            surface_ptr,
        } = storage.get_mut::<Storage>().unwrap();

        let texture_size = pipeline.texture.size;

        // TODO: Optimise this.
        if surface.width() != texture_size.width || surface.height() != texture_size.height {
            force_redraw = true;
            *pipeline = Pipeline::new(device, format, &surface);
        }

        if *generation != self.generation {
            force_redraw = true;
            *generation = self.generation;
        }

        let new_surface_ptr = get_surface_ptr(&surface);
        if *surface_ptr != new_surface_ptr {
            force_redraw = true;
            *surface_ptr = new_surface_ptr
        }

        pipeline.uniform.upload(
            queue,
            UniformsRaw::new(self.offset, self.scale, bounds.size(), surface.size()),
        );

        if force_redraw {
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

fn get_surface_ptr<S: Surface>(surface: &Arc<S>) -> usize {
    Arc::as_ptr(surface) as *const u8 as usize
}
