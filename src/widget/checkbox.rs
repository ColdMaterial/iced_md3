use iced::time::Instant;

use iced::{
    Border, Color, Element, Length, Point, Size, Vector,
    advanced::{
        Widget,
        layout::{self, Node},
        mouse,
        renderer::Quad,
    },
    border::Radius,
    widget::canvas,
};

use crate::{
    animation::{
        Interpolable,
        motion::{self, Spring, SpringValue, fast_effects, fast_spatial},
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

const CHECKBOX_SIZE: f32 = 18.0;
const TOUCH_TARGET_SIZE: f32 = 48.0;
const BORDER_RADIUS: f32 = 2.0;
const OUTLINE_WIDTH: f32 = 2.0;

const STATE_LAYER_SIZE: f32 = 40.0;
const DISABLED_OPACITY: f32 = crate::style::DISABLED_STATE_LAYER_OPACITY;

const DEFAULT_EFFECTS_SPRING: Spring = Spring::new(1.0, 1600.0);
const DEFAULT_SPATIAL_SPRING: Spring = Spring::new(0.9, 1000.0);

fn stroke_line<Renderer>(
    frame: &mut canvas::Frame<Renderer>,
    from: Point,
    to: Point,
    progress: f32,
    color: Color,
) where
    Renderer: iced::advanced::graphics::geometry::Renderer,
{
    let progress = progress.clamp(0.0, 1.0);
    if progress <= 0.0 {
        return;
    }

    let end = Point::new(
        from.x + (to.x - from.x) * progress,
        from.y + (to.y - from.y) * progress,
    );
    let path = canvas::Path::line(from, end);
    let stroke = canvas::Stroke::default()
        .with_color(color)
        .with_width(2.0)
        .with_line_cap(canvas::LineCap::Round);
    frame.stroke(&path, stroke);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Value {
    #[default]
    Unchecked,
    Checked,
    Indeterminate,
}

impl Value {
    pub fn is_checked(self) -> bool {
        matches!(self, Self::Checked)
    }

    pub fn is_unchecked(self) -> bool {
        matches!(self, Self::Unchecked)
    }

    pub fn is_indeterminate(self) -> bool {
        matches!(self, Self::Indeterminate)
    }

    pub fn is_selected(self) -> bool {
        matches!(self, Self::Checked | Self::Indeterminate)
    }

    #[must_use]
    /// Unchecked -> Checked
    pub fn toggle(self) -> Self {
        if self.is_checked() {
            Self::Unchecked
        } else {
            Self::Checked
        }
    }

    #[must_use]
    /// Unchecked -> Checked -> Indeterminate
    pub fn next(self) -> Self {
        match self {
            Self::Unchecked => Self::Checked,
            Self::Checked => Self::Indeterminate,
            Self::Indeterminate => Self::Unchecked,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StateStyle {
    pub background_color: Color,
    pub icon_color: Color,
    pub outline_width: f32,
    pub outline_color: Color,
}

impl Interpolable for StateStyle {
    fn interpolate(self, other: Self, t: f32) -> Self {
        Self {
            background_color: crossfade_color(self.background_color, other.background_color, t),
            icon_color: crossfade_color(self.icon_color, other.icon_color, t),
            outline_width: self.outline_width.interpolate(other.outline_width, t),
            outline_color: crossfade_color(self.outline_color, other.outline_color, t),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    pub state_layer_selected: StateLayer,
    pub state_layer_unselected: StateLayer,
    pub checked: StateStyle,
    pub checked_disabled: StateStyle,
    pub unchecked: StateStyle,
    pub unchecked_disabled: StateStyle,
    pub indeterminate: StateStyle,
    pub indeterminate_disabled: StateStyle,
}

impl Style {
    pub fn new(theme: &(impl ColorScheme + ?Sized)) -> Self {
        Self::with_error_flag(theme, false)
    }

    pub fn error(theme: &(impl ColorScheme + ?Sized)) -> Self {
        Self::with_error_flag(theme, true)
    }

    fn with_error_flag(theme: &(impl ColorScheme + ?Sized), is_error: bool) -> Self {
        let (selected_bg, selected_icon, unselected_outline) = if is_error {
            (theme.error(), theme.on_error(), theme.error())
        } else {
            (
                theme.primary(),
                theme.on_primary(),
                theme.on_surface_variant(),
            )
        };

        let unselected_outline_disabled = if is_error {
            theme.error().scale_alpha(DISABLED_OPACITY)
        } else {
            theme.on_surface().scale_alpha(DISABLED_OPACITY)
        };

        Self {
            state_layer_selected: StateLayer::new(selected_bg),
            state_layer_unselected: StateLayer::new(unselected_outline),
            checked: StateStyle {
                background_color: selected_bg,
                icon_color: selected_icon,
                outline_width: 0.0,
                outline_color: Color::TRANSPARENT,
            },
            checked_disabled: StateStyle {
                background_color: theme.on_surface().scale_alpha(DISABLED_OPACITY),
                icon_color: theme.surface().scale_alpha(DISABLED_OPACITY),
                outline_width: 0.0,
                outline_color: Color::TRANSPARENT,
            },
            unchecked: StateStyle {
                background_color: Color::TRANSPARENT,
                icon_color: Color::TRANSPARENT,
                outline_width: OUTLINE_WIDTH,
                outline_color: unselected_outline,
            },
            unchecked_disabled: StateStyle {
                background_color: Color::TRANSPARENT,
                icon_color: Color::TRANSPARENT,
                outline_width: OUTLINE_WIDTH,
                outline_color: unselected_outline_disabled,
            },
            indeterminate: StateStyle {
                background_color: selected_bg,
                icon_color: selected_icon,
                outline_width: 0.0,
                outline_color: Color::TRANSPARENT,
            },
            indeterminate_disabled: StateStyle {
                background_color: theme.on_surface().scale_alpha(DISABLED_OPACITY),
                icon_color: theme.surface().scale_alpha(DISABLED_OPACITY),
                outline_width: 0.0,
                outline_color: Color::TRANSPARENT,
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

    fn state(&self, value: Value, enabled: bool) -> &StateStyle {
        match (value, enabled) {
            (Value::Checked, true) => &self.checked,
            (Value::Checked, false) => &self.checked_disabled,
            (Value::Indeterminate, true) => &self.indeterminate,
            (Value::Indeterminate, false) => &self.indeterminate_disabled,
            (Value::Unchecked, true) => &self.unchecked,
            (Value::Unchecked, false) => &self.unchecked_disabled,
        }
    }
}

pub struct Checkbox<Message>
where
    Message: Clone,
{
    style: Style,
    value: Value,
    on_toggle: Option<Message>,
    effects_spring: Option<Spring>,
    spatial_spring: Option<Spring>,
    style_checked: bool,
}

impl<Message> Checkbox<Message>
where
    Message: Clone,
{
    pub fn new(theme: &(impl ColorScheme + ?Sized), value: Value) -> Self {
        Self {
            style: Style::new(theme),
            value,
            on_toggle: None,
            effects_spring: None,
            spatial_spring: None,
            style_checked: false,
        }
    }

    pub fn error(mut self, is_error: bool, theme: &(impl ColorScheme + ?Sized)) -> Self {
        self.style = Style::with_error_flag(theme, is_error);
        self
    }

    pub fn on_toggle(mut self, on_toggle: Message) -> Self {
        self.on_toggle = Some(on_toggle);
        self
    }

    crate::macros::maybe_assign!(on_toggle_maybe, on_toggle, Option<Message>);

    pub fn motion_scheme(mut self, scheme: motion::Scheme) -> Self {
        self.effects_spring = Some(fast_effects(scheme));
        self.spatial_spring = Some(fast_spatial(scheme));
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
    check: SpringValue<f32>,
    morph: SpringValue<f32>,
    style_spring: SpringValue<StateStyle>,
    last_effects_spring: Spring,
    last_spatial_spring: Spring,
    last_style: Style,
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for Checkbox<Message>
where
    Message: Clone,
    Renderer: iced_widget::renderer::geometry::Renderer,
{
    fn tag(&self) -> iced::advanced::widget::tree::Tag {
        iced::advanced::widget::tree::Tag::of::<State>()
    }

    fn state(&self) -> iced::advanced::widget::tree::State {
        let enabled = self.on_toggle.is_some();
        let target_style = *self.style.state(self.value, enabled);
        let now = Instant::now();
        let effects_spring = self.effects_spring.unwrap_or(DEFAULT_EFFECTS_SPRING);
        let spatial_spring = self.spatial_spring.unwrap_or(DEFAULT_SPATIAL_SPRING);
        let (check, morph) = match self.value {
            Value::Unchecked => (0.0, 0.0),
            Value::Checked => (1.0, 0.0),
            Value::Indeterminate => (1.0, 1.0),
        };

        let state = State {
            ripple: Ripple::new(),
            is_pressed: false,
            check: SpringValue::new(check, spatial_spring, now),
            morph: SpringValue::new(morph, spatial_spring, now),
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

        let checkbox_bounds = centered_square(bounds, CHECKBOX_SIZE);

        renderer.fill_quad(
            Quad {
                bounds: checkbox_bounds,
                border: Border::default()
                    .rounded(BORDER_RADIUS)
                    .color(style.outline_color)
                    .width(style.outline_width),
                ..Default::default()
            },
            style.background_color,
        );

        state.ripple.draw(
            renderer,
            centered_square(bounds, STATE_LAYER_SIZE),
            Radius::from(f32::MAX),
            *self.style.state_layer(self.value.is_selected()),
            self.on_toggle.is_some(),
        );

        let check = state.check.value().clamp(0.0, 1.0);
        let morph = state.morph.value().clamp(0.0, 1.0);

        if check <= 0.0 {
            return;
        }

        // Keep the icon fully opaque while it is being drawn or erased, even
        // as the fill itself fades. While the fill is retracting the value is
        // already `Unchecked`, but checked and indeterminate icons share the
        // same color, so falling back to `Checked` keeps the icon visible.
        let enabled = self.on_toggle.is_some();
        let icon_source = if self.value.is_selected() {
            self.value
        } else {
            Value::Checked
        };
        let icon_color = self.style.state(icon_source, enabled).icon_color;

        if icon_color.a <= 0.0 {
            return;
        }

        let mut frame = canvas::Frame::new(&*renderer, checkbox_bounds.size());

        // The checkmark's two strokes slide onto the dash as `morph` goes from
        // 0 (checkmark) to 1 (dash), so the icon morphs instead of cross-fading.
        let at = |checkmark: Point, dash: Point| {
            Point::new(
                checkmark.x + (dash.x - checkmark.x) * morph,
                checkmark.y + (dash.y - checkmark.y) * morph,
            )
        };

        let left_progress = (check * 2.0).clamp(0.0, 1.0);
        stroke_line(
            &mut frame,
            at(Point::new(3.62, 9.0), Point::new(3.0, 9.0)),
            at(Point::new(6.75, 12.13), Point::new(9.0, 9.0)),
            left_progress,
            icon_color,
        );

        let right_progress = ((check - 0.5) * 2.0).clamp(0.0, 1.0);
        stroke_line(
            &mut frame,
            at(Point::new(6.75, 12.13), Point::new(9.0, 9.0)),
            at(Point::new(15.75, 5.25), Point::new(15.0, 9.0)),
            right_progress,
            icon_color,
        );

        let geometry = frame.into_geometry();

        renderer.with_translation(
            Vector::new(checkbox_bounds.x, checkbox_bounds.y),
            |renderer| {
                renderer.draw_geometry(geometry);
            },
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
            self.on_toggle.is_some(),
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
        let enabled = self.on_toggle.is_some();

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
            &mut [&mut state.check.spring, &mut state.morph.spring],
        );

        let target_style = *self.style.state(self.value, enabled);

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

        state
            .check
            .set_target(if self.value.is_selected() { 1.0 } else { 0.0 }, now);

        // The icon morphs into the dash only while the checkbox is selected.
        // When it becomes unchecked the dash keeps its shape while it fades
        // out, so the morph target is left untouched.
        match self.value {
            Value::Checked => state.morph.set_target(0.0, now),
            Value::Indeterminate => state.morph.set_target(1.0, now),
            Value::Unchecked => {}
        }

        if !state.check.is_at_rest()
            || !state.morph.is_at_rest()
            || !state.style_spring.is_at_rest()
        {
            shell.request_redraw();
        }

        state.check.step(now);
        state.morph.step(now);
        state.style_spring.step(now);

        if state.check.is_at_rest()
            && state.morph.is_at_rest()
            && state.style_spring.is_at_rest()
            && self.value.is_unchecked()
            && state.morph.to != 0.0
        {
            // The icon is fully hidden, so re-arm the morph for the next time
            // the checkbox is checked without animating anything visible.
            state.morph.reset(0.0, now);
        }

        interaction::toggle_press(
            event,
            &mut state.is_pressed,
            is_over,
            enabled,
            self.on_toggle.as_ref(),
            shell,
        );
    }
}

crate::macros::impl_element_from!(
    ['a, Message] Checkbox<Message> => Element<'a, Message>
    where Message: 'a + Clone,
);
