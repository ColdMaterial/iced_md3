use common::section_header;
use iced::{
    Element,
    widget::{column, stack, text},
};
use iced_m3::theme::{Accent, ColorScheme, Theme};
use iced_m3::widget::{button, dialog};

mod modal;

#[derive(Clone)]
pub enum Message {
    ToggleDialog,
    CloseDialog,
    ToggleCustom,
    CloseCustom,
    ToggleFullscreen,
    CloseFullscreen,
}

#[derive(Default)]
pub struct Page {
    dialog_open: bool,
    dialog_custom_open: bool,
    dialog_fullscreen_open: bool,
}

impl Page {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::ToggleDialog => self.dialog_open = !self.dialog_open,
            Message::CloseDialog => self.dialog_open = false,
            Message::ToggleCustom => self.dialog_custom_open = !self.dialog_custom_open,
            Message::CloseCustom => self.dialog_custom_open = false,
            Message::ToggleFullscreen => self.dialog_fullscreen_open = !self.dialog_fullscreen_open,
            Message::CloseFullscreen => self.dialog_fullscreen_open = false,
        }
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        iced::Subscription::none()
    }

    pub fn view(&self, theme: &Theme) -> Element<'_, Message> {
        let content = column![
            section_header("Simple dialog", theme),
            iced_m3::widget::button(
                button::Style::filled(theme, Accent::Primary),
                button::Content::Label("Open dialog".into()),
            )
            .on_press(Message::ToggleDialog),
            section_header("Custom content dialog", theme),
            iced_m3::widget::button(
                button::Style::filled(theme, Accent::Primary),
                button::Content::Label("Open custom dialog".into()),
            )
            .on_press(Message::ToggleCustom),
            section_header("Fullscreen dialog", theme),
            iced_m3::widget::button(
                button::Style::filled(theme, Accent::Primary),
                button::Content::Label("Open fullscreen dialog".into()),
            )
            .on_press(Message::ToggleFullscreen),
        ]
        .spacing(24);

        let dialog_element = modal::Modal::new(
            dialog::Dialog::new(
                dialog::Style::new(theme),
                text("This is a simple dialog with standard actions.").color(theme.on_surface()),
                vec![
                    dialog::Button {
                        on_press: Some(Message::CloseDialog),
                        label: "Cancel".into(),
                        style: button::Style::text(theme, Accent::Primary),
                    },
                    dialog::Button {
                        on_press: Some(Message::CloseDialog),
                        label: "OK".into(),
                        style: button::Style::filled(theme, Accent::Primary),
                    },
                ],
            )
            .title("Simple Dialog"),
            self.dialog_open,
            false,
            Some(Message::CloseDialog),
        );

        let custom_dialog_element = modal::Modal::new(
            dialog::Dialog::new(
                dialog::Style::new(theme),
                text("You can put any widget here.").color(theme.on_surface_variant()),
                vec![dialog::Button {
                    on_press: Some(Message::CloseCustom),
                    label: "Dismiss".into(),
                    style: button::Style::text(theme, Accent::Primary),
                }],
            )
            .title("Custom Dialog"),
            self.dialog_custom_open,
            false,
            Some(Message::CloseCustom),
        );

        let fullscreen_dialog_element = modal::Modal::new(
            dialog::Dialog::new(
                dialog::Style::new(theme),
                column![
                    text("This dialog covers the whole window.").color(theme.on_surface_variant()),
                    text("The close button in the header dismisses it.")
                        .color(theme.on_surface_variant()),
                ]
                .spacing(12),
                vec![dialog::Button {
                    on_press: Some(Message::CloseFullscreen),
                    label: "Done".into(),
                    style: button::Style::filled(theme, Accent::Primary),
                }],
            )
            .title("Fullscreen Dialog")
            .fullscreen(true)
            .on_close(Message::CloseFullscreen),
            self.dialog_fullscreen_open,
            true,
            Some(Message::CloseFullscreen),
        );

        stack![
            content,
            dialog_element,
            custom_dialog_element,
            fullscreen_dialog_element
        ]
        .into()
    }
}

common::demo!(Page, Message, "Dialog Demo");
