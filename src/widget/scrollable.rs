use iced::advanced::widget::{Tree, Widget, tree};
use iced::advanced::{Clipboard, Layout, Shell, layout, mouse, renderer};
use iced::widget::container;
use iced::widget::scrollable::{
    Anchor, AutoScroll, Catalog, Direction, Rail, Scrollbar, Scroller, Status, Style,
};
use iced::{
    Background, Border, Color, Element, Event, Length, Point, Rectangle, Shadow, Size, Vector,
};

use crate::theme::ColorScheme;
use crate::widget::internal::scrollbar;

#[derive(Default)]
struct WrapperState {
    is_dragging: bool,
    id: Option<String>,
}

struct ScrollableWrapper<'a, Message, Theme, Renderer> {
    content: Element<'a, Message, Theme, Renderer>,
    id: Option<String>,
    scrollbar_width: f32,
    scrollbar_margin: f32,
    scroller_width: f32,
    vertical_alignment: Anchor,
    horizontal_alignment: Anchor,
    is_vertical: bool,
    is_horizontal: bool,
}

impl<'a, Message, Theme, Renderer> ScrollableWrapper<'a, Message, Theme, Renderer> {
    fn is_over_rail(&self, bounds: Rectangle, cursor_position: Point) -> bool {
        if self.is_vertical {
            let total_width =
                self.scrollbar_width.max(self.scroller_width) + 2.0 * self.scrollbar_margin;

            let rail_x = if self.vertical_alignment == Anchor::End {
                bounds.x + bounds.width - total_width
            } else {
                bounds.x
            };

            let rail_bounds = Rectangle {
                x: rail_x,
                y: bounds.y,
                width: total_width,
                height: bounds.height,
            };

            if rail_bounds.contains(cursor_position) {
                return true;
            }
        }

        if self.is_horizontal {
            let total_height =
                self.scrollbar_width.max(self.scroller_width) + 2.0 * self.scrollbar_margin;

            let rail_y = if self.horizontal_alignment == Anchor::End {
                bounds.y + bounds.height - total_height
            } else {
                bounds.y
            };

            let rail_bounds = Rectangle {
                x: bounds.x,
                y: rail_y,
                width: bounds.width,
                height: total_height,
            };

            if rail_bounds.contains(cursor_position) {
                return true;
            }
        }

        false
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for ScrollableWrapper<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<WrapperState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(WrapperState::default())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        let state = tree.state.downcast_mut::<WrapperState>();
        if state.id != self.id {
            state.id = self.id.clone();
            state.is_dragging = false;
            tree.children = vec![Tree::new(&self.content)];
        } else {
            tree.diff_children(std::slice::from_ref(&self.content));
        }
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn iced::advanced::widget::Operation,
    ) {
        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
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
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );

        if let Some(cursor_position) = cursor.position() {
            let bounds = layout.bounds();
            let is_over_rail = self.is_over_rail(bounds, cursor_position);
            let state = tree.state.downcast_mut::<WrapperState>();

            match event {
                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                    if is_over_rail {
                        state.is_dragging = true;
                    }
                }
                Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                    state.is_dragging = false;
                }
                _ => {}
            }
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let state = tree.state.downcast_ref::<WrapperState>();

        if state.is_dragging {
            return mouse::Interaction::Grabbing;
        }

        if let Some(cursor_position) = cursor.position() {
            let bounds = layout.bounds();
            if self.is_over_rail(bounds, cursor_position) {
                return mouse::Interaction::Pointer;
            }
        }

        self.content.as_widget().mouse_interaction(
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
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<iced::advanced::overlay::Element<'b, Message, Theme, Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

pub struct Scrollable<'a, Message, Theme, Renderer> {
    content: Element<'a, Message, Theme, Renderer>,
    id: Option<String>,
    scrollbar_color: Color,
    width: Length,
    height: Length,
    spacing: f32,
    direction: Direction,
    vertical_alignment: Anchor,
    horizontal_alignment: Anchor,
    embed: bool,
}

impl<'a, Message, Theme, Renderer> Scrollable<'a, Message, Theme, Renderer> {
    pub fn new(
        scrollbar_color: Color,
        content: impl Into<Element<'a, Message, Theme, Renderer>>,
    ) -> Self {
        Self {
            content: content.into(),
            id: None,
            scrollbar_color,
            width: Length::Shrink,
            height: Length::Shrink,
            spacing: 0.0,
            direction: Direction::Vertical(Scrollbar::new()),
            vertical_alignment: Anchor::End,
            horizontal_alignment: Anchor::End,
            embed: true,
        }
    }

    /// Resets the scroll position when the identifier changes.
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    pub fn spacing(mut self, spacing: impl Into<iced::Pixels>) -> Self {
        self.spacing = spacing.into().0;
        self
    }

    pub fn direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }

    pub fn vertical_anchor(mut self, alignment: Anchor) -> Self {
        self.vertical_alignment = alignment;
        self
    }

    pub fn horizontal_anchor(mut self, alignment: Anchor) -> Self {
        self.horizontal_alignment = alignment;
        self
    }

    pub fn embed(mut self, embed: bool) -> Self {
        self.embed = embed;
        self
    }
}

impl<'a, Message, Theme, Renderer> From<Scrollable<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a + Catalog,
    <Theme as Catalog>::Class<'a>: From<iced::widget::scrollable::StyleFn<'a, Theme>>,
    Renderer: 'a + iced::advanced::text::Renderer,
{
    fn from(builder: Scrollable<'a, Message, Theme, Renderer>) -> Self {
        let scrollbar_color = builder.scrollbar_color;

        let is_vertical = matches!(
            builder.direction,
            Direction::Vertical(_) | Direction::Both { .. }
        );
        let is_horizontal = matches!(
            builder.direction,
            Direction::Horizontal(_) | Direction::Both { .. }
        );

        let mut scrollable = iced::widget::scrollable(builder.content)
            .width(builder.width)
            .height(builder.height)
            .direction(builder.direction);

        if builder.embed {
            scrollable = scrollable.spacing(builder.spacing);
        }

        let inner = scrollable
            .style(move |_theme: &Theme, status| {
                let color = match status {
                    Status::Active { .. } => {
                        scrollbar::scroller_color(scrollbar_color, false, false, false)
                    }
                    Status::Hovered {
                        is_vertical_scrollbar_hovered,
                        ..
                    } => scrollbar::scroller_color(
                        scrollbar_color,
                        true,
                        is_vertical_scrollbar_hovered,
                        false,
                    ),
                    Status::Dragged { .. } => {
                        scrollbar::scroller_color(scrollbar_color, true, true, true)
                    }
                };

                Style {
                    container: container::Style::default(),
                    vertical_rail: Rail {
                        background: Some(Background::Color(Color::TRANSPARENT)),
                        border: Border::default(),
                        scroller: Scroller {
                            background: Background::Color(color),
                            border: scrollbar::scroller_border(),
                        },
                    },
                    horizontal_rail: Rail {
                        background: Some(Background::Color(Color::TRANSPARENT)),
                        border: Border::default(),
                        scroller: Scroller {
                            background: Background::Color(color),
                            border: scrollbar::scroller_border(),
                        },
                    },
                    gap: None,
                    auto_scroll: AutoScroll {
                        background: Background::Color(Color::TRANSPARENT),
                        border: Border::default(),
                        shadow: Shadow::default(),
                        icon: Color::TRANSPARENT,
                    },
                }
            })
            .into();

        Element::new(ScrollableWrapper {
            content: inner,
            id: builder.id,
            scrollbar_width: scrollbar::WIDTH,
            scrollbar_margin: scrollbar::MARGIN,
            scroller_width: scrollbar::WIDTH,
            vertical_alignment: builder.vertical_alignment,
            horizontal_alignment: builder.horizontal_alignment,
            is_vertical,
            is_horizontal,
        })
    }
}

#[must_use]
pub fn scrollable<'a, Message, Theme, Renderer>(
    theme: &impl ColorScheme,
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Scrollable<'a, Message, Theme, Renderer>
where
    Theme: 'a + Catalog,
    <Theme as Catalog>::Class<'a>: From<iced::widget::scrollable::StyleFn<'a, Theme>>,
    Renderer: 'a + iced::advanced::text::Renderer,
{
    scrollable_with_color(theme.on_surface_variant(), content)
}

pub fn scrollable_with_color<'a, Message, Theme, Renderer>(
    scrollbar_color: Color,
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Scrollable<'a, Message, Theme, Renderer>
where
    Theme: 'a + Catalog,
    <Theme as Catalog>::Class<'a>: From<iced::widget::scrollable::StyleFn<'a, Theme>>,
    Renderer: 'a + iced::advanced::text::Renderer,
{
    Scrollable::new(scrollbar_color, content).direction(Direction::Vertical(
        Scrollbar::new()
            .width(scrollbar::WIDTH)
            .scroller_width(scrollbar::WIDTH)
            .margin(scrollbar::MARGIN)
            .anchor(Anchor::Start),
    ))
}
