use iced::widget::{column, horizontal_rule};
use iced::widget::{self, slider};
use iced::{Color, Element, Length, Size};
mod shader;

fn main() -> iced::Result {
    iced::program("title", ShaderApp::update, ShaderApp::view).run()
}

#[derive(Debug, Clone)]
enum Message {
    R(f32),
    G(f32),
    B(f32),

}
struct ShaderApp {
    program: shader::Bitmap,
    color: Color,
}

impl Default for ShaderApp {
    fn default() -> Self {
        Self {
            program: shader::Bitmap::new(Size::new(500, 500)),
            color: Color::WHITE,
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
                .width(Length::Fixed(512.0))
                .height(Length::Fixed(512.0)),
            slider(0.0..=1.0, self.color.r, Message::R).step(0.01),
            slider(0.0..=1.0, self.color.g, Message::G).step(0.01),
            slider(0.0..=1.0, self.color.b, Message::B).step(0.01),
            horizontal_rule(1.0)
        ]
        .into()
    }
}
