use iced_core::{Color, Shadow, Theme};

pub enum Status {
    None,
    Hovered,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    pub background: Color,
    pub border_color: Color,
    pub border_thickness: f32,
    pub shadow: Shadow,
}

impl Default for Style {
    fn default() -> Self {
        Style {
            background: Color::TRANSPARENT,
            border_color: Color::BLACK,
            border_thickness: 1.0,
            shadow: Shadow::default(),
        }
    }
}

pub trait Catalog: Sized {
    /// The item class of the [`Catalog`].
    type Class<'a>;

    /// The default class produced by the [`Catalog`].
    fn default<'a>() -> Self::Class<'a>;

    /// The [`Style`] of a class with the given status.
    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style;
}

pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme, Status) -> Style + 'a>;

impl Catalog for iced_core::Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(primary)
    }

    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style {
        class(self, status)
    }
}

pub fn primary<Theme>(_theme: &Theme, _status: Status) -> Style {
    Style::default()
}
