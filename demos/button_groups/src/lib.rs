use common::section_header;
use iced::{
    Element,
    widget::{column, text},
};
use iced_m3::theme::{Accent, ColorScheme, Theme};
use iced_m3::widget::{button, button_group, connected_button_group, standard_button_group};

#[derive(Clone)]
pub enum Message {
    SelectIndex(usize),
    ToggleButton(usize),
}

#[derive(Default)]
pub struct Page {
    selected: usize,
    buttons: [bool; 3],
}

impl Page {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::SelectIndex(index) => self.selected = index,
            Message::ToggleButton(index) => self.buttons[index] = !self.buttons[index],
        }
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        iced::Subscription::none()
    }

    pub fn view(&self, theme: &Theme) -> Element<'_, Message> {
        let single_select = connected_button_group(
            button::Style::filled(theme, Accent::Primary),
            (0..4)
                .map(|i| button_group::connected::Button {
                    content: button::Content::Label(format!("Option {}", i + 1).into()),
                    on_press: Some(Message::SelectIndex(i)),
                    selected: self.selected == i,
                })
                .collect(),
        );

        let items = self.buttons.iter().enumerate().map(|(i, &selected)| {
            let btn = iced_m3::widget::button(
                button::Style::filled(theme, Accent::Secondary),
                button::Content::Label(format!("Toggle {}", i + 1).into()),
            )
            .on_press(Message::ToggleButton(i))
            .selected(selected);
            button_group::standard::GroupItem::new(btn, true)
        });
        let multi_select =
            standard_button_group(items).spacing(button_group::standard::SPACING_MEDIUM);

        column![
            section_header("Connected button group (single select)", theme),
            single_select,
            text(format!("Selected index: {}", self.selected)).color(theme.on_surface()),
            section_header("Standard button group (multi select)", theme),
            multi_select,
            text(format!("Selected: {:?}", self.buttons)).color(theme.on_surface()),
        ]
        .spacing(24)
        .into()
    }
}

common::demo!(Page, Message, "Button Groups Demo");
