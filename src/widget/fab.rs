use iced::{Element, Font, Length, Pixels, advanced::text, border::Radius, padding};
use iced_widget::text::IntoFragment;

pub const EDGE_SPACING: f32 = 16.0;

use crate::{
    style::Elevation,
    widget::{
        button::{self, CornerStyle},
        hybrid_icon::Icon,
        tooltip,
    },
};

pub type Style = button::Style;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Size {
    #[default]
    Regular,
    Medium,
    Large,
}

impl Size {
    pub(crate) fn container_height(&self) -> f32 {
        match self {
            Size::Regular => 56.0,
            Size::Medium => 80.0,
            Size::Large => 96.0,
        }
    }

    pub(crate) fn content_spacing(&self) -> f32 {
        match self {
            Size::Regular => 8.0,
            Size::Medium => 12.0,
            Size::Large => 16.0,
        }
    }

    pub(crate) fn label_size(&self) -> Pixels {
        Pixels(match self {
            Size::Regular => 16.0,
            Size::Medium => 22.0,
            Size::Large => 24.0,
        })
    }

    pub(crate) fn padding(&self) -> f32 {
        match self {
            Size::Regular => 16.0,
            Size::Medium => 26.0,
            Size::Large => 28.0,
        }
    }

    pub(crate) fn icon_size(&self) -> Pixels {
        Pixels(match self {
            Size::Regular => 24.0,
            Size::Medium => 28.0,
            Size::Large => 36.0,
        })
    }

    pub(crate) fn rounding(&self) -> Radius {
        match self {
            Size::Regular => 16.0,
            Size::Medium => 20.0,
            Size::Large => 28.0,
        }
        .into()
    }

    fn button_size(
        &self,
        extended: bool,
        corner_style: CornerStyle,
        label_size_scale: f32,
    ) -> button::Size {
        let (padding, container_width) = match extended {
            true => (self.padding(), Length::Shrink),
            false => (0.0, Length::Fixed(self.container_height())),
        };
        button::Size {
            width: container_width,
            height: Length::Fixed(self.container_height()),
            spacing: Pixels(self.content_spacing()),
            padding: padding::horizontal(padding),
            icon_size: self.icon_size(),
            label_size: Pixels(self.label_size().0 * label_size_scale),
            corner_radius: button::CornerRadius {
                style: corner_style,
                shape_morph: false,
                rounded: f32::MAX.into(),
                square: self.rounding(),
                pressed: 0.0.into(),
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tooltip {
    pub style: tooltip::Style,
    pub position: tooltip::Position,
}

#[derive(Clone)]
pub enum Content<'a> {
    Reguar {
        icon: Icon<'a>,
    },
    Extended {
        icon: Icon<'a>,
        label: text::Fragment<'a>,
    },
}

impl<'a> Content<'a> {
    fn icon(&'a self) -> &'a Icon<'a> {
        match self {
            Content::Reguar { icon } => icon,
            Content::Extended { icon, label: _ } => icon,
        }
    }

    pub fn extended(icon: Icon<'a>, label: text::Fragment<'a>) -> Self {
        Content::Extended { icon, label }
    }

    pub fn regular(icon: Icon<'a>) -> Self {
        Content::Reguar { icon }
    }

    pub fn with_label(&'a mut self, label: impl IntoFragment<'a>) -> Self {
        Self::Extended {
            icon: self.icon().clone(),
            label: label.into_fragment(),
        }
    }
}

impl<'a> From<Content<'a>> for button::Content<'a> {
    fn from(value: Content<'a>) -> Self {
        match value {
            Content::Reguar { icon } => button::Content::Icon(icon),
            Content::Extended { icon, label } => button::Content::Full { icon, label },
        }
    }
}

pub struct Fab<'a, Message>
where
    Message: 'a + Clone,
{
    pub content: Content<'a>,
    pub size: Size,
    pub corner_style: CornerStyle,
    pub style: Style,
    pub label_font: Option<Font>,
    pub label_size_scale: f32,
    pub on_press: Message,
    /// Optional tooltip shown when the FAB is hovered.
    pub tooltip: Option<(text::Fragment<'a>, Tooltip)>,
}

impl<'a, Message> Fab<'a, Message>
where
    Message: 'a + Clone,
{
    #[must_use]
    pub fn new(style: Style, content: Content<'a>, on_press: Message) -> Self {
        Self {
            content,
            size: Size::default(),
            corner_style: CornerStyle::Square,
            style,
            label_font: None,
            label_size_scale: 1.0,
            on_press,
            tooltip: None,
        }
    }

    #[must_use]
    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }

    crate::macros::maybe_delegate!(size_maybe, size, Option<Size>);

    #[must_use]
    pub fn label_font(mut self, label_font: Font) -> Self {
        self.label_font = Some(label_font);
        self
    }

    crate::macros::maybe_assign!(label_font_maybe, label_font, Option<Font>);

    #[must_use]
    pub fn corner_style(mut self, corner_style: CornerStyle) -> Self {
        self.corner_style = corner_style;
        self
    }

    #[must_use]
    pub fn label_size_scale(mut self, scale: f32) -> Self {
        self.label_size_scale = scale;
        self
    }

    /// Shows a tooltip with `label` when the [`Fab`] is hovered.
    #[must_use]
    pub fn tooltip(mut self, label: impl IntoFragment<'a>, config: Tooltip) -> Self {
        self.tooltip = Some((label.into_fragment(), config));
        self
    }

    pub fn set_label_size_scale(&mut self, scale: f32) {
        self.label_size_scale = scale;
    }
}

impl<'a, Message: 'a + Clone> Clone for Fab<'a, Message> {
    fn clone(&self) -> Self {
        Self {
            content: self.content.clone(),
            size: self.size,
            corner_style: self.corner_style,
            style: self.style,
            label_font: self.label_font,
            label_size_scale: self.label_size_scale,
            on_press: self.on_press.clone(),
            tooltip: self.tooltip.clone(),
        }
    }
}

impl<'a, Message, Theme, Renderer> From<Fab<'a, Message>> for Element<'a, Message, Theme, Renderer>
where
    Message: 'a + Clone,
    Theme: 'a,
    Renderer: 'a
        + iced::advanced::text::Renderer
        + iced::advanced::svg::Renderer
        + iced_widget::renderer::geometry::Renderer,
    Renderer::Font: From<iced::Font>,
{
    fn from(value: Fab<'a, Message>) -> Self {
        let is_extended_fab = matches!(value.content, Content::Extended { icon: _, label: _ });
        let button = crate::widget::button(value.style, value.content.into())
            .label_font_maybe(value.label_font)
            .size(value.size.button_size(
                is_extended_fab,
                value.corner_style,
                value.label_size_scale,
            ))
            .elevation(Elevation::Level3);

        let button: Element<'a, Message, Theme, Renderer> = button.on_press(value.on_press).into();

        if let Some((label, config)) = value.tooltip {
            crate::widget::tooltip(config.style, button)
                .label(label)
                .position(config.position)
                .into()
        } else {
            button
        }
    }
}
