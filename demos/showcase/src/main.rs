use common::catalog::Tab;
use demo_sliders as sliders;
use iced::{
    Alignment, Element, Font, Length, Subscription, Task,
    widget::{Space, column, container, row, text},
};
use iced_m3::{
    theme::{ColorScheme, Mode, Theme},
    widget::{hybrid_icon::Icon, navrail, scrollable},
};
use menu_demo as menu;
use navbar_demo as navbar;
use radio_demo as radio;
use select_demo as select;
use shapes_demo as shapes;

const APP_NAME: &str = "Material 3 Showcase";

#[derive(Clone)]
enum Message {
    SwitchTab(Tab),
    SetTheme(Mode),
    ToggleNavRailExpanded,
    Buttons(buttons::Message),
    Inputs(inputs::Message),
    Sliders(sliders::Message),
    Progress(progress_indicators::Message),
    Radio(radio::Message),
    Cards(card::Message),
    Fab(fab::Message),

    Navigation(navbar::Message),
    Shapes(shapes::Message),
    Dialog(dialog::Message),
    Groups(button_groups::Message),
    Checkbox(checkbox::Message),
    Switch(::switch::Message),
    Menu(menu::Message),
    Badge(badges::Message),
    NavRail(::navrail::Message),
    Select(select::Message),
    SystemTheme(Mode),
    FontsLoaded,
}

struct State {
    theme: Theme,
    tab: Tab,
    navrail_expanded: bool,
    buttons_page: buttons::Page,
    inputs_page: inputs::Page,
    sliders_page: sliders::Page,
    progress_page: progress_indicators::Page,
    radio_page: radio::Page,
    card_page: card::Page,
    fab_page: fab::Page,
    navbar_page: navbar::Page,
    dialog_page: dialog::Page,
    groups_page: button_groups::Page,
    checkbox_page: checkbox::Page,
    switch_page: ::switch::Page,
    menu_page: menu::Page,
    badge_page: badges::Page,
    navrail_page: ::navrail::Page,

    shapes_page: shapes::Page,
    select_page: select::Page,
}

impl Default for State {
    fn default() -> Self {
        Self {
            theme: Theme::default(Mode::System),
            tab: Tab::default(),
            navrail_expanded: true,
            buttons_page: buttons::Page::default(),
            inputs_page: inputs::Page::default(),
            sliders_page: sliders::Page::default(),
            progress_page: progress_indicators::Page::default(),
            radio_page: radio::Page::default(),
            card_page: card::Page::default(),
            fab_page: fab::Page::default(),
            navbar_page: navbar::Page::default(),
            dialog_page: dialog::Page::default(),
            groups_page: button_groups::Page::default(),
            checkbox_page: checkbox::Page::default(),
            switch_page: ::switch::Page::default(),
            menu_page: menu::Page::default(),
            badge_page: badges::Page,
            navrail_page: ::navrail::Page::default(),

            shapes_page: shapes::Page::default(),
            select_page: select::Page::default(),
        }
    }
}

impl State {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SwitchTab(tab) => self.tab = tab,
            Message::SetTheme(mode) => self.theme.mode = mode,
            Message::ToggleNavRailExpanded => self.navrail_expanded = !self.navrail_expanded,
            Message::Buttons(msg) => self.buttons_page.update(msg),
            Message::Inputs(msg) => self.inputs_page.update(msg),
            Message::Sliders(msg) => self.sliders_page.update(msg),
            Message::Progress(msg) => self.progress_page.update(msg),
            Message::Radio(msg) => self.radio_page.update(msg),
            Message::Cards(msg) => self.card_page.update(msg),
            Message::Fab(msg) => self.fab_page.update(msg),

            Message::Navigation(msg) => self.navbar_page.update(msg),
            Message::Shapes(msg) => self.shapes_page.update(msg),
            Message::Dialog(msg) => self.dialog_page.update(msg),
            Message::Groups(msg) => self.groups_page.update(msg),
            Message::Checkbox(msg) => self.checkbox_page.update(msg),
            Message::Switch(msg) => self.switch_page.update(msg),
            Message::Menu(msg) => self.menu_page.update(msg),
            Message::Badge(msg) => self.badge_page.update(msg),
            Message::NavRail(msg) => self.navrail_page.update(msg),
            Message::Select(msg) => self.select_page.update(msg),
            Message::SystemTheme(mode) => self.theme.system_mode = mode,
            // Fonts are loaded asynchronously; this message exists so the app
            // redraws (and re-measures text) once they are available. Without
            // it the first frame, rendered before the fonts are registered,
            // stays on screen and text is invisible on the web.
            Message::FontsLoaded => {}
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let nav_items: Vec<navrail::Item<Message>> = Tab::ALL
            .iter()
            .copied()
            .map(|tab| navrail::Item {
                icon_active: Icon::Text {
                    text: tab.icon().into(),
                    font: Some(fonts::icons_filled()),
                },
                icon_inactive: Icon::Text {
                    text: tab.icon().into(),
                    font: Some(fonts::icons_outlined()),
                },
                badge: None,
                label: tab.name().into(),
                on_press: Message::SwitchTab(tab),
            })
            .collect();

        let navrail_mode = match self.navrail_expanded {
            true => navrail::Mode::ExpandedAuto,
            false => navrail::Mode::Collapsed,
        };

        let navrail_widget =
            iced_m3::widget::navrail(&self.theme, navrail::Style::new(&self.theme), nav_items)
                .active(self.tab.index())
                .mode(navrail_mode)
                .label_font(fonts::text_regular())
                .hide_labels_when_collapsed(false)
                .on_menu_pressed(|_| Message::ToggleNavRailExpanded)
                .menu_visible(true);

        let header = row![
            text(self.tab.name())
                .font(fonts::text_bold())
                .size(24.0)
                .color(self.theme.on_surface()),
            Space::new().width(Length::Fill),
            common::theme_select(&self.theme, self.theme.mode, Message::SetTheme)
                .width(Length::Fixed(220.0))
        ]
        .align_y(Alignment::Center);

        let content: Element<'_, Message> = match self.tab {
            Tab::Badge => self.badge_page.view(&self.theme).map(Message::Badge),
            Tab::Groups => self.groups_page.view(&self.theme).map(Message::Groups),
            Tab::Buttons => self.buttons_page.view(&self.theme).map(Message::Buttons),
            Tab::Cards => self.card_page.view(&self.theme).map(Message::Cards),
            Tab::Checkbox => self.checkbox_page.view(&self.theme).map(Message::Checkbox),
            Tab::Dialog => self.dialog_page.view(&self.theme).map(Message::Dialog),
            Tab::Fab => self.fab_page.view(&self.theme).map(Message::Fab),

            Tab::Inputs => self.inputs_page.view(&self.theme).map(Message::Inputs),
            Tab::Menu => self.menu_page.view(&self.theme).map(Message::Menu),
            Tab::Navigation => self.navbar_page.view(&self.theme).map(Message::Navigation),
            Tab::NavRail => self.navrail_page.view(&self.theme).map(Message::NavRail),
            Tab::Progress => self.progress_page.view(&self.theme).map(Message::Progress),
            Tab::Radio => self.radio_page.view(&self.theme).map(Message::Radio),
            Tab::Select => self.select_page.view(&self.theme).map(Message::Select),
            Tab::Shapes => self.shapes_page.view(&self.theme).map(Message::Shapes),
            Tab::Sliders => self.sliders_page.view(&self.theme).map(Message::Sliders),
            Tab::Switch => self.switch_page.view(&self.theme).map(Message::Switch),
        };

        // The navigation rail page must not be wrapped in a scrollable: the
        // scrollable lays its content out with a compressed cross axis, so the
        // rail would only be as tall as its siblings instead of filling the
        // remaining height.
        let body: Element<'_, Message> = match self.tab {
            Tab::NavRail => content,
            _ => scrollable(&self.theme, content)
                .id(self.tab.name())
                .spacing(16)
                .width(Length::Fill)
                .into(),
        };

        let main_area = container(column![header, body].spacing(16).padding(16))
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_| {
                iced::widget::container::Style::default().background(self.theme.background())
            });

        row![
            container(navrail_widget).style(|_| {
                iced::widget::container::Style::default().background(self.theme.surface_container())
            }),
            main_area,
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        let page = match self.tab {
            Tab::Progress => self.progress_page.subscription().map(Message::Progress),
            _ => Subscription::none(),
        };

        Subscription::batch([
            page,
            iced::system::theme_changes().map(|mode| Message::SystemTheme(mode.into())),
        ])
    }
}

fn main() -> iced::Result {
    iced::application(
        || {
            (
                State::default(),
                Task::batch([
                    Task::batch(
                        fonts::get_fonts()
                            .into_iter()
                            .map(|b| iced::font::load(b).map(|_| Message::FontsLoaded)),
                    ),
                    iced::system::theme().map(|mode| Message::SystemTheme(mode.into())),
                ]),
            )
        },
        State::update,
        State::view,
    )
    .title(APP_NAME)
    .default_font(Font::with_name("Noto Sans"))
    .subscription(State::subscription)
    .run()
}
