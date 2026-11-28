//! Radio buttons let people select a single option from a set.
//!
//! The inner dot grows from the center of the radio to its edge when the
//! radio is selected and shrinks back in reverse when it is deselected.

use iced::time::Instant;

use iced::{
    Border, Color, Element, Length, Rectangle, Size,
    advanced::{
        Widget,
        layout::{self, Node},
        mouse,
        renderer::Quad,
    },
    border::Radius,
};

use crate::{
    animation::{
        Interpolable,
        motion::{self, Spring, SpringValue},
    },
    style::{StateLayer, crossfade_color},
    theme::ColorScheme,
    widget::{
        internal::{
            geometry::centered_square,
            interaction,
            motion::{sync_spring_model, sync_style_spring},
        },
        ripple::Ripple,
    },
};

const RADIO_SIZE: f32 = 20.0;
const TOUCH_TARGET_SIZE: f32 = 48.0;
const OUTLINE_WIDTH: f32 = 2.0;
/// Diameter of the selected dot.
const DOT_SIZE: f32 = 10.0;

const STATE_LAYER_SIZE: f32 = 40.0;
const DISABLED_OPACITY: f32 = crate::style::DISABLED_STATE_LAYER_OPACITY;

/// Springs used by the radio when no explicit one is provided.
///
/// The expressive scheme is the default everywhere, so the dot overshoots
/// slightly and the fill from the center stays visible instead of looking
/// instant.
const DEFAULT_MOTION_SCHEME: motion::Scheme = motion::Scheme::Expressive;

/// The colors of a radio for a specific selection state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StateStyle {
    pub outline_color: Color,
    pub outline_width: f32,
    pub dot_color: Color,
}

impl Interpolable for StateStyle {
    fn interpolate(self, other: Self, t: f32) -> Self {
        Self {
            outline_color: crossfade_color(self.outline_color, other.outline_color, t),
            outline_width: self.outline_width.interpolate(other.outline_width, t),
            dot_color: crossfade_color(self.dot_color, other.dot_color, t),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    pub state_layer_selected: StateLayer,
    pub state_layer_unselected: StateLayer,
    pub selected: StateStyle,
    pub selected_disabled: StateStyle,
    pub unselected: StateStyle,
    pub unselected_disabled: StateStyle,
}

impl Style {
    pub fn new(theme: &(impl ColorScheme + ?Sized)) -> Self {
        Self::with_error_flag(theme, false)
    }

    pub fn error(theme: &(impl ColorScheme + ?Sized)) -> Self {
        Self::with_error_flag(theme, true)
    }

    fn with_error_flag(theme: &(impl ColorScheme + ?Sized), is_error: bool) -> Self {
        let (enabled_dot, unselected_outline) = if is_error {
            (theme.error(), theme.error())
        } else {
            (theme.primary(), theme.on_surface_variant())
        };
        let disabled = theme.on_surface().scale_alpha(DISABLED_OPACITY);

        Self {
            state_layer_selected: StateLayer::new(enabled_dot),
            state_layer_unselected: StateLayer::new(unselected_outline),
            selected: StateStyle {
                outline_color: enabled_dot,
                outline_width: OUTLINE_WIDTH,
                dot_color: enabled_dot,
            },
            selected_disabled: StateStyle {
                outline_color: disabled,
                outline_width: OUTLINE_WIDTH,
                dot_color: disabled,
            },
            unselected: StateStyle {
                outline_color: unselected_outline,
                outline_width: OUTLINE_WIDTH,
                dot_color: Color::TRANSPARENT,
            },
            unselected_disabled: StateStyle {
                outline_color: disabled,
                outline_width: OUTLINE_WIDTH,
                dot_color: Color::TRANSPARENT,
            },
        }
    }

    fn state_layer(&self, selected: bool) -> &StateLayer {
        if selected {
            &self.state_layer_selected
        } else {
            &self.state_layer_unselected
        }
    }

    fn state(&self, selected: bool, enabled: bool) -> &StateStyle {
        match (selected, enabled) {
            (true, true) => &self.selected,
            (true, false) => &self.selected_disabled,
            (false, true) => &self.unselected,
            (false, false) => &self.unselected_disabled,
        }
    }
}

pub struct Radio<Message>
where
    Message: Clone,
{
    style: Style,
    selected: bool,
    on_select: Option<Message>,
    effects_spring: Option<Spring>,
    spatial_spring: Option<Spring>,
    style_checked: bool,
}

impl<Message> Radio<Message>
where
    Message: Clone,
{
    pub fn new(theme: &(impl ColorScheme + ?Sized), selected: bool) -> Self {
        Self {
            style: Style::new(theme),
            selected,
            on_select: None,
            effects_spring: None,
            spatial_spring: None,
            style_checked: false,
        }
    }

    /// Switches the radio to the error colour scheme.
    pub fn error(mut self, is_error: bool, theme: &(impl ColorScheme + ?Sized)) -> Self {
        self.style = Style::with_error_flag(theme, is_error);
        self
    }

    /// Sets the message produced when the radio is selected.
    pub fn on_select(mut self, on_select: Message) -> Self {
        self.on_select = Some(on_select);
        self
    }

    crate::macros::maybe_assign!(on_select_maybe, on_select, Option<Message>);

    pub fn motion_scheme(mut self, scheme: motion::Scheme) -> Self {
        self.effects_spring = Some(motion::default_effects(scheme));
        self.spatial_spring = Some(motion::default_spatial(scheme));
        self
    }

    pub fn effects_spring(mut self, spring: Spring) -> Self {
        self.effects_spring = Some(spring);
        self
    }

    pub fn spatial_spring(mut self, spring: Spring) -> Self {
        self.spatial_spring = Some(spring);
        self
    }
}

struct State {
    ripple: Ripple,
    is_pressed: bool,
    fill: SpringValue<f32>,
    style_spring: SpringValue<StateStyle>,
    last_effects_spring: Spring,
    last_spatial_spring: Spring,
    last_style: Style,
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for Radio<Message>
where
    Message: Clone,
    Renderer: iced_widget::renderer::geometry::Renderer,
{
    fn tag(&self) -> iced::advanced::widget::tree::Tag {
        iced::advanced::widget::tree::Tag::of::<State>()
    }

    fn state(&self) -> iced::advanced::widget::tree::State {
        let enabled = self.on_select.is_some();
        let target_style = *self.style.state(self.selected, enabled);
        let now = Instant::now();
        let effects_spring = self
            .effects_spring
            .unwrap_or_else(|| motion::default_effects(DEFAULT_MOTION_SCHEME));
        let spatial_spring = self
            .spatial_spring
            .unwrap_or_else(|| motion::default_spatial(DEFAULT_MOTION_SCHEME));

        let state = State {
            ripple: Ripple::new(),
            is_pressed: false,
            fill: SpringValue::new(f32::from(self.selected), spatial_spring, now),
            style_spring: SpringValue::new(target_style, effects_spring, now),
            last_effects_spring: effects_spring,
            last_spatial_spring: spatial_spring,
            last_style: self.style,
        };

        iced::advanced::widget::tree::State::new(state)
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: Length::Fixed(TOUCH_TARGET_SIZE),
            height: Length::Fixed(TOUCH_TARGET_SIZE),
        }
    }

    fn layout(
        &mut self,
        _tree: &mut iced::advanced::widget::Tree,
        _renderer: &Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> Node {
        layout::atomic(limits, TOUCH_TARGET_SIZE, TOUCH_TARGET_SIZE)
    }

    fn draw(
        &self,
        tree: &iced::advanced::widget::Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &iced::advanced::renderer::Style,
        layout: iced::advanced::Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &iced::Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State>();
        let style = state.style_spring.value();
        let bounds = layout.bounds();

        let radio_bounds = centered_square(bounds, RADIO_SIZE);

        renderer.fill_quad(
            Quad {
                bounds: radio_bounds,
                border: Border::default()
                    .rounded(RADIO_SIZE / 2.0)
                    .color(style.outline_color)
                    .width(style.outline_width),
                ..Default::default()
            },
            Color::TRANSPARENT,
        );

        state.ripple.draw(
            renderer,
            centered_square(bounds, STATE_LAYER_SIZE),
            Radius::from(f32::MAX),
            *self.style.state_layer(self.selected),
            self.on_select.is_some(),
        );

        let fill = state.fill.value().clamp(0.0, 1.0);

        if fill <= 0.0 {
            return;
        }

        // The dot grows from the center outwards, so selecting fills the radio
        // towards its edge and deselecting retracts it back to the center.
        let dot_size = DOT_SIZE * fill;
        let dot_bounds = Rectangle {
            x: radio_bounds.x + (RADIO_SIZE - dot_size) / 2.0,
            y: radio_bounds.y + (RADIO_SIZE - dot_size) / 2.0,
            width: dot_size,
            height: dot_size,
        };

        renderer.fill_quad(
            Quad {
                bounds: dot_bounds,
                border: Border::default().rounded(dot_size / 2.0),
                ..Default::default()
            },
            style.dot_color,
        );
    }

    fn mouse_interaction(
        &self,
        _tree: &iced::advanced::widget::Tree,
        layout: iced::advanced::Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &iced::Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        interaction::pointer_when(
            self.on_select.is_some(),
            cursor,
            layout.bounds(),
            mouse::Interaction::Pointer,
        )
    }

    fn update(
        &mut self,
        tree: &mut iced::advanced::widget::Tree,
        event: &iced::Event,
        layout: iced::advanced::Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn iced::advanced::Clipboard,
        shell: &mut iced::advanced::Shell<'_, Message>,
        _viewport: &iced::Rectangle,
    ) {
        let state = tree.state.downcast_mut::<State>();
        let bounds = layout.bounds();
        let is_over = cursor.is_over(bounds);
        let now = Instant::now();
        let enabled = self.on_select.is_some();

        let layer_bounds = centered_square(bounds, STATE_LAYER_SIZE);
        state
            .ripple
            .handle_in(event, bounds, layer_bounds, cursor, enabled, shell);

        sync_spring_model(
            &mut state.last_effects_spring,
            self.effects_spring,
            &mut [&mut state.style_spring.spring],
        );
        sync_spring_model(
            &mut state.last_spatial_spring,
            self.spatial_spring,
            &mut [&mut state.fill.spring],
        );

        let target_style = *self.style.state(self.selected, enabled);

        if sync_style_spring(
            &mut state.style_spring,
            &mut self.style_checked,
            &mut state.last_style,
            self.style,
            target_style,
            now,
        ) {
            shell.request_redraw();
        }

        state.fill.set_target(f32::from(self.selected), now);

        if !state.fill.is_at_rest() || !state.style_spring.is_at_rest() {
            shell.request_redraw();
        }

        state.fill.step(now);
        state.style_spring.step(now);

        interaction::toggle_press(
            event,
            &mut state.is_pressed,
            is_over,
            enabled,
            self.on_select.as_ref(),
            shell,
        );
    }
}

crate::macros::impl_element_from!(
    ['a, Message] Radio<Message> => Element<'a, Message>
    where Message: 'a + Clone,
);
