mod item;

use iced::time::Instant;

use iced::{
    Color, Element, Font, Length, Padding, Pixels, Rectangle, Size, Vector,
    advanced::{Widget, layout::Node, mouse, overlay, text, widget::Tree},
};
use iced_widget::text::IntoFragment;

use crate::{
    animation::motion::{self, Spring, SpringValue, default_spatial},
    style::StateLayer,
    theme::ColorScheme,
    widget::{
        self, badge,
        button::{self, ElevationStates, Outline},
        common_icons::{self},
        hybrid_icon::Icon,
        navrail::item::State as ItemState,
        tooltip,
    },
};

pub use item::Content as Item;
pub use item::Style as ItemStyle;

const MENU_ICON_SIZE: Pixels = Pixels(24.0);
const MENU_BUTTON_PADDING: f32 = 4.0;

const MENU_SECTION_SPACE: f32 = 16.0;
const ITEMS_SECTION_SPACE: f32 = 40.0;
// Small FAB container size
const FAB_OFFSET: f32 = (constants::CONTAINER_COLLAPSED_WIDTH - 56.0) / 2.0;
const MENU_OFFSET: f32 =
    (constants::CONTAINER_COLLAPSED_WIDTH - MENU_ICON_SIZE.0 - MENU_BUTTON_PADDING * 2.0) / 2.0;

pub(crate) mod constants {
    use crate::widget::navrail::item::INDICATOR_SMALL_WIDTH;

    pub const CONTAINER_COLLAPSED_WIDTH: f32 = 96.0;
    #[allow(dead_code)]
    pub const CONTAINER_EXPANDED_MIN_WIDTH: f32 = 220.0;
    pub const CONTAINER_EXPANDED_MAX_WIDTH: f32 = 360.0;
    pub const ITEM_OFFSET: f32 = (CONTAINER_COLLAPSED_WIDTH - INDICATOR_SMALL_WIDTH) / 2.0;
    pub const CONTAINER_VERTICAL_PADDING: f32 = 44.0;
}

#[cfg(feature = "pub-internal-const")]
pub use constants::*;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ItemAlignment {
    #[default]
    Top,
    Center,
    Bottom,
}

pub struct Fab<'a, Message>
where
    Message: Clone,
{
    pub icon: Icon<'a>,
    pub label: text::Fragment<'a>,
    pub style: crate::widget::fab::Style,
    pub label_size_scale: f32,
    pub on_press: Message,
}

impl<'a, Message: Clone> Clone for Fab<'a, Message> {
    fn clone(&self) -> Self {
        Self {
            icon: self.icon.clone(),
            label: self.label.clone(),
            style: self.style,
            label_size_scale: self.label_size_scale,
            on_press: self.on_press.clone(),
        }
    }
}

impl<'a, Message: Clone> Fab<'a, Message> {
    pub fn set_label_size_scale(&mut self, scale: f32) {
        self.label_size_scale = scale;
    }
}

#[derive(Default, Clone, Copy, PartialEq, PartialOrd)]
pub enum Mode {
    #[default]
    Collapsed,
    Expanded {
        /// The width of the navigation rail container.
        ///
        /// This value will be clamped between the minimum and maximum container width.
        width: Pixels,
    },
    /// Width is automatically determined by the widest item.
    ExpandedAuto,
}

impl Mode {
    fn width(&self) -> Pixels {
        match self {
            Mode::Collapsed => Pixels(constants::CONTAINER_COLLAPSED_WIDTH),
            Mode::Expanded { width } => Pixels(Into::<f32>::into(*width).clamp(
                constants::CONTAINER_EXPANDED_MIN_WIDTH,
                constants::CONTAINER_EXPANDED_MAX_WIDTH,
            )),
            Mode::ExpandedAuto => Pixels(constants::CONTAINER_EXPANDED_MIN_WIDTH),
        }
    }

    fn is_expanded(&self) -> bool {
        matches!(self, Mode::Expanded { .. } | Mode::ExpandedAuto)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    pub menu_button_style: button::Style,
    pub item_style: ItemStyle,
    pub badge_style: badge::Style,
    pub tooltip_style: tooltip::Style,
}

impl Style {
    pub fn new(theme: &(impl ColorScheme + ?Sized)) -> Self {
        let style = button::StateStyle {
            container: Color::TRANSPARENT,
            label: Color::TRANSPARENT,
            icon: theme.on_surface(),
            outline: Outline::default(),
        };
        Self {
            menu_button_style: button::Style {
                regular: style,
                unselected: style,
                selected: style,
                disabled: style,
                disabled_unselected: style,
                disabled_selected: style,
                elevation: ElevationStates::new(theme.shadow()),
                state_layer: StateLayer::new(theme.on_surface()),
                state_layer_unselected: StateLayer::new(theme.on_surface()),
                state_layer_selected: StateLayer::new(theme.on_surface()),
            },
            item_style: {
                let mut item_style = ItemStyle::new(theme);
                // Active icon uses `on_secondary_container` while the active
                // label stays `secondary`, per the M3 expressive spec.
                item_style.content_active = theme.on_secondary_container();
                item_style
            },
            badge_style: badge::Style::new(theme),
            tooltip_style: tooltip::Style::new(theme),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Labels<'a> {
    pub expand_menu: text::Fragment<'a>,
    pub collapse_menu: text::Fragment<'a>,
}

impl Default for Labels<'_> {
    fn default() -> Self {
        Self {
            expand_menu: "Expand menu".into(),
            collapse_menu: "Collapse menu".into(),
        }
    }
}

pub struct NavRail<'a, Message, Theme, Renderer>
where
    Message: 'a + Clone,
    Renderer: iced::advanced::text::Renderer
        + iced::advanced::svg::Renderer
        + iced_widget::renderer::geometry::Renderer,
    Renderer::Font: From<iced::Font>,
{
    style: Style,
    mode: Mode,
    on_menu_pressed: Option<Box<dyn Fn(Mode) -> Message + 'a>>,
    fab: Option<Fab<'a, Message>>,
    items: Vec<Item<'a, Message>>,
    label_font: Option<Font>,
    active_index: usize,
    item_alignment: ItemAlignment,
    container_vertical_padding: Option<f32>,
    motion_scheme: Option<motion::Scheme>,
    item_active_transition_spring: Option<Spring>,
    item_color_spring: Option<Spring>,
    expand_spring: Option<Spring>,
    hide_labels_when_collapsed: bool,
    menu_visible: bool,
    fab_visible: bool,
    scrollbar_color: Option<Color>,
    labels: Labels<'a>,
    tooltips_enabled: bool,

    menu_button: Option<Element<'a, Message, Theme, Renderer>>,
    fab_element: Option<Element<'a, Message, Theme, Renderer>>,
    item_elements: Vec<Element<'a, Message, Theme, Renderer>>,
    items_container: Option<Element<'a, Message, Theme, Renderer>>,
}

impl<'a, Message, Theme, Renderer> NavRail<'a, Message, Theme, Renderer>
where
    Message: 'a + Clone,
    Renderer: iced::advanced::text::Renderer
        + iced::advanced::svg::Renderer
        + iced_widget::renderer::geometry::Renderer,
    Renderer::Font: From<iced::Font>,
{
    #[must_use]
    pub fn new(style: Style, items: Vec<Item<'a, Message>>) -> Self {
        Self {
            style,
            mode: Mode::default(),
            on_menu_pressed: None,
            fab: None,
            items,
            label_font: None,
            active_index: 0,
            item_alignment: ItemAlignment::default(),
            container_vertical_padding: None,
            motion_scheme: None,
            item_active_transition_spring: None,
            item_color_spring: None,
            expand_spring: None,
            hide_labels_when_collapsed: false,
            menu_visible: true,
            fab_visible: true,
            scrollbar_color: None,
            labels: Labels::default(),
            tooltips_enabled: false,
            menu_button: None,
            fab_element: None,
            item_elements: Vec::new(),
            items_container: None,
        }
    }

    #[must_use]
    pub fn hide_labels_when_collapsed(mut self, hide: bool) -> Self {
        self.hide_labels_when_collapsed = hide;
        self
    }

    #[must_use]
    pub fn menu_visible(mut self, visible: bool) -> Self {
        self.menu_visible = visible;
        self
    }

    #[must_use]
    pub fn mode(mut self, mode: Mode) -> Self {
        self.mode = mode;
        self
    }

    crate::macros::maybe_delegate!(mode_maybe, mode, Option<Mode>);

    #[must_use]
    pub fn scrollbar_color(mut self, color: Color) -> Self {
        self.scrollbar_color = Some(color);
        self
    }

    crate::macros::maybe_set_some!(scrollbar_color_maybe, scrollbar_color, Option<Color>);

    #[must_use]
    pub fn fab(mut self, fab: Fab<'a, Message>) -> Self {
        self.fab = Some(fab);
        self
    }

    #[must_use]
    pub fn fab_label_size_scale(mut self, scale: f32) -> Self {
        if let Some(fab) = &mut self.fab {
            fab.label_size_scale = scale;
        }
        self
    }

    crate::macros::maybe_assign!(fab_maybe, fab, Option<Fab<'a, Message>>);

    /// Shows or hides the FAB immediately while keeping it configured on the rail.
    #[must_use]
    pub fn fab_visible(mut self, visible: bool) -> Self {
        self.fab_visible = visible;
        self
    }

    #[must_use]
    pub fn on_menu_pressed(mut self, on_press: impl Fn(Mode) -> Message + 'a) -> Self {
        self.on_menu_pressed = Some(Box::new(on_press));
        self
    }

    #[must_use]
    pub fn on_menu_pressed_maybe(
        mut self,
        on_press: Option<impl Fn(Mode) -> Message + 'a>,
    ) -> Self {
        self.on_menu_pressed =
            on_press.map(|callback| Box::new(callback) as Box<dyn Fn(Mode) -> Message + 'a>);
        self
    }

    #[must_use]
    pub fn active(mut self, index: usize) -> Self {
        self.active_index = index;
        self
    }

    #[must_use]
    pub fn label_font(mut self, font: iced::Font) -> Self {
        self.label_font = Some(font);
        self
    }

    crate::macros::maybe_assign!(label_font_maybe, label_font, Option<iced::Font>);

    #[must_use]
    pub fn item_alignment(mut self, alignment: ItemAlignment) -> Self {
        self.item_alignment = alignment;
        self
    }

    crate::macros::maybe_delegate!(item_alignment_maybe, item_alignment, Option<ItemAlignment>);

    #[must_use]
    pub fn container_vertical_padding(mut self, padding: f32) -> Self {
        self.container_vertical_padding = Some(padding);
        self
    }

    crate::macros::maybe_assign!(
        container_vertical_padding_maybe,
        container_vertical_padding,
        Option<f32>
    );

    #[must_use]
    pub fn motion_scheme(mut self, scheme: motion::Scheme) -> Self {
        self.motion_scheme = Some(scheme);
        self
    }

    crate::macros::maybe_assign!(motion_scheme_maybe, motion_scheme, Option<motion::Scheme>);

    #[must_use]
    pub fn item_active_transition_spring(mut self, spring: Spring) -> Self {
        self.item_active_transition_spring = Some(spring);
        self
    }

    crate::macros::maybe_assign!(
        item_active_transition_spring_maybe,
        item_active_transition_spring,
        Option<Spring>
    );

    #[must_use]
    pub fn item_color_spring(mut self, spring: Spring) -> Self {
        self.item_color_spring = Some(spring);
        self
    }

    crate::macros::maybe_assign!(item_color_spring_maybe, item_color_spring, Option<Spring>);

    #[must_use]
    pub fn expand_spring(mut self, spring: Spring) -> Self {
        self.expand_spring = Some(spring);
        self
    }

    crate::macros::maybe_assign!(expand_spring_maybe, expand_spring, Option<Spring>);

    #[must_use]
    pub fn tooltips_enabled(mut self, enabled: bool) -> Self {
        self.tooltips_enabled = enabled;
        self
    }

    #[must_use]
    pub fn labels(mut self, labels: Labels<'a>) -> Self {
        self.labels = labels;
        self
    }

    #[must_use]
    pub fn expand_menu_label(mut self, label: impl IntoFragment<'a>) -> Self {
        self.labels.expand_menu = label.into_fragment();
        self
    }

    #[must_use]
    pub fn collapse_menu_label(mut self, label: impl IntoFragment<'a>) -> Self {
        self.labels.collapse_menu = label.into_fragment();
        self
    }

    fn resolve_expand_spring(&self) -> Option<Spring> {
        match (self.expand_spring, self.motion_scheme) {
            (Some(spring), _) => Some(spring),
            (None, Some(scheme)) => Some(default_spatial(scheme)),
            (None, None) => None,
        }
    }

    fn mode_is_expanded(&self) -> bool {
        self.mode.is_expanded()
    }
}

struct State {
    width: SpringValue<f32>,
    /// Normalized expansion progress: 0 = collapsed, 1 = expanded
    expansion: SpringValue<f32>,
    generation: u64,
    auto_width: Option<(u64, f32)>,
}

/// An empty element used to keep the child tree shape stable when an optional
/// section (`menu`, `fab`, items container) is absent.
fn placeholder<'a, Message, Theme, Renderer>() -> Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: 'a + iced::advanced::Renderer,
{
    iced::widget::space().into()
}

/// The widest label the rail can show, or `None` when the rail sizes itself to its content.
fn label_max_width(mode: Mode) -> Option<f32> {
    match mode {
        Mode::ExpandedAuto => None,
        mode => Some(mode.width().0),
    }
}

/// Builds throwaway item widgets used to measure item widths and heights during
/// layout. Building them also shapes text, so callers only do it when a
/// measurement is really needed.
fn measure_items<'a, Message, Theme, Renderer>(
    nav_rail: &NavRail<'a, Message, Theme, Renderer>,
) -> Vec<Element<'a, Message, Theme, Renderer>>
where
    Message: 'a + Clone,
    Theme: 'a,
    Renderer: 'a
        + iced::advanced::text::Renderer
        + iced::advanced::svg::Renderer
        + iced_widget::renderer::geometry::Renderer,
    Renderer::Font: From<iced::Font>,
{
    nav_rail
        .items
        .iter()
        .enumerate()
        .map(|(index, content)| {
            let active = index == nav_rail.active_index;

            item::Item::new(
                nav_rail.style.item_style,
                content.clone(),
                nav_rail.mode.is_expanded(),
                nav_rail.label_font,
                active,
                nav_rail.hide_labels_when_collapsed,
                label_max_width(nav_rail.mode),
                nav_rail.style.tooltip_style,
            )
            .motion_scheme_maybe(nav_rail.motion_scheme)
            .active_transition_spring_maybe(nav_rail.item_active_transition_spring)
            .color_spring_maybe(nav_rail.item_color_spring)
            .into()
        })
        .collect()
}

fn measure_auto_width<'a, Message, Theme, Renderer>(
    nav_rail: &NavRail<'a, Message, Theme, Renderer>,
    renderer: &Renderer,
) -> f32
where
    Message: 'a + Clone,
    Theme: 'a,
    Renderer: 'a
        + iced::advanced::text::Renderer
        + iced::advanced::svg::Renderer
        + iced_widget::renderer::geometry::Renderer,
    Renderer::Font: From<iced::Font>,
{
    const FULLY_EXPANDED: f32 = 1.0;

    let limits = iced::advanced::layout::Limits::new(Size::ZERO, Size::new(f32::MAX, f32::MAX));
    let mut max_item_width = 0.0f32;

    for mut item_element in measure_items(nav_rail) {
        let mut item_tree = Tree::new(item_element.as_widget());
        item_tree.state.downcast_mut::<ItemState>().expansion = FULLY_EXPANDED;
        let item_layout = item_element
            .as_widget_mut()
            .layout(&mut item_tree, renderer, &limits);

        max_item_width = max_item_width.max(item_layout.bounds().width);
    }

    max_item_width.max(constants::CONTAINER_COLLAPSED_WIDTH)
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for NavRail<'a, Message, Theme, Renderer>
where
    Message: Clone,
    Theme: 'a,
    Renderer: 'a
        + iced::advanced::text::Renderer
        + iced::advanced::svg::Renderer
        + iced_widget::renderer::geometry::Renderer,
    Renderer::Font: From<iced::Font>,
{
    fn tag(&self) -> iced::advanced::widget::tree::Tag {
        iced::advanced::widget::tree::Tag::of::<State>()
    }

    fn state(&self) -> iced::advanced::widget::tree::State {
        let now = Instant::now();
        let spring = self
            .resolve_expand_spring()
            .unwrap_or_else(|| default_spatial(motion::Scheme::default()));
        let width = self.mode.width().0;
        let expansion = self.mode_is_expanded() as usize as f32;
        let mut state = State {
            width: SpringValue::new(width, spring, now),
            expansion: SpringValue::new(expansion, spring, now),
            generation: 0,
            auto_width: None,
        };
        state.width.reset(width, now);
        state.expansion.reset(expansion, now);

        iced::advanced::widget::tree::State::new(state)
    }

    fn diff(&self, tree: &mut Tree) {
        let now = Instant::now();
        let state = tree.state.downcast_mut::<State>();
        state.generation = state.generation.wrapping_add(1);

        let spring = self.resolve_expand_spring();
        state.width.set_model_if_some(spring);
        state.expansion.set_model_if_some(spring);
        state
            .expansion
            .set_target(self.mode_is_expanded() as usize as f32, now);

        // For ExpandedAuto the real width target is computed in layout() from
        // measured item widths. Setting it here to mode.width() (220.0) and
        // then overriding it in layout() causes the spring to be reset twice
        // per frame, breaking the animation. Only set it in diff() for modes
        // with a fixed known width.
        if !matches!(self.mode, Mode::ExpandedAuto) {
            state.width.set_target(self.mode.width().0, now);
        }

        let target_len = 3;
        if tree.children.len() != target_len {
            tree.children = self.children();
        } else {
            let placeholder: Element<'a, Message, Theme, Renderer> = placeholder();

            if let Some(menu) = self.menu_button.as_ref() {
                tree.children[0].diff(menu.as_widget());
            } else {
                tree.children[0].diff(placeholder.as_widget());
            }
            if let Some(fab) = self.fab_element.as_ref() {
                tree.children[1].diff(fab.as_widget());
            } else {
                tree.children[1].diff(placeholder.as_widget());
            }
            if let Some(container) = self.items_container.as_ref() {
                tree.children[2].diff(container.as_widget());
            } else {
                tree.children[2].diff(placeholder.as_widget());
            }
        }
    }

    fn children(&self) -> Vec<Tree> {
        let mut children = Vec::with_capacity(3);
        let placeholder: Element<'a, Message, Theme, Renderer> = placeholder();

        if let Some(menu) = self.menu_button.as_ref() {
            children.push(Tree::new(menu.as_widget()));
        } else {
            children.push(Tree::new(placeholder.as_widget()));
        }

        if let Some(fab) = self.fab_element.as_ref() {
            children.push(Tree::new(fab.as_widget()));
        } else {
            children.push(Tree::new(placeholder.as_widget()));
        }

        if let Some(container) = self.items_container.as_ref() {
            children.push(Tree::new(container.as_widget()));
        } else {
            children.push(Tree::new(placeholder.as_widget()));
        }

        children
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Shrink, Length::Fill)
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> iced::advanced::layout::Node {
        let rail_state = tree.state.downcast_mut::<State>();
        let collapsed_width = constants::CONTAINER_COLLAPSED_WIDTH;
        let expansion = rail_state.expansion.value().clamp(0.0, 1.0);
        let fab_visibility = (self.fab_element.is_some() && self.fab_visible) as usize as f32;

        let mut measure_widgets =
            if self.items_container.is_some() && self.item_alignment != ItemAlignment::Top {
                measure_items(self)
            } else {
                Vec::new()
            };

        let effective_target = match self.mode {
            Mode::ExpandedAuto => match rail_state.auto_width {
                Some((generation, width)) if generation == rail_state.generation => width,
                _ => {
                    let width = measure_auto_width(self, renderer);
                    rail_state.auto_width = Some((rail_state.generation, width));
                    width
                }
            },
            _ => self.mode.width().0,
        };

        let width = rail_state.width.value().max(collapsed_width);
        let container_bounds = limits.resolve(Length::Fixed(width), Length::Fill, Size::ZERO);
        let container_vertical_padding = self
            .container_vertical_padding
            .unwrap_or(constants::CONTAINER_VERTICAL_PADDING);
        let mut offset = container_vertical_padding;

        let mut children = Vec::with_capacity(3);
        let mut tree_index = 0;

        // Menu
        let mut menu_placeholder: Element<'a, Message, Theme, Renderer> = placeholder();
        let menu_layout = self
            .menu_button
            .as_mut()
            .unwrap_or(&mut menu_placeholder)
            .as_widget_mut()
            .layout(&mut tree.children[tree_index], renderer, limits)
            .translate(Vector::new(MENU_OFFSET, offset));
        let menu_natural_height = menu_layout.bounds().height;
        let menu_effective_height = if self.menu_button.is_some() && self.menu_visible {
            menu_natural_height
        } else {
            0.0
        };
        children.push(menu_layout);
        tree_index += 1;
        offset += menu_effective_height;
        if self.menu_button.is_some() && self.menu_visible {
            offset += match self.fab_element.is_some() {
                true => MENU_SECTION_SPACE,
                false => ITEMS_SECTION_SPACE,
            };
        }
        offset = offset.round();

        // Fab
        let mut fab_placeholder: Element<'a, Message, Theme, Renderer> = placeholder();
        let fab_layout = self
            .fab_element
            .as_mut()
            .unwrap_or(&mut fab_placeholder)
            .as_widget_mut()
            .layout(&mut tree.children[tree_index], renderer, limits)
            .translate(Vector::new(FAB_OFFSET, offset));
        let fab_height = fab_layout.bounds().height;
        children.push(fab_layout);
        tree_index += 1;
        if fab_visibility > 0.0 {
            offset += fab_height + ITEMS_SECTION_SPACE;
            offset = offset.round();
        }

        let item_start_offset = offset;
        let available_height =
            (container_bounds.height - item_start_offset - container_vertical_padding).max(0.0);

        // Set expansion on real items inside the scrollable container
        if let Some(container) = self.items_container.as_mut() {
            let scrollable_tree = &mut tree.children[tree_index];
            if !scrollable_tree.children.is_empty() {
                let inner_scrollable_tree = &mut scrollable_tree.children[0];
                if !inner_scrollable_tree.children.is_empty() {
                    let column_tree = &mut inner_scrollable_tree.children[0];
                    for item_tree in column_tree.children.iter_mut() {
                        let item_state = if self.tooltips_enabled
                            && !self.mode.is_expanded()
                            && self.hide_labels_when_collapsed
                        {
                            item_tree.children.first_mut().map(|inner| &mut inner.state)
                        } else {
                            Some(&mut item_tree.state)
                        };

                        if let Some(item_state) = item_state {
                            item_state.downcast_mut::<ItemState>().expansion = expansion;
                        }
                    }
                }
            }

            let item_limits =
                iced::advanced::layout::Limits::new(Size::ZERO, Size::new(width, available_height));

            // Measure total content height for item alignment
            let mut content_height = 0.0;
            for item_element in &mut measure_widgets {
                let mut temp_tree = Tree::new(item_element.as_widget());
                temp_tree.state.downcast_mut::<ItemState>().expansion = expansion;
                let item_layout =
                    item_element
                        .as_widget_mut()
                        .layout(&mut temp_tree, renderer, &item_limits);
                content_height += item_layout.bounds().height;
            }

            let alignment_offset = match self.item_alignment {
                ItemAlignment::Top => 0.0,
                ItemAlignment::Center => ((available_height - content_height) / 2.0).max(0.0),
                ItemAlignment::Bottom => (available_height - content_height).max(0.0),
            };

            let scrollable_layout = container
                .as_widget_mut()
                .layout(scrollable_tree, renderer, &item_limits)
                .translate(Vector::new(0.0, item_start_offset + alignment_offset));
            children.push(scrollable_layout);
        }

        let now = Instant::now();
        rail_state.width.set_target(effective_target, now);

        Node::with_children(container_bounds, children)
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
        let mut layouts = layout.children();
        let container_vertical_padding = self
            .container_vertical_padding
            .unwrap_or(constants::CONTAINER_VERTICAL_PADDING);

        // Collect all child layouts up front so we can control draw order.
        let menu_layout = layouts.next().unwrap();
        let fab_layout = layouts.next().unwrap();
        let scrollable_layout = layouts.next().unwrap();

        // Menu visibility changes immediately; no clipping animation is applied.
        if self.menu_visible
            && let Some(menu) = self.menu_button.as_ref()
        {
            menu.as_widget().draw(
                &tree.children[0],
                renderer,
                theme,
                style,
                menu_layout,
                cursor,
                viewport,
            );
        }

        // Items area follows the FAB only while it is visible.
        let fab_visibility = (self.fab_element.is_some() && self.fab_visible) as usize as f32;
        let start_y = fab_layout.bounds().y
            + (fab_layout.bounds().height + ITEMS_SECTION_SPACE) * fab_visibility;
        let clip_height =
            (layout.bounds().y + layout.bounds().height - container_vertical_padding - start_y)
                .max(0.0);

        // Items
        if self.items_container.is_some() {
            let items_clip = Rectangle {
                x: layout.bounds().x,
                y: start_y,
                width: layout.bounds().width,
                height: clip_height,
            };

            renderer.with_layer(items_clip, |renderer| {
                self.items_container.as_ref().unwrap().as_widget().draw(
                    &tree.children[2],
                    renderer,
                    theme,
                    style,
                    scrollable_layout,
                    cursor,
                    viewport,
                );
            });
        }

        // Fab — drawn last so it is not affected by any clip state left over
        // from the items `with_layer` operation (which can leak between frames
        // and cause the Fab icon to disappear when ItemAlignment changes).
        if self.fab_visible
            && let Some(fab) = self.fab_element.as_ref()
        {
            fab.as_widget().draw(
                &tree.children[1],
                renderer,
                theme,
                style,
                fab_layout,
                cursor,
                viewport,
            );
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        viewport: &iced::Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let mut tree_index = 0;
        let mut layouts = layout.children();

        // Menu
        let menu_layout = layouts.next().unwrap();
        if self.menu_visible
            && let Some(menu) = self.menu_button.as_ref()
            && cursor.is_over(menu_layout.bounds())
        {
            let interaction = menu.as_widget().mouse_interaction(
                &tree.children[tree_index],
                menu_layout,
                cursor,
                viewport,
                renderer,
            );
            if interaction != mouse::Interaction::None {
                return interaction;
            }
        }
        tree_index += 1;

        // Fab
        let fab_layout = layouts.next().unwrap();
        if self.fab_visible
            && let Some(fab) = self.fab_element.as_ref()
            && cursor.is_over(fab_layout.bounds())
        {
            let interaction = fab.as_widget().mouse_interaction(
                &tree.children[tree_index],
                fab_layout,
                cursor,
                viewport,
                renderer,
            );
            if interaction != mouse::Interaction::None {
                return interaction;
            }
        }
        tree_index += 1;

        if let Some(container) = self.items_container.as_ref() {
            let scrollable_layout = layouts.next().unwrap();
            let interaction = container.as_widget().mouse_interaction(
                &tree.children[tree_index],
                scrollable_layout,
                cursor,
                viewport,
                renderer,
            );
            if interaction != mouse::Interaction::None {
                return interaction;
            }
        }

        mouse::Interaction::None
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &iced::Event,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn iced::advanced::Clipboard,
        shell: &mut iced::advanced::Shell<'_, Message>,
        viewport: &iced::Rectangle,
    ) {
        let mut layouts = layout.children();
        let mut tree_index = 0;

        // Menu visibility changes immediately.
        let menu_layout = layouts.next().unwrap();
        if self.menu_visible
            && let Some(menu) = &mut self.menu_button
        {
            menu.as_widget_mut().update(
                &mut tree.children[tree_index],
                event,
                menu_layout,
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );
        }
        tree_index += 1;

        // FAB visibility changes immediately.
        let fab_layout = layouts.next().unwrap();
        if self.fab_visible
            && let Some(fab) = &mut self.fab_element
        {
            fab.as_widget_mut().update(
                &mut tree.children[tree_index],
                event,
                fab_layout,
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );
        }
        tree_index += 1;

        let state = tree.state.downcast_mut::<State>();

        // Delegate events to scrollable items container
        if let Some(container) = self.items_container.as_mut() {
            let scrollable_layout = layouts.next().unwrap();
            container.as_widget_mut().update(
                &mut tree.children[tree_index],
                event,
                scrollable_layout,
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );
        }

        let now = match event {
            iced::Event::Window(iced::window::Event::RedrawRequested(at)) => *at,
            _ => Instant::now(),
        };

        let spring = self.resolve_expand_spring();
        state.width.set_model_if_some(spring);
        state.expansion.set_model_if_some(spring);

        let width_before = state.width.value();
        let expansion_before = state.expansion.value();

        state.width.step(now);
        state.expansion.step(now);

        let layout_moved =
            state.width.value() != width_before || state.expansion.value() != expansion_before;

        if !state.width.is_at_rest() || !state.expansion.is_at_rest() {
            shell.request_redraw();
        }

        if layout_moved {
            shell.invalidate_layout();
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: iced::advanced::Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        // The children own overlays of their own (the tooltips of the menu button, the FAB and the
        // items), so they have to be forwarded to the runtime.
        let mut layouts = layout.children();
        let menu_layout = layouts.next();
        let fab_layout = layouts.next();
        let container_layout = layouts.next();

        let mut states = tree.children.iter_mut();
        let menu_state = states.next();
        let fab_state = states.next();
        let container_state = states.next();

        let mut children: Vec<overlay::Element<'b, Message, Theme, Renderer>> =
            Vec::with_capacity(3);

        if let (Some(menu), Some(state), Some(layout)) =
            (self.menu_button.as_mut(), menu_state, menu_layout)
            && let Some(overlay) =
                menu.as_widget_mut()
                    .overlay(state, layout, renderer, viewport, translation)
        {
            children.push(overlay);
        }

        if let (Some(fab), Some(state), Some(layout)) =
            (self.fab_element.as_mut(), fab_state, fab_layout)
            && let Some(overlay) =
                fab.as_widget_mut()
                    .overlay(state, layout, renderer, viewport, translation)
        {
            children.push(overlay);
        }

        if let (Some(container), Some(state), Some(layout)) = (
            self.items_container.as_mut(),
            container_state,
            container_layout,
        ) && let Some(overlay) =
            container
                .as_widget_mut()
                .overlay(state, layout, renderer, viewport, translation)
        {
            children.push(overlay);
        }

        (!children.is_empty()).then(|| overlay::Group::with_children(children).overlay())
    }
}

impl<'a, Message, Theme, Renderer> From<NavRail<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: Clone,
    Theme: 'a + iced_widget::scrollable::Catalog,
    <Theme as iced_widget::scrollable::Catalog>::Class<'a>:
        From<iced_widget::scrollable::StyleFn<'a, Theme>>,
    Renderer: 'a
        + iced::advanced::text::Renderer
        + iced::advanced::svg::Renderer
        + iced_widget::renderer::geometry::Renderer,
    Renderer::Font: From<iced::Font>,
{
    fn from(mut value: NavRail<'a, Message, Theme, Renderer>) -> Self {
        value.menu_button = value.on_menu_pressed.as_ref().map(|on_press| {
            let icon = if value.mode.is_expanded() {
                common_icons::MENU_OPEN.clone()
            } else {
                common_icons::MENU.clone()
            };
            let button_size = MENU_ICON_SIZE + MENU_BUTTON_PADDING * 2.0;
            let button: Element<'a, Message, Theme, Renderer> = crate::widget::button(
                value.style.menu_button_style,
                button::Content::Icon(Icon::Svg(icon)),
            )
            .size(button::Size {
                width: Length::Fixed(button_size.0),
                height: Length::Fixed(button_size.0),
                spacing: Pixels::ZERO,
                padding: Padding::default(),
                icon_size: MENU_ICON_SIZE,
                label_size: Pixels(0.0),
                corner_radius: button::CornerRadius {
                    style: button::CornerStyle::Rounded,
                    shape_morph: false,
                    rounded: f32::MAX.into(),
                    square: f32::MAX.into(),
                    pressed: f32::MAX.into(),
                },
            })
            .on_press((on_press)(value.mode))
            .into();

            let label = if value.mode.is_expanded() {
                value.labels.collapse_menu.clone()
            } else {
                value.labels.expand_menu.clone()
            };

            if value.tooltips_enabled {
                widget::tooltip(value.style.tooltip_style, button)
                    .label(label)
                    .position(tooltip::Position::Above)
                    .into()
            } else {
                button
            }
        });

        let initial_fab_scale = if value.mode_is_expanded() { 1.0 } else { 0.0 };
        value.fab_element = value.fab.as_ref().map(|fab| {
            let fab_clone = fab.clone();
            let tooltip_label = (!value.mode_is_expanded()).then(|| fab_clone.label.clone());
            let content = widget::fab::Content::Extended {
                icon: fab_clone.icon,
                label: fab_clone.label,
            };

            let button: Element<'a, Message, Theme, Renderer> =
                widget::fab(fab_clone.style, content, fab_clone.on_press)
                    .label_size_scale(initial_fab_scale)
                    .label_font_maybe(value.label_font)
                    .into();

            // The label is hidden while collapsed, so surface it as a tooltip.
            if value.tooltips_enabled {
                widget::tooltip(value.style.tooltip_style, button)
                    .label_maybe(tooltip_label)
                    .position(tooltip::Position::Right)
                    .into()
            } else {
                button
            }
        });

        let items = std::mem::take(&mut value.items);
        let expanded = value.mode.is_expanded();
        value.item_elements = items
            .iter()
            .enumerate()
            .map(|(index, content)| {
                let active = index == value.active_index;
                let element: Element<'a, Message, Theme, Renderer> = item::Item::new(
                    value.style.item_style,
                    content.clone(),
                    expanded,
                    value.label_font,
                    active,
                    value.hide_labels_when_collapsed,
                    label_max_width(value.mode),
                    value.style.tooltip_style,
                )
                .motion_scheme_maybe(value.motion_scheme)
                .active_transition_spring_maybe(value.item_active_transition_spring)
                .color_spring_maybe(value.item_color_spring)
                .into();

                if value.tooltips_enabled && !expanded && value.hide_labels_when_collapsed {
                    widget::tooltip(value.style.tooltip_style, element)
                        .label(content.label.clone())
                        .position(tooltip::Position::Right)
                        .into()
                } else {
                    element
                }
            })
            .collect();

        // Build scrollable container from items
        let scrollbar_color = value
            .scrollbar_color
            .unwrap_or(value.style.item_style.content_inactive);
        let taken_items = std::mem::take(&mut value.item_elements);
        let col = iced::widget::column(taken_items);
        let scroller = crate::widget::scrollable::scrollable_with_color(scrollbar_color, col);
        value.items_container = Some(scroller.into());

        // Restore raw items for potential future layout measurements
        value.items = items;

        Element::new(value)
    }
}
