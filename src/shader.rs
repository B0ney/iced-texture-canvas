pub mod handle;
pub mod pipeline;
pub mod texture;
pub mod uniforms;

use glam::Vec2;
use std::sync::Weak;

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
    on_pressed: Option<Box<dyn Fn(Point) -> Message + 'a>>,
    on_move: Option<Box<dyn Fn(Point) -> Message + 'a>>,
    on_release: Option<Box<dyn Fn(Point) -> Message + 'a>>,
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
            on_pressed: None,
            on_move: None,
            on_release: None,
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

    // TODO include which button was pressed.
    pub fn on_press(mut self, on_press: impl Fn(Point) -> Message + 'a) -> Self {
        self.on_pressed = Some(Box::new(on_press));
        self
    }

    pub fn on_move(mut self, on_move: impl Fn(Point) -> Message + 'a) -> Self {
        self.on_move = Some(Box::new(on_move));
        self
    }

    pub fn on_release(mut self, on_release: impl Fn(Point) -> Message + 'a) -> Self {
        self.on_release = Some(Box::new(on_release));
        self
    }
}

impl<'a, Message, Theme, Renderer> From<TextureCanvas<'a, Message>>
    for iced_core::Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
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
            self.buffer,
            state.canvas_offset,
            state.zoom.clamp(1.0, 100.),
        )
    }

    fn mouse_interaction(
        &self,
        _state: &Self::State,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if _state.mouse_over_image {
            mouse::Interaction::Crosshair
        } else {
            mouse::Interaction::None
        }
    }

    fn update(
        &self,
        state: &mut Self::State,
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<shader::Action<Message>> {
        if !cursor.is_over(bounds) {
            state.reset();
            return None;
        }

        if let mouse::Cursor::Available(mouse_pos) = cursor {
            let glam::Vec2 { x, y } = state.canvas_offset;

            let canvas_bounds = Rectangle {
                x,
                y,
                width: self.buffer.width() as f32 * state.zoom,
                height: self.buffer.height() as f32 * state.zoom,
            };

            if canvas_bounds.contains(mouse_pos) {
                state.mouse_over_image = true;
            } else {
                state.mouse_over_image = false;
            }

            fn to_canvas_coords(mouse: Point, offset: Vec2, scale: f32) -> Point {
                let mouse = glam::vec2(mouse.x, mouse.y);
                let Vec2 { x, y } = (mouse - offset) / scale;
                Point { x, y }
            }

            match event {
                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                    state.mouse_down = true;

                    if state.mouse_over_image {
                        if let Some(on_press) = &self.on_pressed {
                            return Some(shader::Action::publish(on_press(to_canvas_coords(
                                mouse_pos,
                                state.canvas_offset,
                                state.zoom,
                            ))));
                        }
                    }
                }

                Event::Mouse(mouse::Event::CursorMoved { position }) => {
                    let mouse_pos = *position;

                    if state.mouse_over_image && state.mouse_down {
                        if let Some(on_move) = &self.on_move {
                            return Some(shader::Action::publish(on_move(to_canvas_coords(
                                mouse_pos,
                                state.canvas_offset,
                                state.zoom,
                            ))));
                        }
                    }

                    if state.grabbing {
                        match state.canvas_grab {
                            Some(pos) => {
                                state.canvas_offset = Vec2::new(mouse_pos.x, mouse_pos.y) - pos
                            }
                            None => {
                                let position = Vec2::new(mouse_pos.x, mouse_pos.y);
                                state.canvas_grab = Some(position - state.canvas_offset);
                            }
                        }

                        return Some(shader::Action::request_redraw());
                    }
                }

                Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                    state.mouse_down = false;

                    if state.mouse_over_image {
                        if let Some(on_release) = &self.on_release {
                            return Some(shader::Action::publish(on_release(to_canvas_coords(
                                mouse_pos,
                                state.canvas_offset,
                                state.zoom,
                            ))));
                        }
                    }
                }

                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Middle)) => {
                    state.grabbing = true;
                }
                Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Middle)) => {
                    state.grabbing = false;
                    state.canvas_grab = None;
                }
                // TODO
                Event::Mouse(mouse::Event::WheelScrolled { delta }) => match delta {
                    mouse::ScrollDelta::Lines { x, y } => {
                        // TODO: align the canvas to the mouse position when scaling.
                        // we calculate what % the cursor is from the canvas on both axes.
                        // 0% = far left, or top
                        // 100% = far right, or bottm
                        //
                        // after scaling, we adjust the offset of the canvas to match this.
                        // println!("{}", y);
                        state.zoom = (state.zoom + y).clamp(1.0, 5.0);
                        state.canvas_offset = Vec2::new(mouse_pos.x, mouse_pos.y);

                        return Some(shader::Action::request_redraw());
                    }

                    mouse::ScrollDelta::Pixels { y, .. } => {
                        println!("h;");
                    }
                },
                _ => (),
            }
        } else {
            state.reset();
        };

        None
    }
}

#[derive(Default, Clone)]
pub struct State {
    canvas_grab: Option<glam::Vec2>,
    grabbing: bool,
    canvas_offset: glam::Vec2,
    zoom: f32,
    mouse_over_image: bool,
    mouse_down: bool,
}

impl State {
    pub fn reset(&mut self) {
        self.mouse_over_image = false;
        self.grabbing = false;
        self.canvas_grab = None;
        self.mouse_down = false;
    }
}

#[derive(Debug)]
pub struct Primitive {
    surface: Weak<SurfaceInner>,
    offset: glam::Vec2,
    scale: f32,
}

impl Primitive {
    pub fn new(pixmap: &handle::Surface, offset: glam::Vec2, scale: f32) -> Self {
        Self {
            surface: pixmap.create_weak(),
            offset,
            scale,
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

        let texture_size = pipeline.texture.size;

        if surface.width() != texture_size.width || surface.height() != texture_size.height {
            *pipeline = Pipeline::new(device, format, &surface);
        }

        let scale = self.scale;

        pipeline.uniform.upload(
            queue,
            UniformsRaw::new(self.offset, scale, bounds.size(), surface.size()),
        );

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
