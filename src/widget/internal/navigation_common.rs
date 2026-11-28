use iced::time::Instant;

use iced::{
    Border, Color, Element, Event, Font, Pixels, Rectangle, Size,
    advanced::{
        Shell,
        renderer::Quad,
        svg,
        text::{self},
    },
    alignment::Vertical,
    mouse, touch,
};
use iced_widget::{core::Svg, space};

use crate::{
    animation::{
        Interpolable,
        motion::{self, Spring, SpringValue, default_effects, fast_effects},
    },
    style::StateLayer,
    theme::ColorScheme,
    widget::{self, Badge, hybrid_icon::Icon, ripple::Ripple},
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NavigationItemStyle {
    pub content_active: Color,
    pub label_content_active: Color,
    pub content_inactive: Color,
    pub container_color: Color,
    pub badge_style: widget::badge::Style,
    pub state_layer: StateLayer,
}

impl NavigationItemStyle {
    pub fn new(theme: &(impl ColorScheme + ?Sized)) -> Self {
        Self {
            content_active: theme.secondary(),
            label_content_active: theme.secondary(),
            content_inactive: theme.on_surface_variant(),
            container_color: theme.secondary_container(),
            badge_style: widget::badge::Style::new(theme),
            state_layer: StateLayer::new(theme.on_secondary_container()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct NavigationItemContent<'a, Message>
where
    Message: Clone,
{
    pub icon_active: Icon<'a>,
    pub icon_inactive: Icon<'a>,
    pub badge: Option<Badge<'a>>,
    pub label: text::Fragment<'a>,
    pub on_press: Message,
}

impl<'a, Message> NavigationItemContent<'a, Message>
where
    Message: Clone,
{
    pub fn icon(&self, active: bool) -> &Icon<'a> {
        match active {
            true => &self.icon_active,
            false => &self.icon_inactive,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct Colors {
    pub icon: Color,
    pub label: Color,
}

impl Interpolable for Colors {
    fn interpolate(self, other: Self, t: f32) -> Self {
        Self {
            icon: self.icon.interpolate(other.icon, t),
            label: self.label.interpolate(other.label, t),
        }
    }
}

pub fn colors_for(active: bool, style: &NavigationItemStyle) -> Colors {
    match active {
        true => Colors {
            icon: style.content_active,
            label: style.label_content_active,
        },
        false => Colors {
            icon: style.content_inactive,
            label: style.content_inactive,
        },
    }
}

pub struct State {
    pub is_pressed: bool,
    pub ripple: Ripple,
    pub active_transition_spring: SpringValue<f32>,
    pub color_spring: SpringValue<Colors>,
}

impl State {
    #[must_use]
    pub fn new(
        active: bool,
        colors: Colors,
        active_spring: Option<Spring>,
        color_spring: Option<Spring>,
        now: Instant,
    ) -> Self {
        let progress = active as usize as f32;
        let mut state = Self {
            is_pressed: false,
            ripple: Ripple::new(),
            active_transition_spring: SpringValue::new(
                progress,
                active_spring.unwrap_or_else(|| default_effects(motion::Scheme::default())),
                now,
            ),
            color_spring: SpringValue::new(
                colors,
                color_spring.unwrap_or_else(|| fast_effects(motion::Scheme::default())),
                now,
            ),
        };
        state.active_transition_spring.reset(progress, now);
        state.color_spring.reset(colors, now);
        state
    }
}

pub fn badge_children<'a, Message, Theme, Renderer>(
    badge: &Option<Element<'a, Message, Theme, Renderer>>,
) -> Vec<iced::advanced::widget::Tree>
where
    Renderer: iced::advanced::Renderer,
{
    match badge {
        Some(badge) => vec![iced::advanced::widget::Tree::new(badge.as_widget())],
        None => Vec::new(),
    }
}

pub fn diff_badge<'a, Message, Theme, Renderer>(
    tree: &mut iced::advanced::widget::Tree,
    badge: &Option<Element<'a, Message, Theme, Renderer>>,
) where
    Renderer: iced::advanced::Renderer,
{
    let children = match badge {
        Some(badge) => vec![badge.as_widget()],
        None => Vec::new(),
    };

    tree.diff_children(&children);
}

/// Builds the optional badge element shared by navigation items.
pub fn build_badge<'a, Message, Theme, Renderer>(
    style: &NavigationItemStyle,
    badge: Option<&Badge<'a>>,
    icon_size: f32,
) -> Option<Element<'a, Message, Theme, Renderer>>
where
    Message: 'a + Clone,
    Theme: 'a,
    Renderer: 'a + text::Renderer,
{
    badge.map(|badge| {
        widget::badge(
            style.badge_style,
            space().width(icon_size).height(icon_size),
        )
        .label_maybe(badge.label.clone())
        .into()
    })
}

/// Creates a single-line paragraph with the navigation items' common text
/// options (`Advanced` shaping, no wrapping, left alignment).
pub fn paragraph<Renderer>(
    renderer: &Renderer,
    content: &str,
    bounds: Size,
    size: Pixels,
    line_height: text::LineHeight,
    font: Option<Font>,
    align_y: Vertical,
) -> Renderer::Paragraph
where
    Renderer: text::Renderer,
    Renderer::Font: From<Font>,
{
    super::text::paragraph(
        renderer,
        content,
        bounds,
        size,
        line_height,
        font.map(Renderer::Font::from),
        align_y,
    )
}

/// Creates the paragraph cache for a text icon, or `None` for SVG icons.
pub fn icon_paragraph<Renderer>(
    renderer: &Renderer,
    icon: &Icon<'_>,
    size: f32,
) -> Option<Renderer::Paragraph>
where
    Renderer: text::Renderer,
    Renderer::Font: From<Font>,
{
    match icon {
        Icon::Text { text, font } => Some(paragraph(
            renderer,
            text,
            Size::INFINITE,
            Pixels(size),
            text::LineHeight::Absolute(Pixels(size)),
            *font,
            Vertical::Center,
        )),
        Icon::Svg(_) => None,
    }
}

/// Draws the active indicator pill, scaled from 0 to 1 by `progress`.
pub fn draw_indicator<R: iced::advanced::Renderer>(
    renderer: &mut R,
    bounds: Rectangle,
    progress: f32,
    color: Color,
) {
    let width = progress * bounds.width;
    if width <= 0.0 {
        return;
    }

    let bounds = Rectangle {
        x: bounds.x + (bounds.width - width) / 2.0,
        width,
        ..bounds
    };

    renderer.fill_quad(
        Quad {
            bounds,
            border: Border::default().rounded(f32::MAX),
            ..Default::default()
        },
        color.scale_alpha(progress),
    );
}

/// Draws an icon from its cached text paragraph, falling back to the SVG
/// handle.
pub fn draw_icon<R>(
    renderer: &mut R,
    paragraph: Option<&R::Paragraph>,
    icon: &Icon<'_>,
    bounds: Rectangle,
    color: Color,
) where
    R: text::Renderer + svg::Renderer,
{
    if let Some(paragraph) = paragraph {
        renderer.fill_paragraph(paragraph, bounds.position(), color, bounds);
    } else if let Icon::Svg(handle) = icon {
        renderer.draw_svg(Svg::new(handle.clone()).color(color), bounds, bounds);
    }
}

pub fn draw_label<R: text::Renderer>(
    renderer: &mut R,
    paragraph: &R::Paragraph,
    bounds: Rectangle,
    color: Color,
) {
    renderer.fill_paragraph(paragraph, bounds.position(), color, bounds);
}

/// Handles the shared press/release interaction of navigation items.
pub fn handle_press<Message>(
    event: &Event,
    state: &mut State,
    ripple_bounds: Rectangle,
    hit_bounds: Rectangle,
    cursor: mouse::Cursor,
    on_press: &Message,
    shell: &mut Shell<'_, Message>,
) where
    Message: Clone,
{
    let is_hovered = cursor.is_over(hit_bounds);
    state
        .ripple
        .handle(event, ripple_bounds, cursor, true, shell);

    match event {
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
        | Event::Touch(touch::Event::FingerPressed { .. }) => {
            if is_hovered {
                state.is_pressed = true;
                shell.request_redraw();
            }
        }
        Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
        | Event::Touch(touch::Event::FingerLifted { .. }) => {
            if is_hovered && state.is_pressed {
                shell.publish(on_press.clone());
            }
            state.is_pressed = false;
            shell.request_redraw();
        }
        Event::Touch(touch::Event::FingerLost { .. }) => {
            state.is_pressed = false;
            shell.request_redraw();
        }
        _ => {}
    }
}

/// Advances the color and active-indicator springs of a navigation item.
pub fn update_springs<Message>(
    event: &Event,
    state: &mut State,
    active: bool,
    style: &NavigationItemStyle,
    active_spring: Option<Spring>,
    color_spring: Option<Spring>,
    shell: &mut Shell<'_, Message>,
) {
    state
        .active_transition_spring
        .set_model_if_some(active_spring);
    state.color_spring.set_model_if_some(color_spring);

    let now = match event {
        Event::Window(iced::window::Event::RedrawRequested(at)) => *at,
        _ => Instant::now(),
    };

    state
        .color_spring
        .set_target(colors_for(active, style), now);
    state
        .active_transition_spring
        .set_target(active as usize as f32, now);

    if !state.active_transition_spring.is_at_rest() || !state.color_spring.is_at_rest() {
        shell.request_redraw();
    }
    state.active_transition_spring.step(now);
    state.color_spring.step(now);
}
