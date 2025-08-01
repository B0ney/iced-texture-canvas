pub mod operation;
mod primitive;
pub mod style;
pub mod surface;

use primitive::Primitive;
use style::{Catalog, Status, Style, StyleFn};
use surface::{Surface, SurfaceHandler};

use iced_core::{
    Border, Element, Event, Layout, Length, Point, Rectangle, Shadow, Shell, Size, Widget, layout,
    mouse, renderer,
    widget::{self, Id},
    window,
};
use iced_widget::runtime::{
    Action,
    task::{self, Task},
};

const MIN_SCALE: f32 = 1.0; //0.05; // TODO
const MAX_SCALE: f32 = 64.0; // 6,400%

/// Create a new [`TextureCanvas`] with the given [`SurfaceHandler`].
///
/// You can use the provided [`Bitmap`](crate::Bitmap).
pub fn texture_canvas<'a, Message, Theme, Handler>(
    buffer: &'a Handler,
) -> TextureCanvas<'a, Message, Theme, Handler>
where
    Message: 'a,
    Theme: Catalog,
    Handler: SurfaceHandler,
{
    TextureCanvas::new(buffer)
}

/// A [`Task`] that centers the image in the [`TextureCanvas`] with the given [`Id`].
///
/// This requires that you also [`set the id`](TextureCanvas::id) of the [`TextureCanvas`].
pub fn center_image<Message>(id: impl Into<Id>) -> Task<Message> {
    task::effect(Action::widget(operation::center_image_raw(id.into())))
}

/// A [`Task`] that scales the image in the [`TextureCanvas`] with the given [`Id`].
///
/// This requires that you also [`set the id`](TextureCanvas::id) of the [`TextureCanvas`].
pub fn scale_image<Message>(id: impl Into<Id>, scale: f32) -> Task<Message> {
    task::effect(Action::widget(operation::scale_image_raw(id.into(), scale)))
}

pub struct TextureCanvas<'a, Message, Theme, Handler>
where
    Theme: Catalog,
{
    buffer: &'a Handler,
    width: Length,
    height: Length,

    class: Theme::Class<'a>,
    id: Option<Id>,
    default_zoom: f32,

    on_grab: Option<Box<dyn Fn() -> Message + 'a>>,
    on_zoom: Option<Box<dyn Fn(f32) -> Message + 'a>>,
    on_pressed: Option<Box<dyn Fn(Point, mouse::Button) -> Message + 'a>>,
    on_move: Option<Box<dyn Fn(Point) -> Message + 'a>>,
    on_release: Option<Box<dyn Fn(Point, mouse::Button) -> Message + 'a>>,
    on_enter: Option<Box<dyn Fn() -> Message + 'a>>,
    on_exit: Option<Box<dyn Fn() -> Message + 'a>>,

    interaction: Option<mouse::Interaction>,
}

impl<'a, Message, Theme, Handler> TextureCanvas<'a, Message, Theme, Handler>
where
    Theme: style::Catalog,
    Handler: SurfaceHandler,
{
    /// Create a new [`TextureCanvas`] with the given [`SurfaceHandler`].
    ///
    /// You can use the provided [`Bitmap`](crate::Bitmap).
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
            id: None,
            default_zoom: 1.0,
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

    /// Set the `style` of the image displayed by the [`TextureCanvas`].
    #[must_use]
    pub fn style(mut self, style: impl Fn(&Theme, Status) -> Style + 'a) -> Self
    where
        Theme::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        self.class = (Box::new(style) as StyleFn<'a, Theme>).into();
        self
    }

    /// Set the style `class` of the image displayed by the [`TextureCanvas`].
    #[must_use]
    pub fn class(mut self, class: impl Into<Theme::Class<'a>>) -> Self {
        self.class = class.into();
        self
    }

    /// Set the message to emit when the image is panned.
    pub fn on_drag(mut self, on_drag: impl Fn() -> Message + 'a) -> Self {
        self.on_grab = Some(Box::new(on_drag));
        self
    }

    /// Set the message to emit when the image has been zoomed in/out.
    pub fn on_zoom(mut self, on_zoom: impl Fn(f32) -> Message + 'a) -> Self {
        self.on_zoom = Some(Box::new(on_zoom));
        self
    }

    /// Set the message to emit when the [`TextureCanvas`] area is pressed.
    ///
    /// The [`Point`] it produces is relative to the position of the displayed image.
    pub fn on_press(mut self, on_press: impl Fn(Point, mouse::Button) -> Message + 'a) -> Self {
        self.on_pressed = Some(Box::new(on_press));
        self
    }

    /// Set the message to emit when the mouse moves over the [`TextureCanvas`].
    ///
    /// The [`Point`] it produces is relative to the position of the displayed image.
    pub fn on_move(mut self, on_move: impl Fn(Point) -> Message + 'a) -> Self {
        self.on_move = Some(Box::new(on_move));
        self
    }

    /// Set the message to emit when the mouse clicks on the [`TextureCanvas`].
    ///
    /// The [`Point`] it produces is relative to the position of the displayed image.
    pub fn on_release(mut self, on_release: impl Fn(Point, mouse::Button) -> Message + 'a) -> Self {
        self.on_release = Some(Box::new(on_release));
        self
    }

    /// Set the message to emit when the mouse hovers over the image in the [`TextureCanvas`].
    ///
    /// This is analogous to [`TextureCanvas::on_enter`], but using a closure to produce the message.
    ///
    /// This closure will only be called when the image in the [`TextureCanvas`] is actually entered and,
    /// therefore, this method is useful to reduce overhead if creating the resulting message is slow.
    pub fn on_enter_with(mut self, on_exit: impl Fn() -> Message + 'a) -> Self {
        self.on_exit = Some(Box::new(on_exit));
        self
    }

    /// Set the message to emit when the mouse hovers over the image in the [`TextureCanvas`].
    ///
    /// This requires that `Message` is [`Clone`].
    ///
    /// If you can't make your `Message` [`Clone`], use [`TextureCanvas::on_enter_with`] instead.
    pub fn on_enter(mut self, on_enter: impl Into<Option<Message>>) -> Self
    where
        Message: Clone + 'a,
    {
        if let Some(on_enter) = on_enter.into() {
            self.on_enter = Some(Box::new(move || on_enter.clone()))
        }

        self
    }

    /// Set the message to emit when the mouse leaves the image in the [`TextureCanvas`].
    ///
    /// This is analogous to [`TextureCanvas::on_exit`], but using a closure to produce the message.
    ///
    /// This closure will only be called when the mouse actually leaves the image in the [`TextureCanvas`] and,
    /// therefore, this method is useful to reduce overhead if creating the resulting message is slow.
    pub fn on_exit_with(mut self, on_exit: impl Fn() -> Message + 'a) -> Self {
        self.on_exit = Some(Box::new(on_exit));
        self
    }

    /// Set the message to emit when the mouse leaves the image in the [`TextureCanvas`].
    ///
    /// This requires that `Message` is [`Clone`].
    ///
    /// If you can't make your `Message` [`Clone`], use [`TextureCanvas::on_exit_with`] instead.
    pub fn on_exit(mut self, on_exit: impl Into<Option<Message>>) -> Self
    where
        Message: Clone + 'a,
    {
        if let Some(on_exit) = on_exit.into() {
            self.on_exit = Some(Box::new(move || on_exit.clone()))
        }
        self
    }

    /// Set the mouse icon when the mouse hovers over the image in the [`TextureCanvas`].
    pub fn mouse_interaction(mut self, mouse_interaction: mouse::Interaction) -> Self {
        self.interaction = Some(mouse_interaction);
        self
    }

    /// Set the [`Id`] of the [`TextureCanvas`].
    ///
    /// You will need to do this if you need to manually
    /// [`scale`](scale_image)/[`center`](center_image) the contained image.
    pub fn id(mut self, id: impl Into<Id>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Set the default scale of the image displayed in the [`TextureCanvas`]
    pub fn default_zoom(mut self, default_zoom: f32) -> Self {
        self.default_zoom = default_zoom;
        self
    }
}

impl<'a, Message, Theme, Renderer, Handler> Widget<Message, Theme, Renderer>
    for TextureCanvas<'a, Message, Theme, Handler>
where
    Renderer: iced_wgpu::primitive::Renderer,
    Theme: Catalog,
    Handler: SurfaceHandler,
{
    fn tag(&self) -> widget::tree::Tag {
        struct Tag<T>(T);
        widget::tree::Tag::of::<Tag<State>>()
    }

    fn state(&self) -> widget::tree::State {
        let default_zoom = self.default_zoom.clamp(MIN_SCALE, MAX_SCALE);
        widget::tree::State::new(State::new(default_zoom))
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

        let glam::Vec2 { x, y } = state.canvas_offset;
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
                    state.generation,
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

        if state.grabbing {
            mouse::Interaction::Grabbing
        } else if state.is_hovered {
            self.interaction.unwrap_or_default()
        } else {
            mouse::Interaction::None
        }
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

        let image_width = self.buffer.width() as f32;
        let image_height = self.buffer.height() as f32;

        if state.should_center {
            state.should_center = false;
            state.canvas_offset = glam::Vec2::new(
                bounds.center_x() - (image_width * state.scale) / 2.,
                bounds.center_y() - (image_height * state.scale) / 2.,
            );
        }

        // TODO: refactor
        if let Some(new_scale) = state.suggested_scale.take() {
            let center = bounds.center();

            let point = to_canvas_coords(bounds, center, state.canvas_offset, state.scale);

            let x_percent = point.x / image_width;
            let y_percent = point.y / image_height;

            state.scale = (new_scale).clamp(MIN_SCALE, MAX_SCALE);

            // recalculate the bounds of the canvas
            let new_canvas_width = image_width * state.scale;
            let new_canvas_height = image_height * state.scale;

            // move the canvas offset to satisfy the percentages.
            state.canvas_offset = glam::Vec2::new(
                (center.x - new_canvas_width * x_percent) - bounds.x,
                (center.y - new_canvas_height * y_percent) - bounds.y,
            );

            shell.request_redraw();
            if let Some(on_zoom) = &self.on_zoom {
                shell.publish(on_zoom(state.scale));
            };
        }

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
                width: image_width * state.scale,
                height: image_height * state.scale,
            };

            if !state.grabbing {
                let was_hovered = state.is_hovered;
                state.is_hovered = cursor.is_over(canvas_bounds);

                match (was_hovered, state.is_hovered) {
                    (false, true) => {
                        if let Some(on_enter) = &self.on_enter {
                            shell.publish(on_enter());
                        }
                    }

                    (true, false) => {
                        if let Some(on_exit) = &self.on_exit {
                            shell.publish(on_exit());
                        }
                    }
                    _ => (),
                }
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
                        let mouse_pos = glam::Vec2::new(mouse_pos.x, mouse_pos.y);

                        if let Some(pos) = state.canvas_grab {
                            state.canvas_offset = mouse_pos - pos
                        } else {
                            state.canvas_grab = Some(mouse_pos - state.canvas_offset);
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
                    mouse::ScrollDelta::Lines { y, .. } => {
                        if state.grabbing {
                            return;
                        }
                        // align the canvas to the mouse position when scaling.
                        // first we calculate what % the cursor is from the canvas on both axes.
                        // 0% = far left, or top
                        // 100% = far right, or bottom
                        //
                        // then after scaling, we adjust the offset of the canvas to match this.

                        // calculate the % the cursor is from the canvas.
                        let point =
                            to_canvas_coords(bounds, mouse_pos, state.canvas_offset, state.scale);

                        let x_percent = point.x / image_width;
                        let y_percent = point.y / image_height;

                        // TODO
                        // let y = if state.zoom < 1. {
                        //     if state.zoom + y < 1. { *y / 4.0 } else { *y }
                        // } else {
                        //     *y
                        // };

                        state.scale = (state.scale + y).clamp(MIN_SCALE, MAX_SCALE);

                        // recalculate the bounds of the canvas
                        let new_canvas_width = image_width * state.scale;
                        let new_canvas_height = image_height * state.scale;

                        // move the canvas offset to satisfy the percentages.
                        state.canvas_offset = glam::Vec2::new(
                            (mouse_pos.x - new_canvas_width * x_percent) - bounds.x,
                            (mouse_pos.y - new_canvas_height * y_percent) - bounds.y,
                        );

                        shell.request_redraw();

                        if let Some(on_zoom) = &self.on_zoom {
                            shell.publish(on_zoom(state.scale));
                        };
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

    fn operate(
        &self,
        tree: &mut widget::Tree,
        layout: Layout<'_>,
        _renderer: &Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        let state: &mut State = tree.state.downcast_mut();
        operation.custom(self.id.as_ref(), layout.bounds(), state);
    }
}

impl<'a, Message, Theme, Renderer, Handler> From<TextureCanvas<'a, Message, Theme, Handler>>
    for iced_core::Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: Catalog + 'a,
    Renderer: iced_wgpu::primitive::Renderer,
    Handler: SurfaceHandler,
{
    fn from(value: TextureCanvas<'a, Message, Theme, Handler>) -> Self {
        Element::new(value)
    }
}

fn to_canvas_coords(bounds: Rectangle, mouse: Point, offset: glam::Vec2, scale: f32) -> Point {
    let mouse = glam::vec2(mouse.x, mouse.y);
    let bounds_offset = glam::vec2(bounds.x, bounds.y) / scale;
    let glam::Vec2 { x, y } = (mouse - offset) / scale - bounds_offset;

    Point { x, y }
}

pub(crate) struct State {
    canvas_grab: Option<glam::Vec2>,
    grabbing: bool,
    canvas_offset: glam::Vec2,
    pub scale: f32,
    is_hovered: bool,
    /// Used to force the shader pipeline to update the texture
    /// if there's a mismatch.
    generation: u64,
    pub should_center: bool,
    pub suggested_scale: Option<f32>,
}

impl State {
    fn new(default_scale: f32) -> Self {
        Self {
            canvas_grab: Default::default(),
            grabbing: Default::default(),
            canvas_offset: Default::default(),
            scale: default_scale,
            is_hovered: Default::default(),
            generation: new_generation(),
            should_center: true,
            suggested_scale: None,
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

fn new_generation() -> u64 {
    use std::sync::atomic::AtomicU64;

    static GENERATION: AtomicU64 = AtomicU64::new(0);

    GENERATION.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}
