use iced::alignment::Horizontal;
use iced::widget::{button, column, container, horizontal_rule};
use iced::{Color, Element, Point, mouse};

use iced_texture::{Bitmap, texture};

fn main() -> iced::Result {
    iced::application(BasicPaint::default, BasicPaint::update, BasicPaint::view)
        .title(BasicPaint::title)
        .run()
}

#[derive(Debug, Clone)]
enum Message {
    White,
    Black,
    StartDraw(Point, mouse::Button),
    Move(Point),
    EndDraw(Point, mouse::Button),
}

const WHITE: u32 = 0xffffffff;
const BLACK: u32 = 0xff000000;

struct BasicPaint {
    bitmap: Bitmap,
    color: u32,
    size: u8,
    drawing: bool,
    pending: Pending,
}

impl Default for BasicPaint {
    fn default() -> Self {
        Self {
            bitmap: load_image(),
            color: WHITE,
            size: 5,
            drawing: false,
            pending: Pending::None,
        }
    }
}

impl BasicPaint {
    fn title(&self) -> String {
        "Basic Paint App".into()
    }

    fn update(&mut self, msg: Message) {
        match msg {
            Message::White => self.color = WHITE,
            Message::Black => self.color = BLACK,
            Message::StartDraw(point, button) => {
                self.pending.update(point);

                if button == mouse::Button::Left {
                    self.drawing = true
                }
            }
            Message::Move(point) => {
                self.pending.update(point);

                if self.drawing {
                    if let Pending::Line(p1, p2) = self.pending {
                        draw_line(
                            &mut self.bitmap,
                            p1.x,
                            p1.y,
                            p2.x,
                            p2.y,
                            |buffer, px, py| {
                                put_pixel(buffer, px, py, self.color, self.size);
                            },
                        );
                    }
                }
            }
            Message::EndDraw(last_point, button) => {
                let was_drawing = self.drawing;

                if button == mouse::Button::Left {
                    self.drawing = false
                }

                if was_drawing {
                    put_pixel(
                        &mut self.bitmap,
                        last_point.x.floor() as i32,
                        last_point.y.floor() as i32,
                        self.color,
                        self.size,
                    );
                }
            }
        };
    }

    fn view(&self) -> Element<Message> {
        column![
            container(
                texture(&self.bitmap)
                    .mouse_interaction(mouse::Interaction::Crosshair)
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
            button("Black").on_press(Message::Black),
            button("White").on_press(Message::White),
        ]
        .align_x(Horizontal::Center)
        .into()
    }
}

fn load_image() -> Bitmap {
    use image::codecs::png::PngDecoder;
    use std::io::Cursor;

    let png_data = Cursor::new(include_bytes!("happy-tree.png").as_slice());
    let png_decoder = PngDecoder::new(png_data).expect("creating png decoder");

    let image = image::DynamicImage::from_decoder(png_decoder).expect("valid png image");

    // create a new bitmap
    let mut bitmap = iced_texture::bitmap(image.width(), image.height());

    // update bitmap with the image data
    let rgba = image.to_rgba8().into_raw();

    bitmap.update(&rgba);

    bitmap
}

enum Pending {
    None,
    One(Point<i32>),
    Line(Point<i32>, Point<i32>),
}

impl Pending {
    pub fn update(&mut self, point: Point) {
        let point = Point::new(point.x.floor() as i32, point.y.floor() as i32);

        *self = match self {
            Pending::None => Pending::One(point),
            Pending::One(p1) => Pending::Line(*p1, point),
            Pending::Line(_, p2) => Pending::Line(*p2, point),
        }
    }
}

fn put_pixel(buffer: &mut Bitmap, px: i32, py: i32, color: u32, size: u8) {
    let width = buffer.width() as usize;
    let height = buffer.height() as usize;

    let px = px - (size as i32 / 2);
    let py = py - (size as i32 / 2);
    let size = size as i32;

    for x in 0..size {
        for y in 0..size {
            let x = (px + x) as usize;
            let y = (py + y) as usize;

            if x >= width || y >= height {
                continue;
            }

            buffer.buffer_mut()[y * width + x] = color;
        }
    }
}

// https://en.wikipedia.org/wiki/Bresenham%27s_line_algorithm
pub fn draw_line<F, C>(canvas: &mut C, mut x0: i32, mut y0: i32, x1: i32, y1: i32, mut draw: F)
where
    F: FnMut(&mut C, i32, i32),
{
    let dx = (x1 - x0).abs();
    let dy = (y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };

    let mut error: i32 = if dx > dy { dx / 2 } else { -dy / 2 };

    loop {
        draw(canvas, x0, y0);

        if x0 == x1 && y0 == y1 {
            break;
        }
        let error_copy = error;

        if error_copy > -dx {
            error -= dy;
            x0 += sx;
        }

        if error_copy < dy {
            error += dx;
            y0 += sy;
        }
    }
}
