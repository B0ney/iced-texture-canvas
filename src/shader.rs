pub mod pipeline;
mod texture;
pub mod uniforms;

use glam::Vec2;
use iced::widget::shader;
use iced::{mouse, Event, Size, Vector};
use iced::{wgpu, Point};
use pipeline::Pipeline;
use uniforms::UniformsRaw;

#[derive(Default, Clone)]
pub struct State {
    canvas_grab: Option<glam::Vec2>,
    grabbing: bool,
    canvas_offset: glam::Vec2,
    zoom: f32,
}

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
    type State = State;

    type Primitive = BitmapPrimatrive;

    fn draw(
        &self,
        state: &Self::State,
        cursor: mouse::Cursor,
        bounds: iced::Rectangle,
    ) -> Self::Primitive {
        Self::Primitive::new(
            self.controls,
            self.buffer.clone(),
            state.canvas_offset,
            state.zoom.clamp(1.0, 100.),
        )
    }

    fn update(
        &self,
        state: &mut Self::State,
        event: &iced::Event,
        bounds: iced::Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<shader::Action<Message>> {
        if !cursor.is_over(bounds) {
            return None;
        }
        let mut action = None;

        if let mouse::Cursor::Available(mouse_pos) = cursor {
            if state.grabbing {
                let scale = state.zoom.clamp(1.0, 5.0);
                match state.canvas_grab {
                    Some(pos) => {
                        state.canvas_offset = Vec2::new(mouse_pos.x, mouse_pos.y) / scale - pos;
                        action = Some(shader::Action::request_redraw());
                    }
                    None => {
                        let position = Vec2::new(mouse_pos.x, mouse_pos.y);
                        state.canvas_grab = Some(position / scale - state.canvas_offset);
                        action = Some(shader::Action::request_redraw());
                    }
                }
            }

            match event {
                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Middle)) => {
                    state.grabbing = true;
                }
                Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Middle)) => {
                    state.grabbing = false;
                    state.canvas_grab = None;
                }
                Event::Mouse(mouse::Event::WheelScrolled { delta }) => match delta {
                    mouse::ScrollDelta::Lines { x, y } => {
                        // TODO: align the canvas to the mouse position when scaling.
                        // we calculate what % the cursor is from the canvas on both axes.
                        // 0% = far left, or top
                        // 100% = far right, or bottm
                        //
                        // after scaling, we adjust the offset of the canvas to match this.
                        println!("{}", y);
                        state.zoom = (state.zoom + y).clamp(1.0, 5.0);
                        action = Some(shader::Action::request_redraw());
                        state.canvas_offset = Vec2::new(mouse_pos.x, mouse_pos.y) / state.zoom;
                    }
                    mouse::ScrollDelta::Pixels { y, .. } => {
                        println!("h;");
                    }
                },
                _ => (),
            }
        };

        action
    }
}

#[derive(Debug)]
pub struct BitmapPrimatrive {
    controls: Controls,
    pixmap: texture::Pixmap,
    offset: glam::Vec2,
    zoom_override: f32,
}

impl BitmapPrimatrive {
    pub fn new(
        controls: Controls,
        pixmap: texture::Pixmap,
        offset: glam::Vec2,
        zoom_override: f32,
    ) -> Self {
        Self {
            controls,
            pixmap,
            offset,
            zoom_override,
        }
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

        // let scale = self.controls.zoom;
        let scale = self.zoom_override;
        let size = pipeline.texture.size;

        // TODO: only update if dirty
        pipeline.update(
            queue,
            &self.pixmap,
            UniformsRaw::new(
                {
                    let center = self.controls.center;
                    let offset = self.offset;

                    let center_x = center.x / 2.;
                    let center_y = center.y / 2.;

                    let tex_width = size.width as f32 / 2.0;
                    let tex_height = size.height as f32 / 2.0;

                    let canvas_x = (center_x - (tex_width * scale)).ceil() + offset.x * scale;
                    let canvas_y = (center_y - (tex_height * scale)).ceil() + offset.y * scale;

                    (canvas_x, canvas_y).into()
                },
                scale,
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
        if let Some(pipeline) = storage.get::<Pipeline>() {
            pipeline.render(target, clip_bounds, encoder);
        }
    }
}
