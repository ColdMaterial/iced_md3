use common::{label, section_header};
use iced::{
    Alignment, Element,
    widget::{column, row},
};
use iced_m3::{theme::Theme, widget};

const OPTIONS: [&str; 3] = ["Option A", "Option B", "Option C"];

#[derive(Clone, Copy, PartialEq, Eq, Default)]
enum Choice {
    #[default]
    First,
    Second,
    Third,
}

impl Choice {
    fn index(self) -> usize {
        match self {
            Self::First => 0,
            Self::Second => 1,
            Self::Third => 2,
        }
    }

    fn from_index(index: usize) -> Self {
        match index {
            1 => Self::Second,
            2 => Self::Third,
            _ => Self::First,
        }
    }
}

#[derive(Clone)]
pub enum Message {
    Select(usize),
    SelectError(usize),
}

#[derive(Default)]
pub struct Demo {
    selected: Choice,
    error: Choice,
}

fn radio_row<'a>(
    theme: &Theme,
    label_text: &'a str,
    selected: bool,
    on_select: Message,
) -> Element<'a, Message> {
    row![
        widget::radio(theme, selected).on_select(on_select),
        label(label_text, theme),
    ]
    .spacing(12)
    .align_y(Alignment::Center)
    .into()
}

fn error_radio_row<'a>(
    theme: &Theme,
    label_text: &'a str,
    selected: bool,
    on_select: Message,
) -> Element<'a, Message> {
    row![
        widget::radio(theme, selected)
            .error(true, theme)
            .on_select(on_select),
        label(label_text, theme),
    ]
    .spacing(12)
    .align_y(Alignment::Center)
    .into()
}

impl Demo {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::Select(index) => self.selected = Choice::from_index(index),
            Message::SelectError(index) => self.error = Choice::from_index(index),
        }
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        iced::Subscription::none()
    }

    pub fn view(&self, theme: &Theme) -> Element<'_, Message> {
        // Radios use the expressive motion scheme by default, so the dot
        // overshoots a little as it fills from the center.
        let interactive_section = column![
            section_header("Interactive", theme),
            radio_row(
                theme,
                OPTIONS[0],
                self.selected.index() == 0,
                Message::Select(0)
            ),
            radio_row(
                theme,
                OPTIONS[1],
                self.selected.index() == 1,
                Message::Select(1)
            ),
            radio_row(
                theme,
                OPTIONS[2],
                self.selected.index() == 2,
                Message::Select(2)
            ),
        ]
        .spacing(12);

        let disabled_section = column![
            section_header("Disabled", theme),
            row![widget::radio(theme, true), label("Selected", theme)]
                .spacing(12)
                .align_y(Alignment::Center),
            row![widget::radio(theme, false), label("Unselected", theme)]
                .spacing(12)
                .align_y(Alignment::Center),
        ]
        .spacing(12);

        let error_section = column![
            section_header("Error", theme),
            error_radio_row(
                theme,
                "Error option A",
                self.error.index() == 0,
                Message::SelectError(0)
            ),
            error_radio_row(
                theme,
                "Error option B",
                self.error.index() == 1,
                Message::SelectError(1)
            ),
        ]
        .spacing(12);

        column![interactive_section, disabled_section, error_section]
            .spacing(24)
            .into()
    }
}

pub type Page = Demo;

common::demo!(Page, Message, "Radio Demo");
