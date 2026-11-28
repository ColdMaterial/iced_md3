use iced::time::Instant;

use iced::{
    Border, Color, Element, Event, Font, Length, Point, Radians, Rectangle, Size, Transformation,
    Vector,
    advanced::{
        Widget,
        layout::{Limits, Node},
        mouse, overlay,
        renderer::Quad,
        svg::{self},
        text,
        widget::{Tree, tree},
    },
    border::Radius,
    touch, window,
};

use crate::{
    animation::{
        Interpolable,
        motion::{self, Spring, SpringValue, fast_effects, fast_spatial},
    },
    style::StateLayer,
    theme::{Accent, ColorScheme},
    widget::{
        self, Action,
        button::{self, CornerStyle},
        common_icons,
        fab::{self},
        hybrid_icon::Icon,
        internal::{
            interaction::{self, PressUpdate},
            motion::sync_style_spring,
            placement,
        },
        ripple::Ripple,
    },
};

const BUTTON_SIZE: fab::Size = fab::Size::Regular;
const BUTTON_TRIGGER_BETWEEN_SPACE: f32 = 8.0;
const BUTTON_SPACING: f32 = 4.0;
const ITEM_START_SCALE: f32 = 0.4;
const ITEM_STAGGER: f32 = 0.12;
const ITEMS_READY_PROGRESS: f32 = 0.98;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StateStyle {
    pub container_color: Color,
    pub icon_color: Color,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    pub expanded: StateStyle,
    pub collapsed: StateStyle,
    pub state_layer_expanded: StateLayer,
    pub state_layer_collapsed: StateLayer,
    pub item_style: button::Style,
    pub accent: Accent,
}

impl Style {
    pub fn new(theme: &(impl ColorScheme + ?Sized), accent: Accent) -> Self {
        Self {
            expanded: StateStyle {
                container_color: accent.color(theme),
                icon_color: accent.on_color(theme),
            },
            collapsed: StateStyle {
                container_color: accent.color_container(theme),
                icon_color: accent.on_color_container(theme),
            },
            state_layer_expanded: StateLayer::new(accent.on_color(theme)),
            state_layer_collapsed: StateLayer::new(accent.on_color_container(theme)),
            item_style: button::Style::fab_tonal(theme, accent),
            accent,
        }
    }
}

impl Interpolable for StateStyle {
    fn interpolate(self, other: Self, t: f32) -> Self {
        Self {
            container_color: self.container_color.interpolate(other.container_color, t),
            icon_color: self.icon_color.interpolate(other.icon_color, t),
        }
    }
}

pub struct Item<'a, Message>
where
    Message: Clone,
{
    pub on_press: Action<'a, Message>,
    pub label: text::Fragment<'a>,
    pub icon: Icon<'a>,
}

pub struct FABMenu<'a, Message, Theme, Renderer>
where
    Message: Clone,
{
    style: Style,
    size: Option<fab::Size>,
    spatial_spring: Option<Spring>,
    effects_spring: Option<Spring>,
    items: Vec<Element<'a, Message, Theme, Renderer>>,

    // Cache
    style_checked: bool,
}

impl<'a, Message, Theme, Renderer> FABMenu<'a, Message, Theme, Renderer>
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
    pub fn new<I>(style: Style, items: I, label_font: Option<Font>) -> Self
    where
        I: IntoIterator<Item = Item<'a, Message>>,
    {
        let items = items
            .into_iter()
            .map(|i| {
                let button = widget::button(
                    style.item_style,
                    button::Content::Full {
                        icon: i.icon,
                        label: i.label,
                    },
                )
                .label_font_maybe(label_font)
                .size(button::Size {
                    width: Length::Shrink,
                    height: Length::Fixed(BUTTON_SIZE.container_height()),
                    spacing: iced::Pixels(BUTTON_SIZE.content_spacing()),
                    padding: iced::Padding::from([0.0, BUTTON_SIZE.padding()]),
                    icon_size: BUTTON_SIZE.icon_size(),
                    label_size: BUTTON_SIZE.label_size(),
                    corner_radius: button::CornerRadius {
                        style: CornerStyle::Rounded,
                        shape_morph: false,
                        rounded: f32::MAX.into(),
                        square: BUTTON_SIZE.rounding(),
                        pressed: 0.0.into(),
                    },
                });

                match i.on_press {
                    Action::Direct(message) => button.on_press(message).into(),
                    Action::Closure(callback) => button.on_press_with(callback).into(),
                }
            })
            .collect();

        Self {
            size: None,
            items,
            style,
            spatial_spring: None,
            effects_spring: None,

            style_checked: false,
        }
    }

    #[must_use]
    pub fn size(mut self, size: fab::Size) -> Self {
        self.size = Some(size);
        self
    }

    crate::macros::maybe_assign!(size_maybe, size, Option<fab::Size>);

    #[must_use]
    pub fn motion_scheme(mut self, scheme: motion::Scheme) -> Self {
        self.effects_spring = Some(fast_effects(scheme));
        self.spatial_spring = Some(fast_spatial(scheme));
        self
    }

    #[must_use]
    pub fn spatial_spring(mut self, spring: Spring) -> Self {
        self.spatial_spring = Some(spring);
        self
    }

    crate::macros::maybe_assign!(spatial_spring_maybe, spatial_spring, Option<Spring>);

    #[must_use]
    pub fn effects_spring(mut self, spring: Spring) -> Self {
        self.effects_spring = Some(spring);
        self
    }

    crate::macros::maybe_assign!(effects_spring_maybe, effects_spring, Option<Spring>);
}

struct State {
    is_expanded: bool,
    is_pressed: bool,
    style_spring: SpringValue<StateStyle>,
    icon_rotation_spring: SpringValue<Radians>,
    corner_radius_spring: SpringValue<Radius>,
    expansion_spring: SpringValue<f32>,
    presented_progress: f32,
    items_ready: bool,
    ripple: Ripple,
    last_style: Style,
}

impl State {
    fn is_open(&self) -> bool {
        self.is_expanded && self.items_ready
    }
}

fn item_progress(progress: f32, index: usize, item_count: usize) -> f32 {
    let stagger = ITEM_STAGGER * index as f32;
    let span = 1.0 + ITEM_STAGGER * item_count.saturating_sub(1) as f32;

    (progress * span - stagger).clamp(0.0, 1.0)
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for FABMenu<'a, Message, Theme, Renderer>
where
    Message: Clone,
    Renderer: 'a
        + iced::advanced::text::Renderer
        + iced::advanced::svg::Renderer
        + iced_widget::renderer::geometry::Renderer,
    Renderer::Font: From<iced::Font>,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        let now = Instant::now();
        let spatial_spring = self
            .spatial_spring
            .unwrap_or(fast_spatial(motion::Scheme::default()));
        let effects_spring = self
            .effects_spring
            .unwrap_or(fast_effects(motion::Scheme::default()));
        let state = State {
            is_expanded: false,
            is_pressed: false,
            style_spring: SpringValue::new(self.style.collapsed, effects_spring, now),
            icon_rotation_spring: SpringValue::new(Radians(0.0), spatial_spring, now),
            corner_radius_spring: SpringValue::new(
                self.size.unwrap_or_default().rounding(),
                spatial_spring,
                now,
            ),
            expansion_spring: SpringValue::new(0.0, spatial_spring, now),
            presented_progress: 0.0,
            items_ready: false,
            ripple: Ripple::new(),
            last_style: self.style,
        };

        tree::State::new(state)
    }

    fn size(&self) -> Size<Length> {
        let size = self.size.unwrap_or_default().container_height();

        Size {
            width: Length::Fixed(size),
            height: Length::Fixed(size),
        }
    }

    fn children(&self) -> Vec<Tree> {
        self.items.iter().map(Tree::new).collect()
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&self.items);
    }

    fn layout(
        &mut self,
        _tree: &mut iced::advanced::widget::Tree,
        _renderer: &Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> iced::advanced::layout::Node {
        let size = self.size.unwrap_or_default();
        let container_size = size.container_height();
        let intrinsic_size = Size::new(container_size, container_size);
        let container_bounds =
            limits.resolve(intrinsic_size.width, intrinsic_size.height, intrinsic_size);

        let icon_size = size.icon_size();
        let icon_bounds = Size::new(icon_size.0, icon_size.0);
        let icon_node =
            Node::new(icon_bounds).move_to(placement::centered_in(container_bounds, icon_bounds));

        Node::with_children(container_bounds, vec![icon_node])
    }

    fn draw(
        &self,
        tree: &iced::advanced::widget::Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &iced::advanced::renderer::Style,
        layout: iced::advanced::Layout<'_>,
        _cursor: iced::advanced::mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State>();
        let style = state.style_spring.value();
        let corner_radius = state.corner_radius_spring.value();
        let container_bounds = layout.bounds();

        renderer.fill_quad(
            Quad {
                bounds: container_bounds,
                border: Border::default().rounded(corner_radius),
                ..Default::default()
            },
            style.container_color,
        );

        let bounds = layout.child(0).bounds();
        renderer.draw_svg(
            svg::Svg::new(common_icons::ADD.clone())
                .rotation(state.icon_rotation_spring.value())
                .color(style.icon_color),
            bounds,
            container_bounds,
        );

        let state_layer = match state.is_expanded {
            true => self.style.state_layer_expanded,
            false => self.style.state_layer_collapsed,
        };
        state
            .ripple
            .draw(renderer, container_bounds, corner_radius, state_layer, true);
    }

    fn update(
        &mut self,
        tree: &mut iced::advanced::widget::Tree,
        event: &iced::Event,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn iced::advanced::Clipboard,
        shell: &mut iced::advanced::Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<State>();

        let is_hovered = cursor.is_over(layout.bounds());
        state
            .ripple
            .handle(event, layout.bounds(), cursor, true, shell);

        match interaction::update_press(event, &mut state.is_pressed, true, is_hovered) {
            PressUpdate::Pressed => shell.capture_event(),
            PressUpdate::Released => state.is_expanded = !state.is_expanded,
            PressUpdate::ReleasedOutside | PressUpdate::Lost | PressUpdate::Ignored => {}
        }

        state
            .icon_rotation_spring
            .set_model_if_some(self.spatial_spring);
        state
            .corner_radius_spring
            .set_model_if_some(self.spatial_spring);
        state
            .expansion_spring
            .set_model_if_some(self.spatial_spring);
        state.style_spring.set_model_if_some(self.effects_spring);

        let now = Instant::now();
        let (style, corner_radius, icon_rotation) = match state.is_expanded {
            true => {
                let bounds = layout.bounds();
                let radius = bounds.width.min(bounds.height) / 2.0;
                (self.style.expanded, radius.into(), Radians::PI / 4.0)
            }
            false => (
                self.style.collapsed,
                self.size.unwrap_or_default().rounding(),
                Radians(0.0),
            ),
        };

        sync_style_spring(
            &mut state.style_spring,
            &mut self.style_checked,
            &mut state.last_style,
            self.style,
            style,
            now,
        );
        state.corner_radius_spring.set_target(corner_radius, now);
        state.icon_rotation_spring.set_target(icon_rotation, now);
        state
            .expansion_spring
            .set_target(f32::from(state.is_expanded), now);

        if !state.style_spring.is_at_rest()
            || !state.corner_radius_spring.is_at_rest()
            || !state.icon_rotation_spring.is_at_rest()
            || !state.expansion_spring.is_at_rest()
        {
            shell.request_redraw();
        }

        state.style_spring.step(now);
        state.corner_radius_spring.step(now);
        state.icon_rotation_spring.step(now);
        state.expansion_spring.step(now);
        state.presented_progress = match state.is_expanded {
            true => state.expansion_spring.value().max(0.0),
            false => state.expansion_spring.value().min(state.presented_progress),
        };
        state.items_ready = match state.is_expanded {
            true => state.items_ready || state.presented_progress >= ITEMS_READY_PROGRESS,
            false => false,
        };
    }

    fn mouse_interaction(
        &self,
        _tree: &iced::advanced::widget::Tree,
        layout: iced::advanced::Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        if cursor.is_over(layout.bounds()) {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut iced::advanced::widget::Tree,
        layout: iced::advanced::Layout<'b>,
        _renderer: &Renderer,
        _viewport: &Rectangle,
        translation: Vector,
    ) -> Option<iced::advanced::overlay::Element<'b, Message, Theme, Renderer>> {
        let (is_expanded, is_animating) = {
            let state = tree.state.downcast_ref::<State>();
            (state.is_expanded, state.presented_progress > 0.0)
        };

        let mut base_bounds = layout.bounds();
        base_bounds.x += translation.x;
        base_bounds.y += translation.y;

        let mut children: Vec<iced::advanced::overlay::Element<'b, Message, Theme, Renderer>> =
            Vec::new();

        if is_expanded || is_animating {
            children.push(overlay::Element::new(Box::new(Overlay {
                base_bounds,
                tree,
                items: &mut self.items,
            })));
        }

        if children.is_empty() {
            None
        } else {
            Some(iced::advanced::overlay::Group::with_children(children).overlay())
        }
    }
}

crate::macros::impl_element_from!(
    ['a, Message, Theme, Renderer]
    FABMenu<'a, Message, Theme, Renderer> => Element<'a, Message, Theme, Renderer>
    where
        Message: 'a + Clone,
        Theme: 'a,
        Renderer: 'a
            + iced::advanced::text::Renderer
            + iced::advanced::svg::Renderer
            + iced_widget::renderer::geometry::Renderer,
        Renderer::Font: From<iced::Font>,
);

struct Overlay<'a, 'b, Message, Theme, Renderer> {
    base_bounds: Rectangle,
    tree: &'b mut Tree,
    items: &'b mut Vec<Element<'a, Message, Theme, Renderer>>,
}

impl<'a, 'b, Message, Theme, Renderer> iced::advanced::Overlay<Message, Theme, Renderer>
    for Overlay<'a, 'b, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    fn layout(&mut self, renderer: &Renderer, bounds: iced::Size) -> iced::advanced::layout::Node {
        let item_height = BUTTON_SIZE.container_height();
        let items: Vec<_> = self
            .items
            .iter_mut()
            .enumerate()
            .map(|(i, item)| {
                let item = item.as_widget_mut().layout(
                    &mut self.tree.children[i],
                    renderer,
                    &Limits::new(Size::ZERO, bounds),
                );

                let y = -item_height * (i + 1) as f32
                    - BUTTON_TRIGGER_BETWEEN_SPACE
                    - BUTTON_SPACING * i as f32;
                let x = self.base_bounds.width - item.size().width;

                item.move_to(Point::new(x, y))
            })
            .collect();

        let min_x = items
            .iter()
            .map(|item| item.bounds().x)
            .fold(0.0f32, f32::min);
        let min_y = items
            .iter()
            .map(|item| item.bounds().y)
            .fold(0.0f32, f32::min);
        let children = items
            .into_iter()
            .map(|item| {
                let bounds = item.bounds();
                item.move_to(Point::new(bounds.x - min_x, bounds.y - min_y))
            })
            .collect();

        let size = Size::new(
            self.base_bounds.width - min_x,
            self.base_bounds.height - min_y,
        );

        Node::with_children(size, children)
            .move_to(self.base_bounds.position() + Vector::new(min_x, min_y))
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &iced::advanced::renderer::Style,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
    ) {
        let state = self.tree.state.downcast_ref::<State>();
        let progress = state.presented_progress;
        let viewport = layout.bounds();
        let trigger_center = self.base_bounds.center();

        for (i, item) in self.items.iter().enumerate() {
            let item_progress = item_progress(progress, i, self.items.len());
            if item_progress <= 0.0 {
                continue;
            }

            let item_layout = layout.child(i);
            let bounds = item_layout.bounds();
            let item_center = bounds.center();

            let translation = Vector::new(
                (trigger_center.x - item_center.x) * (1.0 - item_progress),
                (trigger_center.y - item_center.y) * (1.0 - item_progress),
            );
            let scale = ITEM_START_SCALE + (1.0 - ITEM_START_SCALE) * item_progress;

            let center = item_center + translation;
            let transformation = Transformation::translate(center.x, center.y)
                * Transformation::scale(scale)
                * Transformation::translate(-center.x, -center.y);

            renderer.with_transformation(transformation, |renderer| {
                item.as_widget().draw(
                    &self.tree.children[i],
                    renderer,
                    theme,
                    style,
                    item_layout,
                    cursor,
                    &viewport,
                );
            });
        }
    }

    fn update(
        &mut self,
        event: &iced::Event,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn iced::advanced::Clipboard,
        shell: &mut iced::advanced::Shell<'_, Message>,
    ) {
        let is_open = self.tree.state.downcast_ref::<State>().is_open();

        if is_open {
            let viewport = layout.bounds();
            for (i, item) in self.items.iter_mut().enumerate() {
                item.as_widget_mut().update(
                    &mut self.tree.children[i],
                    event,
                    layout.child(i),
                    cursor,
                    renderer,
                    clipboard,
                    shell,
                    &viewport,
                );
            }
        } else if let Event::Window(window::Event::RedrawRequested(_)) = event {
            let viewport = layout.bounds();
            let cursor = iced::advanced::mouse::Cursor::Unavailable;

            for (i, item) in self.items.iter_mut().enumerate() {
                item.as_widget_mut().update(
                    &mut self.tree.children[i],
                    event,
                    layout.child(i),
                    cursor,
                    renderer,
                    clipboard,
                    shell,
                    &viewport,
                );
            }
        }

        let over_menu = cursor.is_over(self.base_bounds)
            || (0..self.items.len()).any(|i| cursor.is_over(layout.child(i).bounds()));

        let state = self.tree.state.downcast_ref::<State>();
        let should_close = state.is_expanded
            && match event {
                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                | Event::Touch(touch::Event::FingerPressed { .. }) => {
                    !shell.is_event_captured() && !over_menu
                }
                Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
                | Event::Touch(touch::Event::FingerLifted { .. }) => {
                    !over_menu || (is_open && shell.is_event_captured())
                }
                _ => false,
            };

        if should_close {
            shell.capture_event();

            let state = self.tree.state.downcast_mut::<State>();
            state.is_expanded = false;

            state.is_pressed = false;
            shell.request_redraw();
        }
    }

    fn mouse_interaction(
        &self,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        _renderer: &Renderer,
    ) -> iced::advanced::mouse::Interaction {
        let state = self.tree.state.downcast_ref::<State>();
        if !state.is_open() {
            return iced::mouse::Interaction::None;
        }

        for (i, _) in self.items.iter().enumerate() {
            if cursor.is_over(layout.child(i).bounds()) {
                return iced::mouse::Interaction::Pointer;
            }
        }

        iced::mouse::Interaction::None
    }
}
