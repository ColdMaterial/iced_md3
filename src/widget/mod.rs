#[cfg(feature = "advanced")]
pub mod advanced;
#[cfg(not(feature = "advanced"))]
pub(crate) mod advanced;
pub mod badge;
pub mod button;
pub mod button_group;
pub mod card;
pub mod checkbox;
mod common_icons;
mod common_types;
pub mod dialog;
pub mod fab;
pub mod fab_menu;
pub mod hybrid_icon;
pub(crate) mod internal;
pub mod loading_indicator;
pub mod menu;
pub mod navbar;
pub mod navrail;
pub mod progress_bar;
pub mod radio;
pub(crate) mod ripple;
pub mod scrollable;
pub mod select;
#[cfg(feature = "shapes")]
pub mod shapes;
pub mod slider;
pub mod switch;
pub mod text_input;
pub mod tooltip;
mod wrappers;

pub use common_types::*;
pub use scrollable::scrollable;

pub use wrappers::*;

use std::ops::RangeInclusive;

use iced::{Color, Element, Font, Pixels};

use crate::widget::loading_indicator::CircularLoadingIndicator;
#[cfg(feature = "shapes")]
use crate::widget::loading_indicator::LoadingIndicator;
use crate::{
    theme::ColorScheme,
    widget::{
        button::Button,
        button_group::connected::ButtonGroup,
        card::Card,
        checkbox::Checkbox,
        dialog::Dialog,
        fab::Fab,
        fab_menu::FABMenu,
        hybrid_icon::{HybridIcon, Icon},
        menu::Menu,
        navbar::NavBar,
        navrail::NavRail,
        progress_bar::ProgressBar,
        radio::Radio,
        select::{Item as SelectItem, Select},
        switch::Switch,
        text_input::TextInput,
    },
};

#[must_use]
pub fn text_input<'a, Message, Renderer>(
    placeholder: &str,
    value: &'a str,
    theme: &'a impl ColorScheme,
) -> TextInput<'a, Message>
where
    Message: Clone,
{
    TextInput::new(placeholder, value, theme)
}

#[must_use]
pub fn navbar<'a, Message, Theme, Renderer>(
    style: navbar::Style,
    items: Vec<navbar::Item<'a, Message>>,
) -> NavBar<'a, Message, Theme, Renderer>
where
    Message: 'a + Clone,
    Theme: 'a,
    Renderer: 'a
        + iced::advanced::text::Renderer
        + iced::advanced::svg::Renderer
        + iced_widget::renderer::geometry::Renderer,
    Renderer::Font: From<iced::Font>,
{
    NavBar::new(style, items)
}

/// Buttons prompt most actions in a UI
#[must_use]
pub fn button<'a, Message, Renderer>(
    style: button::Style,
    content: button::Content<'a>,
) -> Button<'a, Message, Renderer>
where
    Message: Clone,
    Renderer: iced::advanced::text::Renderer,
{
    Button::new(style, content)
}

#[must_use]
pub fn fab_menu<'a, I, Message, Theme, Renderer>(
    style: fab_menu::Style,
    items: I,
    label_font: Option<Font>,
) -> FABMenu<'a, Message, Theme, Renderer>
where
    Message: 'a + Clone,
    Theme: 'a,
    I: IntoIterator<Item = fab_menu::Item<'a, Message>>,
    Renderer: 'a
        + iced::advanced::text::Renderer
        + iced::advanced::svg::Renderer
        + iced_widget::renderer::geometry::Renderer,
    Renderer::Font: From<iced::Font>,
{
    FABMenu::new(style, items, label_font)
}

#[must_use]
pub fn slider<'a, F, T, Message>(
    range: RangeInclusive<T>,
    value: T,
    on_change: F,
    style: slider::Style,
) -> slider::Slider<'a, T, Message>
where
    F: 'a + Fn(T) -> Message,
    T: 'a + Copy + std::fmt::Display + From<u8> + PartialOrd + num_traits::cast::FromPrimitive,
    Message: 'a + Clone,
    f64: std::convert::From<T>,
{
    slider::Slider::new(range, value, on_change, style)
}

#[must_use]
pub fn range_slider<'a, F, T, Message>(
    range: RangeInclusive<T>,
    start: T,
    end: T,
    on_change: F,
    style: slider::Style,
) -> slider::RangeSlider<'a, T, Message>
where
    F: 'a + Fn(T, T) -> Message,
    T: 'a + Copy + std::fmt::Display + From<u8> + PartialOrd + num_traits::cast::FromPrimitive,
    Message: 'a + Clone,
    f64: std::convert::From<T>,
{
    slider::RangeSlider::new(range, start, end, on_change, style)
}

#[must_use]
pub fn dialog<'a, Message>(
    style: dialog::Style,
    body: impl Into<Element<'a, Message>>,
    buttons: Vec<dialog::Button<Message>>,
) -> Dialog<'a, Message> {
    Dialog::new(style, body, buttons)
}

#[must_use]
pub fn progress_bar(style: progress_bar::Style) -> ProgressBar {
    ProgressBar::new(style)
}

#[must_use]
pub fn fab<'a, Message>(
    style: fab::Style,
    content: fab::Content<'a>,
    on_press: Message,
) -> Fab<'a, Message>
where
    Message: 'a + Clone,
{
    Fab::new(style, content, on_press)
}

#[must_use]
pub fn badge<'a, Message, Theme, Renderer>(
    style: badge::Style,
    base: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> badge::Badge<'a, Message, Theme, Renderer>
where
    Renderer: 'a + iced::advanced::text::Renderer,
{
    badge::Badge::new(style, base)
}

#[must_use]
pub fn navrail<'a, Message, Theme, Renderer>(
    theme: &(impl ColorScheme + ?Sized),
    style: navrail::Style,
    items: Vec<navrail::Item<'a, Message>>,
) -> NavRail<'a, Message, Theme, Renderer>
where
    Message: 'a + Clone,
    Renderer: iced::advanced::text::Renderer
        + iced::advanced::svg::Renderer
        + iced_widget::renderer::geometry::Renderer,
    Renderer::Font: From<iced::Font>,
{
    NavRail::new(style, items).scrollbar_color(theme.on_surface_variant())
}

#[must_use]
pub fn switch<'a, Message, Theme, Renderer>(
    style: switch::Style,
    toggled: bool,
) -> Switch<Message, Theme, Renderer>
where
    Message: 'a + Clone,
    Theme: 'a,
    Renderer: 'a + iced::advanced::svg::Renderer + iced_widget::renderer::geometry::Renderer,
{
    Switch::new(style, toggled)
}

#[must_use]
pub fn checkbox<Message>(
    theme: &(impl ColorScheme + ?Sized),
    value: checkbox::Value,
) -> Checkbox<Message>
where
    Message: Clone,
{
    Checkbox::new(theme, value)
}

/// Radio buttons let people select a single option from a set.
#[must_use]
pub fn radio<Message>(theme: &(impl ColorScheme + ?Sized), selected: bool) -> Radio<Message>
where
    Message: Clone,
{
    Radio::new(theme, selected)
}

#[must_use]
pub fn card<'a, Message, Theme, Renderer>(
    style: card::Style,
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Card<'a, Message, Theme, Renderer>
where
    Message: Clone,
    Renderer: iced::advanced::Renderer,
{
    Card::new(style, content)
}

#[must_use]
pub fn hybrid_icon<'a>(icon: Icon<'a>, size: impl Into<Pixels>, color: Color) -> HybridIcon<'a> {
    HybridIcon::new(icon, size, color)
}

/// Shows a short description of another element when it is hovered.
#[must_use]
pub fn tooltip<'a, Message, Theme, Renderer>(
    style: tooltip::Style,
    base: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> tooltip::Tooltip<'a, Message, Theme, Renderer>
where
    Renderer: 'a + iced::advanced::text::Renderer,
    Renderer::Font: From<iced::Font>,
{
    tooltip::Tooltip::new(style, base)
}

#[cfg(feature = "shapes")]
#[must_use]
pub fn shape<'a, Renderer>(
    polygon: polymorpher::RoundedPolygon,
    style: shapes::Style<'a>,
) -> shapes::MaterialShape<'a, Renderer>
where
    Renderer: 'a + iced_widget::renderer::geometry::Renderer,
{
    shapes::MaterialShape::new(polygon, style)
}

/// Loading indicators show the progress of a process for a short wait time
#[cfg(feature = "shapes")]
#[must_use]
pub fn loading_indicator(style: loading_indicator::Style) -> LoadingIndicator {
    LoadingIndicator::new(style)
}

/// A classic indeterminate circular spinner.
#[must_use]
pub fn circular_loading_indicator(style: loading_indicator::Style) -> CircularLoadingIndicator {
    CircularLoadingIndicator::new(style)
}

/// Exposed dropdown menu that lets people pick one option from a list.
#[must_use]
pub fn select<'a, T, Message>(
    theme: &(impl ColorScheme + ?Sized),
    items: impl IntoIterator<Item = SelectItem<T>>,
    selected: Option<T>,
    on_selected: impl Fn(Option<T>) -> Message + 'a,
) -> Select<'a, T, Message>
where
    T: Clone + PartialEq,
    Message: Clone,
{
    Select::new(theme, items, selected, on_selected)
}

/// A small set of SVG icons bundled with the library.
#[cfg(feature = "icons")]
pub mod icons {
    pub use super::common_icons::*;
}

/// Connected button groups help people select options, switch views, or sort elements in a page.
#[must_use]
pub fn connected_button_group<'a, Message>(
    style: button::Style,
    model: Vec<button_group::connected::Button<'a, Message>>,
) -> button_group::connected::ButtonGroup<'a, Message>
where
    Message: Clone,
{
    ButtonGroup::new(style, model)
}

/// Standard button groups organize [`Button`] widgets in a row and affect the size of pressed buttons.
#[must_use]
pub fn standard_button_group<'a, Message, Theme, Renderer>(
    buttons: impl IntoIterator<Item = button_group::standard::GroupItem<'a, Message, Theme, Renderer>>,
) -> button_group::standard::ButtonGroup<'a, Message, Theme, Renderer>
where
    Message: 'a + Clone,
    Renderer: 'a
        + iced::advanced::text::Renderer
        + iced::advanced::svg::Renderer
        + iced_widget::renderer::geometry::Renderer,
    Renderer::Font: From<iced::Font>,
{
    button_group::standard::ButtonGroup::new(buttons)
}

/// Creates a new [`Menu`] widget.
#[must_use]
pub fn menu<'a, Message, Theme, Renderer>(
    style: menu::Style,
    groups: impl IntoIterator<Item = menu::Group<'a, Message, Theme, Renderer>>,
) -> Menu<'a, Message, Theme, Renderer>
where
    Message: Clone,
    Renderer: iced::advanced::text::Renderer,
{
    Menu::new(style, groups)
}
