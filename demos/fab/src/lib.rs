use common::section_header;
use iced::{
    Alignment, Element,
    widget::{column, row, text},
};
use iced_m3::theme::{Accent, ColorScheme, Theme};
use iced_m3::widget::{Action, button, fab, fab_menu, hybrid_icon::Icon, tooltip};

#[derive(Clone)]
pub enum Message {
    ToggleCornerStyle,
    Noop,
}

#[derive(Default)]
pub struct Page {
    corner_style: button::CornerStyle,
}

impl Page {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::ToggleCornerStyle => {
                self.corner_style = match self.corner_style {
                    button::CornerStyle::Rounded => button::CornerStyle::Square,
                    button::CornerStyle::Square => button::CornerStyle::Rounded,
                };
            }
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
        let edit_icon = Icon::Text {
            text: "edit".into(),
            font: Some(fonts::icons_filled()),
        };

        column![
            section_header("FAB sizes", theme),
            row![
                fab(
                    fab::Style::fab_tonal(theme, Accent::Secondary),
                    fab::Content::Reguar {
                        icon: add_icon.clone()
                    },
                    Message::Noop,
                )
                .size(fab::Size::Regular)
                .tooltip(
                    "Add",
                    fab::Tooltip {
                        style: tooltip::Style::new(theme),
                        position: tooltip::Position::Right,
                    },
                ),
                fab(
                    fab::Style::fab_tonal(theme, Accent::Secondary),
                    fab::Content::Reguar {
                        icon: add_icon.clone()
                    },
                    Message::Noop,
                )
                .size(fab::Size::Medium),
                fab(
                    fab::Style::fab_tonal(theme, Accent::Secondary),
                    fab::Content::Reguar {
                        icon: add_icon.clone()
                    },
                    Message::Noop,
                )
                .size(fab::Size::Large),
            ]
            .spacing(16)
            .align_y(Alignment::Center),
            section_header("FAB styles", theme),
            row![
                fab(
                    fab::Style::fab_vibrant(theme, Accent::Primary),
                    fab::Content::Reguar {
                        icon: add_icon.clone()
                    },
                    Message::Noop,
                ),
                fab(
                    fab::Style::fab_tonal(theme, Accent::Secondary),
                    fab::Content::Reguar {
                        icon: add_icon.clone()
                    },
                    Message::Noop,
                ),
                fab(
                    fab::Style::fab_vibrant(theme, Accent::Tertiary),
                    fab::Content::Reguar {
                        icon: add_icon.clone()
                    },
                    Message::Noop,
                ),
            ]
            .spacing(16)
            .align_y(Alignment::Center),
            section_header("Extended FAB", theme),
            row![
                fab(
                    fab::Style::fab_vibrant(theme, Accent::Primary),
                    fab::Content::Extended {
                        icon: add_icon.clone(),
                        label: "Create".into(),
                    },
                    Message::Noop,
                ),
                fab(
                    fab::Style::fab_tonal(theme, Accent::Secondary),
                    fab::Content::Extended {
                        icon: edit_icon,
                        label: "Edit".into(),
                    },
                    Message::Noop,
                ),
            ]
            .spacing(16)
            .align_y(Alignment::Center),
            section_header("FAB Menu", theme),
            fab_menu(
                fab_menu::Style::new(theme, Accent::default()),
                vec![
                    fab_menu::Item {
                        icon: Icon::Text {
                            text: "email".into(),
                            font: Some(fonts::icons_filled()),
                        },
                        label: "Email".into(),
                        on_press: Action::Direct(Message::Noop),
                    },
                    fab_menu::Item {
                        icon: Icon::Text {
                            text: "chat".into(),
                            font: Some(fonts::icons_filled()),
                        },
                        label: "Chat".into(),
                        on_press: Action::Direct(Message::Noop),
                    },
                    fab_menu::Item {
                        icon: Icon::Text {
                            text: "call".into(),
                            font: Some(fonts::icons_filled()),
                        },
                        label: "Call".into(),
                        on_press: Action::Direct(Message::Noop),
                    },
                ],
                None,
            ),
            section_header("FAB corner styles", theme),
            row![
                fab(
                    fab::Style::fab_vibrant(theme, Accent::Primary),
                    fab::Content::Reguar {
                        icon: add_icon.clone()
                    },
                    Message::Noop,
                )
                .corner_style(self.corner_style),
                fab(
                    fab::Style::fab_tonal(theme, Accent::Secondary),
                    fab::Content::Reguar {
                        icon: add_icon.clone()
                    },
                    Message::Noop,
                )
                .corner_style(self.corner_style),
                iced_m3::widget::button(
                    button::Style::elevated(theme, Accent::default()),
                    button::Content::Label("Toggle".into()),
                )
                .on_press(Message::ToggleCornerStyle),
                text(format!("{:?}", self.corner_style)).color(theme.on_surface_variant()),
            ]
            .spacing(16)
            .align_y(Alignment::Center),
        ]
        .spacing(24)
        .into()
    }
}

common::demo!(Page, Message, "Fab Demo");
