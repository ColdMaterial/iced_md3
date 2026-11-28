use iced::{
    Element, Length, Task,
    widget::{container, text},
};
use iced_m3::theme::{ColorScheme, Mode, Theme};

const APP_NAME: &str = "Demo Template";

#[derive(Clone)]
enum Message {}

struct State {
    theme: Theme,
}

impl Default for State {
    fn default() -> Self {
        Self {
            theme: Theme::default(Mode::Dark),
        }
    }
}

impl State {
    fn view(&self) -> Element<'_, Message> {
        container(
            text(APP_NAME)
                .font(fonts::text_bold())
                .size(24.0)
                .color(self.theme.on_surface()),
        )
        .padding(10.0)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_| iced::widget::container::Style::default().background(self.theme.background()))
        .into()
    }

    fn update(&mut self, _message: Message) -> Task<Message> {
        Task::none()
    }
}

fn main() -> iced::Result {
    common::run_custom(APP_NAME, State::default, State::update, State::view)
}
