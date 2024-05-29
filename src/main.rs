use iced::widget;
use iced::{Element, Length, Size};
mod shader;

fn main() -> iced::Result {
    iced::program("title", ShaderApp::update, ShaderApp::view).run()
}

#[derive(Debug)]
enum Message {}
struct ShaderApp {
    program: shader::Bitmap,
}

impl Default for ShaderApp {
    fn default() -> Self {
        Self {
            program: shader::Bitmap::new(Size::new(500, 500)),
        }
    }
}

impl ShaderApp {
    fn update(&mut self, msg: Message) {}

    fn view(&self) -> Element<Message> {
        widget::shader(&self.program)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
