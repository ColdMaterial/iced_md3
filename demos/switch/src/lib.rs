use common::{label, section_header};
use iced::{
    Alignment, Element,
    widget::{column, row},
};
use iced_m3::{
    animation::motion,
    theme::Theme,
    widget::{
        self, button,
        switch::{self, IconMode},
    },
};

#[derive(Clone)]
pub enum Message {
    ToggleDemo,
    ToggleNever,
    ToggleWhenSelected,
    ToggleAlways,
    SetIconMode(IconMode),
}

#[derive(Default)]
pub struct Demo {
    toggled_demo: bool,
    toggled_never: bool,
    toggled_when_selected: bool,
    toggled_always: bool,
    icon_mode: IconMode,
}

fn switch_row<'a>(
    theme: &Theme,
    label_text: &'a str,
    toggled: bool,
    icon_mode: IconMode,
    on_toggle: Option<Message>,
) -> Element<'a, Message> {
    let s = widget::switch(switch::Style::new(theme), toggled).icon_mode(icon_mode);
    let s = if let Some(msg) = on_toggle {
        s.on_toggle(msg)
    } else {
        s
    };
    row![label(label_text, theme), s]
        .spacing(12)
        .align_y(Alignment::Center)
        .into()
}

impl Demo {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::ToggleDemo => self.toggled_demo = !self.toggled_demo,
            Message::ToggleNever => self.toggled_never = !self.toggled_never,
            Message::ToggleWhenSelected => self.toggled_when_selected = !self.toggled_when_selected,
            Message::ToggleAlways => self.toggled_always = !self.toggled_always,
            Message::SetIconMode(icon_mode) => self.icon_mode = icon_mode,
        }
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        iced::Subscription::none()
    }

    pub fn view(&self, theme: &Theme) -> Element<'_, Message> {
        let dynamic_section = column![
            section_header("Dynamic Icon Mode", theme),
            row![
                label("Interactive switch", theme),
                widget::switch(switch::Style::new(theme), self.toggled_demo)
                    .motion_scheme(motion::Scheme::Expressive)
                    .icon_mode(self.icon_mode)
                    .on_toggle(Message::ToggleDemo),
            ]
            .spacing(12)
            .align_y(Alignment::Center),
            row![
                widget::button(
                    button::Style::filled(theme, iced_m3::theme::Accent::default()),
                    button::Content::Label("Never".into()),
                )
                .on_press(Message::SetIconMode(IconMode::Never)),
                widget::button(
                    button::Style::filled(theme, iced_m3::theme::Accent::default()),
                    button::Content::Label("When Selected".into()),
                )
                .on_press(Message::SetIconMode(IconMode::WhenSelected)),
                widget::button(
                    button::Style::filled(theme, iced_m3::theme::Accent::default()),
                    button::Content::Label("Always".into()),
                )
                .on_press(Message::SetIconMode(IconMode::Always))
            ]
            .spacing(8)
        ]
        .spacing(12);

        let never_section = column![
            section_header("IconMode::Never", theme),
            switch_row(
                theme,
                "On",
                self.toggled_never,
                IconMode::Never,
                Some(Message::ToggleNever)
            ),
            switch_row(
                theme,
                "Off",
                !self.toggled_never,
                IconMode::Never,
                Some(Message::ToggleNever)
            ),
            switch_row(theme, "Disabled on", true, IconMode::Never, None),
            switch_row(theme, "Disabled off", false, IconMode::Never, None),
        ]
        .spacing(12);

        let when_selected_section = column![
            section_header("IconMode::WhenSelected", theme),
            switch_row(
                theme,
                "On",
                self.toggled_when_selected,
                IconMode::WhenSelected,
                Some(Message::ToggleWhenSelected)
            ),
            switch_row(
                theme,
                "Off",
                !self.toggled_when_selected,
                IconMode::WhenSelected,
                Some(Message::ToggleWhenSelected)
            ),
            switch_row(theme, "Disabled on", true, IconMode::WhenSelected, None),
            switch_row(theme, "Disabled off", false, IconMode::WhenSelected, None),
        ]
        .spacing(12);

        let always_section = column![
            section_header("IconMode::Always", theme),
            switch_row(
                theme,
                "On",
                self.toggled_always,
                IconMode::Always,
                Some(Message::ToggleAlways)
            ),
            switch_row(
                theme,
                "Off",
                !self.toggled_always,
                IconMode::Always,
                Some(Message::ToggleAlways)
            ),
            switch_row(theme, "Disabled on", true, IconMode::Always, None),
            switch_row(theme, "Disabled off", false, IconMode::Always, None),
        ]
        .spacing(12);

        column![
            dynamic_section,
            never_section,
            when_selected_section,
            always_section,
        ]
        .spacing(24)
        .into()
    }
}

pub type Page = Demo;

common::demo!(Page, Message, "Switch Demo");
