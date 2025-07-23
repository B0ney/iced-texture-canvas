use iced::widget::{button, column, horizontal_rule, stack};
use iced::{Color, Element, Length, Point};

use iced_texture::{Controls, Surface, texture};

fn main() -> iced::Result {
    iced::application(ShaderApp::default, ShaderApp::update, ShaderApp::view).run()
}

#[derive(Debug, Clone)]
enum Message {
    Scale(f32),
    PutPixel,
    PutPixelWhite,
}

struct ShaderApp {
    pixmap: Surface,
    controls: Controls,
    color: Color,
    offset: Point<f32>,
}

impl Default for ShaderApp {
    fn default() -> Self {
        let mut bitmap = Surface::new(256, 192);
        bitmap.update(include_bytes!("out.rgba").as_slice());

        Self {
            pixmap: bitmap,
            color: Color::WHITE,
            offset: Point::ORIGIN,
            controls: Controls {
                scale: 1.0,
                center: Default::default(),
            },
        }
    }
}

impl ShaderApp {
    fn update(&mut self, msg: Message) {
        match msg {
            Message::PutPixelWhite => {
                let width = self.pixmap.width() as usize;
                let buffer = self.pixmap.buffer_mut();
                for x in 0..10 {
                    for y in 0..10 {
                        buffer[y * width + x] = 0xffffff;
                    }
                }
            }
            Message::PutPixel => {
                let width = self.pixmap.width() as usize;
                let buffer = self.pixmap.buffer_mut();
                for x in 0..10 {
                    for y in 0..10 {
                        buffer[y * width + x] = 0;
                    }
                }
            }
            _ => (),
        };
    }

    fn view(&self) -> Element<Message> {
        column![
            texture(&self.pixmap, &self.controls)
                .width(Length::Fill)
                .height(Length::Fixed(512.0)),
            // slider(0.0..=1.0, self.color.r, Message::R).step(0.01),
            // slider(0.0..=1.0, self.color.g, Message::G).step(0.01),
            // slider(0.0..=1.0, self.color.b, Message::B).step(0.01),
            horizontal_rule(1.0),
            button("Black Square").on_press(Message::PutPixel),
            button("White Square").on_press(Message::PutPixelWhite),
        ]
        .into()
    }
}
