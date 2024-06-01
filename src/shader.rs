pub mod pipeline;
mod texture;
pub mod uniforms;

use glam::Vec2;
use iced::widget::shader::{self, wgpu};
use iced::{mouse, Size};
use pipeline::Pipeline;
use uniforms::UniformsRaw;

#[derive(Debug, Clone)]
pub struct Bitmap {
    pub controls: Controls,
    pub buffer: texture::Pixmap,
}

impl Bitmap {
    pub fn new(size: Size<u32>) -> Self {
        Self {
            controls: Controls::default(),
            buffer: texture::Pixmap::new(size.width, size.height),
        }
    }
}

impl<Message> shader::Program<Message> for Bitmap {
    type State = ();

    type Primitive = BitmapPrimatrive;

    fn draw(
        &self,
        state: &Self::State,
        cursor: mouse::Cursor,
        bounds: iced::Rectangle,
    ) -> Self::Primitive {
        Self::Primitive::new(self.controls, self.buffer.clone())
    }
}

#[derive(Debug)]
pub struct BitmapPrimatrive {
    controls: Controls,
    pixmap: texture::Pixmap,
}

impl BitmapPrimatrive {
    pub fn new(controls: Controls, pixmap: texture::Pixmap) -> Self {
        Self { controls, pixmap }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Controls {
    pub zoom: f32,
    pub center: Vec2,
}

impl Default for Controls {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            center: Default::default(),
        }
    }
}

impl shader::Primitive for BitmapPrimatrive {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        storage: &mut shader::Storage,
        bounds: &iced::Rectangle,
        _viewport: &shader::Viewport,
    ) {
        if !storage.has::<Pipeline>() {
            storage.store(Pipeline::new(device, format, &self.pixmap));
        }

        // TODO: recreate texture if texture size changed
        let pipeline = storage.get_mut::<Pipeline>().unwrap();

        pipeline.update(
            queue,
            &self.pixmap,
            UniformsRaw::new(
                self.controls.center,
                self.controls.zoom,
                bounds.size(),
                self.pixmap.size(),
            ),
        );
    }

    fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        storage: &shader::Storage,
        target: &wgpu::TextureView,
        clip_bounds: &iced::Rectangle<u32>,
    ) {
        let pipeline = storage.get::<Pipeline>().unwrap();
        pipeline.render(target, clip_bounds, encoder);
    }
}
