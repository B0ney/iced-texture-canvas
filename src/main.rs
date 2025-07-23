use iced::widget::{self, slider};
use iced::widget::{column, horizontal_rule};
use iced::{Color, Element, Length, Point, Size};
use shader::TextureCanvas;
mod shader;

fn main() -> iced::Result {
    iced::application(ShaderApp::default, ShaderApp::update, ShaderApp::view).run()
}

#[derive(Debug, Clone)]
enum Message {
    R(f32),
    G(f32),
    B(f32),
    X(f32),
    Y(f32),
    Scale(f32),
}

struct ShaderApp {
    pixmap: shader::texture::Pixmap,
    controls: shader::Controls,
    color: Color,
    offset: Point<f32>,
}

impl Default for ShaderApp {
    fn default() -> Self {
        let bitmap = shader::texture::Pixmap::new(256, 192);
        bitmap.write(|pixmap| {
            bytemuck::cast_slice_mut(pixmap.buffer)
                .clone_from_slice(include_bytes!("out.rgba").as_slice());
        });

        Self {
            pixmap: bitmap,
            color: Color::WHITE,
            offset: Point::ORIGIN,
            controls: shader::Controls {
                zoom: 1.0,
                center: Default::default(),
            },
        }
    }
}

impl ShaderApp {
    fn update(&mut self, msg: Message) {
        // let old = self.color;
        // match msg {
        //     Message::R(r) => self.color.r = r,
        //     Message::G(g) => self.color.g = g,
        //     Message::B(b) => self.color.b = b,
        //     Message::X(x) => self.program.controls.center.x = x,
        //     Message::Y(y) => self.program.controls.center.y = y,
        //     Message::Scale(scale) => self.program.controls.zoom = scale,
        // }

        // if old != self.color {
        //     let Color { r, g, b, .. } = self.color;

        //     let color: u32 =
        //         ((r * 255.0) as u32) << 0 | ((g * 255.0) as u32) << 8 | ((b * 255.0) as u32) << 16;

        //     self.program.buffer.write(|pixmap| {
        //         for b in pixmap.buffer {
        //             *b = color
        //         }
        //     })
        // }
    }

    fn view(&self) -> Element<Message> {
        column![
            widget::shader(TextureCanvas::new(&self.pixmap, &self.controls))
                .width(Length::Fill)
                .height(Length::Fixed(512.0)),
            // slider(0.0..=1.0, self.color.r, Message::R).step(0.01),
            // slider(0.0..=1.0, self.color.g, Message::G).step(0.01),
            // slider(0.0..=1.0, self.color.b, Message::B).step(0.01),
            horizontal_rule(1.0),
            // slider(-100.0..=1000.0, self.program.controls.center.x, Message::X),
            // slider(-100.0..=1000.0, self.program.controls.center.y, Message::Y),
            // slider(1.0..=5.0, self.program.controls.zoom, Message::Scale).step(1.),
        ]
        .into()
    }
}
