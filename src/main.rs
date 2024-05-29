use iced::widget::{column, horizontal_rule};
use iced::widget::{self, slider};
use iced::{Color, Element, Length, Point, Size};
mod shader;

fn main() -> iced::Result {
    iced::program("title", ShaderApp::update, ShaderApp::view).run()
}

#[derive(Debug, Clone)]
enum Message {
    R(f32),
    G(f32),
    B(f32),
    X(f32),
    Y(f32),
}

struct ShaderApp {
    program: shader::Bitmap,
    color: Color,
    offset: Point<f32>,
}

impl Default for ShaderApp {
    fn default() -> Self {
        Self {
            program: shader::Bitmap::new(Size::new(500, 500)),
            color: Color::WHITE,
            offset: Point::ORIGIN,
        }
    }
}

impl ShaderApp {
    fn update(&mut self, msg: Message) {
        let old = self.color;
        match msg {
            Message::R(r) => self.color.r = r,
            Message::G(g) => self.color.g = g,
            Message::B(b) => self.color.b = b,
            Message::X(x) => self.program.controls.center.x = x,
            Message::Y(y) => self.program.controls.center.y = y,
        }

        if old != self.color {
            let Color {
                r,
                g,
                b,
                ..
            } = self.color;
            
            let color: u32 = 
                ((r * 255.0) as u32) << 0 |
                ((g * 255.0) as u32) << 8 |
                ((b * 255.0) as u32) << 16;


            self.program.buffer.write(|pixmap| {
                for b in pixmap.buffer {
                    *b = color
                }
            })
        }
    }

    fn view(&self) -> Element<Message> {
        column![
            widget::shader(&self.program)
                .width(Length::Fill)
                .height(Length::Fixed(512.0)),
            slider(0.0..=1.0, self.color.r, Message::R).step(0.01),
            slider(0.0..=1.0, self.color.g, Message::G).step(0.01),
            slider(0.0..=1.0, self.color.b, Message::B).step(0.01),
            horizontal_rule(1.0),
            slider(-100.0..=100.0, self.program.controls.center.x, Message::X),
            slider(-100.0..=100.0, self.program.controls.center.y, Message::Y),

        ]
        .into()
    }
}
