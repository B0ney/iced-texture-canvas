use iced::alignment::Horizontal;
use iced::widget::{button, column, container, horizontal_rule};
use iced::{Alignment, Border, Color, Element, Length, Point, mouse};

use iced_texture::{Controls, texture};

fn main() -> iced::Result {
    iced::application(ShaderApp::default, ShaderApp::update, ShaderApp::view).run()
}

#[derive(Debug, Clone)]
enum Message {
    Scale(f32),
    White,
    Black,
    Entered,
    Exited,
    Move(Point),
    StartDraw(Point, mouse::Button),
    EndDraw(Point, mouse::Button),
}

const WHITE: u32 = 0xffffffff;
const BLACK: u32 = 0xff000000;

struct ShaderApp {
    pixmap: iced_texture::bitmap::Bitmap,
    controls: Controls,
    color: u32,
    size: u8,
    offset: Point<f32>,
    on_canvas: bool,
    drawing: bool,
}

impl Default for ShaderApp {
    fn default() -> Self {
        let mut bitmap = iced_texture::bitmap(256, 192);
        // bitmap.update(include_bytes!("out.rgba").as_slice());
        bitmap.buffer_mut().fill(BLACK);
        bitmap.update(include_bytes!("out.rgba").as_slice());

        // bitmap.buffer_mut().fill(0xffffffff);

        Self {
            pixmap: bitmap,
            color: WHITE,
            offset: Point::ORIGIN,
            controls: Controls {
                scale: 1.0,
                center: Default::default(),
            },
            size: 10,
            on_canvas: false,
            drawing: false,
        }
    }
}

impl ShaderApp {
    fn put_pixel(&mut self, point: Point) {
        let width = self.pixmap.width() as usize;
        let height = self.pixmap.height() as usize;

        let buffer = self.pixmap.buffer_mut();

        let px = point.x.floor() as usize;
        let py = point.y.floor() as usize;

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

    fn update(&mut self, msg: Message) {
        match msg {
            Message::White => self.color = WHITE,
            Message::Black => self.color = BLACK,
            Message::Scale(_) => todo!(),
            Message::Move(point) => {
                if self.on_canvas && self.drawing {
                    self.put_pixel(point)
                }
            }
            Message::Entered => self.on_canvas = true,
            Message::Exited => self.on_canvas = false,
            Message::StartDraw(_, button) => {
                if button == mouse::Button::Left {
                    self.drawing = true
                }
            }
            Message::EndDraw(last_point, button) => {
                let was_drawing = self.drawing;

                if button == mouse::Button::Left {
                    self.drawing = false
                }

                if was_drawing {
                    self.put_pixel(last_point);
                }
            }
        };
    }

    fn view(&self) -> Element<Message> {
        column![
            container(
                texture(&self.pixmap)
                    // .width(512)
                    // .height(Length::Fixed(512.0))
                    .mouse_interaction(mouse::Interaction::Crosshair)
                    .on_enter(Message::Entered)
                    .on_exit(Message::Exited)
                    .on_move(Message::Move)
                    .on_press(Message::StartDraw)
                    .on_release(Message::EndDraw)
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
