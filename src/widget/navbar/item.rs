use iced::time::Instant;

use iced::{
    Element, Font, Length, Pixels, Point, Size,
    advanced::{
        Widget,
        layout::Node,
        text::{self, Paragraph},
        widget::{Tree, tree},
    },
    border::Radius,
};

const LABEL_SIZE: f32 = 12.0;
const LABEL_LINE_HEIGHT: f32 = 16.0;
const ICON_LABEL_SPACE: f32 = 4.0;
const ICON_SIZE: f32 = 24.0;

const INDICATOR_SMALL_SIZE: Size = Size::new(56.0, 32.0);
const ITEM_SMALL_TOTAL_HEIGHT: f32 =
    INDICATOR_SMALL_SIZE.height + ICON_LABEL_SPACE + LABEL_LINE_HEIGHT;

const INDICATOR_LARGE_PADDING: f32 = 16.0;
const INDICATOR_LARGE_HEIGHT: f32 = 40.0;

pub use crate::widget::internal::navigation_common::NavigationItemContent as Content;
pub use crate::widget::internal::navigation_common::NavigationItemStyle as Style;
pub use crate::widget::internal::navigation_common::{Colors, State};
use crate::{
    animation::motion::{self, Spring, default_effects, fast_effects},
    widget::{
        hybrid_icon::Icon,
        internal::{
            navigation_common::{
                badge_children, build_badge, diff_badge, draw_icon, draw_indicator, draw_label,
                handle_press, paragraph, update_springs,
            },
            text::CachedParagraph,
        },
    },
};

#[derive(Default, Clone, Copy, PartialEq, PartialOrd)]
pub enum Mode {
    Large {
        width: Option<Pixels>,
    },
    #[default]
    Compact,
}

pub struct Item<'a, Message, Theme, Renderer>
where
    Message: Clone,
    Renderer: iced::advanced::text::Renderer
        + iced::advanced::svg::Renderer
        + iced_widget::renderer::geometry::Renderer,
    Renderer::Font: From<iced::Font>,
{
    mode: Mode,
    style: Style,
    content: Content<'a, Message>,
    label_font: Option<Font>,
    active: bool,
    active_transition_spring: Option<Spring>,
    color_spring: Option<Spring>,

    // Cache
    label_paragraph: CachedParagraph<Renderer::Font, Renderer::Paragraph>,
    icon_paragraph: CachedParagraph<Renderer::Font, Renderer::Paragraph>,
    badge: Option<Element<'a, Message, Theme, Renderer>>,
}

impl<'a, Message, Theme, Renderer> Item<'a, Message, Theme, Renderer>
where
    Message: 'a + Clone,
    Theme: 'a,
    Renderer: 'a
        + iced::advanced::text::Renderer
        + iced::advanced::svg::Renderer
        + iced_widget::renderer::geometry::Renderer,
    Renderer::Font: From<iced::Font>,
{
    #[must_use]
    pub fn new(
        style: Style,
        content: Content<'a, Message>,
        label_font: Option<Font>,
        active: bool,
    ) -> Self {
        let badge =
            build_badge::<Message, Theme, Renderer>(&style, content.badge.as_ref(), ICON_SIZE);

        Self {
            mode: Mode::default(),
            style,
            content,
            label_font,
            active,
            active_transition_spring: None,
            color_spring: None,
            label_paragraph: CachedParagraph::default(),
            icon_paragraph: CachedParagraph::default(),
            badge,
        }
    }

    #[must_use]
    pub fn motion_scheme_maybe(mut self, scheme: Option<motion::Scheme>) -> Self {
        if let Some(scheme) = scheme {
            self.color_spring = Some(fast_effects(scheme));
            self.active_transition_spring = Some(default_effects(scheme));
        }
        self
    }

    crate::macros::maybe_assign!(
        active_transition_spring_maybe,
        active_transition_spring,
        Option<Spring>
    );

    crate::macros::maybe_assign!(color_spring_maybe, color_spring, Option<Spring>);

    #[must_use]
    pub fn mode(mut self, mode: Mode) -> Self {
        self.mode = mode;
        self
    }

    /// Reuses an already-built item for a different mode, avoiding a rebuild
    /// (and a content clone) between measuring and laying out.
    pub(crate) fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Item<'a, Message, Theme, Renderer>
where
    Message: Clone,
    Renderer: iced::advanced::text::Renderer
        + iced::advanced::svg::Renderer
        + iced_widget::renderer::geometry::Renderer,
    Renderer::Font: From<iced::Font>,
{
    fn size(&self) -> Size<Length> {
        match self.mode {
            Mode::Large { width } => Size::new(
                width.map(|w| Length::Fixed(w.0)).unwrap_or(Length::Shrink),
                Length::Fixed(INDICATOR_LARGE_HEIGHT),
            ),
            Mode::Compact => Size::new(
                Length::Fixed(INDICATOR_SMALL_SIZE.width),
                Length::Fixed(INDICATOR_SMALL_SIZE.height),
            ),
        }
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        let content_color = match self.active {
            true => self.style.content_active,
            false => self.style.content_inactive,
        };
        let colors = Colors {
            icon: content_color,
            label: content_color,
        };
        let state = State::new(
            self.active,
            colors,
            self.active_transition_spring,
            self.color_spring,
            Instant::now(),
        );

        iced::advanced::widget::tree::State::new(state)
    }

    fn children(&self) -> Vec<Tree> {
        badge_children(&self.badge)
    }

    fn diff(&self, tree: &mut Tree) {
        diff_badge(tree, &self.badge);
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> iced::advanced::layout::Node {
        match self.content.icon(self.active) {
            Icon::Text { text, font } => {
                let icon_size = Pixels(ICON_SIZE);
                let line_height = text::LineHeight::Absolute(icon_size);

                self.icon_paragraph.update(
                    text,
                    Size::INFINITE,
                    icon_size,
                    line_height,
                    (*font).map(Renderer::Font::from),
                    iced::alignment::Vertical::Center,
                    || {
                        paragraph(
                            renderer,
                            text,
                            Size::INFINITE,
                            icon_size,
                            line_height,
                            *font,
                            iced::alignment::Vertical::Center,
                        )
                    },
                );
            }
            Icon::Svg(_) => self.icon_paragraph.clear(),
        }

        let icon_bounds = Size::new(ICON_SIZE, ICON_SIZE);
        let icon_node = Node::new(icon_bounds);

        let badge_node = self.badge.as_mut().map(|badge| {
            badge
                .as_widget_mut()
                .layout(&mut tree.children[0], renderer, limits)
        });

        let label = self.content.label.as_ref();
        let label_size = Pixels(LABEL_SIZE);
        let label_line_height = text::LineHeight::Absolute(Pixels(LABEL_LINE_HEIGHT));

        let label_bounds = self
            .label_paragraph
            .update(
                label,
                Size::INFINITE,
                label_size,
                label_line_height,
                self.label_font.map(Renderer::Font::from),
                iced::alignment::Vertical::Center,
                || {
                    paragraph(
                        renderer,
                        label,
                        Size::INFINITE,
                        label_size,
                        label_line_height,
                        self.label_font,
                        iced::alignment::Vertical::Center,
                    )
                },
            )
            .min_bounds();

        match self.mode {
            Mode::Large { width } => {
                let indicator_bounds = Size::new(
                    width.map(|w| w.0).unwrap_or(
                        label_bounds.width
                            + icon_bounds.width
                            + 2.0 * INDICATOR_LARGE_PADDING
                            + ICON_LABEL_SPACE,
                    ),
                    INDICATOR_LARGE_HEIGHT,
                );

                let content_width = icon_bounds.width + label_bounds.width + ICON_LABEL_SPACE;
                let offset = (indicator_bounds.width - content_width) / 2.0;
                let icon_position = Point::new(offset, (indicator_bounds.height - ICON_SIZE) / 2.0);

                let icon_node = icon_node.move_to(icon_position);

                let label_node = Node::new(label_bounds).move_to(Point::new(
                    offset + ICON_SIZE + ICON_LABEL_SPACE,
                    (indicator_bounds.height - label_bounds.height) / 2.0,
                ));

                let mut children = vec![icon_node, label_node];
                if let Some(node) = badge_node {
                    let position = Point::new(
                        icon_position.x,
                        icon_position.y + icon_bounds.height - node.bounds().height,
                    );
                    children.push(node.move_to(position))
                }

                Node::with_children(indicator_bounds, children)
            }
            Mode::Compact => {
                let indicator_bounds = INDICATOR_SMALL_SIZE;

                let parent_bounds = Size::new(indicator_bounds.width, ITEM_SMALL_TOTAL_HEIGHT);

                let indicator_node = Node::new(indicator_bounds);

                let icon_position = Point::new(
                    (indicator_bounds.width - icon_bounds.width) / 2.0,
                    (indicator_bounds.height - icon_bounds.height) / 2.0,
                );
                let icon_node = icon_node.move_to(icon_position);

                let label_node = Node::new(indicator_bounds).move_to(Point::new(
                    (indicator_bounds.width - label_bounds.width) / 2.0,
                    parent_bounds.height - label_bounds.height,
                ));

                let mut children = vec![indicator_node, icon_node, label_node];
                if let Some(node) = badge_node {
                    let position = Point::new(
                        icon_position.x,
                        icon_position.y + icon_bounds.height - node.bounds().height,
                    );
                    children.push(node.move_to(position))
                }

                Node::with_children(parent_bounds, children)
            }
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &iced::advanced::renderer::Style,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        viewport: &iced::Rectangle,
    ) {
        let (indicator_bounds, icon_bounds, label_bounds, badge_node) = match self.mode {
            Mode::Large { width: _ } => (
                layout.bounds(),
                layout.child(0).bounds(),
                layout.child(1).bounds(),
                self.badge.as_ref().map(|_| layout.child(2)),
            ),
            Mode::Compact => (
                layout.child(0).bounds(),
                layout.child(1).bounds(),
                layout.child(2).bounds(),
                self.badge.as_ref().map(|_| layout.child(3)),
            ),
        };

        let state = tree.state.downcast_ref::<State>();
        draw_indicator(
            renderer,
            indicator_bounds,
            state.active_transition_spring.value(),
            self.style.container_color,
        );

        let colors = state.color_spring.value();
        draw_icon(
            renderer,
            self.icon_paragraph.as_ref(),
            self.content.icon(self.active),
            icon_bounds,
            colors.icon,
        );

        if let Some(label) = self.label_paragraph.as_ref() {
            draw_label(renderer, label, label_bounds, colors.label);
        }

        if let Some(badge) = &self.badge {
            badge.as_widget().draw(
                &tree.children[0],
                renderer,
                theme,
                style,
                badge_node.unwrap(),
                cursor,
                viewport,
            );
        }

        state.ripple.draw(
            renderer,
            indicator_bounds,
            Radius::from(f32::MAX),
            self.style.state_layer,
            true,
        );
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &iced::Event,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn iced::advanced::Clipboard,
        shell: &mut iced::advanced::Shell<'_, Message>,
        _viewport: &iced::Rectangle,
    ) {
        let state = tree.state.downcast_mut::<State>();
        let indicator_bounds = match self.mode {
            Mode::Large { width: _ } => layout.bounds(),
            Mode::Compact => layout.child(0).bounds(),
        };

        handle_press(
            event,
            state,
            indicator_bounds,
            indicator_bounds,
            cursor,
            &self.content.on_press,
            shell,
        );
        update_springs(
            event,
            state,
            self.active,
            &self.style,
            self.active_transition_spring,
            self.color_spring,
            shell,
        );
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        _viewport: &iced::Rectangle,
        _renderer: &Renderer,
    ) -> iced::advanced::mouse::Interaction {
        let indicator_bounds = match self.mode {
            Mode::Large { width: _ } => layout.bounds(),
            Mode::Compact => layout.child(0).bounds(),
        };

        match cursor.is_over(indicator_bounds) {
            true => iced::advanced::mouse::Interaction::Pointer,
            false => iced::advanced::mouse::Interaction::None,
        }
    }
}
