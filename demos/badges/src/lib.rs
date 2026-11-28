use common::section_header;
use iced::{
    Alignment, Element,
    widget::{Space, center, column, row, text},
};
use iced_m3::theme::{ColorScheme, Theme};

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
        use iced_m3::widget::badge;

        let style = badge::Style::new(theme);

        let mail_icon = text("\u{e158}")
            .font(fonts::icons_filled())
            .size(24.0)
            .color(theme.on_surface());

        let notif_icon = text("\u{e7f4}")
            .font(fonts::icons_filled())
            .size(24.0)
            .color(theme.on_surface());

        column![
            section_header("Badges", theme),
            row![
                badge(style, Space::new().width(6.0).height(6.0)),
                badge(style, Space::new().width(12.0).height(16.0)).label("1"),
                badge(style, Space::new().width(12.0).height(16.0)).label("99"),
                badge(style, Space::new().width(12.0).height(16.0)).label("999+"),
            ]
            .spacing(16)
            .align_y(Alignment::Center),
            section_header("Badges on icons", theme),
            row![
                center(
                    badge(style, mail_icon)
                        .label("3")
                        .bounds_mode(badge::BoundsMode::Symmetrical),
                )
                .width(48)
                .height(48),
                center(
                    badge(style, notif_icon)
                        .label("99+")
                        .bounds_mode(badge::BoundsMode::Symmetrical),
                )
                .width(48)
                .height(48),
            ]
            .spacing(16),
        ]
        .spacing(24)
        .into()
    }
}

common::demo!(Page, Message, "Badges Demo");
