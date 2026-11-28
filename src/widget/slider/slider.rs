use std::{fmt::Display, mem, ops::RangeInclusive};

use iced::{
    Border, Element, Event, Length, Point, Rectangle,
    advanced::{
        Widget, layout, mouse, overlay,
        renderer::Quad,
        widget::{Id, Tree, operation},
    },
    keyboard::{self, Key, key},
    time::Instant,
    touch,
};
use iced_widget::core::text;

use super::*;
use crate::{
    animation::motion::{self, Spring, SpringValue, fast_effects, fast_spatial},
    widget::{internal::geometry::expanded_bounds, ripple::Ripple},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Thumb {
    Lower,
    Upper,
}

struct State<P: text::Paragraph> {
    drag: Option<(Thumb, Option<f32>)>,

    drag_display: f32,
    active: Thumb,

    common: CommonState<P>,

    ripples: [Ripple; 2],
    lower_width_spring: SpringValue<f32>,
    upper_width_spring: SpringValue<f32>,
    lower_position_spring: SpringValue<f32>,
    upper_position_spring: SpringValue<f32>,

    label_anchor_offset_spring: SpringValue<f32>,
}

impl<P: text::Paragraph> State<P> {
    fn position_spring(&self, thumb: Thumb) -> f32 {
        match thumb {
            Thumb::Lower => self.lower_position_spring.value(),
            Thumb::Upper => self.upper_position_spring.value(),
        }
    }

    fn position_spring_mut(&mut self, thumb: Thumb) -> &mut SpringValue<f32> {
        match thumb {
            Thumb::Lower => &mut self.lower_position_spring,
            Thumb::Upper => &mut self.upper_position_spring,
        }
    }

    fn position_spring_at_rest(&self, thumb: Thumb) -> bool {
        match thumb {
            Thumb::Lower => self.lower_position_spring.is_at_rest(),
            Thumb::Upper => self.upper_position_spring.is_at_rest(),
        }
    }

    fn width_spring(&self, thumb: Thumb) -> f32 {
        match thumb {
            Thumb::Lower => self.lower_width_spring.value(),
            Thumb::Upper => self.upper_width_spring.value(),
        }
    }

    fn is_animating(&self) -> bool {
        self.common.is_animating()
            || !self.lower_width_spring.is_at_rest()
            || !self.upper_width_spring.is_at_rest()
            || !self.lower_position_spring.is_at_rest()
            || !self.upper_position_spring.is_at_rest()
            || !self.label_anchor_offset_spring.is_at_rest()
    }

    // Chase cursor during continuous drag, then step all springs.
    fn step(&mut self, now: Instant, continuous_drag: bool) {
        if continuous_drag && let Some((thumb, Some(target))) = self.drag {
            let spring = match thumb {
                Thumb::Lower => &mut self.lower_position_spring,
                Thumb::Upper => &mut self.upper_position_spring,
            };
            let delta = now.duration_since(spring.spring.time).as_secs_f32();
            let decay = 1.0 - (-delta / DRAG_CHASE_DECAY).exp();
            self.drag_display += (target - self.drag_display) * decay;
            spring.reset(self.drag_display, now);
        }

        self.common.step(now);
        self.lower_width_spring.step(now);
        self.upper_width_spring.step(now);
        step_handle_spring(&mut self.lower_position_spring, now);
        step_handle_spring(&mut self.upper_position_spring, now);
        self.label_anchor_offset_spring.step(now);
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

pub struct RangeSlider<'a, T, Message> {
    id: Option<Id>,
    range: RangeInclusive<T>,
    step: T,
    shift_step: Option<T>,
    lower: T,
    upper: T,
    min_separation: Option<T>,
    lower_enabled: bool,
    upper_enabled: bool,
    on_change: Option<Box<dyn Fn(T, T) -> Message + 'a>>,
    on_release: Option<Message>,
    width: Length,
    height: Length,
    style: Style,
    size: Size,
    axis: Axis,
    stops: bool,
    continuous_tick_count: Option<u32>,
    value_indicator: ValueIndicator<'a, T>,
    decimal_places: usize,
    effects_spring: Option<Spring>,
    spatial_spring: Option<Spring>,
}

impl<'a, T, Message> RangeSlider<'a, T, Message>
where
    T: Copy + From<u8> + PartialOrd,
{
    #[must_use]
    pub fn new<F>(range: RangeInclusive<T>, start: T, end: T, on_change: F, style: Style) -> Self
    where
        F: 'a + Fn(T, T) -> Message,
    {
        let (start, end) = if start <= end {
            (start, end)
        } else {
            (end, start)
        };
        let clamp = |mut value: T| {
            if value < *range.start() {
                value = *range.start();
            }
            if value > *range.end() {
                value = *range.end();
            }
            value
        };
        let (lower, upper) = (clamp(start), clamp(end));
        let (lower, upper) = if lower <= upper {
            (lower, upper)
        } else {
            (upper, lower)
        };

        RangeSlider {
            id: None,
            range,

            step: T::from(1),
            shift_step: None,
            lower,
            upper,
            min_separation: None,
            lower_enabled: true,
            upper_enabled: true,
            on_change: Some(Box::new(on_change)),
            on_release: None,
            width: Length::Fill,
            height: Length::Fill,
            style,
            size: Size::default(),
            axis: Axis::default(),
            stops: true,
            continuous_tick_count: None,
            value_indicator: ValueIndicator::None,
            decimal_places: 1,
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
        F: 'a + Fn(T, T) -> Message,
    {
        self.on_change = on_change.map(|on_change| Box::new(on_change) as _);
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
    pub fn min_separation(mut self, min_separation: impl Into<T>) -> Self {
        self.min_separation = Some(min_separation.into());
        self
    }

    #[must_use]
    pub fn lower_enabled(mut self, enabled: bool) -> Self {
        self.lower_enabled = enabled;
        self
    }

    #[must_use]
    pub fn upper_enabled(mut self, enabled: bool) -> Self {
        self.upper_enabled = enabled;
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

impl<T, Message> RangeSlider<'_, T, Message> {
    fn enabled_of(&self, thumb: Thumb) -> bool {
        self.on_change.is_some()
            && match thumb {
                Thumb::Lower => self.lower_enabled,
                Thumb::Upper => self.upper_enabled,
            }
    }

    fn is_enabled(&self) -> bool {
        self.enabled_of(Thumb::Lower) || self.enabled_of(Thumb::Upper)
    }
}

impl<T, Message> RangeSlider<'_, T, Message>
where
    T: Copy + Display + PartialOrd,
{
    fn value_of(&self, thumb: Thumb) -> T {
        match thumb {
            Thumb::Lower => self.lower,
            Thumb::Upper => self.upper,
        }
    }

    fn format_value(&self, thumb: Thumb) -> Option<String>
    where
        T: Into<f64>,
    {
        let step_f64: f64 = self.step.into();
        let min_decimal_places = step_decimal_places(step_f64);
        let decimal_places = self.decimal_places.max(min_decimal_places);
        self.value_indicator
            .format(self.value_of(thumb), decimal_places, min_decimal_places)
    }
}

impl<T, Message> RangeSlider<'_, T, Message>
where
    T: Copy + Into<f64> + PartialOrd + num_traits::FromPrimitive,
{
    fn move_thumb(&mut self, thumb: Thumb, value: T) -> Thumb {
        if !self.enabled_of(thumb) {
            return thumb;
        }

        let lower: f64 = self.lower.into();
        let upper: f64 = self.upper.into();
        let value: f64 = value.into();
        let full_range = range_f64(&self.range);
        let separation = self
            .min_separation
            .map(|separation: T| separation.into())
            .unwrap_or(0.0)
            .min(full_range.end - full_range.start)
            .max(0.0);

        let (new_lower, new_upper, thumb) = match thumb {
            Thumb::Lower if separation <= 0.0 && self.upper_enabled && value >= upper => {
                (upper, value, Thumb::Upper)
            }
            Thumb::Lower => (
                value.min(upper - separation).max(full_range.start),
                upper,
                Thumb::Lower,
            ),
            Thumb::Upper if separation <= 0.0 && self.lower_enabled && value <= lower => {
                (value, lower, Thumb::Lower)
            }
            Thumb::Upper => (
                lower,
                value.max(lower + separation).min(full_range.end),
                Thumb::Upper,
            ),
        };

        self.lower = T::from_f64(new_lower).unwrap_or(self.lower);
        self.upper = T::from_f64(new_upper).unwrap_or(self.upper);
        thumb
    }
}

impl<T, Message, Theme, Renderer> Widget<Message, Theme, Renderer> for RangeSlider<'_, T, Message>
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
        let label_content = self.format_value(Thumb::Upper).unwrap_or_default();

        let state: State<Renderer::Paragraph> = State {
            drag: None,
            drag_display: value_factor(self.upper, &self.range),
            active: Thumb::Upper,
            common: CommonState::new(
                now,
                effects_spring,
                spatial_spring,
                self.style,
                label_content,
                Renderer::ICON_FONT,
            ),
            ripples: [Ripple::new(), Ripple::new()],
            lower_width_spring: SpringValue::new(HANDLE_WIDTH_IDLE, spatial_spring, now),
            upper_width_spring: SpringValue::new(HANDLE_WIDTH_IDLE, spatial_spring, now),
            lower_position_spring: SpringValue::new(
                value_factor(self.lower, &self.range),
                spatial_spring,
                now,
            ),
            upper_position_spring: SpringValue::new(
                value_factor(self.upper, &self.range),
                spatial_spring,
                now,
            ),
            label_anchor_offset_spring: SpringValue::new(0.0, spatial_spring, now),
        };

        iced::advanced::widget::tree::State::new(state)
    }

    fn diff(&self, tree: &mut Tree) {
        let state = tree.state.downcast_mut::<State<Renderer::Paragraph>>();

        if !self.is_enabled() {
            state.drag = None;
        }

        for thumb in [Thumb::Lower, Thumb::Upper] {
            if !self.enabled_of(thumb) {
                if state.active == thumb {
                    state.active = match thumb {
                        Thumb::Lower => Thumb::Upper,
                        Thumb::Upper => Thumb::Lower,
                    };
                }
                if state.drag.is_some_and(|(dragged, _)| dragged == thumb) {
                    state.drag = None;
                }
            }
        }

        // Sync handle positions with current values when not dragging.
        // This ensures the visual position reflects the value even when
        // the widget is rebuilt between events (e.g., via view()).
        if state.drag.is_none() {
            let now = Instant::now();
            set_handle_target(
                &mut state.lower_position_spring,
                value_factor(self.lower, &self.range),
                now,
            );
            set_handle_target(
                &mut state.upper_position_spring,
                value_factor(self.upper, &self.range),
                now,
            );
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
        let enabled = self.is_enabled();
        let now = Instant::now();
        let len = main_len(self.axis, bounds);

        let active_at_entry = state.active;

        let lower_circle = handle_circle(
            self.axis,
            bounds,
            self.size.handle_height,
            center_at(
                len,
                state.width_spring(Thumb::Lower),
                state.position_spring(Thumb::Lower),
            ),
        );
        let upper_circle = handle_circle(
            self.axis,
            bounds,
            self.size.handle_height,
            center_at(
                len,
                state.width_spring(Thumb::Upper),
                state.position_spring(Thumb::Upper),
            ),
        );

        let [lower_ripple, upper_ripple] = &mut state.ripples;
        for (ripple, circle, thumb, thumb_enabled) in [
            (
                lower_ripple,
                lower_circle,
                Thumb::Lower,
                self.enabled_of(Thumb::Lower),
            ),
            (
                upper_ripple,
                upper_circle,
                Thumb::Upper,
                self.enabled_of(Thumb::Upper),
            ),
        ] {
            let interaction_bounds = if state.drag.is_some_and(|(t, _)| t == thumb) {
                expanded_bounds(bounds)
            } else {
                circle
            };
            ripple.handle_in(
                event,
                interaction_bounds,
                circle,
                cursor,
                thumb_enabled,
                shell,
            );
        }

        sync_spring_model(
            &mut state.common.last_effects_spring,
            self.effects_spring,
            &mut [
                &mut state.common.label_scale_spring.spring,
                &mut state.common.focus_ring_spring.spring,
                &mut state.common.style_spring.spring,
            ],
        );
        sync_spring_model(
            &mut state.common.last_spatial_spring,
            self.spatial_spring,
            &mut [
                &mut state.lower_width_spring.spring,
                &mut state.upper_width_spring.spring,
                &mut state.lower_position_spring.spring,
                &mut state.upper_position_spring.spring,
                &mut state.label_anchor_offset_spring.spring,
            ],
        );

        let axis = self.axis;
        let range = self.range.clone();
        let range_step = effective_step(self.step);
        let full_range = range_f64(&range);
        let pixels_per_step = if range_step > 0.0 {
            len as f64 * range_step / (full_range.end - full_range.start)
        } else {
            0.0
        };
        let continuous_drag = pixels_per_step < MIN_PIXELS_PER_STEP;

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

        let lower_target = value_factor(self.lower, &self.range);
        let upper_target = value_factor(self.upper, &self.range);

        if !matches!(state.drag, Some((Thumb::Lower, Some(_)))) || !continuous_drag {
            set_handle_target(&mut state.lower_position_spring, lower_target, now);
        }
        if !matches!(state.drag, Some((Thumb::Upper, Some(_)))) || !continuous_drag {
            set_handle_target(&mut state.upper_position_spring, upper_target, now);
        }
        state.lower_width_spring.set_target(
            match state.drag {
                Some((Thumb::Lower, _)) => HANDLE_WIDTH_DRAGGED,
                _ => HANDLE_WIDTH_IDLE,
            },
            now,
        );
        state.upper_width_spring.set_target(
            match state.drag {
                Some((Thumb::Upper, _)) => HANDLE_WIDTH_DRAGGED,
                _ => HANDLE_WIDTH_IDLE,
            },
            now,
        );

        let active_circle_at_entry = match active_at_entry {
            Thumb::Lower => lower_circle,
            Thumb::Upper => upper_circle,
        };
        let cursor_over_active_at_entry = cursor.is_over(bounds)
            && self.enabled_of(active_at_entry)
            && cursor.is_over(active_circle_at_entry);

        let label_following_at_entry = label_follows_handle(
            state.position_spring_at_rest(active_at_entry),
            state.common.label_scale_spring.to > 0.5,
        );
        let label_visible_at_entry = !matches!(self.value_indicator, ValueIndicator::None)
            && enabled
            && (state.drag.is_some()
                || cursor_over_active_at_entry
                || state.common.keyboard_focus
                || label_following_at_entry);

        if state.drag.is_none() && cursor.is_over(bounds) {
            let lower_hit = self.enabled_of(Thumb::Lower) && cursor.is_over(lower_circle);
            let upper_hit = self.enabled_of(Thumb::Upper) && cursor.is_over(upper_circle);

            match (lower_hit, upper_hit) {
                (true, false) => state.active = Thumb::Lower,
                (false, true) => state.active = Thumb::Upper,
                (true, true) => {
                    let cursor_pos =
                        cursor_main(self.axis, bounds, cursor.position().unwrap_or_default());
                    let lower_center = center_at(
                        len,
                        state.width_spring(Thumb::Lower),
                        state.position_spring(Thumb::Lower),
                    );
                    let upper_center = center_at(
                        len,
                        state.width_spring(Thumb::Upper),
                        state.position_spring(Thumb::Upper),
                    );
                    state.active =
                        if (cursor_pos - lower_center).abs() <= (cursor_pos - upper_center).abs() {
                            Thumb::Lower
                        } else {
                            Thumb::Upper
                        };
                }
                (false, false) => {}
            }
        }

        let focus_target = (state.common.keyboard_focus && enabled) as u32 as f32;
        state
            .common
            .label_scale_spring
            .set_target(label_visible_at_entry as u32 as f32, now);
        state.common.focus_ring_spring.set_target(focus_target, now);

        let label_content = self.format_value(state.active).unwrap_or_default();
        sync_label_content(
            &mut state.common.label_paragraph,
            &mut state.common.label_content,
            label_content,
            renderer.default_font(),
        );

        let drag_is_chasing = state.drag.is_some_and(|(_, target)| {
            target.is_some_and(|target| (target - state.drag_display).abs() > 0.001)
        });
        if state.is_animating() || drag_is_chasing {
            shell.request_redraw();
        }

        state.step(now, continuous_drag);

        let locate = move |position: Point| -> Option<T> {
            value_at(axis, bounds, position, &range, range_step)
        };

        let publish =
            |slider: &Self, shell: &mut iced::advanced::Shell<'_, Message>, previous: (T, T)| {
                let changed = (previous.0.into() - slider.lower.into()).abs() > f64::EPSILON
                    || (previous.1.into() - slider.upper.into()).abs() > f64::EPSILON;
                if changed && let Some(on_change) = &slider.on_change {
                    shell.publish(on_change(slider.lower, slider.upper));
                }
            };

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                if enabled
                    && !shell.is_event_captured()
                    && let Some(position) = cursor.position_over(bounds)
                {
                    if !state.common.is_focused {
                        state.common.is_focused = true;
                        shell.request_redraw();
                    }

                    state.common.keyboard_focus = false;

                    let press_main = cursor_main(self.axis, bounds, position);
                    let lower_center = center_at(
                        len,
                        state.width_spring(Thumb::Lower),
                        state.position_spring(Thumb::Lower),
                    );
                    let upper_center = center_at(
                        len,
                        state.width_spring(Thumb::Upper),
                        state.position_spring(Thumb::Upper),
                    );
                    let thumb = if !self.enabled_of(Thumb::Lower) {
                        Thumb::Upper
                    } else if !self.enabled_of(Thumb::Upper)
                        || (press_main - lower_center).abs() <= (press_main - upper_center).abs()
                    {
                        Thumb::Lower
                    } else {
                        Thumb::Upper
                    };

                    if let Some(new_value) = locate(position) {
                        let previous = (self.lower, self.upper);
                        state.active = self.move_thumb(thumb, new_value);

                        state.drag = Some((state.active, None));
                        publish(self, shell, previous);
                    }

                    shell.capture_event();
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerLifted { .. })
            | Event::Touch(touch::Event::FingerLost { .. }) => {
                if let Some((thumb, chase)) = state.drag.take() {
                    // Let the thumb settle onto its final position; the value is
                    // already final, only the drawn position moves.
                    let settle = if range_step > 0.0 {
                        Some(value_factor(self.value_of(thumb), &self.range))
                    } else {
                        chase
                    };

                    if continuous_drag && let Some(target) = settle {
                        settle_handle_spring(state.position_spring_mut(thumb), target, now);
                    }

                    if let Some(on_release) = self.on_release.clone() {
                        shell.publish(on_release);
                    }
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { .. })
            | Event::Touch(touch::Event::FingerMoved { .. }) => {
                if let Some((thumb, _)) = state.drag
                    && enabled
                {
                    if let Some(position) = cursor.land().position()
                        && let Some(new_value) = locate(position)
                    {
                        let current_value = self.value_of(thumb);
                        let value_changed =
                            (current_value.into() - new_value.into()).abs() > f64::EPSILON;

                        let previous = (self.lower, self.upper);
                        if value_changed {
                            state.active = self.move_thumb(thumb, new_value);

                            if state.active != thumb {
                                mem::swap(
                                    &mut state.lower_position_spring,
                                    &mut state.upper_position_spring,
                                );
                                mem::swap(
                                    &mut state.lower_width_spring,
                                    &mut state.upper_width_spring,
                                );
                            }

                            if !continuous_drag {
                                let new_target =
                                    value_factor(self.value_of(state.active), &self.range);
                                set_handle_target(
                                    state.position_spring_mut(state.active),
                                    new_target,
                                    now,
                                );
                            }
                        }

                        if state.drag.is_some_and(|(_, factor)| factor.is_none()) {
                            state.drag_display = state.position_spring(state.active);
                        }

                        let current_width = state.width_spring(state.active);
                        let continuous_factor =
                            factor_at(len, current_width, cursor_main(axis, bounds, position));
                        let drag_factor = if continuous_drag {
                            continuous_factor
                        } else {
                            value_factor(new_value, &self.range)
                        };
                        if !continuous_drag {
                            state.drag_display = drag_factor;
                        }
                        state.drag = Some((state.active, Some(drag_factor)));
                        if value_changed {
                            publish(self, shell, previous);
                        }
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

                    let current: f64 = self.value_of(state.active).into();
                    let new_value = wheel_value(scroll_delta(delta), current, step, &range);

                    if let Some(new_value) = T::from_f64(new_value) {
                        let previous = (self.lower, self.upper);
                        state.active = self.move_thumb(state.active, new_value);
                        publish(self, shell, previous);
                    }

                    shell.capture_event();
                }
            }
            Event::Keyboard(keyboard::Event::KeyPressed { key, .. }) => {
                if state.common.is_focused && enabled {
                    let range = range_f64(&self.range);
                    let step = keyboard_step(
                        self.step,
                        self.shift_step,
                        state.common.keyboard_modifiers.shift(),
                        &range,
                    );
                    let current: f64 = self.value_of(state.active).into();
                    let new_value = keyboard_value(key, current, step, &range);

                    if let Some(new_value) = new_value
                        && let Some(new_value) = T::from_f64(new_value)
                    {
                        let previous = (self.lower, self.upper);
                        state.active = self.move_thumb(state.active, new_value);
                        publish(self, shell, previous);
                        shell.capture_event();
                    }
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

        let active_circle = match state.active {
            Thumb::Lower => lower_circle,
            Thumb::Upper => upper_circle,
        };
        let cursor_over_active = cursor.is_over(bounds)
            && self.enabled_of(state.active)
            && cursor.is_over(active_circle);
        let label_following = label_follows_handle(
            state.position_spring_at_rest(state.active),
            state.common.label_scale_spring.to > 0.5,
        );
        let label_visible = !matches!(self.value_indicator, ValueIndicator::None)
            && enabled
            && (state.drag.is_some()
                || cursor_over_active
                || state.common.keyboard_focus
                || label_following);
        state
            .common
            .label_scale_spring
            .set_target(label_visible as u32 as f32, now);

        if state.active != active_at_entry {
            let label_should_travel = label_visible_at_entry && label_visible;
            if label_should_travel {
                let previous_center = center_at(
                    len,
                    state.width_spring(active_at_entry),
                    state.position_spring(active_at_entry),
                );
                let current_center = center_at(
                    len,
                    state.width_spring(state.active),
                    state.position_spring(state.active),
                );
                let offset = state.label_anchor_offset_spring.value();
                let travel = match self.axis {
                    Axis::Horizontal => previous_center - current_center + offset,

                    Axis::Vertical => current_center - previous_center + offset,
                };
                state.label_anchor_offset_spring.reset(travel, now);
            } else {
                state.label_anchor_offset_spring.reset(0.0, now);
            }
        }
        state.label_anchor_offset_spring.set_target(0.0, now);
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &iced::advanced::renderer::Style,
        layout: layout::Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &iced::Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State<Renderer::Paragraph>>();
        let style = state.common.style_spring.value();
        let enabled = self.is_enabled();
        let bounds = layout.bounds();
        let len = main_len(self.axis, bounds);
        let track_height = self.size.track_height;

        let lower_width = state.lower_width_spring.value();
        let lower_center = center_at(len, lower_width, state.lower_position_spring.value());
        let upper_width = state.upper_width_spring.value();
        let upper_center = center_at(len, upper_width, state.upper_position_spring.value());

        let cross_center = cross_center(self.axis, bounds);

        let active = Segment::new(
            lower_center + lower_width / 2.0 + HANDLE_GAP,
            upper_center - upper_width / 2.0 - HANDLE_GAP,
            false,
            false,
        );
        let inactive = [
            Segment::new(
                0.0,
                lower_center - lower_width / 2.0 - HANDLE_GAP,
                true,
                false,
            ),
            Segment::new(
                upper_center + upper_width / 2.0 + HANDLE_GAP,
                len,
                false,
                true,
            ),
        ];

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

        let overlap =
            (upper_center - lower_center).abs() <= (lower_width + upper_width) / 2.0 + f32::EPSILON;
        let lower = handle_rect(
            self.axis,
            bounds,
            self.size.handle_height,
            lower_width,
            lower_center,
        );
        let upper = handle_rect(
            self.axis,
            bounds,
            self.size.handle_height,
            upper_width,
            upper_center,
        );

        let draw_handle =
            |renderer: &mut Renderer, handle: Rectangle, thumb: Thumb, outline: bool| {
                renderer.fill_quad(
                    Quad {
                        bounds: handle,
                        border: if outline {
                            Border::default()
                                .rounded(f32::MAX)
                                .width(HANDLE_OVERLAP_WIDTH)
                                .color(style.handle_overlap_outline)
                        } else {
                            Border::default().rounded(f32::MAX)
                        },
                        ..Quad::default()
                    },
                    if self.enabled_of(thumb) {
                        style.handle
                    } else {
                        style.handle_disabled
                    },
                );
            };

        match state.active {
            Thumb::Upper => {
                draw_handle(renderer, lower, Thumb::Lower, overlap);
                draw_handle(renderer, upper, Thumb::Upper, false);
            }
            Thumb::Lower => {
                draw_handle(renderer, upper, Thumb::Upper, overlap);
                draw_handle(renderer, lower, Thumb::Lower, false);
            }
        }
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
            !self.is_enabled(),
            state.drag.is_some(),
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
        let enabled = self.is_enabled();

        let widget_bounds = layout.bounds();
        let bounds = Rectangle {
            x: widget_bounds.x + translation.x,
            y: widget_bounds.y + translation.y,
            ..widget_bounds
        };
        let len = main_len(self.axis, bounds);

        let lower_circle = handle_circle(
            self.axis,
            bounds,
            self.size.handle_height,
            center_at(
                len,
                state.width_spring(Thumb::Lower),
                state.position_spring(Thumb::Lower),
            ),
        );
        let upper_circle = handle_circle(
            self.axis,
            bounds,
            self.size.handle_height,
            center_at(
                len,
                state.width_spring(Thumb::Upper),
                state.position_spring(Thumb::Upper),
            ),
        );

        let style = state.common.style_spring.value();
        let focus = state.common.focus_ring_spring.value();
        let label_scale = state.common.label_scale_spring.value();

        let ripple_layer = |ripple: Ripple, circle: Rectangle, thumb: Thumb| {
            let enabled = self.enabled_of(thumb);
            ripple
                .is_visible(circle.size(), enabled)
                .then_some(RippleLayer {
                    ripple,
                    layer: style.state_layer,
                    bounds: circle,
                    enabled,
                })
        };
        let ripples = [
            ripple_layer(state.ripples[0], lower_circle, Thumb::Lower),
            ripple_layer(state.ripples[1], upper_circle, Thumb::Upper),
        ];

        let active_circle = match state.active {
            Thumb::Lower => lower_circle,
            Thumb::Upper => upper_circle,
        };
        let focus_ring = if focus > 0.01 && enabled && state.common.keyboard_focus {
            focus_ring_bounds(
                active_circle,
                self.size.handle_height,
                focus,
                style.focus_ring,
            )
        } else {
            None
        };

        let active_factor = state.position_spring(state.active);
        let active_handle = handle_rect(
            self.axis,
            bounds,
            self.size.handle_height,
            state.width_spring(state.active),
            center_at(len, state.width_spring(state.active), active_factor),
        );
        let label = value_label(
            self.axis,
            &active_handle,
            state.label_anchor_offset_spring.value(),
            &state.common.label_paragraph,
            label_scale,
            style,
            &state.common.label_content,
        );

        if ripples.iter().all(Option::is_none) && focus_ring.is_none() && label.is_none() {
            return None;
        }

        let overlay_element = SliderOverlay {
            ripples,
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
    RangeSlider<'a, T, Message> => Element<'a, Message, Theme, Renderer>
    where
        T: Copy + Display + Into<f64> + num_traits::FromPrimitive + PartialOrd + 'a,
        Message: Clone + 'a,
        Renderer: iced::advanced::text::Renderer
            + iced::advanced::svg::Renderer
            + iced_widget::renderer::geometry::Renderer
            + 'a,
        Renderer::Font: From<iced::Font>,
);
