use common::section_header;
use iced::{
    Alignment, Element,
    alignment::Horizontal,
    widget::{column, row, text},
};
use iced_m3::{
    theme::{Accent, ColorScheme, Theme},
    widget::{button, text_input::TextInput},
};

#[derive(Clone)]
pub enum Message {
    TextInputChanged(String),
    SecureInputChanged(String),
    SecureVisibilityToggled,
    ErrorInputChanged(String),
    ErrorToggle,
    SubmitInputChanged(String),
    SubmitInputSubmitted,
    TextInputWithIconChanged(String),
    TextInputWithLabelChanged(String),
    TextInputAlignedChanged(String),
}

#[derive(Default)]
pub struct Page {
    text_input_value: String,
    secure_text_input_value: String,
    secure_visibility: bool,
    error_input_value: String,
    error: bool,
    submit_input_value: String,
    submitted: Option<String>,
    text_input_with_icon: String,
    text_input_with_label: String,
    text_input_aligned: String,
}

impl Page {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::TextInputChanged(value) => self.text_input_value = value,
            Message::SecureInputChanged(value) => self.secure_text_input_value = value,
            Message::SecureVisibilityToggled => self.secure_visibility = !self.secure_visibility,
            Message::ErrorInputChanged(value) => self.error_input_value = value,
            Message::ErrorToggle => self.error = !self.error,
            Message::SubmitInputChanged(value) => self.submit_input_value = value,
            Message::SubmitInputSubmitted => {
                self.submitted = Some(self.submit_input_value.clone());
                self.submit_input_value.clear();
            }
            Message::TextInputWithIconChanged(v) => self.text_input_with_icon = v,
            Message::TextInputWithLabelChanged(v) => self.text_input_with_label = v,
            Message::TextInputAlignedChanged(v) => self.text_input_aligned = v,
        }
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        iced::Subscription::none()
    }

    pub fn view<'a>(&self, theme: &'a Theme) -> Element<'a, Message> {
        column![
            section_header("Text Input", theme),
            text("Right-click the input to open its context menu",)
                .color(theme.on_surface_variant()),
            TextInput::new("Type here...", &self.text_input_value, theme)
                .on_input(Message::TextInputChanged)
                .clearable(true)
                .width(300.0),
            section_header("Secure Input", theme),
            TextInput::new("Password...", &self.secure_text_input_value, theme)
                .on_input(Message::SecureInputChanged)
                .secure(!self.secure_visibility)
                .on_toggle_secure(Message::SecureVisibilityToggled)
                .width(300.0),
            section_header("Error Input", theme),
            row![
                TextInput::new("Type to trigger...", &self.error_input_value, theme)
                    .on_input(Message::ErrorInputChanged)
                    .error(self.error)
                    .width(300.0),
                iced_m3::widget::button(
                    button::Style::filled(theme, Accent::Primary),
                    button::Content::Label("Toggle Error".into()),
                )
                .on_press(Message::ErrorToggle),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
            section_header("Text input with icon", theme),
            TextInput::new("Search...", &self.text_input_with_icon, theme)
                .on_input(Message::TextInputWithIconChanged)
                .clearable(true)
                .icon_tooltip("Search")
                .icon(iced_m3::widget::text_input::Icon {
                    font: fonts::icons_filled(),
                    code_point: '\u{e8b6}',
                    size: None,
                    spacing: 8.0,
                    side: iced_m3::widget::text_input::Side::Left,
                })
                .width(300.0),
            section_header("Text input with label", theme),
            TextInput::new("Labelled input...", &self.text_input_with_label, theme)
                .on_input(Message::TextInputWithLabelChanged)
                .with_label_text("Label", theme.primary_container())
                .width(300.0),
            section_header("Text input on submit", theme),
            TextInput::new("Press Enter...", &self.submit_input_value, theme)
                .on_input(Message::SubmitInputChanged)
                .on_submit(Message::SubmitInputSubmitted)
                .width(300.0),
            text(match &self.submitted {
                Some(value) => format!("Submitted: \"{value}\""),
                None => "Press Enter to submit".to_owned(),
            })
            .color(theme.on_surface_variant()),
            section_header("Text input aligned", theme),
            TextInput::new("Centered...", &self.text_input_aligned, theme)
                .on_input(Message::TextInputAlignedChanged)
                .align_x(Horizontal::Center)
                .size(18.0)
                .line_height(1.5)
                .padding(16.0)
                .width(300.0),
        ]
        .spacing(24)
        .into()
    }
}

common::demo!(Page, Message, "Inputs Demo");
