use iced::widget::{self, button, slider, stack};
use iced::widget::{column, horizontal_rule};
use iced::{Color, Element, Length, Point, Size};

use iced_texture::{Controls, Surface, handle, texture};

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
        // let old = self.color;
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
            stack![
                button("hi").width(Length::Shrink),
                texture(&self.pixmap, &self.controls)
                    .width(Length::Fill)
                    .height(Length::Fixed(512.0)),
            ]
            .width(Length::Fill)
            .height(Length::Fixed(512.)),
            // slider(0.0..=1.0, self.color.r, Message::R).step(0.01),
            // slider(0.0..=1.0, self.color.g, Message::G).step(0.01),
            // slider(0.0..=1.0, self.color.b, Message::B).step(0.01),
            horizontal_rule(1.0),
            // slider(-100.0..=1000.0, self.program.controls.center.x, Message::X),
            // slider(-100.0..=1000.0, self.program.controls.center.y, Message::Y),
            // slider(1.0..=5.0, self.program.controls.zoom, Message::Scale).step(1.),
            button("a").on_press(Message::PutPixel),
            button("a").on_press(Message::PutPixelWhite),
        ]
        .into()
    }
}
