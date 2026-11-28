use common::section_header;
use iced::{
    Element, Length, Pixels,
    widget::{column, text},
};
use iced_m3::theme::{Accent, ColorScheme, Theme};
use iced_m3::widget::{
    Action,
    advanced::{menu_overlay, menu_overlay::Placement},
    button,
    button::Content,
    hybrid_icon::Icon,
    menu,
};

#[derive(Clone)]
pub enum Message {
    ToggleMenu,
    CloseMenu,
    ToggleMoreOptions,
    Action(&'static str),
}

#[derive(Default)]
pub struct Page {
    is_menu_opened: bool,
    is_more_options_opened: bool,
    last_action: Option<&'static str>,
}

impl Page {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::ToggleMenu => {
                self.is_menu_opened = !self.is_menu_opened;

                if !self.is_menu_opened {
                    self.is_more_options_opened = false;
                }
            }
            Message::CloseMenu => {
                self.is_menu_opened = false;
                self.is_more_options_opened = false;
            }
            Message::ToggleMoreOptions => {
                self.is_more_options_opened = !self.is_more_options_opened;
            }
            Message::Action(action) => {
                self.last_action = Some(action);
                self.is_menu_opened = false;
                self.is_more_options_opened = false;
            }
        }
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        iced::Subscription::none()
    }

    pub fn view<'a>(&'a self, theme: &'a Theme) -> Element<'a, Message> {
        let anchor = iced_m3::widget::button(
            button::Style::filled(theme, Accent::Primary),
            Content::Full {
                icon: menu_icon("menu"),
                label: "Menu".into(),
            },
        )
        .on_press(Message::ToggleMenu);

        let is_more_options_opened = self.is_more_options_opened;

        let menu_button = menu_overlay(
            anchor,
            move || menu_surface(theme, is_more_options_opened),
            self.is_menu_opened,
        )
        .placement(Placement::BottomRight {
            padding: Pixels(4.0),
        })
        .on_press(|press| {
            let is_inside_surface = press
                .position
                .is_some_and(|position| press.bounds.contains(position));

            (!is_inside_surface).then_some(Message::CloseMenu)
        });

        let last_action: Element<'_, Message> = match self.last_action {
            Some(action) => text(format!("Last action: {action}")).color(theme.on_surface()),
            None => text("No action picked yet").color(theme.on_surface_variant()),
        }
        .into();

        column![
            section_header("Menu button", theme),
            text("Press the button to open a menu and pick an item. \"More options\" opens a nested menu.").color(
                theme.on_surface_variant()
            ),
            menu_button,
            last_action,
        ]
        .spacing(24)
        .width(Length::Shrink)
        .into()
    }
}

fn menu_surface<'a>(theme: &'a Theme, is_more_options_opened: bool) -> Element<'a, Message> {
    let groups = vec![
        menu::Group::new(vec![
            menu::Item::Button(menu_item(
                "content_copy",
                "Copy",
                "Ctrl+C",
                Some(Message::Action("Copy")),
            )),
            menu::Item::Button(menu_item(
                "content_cut",
                "Cut",
                "Ctrl+X",
                Some(Message::Action("Cut")),
            )),
            menu::Item::Separator,
            menu::Item::Button(menu_item(
                "content_paste",
                "Paste",
                "Ctrl+V",
                Some(Message::Action("Paste")),
            )),
        ]),
        menu::Group {
            label: Some("Actions".into()),
            items: vec![
                menu::Item::Button(menu_item("undo", "Undo", "", None)),
                menu::Item::Button(menu_item(
                    "redo",
                    "Redo",
                    "Ctrl+Y",
                    Some(Message::Action("Redo")),
                )),
                menu::Item::Button(menu::button::Button {
                    icon: Some(menu_icon("more_horiz")),
                    label: "More options".into(),
                    supporting_text: None,
                    action: menu::button::MenuAction::SubMenu {
                        is_opened: is_more_options_opened,
                        surface: Box::new(move || more_options_surface(theme)),
                        on_press: Action::Direct(Message::ToggleMoreOptions),
                    },
                }),
            ],
        },
    ];

    iced_m3::widget::menu(menu::Style::new(theme, false), groups).into()
}

fn more_options_surface<'a>(theme: &'a Theme) -> menu::Menu<'a, Message> {
    menu::Menu::new(
        menu::Style::new(theme, true),
        vec![menu::Group::new(vec![
            menu::Item::Button(menu_item(
                "settings",
                "Settings",
                "",
                Some(Message::Action("Settings")),
            )),
            menu::Item::Button(menu_item(
                "share",
                "Share",
                "Ctrl+Shift+S",
                Some(Message::Action("Share")),
            )),
            menu::Item::Separator,
            menu::Item::Button(menu_item(
                "delete",
                "Delete",
                "",
                Some(Message::Action("Delete")),
            )),
        ])],
    )
}

fn menu_item<'a>(
    icon: &'a str,
    label: &'a str,
    supporting_text: &'a str,
    on_press: Option<Message>,
) -> menu::button::Button<'a, Message, iced::Theme, iced::Renderer> {
    menu::button::Button {
        icon: Some(menu_icon(icon)),
        label: label.into(),
        supporting_text: (!supporting_text.is_empty()).then(|| supporting_text.into()),
        action: menu::button::MenuAction::Message {
            is_selected: false,
            on_press: on_press.map(Action::Direct),
        },
    }
}

fn menu_icon(text: &str) -> Icon<'_> {
    Icon::Text {
        text: text.into(),
        font: Some(fonts::icons_filled()),
    }
}

common::demo!(Page, Message, "Menu Demo");
