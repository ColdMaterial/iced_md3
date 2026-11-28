pub const INDICATOR_SMALL_HEIGHT: f32 = 32.0;
pub const INDICATOR_SMALL_WIDTH: f32 = 56.0;
pub const ITEM_SMALL_ICON_LABEL_SPACE: f32 = 4.0;
pub const ITEM_SMALL_LABEL_SIZE: f32 = 12.0;
pub const ITEM_SMALL_LABEL_LINE_HEIGHT: f32 = 16.0;
/// Total height of a collapsed item, matching the M3 spec (`container_height`
/// of 64dp: 6 padding + 32 indicator + 4 space + 16 label + 6 padding).
pub const ITEM_SMALL_TOTAL_HEIGHT: f32 = 64.0;
const ITEM_SMALL_VERTICAL_PADDING: f32 = (ITEM_SMALL_TOTAL_HEIGHT
    - INDICATOR_SMALL_HEIGHT
    - ITEM_SMALL_ICON_LABEL_SPACE
    - ITEM_SMALL_LABEL_LINE_HEIGHT)
    / 2.0;
/// Height of a collapsed item when labels are hidden (icon-only).
pub const ITEM_SMALL_NO_LABEL_HEIGHT: f32 =
    INDICATOR_SMALL_HEIGHT + 2.0 * ITEM_SMALL_VERTICAL_PADDING + ITEM_SMALL_ICON_LABEL_SPACE;

pub const INDICATOR_LARGE_HEIGHT: f32 = 56.0;
pub const INDICATOR_LARGE_PADDING: f32 = 16.0;
pub const ITEM_LARGE_ICON_LABEL_SPACE: f32 = 8.0;
pub const ITEM_LARGE_LABEL_SIZE: f32 = 14.0;
pub const ITEM_LARGE_LABEL_LINE_HEIGHT: f32 = 20.0;

const ITEM_LARGE_LABEL_START_X: f32 = 28.0;
const ITEM_LABEL_TRAVEL_PORTION: f32 = 0.75;
const ITEM_LARGE_LABEL_FIT_SLACK: f32 = 1.0;

pub const ICON_SIZE: f32 = 24.0;

/// Horizontal breathing room kept on each side of a collapsed item's label.
const ITEM_SMALL_LABEL_MARGIN: f32 = 8.0;
/// Width of an expanded item that is not the label itself: the icon, the space between the icon
/// and the label, and the padding on both sides of the pill. Subtracting it from the available
/// rail width yields the widest label that still fits.
const ITEM_LARGE_LABEL_CHROME: f32 =
    ICON_SIZE + 4.0 * INDICATOR_LARGE_PADDING + ITEM_LARGE_ICON_LABEL_SPACE;

use iced::time::Instant;

use iced::{
    Element, Font, Length, Pixels, Point, Rectangle, Size, Vector,
    advanced::{
        Widget,
        layout::Node,
        text::{self, Paragraph},
        widget::Tree,
    },
    border::Radius,
};

use crate::{
    animation::{
        Interpolable, lerp,
        motion::{self, Spring, default_effects, fast_effects},
    },
    widget::{
        internal::navigation_common::{
            badge_children, build_badge, colors_for, diff_badge, draw_icon, draw_indicator,
            draw_label, handle_press, icon_paragraph, update_springs,
        },
        tooltip,
    },
};

pub use crate::widget::internal::navigation_common::NavigationItemContent as Content;
pub use crate::widget::internal::navigation_common::NavigationItemStyle as Style;
pub use crate::widget::internal::navigation_common::State as CommonState;

pub(crate) struct State {
    pub common: CommonState,
    pub expansion: f32,
    /// Hover state of the tooltip that spells out a label that does not fit.
    tooltip: tooltip::Hover,
}

struct LabelBlend {
    travel: f32,
    small: f32,
    large: f32,
}

fn label_blend(expansion: f32, hide_when_collapsed: bool) -> LabelBlend {
    let expansion = expansion.clamp(0.0, 1.0);

    if hide_when_collapsed {
        return LabelBlend {
            travel: 1.0,
            small: 0.0,
            large: expansion,
        };
    }

    let swap = ((expansion - ITEM_LABEL_TRAVEL_PORTION) / (1.0 - ITEM_LABEL_TRAVEL_PORTION))
        .clamp(0.0, 1.0);

    LabelBlend {
        travel: (expansion / ITEM_LABEL_TRAVEL_PORTION).min(1.0),
        small: 1.0 - swap,
        large: swap,
    }
}

pub struct Item<'a, Message, Theme, Renderer>
where
    Message: Clone,
    Renderer: iced::advanced::text::Renderer
        + iced::advanced::svg::Renderer
        + iced_widget::renderer::geometry::Renderer,
    Renderer::Font: From<iced::Font>,
{
    expanded: bool,
    style: Style,
    content: Content<'a, Message>,
    label_font: Option<Font>,
    active: bool,
    active_transition_spring: Option<Spring>,
    color_spring: Option<Spring>,
    hide_labels_when_collapsed: bool,
    /// Width available to the labels, or `None` when the rail grows with its content.
    max_label_width: Option<f32>,
    tooltip_style: tooltip::Style,

    // Cache
    small_label_paragraph: Option<Renderer::Paragraph>,
    large_label_paragraph: Option<Renderer::Paragraph>,
    small_label_text: Option<String>,
    large_label_text: Option<String>,
    small_label_fit_width: Option<f32>,
    large_label_fit_width: Option<f32>,
    small_label_truncated: bool,
    large_label_truncated: bool,
    icon_paragraph: Option<Renderer::Paragraph>,
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
        expanded: bool,
        label_font: Option<Font>,
        active: bool,
        hide_labels_when_collapsed: bool,
        max_label_width: Option<f32>,
        tooltip_style: tooltip::Style,
    ) -> Self {
        let badge =
            build_badge::<Message, Theme, Renderer>(&style, content.badge.as_ref(), ICON_SIZE);

        Self {
            expanded,
            style,
            content,
            label_font,
            active,
            active_transition_spring: None,
            color_spring: None,
            hide_labels_when_collapsed,
            max_label_width,
            tooltip_style,
            small_label_paragraph: None,
            large_label_paragraph: None,
            small_label_text: None,
            large_label_text: None,
            small_label_fit_width: None,
            large_label_fit_width: None,
            small_label_truncated: false,
            large_label_truncated: false,
            icon_paragraph: None,
            badge,
        }
    }

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
        let height = if self.expanded {
            INDICATOR_LARGE_HEIGHT
        } else if self.hide_labels_when_collapsed {
            ITEM_SMALL_NO_LABEL_HEIGHT
        } else {
            ITEM_SMALL_TOTAL_HEIGHT
        };
        Size {
            width: Length::Shrink,
            height: Length::Fixed(height),
        }
    }

    fn tag(&self) -> iced::advanced::widget::tree::Tag {
        iced::advanced::widget::tree::Tag::of::<State>()
    }

    fn state(&self) -> iced::advanced::widget::tree::State {
        let colors = colors_for(self.active, &self.style);
        let common = CommonState::new(
            self.active,
            colors,
            self.active_transition_spring,
            self.color_spring,
            Instant::now(),
        );
        let state = State {
            common,
            expansion: self.expanded as usize as f32,
            tooltip: tooltip::Hover::default(),
        };

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
        let expansion = tree.state.downcast_ref::<State>().expansion.clamp(0.0, 1.0);

        if self.icon_paragraph.is_none() {
            self.icon_paragraph =
                icon_paragraph(renderer, self.content.icon(self.active), ICON_SIZE);
        }

        // A label that does not fit the rail is ellipsized; hovering the item then shows the full
        // text as a tooltip. Only the label that is actually visible for the current mode is
        // shortened, so the one that is fading out during the expansion animation keeps its size.
        let small_max_width = match (self.expanded, self.max_label_width) {
            (false, Some(width)) => (width - 2.0 * ITEM_SMALL_LABEL_MARGIN).max(0.0),
            _ => f32::INFINITY,
        };
        let large_max_width = match (self.expanded, self.max_label_width) {
            (true, Some(width)) => (width - ITEM_LARGE_LABEL_CHROME).max(0.0),
            _ => f32::INFINITY,
        };

        if self.small_label_fit_width != Some(small_max_width) {
            let (small_text, small_truncated, small_paragraph) =
                crate::widget::internal::text::fit_label(
                    renderer,
                    &self.content.label,
                    small_max_width,
                    Pixels(ITEM_SMALL_LABEL_SIZE),
                    text::LineHeight::Absolute(Pixels(ITEM_SMALL_LABEL_LINE_HEIGHT)),
                    self.label_font.map(Renderer::Font::from),
                    iced::alignment::Vertical::Center,
                );

            self.small_label_truncated = small_truncated && !self.hide_labels_when_collapsed;
            self.small_label_fit_width = Some(small_max_width);

            self.small_label_paragraph = Some(small_paragraph);
            self.small_label_text = Some(small_text);
        }

        if self.large_label_fit_width != Some(large_max_width) {
            let (large_text, large_truncated, large_paragraph) =
                crate::widget::internal::text::fit_label(
                    renderer,
                    &self.content.label,
                    large_max_width,
                    Pixels(ITEM_LARGE_LABEL_SIZE),
                    text::LineHeight::Absolute(Pixels(ITEM_LARGE_LABEL_LINE_HEIGHT)),
                    self.label_font.map(Renderer::Font::from),
                    iced::alignment::Vertical::Center,
                );

            self.large_label_truncated = large_truncated;
            self.large_label_fit_width = Some(large_max_width);

            self.large_label_paragraph = Some(large_paragraph);
            self.large_label_text = Some(large_text);
        }

        let small_bounds = self
            .small_label_paragraph
            .as_ref()
            .map(|paragraph| paragraph.min_bounds())
            .unwrap_or(Size::ZERO);
        let large_bounds = self
            .large_label_paragraph
            .as_ref()
            .map(|paragraph| paragraph.min_bounds())
            .unwrap_or(Size::ZERO);

        let hug_width = large_bounds.width
            + ICON_SIZE
            + 2.0 * INDICATOR_LARGE_PADDING
            + ITEM_LARGE_ICON_LABEL_SPACE;

        let node_width = if self.expanded {
            hug_width + 2.0 * INDICATOR_LARGE_PADDING
        } else {
            let max = limits.max().width;
            match max.is_finite() {
                true => max,
                false => hug_width + 2.0 * super::constants::ITEM_OFFSET,
            }
        };

        let small_height = if self.hide_labels_when_collapsed {
            ITEM_SMALL_NO_LABEL_HEIGHT
        } else {
            ITEM_SMALL_TOTAL_HEIGHT
        };
        let small_pill_y = if self.hide_labels_when_collapsed {
            (small_height - INDICATOR_SMALL_HEIGHT) / 2.0
        } else {
            ITEM_SMALL_VERTICAL_PADDING
        };

        let pill_height = lerp(INDICATOR_SMALL_HEIGHT, INDICATOR_LARGE_HEIGHT, expansion).round();
        let pill_width = lerp(INDICATOR_SMALL_WIDTH, hug_width, expansion).round();

        let pill_x = lerp(
            super::constants::ITEM_OFFSET,
            INDICATOR_LARGE_PADDING,
            expansion,
        )
        .round();
        let pill_y = lerp(small_pill_y, 0.0, expansion).round();
        let node_height = lerp(small_height, INDICATOR_LARGE_HEIGHT, expansion).round();

        let indicator_node =
            Node::new(Size::new(pill_width, pill_height)).move_to(Point::new(pill_x, pill_y));

        // Leading padding of 16dp keeps the icon centered in the collapsed
        // pill (56/2 - 24/2) and leading-aligned in the expanded pill.
        let icon_x = pill_x + INDICATOR_LARGE_PADDING;
        let icon_y = (pill_y + (pill_height - ICON_SIZE) / 2.0).round();
        let icon_node =
            Node::new(Size::new(ICON_SIZE, ICON_SIZE)).move_to(Point::new(icon_x, icon_y));

        let small_label_y = pill_y + pill_height + ITEM_SMALL_ICON_LABEL_SPACE;
        let small_label_node = Node::new(small_bounds).move_to(Point::new(
            (pill_x + (INDICATOR_SMALL_WIDTH - small_bounds.width) / 2.0).round(),
            small_label_y,
        ));

        let large_label_node = Node::new(large_bounds).move_to(Point::new(
            (pill_x
                + lerp(
                    ITEM_LARGE_LABEL_START_X,
                    INDICATOR_LARGE_PADDING + ICON_SIZE + ITEM_LARGE_ICON_LABEL_SPACE,
                    expansion,
                ))
            .round(),
            (pill_y + (pill_height - large_bounds.height) / 2.0).round(),
        ));

        let has_label = self
            .content
            .badge
            .as_ref()
            .is_some_and(|badge| badge.label.is_some());

        let mut expanded_badge_right = 0.0f32;

        let badge_node = self.badge.as_mut().map(|badge| {
            let node = badge
                .as_widget_mut()
                .layout(&mut tree.children[0], renderer, limits);

            let large_label_end = large_label_node.bounds().x + large_bounds.width;
            let icon_center_y = icon_y + ICON_SIZE / 2.0;
            let label_center_y = large_label_node.bounds().y + large_bounds.height / 2.0;

            let (collapsed_x, expanded_x) = if has_label {
                (icon_x, large_label_end - 8.0)
            } else {
                (icon_x, large_label_end - 14.0)
            };
            let (collapsed_y, expanded_y) = if has_label {
                (icon_center_y - 12.0, label_center_y - 8.0)
            } else {
                (icon_center_y - ICON_SIZE / 2.0, label_center_y - 3.0)
            };

            let badge_x = lerp(collapsed_x, expanded_x, expansion).round();
            let badge_y = lerp(collapsed_y, expanded_y, expansion).round();

            expanded_badge_right = expanded_x + node.bounds().width;

            // Collapsed items are as wide as the rail itself, so keep the badge
            // inside that width instead of letting it spill over the edge.
            let badge_x = if self.expanded {
                badge_x
            } else {
                badge_x
                    .min((node_width - node.bounds().width).max(0.0))
                    .round()
            };

            node.move_to(Point::new(badge_x, badge_y))
        });

        let mut children = vec![
            indicator_node,
            icon_node,
            small_label_node,
            large_label_node,
        ];

        let mut node_size = Size::new(node_width, node_height);
        if let Some(node) = badge_node {
            if self.expanded && expanded_badge_right > node_size.width {
                node_size.width = expanded_badge_right + INDICATOR_LARGE_PADDING;
            }

            children.push(node);
        }

        Node::with_children(node_size, children)
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
        let state = tree.state.downcast_ref::<State>();
        let expansion = state.expansion;

        let indicator_bounds = layout.child(0).bounds();
        let icon_bounds = layout.child(1).bounds();
        let small_label_bounds = layout.child(2).bounds();
        let large_label_bounds = layout.child(3).bounds();
        let badge_layout = self.badge.as_ref().map(|_| layout.child(4));

        // Active indicator pill.
        draw_indicator(
            renderer,
            indicator_bounds,
            state.common.active_transition_spring.value(),
            self.style.container_color,
        );

        let colors = state.common.color_spring.value();

        let label_limit = indicator_bounds.x + indicator_bounds.width - INDICATOR_LARGE_PADDING;
        let room_clip_width =
            (label_limit - large_label_bounds.x).max(0.0) + ITEM_LARGE_LABEL_FIT_SLACK;
        let blend = label_blend(expansion, self.hide_labels_when_collapsed);

        if blend.small > 0.0
            && let Some(label) = &self.small_label_paragraph
        {
            let mut bounds = Rectangle::new(
                small_label_bounds
                    .position()
                    .interpolate(large_label_bounds.position(), blend.travel),
                small_label_bounds.size(),
            );

            if blend.travel >= 1.0 {
                bounds.width = bounds.width.min(room_clip_width);
            }

            draw_label(
                renderer,
                label,
                bounds,
                colors.label.scale_alpha(blend.small),
            );
        }

        draw_icon(
            renderer,
            self.icon_paragraph.as_ref(),
            self.content.icon(self.active),
            icon_bounds,
            colors.icon,
        );

        if blend.large > 0.0
            && let Some(label) = &self.large_label_paragraph
        {
            let clip = Rectangle::new(
                large_label_bounds.position(),
                Size::new(
                    large_label_bounds.width.min(room_clip_width),
                    large_label_bounds.height,
                ),
            );

            draw_label(renderer, label, clip, colors.label.scale_alpha(blend.large));
        }

        if let Some(badge) = &self.badge {
            badge.as_widget().draw(
                &tree.children[0],
                renderer,
                theme,
                style,
                badge_layout.unwrap(),
                cursor,
                viewport,
            );
        }

        state.common.ripple.draw(
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

        let bounds = layout.bounds();
        let indicator_bounds = layout.child(0).bounds();

        handle_press(
            event,
            &mut state.common,
            indicator_bounds,
            bounds,
            cursor,
            &self.content.on_press,
            shell,
        );
        update_springs(
            event,
            &mut state.common,
            self.active,
            &self.style,
            self.active_transition_spring,
            self.color_spring,
            shell,
        );

        let truncated = if self.expanded {
            self.large_label_truncated
        } else {
            self.small_label_truncated
        };

        state.tooltip.update(
            event,
            cursor,
            Some(bounds),
            truncated,
            tooltip::DEFAULT_DELAY,
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
        match cursor.is_over(layout.bounds()) {
            true => iced::advanced::mouse::Interaction::Pointer,
            false => iced::advanced::mouse::Interaction::None,
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: iced::advanced::Layout<'b>,
        _renderer: &Renderer,
        _viewport: &Rectangle,
        translation: Vector,
    ) -> Option<iced::advanced::overlay::Element<'b, Message, Theme, Renderer>> {
        let state = tree.state.downcast_ref::<State>();
        let truncated = if self.expanded {
            self.large_label_truncated
        } else {
            self.small_label_truncated
        };

        if !truncated || !state.tooltip.is_open() {
            return None;
        }

        // The label nodes are pushed in a fixed order at the end of `layout`.
        let label_bounds = layout.child(if self.expanded { 3 } else { 2 }).bounds();
        let anchor = Rectangle::new(label_bounds.position() + translation, label_bounds.size());

        Some(tooltip::label_overlay(
            anchor,
            text::Fragment::Borrowed(self.content.label.as_ref()),
            self.label_font.map(Renderer::Font::from),
            self.tooltip_style,
            tooltip::Position::Right,
            tooltip::DEFAULT_GAP,
            tooltip::DEFAULT_MAX_WIDTH,
            None,
        ))
    }
}

crate::macros::impl_element_from!(
    ['a, Message, Theme, Renderer]
    Item<'a, Message, Theme, Renderer> => Element<'a, Message, Theme, Renderer>
    where
        Message: 'a + Clone,
        Theme: 'a,
        Renderer: 'a
            + iced::advanced::text::Renderer
            + iced::advanced::svg::Renderer
            + iced_widget::renderer::geometry::Renderer,
        Renderer::Font: From<iced::Font>,
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collapsed_shows_only_the_short_label() {
        let blend = label_blend(0.0, false);

        assert_eq!(blend.travel, 0.0);
        assert_eq!(blend.small, 1.0);
        assert_eq!(blend.large, 0.0);
    }

    #[test]
    fn expanded_shows_only_the_long_label() {
        let blend = label_blend(1.0, false);

        assert_eq!(blend.travel, 1.0);
        assert_eq!(blend.small, 0.0);
        assert_eq!(blend.large, 1.0);
    }

    #[test]
    fn label_arrives_before_its_text_changes() {
        let blend = label_blend(ITEM_LABEL_TRAVEL_PORTION, false);

        assert_eq!(blend.travel, 1.0);
        assert_eq!(blend.small, 1.0);
        assert_eq!(blend.large, 0.0);
    }

    #[test]
    fn text_never_changes_while_the_label_is_moving() {
        for step in 0..=(ITEM_LABEL_TRAVEL_PORTION * 100.0).min(100.0) as u32 {
            let blend = label_blend(step as f32 / 100.0, false);

            assert_eq!(blend.small, 1.0);
            assert_eq!(blend.large, 0.0);
        }
    }

    #[test]
    fn text_swaps_after_the_label_has_arrived() {
        let mid = label_blend((ITEM_LABEL_TRAVEL_PORTION + 1.0) / 2.0, false);

        assert_eq!(mid.travel, 1.0);
        assert!(mid.small > 0.0 && mid.large > 0.0);
    }

    #[test]
    fn labels_never_leave_a_frame_without_one() {
        for step in 0..=100 {
            let blend = label_blend(step as f32 / 100.0, false);

            assert!((blend.small + blend.large - 1.0).abs() < f32::EPSILON);
            assert!((0.0..=1.0).contains(&blend.small));
            assert!((0.0..=1.0).contains(&blend.large));
            assert!((0.0..=1.0).contains(&blend.travel));
        }
    }

    #[test]
    fn label_travels_monotonically() {
        let mut previous = 0.0;

        for step in 0..=100 {
            let travel = label_blend(step as f32 / 100.0, false).travel;

            assert!(travel >= previous);
            previous = travel;
        }
    }

    #[test]
    fn hidden_labels_never_show_the_short_one() {
        for step in 0..=100 {
            assert_eq!(label_blend(step as f32 / 100.0, true).small, 0.0);
        }

        assert_eq!(label_blend(0.0, true).large, 0.0);
        assert_eq!(label_blend(1.0, true).large, 1.0);
    }
}
