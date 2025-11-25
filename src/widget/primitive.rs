pub mod pipeline;
pub mod texture;
pub mod uniforms;

use crate::widget::surface::Surface;

use pipeline::Pipeline as PipelineInner;
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

impl iced_wgpu::primitive::Pipeline for Pipeline {
    fn new(_device: &wgpu::Device, _queue: &wgpu::Queue, format: wgpu::TextureFormat) -> Self
    where
        Self: Sized,
    {
        Pipeline {
            pipeline: None,
            format,
        }
    }
}

pub struct Pipeline {
    pipeline: Option<PipelineInner>,
    format: wgpu::TextureFormat,
}

impl<Buffer: Surface> shader::Primitive for Primitive<Buffer> {
    type Pipeline = Pipeline;

    fn prepare(
        &self,
        Pipeline { pipeline, format }: &mut Self::Pipeline,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bounds: &Rectangle,
        _viewport: &shader::Viewport,
    ) {
        let Some(surface) = self.surface.upgrade() else {
            return;
        };

        let format = *format;

        let pipeline = pipeline.get_or_insert_with(|| {
            PipelineInner::new(
                device,
                format,
                &surface,
                self.generation,
                get_surface_ptr(&surface),
            )
        });

        let texture_size = pipeline.texture.size;

        let mut force_update = false;

        // TODO: Optimise this.
        if surface.width() != texture_size.width || surface.height() != texture_size.height {
            force_update = true;
            *pipeline = PipelineInner::new(
                device,
                format,
                &surface,
                self.generation,
                get_surface_ptr(&surface),
            );
        }

        // Update the texture when the widget tree changes
        if pipeline.generation != self.generation {
            force_update = true;
            pipeline.generation = self.generation;
        }

        // Update the texture if the pointer to the surface changes.
        //
        // This lets you swap multiple images.
        let new_surface_ptr = get_surface_ptr(&surface);
        if pipeline.surface_ptr != new_surface_ptr {
            force_update = true;
            pipeline.surface_ptr = new_surface_ptr
        }

        pipeline.uniform.upload(
            queue,
            UniformsRaw::new(self.offset, self.scale, bounds.size(), surface.size()),
        );

        if force_update {
            pipeline
                .texture
                .upload(queue, surface.width(), surface.height(), surface.data());
        } else {
            surface.run_if_modified(|width, height, buffer| {
                pipeline.texture.upload(queue, width, height, buffer);
            });
        }
    }

    fn draw(&self, renderer: &Self::Pipeline, pass: &mut wgpu::RenderPass) -> bool {
        if let Some(pipeline) = &renderer.pipeline {
            pipeline.draw(pass);
        }

        true
    }
}

fn get_surface_ptr<S: Surface>(surface: &Arc<S>) -> usize {
    Arc::as_ptr(surface) as *const () as usize
}
