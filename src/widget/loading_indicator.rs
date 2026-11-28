//! Loading indicators show the progress of a process for a short wait time

#[cfg(feature = "shapes")]
use std::f32::consts::PI;
use std::f32::consts::TAU;

use iced::{
    Color, Element, Pixels, Point, Radians,
    time::{Duration, Instant},
};
#[cfg(feature = "shapes")]
use iced::{Size, Vector};
#[cfg(feature = "shapes")]
use iced_widget::canvas::Fill;
use iced_widget::{Canvas, canvas};

use crate::theme::ColorScheme;
#[cfg(feature = "shapes")]
use crate::{animation::motion::SimulatedSpring, widget::shapes::cubics_to_iced_path};

pub(crate) mod constants {
    use iced::Pixels;

    pub const DEFAULT_CONTAINER_SIZE: Pixels = Pixels(48.0);
    pub const DEFAULT_INDICATOR_SIZE: Pixels = Pixels(38.0);

    #[cfg(feature = "shapes")]
    use crate::animation::motion::Spring;
    #[cfg(feature = "shapes")]
    use polymorpher::{
        Morph, RoundedPolygon,
        shapes::{cookie4, cookie9, oval, pentagon, pill, soft_burst, sunny},
    };
    #[cfg(feature = "shapes")]
    use std::sync::LazyLock;

    #[cfg(feature = "shapes")]
    pub const LINEAR_ROTATION_CYCLE_SECONDS: f32 = 4.666;
    #[cfg(feature = "shapes")]
    pub const MORPH_CYCLE_SECONDS: f32 = 0.650;
    // The Compose reference morphs with `spring(dampingRatio = 0.6, stiffness =
    // 200, visibilityThreshold = 0.1)`, hardcoded and independent from the
    // motion scheme. The threshold just ends the spring early; the morph
    // cycle clock plays the same role here.
    #[cfg(feature = "shapes")]
    pub const MORPH_SPRING: Spring = Spring::new(0.6, 200.0);
    #[cfg(feature = "shapes")]
    pub static CYCLE_POLYGONS: LazyLock<[RoundedPolygon; 7]> = LazyLock::new(|| {
        [
            soft_burst(),
            cookie9(),
            pentagon(),
            pill(),
            sunny(),
            cookie4(),
            oval(),
        ]
    });
    #[cfg(feature = "shapes")]
    pub static MORPHS: LazyLock<[Morph; 7]> = LazyLock::new(|| {
        [
            Morph::new(soft_burst(), cookie9()),
            Morph::new(cookie9(), pentagon()),
            Morph::new(pentagon(), pill()),
            Morph::new(pill(), sunny()),
            Morph::new(sunny(), cookie4()),
            Morph::new(cookie4(), oval()),
            Morph::new(oval(), soft_burst()),
        ]
    });
    #[cfg(feature = "shapes")]
    pub static INDICATOR_SCALING_FACTOR: LazyLock<f32> = LazyLock::new(|| {
        let mut factor = f32::MAX;
        for polygon in &*CYCLE_POLYGONS {
            let bounds = polygon.aabb(false).max;
            factor = factor.min(bounds.x.min(bounds.y));
        }
        factor
    });
}

#[cfg(feature = "pub-internal-const")]
pub use constants::*;

#[derive(Clone, Copy, PartialEq)]
pub enum Style {
    Contained {
        indicator_color: Color,
        container_color: Color,
    },
    Uncontained {
        indicator_color: Color,
    },
}

impl Style {
    pub fn contained(theme: &(impl ColorScheme + ?Sized)) -> Self {
        Self::Contained {
            indicator_color: theme.on_primary_container(),
            container_color: theme.primary_container(),
        }
    }

    pub fn uncontained(theme: &(impl ColorScheme + ?Sized)) -> Self {
        Self::Uncontained {
            indicator_color: theme.primary(),
        }
    }
}

// TODO: Custom sizes and animation params
/// Loading indicators show the progress of a process for a short wait time
#[cfg(feature = "shapes")]
pub struct LoadingIndicator {
    style: Style,
    size: Pixels,
}

#[cfg(feature = "shapes")]
impl LoadingIndicator {
    #[must_use]
    pub fn new(style: Style) -> Self {
        Self {
            style,
            size: constants::DEFAULT_CONTAINER_SIZE,
        }
    }

    #[must_use]
    pub fn size(mut self, size: impl Into<Pixels>) -> Self {
        self.size = size.into();
        self
    }
}

#[cfg(feature = "shapes")]
pub struct State {
    start: Instant,
    last_cycle: Duration,
    spring: SimulatedSpring,
}

#[cfg(feature = "shapes")]
impl State {
    pub fn new(now: Instant) -> Self {
        Self {
            start: now,
            spring: SimulatedSpring::new(constants::MORPH_SPRING, now),
            last_cycle: Duration::default(),
        }
    }
}

#[cfg(feature = "shapes")]
impl Default for State {
    fn default() -> Self {
        Self::new(Instant::now())
    }
}

#[cfg(feature = "shapes")]
impl<Message, Theme, Renderer> canvas::Program<Message, Theme, Renderer> for LoadingIndicator
where
    Renderer: iced_widget::renderer::geometry::Renderer,
{
    type State = State;

    fn draw(
        &self,
        state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: iced::Rectangle,
        _cursor: iced::advanced::mouse::Cursor,
    ) -> Vec<canvas::Geometry<Renderer>> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());

        let (indicator_color, container_color) = match self.style {
            Style::Contained {
                indicator_color,
                container_color,
            } => (indicator_color, Some(container_color)),
            Style::Uncontained { indicator_color } => (indicator_color, None),
        };

        if let Some(color) = container_color {
            let radius = self.size.0 / 2.0;
            let circle = canvas::Path::circle(frame.center(), radius);
            frame.fill(&circle, color);
        }

        let indicator_size = (self.size.0 / constants::DEFAULT_CONTAINER_SIZE.0)
            * constants::DEFAULT_INDICATOR_SIZE.0;
        let offset = Vector::new(
            (frame.width() - indicator_size) / 2.0,
            (frame.height() - indicator_size) / 2.0,
        );
        let half = indicator_size / 2.0;

        let current_morph = &constants::MORPHS[(state.last_cycle.as_secs_f32()
            / constants::MORPH_CYCLE_SECONDS)
            as usize
            % constants::MORPHS.len()];

        let path = cubics_to_iced_path(
            &current_morph.as_cubics(state.spring.position),
            Size::new(indicator_size, indicator_size),
        );

        let elapsed = Instant::now().duration_since(state.start).as_secs_f32();
        let morph_turns =
            (state.last_cycle.as_secs_f32() / constants::MORPH_CYCLE_SECONDS + 1.0) % 4.0;
        let angle = Radians(
            elapsed * std::f32::consts::TAU / constants::LINEAR_ROTATION_CYCLE_SECONDS
                + PI / 2.0 * (state.spring.position + morph_turns),
        );

        frame.translate(Vector::new(offset.x + half, offset.y + half));
        frame.rotate(angle);
        frame.scale(*constants::INDICATOR_SCALING_FACTOR);
        frame.translate(Vector::new(-half, -half));

        frame.fill(
            &path,
            Fill {
                style: canvas::Style::Solid(indicator_color),
                rule: canvas::fill::Rule::NonZero,
            },
        );

        vec![frame.into_geometry()]
    }

    fn update(
        &self,
        state: &mut Self::State,
        _event: &iced::Event,
        _bounds: iced::Rectangle,
        _cursor: iced::advanced::mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        let now = Instant::now();
        state.spring.step(now);

        let elapsed = Instant::now().duration_since(state.start);
        if elapsed.as_secs_f32() - state.last_cycle.as_secs_f32() > constants::MORPH_CYCLE_SECONDS {
            state.last_cycle += Duration::from_secs_f32(constants::MORPH_CYCLE_SECONDS);
            state.spring.reset(now);
        }

        Some(canvas::Action::request_redraw())
    }
}

#[cfg(feature = "shapes")]
impl<'a, Message, Theme, Renderer> From<LoadingIndicator> for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: 'a + iced_widget::renderer::geometry::Renderer,
{
    fn from(value: LoadingIndicator) -> Self {
        let size = value.size.0;
        Element::new(Canvas::new(value).width(size).height(size))
    }
}

const SPINNER_CYCLE: Duration = Duration::from_millis(1333);
const SPINNER_ARC_MIN: Radians = Radians(std::f32::consts::PI / 6.0);
const SPINNER_ARC_MAX: Radians = Radians(3.0 * std::f32::consts::PI / 2.0);

/// A classic indeterminate circular spinner.
pub struct CircularLoadingIndicator {
    style: Style,
    size: Pixels,
}

impl CircularLoadingIndicator {
    #[must_use]
    pub fn new(style: Style) -> Self {
        Self {
            style,
            size: constants::DEFAULT_CONTAINER_SIZE,
        }
    }

    #[must_use]
    pub fn size(mut self, size: impl Into<Pixels>) -> Self {
        self.size = size.into();
        self
    }
}

pub struct CircularState {
    start: Instant,
}

impl CircularState {
    pub fn new(start: Instant) -> Self {
        Self { start }
    }
}

impl Default for CircularState {
    fn default() -> Self {
        Self::new(Instant::now())
    }
}

impl<Message, Theme, Renderer> canvas::Program<Message, Theme, Renderer>
    for CircularLoadingIndicator
where
    Renderer: iced_widget::renderer::geometry::Renderer,
{
    type State = CircularState;

    fn draw(
        &self,
        state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: iced::Rectangle,
        _cursor: iced::advanced::mouse::Cursor,
    ) -> Vec<canvas::Geometry<Renderer>> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());

        let (indicator_color, container_color) = match self.style {
            Style::Contained {
                indicator_color,
                container_color,
            } => (indicator_color, Some(container_color)),
            Style::Uncontained { indicator_color } => (indicator_color, None),
        };

        if let Some(color) = container_color {
            let radius = self.size.0 / 2.0;
            let circle = canvas::Path::circle(frame.center(), radius);
            frame.fill(&circle, color);
        }

        let indicator_size = (self.size.0 / constants::DEFAULT_CONTAINER_SIZE.0)
            * constants::DEFAULT_INDICATOR_SIZE.0;
        let stroke_width = (indicator_size * (4.0 / constants::DEFAULT_INDICATOR_SIZE.0))
            .max(2.0)
            .min(indicator_size / 2.0);
        let radius = ((indicator_size - stroke_width) / 2.0).max(0.0);

        let elapsed = Instant::now().duration_since(state.start).as_secs_f32();
        let cycle = (elapsed / SPINNER_CYCLE.as_secs_f32()).fract();

        // Head eases out, tail eases in: both edges keep moving forward.
        let start = cycle.powi(3) * TAU;
        let sweep = SPINNER_ARC_MIN.0
            + (SPINNER_ARC_MAX.0 - SPINNER_ARC_MIN.0) * 4.0 * cycle * (1.0 - cycle);

        let path = arc_path(frame.center(), radius, start, sweep);
        let stroke = canvas::Stroke::default()
            .with_color(indicator_color)
            .with_width(stroke_width)
            .with_line_cap(canvas::LineCap::Round);

        frame.stroke(&path, stroke);

        vec![frame.into_geometry()]
    }

    fn update(
        &self,
        _state: &mut Self::State,
        _event: &iced::Event,
        _bounds: iced::Rectangle,
        _cursor: iced::advanced::mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        Some(canvas::Action::request_redraw())
    }
}

fn arc_path(center: Point, radius: f32, start: f32, sweep: f32) -> canvas::Path {
    let steps = ((sweep.abs() * radius / 1.5).ceil() as usize).clamp(2, 128);

    canvas::Path::new(|builder| {
        for i in 0..=steps {
            let angle = start + sweep * (i as f32 / steps as f32);
            let point = Point::new(
                center.x + radius * angle.cos(),
                center.y + radius * angle.sin(),
            );

            if i == 0 {
                builder.move_to(point);
            } else {
                builder.line_to(point);
            }
        }
    })
}

impl<'a, Message, Theme, Renderer> From<CircularLoadingIndicator>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: 'a + iced_widget::renderer::geometry::Renderer,
{
    fn from(value: CircularLoadingIndicator) -> Self {
        let size = value.size.0;
        Element::new(Canvas::new(value).width(size).height(size))
    }
}
