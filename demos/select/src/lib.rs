use common::{catalog::Tab, label, section_header};
use iced::{
    Alignment, Element, Length,
    widget::{column, row},
};
use iced_m3::{
    theme::{Accent, Theme},
    widget::{
        self, button,
        checkbox::Value,
        hybrid_icon::Icon,
        icons,
        select::{Item, OpenDirection},
    },
};

#[derive(Clone)]
pub enum Message {
    SmallSelected(Option<String>),
    LargeSelected(Option<String>),
    SetDirection(OpenDirection),
    ToggleClearable,
    ToggleCloseOnSelect,
    ToggleSearch,
}

pub struct Page {
    direction: OpenDirection,
    clearable: bool,
    close_on_select: bool,
    search: bool,
    small: Option<String>,
    large: Option<String>,
}

impl Default for Page {
    fn default() -> Self {
        Self {
            direction: OpenDirection::Down,
            clearable: true,
            close_on_select: true,
            search: true,
            small: None,
            large: Some("Select".to_owned()),
        }
    }
}

fn direction_button<'a>(
    theme: &Theme,
    content: &'a str,
    active: bool,
    message: Message,
) -> Element<'a, Message> {
    let style = if active {
        button::Style::filled(theme, Accent::Primary)
    } else {
        button::Style::outlined(theme)
    };

    widget::button(style, button::Content::Label(content.into()))
        .on_press(message)
        .into()
}

fn toggle<'a>(
    theme: &Theme,
    checked: bool,
    content: &'a str,
    message: Message,
) -> Element<'a, Message> {
    row![
        widget::checkbox(
            theme,
            if checked {
                Value::Checked
            } else {
                Value::Unchecked
            },
        )
        .on_toggle(message),
        label(content, theme),
    ]
    .spacing(12)
    .align_y(Alignment::Center)
    .into()
}

impl Page {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::SmallSelected(value) => self.small = value,
            Message::LargeSelected(value) => self.large = value,
            Message::SetDirection(direction) => self.direction = direction,
            Message::ToggleClearable => self.clearable = !self.clearable,
            Message::ToggleCloseOnSelect => self.close_on_select = !self.close_on_select,
            Message::ToggleSearch => self.search = !self.search,
        }
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        iced::Subscription::none()
    }

    pub fn view<'a>(&self, theme: &'a Theme) -> Element<'a, Message> {
        // Only some options carry an icon: the icon slot is still reserved so the labels stay
        // aligned.
        let small_items: Vec<Item<String>> = vec![
            Item::new("short text".to_owned()),
            Item::separator(),
            Item::new("a long text-so long, in fact, that it doesn't all fit".to_owned())
                .icon(icons::CLOSE.clone()),
        ];

        // The page names and icons come from the shared catalog so the showcase and this demo
        // stay in sync.
        let large_items: Vec<Item<String>> = Tab::ALL
            .iter()
            .copied()
            .map(|tab| {
                Item::new(tab.name().to_owned()).icon(Icon::Text {
                    text: tab.icon().into(),
                    font: Some(fonts::icons_outlined()),
                })
            })
            .collect();

        column![
            section_header("Small list with icons and a separator", theme),
            widget::select(
                theme,
                small_items,
                self.small.clone(),
                Message::SmallSelected,
            )
            .placeholder("Select an option")
            .open_direction(self.direction)
            .clearable(self.clearable)
            .close_on_select(self.close_on_select)
            .width(Length::Fixed(280.0)),
            section_header("Widgets (scrollable menu)", theme),
            widget::select(
                theme,
                large_items,
                self.large.clone(),
                Message::LargeSelected,
            )
            .placeholder("Pick a widget")
            .open_direction(self.direction)
            .clearable(self.clearable)
            .close_on_select(self.close_on_select)
            .searchable(self.search)
            .width(Length::Fixed(280.0)),
            section_header("Open direction", theme),
            row![
                direction_button(
                    theme,
                    "Down",
                    self.direction == OpenDirection::Down,
                    Message::SetDirection(OpenDirection::Down),
                ),
                direction_button(
                    theme,
                    "Up",
                    self.direction == OpenDirection::Up,
                    Message::SetDirection(OpenDirection::Up),
                ),
                direction_button(
                    theme,
                    "Center",
                    self.direction == OpenDirection::Center,
                    Message::SetDirection(OpenDirection::Center),
                ),
            ]
            .spacing(8),
            section_header("Options", theme),
            toggle(theme, self.clearable, "Clearable", Message::ToggleClearable),
            toggle(
                theme,
                self.close_on_select,
                "Close on select",
                Message::ToggleCloseOnSelect,
            ),
            toggle(
                theme,
                self.search,
                "Search in the widget menu",
                Message::ToggleSearch,
            ),
        ]
        .spacing(16)
        .into()
    }
}

common::demo!(Page, Message, "Select Demo");
