//! Tooltips display a short description of another element when it is hovered.

use iced::{
    Border, Color, Element, Event, Pixels, Point, Rectangle, Shadow, Size, Vector,
    advanced::{
        Clipboard, Layout, Shell, Widget,
        layout::{Limits, Node},
        mouse, overlay, renderer,
        renderer::Quad,
        text::{self, Paragraph, Text},
        widget::{Tree, tree},
    },
    alignment::Vertical,
    time::{Duration, Instant},
};
use iced_widget::text::IntoFragment;

use crate::{
    style::{Elevation, shadow},
    theme::ColorScheme,
};

/// Size of the label, as specified for tooltips by Material 3 (body small).
const LABEL_TEXT_SIZE: Pixels = Pixels(12.0);
/// Line height of the label (body small).
const LABEL_TEXT_LINE_HEIGHT: Pixels = Pixels(16.0);
pub(crate) const PADDING_HORIZONTAL: f32 = 8.0;
const PADDING_VERTICAL: f32 = 4.0;
const CORNER_RADIUS: f32 = 4.0;
pub(crate) const DEFAULT_MAX_WIDTH: f32 = 200.0;
pub(crate) const DEFAULT_GAP: f32 = 8.0;
const VIEWPORT_MARGIN: f32 = 4.0;
pub(crate) const DEFAULT_DELAY: Duration = Duration::from_millis(500);

/// The side of the target the tooltip is placed on.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Position {
    /// Above the target, flipped below when there is not enough room.
    #[default]
    Above,
    /// Below the target, flipped above when there is not enough room.
    Below,
    /// To the left of the target, flipped to the right when there is not enough room.
    Left,
    /// To the right of the target, flipped to the left when there is not enough room.
    Right,
    /// Centered on the target, covering it instead of sitting next to it.
    Cover,
    /// Near the cursor, following it while it moves.
    FollowCursor,
    /// Beside the target, trying the sides in `order` until one of them fits.
    ///
    /// When none of the sides has room, the tooltip is not shown at all.
    Auto(Order),
}

impl Position {
    /// Places the tooltip beside the target, preferring the [`Order::RIGHT_LEFT_ABOVE_BELOW`]
    /// sides.
    #[must_use]
    pub const fn auto() -> Self {
        Self::Auto(Order::RIGHT_LEFT_ABOVE_BELOW)
    }
}

/// One side of a target, as used by [`Order`] and [`Position::Auto`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Side {
    /// Above the target.
    Above,
    /// Below the target.
    Below,
    /// To the left of the target.
    Left,
    /// To the right of the target.
    Right,
}

impl Side {
    /// The side facing `self` across the target.
    #[must_use]
    pub const fn opposite(self) -> Self {
        match self {
            Self::Above => Self::Below,
            Self::Below => Self::Above,
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }
}

/// The order in which [`Position::Auto`] tries the sides of its target.
///
/// A side is used as soon as the whole tooltip fits next to the target. Before moving on to the next
/// side, the tooltip slides along the current one, so a bit lower or higher placement on the same
/// side is preferred over switching sides.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Order([Side; 4]);

impl Order {
    /// Right, then left, then above, then below.
    ///
    /// Tooltips sit next to the target, which suits wide or tall targets that span most of the side
    /// they are anchored to, such as menu rows.
    pub const RIGHT_LEFT_ABOVE_BELOW: Self =
        Self([Side::Right, Side::Left, Side::Above, Side::Below]);

    /// Above, then below, then right, then left.
    ///
    /// Tooltips stack against the target, which keeps them clear of whatever is beside it.
    pub const ABOVE_BELOW_RIGHT_LEFT: Self =
        Self([Side::Above, Side::Below, Side::Right, Side::Left]);

    /// Creates a custom order, tried from first to last.
    #[must_use]
    pub const fn new(sides: [Side; 4]) -> Self {
        Self(sides)
    }
}

impl Default for Order {
    fn default() -> Self {
        Self::RIGHT_LEFT_ABOVE_BELOW
    }
}

/// The appearance of a [`Tooltip`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// The color of the tooltip container.
    pub container_color: Color,
    /// The color of the label.
    pub label_color: Color,
    /// The border of the tooltip container.
    pub container_border: Border,
    /// The shadow cast by the tooltip container.
    pub shadow: Shadow,
}

impl Style {
    /// Creates the default Material 3 tooltip style for the given theme.
    ///
    /// The container uses a raised surface of the current scheme, so the tooltip stays dark with
    /// light text in a dark theme and light with dark text in a light theme.
    #[must_use]
    pub fn new(theme: &(impl ColorScheme + ?Sized)) -> Self {
        Self {
            container_color: theme.surface_container_highest(),
            label_color: theme.on_surface(),
            container_border: Border::default().rounded(CORNER_RADIUS),
            shadow: shadow(theme.shadow(), Elevation::Level2),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
enum State {
    #[default]
    Idle,
    Hovered {
        at: Instant,
    },
    Open {
        cursor: Point,
    },
}

impl State {
    fn cursor(self) -> Option<Point> {
        match self {
            Self::Open { cursor } => Some(cursor),
            _ => None,
        }
    }

    /// Advances the hover state machine for a pointer at `cursor`.
    ///
    /// `follow_cursor` keeps an open tooltip attached to the pointer while it moves. `shell` is
    /// only touched when the state actually changes, so callers can forward every mouse and redraw
    /// event without filtering them further.
    fn advance<Message>(
        &mut self,
        event: &Event,
        cursor: Option<Point>,
        delay: Duration,
        follow_cursor: bool,
        shell: &mut Shell<'_, Message>,
    ) {
        let now = Instant::now();

        match (*self, cursor) {
            (Self::Idle, Some(cursor)) => {
                if delay == Duration::ZERO {
                    *self = Self::Open { cursor };
                    shell.invalidate_layout();
                } else {
                    *self = Self::Hovered { at: now };
                    shell.request_redraw_at(now + delay);
                }
            }
            (Self::Hovered { .. }, None) => {
                *self = Self::Idle;
            }
            (Self::Hovered { at }, Some(cursor)) => {
                let elapsed = at.elapsed();

                if elapsed < delay {
                    shell.request_redraw_at(now + delay - elapsed);
                } else {
                    *self = Self::Open { cursor };
                    shell.invalidate_layout();
                }
            }
            (Self::Open { cursor: previous }, Some(cursor))
                if follow_cursor && previous != cursor =>
            {
                *self = Self::Open { cursor };
                shell.request_redraw();
            }
            (Self::Open { .. }, None) => {
                *self = Self::Idle;
                shell.invalidate_layout();

                if !is_redraw(event) {
                    shell.request_redraw();
                }
            }
            (Self::Open { .. }, Some(_)) | (Self::Idle, None) => {}
        }
    }
}

/// Whether `event` can change the hover state of a tooltip.
fn is_hover_event(event: &Event) -> bool {
    matches!(event, Event::Mouse(_)) || is_redraw(event)
}

fn is_redraw(event: &Event) -> bool {
    matches!(
        event,
        Event::Window(iced::window::Event::RedrawRequested(_))
    )
}

/// Shows a short description of `base` when the pointer hovers it.
///
/// The label can be changed at any time; passing `None` disables the tooltip while keeping the
/// wrapped element interactive.
pub struct Tooltip<'a, Message, Theme, Renderer>
where
    Renderer: 'a + text::Renderer,
{
    style: Style,
    base: Element<'a, Message, Theme, Renderer>,
    label: Option<text::Fragment<'a>>,
    label_font: Option<iced::Font>,
    position: Position,
    gap: f32,
    max_width: f32,
    delay: Duration,
}

impl<'a, Message, Theme, Renderer> Tooltip<'a, Message, Theme, Renderer>
where
    Renderer: 'a + text::Renderer,
    Renderer::Font: From<iced::Font>,
{
    /// Creates a new [`Tooltip`] around `base`.
    #[must_use]
    pub fn new(style: Style, base: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        Self {
            style,
            base: base.into(),
            label: None,
            label_font: None,
            position: Position::default(),
            gap: DEFAULT_GAP,
            max_width: DEFAULT_MAX_WIDTH,
            delay: DEFAULT_DELAY,
        }
    }

    /// Sets the text shown inside the tooltip.
    #[must_use]
    pub fn label(mut self, label: impl IntoFragment<'a>) -> Self {
        self.label = Some(label.into_fragment());
        self
    }

    /// Sets the text shown inside the tooltip, if `Some`.
    #[must_use]
    pub fn label_maybe(mut self, label: Option<impl IntoFragment<'a>>) -> Self {
        self.label = label.map(|label| label.into_fragment());
        self
    }

    /// Sets the font of the label.
    #[must_use]
    pub fn label_font(mut self, font: iced::Font) -> Self {
        self.label_font = Some(font);
        self
    }

    crate::macros::maybe_assign!(label_font_maybe, label_font, Option<iced::Font>);

    /// Sets the side the tooltip is placed on.
    #[must_use]
    pub fn position(mut self, position: Position) -> Self {
        self.position = position;
        self
    }

    /// Sets the side the tooltip is placed on, if `Some`.
    #[must_use]
    pub fn position_maybe(mut self, position: Option<Position>) -> Self {
        if let Some(position) = position {
            self.position = position;
        }
        self
    }

    /// Sets the gap between the target and the tooltip.
    #[must_use]
    pub fn gap(mut self, gap: impl Into<Pixels>) -> Self {
        self.gap = gap.into().0;
        self
    }

    /// Sets the maximum width of the tooltip. Longer labels wrap.
    #[must_use]
    pub fn max_width(mut self, max_width: impl Into<Pixels>) -> Self {
        self.max_width = max_width.into().0;
        self
    }

    /// Sets how long the pointer must rest on the target before the tooltip opens.
    ///
    /// Set to [`Duration::ZERO`] to open immediately.
    #[must_use]
    pub fn delay(mut self, delay: Duration) -> Self {
        self.delay = delay;
        self
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Tooltip<'a, Message, Theme, Renderer>
where
    Renderer: 'a + text::Renderer,
    Renderer::Font: From<iced::Font>,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn size(&self) -> Size<iced::Length> {
        self.base.as_widget().size()
    }

    fn size_hint(&self) -> Size<iced::Length> {
        self.base.as_widget().size_hint()
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(self.base.as_widget())]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[self.base.as_widget()]);
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &Renderer, limits: &Limits) -> Node {
        self.base
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        if self.label.is_some() && is_hover_event(event) {
            tree.state.downcast_mut::<State>().advance(
                event,
                cursor.position_over(layout.bounds()),
                self.delay,
                self.position == Position::FollowCursor,
                shell,
            );
        }

        self.base.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.base.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        inherited_style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.base.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            inherited_style,
            layout,
            cursor,
            viewport,
        );
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn iced::advanced::widget::Operation,
    ) {
        self.base
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        let state = *tree.state.downcast_ref::<State>();

        let base = self.base.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        );

        let anchor = Rectangle::new(
            layout.bounds().position() + translation,
            layout.bounds().size(),
        );

        let tooltip = match (state, self.label.as_ref()) {
            (State::Open { .. }, Some(label)) => Some(label_overlay(
                anchor,
                label.clone(),
                self.label_font.map(Renderer::Font::from),
                self.style,
                self.position,
                self.gap,
                self.max_width,
                state.cursor(),
            )),
            _ => None,
        };

        if base.is_some() || tooltip.is_some() {
            Some(overlay::Group::with_children(base.into_iter().chain(tooltip).collect()).overlay())
        } else {
            None
        }
    }
}

/// A hover-driven tooltip state machine for widgets that draw their own targets.
///
/// Widgets with hand-drawn icon buttons can embed one of these in their state, feed it events from
/// `update`, and turn it into a [`label_overlay`] from `overlay`. Passing `enabled = false`
/// disables the tooltip without affecting the wrapped widget's interactivity.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub(crate) struct Hover {
    state: State,
    /// The element the pointer was last checked against.
    ///
    /// Moving to another element restarts the delay, so every target gets its own hover timer
    /// instead of inheriting the state of the previous one.
    target: Option<Rectangle>,
}

impl Hover {
    /// Whether the tooltip is currently shown.
    pub(crate) fn is_open(&self) -> bool {
        matches!(self.state, State::Open { .. })
    }

    /// Advances the state machine for the hoverable `target`.
    ///
    /// `target` is expressed in the same coordinate space as `cursor`. Passing `None` or
    /// `enabled = false` disables the tooltip and resets the hover timer.
    pub(crate) fn update<Message>(
        &mut self,
        event: &Event,
        cursor: mouse::Cursor,
        target: Option<Rectangle>,
        enabled: bool,
        delay: Duration,
        shell: &mut Shell<'_, Message>,
    ) {
        if !is_hover_event(event) {
            return;
        }

        let target = target.filter(|_| enabled);

        // A different target means the pointer moved to another hoverable element, so the tooltip
        // has to wait for its own delay instead of reusing the state of the previous element.
        if self.target != target {
            let was_open = self.is_open();

            self.state = State::Idle;
            self.target = target;

            if was_open {
                shell.invalidate_layout();
            }
        }

        let Some(target) = target else {
            return;
        };

        self.state
            .advance(event, cursor.position_over(target), delay, false, shell);
    }
}

/// Builds a standalone label tooltip overlay anchored to `anchor`.
///
/// Used by widgets that draw their own text (such as a [`Select`](crate::widget::Select)) to show
/// the full label when the visible text does not fit.
pub(crate) fn label_overlay<'a, Message, Theme, Renderer>(
    anchor: Rectangle,
    label: text::Fragment<'a>,
    font: Option<Renderer::Font>,
    style: Style,
    position: Position,
    gap: f32,
    max_width: f32,
    cursor: Option<Point>,
) -> overlay::Element<'a, Message, Theme, Renderer>
where
    Renderer: 'a + text::Renderer,
{
    overlay::Element::new(Box::new(LabelOverlay::<Renderer> {
        anchor,
        label,
        font,
        style,
        position,
        gap,
        max_width,
        cursor,
        renderer: std::marker::PhantomData,
    }))
}

struct LabelOverlay<'a, Renderer>
where
    Renderer: text::Renderer,
{
    anchor: Rectangle,
    label: text::Fragment<'a>,
    font: Option<Renderer::Font>,
    style: Style,
    position: Position,
    gap: f32,
    max_width: f32,
    cursor: Option<Point>,
    renderer: std::marker::PhantomData<Renderer>,
}

impl<Renderer> LabelOverlay<'_, Renderer>
where
    Renderer: text::Renderer,
{
    /// The maximum width available to the label, excluding the padding.
    fn max_content(&self) -> f32 {
        (self.max_width - 2.0 * PADDING_HORIZONTAL).max(1.0)
    }

    /// The size of the label box for a container of the given `bounds`.
    fn content_size(&self, bounds: Rectangle) -> Size {
        Size::new(
            self.max_content(),
            (bounds.height - 2.0 * PADDING_VERTICAL).max(0.0),
        )
    }

    /// Builds the label with the typography shared by shaping and drawing.
    fn text<Content>(
        &self,
        renderer: &Renderer,
        content: Content,
        bounds: Size,
    ) -> Text<Content, Renderer::Font> {
        Text {
            content,
            bounds,
            size: LABEL_TEXT_SIZE,
            line_height: text::LineHeight::Absolute(LABEL_TEXT_LINE_HEIGHT),
            font: self.font.unwrap_or_else(|| renderer.default_font()),
            shaping: text::Shaping::Advanced,
            wrapping: text::Wrapping::Word,
            align_x: text::Alignment::Left,
            align_y: Vertical::Top,
        }
    }

    /// Shapes the label.
    ///
    /// The overlay element is recreated by the runtime before every draw call, so the paragraph
    /// must not be cached between `layout` and `draw`.
    fn paragraph(&self, renderer: &Renderer) -> Renderer::Paragraph {
        <Renderer::Paragraph as Paragraph>::with_text(self.text(
            renderer,
            self.label.as_ref(),
            Size::new(self.max_content(), f32::INFINITY),
        ))
    }
}

impl<Message, Theme, Renderer> overlay::Overlay<Message, Theme, Renderer>
    for LabelOverlay<'_, Renderer>
where
    Renderer: text::Renderer,
{
    fn layout(&mut self, renderer: &Renderer, bounds: Size) -> Node {
        let content = self.paragraph(renderer).min_bounds();
        let content = Size::new(content.width.min(self.max_content()), content.height);
        let size = Size::new(
            content.width + 2.0 * PADDING_HORIZONTAL,
            content.height + 2.0 * PADDING_VERTICAL,
        );

        let Some(position) = place(
            self.anchor,
            size,
            bounds,
            self.position,
            self.gap,
            VIEWPORT_MARGIN,
            self.cursor,
        ) else {
            // None of the sides of the anchor had room for the tooltip. The runtime draws an overlay
            // from a fresh instance and the cached node, so this has to be encoded in the node:
            // the empty bounds are skipped by `draw`.
            return Node::new(Size::ZERO);
        };

        Node::new(size).move_to(position)
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
    ) {
        let bounds = layout.bounds();

        if bounds.width <= 0.0 || bounds.height <= 0.0 {
            // The tooltip did not fit anywhere; see `layout`.
            return;
        }

        renderer.fill_quad(
            Quad {
                bounds,
                border: self.style.container_border,
                shadow: self.style.shadow,
                ..Quad::default()
            },
            self.style.container_color,
        );

        // Backends keep shaped paragraphs behind a `Weak`, so a paragraph built here would be
        // dropped before it is rasterized. `fill_text` hands an owned string to the renderer's
        // text cache instead, which survives until the frame is presented.
        let text = self.text(renderer, self.label.to_string(), self.content_size(bounds));

        renderer.fill_text(
            text,
            bounds.position() + Vector::new(PADDING_HORIZONTAL, PADDING_VERTICAL),
            self.style.label_color,
            bounds,
        );
    }

    fn index(&self) -> f32 {
        // Tooltips belong on top of any overlay owned by the wrapped element.
        2.0
    }
}

/// Places a `size` box next to `anchor`, keeping it inside `viewport`.
///
/// Returns `None` only for [`Position::Auto`], when no side of the anchor has room for the box.
fn place(
    anchor: Rectangle,
    size: Size,
    viewport: Size,
    position: Position,
    gap: f32,
    margin: f32,
    cursor: Option<Point>,
) -> Option<Point> {
    Some(match position {
        Position::Above => preferred(anchor, size, viewport, Side::Above, gap, margin),
        Position::Below => preferred(anchor, size, viewport, Side::Below, gap, margin),
        Position::Left => preferred(anchor, size, viewport, Side::Left, gap, margin),
        Position::Right => preferred(anchor, size, viewport, Side::Right, gap, margin),
        Position::Cover => aligned(anchor, size),
        Position::FollowCursor => {
            let cursor = cursor.unwrap_or(anchor.center());

            Point::new(cursor.x, cursor.y - gap - size.height)
        }
        Position::Auto(order) => order
            .0
            .into_iter()
            .find_map(|side| beside(anchor, size, viewport, side, gap, margin))?,
    })
}

/// Places the box on `side`, flipping to the opposite side only when `side` has no room at all.
///
/// The result is always clamped into the viewport so that a tooltip pinned to a single side keeps
/// being shown, unlike [`Position::Auto`] which gives up instead.
fn preferred(
    anchor: Rectangle,
    size: Size,
    viewport: Size,
    side: Side,
    gap: f32,
    margin: f32,
) -> Point {
    beside(anchor, size, viewport, side, gap, margin)
        .or_else(|| beside(anchor, size, viewport, side.opposite(), gap, margin))
        .unwrap_or_else(|| clamp(offset(anchor, size, side, gap), size, viewport, margin))
}

/// Places the box on `side` of `anchor`, sliding it along that side to keep it inside `viewport`.
///
/// Returns `None` when the box does not fit on that side, either because the side itself is too
/// narrow or because the box is larger than the viewport.
fn beside(
    anchor: Rectangle,
    size: Size,
    viewport: Size,
    side: Side,
    gap: f32,
    margin: f32,
) -> Option<Point> {
    let point = offset(anchor, size, side, gap);

    // Slide along the side: the box keeps its distance from the anchor but dodges the edge of the
    // viewport, so a slightly shifted position is preferred over another side.
    let point = match side {
        Side::Above | Side::Below => {
            Point::new(slide(point.x, size.width, viewport.width, margin), point.y)
        }
        Side::Left | Side::Right => Point::new(
            point.x,
            slide(point.y, size.height, viewport.height, margin),
        ),
    };

    fits(point, size, viewport, margin).then_some(point)
}

/// Position of the box on `side` of `anchor`, aligned with the center of the anchor.
fn offset(anchor: Rectangle, size: Size, side: Side, gap: f32) -> Point {
    let center = aligned(anchor, size);

    match side {
        Side::Above => Point::new(center.x, anchor.y - gap - size.height),
        Side::Below => Point::new(center.x, anchor.y + anchor.height + gap),
        Side::Left => Point::new(anchor.x - gap - size.width, center.y),
        Side::Right => Point::new(anchor.x + anchor.width + gap, center.y),
    }
}

/// Position of the box centered on `anchor`.
fn aligned(anchor: Rectangle, size: Size) -> Point {
    Point::new(
        anchor.center_x() - size.width / 2.0,
        anchor.center_y() - size.height / 2.0,
    )
}

/// Moves the box into the viewport, keeping `margin` on every side.
fn clamp(point: Point, size: Size, viewport: Size, margin: f32) -> Point {
    Point::new(
        slide(point.x, size.width, viewport.width, margin),
        slide(point.y, size.height, viewport.height, margin),
    )
}

/// Moves `value` so that a `length` box stays inside `total`, keeping `margin` on both ends.
fn slide(value: f32, length: f32, total: f32, margin: f32) -> f32 {
    value.clamp(margin, (total - length - margin).max(margin))
}

/// Whether a `size` box at `point` stays inside `viewport` with `margin` on every side.
fn fits(point: Point, size: Size, viewport: Size, margin: f32) -> bool {
    point.x >= margin
        && point.y >= margin
        && point.x + size.width <= viewport.width - margin
        && point.y + size.height <= viewport.height - margin
}

crate::macros::impl_element_from!(
    ['a, Message, Theme, Renderer]
    Tooltip<'a, Message, Theme, Renderer> => Element<'a, Message, Theme, Renderer>
    where
        Message: 'a,
        Theme: 'a,
        Renderer: 'a + text::Renderer,
        Renderer::Font: From<iced::Font>,
);

#[cfg(test)]
mod tests {
    use super::*;

    const VIEWPORT: Size = Size::new(800.0, 600.0);

    #[test]
    fn places_above_when_there_is_room() {
        let anchor = Rectangle::new(Point::new(400.0, 300.0), Size::new(100.0, 40.0));
        let size = Size::new(120.0, 24.0);

        let position = place(anchor, size, VIEWPORT, Position::Above, 8.0, 4.0, None);

        assert_eq!(position, Some(Point::new(390.0, 268.0)));
    }

    #[test]
    fn covers_the_anchor() {
        let anchor = Rectangle::new(Point::new(400.0, 300.0), Size::new(100.0, 40.0));
        let size = Size::new(120.0, 24.0);

        let position = place(anchor, size, VIEWPORT, Position::Cover, 8.0, 4.0, None);

        assert_eq!(position, Some(Point::new(390.0, 308.0)));
    }

    #[test]
    fn flips_below_when_above_does_not_fit() {
        let anchor = Rectangle::new(Point::new(400.0, 10.0), Size::new(100.0, 40.0));
        let size = Size::new(120.0, 24.0);

        let position = place(anchor, size, VIEWPORT, Position::Above, 8.0, 4.0, None);

        assert_eq!(position, Some(Point::new(390.0, 58.0)));
    }

    #[test]
    fn flips_above_when_below_does_not_fit() {
        let anchor = Rectangle::new(Point::new(400.0, 560.0), Size::new(100.0, 40.0));
        let size = Size::new(120.0, 24.0);

        let position = place(anchor, size, VIEWPORT, Position::Below, 8.0, 4.0, None);

        assert_eq!(position, Some(Point::new(390.0, 528.0)));
    }

    #[test]
    fn clamps_to_the_viewport_edges() {
        let anchor = Rectangle::new(Point::new(-50.0, 300.0), Size::new(40.0, 40.0));
        let size = Size::new(120.0, 24.0);

        let position = place(anchor, size, VIEWPORT, Position::Above, 8.0, 4.0, None);

        assert_eq!(position.map(|position| position.x), Some(4.0));
    }

    #[test]
    fn follows_the_cursor() {
        let anchor = Rectangle::new(Point::new(400.0, 300.0), Size::new(100.0, 40.0));
        let size = Size::new(120.0, 24.0);
        let cursor = Some(Point::new(500.0, 350.0));

        let position = place(
            anchor,
            size,
            VIEWPORT,
            Position::FollowCursor,
            8.0,
            4.0,
            cursor,
        );

        assert_eq!(position, Some(Point::new(500.0, 318.0)));
    }

    #[test]
    fn auto_prefers_the_right_side() {
        let anchor = Rectangle::new(Point::new(200.0, 300.0), Size::new(100.0, 40.0));
        let size = Size::new(120.0, 24.0);

        let position = place(anchor, size, VIEWPORT, Position::auto(), 8.0, 4.0, None);

        assert_eq!(position, Some(Point::new(308.0, 308.0)));
    }

    #[test]
    fn auto_slides_along_the_side_before_moving_on() {
        // The right side has room, but only once the tooltip slides up: its centered position would
        // hang off the bottom of the viewport.
        let anchor = Rectangle::new(Point::new(200.0, 540.0), Size::new(100.0, 40.0));
        let size = Size::new(120.0, 200.0);

        let position = place(anchor, size, VIEWPORT, Position::auto(), 8.0, 4.0, None);

        assert_eq!(position, Some(Point::new(308.0, 396.0)));
    }

    #[test]
    fn auto_follows_the_requested_order() {
        let anchor = Rectangle::new(Point::new(200.0, 300.0), Size::new(100.0, 40.0));
        let size = Size::new(120.0, 24.0);

        let position = place(
            anchor,
            size,
            VIEWPORT,
            Position::Auto(Order::ABOVE_BELOW_RIGHT_LEFT),
            8.0,
            4.0,
            None,
        );

        assert_eq!(position, Some(Point::new(190.0, 268.0)));

        let position = place(
            anchor,
            size,
            VIEWPORT,
            Position::Auto(Order::new([
                Side::Below,
                Side::Above,
                Side::Left,
                Side::Right,
            ])),
            8.0,
            4.0,
            None,
        );

        assert_eq!(position, Some(Point::new(190.0, 348.0)));
    }

    #[test]
    fn auto_falls_back_to_the_left_side() {
        let anchor = Rectangle::new(Point::new(700.0, 300.0), Size::new(96.0, 40.0));
        let size = Size::new(120.0, 24.0);

        let position = place(anchor, size, VIEWPORT, Position::auto(), 8.0, 4.0, None);

        assert_eq!(position, Some(Point::new(572.0, 308.0)));
    }

    #[test]
    fn auto_falls_back_above_and_below() {
        let anchor = Rectangle::new(Point::new(340.0, 500.0), Size::new(120.0, 40.0));
        let size = Size::new(400.0, 24.0);

        let position = place(anchor, size, VIEWPORT, Position::auto(), 8.0, 4.0, None);

        assert_eq!(position, Some(Point::new(200.0, 468.0)));

        let anchor = Rectangle::new(Point::new(340.0, 10.0), Size::new(120.0, 40.0));

        let position = place(anchor, size, VIEWPORT, Position::auto(), 8.0, 4.0, None);

        assert_eq!(position, Some(Point::new(200.0, 58.0)));
    }

    #[test]
    fn auto_hides_when_no_side_fits() {
        let anchor = Rectangle::new(Point::new(300.0, 300.0), Size::new(200.0, 40.0));
        let size = Size::new(795.0, 24.0);

        let position = place(anchor, size, VIEWPORT, Position::auto(), 8.0, 4.0, None);

        assert_eq!(position, None);
    }

    #[test]
    fn single_side_slides_to_stay_on_screen() {
        // The box is taller than the space left below the anchor, so it slides up instead of being
        // pinned to the viewport or dropped.
        let anchor = Rectangle::new(Point::new(200.0, 560.0), Size::new(100.0, 40.0));
        let size = Size::new(120.0, 200.0);

        let position = place(anchor, size, VIEWPORT, Position::Right, 8.0, 4.0, None);

        assert_eq!(position, Some(Point::new(308.0, 396.0)));
    }

    /// Moving the pointer to another hoverable element must wait for the delay again instead of
    /// reusing the hover that opened the previous tooltip.
    #[test]
    fn changing_the_target_restarts_the_delay() {
        let delay = Duration::from_millis(500);
        let first = Rectangle::new(Point::new(0.0, 0.0), Size::new(100.0, 40.0));
        let second = Rectangle::new(Point::new(0.0, 100.0), Size::new(100.0, 40.0));
        let position = Point::new(50.0, 120.0);
        let event = Event::Mouse(mouse::Event::CursorMoved { position });
        let cursor = mouse::Cursor::Available(position);
        let mut messages: Vec<()> = Vec::new();
        let mut hover = Hover {
            state: State::Open {
                cursor: Point::new(50.0, 20.0),
            },
            target: Some(first),
        };

        let mut shell = Shell::new(&mut messages);
        hover.update(&event, cursor, Some(second), true, delay, &mut shell);

        assert!(!hover.is_open());
        assert!(matches!(hover.state, State::Hovered { .. }));
        assert_eq!(hover.target, Some(second));

        // Staying on the same target keeps counting the same delay instead of restarting it.
        let mut shell = Shell::new(&mut messages);
        hover.update(&event, cursor, Some(second), true, delay, &mut shell);

        assert!(matches!(hover.state, State::Hovered { .. }));
    }

    /// Disabling a tooltip cancels a pending hover, so it waits for the full delay once enabled
    /// again.
    #[test]
    fn disabling_resets_the_hover() {
        let target = Rectangle::new(Point::new(0.0, 0.0), Size::new(100.0, 40.0));
        let event = Event::Mouse(mouse::Event::CursorMoved {
            position: Point::new(50.0, 20.0),
        });
        let mut messages: Vec<()> = Vec::new();
        let mut hover = Hover {
            state: State::Open {
                cursor: Point::new(50.0, 20.0),
            },
            target: Some(target),
        };

        let mut shell = Shell::new(&mut messages);
        hover.update(
            &event,
            mouse::Cursor::Available(Point::new(50.0, 20.0)),
            Some(target),
            false,
            Duration::from_millis(500),
            &mut shell,
        );

        assert!(!hover.is_open());
        assert_eq!(hover.state, State::Idle);
        assert_eq!(hover.target, None);
    }

    /// A renderer that records what was drawn instead of painting it.
    #[derive(Debug, Default)]
    struct Recording {
        quads: Vec<Rectangle>,
        texts: Vec<(String, Point)>,
    }

    impl iced::advanced::Renderer for Recording {
        fn start_layer(&mut self, _bounds: Rectangle) {}

        fn end_layer(&mut self) {}

        fn start_transformation(&mut self, _transformation: iced::Transformation) {}

        fn end_transformation(&mut self) {}

        fn reset(&mut self, _new_bounds: Rectangle) {}

        fn fill_quad(&mut self, quad: Quad, _background: impl Into<iced::Background>) {
            self.quads.push(quad.bounds);
        }

        fn allocate_image(
            &mut self,
            _handle: &iced::advanced::image::Handle,
            _callback: impl FnOnce(
                Result<iced::advanced::image::Allocation, iced::advanced::image::Error>,
            ) + Send
            + 'static,
        ) {
        }
    }

    /// A paragraph that estimates its size from the amount of characters.
    #[derive(Debug, Default)]
    struct MockParagraph {
        width: f32,
        height: f32,
        size: Pixels,
        line_height: text::LineHeight,
    }

    impl Paragraph for MockParagraph {
        type Font = iced::Font;

        fn with_text(text: Text<&str, Self::Font>) -> Self {
            Self {
                width: (text.content.chars().count() as f32 * 6.0).min(text.bounds.width),
                height: text.line_height.to_absolute(text.size).0,
                size: text.size,
                line_height: text.line_height,
            }
        }

        fn with_spans<Link>(_text: Text<&[text::Span<'_, Link, Self::Font>], Self::Font>) -> Self {
            Self::default()
        }

        fn resize(&mut self, new_bounds: Size) {
            self.width = self.width.min(new_bounds.width);
        }

        fn compare(&self, _text: Text<(), Self::Font>) -> text::Difference {
            text::Difference::None
        }

        fn size(&self) -> Pixels {
            self.size
        }

        fn font(&self) -> Self::Font {
            iced::Font::DEFAULT
        }

        fn line_height(&self) -> text::LineHeight {
            self.line_height
        }

        fn align_x(&self) -> text::Alignment {
            text::Alignment::Left
        }

        fn align_y(&self) -> Vertical {
            Vertical::Top
        }

        fn wrapping(&self) -> text::Wrapping {
            text::Wrapping::Word
        }

        fn shaping(&self) -> text::Shaping {
            text::Shaping::Advanced
        }

        fn bounds(&self) -> Size {
            Size::new(self.width, self.height)
        }

        fn min_bounds(&self) -> Size {
            Size::new(self.width, self.height)
        }

        fn hit_test(&self, _point: Point) -> Option<text::Hit> {
            None
        }

        fn hit_span(&self, _point: Point) -> Option<usize> {
            None
        }

        fn span_bounds(&self, _index: usize) -> Vec<Rectangle> {
            Vec::new()
        }

        fn grapheme_position(&self, _line: usize, _index: usize) -> Option<Point> {
            None
        }
    }

    #[derive(Debug, Default)]
    struct MockEditor;

    impl text::Editor for MockEditor {
        type Font = iced::Font;

        fn with_text(_text: &str) -> Self {
            Self
        }

        fn is_empty(&self) -> bool {
            true
        }

        fn cursor(&self) -> text::editor::Cursor {
            text::editor::Cursor {
                position: text::editor::Position { line: 0, column: 0 },
                selection: None,
            }
        }

        fn selection(&self) -> text::editor::Selection {
            text::editor::Selection::Caret(Point::ORIGIN)
        }

        fn copy(&self) -> Option<String> {
            None
        }

        fn line(&self, _index: usize) -> Option<text::editor::Line<'_>> {
            None
        }

        fn line_count(&self) -> usize {
            0
        }

        fn perform(&mut self, _action: text::editor::Action) {}

        fn move_to(&mut self, _cursor: text::editor::Cursor) {}

        fn bounds(&self) -> Size {
            Size::ZERO
        }

        fn min_bounds(&self) -> Size {
            Size::ZERO
        }

        fn update(
            &mut self,
            _new_bounds: Size,
            _new_font: Self::Font,
            _new_size: Pixels,
            _new_line_height: text::LineHeight,
            _new_wrapping: text::Wrapping,
            _new_highlighter: &mut impl text::Highlighter,
        ) {
        }

        fn highlight<H: text::Highlighter>(
            &mut self,
            _font: Self::Font,
            _highlighter: &mut H,
            _format_highlight: impl Fn(&H::Highlight) -> text::highlighter::Format<Self::Font>,
        ) {
        }
    }

    impl text::Renderer for Recording {
        type Font = iced::Font;
        type Paragraph = MockParagraph;
        type Editor = MockEditor;

        const ICON_FONT: iced::Font = iced::Font::DEFAULT;
        const CHECKMARK_ICON: char = '0';
        const ARROW_DOWN_ICON: char = '0';
        const SCROLL_UP_ICON: char = '0';
        const SCROLL_DOWN_ICON: char = '0';
        const SCROLL_LEFT_ICON: char = '0';
        const SCROLL_RIGHT_ICON: char = '0';
        const ICED_LOGO: char = '0';

        fn default_font(&self) -> Self::Font {
            iced::Font::DEFAULT
        }

        fn default_size(&self) -> Pixels {
            Pixels(16.0)
        }

        fn fill_paragraph(
            &mut self,
            _text: &Self::Paragraph,
            _position: Point,
            _color: Color,
            _clip_bounds: Rectangle,
        ) {
        }

        fn fill_editor(
            &mut self,
            _editor: &Self::Editor,
            _position: Point,
            _color: Color,
            _clip_bounds: Rectangle,
        ) {
        }

        fn fill_text(
            &mut self,
            text: Text,
            position: Point,
            _color: Color,
            _clip_bounds: Rectangle,
        ) {
            self.texts.push((text.content, position));
        }
    }

    fn label_overlay_with<'a>(
        anchor: Rectangle,
        label: &'a str,
        position: Position,
    ) -> overlay::Element<'a, (), (), Recording> {
        label_overlay(
            anchor,
            text::Fragment::Borrowed(label),
            None,
            Style {
                container_color: Color::BLACK,
                label_color: Color::WHITE,
                container_border: Border::default().rounded(CORNER_RADIUS),
                shadow: Shadow::default(),
            },
            position,
            DEFAULT_GAP,
            DEFAULT_MAX_WIDTH,
            None,
        )
    }

    /// The runtime lays overlays out during `update`, drops them, and recreates them to `draw` with
    /// the cached layout node. The label must therefore not rely on state computed in `layout`.
    #[test]
    fn draws_when_recreated_after_layout() {
        let anchor = Rectangle::new(Point::new(400.0, 300.0), Size::new(100.0, 40.0));
        let mut renderer = Recording::default();

        let mut laid_out = label_overlay_with(anchor, "Add", Position::Above);
        let node = laid_out.as_overlay_mut().layout(&renderer, VIEWPORT);
        drop(laid_out);

        let drawn = label_overlay_with(anchor, "Add", Position::Above);
        drawn.as_overlay().draw(
            &mut renderer,
            &(),
            &renderer::Style::default(),
            Layout::new(&node),
            mouse::Cursor::Unavailable,
        );

        assert_eq!(renderer.quads, vec![node.bounds()]);
        assert_eq!(
            renderer.texts,
            vec![(
                String::from("Add"),
                node.bounds().position() + Vector::new(PADDING_HORIZONTAL, PADDING_VERTICAL)
            )]
        );
    }

    /// A tooltip that cannot fit on any side of its anchor is laid out as an empty node so that
    /// `draw` skips it, even though the runtime recreates the overlay before drawing.
    #[test]
    fn hidden_when_no_side_has_room() {
        let anchor = Rectangle::new(Point::new(4.0, 4.0), Size::new(100.0, 40.0));
        let mut renderer = Recording::default();

        let mut laid_out = label_overlay_with(anchor, "Add", Position::auto());
        let node = laid_out
            .as_overlay_mut()
            .layout(&renderer, Size::new(20.0, 20.0));
        drop(laid_out);

        assert_eq!(node.bounds().size(), Size::ZERO);

        let drawn = label_overlay_with(anchor, "Add", Position::auto());
        drawn.as_overlay().draw(
            &mut renderer,
            &(),
            &renderer::Style::default(),
            Layout::new(&node),
            mouse::Cursor::Unavailable,
        );

        assert!(renderer.quads.is_empty());
        assert!(renderer.texts.is_empty());
    }
}
