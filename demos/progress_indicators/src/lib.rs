use std::time::Duration;

use common::section_header;
use iced::{
    Alignment, Element,
    widget::{column, row},
};
use iced_m3::theme::{Accent, Theme};
use iced_m3::widget::{button, circular_loading_indicator, loading_indicator, progress_bar};

/// How often the animation advances.
const TICK_RATE: Duration = Duration::from_millis(50);
/// How long a full sweep of the progress bar takes.
const ANIM_DURATION: Duration = Duration::from_secs(10);

#[derive(Clone)]
pub enum Message {
    Tick,
    ToggleMaybe,
    ToggleHeightMaybe,
}

pub struct Page {
    progress_value: f32,
    progress_maybe: Option<f32>,
    progress_height_maybe: Option<f32>,
}

impl Default for Page {
    fn default() -> Self {
        Self {
            progress_value: 0.0,
            progress_maybe: Some(0.5),
            progress_height_maybe: Some(12.0),
        }
    }
}

impl Page {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::Tick => {
                self.progress_value += TICK_RATE.as_secs_f32() / ANIM_DURATION.as_secs_f32();
                if self.progress_value > 1.0 {
                    self.progress_value = 0.0;
                }
            }
            Message::ToggleMaybe => {
                self.progress_maybe = if self.progress_maybe.is_some() {
                    None
                } else {
                    Some(0.5)
                };
            }
            Message::ToggleHeightMaybe => {
                self.progress_height_maybe = if self.progress_height_maybe.is_some() {
                    None
                } else {
                    Some(12.0)
                };
            }
        }
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        iced::time::every(TICK_RATE).map(|_| Message::Tick)
    }

    pub fn view(&self, theme: &Theme) -> Element<'_, Message> {
        column![
            section_header("Linear progress bars", theme),
            row![
                progress_bar(progress_bar::Style::new(theme)).progress(0.0),
                progress_bar(progress_bar::Style::new(theme)).progress(0.25),
                progress_bar(progress_bar::Style::new(theme)).progress(0.5),
                progress_bar(progress_bar::Style::new(theme)).progress(0.75),
                progress_bar(progress_bar::Style::new(theme)).progress(1.0),
            ]
            .spacing(8),
            section_header("Animated progress", theme),
            progress_bar(progress_bar::Style::new(theme)).progress(self.progress_value),
            section_header("With custom height", theme),
            progress_bar(progress_bar::Style::new(theme))
                .progress(0.6)
                .height(24.0),
            section_header("Indeterminate", theme),
            progress_bar(progress_bar::Style::new(theme)),
            progress_bar(progress_bar::Style::new(theme)).height(16.0),
            section_header("Circular loading indicators", theme),
            row![
                loading_indicator(loading_indicator::Style::contained(theme)),
                loading_indicator(loading_indicator::Style::uncontained(theme)),
            ]
            .spacing(16)
            .align_y(Alignment::Center),
            section_header("Circular spinner", theme),
            row![
                circular_loading_indicator(loading_indicator::Style::contained(theme)),
                circular_loading_indicator(loading_indicator::Style::uncontained(theme)),
                circular_loading_indicator(loading_indicator::Style::uncontained(theme)).size(24.0),
            ]
            .spacing(16)
            .align_y(Alignment::Center),
            section_header("Maybe value", theme),
            row![
                progress_bar(progress_bar::Style::new(theme)).progress_maybe(self.progress_maybe),
                iced_m3::widget::button(
                    button::Style::elevated(theme, Accent::default()),
                    button::Content::Label("Toggle value".into())
                )
                .on_press(Message::ToggleMaybe),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
            section_header("Maybe height", theme),
            row![
                progress_bar(progress_bar::Style::new(theme))
                    .progress(0.6)
                    .height_maybe(self.progress_height_maybe),
                iced_m3::widget::button(
                    button::Style::elevated(theme, Accent::default()),
                    button::Content::Label("Toggle height".into())
                )
                .on_press(Message::ToggleHeightMaybe),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        ]
        .spacing(24)
        .into()
    }
}

common::demo!(Page, Message, "Progress Indicators Demo");
