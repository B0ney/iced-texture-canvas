pub mod pipeline;
pub mod style;
pub mod surface;
pub mod texture;
pub mod uniforms;

use style::{Catalog, Status, Style, StyleFn};

use pipeline::Pipeline;
use surface::{Surface, SurfaceHandler};
use uniforms::UniformsRaw;

use glam::Vec2;
use std::fmt::Debug;
use std::sync::Weak;

use iced_core::{
    Border, Element, Event, Layout, Length, Point, Rectangle, Shadow, Shell, Size, Widget, layout,
    mouse, renderer, widget, window,
};
use iced_wgpu::wgpu;
use iced_widget::shader;

const MIN_SCALE: f32 = 1.0; //0.05;
const MAX_SCALE: f32 = 1600.0;

pub fn texture<'a, Message, Theme, Handler>(
    buffer: &'a Handler,
) -> TextureCanvas<'a, Message, Theme, Handler>
where
    Message: 'a + Clone,
    Theme: Catalog,
    Handler: SurfaceHandler,
{
    TextureCanvas::new(buffer)
}

pub struct TextureCanvas<'a, Message, Theme, Handler>
where
    Theme: Catalog,
{
    buffer: &'a Handler,
    width: Length,
    height: Length,

    class: Theme::Class<'a>,

    on_grab: Option<Box<dyn Fn() -> Message + 'a>>,
    on_zoom: Option<Box<dyn Fn(f32) -> Message + 'a>>,
    on_pressed: Option<Box<dyn Fn(Point, mouse::Button) -> Message + 'a>>,
    on_move: Option<Box<dyn Fn(Point) -> Message + 'a>>,
    on_release: Option<Box<dyn Fn(Point, mouse::Button) -> Message + 'a>>,
    on_enter: Option<Message>,
    on_exit: Option<Message>,

    interaction: Option<mouse::Interaction>,
}

impl<'a, Message, Theme, Handler> TextureCanvas<'a, Message, Theme, Handler>
where
    Message: Clone,
    Theme: style::Catalog,
    Handler: SurfaceHandler,
{
    pub fn new(buffer: &'a Handler) -> Self {
        Self {
            buffer,
            width: Length::Fill,
            height: Length::Fill,
            on_grab: None,
            on_zoom: None,
            on_pressed: None,
            on_move: None,
            on_release: None,
            on_enter: None,
            on_exit: None,
            interaction: None,
            class: Theme::default(),
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

    #[must_use]
    pub fn style(mut self, style: impl Fn(&Theme, Status) -> Style + 'a) -> Self
    where
        Theme::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        self.class = (Box::new(style) as StyleFn<'a, Theme>).into();
        self
    }

    #[must_use]
    pub fn class(mut self, class: impl Into<Theme::Class<'a>>) -> Self {
        self.class = class.into();
        self
    }

    pub fn on_drag(mut self, on_drag: impl Fn() -> Message + 'a) -> Self {
        self.on_grab = Some(Box::new(on_drag));
        self
    }

    pub fn on_zoom(mut self, on_zoom: impl Fn(f32) -> Message + 'a) -> Self {
        self.on_zoom = Some(Box::new(on_zoom));
        self
    }

    pub fn on_press(mut self, on_press: impl Fn(Point, mouse::Button) -> Message + 'a) -> Self {
        self.on_pressed = Some(Box::new(on_press));
        self
    }

    pub fn on_move(mut self, on_move: impl Fn(Point) -> Message + 'a) -> Self {
        self.on_move = Some(Box::new(on_move));
        self
    }

    pub fn on_release(mut self, on_release: impl Fn(Point, mouse::Button) -> Message + 'a) -> Self {
        self.on_release = Some(Box::new(on_release));
        self
    }

    pub fn on_enter(mut self, on_enter: Message) -> Self {
        self.on_enter = Some(on_enter);
        self
    }

    pub fn on_exit(mut self, on_exit: Message) -> Self {
        self.on_exit = Some(on_exit);
        self
    }

    pub fn mouse_interaction(mut self, mouse_interaction: mouse::Interaction) -> Self {
        self.interaction = Some(mouse_interaction);
        self
    }
}

impl<'a, Message: Clone, Theme, Renderer, Handler: SurfaceHandler> Widget<Message, Theme, Renderer>
    for TextureCanvas<'a, Message, Theme, Handler>
where
    Renderer: iced_wgpu::primitive::Renderer,
    Theme: Catalog,
{
    fn tag(&self) -> widget::tree::Tag {
        struct Tag<T>(T);
        widget::tree::Tag::of::<Tag<State>>()
    }

    fn state(&self) -> widget::tree::State {
        widget::tree::State::new(State::default())
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn layout(
        &self,
        _tree: &mut widget::Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::atomic(limits, self.width, self.height)
    }

    fn draw(
        &self,
        tree: &widget::Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor_position: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let state = tree.state.downcast_ref::<State>();

        let Vec2 { x, y } = state.canvas_offset;
        let scale = state.scale;

        let texture_width = self.buffer.width() as f32 * scale;
        let texture_height = self.buffer.height() as f32 * scale;

        let style::Style {
            background,
            border_color,
            border_thickness,
            shadow,
        } = theme.style(
            &self.class,
            match state.is_hovered {
                true => style::Status::Hovered,
                false => style::Status::None,
            },
        );

        renderer.with_layer(bounds, |renderer| {
            // Draw the outlines, shadows and backdrop.
            renderer.fill_quad(
                renderer::Quad {
                    bounds: Rectangle {
                        x: x - border_thickness,
                        y: y - border_thickness,
                        width: texture_width + (border_thickness * 2.),
                        height: texture_height + (border_thickness * 2.),
                    },
                    border: Border {
                        color: border_color,
                        width: border_thickness,
                        radius: 0.0.into(),
                    },
                    shadow: Shadow {
                        color: shadow.color,
                        offset: shadow.offset * scale,
                        blur_radius: shadow.blur_radius * scale,
                    },
                    snap: false,
                    ..Default::default()
                },
                background,
            );

            // Draw the image.
            renderer.draw_primitive(
                bounds,
                Primitive::new(
                    self.buffer.create_weak(),
                    state.canvas_offset,
                    state.scale.clamp(MIN_SCALE, MAX_SCALE),
                ),
            );
        });
    }

    fn mouse_interaction(
        &self,
        tree: &widget::Tree,
        _layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        let state: &State = tree.state.downcast_ref::<State>();

        if !state.is_hovered {
            return mouse::Interaction::None;
        }

        self.interaction.unwrap_or_default()
    }

    fn update(
        &mut self,
        tree: &mut widget::Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn iced_core::Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();

        let state = tree.state.downcast_mut::<State>();

        if !cursor.is_over(bounds) {
            state.reset();
            return;
        }

        // TODO: move to inner
        if let mouse::Cursor::Available(mouse_pos) = cursor {
            let glam::Vec2 { x, y } = state.canvas_offset;

            let canvas_bounds = Rectangle {
                x: x + bounds.x,
                y: y + bounds.y,
                width: self.buffer.width() as f32 * state.scale,
                height: self.buffer.height() as f32 * state.scale,
            };

            if !state.grabbing {
                let was_hovered = state.is_hovered;
                state.is_hovered = cursor.is_over(canvas_bounds);

                match (was_hovered, state.is_hovered) {
                    (false, true) => {
                        if let Some(on_enter) = &self.on_enter {
                            shell.publish(on_enter.clone());
                        }
                    }

                    (true, false) => {
                        if let Some(on_exit) = &self.on_exit {
                            shell.publish(on_exit.clone());
                        }
                    }
                    _ => (),
                }
            }

            fn to_canvas_coords(
                bounds: Rectangle,
                mouse: Point,
                offset: Vec2,
                scale: f32,
            ) -> Point {
                let mouse = glam::vec2(mouse.x, mouse.y);
                let bounds_offset = glam::vec2(bounds.x, bounds.y) / scale;
                let Vec2 { x, y } = (mouse - offset) / scale - bounds_offset;

                Point { x, y }
            }

            match event {
                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Middle)) => {
                    state.grabbing = true;

                    if let Some(on_grab) = &self.on_grab {
                        shell.publish(on_grab());
                    }
                }
                Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Middle)) => {
                    state.grabbing = false;
                    state.canvas_grab = None;
                }

                Event::Mouse(mouse::Event::ButtonPressed(mouse_button)) => {
                    if let Some(on_press) = &self.on_pressed {
                        shell.publish(on_press(
                            to_canvas_coords(bounds, mouse_pos, state.canvas_offset, state.scale),
                            *mouse_button,
                        ));
                    }
                }

                Event::Mouse(mouse::Event::CursorMoved { position }) => {
                    let mouse_pos = *position;

                    if state.grabbing {
                        if let Some(pos) = state.canvas_grab {
                            state.canvas_offset = Vec2::new(mouse_pos.x, mouse_pos.y) - pos
                        } else {
                            let position = Vec2::new(mouse_pos.x, mouse_pos.y);
                            state.canvas_grab = Some(position - state.canvas_offset);
                        }
                    }

                    if let Some(on_move) = &self.on_move {
                        shell.publish(on_move(to_canvas_coords(
                            bounds,
                            mouse_pos,
                            state.canvas_offset,
                            state.scale,
                        )));
                    } else if state.grabbing {
                        shell.request_redraw();
                    }
                }

                Event::Mouse(mouse::Event::ButtonReleased(mouse_button)) => {
                    if let Some(on_release) = &self.on_release {
                        shell.publish(on_release(
                            to_canvas_coords(bounds, mouse_pos, state.canvas_offset, state.scale),
                            *mouse_button,
                        ));
                    }
                }

                Event::Mouse(mouse::Event::WheelScrolled { delta }) => match delta {
                    mouse::ScrollDelta::Lines { x, y } => {
                        // align the canvas to the mouse position when scaling.
                        // first we calculate what % the cursor is from the canvas on both axes.
                        // 0% = far left, or top
                        // 100% = far right, or bottom
                        //
                        // then after scaling, we adjust the offset of the canvas to match this.

                        // calculate the % the cursor is from the canvas.
                        let point =
                            to_canvas_coords(bounds, mouse_pos, state.canvas_offset, state.scale);

                        let x_percent = (point.x / canvas_bounds.width) * state.scale;
                        let y_percent = (point.y / canvas_bounds.height) * state.scale;

                        // TODO
                        // let y = if state.zoom < 1. {
                        //     if state.zoom + y < 1. { *y / 4.0 } else { *y }
                        // } else {
                        //     *y
                        // };

                        state.scale = (state.scale + y).clamp(MIN_SCALE, MAX_SCALE);

                        // recalculate the bounds of the canvas
                        let new_canvas_bounds = Rectangle {
                            x: x + bounds.x,
                            y: y + bounds.y,
                            width: self.buffer.width() as f32 * state.scale,
                            height: self.buffer.height() as f32 * state.scale,
                        };

                        // move the canvas offset to satisfy the percentages.
                        state.canvas_offset = Vec2::new(
                            (mouse_pos.x - new_canvas_bounds.width * x_percent) - bounds.x,
                            (mouse_pos.y - new_canvas_bounds.height * y_percent) - bounds.y,
                        );

                        shell.request_redraw();
                    }

                    mouse::ScrollDelta::Pixels { y, .. } => {
                        todo!()
                    }
                },
                Event::Window(window::Event::Resized(new_size)) => {
                    // TODO: center texture
                }
                _ => (),
            }
        } else {
            state.reset();
        };
    }
}

impl<'a, Message, Theme, Renderer, Handler> From<TextureCanvas<'a, Message, Theme, Handler>>
    for iced_core::Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: Catalog + 'a,
    Renderer: iced_wgpu::primitive::Renderer,
    Handler: SurfaceHandler,
{
    fn from(value: TextureCanvas<'a, Message, Theme, Handler>) -> Self {
        Element::new(value)
    }
}

/// TODO: move canvas offset and zoom to user state
#[derive(Clone)]
pub struct State {
    canvas_grab: Option<glam::Vec2>,
    grabbing: bool,
    canvas_offset: glam::Vec2,
    scale: f32,
    is_hovered: bool,
}

impl Default for State {
    fn default() -> Self {
        Self {
            canvas_grab: Default::default(),
            grabbing: Default::default(),
            canvas_offset: Default::default(),
            scale: 1.0,
            is_hovered: Default::default(),
        }
    }
}

impl State {
    pub fn reset(&mut self) {
        self.is_hovered = false;
        self.grabbing = false;
        self.canvas_grab = None;
    }
}

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
