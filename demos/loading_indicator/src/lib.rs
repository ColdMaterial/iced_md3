use common::section_header;
use iced::{
    Alignment, Element,
    widget::{column, row},
};
use iced_m3::theme::Theme;

#[derive(Clone)]
pub enum Message {
    Noop,
}

#[derive(Default)]
pub struct Page;

impl Page {
    pub fn update(&mut self, _message: Message) {}

    pub fn subscription(&self) -> iced::Subscription<Message> {
        iced::Subscription::none()
    }

    pub fn view(&self, theme: &Theme) -> Element<'_, Message> {
        column![
            section_header("Circular loading indicators", theme),
            row![
                iced_m3::widget::loading_indicator(
                    iced_m3::widget::loading_indicator::Style::contained(theme)
                ),
                iced_m3::widget::loading_indicator(
                    iced_m3::widget::loading_indicator::Style::uncontained(theme)
                ),
            ]
            .spacing(16)
            .align_y(Alignment::Center),
            section_header("Circular spinner", theme),
            row![
                iced_m3::widget::circular_loading_indicator(
                    iced_m3::widget::loading_indicator::Style::contained(theme)
                ),
                iced_m3::widget::circular_loading_indicator(
                    iced_m3::widget::loading_indicator::Style::uncontained(theme)
                ),
                iced_m3::widget::circular_loading_indicator(
                    iced_m3::widget::loading_indicator::Style::uncontained(theme)
                )
                .size(24.0),
            ]
            .spacing(16)
            .align_y(Alignment::Center),
        ]
        .spacing(24)
        .into()
    }
}

common::demo!(Page, Message, "Loading Indicator Demo");
