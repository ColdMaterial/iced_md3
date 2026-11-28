use common::section_header;
use iced::{
    Alignment, Element,
    widget::{column, row},
};
use iced_m3::theme::{Accent, Theme};
use iced_m3::widget::{button, hybrid_icon::Icon, tooltip};

#[derive(Clone)]
pub enum Message {
    ToggleSelected,
    Noop,
}

#[derive(Default)]
pub struct Page {
    button_selected: bool,
}

impl Page {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::ToggleSelected => self.button_selected = !self.button_selected,
            Message::Noop => {}
        }
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        iced::Subscription::none()
    }

    pub fn view(&self, theme: &Theme) -> Element<'_, Message> {
        let add_icon = Icon::Text {
            text: "add".into(),
            font: Some(fonts::icons_filled()),
        };
        let btn = |style: button::Style, label: &'static str| {
            iced_m3::widget::button(style, button::Content::Label(label.into()))
                .on_press(Message::Noop)
        };
        let icon_btn = |style: button::Style, icon: Icon<'static>| {
            iced_m3::widget::button(style, button::Content::Icon(icon)).on_press(Message::Noop)
        };
        let full_btn = |style: button::Style, label: &'static str, icon: Icon<'static>| {
            iced_m3::widget::button(
                style,
                button::Content::Full {
                    icon,
                    label: label.into(),
                },
            )
            .on_press(Message::Noop)
        };

        column![
            section_header("Filled buttons", theme),
            row![
                btn(button::Style::filled(theme, Accent::Primary), "Primary"),
                btn(button::Style::filled(theme, Accent::Secondary), "Secondary"),
                btn(button::Style::filled(theme, Accent::Tertiary), "Tertiary"),
            ]
            .spacing(8),
            section_header("Elevated buttons", theme),
            row![
                btn(button::Style::elevated(theme, Accent::Primary), "Primary"),
                btn(
                    button::Style::elevated(theme, Accent::Secondary),
                    "Secondary"
                ),
                btn(button::Style::elevated(theme, Accent::Tertiary), "Tertiary"),
            ]
            .spacing(8),
            section_header("Outlined buttons", theme),
            row![btn(button::Style::outlined(theme), "Outlined"),].spacing(8),
            section_header("Text buttons", theme),
            row![
                btn(button::Style::text(theme, Accent::Primary), "Primary"),
                btn(button::Style::text(theme, Accent::Secondary), "Secondary"),
                btn(button::Style::text(theme, Accent::Tertiary), "Tertiary"),
            ]
            .spacing(8),
            section_header("Disabled buttons", theme),
            row![
                iced_m3::widget::button(
                    button::Style::filled(theme, Accent::Primary),
                    button::Content::Label("Filled".into())
                ),
                iced_m3::widget::button(
                    button::Style::elevated(theme, Accent::Primary),
                    button::Content::Label("Elevated".into())
                ),
                iced_m3::widget::button(
                    button::Style::outlined(theme),
                    button::Content::Label("Outlined".into())
                ),
                iced_m3::widget::button(
                    button::Style::text(theme, Accent::Primary),
                    button::Content::Label("Text".into())
                ),
            ]
            .spacing(8),
            section_header("Icon buttons", theme),
            row![
                iced_m3::widget::tooltip(
                    tooltip::Style::new(theme),
                    icon_btn(
                        button::Style::filled(theme, Accent::Primary),
                        add_icon.clone()
                    ),
                )
                .label("Add"),
                icon_btn(
                    button::Style::elevated(theme, Accent::Primary),
                    add_icon.clone()
                ),
                icon_btn(button::Style::outlined(theme), add_icon.clone()),
                icon_btn(
                    button::Style::text(theme, Accent::Primary),
                    add_icon.clone()
                ),
            ]
            .spacing(8),
            section_header("Full buttons (icon + label)", theme),
            row![
                full_btn(
                    button::Style::filled(theme, Accent::Primary),
                    "Add",
                    add_icon.clone()
                ),
                full_btn(
                    button::Style::elevated(theme, Accent::Primary),
                    "Add",
                    add_icon.clone()
                ),
                full_btn(button::Style::outlined(theme), "Add", add_icon.clone()),
                full_btn(button::Style::text(theme, Accent::Primary), "Add", add_icon),
            ]
            .spacing(8),
            section_header("Button sizes", theme),
            row![
                btn(button::Style::filled(theme, Accent::Primary), "Extra Small")
                    .size(button::Size::extra_small()),
                btn(button::Style::filled(theme, Accent::Primary), "Small")
                    .size(button::Size::small()),
                btn(button::Style::filled(theme, Accent::Primary), "Medium")
                    .size(button::Size::medium()),
                btn(button::Style::filled(theme, Accent::Primary), "Large")
                    .size(button::Size::large()),
                btn(button::Style::filled(theme, Accent::Primary), "Extra Large")
                    .size(button::Size::extra_large()),
            ]
            .spacing(8),
            section_header("Selected button", theme),
            row![
                iced_m3::widget::button(
                    if self.button_selected {
                        button::Style::filled(theme, Accent::Primary)
                    } else {
                        button::Style::outlined(theme)
                    },
                    button::Content::Label("Toggle selection".into()),
                )
                .on_press(Message::ToggleSelected)
                .selected(self.button_selected),
            ]
            .spacing(8),
            section_header("Buttons with corner styles", theme),
            row![
                iced_m3::widget::button(
                    button::Style::filled(theme, Accent::Primary),
                    button::Content::Label("Rounded".into()),
                )
                .on_press(Message::Noop),
                iced_m3::widget::button(
                    button::Style::filled(theme, Accent::Primary),
                    button::Content::Label("Square".into()),
                )
                .on_press(Message::Noop)
                .size(button::Size {
                    corner_radius: button::CornerRadius::default()
                        .style(button::CornerStyle::Square),
                    ..button::Size::default()
                })
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        ]
        .spacing(24)
        .into()
    }
}

common::demo!(Page, Message, "Buttons Demo");
