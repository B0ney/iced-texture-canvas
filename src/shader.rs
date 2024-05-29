mod texture;
pub mod uniforms;
pub mod pipeline;

use glam::Vec2;
use iced::widget::shader::{self, wgpu};
use iced::{mouse, Size};
use pipeline::Pipeline;
use uniforms::Uniforms;

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

const ZOOM_PIXELS_FACTOR: f32 = 200.0;

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

#[derive(Debug, Default, Clone, Copy)]
pub struct Controls {
    pub zoom: f32,
    pub center: Vec2,
}

impl Controls {
    fn scale(&self) -> f32 {
        1.0 / 2.0_f32.powf(self.zoom) / ZOOM_PIXELS_FACTOR
    }
}

impl shader::Primitive for BitmapPrimatrive {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        storage: &mut shader::Storage,
        _bounds: &iced::Rectangle,
        _viewport: &shader::Viewport,
    ) {
        if !storage.has::<Pipeline>() {
            storage.store(Pipeline::new(device, format, &self.pixmap));
        }

        // TODO : recreate texture if texture size changed
        let pipeline = storage.get_mut::<Pipeline>().unwrap();

        let texture = &self.pixmap;

        pipeline.update(
            queue,
            texture,
            Uniforms {
                center: self.controls.center,
                // scale: self.controls.scale(),
            },
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
