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
    Blit,
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
        let decoder = image::codecs::png::PngDecoder::new(std::io::Cursor::new(include_bytes!(
            "happy-tree.png"
        )))
        .unwrap();

        let a = image::DynamicImage::from_decoder(decoder)
            .unwrap()
            .to_rgba8();

        let mut bitmap = iced_texture::bitmap(a.width(), a.height());

        let a = a.into_raw();

        bitmap.update(&a);

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
            Message::Blit => {}
        };
    }

    fn view(&self) -> Element<Message> {
        column![
            container(
                texture(&self.pixmap)
                    .mouse_interaction(mouse::Interaction::Crosshair)
                    .on_enter(Message::Entered)
                    .on_exit(Message::Exited)
                    .on_move(Message::Move)
                    .on_press(Message::StartDraw)
                    .on_release(Message::EndDraw)
            )
            .style(|_| container::Style {
                text_color: None,
                background: Some(Color::from_rgba8(127, 127, 127, 1.).into()),
                ..Default::default()
            }),
            horizontal_rule(1.0),
            button("Black Square").on_press(Message::Black),
            button("White Square").on_press(Message::White),
            button("Blit").on_press(Message::Blit),
        ]
        .align_x(Horizontal::Center)
        .into()
    }
}
