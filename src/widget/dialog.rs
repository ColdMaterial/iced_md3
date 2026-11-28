use iced::{Alignment, Border, Color, Element, Font, Length, Padding, Pixels, advanced::text};
use iced_widget::{column, container, row, space, text::IntoFragment};

use crate::{
    style::{Elevation, StateLayer},
    theme::{Accent, ColorScheme},
    widget::{button, common_icons, hybrid_icon::Icon, icon, tooltip},
};

mod constants {
    pub const MIN_WIDTH: f32 = 280.0;
    pub const MAX_WIDTH: f32 = 560.0;
}

const DIALOG_RADIUS: f32 = 24.0;
const ICON_SIZE: f32 = 24.0;
const TITLE_SIZE: f32 = 24.0;
const PADDING: f32 = 24.0;
const ICON_TITLE_SPACING: f32 = 16.0;
const TITLE_BODY_SPACING: f32 = 16.0;
const BODY_BUTTONS_SPACING: f32 = 24.0;
const BUTTON_SPACING: f32 = 8.0;
const CLOSE_BUTTON_SIZE: f32 = 48.0;
const CLOSE_ICON_SIZE: f32 = 24.0;

#[cfg(feature = "pub-internal-const")]
pub use constants::*;

pub struct Button<Message> {
    pub on_press: Option<Message>,
    pub label: String,
    pub style: crate::widget::button::Style,
}

pub struct Style {
    pub icon_color: Color,
    pub title_color: Color,
    pub container_color: Color,
    pub container_border: Border,
    pub elevation: Elevation,
    pub shadow_color: Color,
    /// Style of the close button shown in the header of a fullscreen dialog.
    pub close_button: button::Style,
    /// Style of the tooltip shown for the fullscreen close button.
    pub tooltip_style: tooltip::Style,
}

impl Style {
    pub fn new(theme: &(impl ColorScheme + ?Sized)) -> Self {
        let mut close_button = button::Style::text(theme, Accent::Primary);
        let close_icon = theme.on_surface_variant();
        close_button.regular.icon = close_icon;
        close_button.unselected.icon = close_icon;
        close_button.selected.icon = close_icon;
        let close_layer = StateLayer::new(close_icon);
        close_button.state_layer = close_layer;
        close_button.state_layer_unselected = close_layer;
        close_button.state_layer_selected = close_layer;

        Self {
            icon_color: theme.secondary(),
            title_color: theme.on_surface(),
            container_color: theme.surface_container_high(),
            container_border: Border::default().rounded(DIALOG_RADIUS),
            elevation: Elevation::Level3,
            shadow_color: theme.shadow(),
            close_button,
            tooltip_style: tooltip::Style::new(theme),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Labels<'a> {
    pub close: text::Fragment<'a>,
}

impl Default for Labels<'_> {
    fn default() -> Self {
        Self {
            close: "Close".into(),
        }
    }
}

pub struct Dialog<'a, Message> {
    style: Style,
    icon: Option<char>,
    icon_font: Option<Font>,
    title: Option<text::Fragment<'a>>,
    title_font: Option<Font>,
    body: Element<'a, Message>,
    buttons: Vec<Button<Message>>,
    button_label_font: Option<Font>,
    width: Option<f32>,
    height: Option<f32>,
    close: Option<Message>,
    fullscreen: bool,
    labels: Labels<'a>,
    tooltips_enabled: bool,
}

impl<'a, Message> Dialog<'a, Message> {
    #[must_use]
    pub fn new(
        style: Style,
        body: impl Into<Element<'a, Message>>,
        buttons: Vec<Button<Message>>,
    ) -> Self {
        Self {
            style,
            icon: None,
            icon_font: None,
            title: None,
            title_font: None,
            body: body.into(),
            buttons,
            button_label_font: None,
            width: None,
            height: None,
            close: None,
            fullscreen: false,
            labels: Labels::default(),
            tooltips_enabled: false,
        }
    }

    #[must_use]
    pub fn icon(mut self, icon: char) -> Self {
        self.icon = Some(icon);
        self
    }

    crate::macros::maybe_assign!(icon_maybe, icon, Option<char>);

    #[must_use]
    pub fn icon_font(mut self, font: Font) -> Self {
        self.icon_font = Some(font);
        self
    }

    crate::macros::maybe_assign!(icon_font_maybe, icon_font, Option<Font>);

    #[must_use]
    pub fn title(mut self, title: impl IntoFragment<'a>) -> Self {
        self.title = Some(title.into_fragment());
        self
    }

    #[must_use]
    pub fn title_font(mut self, font: Font) -> Self {
        self.title_font = Some(font);
        self
    }

    crate::macros::maybe_assign!(title_font_maybe, title_font, Option<Font>);

    #[must_use]
    pub fn button_label_font(mut self, font: Font) -> Self {
        self.button_label_font = Some(font);
        self
    }

    crate::macros::maybe_assign!(button_label_font_maybe, button_label_font, Option<Font>);

    #[must_use]
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    crate::macros::maybe_assign!(width_maybe, width, Option<f32>);

    #[must_use]
    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }

    crate::macros::maybe_assign!(height_maybe, height, Option<f32>);

    /// Sets the message produced by the close button of a fullscreen dialog.
    #[must_use]
    pub fn on_close(mut self, on_close: Message) -> Self {
        self.close = Some(on_close);
        self
    }

    crate::macros::maybe_assign!(on_close_maybe, close, Option<Message>);

    /// Makes the dialog surface fill the space allocated by its parent.
    #[must_use]
    pub fn fullscreen(mut self, fullscreen: bool) -> Self {
        self.fullscreen = fullscreen;
        self
    }

    #[must_use]
    pub fn labels(mut self, labels: Labels<'a>) -> Self {
        self.labels = labels;
        self
    }

    #[must_use]
    pub fn close_label(mut self, label: impl IntoFragment<'a>) -> Self {
        self.labels.close = label.into_fragment();
        self
    }

    #[must_use]
    pub fn tooltips_enabled(mut self, enabled: bool) -> Self {
        self.tooltips_enabled = enabled;
        self
    }
}

impl<'a, Message> From<Dialog<'a, Message>> for Element<'a, Message>
where
    Message: 'a + Clone,
{
    fn from(value: Dialog<'a, Message>) -> Self {
        let Dialog {
            style,
            icon: maybe_icon,
            icon_font,
            title,
            title_font,
            body,
            buttons,
            button_label_font,
            width,
            height,
            close,
            fullscreen,
            labels,
            tooltips_enabled,
        } = value;

        let Style {
            icon_color,
            title_color,
            container_color,
            container_border,
            elevation,
            shadow_color,
            close_button: close_button_style,
            tooltip_style,
        } = style;

        let icon_element =
            maybe_icon.map(|i| icon(i, ICON_SIZE).color(icon_color).font_maybe(icon_font));

        let title_element = title.map(|title| {
            iced_widget::text(title)
                .size(TITLE_SIZE)
                .color(title_color)
                .font_maybe(title_font)
        });

        let buttons_present = !buttons.is_empty();
        let buttons = row({
            let mut content = Vec::with_capacity(buttons.len() + 1);
            content.push(space().width(Length::Fill).into());
            content.extend(buttons.into_iter().map(|b| {
                crate::widget::button(b.style, button::Content::Label(b.label.into()))
                    .label_font_maybe(button_label_font)
                    .on_press_maybe(b.on_press)
                    .into()
            }));
            content
        })
        .spacing(BUTTON_SPACING);

        let surface: Element<'a, Message> = if fullscreen {
            let close_button = close.map(|on_press| {
                crate::widget::button(
                    close_button_style,
                    button::Content::Icon(Icon::Svg(common_icons::CLOSE.clone())),
                )
                .size(
                    button::Size::extra_small()
                        .width(Length::Fixed(CLOSE_BUTTON_SIZE))
                        .height(Length::Fixed(CLOSE_BUTTON_SIZE))
                        .padding(Padding::ZERO)
                        .icon_size(Pixels(CLOSE_ICON_SIZE)),
                )
                .on_press(on_press)
            });

            let mut header_items: Vec<Element<'a, Message>> = Vec::new();
            if let Some(icon_element) = icon_element {
                header_items.push(icon_element.into());
            }
            if let Some(title_element) = title_element {
                header_items.push(title_element.into());
            }
            header_items.push(space().width(Length::Fill).into());
            if let Some(close_button) = close_button {
                let close_button: Element<'a, Message> = if tooltips_enabled {
                    crate::widget::tooltip(tooltip_style, close_button)
                        .label(labels.close)
                        .into()
                } else {
                    close_button.into()
                };

                header_items.push(close_button);
            }

            let header = row(header_items)
                .width(Length::Fill)
                .align_y(Alignment::Center)
                .spacing(ICON_TITLE_SPACING);

            let mut content = column![header, space().height(TITLE_BODY_SPACING), body]
                .width(Length::Fill)
                .height(Length::Fill);

            if buttons_present {
                content = content.push(space().height(Length::Fill));
            }
            content = content.push(buttons);

            container(content)
                .padding(PADDING)
                .width(Length::Fill)
                .height(Length::Fill)
                .style(move |_| {
                    iced_widget::container::Style::default()
                        .background(container_color)
                        .border(Border::default().rounded(0.0))
                })
                .into()
        } else {
            let alignment = if icon_element.is_some() {
                Alignment::Center
            } else {
                Alignment::Start
            };

            let content = column![
                space().width(constants::MIN_WIDTH),
                icon_element.map(|icon| column![icon, space().height(ICON_TITLE_SPACING)]),
                title_element.map(|title| column![title, space().height(TITLE_BODY_SPACING)]),
                body,
                buttons_present.then_some(space().height(BODY_BUTTONS_SPACING)),
                buttons
            ]
            .align_x(alignment);

            let dialog = container(content)
                .padding(PADDING)
                .style(move |_| {
                    iced_widget::container::Style::default()
                        .background(container_color)
                        .border(container_border)
                        .shadow(crate::style::shadow(shadow_color, elevation))
                })
                .max_width(constants::MAX_WIDTH);

            let dialog = match width {
                Some(width) => {
                    dialog.width(width.clamp(constants::MIN_WIDTH, constants::MAX_WIDTH))
                }
                None => dialog,
            };

            let dialog = match height {
                Some(height) => dialog.height(height),
                None => dialog,
            };

            dialog.into()
        };

        surface
    }
}
