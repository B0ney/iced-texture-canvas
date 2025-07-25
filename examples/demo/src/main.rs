use iced::alignment::Horizontal;
use iced::widget::{button, column, container, horizontal_rule};
use iced::{Alignment, Border, Color, Element, Length, Point};

use iced_texture::{Controls, texture};

fn main() -> iced::Result {
    iced::application(ShaderApp::default, ShaderApp::update, ShaderApp::view).run()
}

#[derive(Debug, Clone)]
enum Message {
    Scale(f32),
    White,
    Black,
    PutPixel(Point),
}

const WHITE: u32 = 0xffffffff;
const BLACK: u32 = 0xff000000;

struct ShaderApp {
    pixmap: iced_texture::bitmap::Bitmap,
    controls: Controls,
    color: u32,
    size: u8,
    offset: Point<f32>,
}

impl Default for ShaderApp {
    fn default() -> Self {
        let mut bitmap = iced_texture::bitmap(256, 192);
        bitmap.update(include_bytes!("out.rgba").as_slice());

        Self {
            pixmap: bitmap,
            color: WHITE,
            offset: Point::ORIGIN,
            controls: Controls {
                scale: 1.0,
                center: Default::default(),
            },
            size: 10,
        }
    }
}

impl ShaderApp {
    fn update(&mut self, msg: Message) {
        match msg {
            Message::White => {
                self.color = WHITE;
            }

            Message::Black => {
                self.color = BLACK;
            }

            Message::Scale(_) => todo!(),

            Message::PutPixel(point) => {
                let width = self.pixmap.width() as usize;
                let height = self.pixmap.height() as usize;

                let buffer = self.pixmap.buffer_mut();

                let px = point.x.round() as usize;
                let py = point.y.round() as usize;

                for x in 0..10 {
                    for y in 0..10 {
                        let x = px + x;
                        let y = py + y;

                        if x >= width || y >= height {
                            continue;
                        }

                        buffer[y * width + x] = self.color;
                    }
                }
            }
        };
    }

    fn view(&self) -> Element<Message> {
        column![
            container(
                texture(&self.pixmap, &self.controls)
                    .width(512)
                    .height(Length::Fixed(512.0))
                    .on_release(Message::PutPixel)
                    .on_move(Message::PutPixel),
            )
            .style(|_| container::Style {
                text_color: None,
                background: None,
                border: Border {
                    color: Color::BLACK,
                    width: 2.0,
                    radius: 0.0.into()
                },
                shadow: Default::default(),
                snap: false
            }),
            horizontal_rule(1.0),
            button("Black Square").on_press(Message::Black),
            button("White Square").on_press(Message::White),
        ]
        .align_x(Horizontal::Center)
        .into()
    }
}
