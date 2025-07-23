pub mod handle;
pub mod pipeline;
pub mod texture;
pub mod uniforms;

use std::sync::Weak;
use glam::Vec2;

use iced_core::{Event, Length, Point, Rectangle, mouse};
use iced_wgpu::wgpu;
use iced_widget::shader;

use handle::SurfaceInner;
use pipeline::Pipeline;
use uniforms::UniformsRaw;

pub fn texture<'a, Message: 'a>(
    buffer: &'a handle::Surface,
    controls: &'a Controls,
) -> TextureCanvas<'a, Message> {
    TextureCanvas::new(buffer, controls)
}

pub struct TextureCanvas<'a, Message> {
    buffer: &'a handle::Surface,
    controls: &'a Controls,
    width: Length,
    height: Length,
    on_drag: Option<Box<dyn Fn(Point) -> Message + 'a>>,
    on_zoom: Option<Box<dyn Fn(f32) -> Message + 'a>>,
}

impl<'a, Message> TextureCanvas<'a, Message> {
    pub fn new(buffer: &'a handle::Surface, controls: &'a Controls) -> Self {
        Self {
            buffer,
            controls,
            on_drag: None,
            on_zoom: None,
            width: Length::Fill,
            height: Length::Fill,
        }
    }

    /// Set the `width` of the custom [`TextureCanvas`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Set the `height` of the [`TextureCanvas`].
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    pub fn on_drag(mut self, on_drag: impl Fn(Point) -> Message + 'a) -> Self {
        self.on_drag = Some(Box::new(on_drag));
        self
    }

    pub fn on_zoom(mut self, on_zoom: impl Fn(f32) -> Message + 'a) -> Self {
        self.on_zoom = Some(Box::new(on_zoom));
        self
    }
}

impl<'a, Message, Theme, Renderer> From<TextureCanvas<'a, Message>>
    for iced_core::Element<'a, Message, Theme, Renderer>
where
    Message: 'a + Clone,
    Renderer: iced_wgpu::primitive::Renderer,
{
    fn from(value: TextureCanvas<'a, Message>) -> Self {
        let width = value.width;
        let height = value.height;
        shader(value).width(width).height(height).into()
    }
}

impl<'a, Message> shader::Program<Message> for TextureCanvas<'a, Message> {
    type State = State;
    type Primitive = Primitive;

    fn draw(
        &self,
        state: &Self::State,
        cursor: mouse::Cursor,
        bounds: Rectangle,
    ) -> Self::Primitive {
        Self::Primitive::new(
            *self.controls,
            self.buffer,
            state.canvas_offset,
            state.zoom.clamp(1.0, 100.),
        )
    }

    fn update(
        &self,
        state: &mut Self::State,
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<shader::Action<Message>> {
        if !cursor.is_over(bounds) {
            return None;
        }
        let mut action = None;
        // shader::Action::publish(message)

        if let mouse::Cursor::Available(mouse_pos) = cursor {
            if state.grabbing {
                let scale = state.zoom.clamp(1.0, 5.0);
                match state.canvas_grab {
                    Some(pos) => {
                        state.canvas_offset = Vec2::new(mouse_pos.x, mouse_pos.y) / scale - pos;
                        if let Some(on_click) = self.on_drag.as_ref() {
                            todo!()
                            // action = Some(shader::Action::publish(on_click()))
                        } else {
                            action = Some(shader::Action::request_redraw());
                        }
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

#[derive(Default, Clone)]
pub struct State {
    canvas_grab: Option<glam::Vec2>,
    grabbing: bool,
    canvas_offset: glam::Vec2,
    zoom: f32,
}

#[derive(Debug)]
pub struct Primitive {
    controls: Controls,
    surface: Weak<SurfaceInner>,
    offset: glam::Vec2,
    zoom_override: f32,
}

impl Primitive {
    pub fn new(
        controls: Controls,
        pixmap: &handle::Surface,
        offset: glam::Vec2,
        zoom_override: f32,
    ) -> Self {
        Self {
            controls,
            surface: pixmap.create_weak(),
            offset,
            zoom_override,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Controls {
    pub scale: f32,
    pub center: Vec2,
}

impl Default for Controls {
    fn default() -> Self {
        Self {
            scale: 1.0,
            center: Default::default(),
        }
    }
}

impl shader::Primitive for Primitive {
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

        let scale = self.zoom_override;
        // TODO: recreate texture if sizes differ
        let size = pipeline.texture.size;

        let uniforms = UniformsRaw::new(
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
            surface.size(),
        );

        pipeline.uniform.upload(queue, uniforms);

        surface.run_if_modified_or(just_created, |data| {
            pipeline.texture.upload(queue, &data);
        });
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
