use std::{fmt::Display, ops::RangeInclusive};

use iced::{
    Border, Color, Element, Event, Length, Point, Rectangle,
    advanced::{
        Overlay, Widget, layout, mouse, overlay,
        renderer::Quad,
        widget::{Id, Tree, operation},
    },
    border::Radius,
    keyboard::{self, Key, key},
    time::Instant,
    touch,
};
use iced_widget::core::{
    Svg, alignment,
    text::{self, Text, paragraph},
};

use crate::{
    animation::{
        Interpolable,
        motion::{self, Spring, SpringValue, fast_effects, fast_spatial},
    },
    style::StateLayer,
    theme::ColorScheme,
    widget::{
        hybrid_icon::Icon,
        internal::{geometry::expanded_bounds, motion::sync_spring_model},
        ripple::{Ripple, visible_clip},
    },
};

#[allow(clippy::module_inception)]
pub mod slider;
pub use slider::RangeSlider;

const HANDLE_WIDTH_IDLE: f32 = 4.0;
const HANDLE_WIDTH_DRAGGED: f32 = 2.0;
const HANDLE_GAP: f32 = 6.0;

const HANDLE_OVERLAP_WIDTH: f32 = 2.0;
const INNER_CORNER_RADIUS: f32 = 2.0;
const STOP_INDICATOR_SIZE: f32 = 4.0;
const STOP_INDICATOR_TRAILING_SPACE: f32 = 4.0;

const STOP_INDICATOR_MIN_SEGMENT: f32 = 12.0;

const STATE_LAYER_MIN_SIZE: f32 = 48.0;
const FOCUS_RING_MIN_SIZE: f32 = 48.0;
const FOCUS_RING_WIDTH: f32 = 2.0;

const LABEL_MIN_HEIGHT: f32 = 32.0;
const LABEL_HORIZONTAL_PADDING: f32 = 10.0;
const LABEL_TEXT_SIZE: f32 = 12.0;
const LABEL_GAP: f32 = 8.0;

const LABEL_TEXT_FADE_START: f32 = 0.85;

const DISABLED_ACTIVE_OPACITY: f32 = crate::style::DISABLED_STATE_LAYER_OPACITY;
const DISABLED_INACTIVE_OPACITY: f32 = crate::style::PRESSED_STATE_LAYER_OPACITY;
const DISABLED_INDICATOR_OPACITY: f32 = crate::style::DISABLED_STATE_LAYER_OPACITY;

const DRAG_CHASE_DECAY: f32 = 0.008;

/// Velocity kick (in normalized spring units per second) added when a dragged
/// handle settles onto an interior value. It only produces a slight overshoot —
/// a visual wobble — and never changes the value itself.
const DRAG_RELEASE_IMPULSE: f32 = 32.0;

// Steps closer together than this many pixels make dragging follow the
// cursor directly; coarser steps animate the handle to each snapped value.
const MIN_PIXELS_PER_STEP: f64 = 12.0;

pub struct Size {
    pub track_height: f32,
    pub handle_height: f32,
    pub corner_radius: f32,
}

impl Size {
    pub fn extra_small() -> Self {
        Self {
            track_height: 16.0,
            handle_height: 44.0,
            corner_radius: 8.0,
        }
    }

    pub fn small() -> Self {
        Self {
            track_height: 24.0,
            handle_height: 44.0,
            corner_radius: 8.0,
        }
    }

    pub fn medium() -> Self {
        Self {
            track_height: 40.0,
            handle_height: 52.0,
            corner_radius: 12.0,
        }
    }

    pub fn large() -> Self {
        Self {
            track_height: 56.0,
            handle_height: 68.0,
            corner_radius: 16.0,
        }
    }

    pub fn extra_large() -> Self {
        Self {
            track_height: 96.0,
            handle_height: 108.0,
            corner_radius: 28.0,
        }
    }
}

impl Default for Size {
    fn default() -> Self {
        Self::extra_small()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Axis {
    #[default]
    Horizontal,
    Vertical,
}

pub enum ValueIndicator<'a, T> {
    None,
    Value,
    Format(Box<dyn Fn(T) -> String + 'a>),
}

impl<'a, T> ValueIndicator<'a, T> {
    fn format(&self, value: T, decimal_places: usize, min_decimal_places: usize) -> Option<String>
    where
        T: Display + Into<f64>,
    {
        match self {
            ValueIndicator::None => None,
            ValueIndicator::Value => Some(format_decimal(
                value.into(),
                decimal_places,
                min_decimal_places,
            )),
            ValueIndicator::Format(format) => Some(format(value)),
        }
    }
}

fn format_decimal(value: f64, decimal_places: usize, min_decimal_places: usize) -> String {
    let formatted = if value.abs() < 0.5 / 10_f64.powi(decimal_places as i32) {
        format!("{0:.1$}", 0.0, decimal_places)
    } else {
        format!("{value:.decimal_places$}")
    };
    trim_trailing_zeros(formatted, min_decimal_places)
}

fn trim_trailing_zeros(formatted: String, min_decimal_places: usize) -> String {
    if let Some(dot_pos) = formatted.find('.') {
        let integer = &formatted[..dot_pos];
        let fraction = &formatted[dot_pos + 1..];
        let trimmed = fraction.trim_end_matches('0');
        let kept_len = trimmed.len().max(min_decimal_places).min(fraction.len());
        if kept_len > 0 {
            format!("{integer}.{}", &fraction[..kept_len])
        } else {
            integer.to_string()
        }
    } else {
        formatted
    }
}

fn step_decimal_places(step: f64) -> usize {
    const EPS: f64 = 1e-9;
    if step <= 0.0 {
        return 1;
    }
    let mut f = step.fract().abs();
    if f < EPS {
        return 0;
    }
    let mut count = 0;
    while f > EPS && count < 10 {
        f = (f * 10.0).fract();
        count += 1;
    }
    count
}

pub struct InsetIcon<'a> {
    pub on_active: Icon<'a>,

    pub on_inactive: Icon<'a>,
}

impl<'a> InsetIcon<'a> {
    #[must_use]
    pub fn new(icon: Icon<'a>) -> Self {
        Self {
            on_active: icon.clone(),
            on_inactive: icon,
        }
    }

    #[must_use]
    pub fn dual(on_active: Icon<'a>, on_inactive: Icon<'a>) -> Self {
        Self {
            on_active,
            on_inactive,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    pub active_track: Color,
    pub inactive_track: Color,
    pub handle: Color,
    pub stop_indicator: Color,
    pub stop_indicators_active: Color,
    pub stop_indicators_inactive: Color,
    pub label_container: Color,
    pub label_text: Color,
    pub state_layer: StateLayer,
    pub focus_ring: Color,
    pub icon_active: Color,
    pub icon_inactive: Color,

    pub handle_overlap_outline: Color,
    pub active_track_disabled: Color,
    pub inactive_track_disabled: Color,
    pub handle_disabled: Color,
    pub stop_indicators_disabled: Color,
    pub icon_disabled: Color,
}

impl Style {
    #[must_use]
    pub fn new(theme: &(impl ColorScheme + ?Sized)) -> Self {
        Self {
            active_track: theme.primary(),
            inactive_track: theme.surface_container_highest(),
            handle: theme.primary(),
            stop_indicator: theme.primary(),
            stop_indicators_active: theme.surface_container_highest(),
            stop_indicators_inactive: theme.primary(),
            label_container: theme.primary(),
            label_text: theme.on_primary(),
            state_layer: StateLayer::new(theme.primary()),
            focus_ring: theme.primary(),
            icon_active: theme.on_primary(),
            icon_inactive: theme.on_surface_variant(),
            handle_overlap_outline: theme.on_primary(),
            active_track_disabled: theme.on_surface().scale_alpha(DISABLED_ACTIVE_OPACITY),
            inactive_track_disabled: theme.on_surface().scale_alpha(DISABLED_INACTIVE_OPACITY),
            handle_disabled: theme.on_surface().scale_alpha(DISABLED_ACTIVE_OPACITY),
            stop_indicators_disabled: theme.on_surface().scale_alpha(DISABLED_INDICATOR_OPACITY),
            icon_disabled: theme.on_surface().scale_alpha(DISABLED_INDICATOR_OPACITY),
        }
    }

    pub(crate) fn disabled(self) -> Self {
        Self {
            active_track: self.active_track_disabled,
            inactive_track: self.inactive_track_disabled,
            handle: self.handle_disabled,
            stop_indicator: self.stop_indicators_disabled,
            stop_indicators_active: self.stop_indicators_disabled,
            stop_indicators_inactive: self.stop_indicators_disabled,
            icon_active: self.icon_disabled,
            icon_inactive: self.icon_disabled,
            ..self
        }
    }
}

impl Interpolable for Style {
    fn interpolate(self, other: Self, t: f32) -> Self {
        Self {
            active_track: self.active_track.interpolate(other.active_track, t),
            inactive_track: self.inactive_track.interpolate(other.inactive_track, t),
            handle: self.handle.interpolate(other.handle, t),
            stop_indicator: self.stop_indicator.interpolate(other.stop_indicator, t),
            stop_indicators_active: self
                .stop_indicators_active
                .interpolate(other.stop_indicators_active, t),
            stop_indicators_inactive: self
                .stop_indicators_inactive
                .interpolate(other.stop_indicators_inactive, t),
            label_container: self.label_container.interpolate(other.label_container, t),
            label_text: self.label_text.interpolate(other.label_text, t),
            state_layer: self.state_layer.interpolate(other.state_layer, t),
            focus_ring: self.focus_ring.interpolate(other.focus_ring, t),
            icon_active: self.icon_active.interpolate(other.icon_active, t),
            icon_inactive: self.icon_inactive.interpolate(other.icon_inactive, t),
            handle_overlap_outline: self
                .handle_overlap_outline
                .interpolate(other.handle_overlap_outline, t),
            active_track_disabled: self
                .active_track_disabled
                .interpolate(other.active_track_disabled, t),
            inactive_track_disabled: self
                .inactive_track_disabled
                .interpolate(other.inactive_track_disabled, t),
            handle_disabled: self.handle_disabled.interpolate(other.handle_disabled, t),
            stop_indicators_disabled: self
                .stop_indicators_disabled
                .interpolate(other.stop_indicators_disabled, t),
            icon_disabled: self.icon_disabled.interpolate(other.icon_disabled, t),
        }
    }
}

/// Animates the style spring, or jumps straight to the new style when it
/// changed (e.g. on a theme switch).
fn transition_style(
    spring: &mut SpringValue<Style>,
    last: &mut Style,
    target: Style,
    now: Instant,
) {
    if *last != target {
        *last = target;
        spring.reset(target, now);
    } else {
        spring.set_target(target, now);
    }
}

/// Animates a handle spring to `target` and requests the next frame.
fn animate_handle_spring<Message>(
    spring: &mut SpringValue<f32>,
    target: f32,
    now: Instant,
    shell: &mut iced::advanced::Shell<'_, Message>,
) {
    set_handle_target(spring, target, now);
    shell.request_redraw();
}

fn set_handle_target(spring: &mut SpringValue<f32>, target: f32, now: Instant) {
    let target = target.clamp(0.0, 1.0);
    if target <= 0.0 || target >= 1.0 {
        spring.reset(target, now);
    } else {
        spring.set_target(target, now);
    }
}

/// Settles a handle spring onto `target` after a drag release, nudging it with
/// a small impulse so a fast flick ends with a slight wobble. Only the drawn
/// position moves: the value is already final at this point.
fn settle_handle_spring(spring: &mut SpringValue<f32>, target: f32, now: Instant) {
    if target <= 0.0 || target >= 1.0 {
        spring.reset(target.clamp(0.0, 1.0), now);
    } else if spring.to != target {
        spring.set_target(target, now);
        spring.spring.velocity = DRAG_RELEASE_IMPULSE;
    }
}

/// Steps a handle spring, stopping it dead at the track edges so it never
/// overshoots the end of the track and springs back.
fn step_handle_spring(spring: &mut SpringValue<f32>, now: Instant) {
    spring.step(now);

    let value = spring.value();
    if value <= 0.0 {
        spring.reset(0.0, now);
    } else if value >= 1.0 {
        spring.reset(1.0, now);
    }
}

fn label_follows_handle(handle_at_rest: bool, label_requested: bool) -> bool {
    !handle_at_rest && label_requested
}

fn configured_springs(effects: Option<Spring>, spatial: Option<Spring>) -> (Spring, Spring) {
    (
        effects.unwrap_or_else(|| fast_effects(motion::Scheme::default())),
        spatial.unwrap_or_else(|| fast_spatial(motion::Scheme::default())),
    )
}

struct CommonState<P: text::Paragraph> {
    keyboard_modifiers: keyboard::Modifiers,
    is_focused: bool,
    keyboard_focus: bool,

    label_scale_spring: SpringValue<f32>,
    focus_ring_spring: SpringValue<f32>,
    style_spring: SpringValue<Style>,
    label_paragraph: paragraph::Plain<P>,
    label_content: String,

    last_effects_spring: Spring,
    last_spatial_spring: Spring,
    last_style: Style,
}

impl<P: text::Paragraph> CommonState<P> {
    fn new(
        now: Instant,
        effects_spring: Spring,
        spatial_spring: Spring,
        style: Style,
        label_content: String,
        font: P::Font,
    ) -> Self {
        Self {
            keyboard_modifiers: keyboard::Modifiers::default(),
            is_focused: false,
            keyboard_focus: false,
            label_scale_spring: SpringValue::new(0.0, effects_spring, now),
            focus_ring_spring: SpringValue::new(0.0, effects_spring, now),
            style_spring: SpringValue::new(style, effects_spring, now),
            label_paragraph: paragraph::Plain::new(label_text(font, label_content.clone())),
            label_content,
            last_effects_spring: effects_spring,
            last_spatial_spring: spatial_spring,
            last_style: style,
        }
    }

    fn is_animating(&self) -> bool {
        !self.label_scale_spring.is_at_rest()
            || !self.focus_ring_spring.is_at_rest()
            || !self.style_spring.is_at_rest()
    }

    fn step(&mut self, now: Instant) {
        self.label_scale_spring.step(now);
        self.focus_ring_spring.step(now);
        self.style_spring.step(now);
    }

    fn set_focused(&mut self, focused: bool) {
        self.is_focused = focused;
        self.keyboard_focus = focused;
    }
}

struct State<P: text::Paragraph> {
    is_dragging: bool,
    drag_factor: Option<f32>,
    drag_display: f32,

    common: CommonState<P>,

    ripple: Ripple,
    handle_width_spring: SpringValue<f32>,
    handle_position_spring: SpringValue<f32>,
    icon_start_visibility_spring: SpringValue<f32>,
    icon_end_visibility_spring: SpringValue<f32>,
}

impl<P: text::Paragraph> State<P> {
    fn is_animating(&self) -> bool {
        self.common.is_animating()
            || !self.handle_width_spring.is_at_rest()
            || !self.handle_position_spring.is_at_rest()
            || !self.icon_start_visibility_spring.is_at_rest()
            || !self.icon_end_visibility_spring.is_at_rest()
    }

    // Chase the cursor during continuous drag, then step all springs.
    fn step(&mut self, now: Instant, continuous_drag: bool) {
        if continuous_drag && let Some(target) = self.drag_factor {
            let delta = now
                .duration_since(self.handle_position_spring.spring.time)
                .as_secs_f32();
            let decay = 1.0 - (-delta / DRAG_CHASE_DECAY).exp();
            self.drag_display += (target - self.drag_display) * decay;
            self.handle_position_spring.reset(self.drag_display, now);
        }

        self.common.step(now);
        self.handle_width_spring.step(now);
        step_handle_spring(&mut self.handle_position_spring, now);
        self.icon_start_visibility_spring.step(now);
        self.icon_end_visibility_spring.step(now);
    }
}

impl<P: text::Paragraph> operation::Focusable for State<P> {
    fn is_focused(&self) -> bool {
        self.common.is_focused
    }

    fn focus(&mut self) {
        self.common.set_focused(true);
    }

    fn unfocus(&mut self) {
        self.common.set_focused(false);
    }
}

pub struct Slider<'a, T, Message> {
    id: Option<Id>,
    range: RangeInclusive<T>,
    step: T,
    shift_step: Option<T>,
    value: T,
    default: Option<T>,
    on_change: Option<Box<dyn Fn(T) -> Message + 'a>>,
    on_release: Option<Message>,
    width: Length,
    height: Length,
    style: Style,
    size: Size,
    axis: Axis,
    centered: bool,
    stops: bool,
    continuous_tick_count: Option<u32>,
    value_indicator: ValueIndicator<'a, T>,
    decimal_places: usize,
    inset_icon_start: Option<InsetIcon<'a>>,
    inset_icon_end: Option<InsetIcon<'a>>,
    effects_spring: Option<Spring>,
    spatial_spring: Option<Spring>,
}

impl<'a, T, Message> Slider<'a, T, Message>
where
    T: Copy + From<u8> + PartialOrd,
{
    #[must_use]
    pub fn new<F>(range: RangeInclusive<T>, value: T, on_change: F, style: Style) -> Self
    where
        F: 'a + Fn(T) -> Message,
    {
        let value = if value >= *range.start() {
            value
        } else {
            *range.start()
        };

        let value = if value <= *range.end() {
            value
        } else {
            *range.end()
        };

        Slider {
            id: None,
            value,
            default: None,
            range,

            step: T::from(1),
            shift_step: None,
            on_change: Some(Box::new(on_change)),
            on_release: None,
            width: Length::Fill,
            height: Length::Fill,
            style,
            size: Size::default(),
            axis: Axis::default(),
            centered: false,
            stops: true,
            continuous_tick_count: None,
            value_indicator: ValueIndicator::None,
            decimal_places: 1,
            inset_icon_start: None,
            inset_icon_end: None,
            effects_spring: None,
            spatial_spring: None,
        }
    }

    #[must_use]
    pub fn id(mut self, id: impl Into<Id>) -> Self {
        self.id = Some(id.into());
        self
    }

    #[must_use]
    pub fn on_change_maybe<F>(mut self, on_change: Option<F>) -> Self
    where
        F: 'a + Fn(T) -> Message,
    {
        self.on_change = on_change.map(|on_change| Box::new(on_change) as _);
        self
    }

    #[must_use]
    pub fn default(mut self, default: impl Into<T>) -> Self {
        self.default = Some(default.into());
        self
    }

    #[must_use]
    pub fn on_release(mut self, on_release: Message) -> Self {
        self.on_release = Some(on_release);
        self
    }

    #[must_use]
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    #[must_use]
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    #[must_use]
    pub fn step(mut self, step: impl Into<T>) -> Self {
        self.step = step.into();
        self
    }

    #[must_use]
    pub fn shift_step(mut self, shift_step: impl Into<T>) -> Self {
        self.shift_step = Some(shift_step.into());
        self
    }

    #[must_use]
    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }

    #[must_use]
    pub fn axis(mut self, axis: Axis) -> Self {
        self.axis = axis;
        self
    }

    #[must_use]
    pub fn vertical(self) -> Self {
        self.axis(Axis::Vertical)
    }

    #[must_use]
    pub fn centered(mut self, centered: bool) -> Self {
        self.centered = centered;
        self
    }

    #[must_use]
    pub fn stops(mut self, stops: bool) -> Self {
        self.stops = stops;
        self
    }

    #[must_use]
    pub fn continuous_tick_count(mut self, count: u32) -> Self {
        self.continuous_tick_count = Some(count);
        self
    }

    #[must_use]
    pub fn value_indicator(mut self, value_indicator: ValueIndicator<'a, T>) -> Self {
        self.value_indicator = value_indicator;
        self
    }

    #[must_use]
    pub fn decimal_places(mut self, decimal_places: usize) -> Self {
        self.decimal_places = decimal_places;
        self
    }

    #[must_use]
    pub fn inset_icon_start(mut self, icon: InsetIcon<'a>) -> Self {
        self.inset_icon_start = Some(icon);
        self
    }

    #[must_use]
    pub fn inset_icon_end(mut self, icon: InsetIcon<'a>) -> Self {
        self.inset_icon_end = Some(icon);
        self
    }

    #[must_use]
    pub fn motion_scheme(mut self, scheme: motion::Scheme) -> Self {
        self.effects_spring = Some(fast_effects(scheme));
        self.spatial_spring = Some(fast_spatial(scheme));
        self
    }

    #[must_use]
    pub fn effects_spring(mut self, spring: Spring) -> Self {
        self.effects_spring = Some(spring);
        self
    }

    #[must_use]
    pub fn spatial_spring(mut self, spring: Spring) -> Self {
        self.spatial_spring = Some(spring);
        self
    }
}

impl<'a, T, Message> Slider<'a, T, Message>
where
    T: Copy + Display + Into<f64> + PartialOrd,
{
    fn format_value(&self) -> Option<String> {
        let step_f64: f64 = self.step.into();
        let min_decimal_places = step_decimal_places(step_f64);
        let decimal_places = self.decimal_places.max(min_decimal_places);
        self.value_indicator
            .format(self.value, decimal_places, min_decimal_places)
    }
}

impl<T, Message, Theme, Renderer> Widget<Message, Theme, Renderer> for Slider<'_, T, Message>
where
    T: Copy + Display + Into<f64> + num_traits::FromPrimitive + PartialOrd,
    Message: Clone,
    Renderer: iced::advanced::text::Renderer
        + iced::advanced::svg::Renderer
        + iced_widget::renderer::geometry::Renderer,
    Renderer::Font: From<iced::Font>,
{
    fn tag(&self) -> iced::advanced::widget::tree::Tag {
        iced::advanced::widget::tree::Tag::of::<State<Renderer::Paragraph>>()
    }

    fn state(&self) -> iced::advanced::widget::tree::State {
        let now = Instant::now();
        let (effects_spring, spatial_spring) =
            configured_springs(self.effects_spring, self.spatial_spring);
        let factor = value_factor(self.value, &self.range);
        let label_content = self.format_value().unwrap_or_default();

        let state: State<Renderer::Paragraph> = State {
            is_dragging: false,
            drag_factor: None,
            drag_display: factor,
            common: CommonState::new(
                now,
                effects_spring,
                spatial_spring,
                self.style,
                label_content,
                Renderer::ICON_FONT,
            ),
            ripple: Ripple::new(),
            handle_width_spring: SpringValue::new(HANDLE_WIDTH_IDLE, spatial_spring, now),
            handle_position_spring: SpringValue::new(factor, spatial_spring, now),
            icon_start_visibility_spring: SpringValue::new(0.0, effects_spring, now),
            icon_end_visibility_spring: SpringValue::new(0.0, effects_spring, now),
        };

        iced::advanced::widget::tree::State::new(state)
    }

    fn diff(&self, tree: &mut Tree) {
        let state = tree.state.downcast_mut::<State<Renderer::Paragraph>>();

        if self.on_change.is_none() {
            state.is_dragging = false;
            state.drag_factor = None;
        }

        if state.drag_factor.is_none() {
            state
                .handle_position_spring
                .set_target(value_factor(self.value, &self.range), Instant::now());
        }
    }

    fn size(&self) -> iced::Size<Length> {
        slider_size(self.axis, self.width, self.height, self.size.handle_height)
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> layout::Node {
        slider_layout(
            self.axis,
            self.width,
            self.height,
            self.size.handle_height,
            limits,
        )
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: layout::Layout<'_>,
        _renderer: &Renderer,
        operation: &mut dyn operation::Operation,
    ) {
        let state = tree.state.downcast_mut::<State<Renderer::Paragraph>>();
        operation.focusable(self.id.as_ref(), layout.bounds(), state);
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: layout::Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        _clipboard: &mut dyn iced::advanced::Clipboard,
        shell: &mut iced::advanced::Shell<'_, Message>,
        _viewport: &iced::Rectangle,
    ) {
        let state = tree.state.downcast_mut::<State<Renderer::Paragraph>>();
        let bounds = layout.bounds();
        let enabled = self.on_change.is_some();
        let now = Instant::now();

        let len = main_len(self.axis, bounds);
        let range_step = effective_step(self.step);
        let full_range = range_f64(&self.range);
        let pixels_per_step = if range_step > 0.0 {
            f64::from(len) * range_step / (full_range.end - full_range.start)
        } else {
            0.0
        };
        let continuous_drag = pixels_per_step < MIN_PIXELS_PER_STEP;
        let handle_width = state.handle_width_spring.value();
        let handle_center = center_at(len, handle_width, state.handle_position_spring.value());
        let circle = handle_circle(self.axis, bounds, self.size.handle_height, handle_center);

        let interaction_bounds = if state.is_dragging {
            expanded_bounds(bounds)
        } else {
            circle
        };
        state
            .ripple
            .handle_in(event, interaction_bounds, circle, cursor, enabled, shell);

        sync_spring_model(
            &mut state.common.last_effects_spring,
            self.effects_spring,
            &mut [
                &mut state.common.label_scale_spring.spring,
                &mut state.common.focus_ring_spring.spring,
                &mut state.icon_start_visibility_spring.spring,
                &mut state.icon_end_visibility_spring.spring,
                &mut state.common.style_spring.spring,
            ],
        );
        sync_spring_model(
            &mut state.common.last_spatial_spring,
            self.spatial_spring,
            &mut [
                &mut state.handle_width_spring.spring,
                &mut state.handle_position_spring.spring,
            ],
        );

        let resolved_style = if enabled {
            self.style
        } else {
            self.style.disabled()
        };
        transition_style(
            &mut state.common.style_spring,
            &mut state.common.last_style,
            resolved_style,
            now,
        );

        // Sync the handle position spring with the current value, unless a
        // continuous drag is chasing the cursor (the chase owns the spring).
        if !(state.is_dragging && continuous_drag && state.drag_factor.is_some()) {
            set_handle_target(
                &mut state.handle_position_spring,
                value_factor(self.value, &self.range),
                now,
            );
        }
        state.handle_width_spring.set_target(
            match state.is_dragging {
                true => HANDLE_WIDTH_DRAGGED,
                false => HANDLE_WIDTH_IDLE,
            },
            now,
        );

        let label_following = label_follows_handle(
            state.handle_position_spring.is_at_rest(),
            state.common.label_scale_spring.to > 0.5,
        );
        let cursor_over_handle = enabled && cursor.is_over(bounds) && cursor.is_over(circle);
        let label_visible = !matches!(self.value_indicator, ValueIndicator::None)
            && enabled
            && (state.is_dragging
                || cursor_over_handle
                || state.common.keyboard_focus
                || label_following);
        let focus_target = (state.common.keyboard_focus && enabled) as u32 as f32;
        state
            .common
            .label_scale_spring
            .set_target(label_visible as u32 as f32, now);
        state.common.focus_ring_spring.set_target(focus_target, now);

        let label_content = self.format_value().unwrap_or_default();
        sync_label_content(
            &mut state.common.label_paragraph,
            &mut state.common.label_content,
            label_content,
            renderer.default_font(),
        );

        if self.inset_icon_start.is_some() || self.inset_icon_end.is_some() {
            let track_height = self.size.track_height;
            let active = self
                .active_main_range(len, handle_width, handle_center)
                .valid();
            if self.inset_icon_start.is_some() {
                let center = track_height / 2.0;
                state.icon_start_visibility_spring.set_target(
                    active.is_some_and(|active| active.contains(&center)) as u32 as f32,
                    now,
                );
            }
            if self.inset_icon_end.is_some() {
                let center = len - track_height / 2.0;
                state.icon_end_visibility_spring.set_target(
                    active.is_some_and(|active| active.contains(&center)) as u32 as f32,
                    now,
                );
            }
        }

        let drag_is_chasing = continuous_drag
            && state.is_dragging
            && state
                .drag_factor
                .is_some_and(|target| (target - state.drag_display).abs() > 0.001);
        if state.is_animating() || drag_is_chasing {
            shell.request_redraw();
        }

        state.step(now, continuous_drag);

        let axis = self.axis;
        let range = self.range.clone();
        let locate = move |position: Point| -> Option<T> {
            value_at(axis, bounds, position, &range, range_step)
        };

        let change =
            |slider: &mut Self, shell: &mut iced::advanced::Shell<'_, Message>, new_value: T| {
                if (slider.value.into() - new_value.into()).abs() > f64::EPSILON {
                    if let Some(on_change) = &slider.on_change {
                        shell.publish(on_change(new_value));
                    }
                    slider.value = new_value;
                }
            };

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                if !enabled || shell.is_event_captured() {
                    return;
                }

                if let Some(position) = cursor.position_over(bounds) {
                    if !state.common.is_focused {
                        state.common.is_focused = true;
                        shell.request_redraw();
                    }

                    state.common.keyboard_focus = false;

                    if state.common.keyboard_modifiers.command() {
                        let _ = self.default.map(|default| change(self, shell, default));
                        state.is_dragging = false;
                        state.drag_factor = None;
                    } else if let Some(new_value) = locate(position) {
                        let target = value_factor(new_value, &self.range);
                        change(self, shell, new_value);

                        if continuous_drag {
                            state.drag_display = state.handle_position_spring.value();
                        } else {
                            animate_handle_spring(
                                &mut state.handle_position_spring,
                                target,
                                now,
                                shell,
                            );
                        }

                        state.is_dragging = true;
                        state.drag_factor = Some(target);
                    }

                    shell.capture_event();
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerLifted { .. })
            | Event::Touch(touch::Event::FingerLost { .. }) => {
                if state.is_dragging {
                    // While dragging, the handle chased the cursor and lags
                    // behind, so let it settle onto the final value. The value
                    // itself is final here; only the drawn position moves.
                    let settle = if range_step > 0.0 {
                        Some(value_factor(self.value, &self.range))
                    } else {
                        state.drag_factor
                    };

                    if continuous_drag && let Some(target) = settle {
                        settle_handle_spring(&mut state.handle_position_spring, target, now);
                    }

                    if let Some(on_release) = self.on_release.clone() {
                        shell.publish(on_release);
                    }
                    state.is_dragging = false;
                    state.drag_factor = None;
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { .. })
            | Event::Touch(touch::Event::FingerMoved { .. }) => {
                if state.is_dragging && enabled {
                    if let Some(position) = cursor.land().position()
                        && let Some(new_value) = locate(position)
                    {
                        let target = value_factor(new_value, &self.range);
                        change(self, shell, new_value);

                        let drag_factor = if continuous_drag {
                            factor_at(
                                len,
                                state.handle_width_spring.value(),
                                cursor_main(self.axis, bounds, position),
                            )
                        } else {
                            animate_handle_spring(
                                &mut state.handle_position_spring,
                                target,
                                now,
                                shell,
                            );
                            target
                        };

                        state.drag_factor = Some(drag_factor);
                    }

                    shell.capture_event();
                }
            }
            Event::Mouse(mouse::Event::WheelScrolled { delta })
                if state.common.keyboard_modifiers.control() =>
            {
                if cursor.is_over(bounds) && enabled {
                    let range = range_f64(&self.range);
                    let step = keyboard_step(
                        self.step,
                        self.shift_step,
                        state.common.keyboard_modifiers.shift(),
                        &range,
                    );

                    let new_value =
                        wheel_value(scroll_delta(delta), self.value.into(), step, &range);

                    if let Some(new_value) = T::from_f64(new_value) {
                        change(self, shell, new_value);
                    }

                    shell.capture_event();
                }
            }
            Event::Keyboard(keyboard::Event::KeyPressed { key, .. }) => {
                if !state.common.is_focused || !enabled {
                    return;
                }

                let range = range_f64(&self.range);
                let step = keyboard_step(
                    self.step,
                    self.shift_step,
                    state.common.keyboard_modifiers.shift(),
                    &range,
                );

                let new_value = keyboard_value(key, self.value.into(), step, &range);

                if let Some(new_value) = new_value
                    && let Some(new_value) = T::from_f64(new_value)
                {
                    change(self, shell, new_value);
                    shell.capture_event();
                }
            }
            Event::Keyboard(keyboard::Event::KeyReleased {
                key: Key::Named(key::Named::Escape),
                ..
            }) => {
                if state.common.is_focused {
                    state.common.is_focused = false;
                    state.common.keyboard_focus = false;
                    shell.capture_event();
                }
            }
            Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) => {
                state.common.keyboard_modifiers = *modifiers;
            }
            _ => {}
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &iced::advanced::renderer::Style,
        layout: layout::Layout<'_>,
        _cursor: mouse::Cursor,
        viewport: &iced::Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State<Renderer::Paragraph>>();
        let style = state.common.style_spring.value();
        let enabled = self.on_change.is_some();
        let bounds = layout.bounds();
        let len = main_len(self.axis, bounds);
        let track_height = self.size.track_height;

        let handle_width = state.handle_width_spring.value();
        let handle_center = center_at(len, handle_width, state.handle_position_spring.value());

        let cross_center = cross_center(self.axis, bounds);

        let active = self.active_main_range(len, handle_width, handle_center);
        let inactive = self.inactive_main_ranges(len, handle_width, handle_center);

        draw_track(
            renderer,
            self.axis,
            bounds,
            len,
            cross_center,
            track_height,
            self.size.corner_radius,
            enabled,
            style,
            active,
            &inactive,
            self.stops,
            self.continuous_tick_count,
        );

        if self.inset_icon_start.is_some() || self.inset_icon_end.is_some() {
            let icon_size = inset_icon_size(track_height);
            let icon_color = |active: bool| -> Color {
                match (active, enabled) {
                    (true, true) => style.icon_active,
                    (false, true) => style.icon_inactive,
                    (_, false) => style.icon_disabled,
                }
            };

            // The icons are pinned to the track ends; the handle is free to
            // ride over them.
            if let Some(icon) = &self.inset_icon_start {
                let center = track_height / 2.0;
                draw_inset_icon(
                    renderer,
                    self.axis,
                    bounds,
                    cross_center,
                    center,
                    icon_size,
                    &icon.on_active,
                    &icon.on_inactive,
                    icon_color(true),
                    icon_color(false),
                    state.icon_start_visibility_spring.value(),
                    viewport,
                );
            }
            if let Some(icon) = &self.inset_icon_end {
                let center = len - track_height / 2.0;
                draw_inset_icon(
                    renderer,
                    self.axis,
                    bounds,
                    cross_center,
                    center,
                    icon_size,
                    &icon.on_active,
                    &icon.on_inactive,
                    icon_color(true),
                    icon_color(false),
                    state.icon_end_visibility_spring.value(),
                    viewport,
                );
            }
        }

        let handle = handle_rect(
            self.axis,
            bounds,
            self.size.handle_height,
            handle_width,
            handle_center,
        );
        renderer.fill_quad(
            Quad {
                bounds: handle,
                border: Border::default().rounded(f32::MAX),
                ..Quad::default()
            },
            if enabled {
                style.handle
            } else {
                style.handle_disabled
            },
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: layout::Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &iced::Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        let state = tree.state.downcast_ref::<State<Renderer::Paragraph>>();
        slider_mouse_interaction(
            self.on_change.is_none(),
            state.is_dragging,
            cursor.is_over(layout.bounds()),
        )
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: layout::Layout<'b>,
        _renderer: &Renderer,
        viewport: &iced::Rectangle,
        translation: iced::Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        let state = tree.state.downcast_ref::<State<Renderer::Paragraph>>();
        let enabled = self.on_change.is_some();

        let widget_bounds = layout.bounds();
        let bounds = Rectangle {
            x: widget_bounds.x + translation.x,
            y: widget_bounds.y + translation.y,
            ..widget_bounds
        };

        let len = main_len(self.axis, bounds);
        let handle_width = state.handle_width_spring.value();
        let handle_center = center_at(len, handle_width, state.handle_position_spring.value());
        let circle = handle_circle(self.axis, bounds, self.size.handle_height, handle_center);

        let style = state.common.style_spring.value();
        let focus = state.common.focus_ring_spring.value();
        let label_scale = state.common.label_scale_spring.value();

        let ripple_visible = state.ripple.is_visible(circle.size(), enabled);

        if !ripple_visible && focus <= 0.01 && label_scale <= 0.01 {
            return None;
        }

        let focus_ring = if focus > 0.01 && enabled && state.common.keyboard_focus {
            focus_ring_bounds(circle, self.size.handle_height, focus, style.focus_ring)
        } else {
            None
        };

        let handle = handle_rect(
            self.axis,
            bounds,
            self.size.handle_height,
            handle_width,
            handle_center,
        );
        let label = value_label(
            self.axis,
            &handle,
            0.0,
            &state.common.label_paragraph,
            label_scale,
            style,
            &state.common.label_content,
        );

        let overlay_element = SliderOverlay {
            ripples: [
                Some(RippleLayer {
                    ripple: state.ripple,
                    layer: style.state_layer,
                    bounds: circle,
                    enabled,
                }),
                None,
            ],
            focus_ring,
            label,
            clip: Rectangle::default(),
        }
        .clipped(viewport)?;

        Some(overlay::Element::new(Box::new(overlay_element)))
    }
}

crate::macros::impl_element_from!(
    ['a, T, Message, Theme, Renderer]
    Slider<'a, T, Message> => Element<'a, Message, Theme, Renderer>
    where
        T: Copy + Display + Into<f64> + num_traits::FromPrimitive + PartialOrd + 'a,
        Message: Clone + 'a,
        Renderer: iced::advanced::text::Renderer
            + iced::advanced::svg::Renderer
            + iced_widget::renderer::geometry::Renderer
            + 'a,
        Renderer::Font: From<iced::Font>,
);

#[derive(Debug, Clone, Copy)]
pub(crate) struct Segment {
    pub start: f32,
    pub end: f32,

    start_at_edge: bool,
    end_at_edge: bool,
}

impl Segment {
    fn new(start: f32, end: f32, start_at_edge: bool, end_at_edge: bool) -> Self {
        Self {
            start,
            end,
            start_at_edge,
            end_at_edge,
        }
    }

    fn valid(&self) -> Option<Self> {
        (self.end - self.start > 0.5).then_some(*self)
    }

    fn contains(&self, main: &f32) -> bool {
        *main >= self.start && *main <= self.end
    }

    fn radii(&self, axis: Axis, corner_radius: f32) -> Radius {
        let (start, end) = (
            if self.start_at_edge {
                corner_radius
            } else {
                INNER_CORNER_RADIUS
            },
            if self.end_at_edge {
                corner_radius
            } else {
                INNER_CORNER_RADIUS
            },
        );

        match axis {
            Axis::Horizontal => Radius {
                top_left: start,
                bottom_left: start,
                top_right: end,
                bottom_right: end,
            },
            Axis::Vertical => Radius {
                bottom_left: start,
                bottom_right: start,
                top_left: end,
                top_right: end,
            },
        }
    }
}

impl<T, Message> Slider<'_, T, Message>
where
    T: Copy + Into<f64> + PartialOrd,
{
    pub(crate) fn active_main_range(
        &self,
        len: f32,
        handle_width: f32,
        handle_center: f32,
    ) -> Segment {
        if self.centered {
            let mid = center_at(len, handle_width, 0.5);
            let (start, end) = if handle_center < mid {
                (handle_center, mid)
            } else {
                (mid, handle_center)
            };
            Segment::new(start, end, false, false)
        } else {
            Segment::new(
                0.0,
                handle_center - handle_width / 2.0 - HANDLE_GAP,
                true,
                false,
            )
        }
    }

    pub(crate) fn inactive_main_ranges(
        &self,
        len: f32,
        handle_width: f32,
        handle_center: f32,
    ) -> [Segment; 2] {
        if self.centered {
            let mid = center_at(len, handle_width, 0.5);
            [
                Segment::new(0.0, mid.min(handle_center), true, false),
                Segment::new(mid.max(handle_center), len, false, true),
            ]
        } else {
            [
                Segment::new(0.0, 0.0, false, false),
                Segment::new(
                    handle_center + handle_width / 2.0 + HANDLE_GAP,
                    len,
                    false,
                    true,
                ),
            ]
        }
    }
}

fn main_len(axis: Axis, bounds: Rectangle) -> f32 {
    match axis {
        Axis::Horizontal => bounds.width,
        Axis::Vertical => bounds.height,
    }
}

fn center_at(len: f32, handle_width: f32, factor: f32) -> f32 {
    let len = len.max(handle_width);
    handle_width / 2.0 + (len - handle_width) * factor.clamp(0.0, 1.0)
}

fn factor_at(len: f32, handle_width: f32, center: f32) -> f32 {
    let len = len.max(handle_width + 1.0);
    ((center - handle_width / 2.0) / (len - handle_width)).clamp(0.0, 1.0)
}

fn cursor_main(axis: Axis, bounds: Rectangle, position: Point) -> f32 {
    match axis {
        Axis::Horizontal => position.x - bounds.x,
        Axis::Vertical => bounds.y + bounds.height - position.y,
    }
}

fn rect_at(
    axis: Axis,
    bounds: Rectangle,
    main: RangeInclusive<f32>,
    cross: RangeInclusive<f32>,
) -> Rectangle {
    let (main_start, main_end) = main.into_inner();
    let (cross_start, cross_end) = cross.into_inner();

    match axis {
        Axis::Horizontal => Rectangle {
            x: bounds.x + main_start,
            width: main_end - main_start,
            y: cross_start,
            height: cross_end - cross_start,
        },
        Axis::Vertical => Rectangle {
            y: bounds.y + bounds.height - main_end,
            height: main_end - main_start,
            x: cross_start,
            width: cross_end - cross_start,
        },
    }
}

fn cross_center(axis: Axis, bounds: Rectangle) -> f32 {
    match axis {
        Axis::Horizontal => bounds.y + bounds.height / 2.0,
        Axis::Vertical => bounds.x + bounds.width / 2.0,
    }
}

pub(crate) fn handle_rect(
    axis: Axis,
    bounds: Rectangle,
    handle_height: f32,
    handle_width: f32,
    handle_center: f32,
) -> Rectangle {
    let cross_center = cross_center(axis, bounds);

    rect_at(
        axis,
        bounds,
        handle_center - handle_width / 2.0..=handle_center + handle_width / 2.0,
        cross_center - handle_height / 2.0..=cross_center + handle_height / 2.0,
    )
}

fn handle_circle(
    axis: Axis,
    bounds: Rectangle,
    handle_height: f32,
    handle_center: f32,
) -> Rectangle {
    let diameter = state_circle_size(handle_height);
    let cross_center = cross_center(axis, bounds);

    rect_at(
        axis,
        bounds,
        handle_center - diameter / 2.0..=handle_center + diameter / 2.0,
        cross_center - diameter / 2.0..=cross_center + diameter / 2.0,
    )
}

fn state_circle_size(handle_height: f32) -> f32 {
    handle_height.max(STATE_LAYER_MIN_SIZE)
}

fn focus_ring_size(handle_height: f32) -> f32 {
    handle_height.max(FOCUS_RING_MIN_SIZE)
}

fn inset_icon_size(track_height: f32) -> f32 {
    (track_height / 2.0).clamp(16.0, 24.0)
}

fn label_anchor(axis: Axis, handle_rect: &Rectangle) -> Point {
    match axis {
        Axis::Horizontal => Point::new(
            handle_rect.x + handle_rect.width / 2.0,
            handle_rect.y - LABEL_GAP,
        ),
        Axis::Vertical => Point::new(
            handle_rect.x + handle_rect.width + LABEL_GAP,
            handle_rect.y + handle_rect.height / 2.0,
        ),
    }
}

fn focus_ring_bounds(
    circle: Rectangle,
    handle_height: f32,
    focus: f32,
    color: Color,
) -> Option<(Rectangle, Color)> {
    let full = focus_ring_size(handle_height);
    let size = full * focus;

    Some((
        Rectangle {
            x: circle.x + (circle.width - full) / 2.0 + (full - size) / 2.0,
            y: circle.y + (circle.height - full) / 2.0 + (full - size) / 2.0,
            width: size,
            height: size,
        },
        Color {
            a: color.a * focus,
            ..color
        },
    ))
}

fn value_label<P>(
    axis: Axis,
    handle: &Rectangle,
    anchor_offset: f32,
    paragraph: &paragraph::Plain<P>,
    scale: f32,
    style: Style,
    content: &str,
) -> Option<ValueLabel>
where
    P: text::Paragraph,
{
    if scale <= 0.01 {
        return None;
    }

    let mut anchor = label_anchor(axis, handle);
    match axis {
        Axis::Horizontal => anchor.x += anchor_offset,
        Axis::Vertical => anchor.y += anchor_offset,
    }

    Some(ValueLabel {
        axis,
        anchor,
        width: paragraph.min_width() + 2.0 * LABEL_HORIZONTAL_PADDING,
        height: LABEL_MIN_HEIGHT,
        scale,
        background: style.label_container,
        text_color: style.label_text,
        content: content.to_owned(),
    })
}

fn range_f64<T>(range: &RangeInclusive<T>) -> std::ops::Range<f64>
where
    T: Copy + Into<f64>,
{
    let (start, end) = range.clone().into_inner();
    start.into()..end.into()
}

fn fraction_of(value: f64, range: &std::ops::Range<f64>) -> f32 {
    let span = range.end - range.start;
    if span <= 0.0 {
        0.0
    } else {
        ((value - range.start) / span).clamp(0.0, 1.0) as f32
    }
}

fn value_factor<T>(value: T, range: &RangeInclusive<T>) -> f32
where
    T: Copy + Into<f64>,
{
    fraction_of(value.into(), &range_f64(range))
}

fn snap_to_step(value: f64, range: &std::ops::Range<f64>, step: f64) -> f64 {
    let steps = ((value - range.start) / step).round();
    (range.start + steps * step).clamp(range.start, range.end)
}

fn effective_step<T>(step: T) -> f64
where
    T: Copy + Into<f64>,
{
    let step = step.into();
    if step > 0.0 { step } else { 0.0 }
}

fn effective_keyboard_step(step: f64, range: &std::ops::Range<f64>) -> f64 {
    if step > 0.0 {
        step
    } else {
        (range.end - range.start) / 20.0
    }
}

fn keyboard_step<T>(
    step: T,
    shift_step: Option<T>,
    shift: bool,
    range: &std::ops::Range<f64>,
) -> f64
where
    T: Copy + Into<f64>,
{
    if !shift {
        return effective_keyboard_step(step.into(), range);
    }

    match shift_step {
        Some(shift_step) => {
            let shift_step = shift_step.into();
            if shift_step > 0.0 {
                shift_step
            } else {
                effective_keyboard_step(step.into(), range)
            }
        }
        None => effective_keyboard_step(step.into(), range) * 10.0,
    }
}

fn increment(value: f64, range: &std::ops::Range<f64>, step: f64) -> f64 {
    let steps = ((value - range.start) / step).round();
    (range.start + (steps + 1.0) * step).min(range.end)
}

fn decrement(value: f64, range: &std::ops::Range<f64>, step: f64) -> f64 {
    let steps = ((value - range.start) / step).round();
    (range.start + (steps - 1.0) * step).max(range.start)
}

/// Resolves the value a keyboard key asks for.
fn keyboard_value(key: &Key, current: f64, step: f64, range: &std::ops::Range<f64>) -> Option<f64> {
    let page = (range.end - range.start) / 10.0;

    match key {
        Key::Named(key::Named::ArrowUp | key::Named::ArrowRight) => {
            Some(increment(current, range, step))
        }
        Key::Named(key::Named::ArrowDown | key::Named::ArrowLeft) => {
            Some(decrement(current, range, step))
        }
        Key::Named(key::Named::Home) => Some(range.start),
        Key::Named(key::Named::End) => Some(range.end),
        Key::Named(key::Named::PageUp) => Some((current + page).min(range.end)),
        Key::Named(key::Named::PageDown) => Some((current - page).max(range.start)),
        _ => None,
    }
}

fn scroll_delta(delta: &mouse::ScrollDelta) -> f32 {
    match delta {
        mouse::ScrollDelta::Lines { x: _, y } | mouse::ScrollDelta::Pixels { x: _, y } => *y,
    }
}

/// Resolves the value a wheel notch asks for.
fn wheel_value(delta: f32, current: f64, step: f64, range: &std::ops::Range<f64>) -> f64 {
    if delta < 0.0 {
        decrement(current, range, step)
    } else {
        increment(current, range, step)
    }
}

fn value_at<T>(
    axis: Axis,
    bounds: Rectangle,
    position: Point,
    range: &RangeInclusive<T>,
    step: f64,
) -> Option<T>
where
    T: Copy + Into<f64> + num_traits::FromPrimitive,
{
    let full_range = range_f64(range);
    let factor = factor_at(
        main_len(axis, bounds),
        HANDLE_WIDTH_IDLE,
        cursor_main(axis, bounds, position),
    ) as f64;
    let value = full_range.start + factor * (full_range.end - full_range.start);

    let value = if step > 0.0 {
        snap_to_step(value, &full_range, step)
    } else {
        value.clamp(full_range.start, full_range.end)
    };

    T::from_f64(value)
}

fn label_text<C, F>(font: F, content: C) -> Text<C, F> {
    Text {
        font,
        line_height: text::LineHeight::default(),
        content,
        bounds: iced::Size::new(f32::INFINITY, LABEL_MIN_HEIGHT),
        size: LABEL_TEXT_SIZE.into(),
        align_x: text::Alignment::Center,
        align_y: alignment::Vertical::Center,
        shaping: text::Shaping::Advanced,
        wrapping: text::Wrapping::Word,
    }
}

fn sync_label_content<P>(
    paragraph: &mut paragraph::Plain<P>,
    content: &mut String,
    value: String,
    font: P::Font,
) where
    P: text::Paragraph,
{
    if *content != value {
        *content = value;
        paragraph.update(label_text(font, content.as_str()));
    }
}

pub(crate) fn slider_size(
    axis: Axis,
    width: Length,
    height: Length,
    handle_height: f32,
) -> iced::Size<Length> {
    match axis {
        Axis::Horizontal => iced::Size {
            width,
            height: Length::Fixed(handle_height),
        },
        Axis::Vertical => iced::Size {
            width: Length::Fixed(handle_height),
            height,
        },
    }
}

pub(crate) fn slider_layout(
    axis: Axis,
    width: Length,
    height: Length,
    handle_height: f32,
    limits: &iced::advanced::layout::Limits,
) -> iced::advanced::layout::Node {
    match axis {
        Axis::Horizontal => {
            iced::advanced::layout::atomic(limits, width, Length::Fixed(handle_height))
        }
        Axis::Vertical => {
            iced::advanced::layout::atomic(limits, Length::Fixed(handle_height), height)
        }
    }
}

pub(crate) fn slider_mouse_interaction(
    disabled: bool,
    is_dragging: bool,
    cursor_over: bool,
) -> mouse::Interaction {
    if disabled {
        return mouse::Interaction::default();
    }
    if is_dragging {
        return if cfg!(target_os = "windows") {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::Grabbing
        };
    }
    if cursor_over {
        return if cfg!(target_os = "windows") {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::Grab
        };
    }
    mouse::Interaction::default()
}

/// Draws the active and inactive track segments plus the stop indicators.
#[allow(clippy::too_many_arguments)]
fn draw_track<R>(
    renderer: &mut R,
    axis: Axis,
    bounds: Rectangle,
    len: f32,
    cross_center: f32,
    track_height: f32,
    corner_radius: f32,
    enabled: bool,
    style: Style,
    active: Segment,
    inactive: &[Segment],
    stops: bool,
    continuous_tick_count: Option<u32>,
) where
    R: iced_widget::renderer::geometry::Renderer + ?Sized,
{
    let valid_active = active.valid();

    if let Some(active) = valid_active {
        fill_track_segment(
            renderer,
            axis,
            bounds,
            active,
            cross_center,
            track_height,
            corner_radius,
            if enabled {
                style.active_track
            } else {
                style.active_track_disabled
            },
        );
    }

    let inactive_color = if enabled {
        style.inactive_track
    } else {
        style.inactive_track_disabled
    };
    for inactive in inactive.iter().filter_map(Segment::valid) {
        fill_track_segment(
            renderer,
            axis,
            bounds,
            inactive,
            cross_center,
            track_height,
            corner_radius,
            inactive_color,
        );
    }

    if !stops {
        return;
    }

    let track = rect_at(
        axis,
        bounds,
        0.0..=len,
        cross_center - track_height / 2.0..=cross_center + track_height / 2.0,
    );
    let dot_color = |main: f32| -> Color {
        if !enabled {
            style.stop_indicators_disabled
        } else if valid_active.is_some_and(|active| active.contains(&main)) {
            style.stop_indicators_active
        } else {
            style.stop_indicators_inactive
        }
    };
    let edge_color = if enabled {
        style.stop_indicator
    } else {
        style.stop_indicators_disabled
    };

    if let Some(count) = continuous_tick_count.filter(|count| *count > 1) {
        let stride = (count.saturating_sub(1) as f32 / (len / 8.0).floor().max(1.0))
            .ceil()
            .max(1.0) as usize;
        for i in (1..count).step_by(stride) {
            let fraction = f64::from(i) / f64::from(count);
            let main = len * fraction as f32;
            draw_stop_indicator(
                renderer,
                axis,
                bounds,
                cross_center,
                main,
                dot_color(main),
                track,
            );
        }
    }

    for inactive in inactive.iter().filter_map(Segment::valid) {
        if inactive.end - inactive.start <= STOP_INDICATOR_MIN_SEGMENT {
            continue;
        }

        if inactive.start <= 0.0 {
            let main = STOP_INDICATOR_TRAILING_SPACE + STOP_INDICATOR_SIZE / 2.0;
            draw_stop_indicator(
                renderer,
                axis,
                bounds,
                cross_center,
                main,
                edge_color,
                track,
            );
        }
        if inactive.end >= len {
            let main = len - STOP_INDICATOR_TRAILING_SPACE - STOP_INDICATOR_SIZE / 2.0;
            draw_stop_indicator(
                renderer,
                axis,
                bounds,
                cross_center,
                main,
                edge_color,
                track,
            );
        }
    }
}

fn fill_track_segment<R>(
    renderer: &mut R,
    axis: Axis,
    bounds: Rectangle,
    segment: Segment,
    cross_center: f32,
    track_height: f32,
    corner_radius: f32,
    color: Color,
) where
    R: iced_widget::renderer::geometry::Renderer + ?Sized,
{
    let rect = rect_at(
        axis,
        bounds,
        segment.start..=segment.end,
        cross_center - track_height / 2.0..=cross_center + track_height / 2.0,
    );

    renderer.fill_quad(
        Quad {
            bounds: rect,
            border: Border::default().rounded(segment.radii(axis, corner_radius)),
            ..Quad::default()
        },
        color,
    );
}

fn draw_stop_indicator<R>(
    renderer: &mut R,
    axis: Axis,
    bounds: Rectangle,
    cross_center: f32,
    main: f32,
    color: Color,
    track: Rectangle,
) where
    R: iced_widget::renderer::geometry::Renderer + ?Sized,
{
    let rect = rect_at(
        axis,
        bounds,
        main - STOP_INDICATOR_SIZE / 2.0..=main + STOP_INDICATOR_SIZE / 2.0,
        cross_center - STOP_INDICATOR_SIZE / 2.0..=cross_center + STOP_INDICATOR_SIZE / 2.0,
    );

    renderer.with_layer(track, |renderer| {
        renderer.fill_quad(
            Quad {
                bounds: rect,
                border: Border::default().rounded(f32::MAX),
                ..Quad::default()
            },
            color,
        );
    });
}

fn draw_inset_icon<R>(
    renderer: &mut R,
    axis: Axis,
    bounds: Rectangle,
    cross_center: f32,
    main: f32,
    size: f32,
    on_active: &Icon<'_>,
    on_inactive: &Icon<'_>,
    active_color: Color,
    inactive_color: Color,
    visibility: f32,
    viewport: &iced::Rectangle,
) where
    R: iced::advanced::text::Renderer
        + iced::advanced::svg::Renderer
        + iced_widget::renderer::geometry::Renderer,
    R::Font: From<iced::Font>,
{
    let rect = rect_at(
        axis,
        bounds,
        main - size / 2.0..=main + size / 2.0,
        cross_center - size / 2.0..=cross_center + size / 2.0,
    );

    let crossfade = |renderer: &mut R, icon: &Icon<'_>, color: Color, alpha: f32| match icon {
        Icon::Text { text, font } => {
            let font = font
                .map(R::Font::from)
                .unwrap_or_else(|| renderer.default_font());

            renderer.fill_text(
                Text {
                    font,
                    line_height: text::LineHeight::Absolute(size.into()),
                    content: text.to_string(),
                    bounds: rect.size(),
                    size: size.into(),
                    align_x: text::Alignment::Center,
                    align_y: alignment::Vertical::Center,
                    shaping: text::Shaping::Advanced,
                    wrapping: text::Wrapping::Word,
                },
                rect.center(),
                Color {
                    a: color.a * alpha,
                    ..color
                },
                *viewport,
            );
        }
        Icon::Svg(handle) => {
            renderer.draw_svg(
                Svg::new(handle.clone()).color(color).opacity(alpha),
                rect,
                *viewport,
            );
        }
    };

    crossfade(renderer, on_inactive, inactive_color, 1.0 - visibility);
    crossfade(renderer, on_active, active_color, visibility);
}

pub(crate) struct ValueLabel {
    axis: Axis,

    anchor: Point,
    width: f32,
    height: f32,
    scale: f32,
    background: Color,
    text_color: Color,
    content: String,
}

impl ValueLabel {
    fn pill_rect(&self) -> Rectangle {
        let width = self.width * self.scale;
        let height = self.height * self.scale;

        match self.axis {
            Axis::Horizontal => Rectangle {
                x: self.anchor.x - width / 2.0,
                y: self.anchor.y - height,
                width,
                height,
            },
            Axis::Vertical => Rectangle {
                x: self.anchor.x,
                y: self.anchor.y - height / 2.0,
                width,
                height,
            },
        }
    }
}

pub(crate) struct RippleLayer {
    pub ripple: Ripple,
    pub layer: StateLayer,
    pub bounds: Rectangle,
    pub enabled: bool,
}

pub(crate) struct SliderOverlay {
    pub ripples: [Option<RippleLayer>; 2],
    pub focus_ring: Option<(Rectangle, Color)>,
    pub label: Option<ValueLabel>,
    /// Visible region the overlay is clipped to.
    pub clip: Rectangle,
}

impl SliderOverlay {
    fn content_bounds(&self) -> Rectangle {
        self.ripples
            .iter()
            .flatten()
            .map(|layer| layer.bounds)
            .chain(self.focus_ring.map(|(bounds, _)| bounds))
            .chain(self.label.as_ref().map(ValueLabel::pill_rect))
            .reduce(|a, b| a.union(&b))
            .unwrap_or(Rectangle::default())
    }

    /// Restricts the overlay to the part visible inside `viewport`, returning
    /// `None` when it is scrolled completely out of view.
    fn clipped(mut self, viewport: &Rectangle) -> Option<Self> {
        self.clip = visible_clip(self.content_bounds(), viewport)?;
        Some(self)
    }
}

impl<Message, Theme, Renderer> Overlay<Message, Theme, Renderer> for SliderOverlay
where
    Renderer: iced::advanced::text::Renderer + iced_widget::renderer::geometry::Renderer,
{
    fn layout(
        &mut self,
        _renderer: &Renderer,
        _bounds: iced::Size,
    ) -> iced::advanced::layout::Node {
        let bounds = self.content_bounds();

        layout::Node::new(bounds.size()).move_to(bounds.position())
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &iced::advanced::renderer::Style,
        _layout: layout::Layout<'_>,
        _cursor: mouse::Cursor,
    ) {
        // Clip the overlay to the visible viewport so the state layer and
        // label do not linger once the slider is scrolled out of view.
        renderer.with_layer(self.clip, |renderer| {
            for layer in self.ripples.iter().flatten() {
                layer.ripple.draw(
                    renderer,
                    layer.bounds,
                    Radius::from(f32::MAX),
                    layer.layer,
                    layer.enabled,
                );
            }

            if let Some((bounds, color)) = self.focus_ring {
                renderer.fill_quad(
                    Quad {
                        bounds,
                        border: Border::default()
                            .rounded(f32::MAX)
                            .width(FOCUS_RING_WIDTH)
                            .color(color),
                        ..Quad::default()
                    },
                    Color::TRANSPARENT,
                );
            }

            if let Some(label) = &self.label {
                let pill = label.pill_rect();

                renderer.fill_quad(
                    Quad {
                        bounds: pill,
                        border: Border::default().rounded(Radius::from(f32::MAX)),
                        ..Quad::default()
                    },
                    label.background,
                );

                let text_alpha = ((label.scale - LABEL_TEXT_FADE_START)
                    / (1.0 - LABEL_TEXT_FADE_START))
                    .clamp(0.0, 1.0);
                if text_alpha > 0.0 {
                    renderer.fill_text(
                        Text {
                            font: renderer.default_font(),
                            line_height: text::LineHeight::default(),
                            content: label.content.clone(),
                            bounds: pill.size(),
                            size: LABEL_TEXT_SIZE.into(),
                            align_x: text::Alignment::Center,
                            align_y: alignment::Vertical::Center,
                            shaping: text::Shaping::Advanced,
                            wrapping: text::Wrapping::Word,
                        },
                        pill.center(),
                        Color {
                            a: label.text_color.a * text_alpha,
                            ..label.text_color
                        },
                        self.clip,
                    );
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snap_to_step_snaps_to_nearest_multiple() {
        let range = 0.0..100.0;
        assert_eq!(snap_to_step(10.0, &range, 10.0), 10.0);
        assert_eq!(snap_to_step(14.0, &range, 10.0), 10.0);
        assert_eq!(snap_to_step(15.0, &range, 10.0), 20.0);
    }

    #[test]
    fn snap_to_step_clamps_to_range_bounds() {
        let range = 0.0..100.0;
        assert_eq!(snap_to_step(-5.0, &range, 10.0), 0.0);
        assert_eq!(snap_to_step(105.0, &range, 10.0), 100.0);
    }

    #[test]
    fn snap_to_step_with_offset_range() {
        let range = 10.0..20.0;
        assert_eq!(snap_to_step(12.0, &range, 2.0), 12.0);
        assert_eq!(snap_to_step(13.0, &range, 2.0), 14.0);
    }

    #[test]
    fn effective_keyboard_step_returns_step_when_positive() {
        let range = 0.0..100.0;
        assert_eq!(effective_keyboard_step(5.0, &range), 5.0);
    }

    #[test]
    fn effective_keyboard_step_returns_range_fraction_when_zero() {
        let range = 0.0..100.0;
        assert_eq!(effective_keyboard_step(0.0, &range), 5.0);
    }

    #[test]
    fn keyboard_step_without_shift() {
        let range = 0.0..100.0;
        assert_eq!(keyboard_step(5.0, None, false, &range), 5.0);
    }

    #[test]
    fn keyboard_step_with_shift_and_no_shift_step() {
        let range = 0.0..100.0;
        assert_eq!(keyboard_step(5.0, None, true, &range), 50.0);
    }

    #[test]
    fn keyboard_step_with_shift_and_shift_step() {
        let range = 0.0..100.0;
        assert_eq!(keyboard_step(5.0, Some(25.0), true, &range), 25.0);
    }

    #[test]
    fn increment_moves_up_by_step() {
        let range = 0.0..100.0;
        assert_eq!(increment(10.0, &range, 10.0), 20.0);
        assert_eq!(increment(95.0, &range, 10.0), 100.0);
    }

    #[test]
    fn decrement_moves_down_by_step() {
        let range = 0.0..100.0;
        assert_eq!(decrement(20.0, &range, 10.0), 10.0);
        assert_eq!(decrement(5.0, &range, 10.0), 0.0);
    }

    #[test]
    fn value_at_center_of_track() {
        let bounds = Rectangle {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 20.0,
        };
        let position = Point::new(50.0, 10.0);
        let value = value_at(Axis::Horizontal, bounds, position, &(0..=100i32), 1.0);
        assert_eq!(value, Some(50));
    }

    #[test]
    fn value_at_start_of_track() {
        let bounds = Rectangle {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 20.0,
        };
        let position = Point::new(0.0, 10.0);
        let value = value_at(Axis::Horizontal, bounds, position, &(0..=100i32), 1.0);
        assert_eq!(value, Some(0));
    }

    #[test]
    fn value_at_end_of_track() {
        let bounds = Rectangle {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 20.0,
        };
        let position = Point::new(100.0, 10.0);
        let value = value_at(Axis::Horizontal, bounds, position, &(0..=100i32), 1.0);
        assert_eq!(value, Some(100));
    }

    fn ripple_overlay(bounds: Rectangle) -> SliderOverlay {
        SliderOverlay {
            ripples: [
                Some(RippleLayer {
                    ripple: Ripple::new(),
                    layer: StateLayer::new(Color::BLACK),
                    bounds,
                    enabled: true,
                }),
                None,
            ],
            focus_ring: None,
            label: None,
            clip: Rectangle::default(),
        }
    }

    #[test]
    fn overlay_is_hidden_when_scrolled_out_of_view() {
        let overlay = ripple_overlay(Rectangle {
            x: 10.0,
            y: 10.0,
            width: 48.0,
            height: 48.0,
        });
        let viewport = Rectangle {
            x: 0.0,
            y: 200.0,
            width: 400.0,
            height: 300.0,
        };

        assert!(overlay.clipped(&viewport).is_none());
    }

    #[test]
    fn overlay_clip_is_intersected_with_viewport() {
        let overlay = ripple_overlay(Rectangle {
            x: 10.0,
            y: 10.0,
            width: 48.0,
            height: 48.0,
        });
        let viewport = Rectangle {
            x: 0.0,
            y: 40.0,
            width: 400.0,
            height: 300.0,
        };

        let clipped = overlay
            .clipped(&viewport)
            .expect("partially visible overlay should be kept");
        assert_eq!(
            clipped.clip,
            Rectangle {
                x: 10.0,
                y: 40.0,
                width: 48.0,
                height: 18.0,
            }
        );
    }

    #[test]
    fn inset_icons_stay_at_the_track_ends() {
        let len = 200.0;
        let track_height = 24.0;

        let icon_start = track_height / 2.0;
        let icon_end = len - track_height / 2.0;
        assert_eq!(icon_start, 12.0);
        assert_eq!(icon_end, 188.0);

        let at_start = center_at(len, HANDLE_WIDTH_IDLE, 0.0);
        let active = Segment::new(
            0.0,
            at_start - HANDLE_WIDTH_IDLE / 2.0 - HANDLE_GAP,
            true,
            false,
        );
        assert!(
            !active
                .valid()
                .is_some_and(|active| active.contains(&icon_start))
        );

        let slightly_above = center_at(len, HANDLE_WIDTH_IDLE, 0.1);
        let active = Segment::new(
            0.0,
            slightly_above - HANDLE_WIDTH_IDLE / 2.0 - HANDLE_GAP,
            true,
            false,
        );
        assert!(
            active
                .valid()
                .is_some_and(|active| active.contains(&icon_start))
        );
        assert!(
            !active
                .valid()
                .is_some_and(|active| active.contains(&icon_end))
        );
    }

    #[test]
    fn label_follows_handle_only_while_the_spring_settles() {
        assert!(label_follows_handle(false, true));
        assert!(!label_follows_handle(false, false));
        assert!(!label_follows_handle(true, true));
        assert!(!label_follows_handle(true, false));
    }

    #[test]
    fn settle_handle_spring_only_nudges_when_the_handle_moves() {
        let now = Instant::now();
        let mut spring = SpringValue::new(0.0, fast_spatial(motion::Scheme::default()), now);

        // Already at the target: nothing should animate, and no impulse should
        // be injected (the value is final, only the drawn position may move).
        settle_handle_spring(&mut spring, 0.0, now);
        assert_eq!(spring.spring.velocity, 0.0);

        // Interior values settle with a small kick.
        settle_handle_spring(&mut spring, 0.5, now);
        assert_eq!(spring.to, 0.5);
        assert_eq!(spring.spring.velocity, DRAG_RELEASE_IMPULSE);
        assert!(!spring.is_at_rest());

        // Endpoints stop immediately without carrying spring velocity.
        settle_handle_spring(&mut spring, 1.0, now);
        assert_eq!(spring.value(), 1.0);
        assert_eq!(spring.spring.velocity, 0.0);
        assert!(spring.is_at_rest());
    }

    #[test]
    fn stepping_stops_the_handle_at_the_track_edges() {
        let now = Instant::now();
        let spring_model = fast_spatial(motion::Scheme::default());

        let mut spring = SpringValue::new(0.9, spring_model, now);
        spring.set_target(1.0, now);
        spring.spring.position = 1.2;
        step_handle_spring(&mut spring, now);
        assert_eq!(spring.value(), 1.0);
        assert!(spring.is_at_rest());

        let mut spring = SpringValue::new(0.1, spring_model, now);
        spring.set_target(0.0, now);
        spring.spring.position = 3.0;
        step_handle_spring(&mut spring, now);
        assert_eq!(spring.value(), 0.0);
        assert!(spring.is_at_rest());
    }
}
