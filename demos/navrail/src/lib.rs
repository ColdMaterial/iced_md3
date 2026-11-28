use iced::{
    Alignment, Element, padding,
    widget::{column, container, row, text},
};
use iced_m3::theme::{Accent, ColorScheme, Theme};
use iced_m3::widget::{Badge, button, hybrid_icon::Icon, navrail, switch};

#[derive(Clone)]
pub enum Message {
    GoTo(usize),
    ToggleExpanded,
    SetAlignment(navrail::ItemAlignment),
    ToggleMenu,
    ToggleFAB,
    Noop,
}

pub struct Page {
    active: usize,
    expanded: bool,
    fab: bool,
    menu: bool,
    alignment: navrail::ItemAlignment,
}

impl Default for Page {
    fn default() -> Self {
        Self {
            active: 0,
            expanded: false,
            fab: true,
            menu: true,
            alignment: navrail::ItemAlignment::default(),
        }
    }
}

impl Page {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::GoTo(index) => self.active = index,
            Message::ToggleExpanded => self.expanded = !self.expanded,
            Message::SetAlignment(a) => self.alignment = a,
            Message::ToggleMenu => self.menu = !self.menu,
            Message::ToggleFAB => self.fab = !self.fab,
            Message::Noop => {}
        }
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        iced::Subscription::none()
    }

    pub fn view<'a>(&self, theme: &'a Theme) -> Element<'a, Message> {
        let items = vec![
            navrail::Item {
                icon_active: Icon::Text {
                    text: "star".into(),
                    font: Some(fonts::icons_filled()),
                },
                icon_inactive: Icon::Text {
                    text: "star".into(),
                    font: Some(fonts::icons_outlined()),
                },
                badge: None,
                label: "Star".into(),
                on_press: Message::GoTo(0),
            },
            navrail::Item {
                icon_active: Icon::Text {
                    text: "favorite".into(),
                    font: Some(fonts::icons_filled()),
                },
                icon_inactive: Icon::Text {
                    text: "favorite".into(),
                    font: Some(fonts::icons_outlined()),
                },
                badge: Some(Badge { label: None }),
                label: "the absolute best, top-tier favorite".into(),
                on_press: Message::GoTo(1),
            },
            navrail::Item {
                icon_active: Icon::Text {
                    text: "search".into(),
                    font: Some(fonts::icons_filled()),
                },
                icon_inactive: Icon::Text {
                    text: "search".into(),
                    font: Some(fonts::icons_outlined()),
                },
                badge: Some(Badge {
                    label: Some("3".into()),
                }),
                label: "Search".into(),
                on_press: Message::GoTo(2),
            },
            navrail::Item {
                icon_active: Icon::Text {
                    text: "settings".into(),
                    font: Some(fonts::icons_filled()),
                },
                icon_inactive: Icon::Text {
                    text: "settings".into(),
                    font: Some(fonts::icons_outlined()),
                },
                badge: Some(Badge {
                    label: Some("999+".into()),
                }),
                label: "Settings".into(),
                on_press: Message::GoTo(3),
            },
        ];

        let mode = match self.expanded {
            true => navrail::Mode::Expanded {
                width: iced::Pixels(navrail::CONTAINER_EXPANDED_MIN_WIDTH),
            },
            false => navrail::Mode::Collapsed,
        };

        let fab = navrail::Fab {
            icon: Icon::Text {
                text: "edit".into(),
                font: Some(fonts::icons_filled()),
            },
            label: "Edit".into(),
            style: iced_m3::widget::fab::Style::fab_tonal(theme, Accent::Primary),
            label_size_scale: 1.0,
            on_press: Message::Noop,
        };

        let navrail_widget = iced_m3::widget::navrail(theme, navrail::Style::new(theme), items)
            .active(self.active)
            .label_font(fonts::text_regular())
            .mode(mode)
            .fab_maybe(self.fab.then_some(fab))
            .item_alignment(self.alignment)
            .on_menu_pressed_maybe(self.menu.then_some(|_| Message::ToggleExpanded));

        let controls = column![
            text("Alignment").color(theme.on_surface()),
            row![
                button(
                    button::Style::elevated(theme, Accent::default()),
                    button::Content::Label("Top".into())
                )
                .on_press(Message::SetAlignment(navrail::ItemAlignment::Top)),
                button(
                    button::Style::elevated(theme, Accent::default()),
                    button::Content::Label("Center".into())
                )
                .on_press(Message::SetAlignment(navrail::ItemAlignment::Center)),
                button(
                    button::Style::elevated(theme, Accent::default()),
                    button::Content::Label("Bottom".into())
                )
                .on_press(Message::SetAlignment(navrail::ItemAlignment::Bottom)),
            ]
            .spacing(8),
            row![
                text("Menu").color(theme.on_surface()),
                switch(switch::Style::new(theme), self.menu).on_toggle(Message::ToggleMenu)
            ]
            .align_y(Alignment::Center)
            .spacing(8),
            row![
                text("Floating Action Button").color(theme.on_surface()),
                switch(switch::Style::new(theme), self.fab).on_toggle(Message::ToggleFAB)
            ]
            .align_y(Alignment::Center)
            .spacing(8),
            text(format!("Current tab index: {}", self.active)).color(theme.on_surface())
        ]
        .spacing(8);

        row![
            container(navrail_widget).style(|_| {
                iced::widget::container::Style::default().background(theme.surface_container())
            }),
            container(controls).padding(padding::horizontal(12)),
        ]
        .spacing(0)
        .into()
    }
}

common::demo!(Page, Message, "Navrail Demo", fill);
