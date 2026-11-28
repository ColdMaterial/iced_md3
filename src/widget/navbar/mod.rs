mod item;

use iced::Color;
use iced::Element;
use iced::Font;
use iced::Length;
use iced::Pixels;
use iced::Point;
use iced::Size;
use iced::advanced::Widget;
use iced::advanced::layout::Node;
use iced::advanced::mouse;
use iced::advanced::renderer::Quad;
use iced::advanced::widget::Tree;

use crate::animation::motion;
use crate::animation::motion::Spring;
use crate::theme::ColorScheme;

pub use item::Content as Item;
pub use item::Style as ItemStyle;

pub(crate) mod constants {
    pub const CONTAINER_HEIGHT: f32 = 64.0;
    pub const ITEM_LARGE_SPACING: f32 = 20.0;
}

#[cfg(feature = "pub-internal-const")]
pub use constants::*;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Mode {
    Compact,
    Large,
    #[default]
    Auto,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    pub container_color: Color,
    pub item_style: ItemStyle,
}

impl Style {
    pub fn new(theme: &(impl ColorScheme + ?Sized)) -> Self {
        Self {
            container_color: theme.surface_container(),
            item_style: ItemStyle::new(theme),
        }
    }
}

pub struct NavBar<'a, Message, Theme, Renderer>
where
    Message: Clone,
    Renderer: iced::advanced::text::Renderer
        + iced::advanced::svg::Renderer
        + iced_widget::renderer::geometry::Renderer,
    Renderer::Font: From<iced::Font>,
{
    items: Vec<Item<'a, Message>>,
    style: Style,
    label_font: Option<Font>,
    mode: Mode,
    active_index: usize,
    motion_scheme: Option<motion::Scheme>,
    effects_spring: Option<Spring>,
    spatial_spring: Option<Spring>,

    // Cache
    item_widgets: Vec<item::Item<'a, Message, Theme, Renderer>>,
}

impl<'a, Message, Theme, Renderer> NavBar<'a, Message, Theme, Renderer>
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
    pub fn new(style: Style, items: Vec<Item<'a, Message>>) -> Self {
        Self {
            items,
            style,
            label_font: None,
            mode: Mode::default(),
            active_index: 0,
            motion_scheme: None,
            effects_spring: None,
            spatial_spring: None,
            item_widgets: Vec::new(),
        }
    }

    #[must_use]
    pub fn item(mut self, item: Item<'a, Message>) -> Self {
        self.items.push(item);
        self
    }

    #[must_use]
    pub fn mode(mut self, mode: Mode) -> Self {
        self.mode = mode;
        self
    }

    #[must_use]
    pub fn active_index(mut self, index: usize) -> Self {
        self.active_index = index;
        self
    }

    #[must_use]
    pub fn label_font(mut self, font: Font) -> Self {
        self.label_font = Some(font);
        self
    }

    crate::macros::maybe_assign!(label_font_maybe, label_font, Option<Font>);

    #[must_use]
    pub fn motion_scheme(mut self, scheme: motion::Scheme) -> Self {
        self.motion_scheme = Some(scheme);
        self
    }

    crate::macros::maybe_assign!(motion_scheme_maybe, motion_scheme, Option<motion::Scheme>);

    #[must_use]
    pub fn effects_spring(mut self, spring: Spring) -> Self {
        self.effects_spring = Some(spring);
        self
    }

    crate::macros::maybe_assign!(effects_spring_maybe, effects_spring, Option<Spring>);

    #[must_use]
    pub fn spatial_spring(mut self, spring: Spring) -> Self {
        self.spatial_spring = Some(spring);
        self
    }

    crate::macros::maybe_assign!(spatial_spring_maybe, spatial_spring, Option<Spring>);

    fn get_items(&self, mode: item::Mode) -> Vec<item::Item<'a, Message, Theme, Renderer>> {
        self.items
            .clone()
            .into_iter()
            .enumerate()
            .map(|(index, item)| {
                item::Item::new(
                    self.style.item_style,
                    item,
                    self.label_font,
                    index == self.active_index,
                )
                .mode(mode)
                .motion_scheme_maybe(self.motion_scheme)
                .active_transition_spring_maybe(self.spatial_spring)
                .color_spring_maybe(self.effects_spring)
            })
            .collect()
    }

    fn compact_layout(
        &mut self,
        container_bounds: Size,
        tree: &mut iced::advanced::widget::Tree,
        renderer: &Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> Node {
        let cell_width = container_bounds.width / self.items.len() as f32;

        self.layout_items(
            container_bounds,
            tree,
            renderer,
            limits,
            move |index, item_width| cell_width * index as f32 + (cell_width - item_width) / 2.0,
        )
    }

    fn large_layout(
        &mut self,
        container_bounds: Size,
        item_width: f32,
        tree: &mut iced::advanced::widget::Tree,
        renderer: &Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> Node {
        let spacing = constants::ITEM_LARGE_SPACING;
        let total_width = item_width * self.item_widgets.len() as f32
            + spacing * self.item_widgets.len().saturating_sub(1) as f32;
        let offset = (container_bounds.width - total_width) / 2.0;

        self.layout_items(container_bounds, tree, renderer, limits, move |index, _| {
            offset + (item_width + spacing) * index as f32
        })
    }

    /// Lays out every item, positioning it horizontally with `x_of`
    /// (`index`, `item_width`) and centering it vertically.
    fn layout_items(
        &mut self,
        container_bounds: Size,
        tree: &mut iced::advanced::widget::Tree,
        renderer: &Renderer,
        limits: &iced::advanced::layout::Limits,
        x_of: impl Fn(usize, f32) -> f32,
    ) -> Node {
        let mut item_widgets = std::mem::take(&mut self.item_widgets);

        let item_nodes = item_widgets
            .iter_mut()
            .enumerate()
            .map(|(index, item)| {
                let item = (item as &mut dyn Widget<Message, Theme, Renderer>).layout(
                    &mut tree.children[index],
                    renderer,
                    limits,
                );

                let item_bounds = item.size();
                item.move_to(Point::new(
                    x_of(index, item_bounds.width),
                    (container_bounds.height - item_bounds.height) / 2.0,
                ))
            })
            .collect();
        self.item_widgets = item_widgets;

        Node::with_children(container_bounds, item_nodes)
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for NavBar<'a, Message, Theme, Renderer>
where
    Message: 'a + Clone,
    Theme: 'a,
    Renderer: 'a
        + iced::advanced::text::Renderer
        + iced::advanced::svg::Renderer
        + iced_widget::renderer::geometry::Renderer,
    Renderer::Font: From<iced::Font>,
{
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Fixed(constants::CONTAINER_HEIGHT))
    }

    fn diff(&self, tree: &mut iced::advanced::widget::Tree) {
        let children: Vec<&dyn Widget<Message, Theme, Renderer>> = self
            .item_widgets
            .iter()
            .map(|item| item as &dyn Widget<Message, Theme, Renderer>)
            .collect();

        tree.diff_children(&children);
    }

    fn children(&self) -> Vec<iced::advanced::widget::Tree> {
        self.item_widgets
            .iter()
            .map(|item| Tree::new(item as &dyn Widget<Message, Theme, Renderer>))
            .collect()
    }

    fn layout(
        &mut self,
        tree: &mut iced::advanced::widget::Tree,
        renderer: &Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> iced::advanced::layout::Node {
        let container_bounds = limits.resolve(
            Length::Fill,
            Length::Fixed(constants::CONTAINER_HEIGHT),
            Size::ZERO,
        );

        if self.mode == Mode::Compact {
            self.item_widgets = self.get_items(item::Mode::Compact);

            return self.compact_layout(container_bounds, tree, renderer, limits);
        }

        // Measure the natural width of every item once, then reuse the same
        // widgets for the final layout instead of rebuilding them.
        self.item_widgets = self.get_items(item::Mode::Large { width: None });

        if self.item_widgets.is_empty() {
            return Node::new(container_bounds);
        }

        let mut widest = 0.0f32;
        for (index, item) in self.item_widgets.iter_mut().enumerate() {
            let node = (item as &mut dyn Widget<Message, Theme, Renderer>).layout(
                &mut tree.children[index],
                renderer,
                limits,
            );

            widest = widest.max(node.bounds().width);
        }

        let item_count = self.item_widgets.len();
        let large_total_width = widest * item_count as f32
            + constants::ITEM_LARGE_SPACING * item_count.saturating_sub(1) as f32;

        let use_large = match self.mode {
            Mode::Large => true,
            Mode::Auto => large_total_width <= container_bounds.width,
            Mode::Compact => false,
        };

        if !use_large {
            self.item_widgets = self.get_items(item::Mode::Compact);

            return self.compact_layout(container_bounds, tree, renderer, limits);
        }

        let mode = item::Mode::Large {
            width: Some(Pixels(widest)),
        };
        for item in &mut self.item_widgets {
            item.set_mode(mode);
        }

        self.large_layout(container_bounds, widest, tree, renderer, limits)
    }

    fn draw(
        &self,
        tree: &iced::advanced::widget::Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &iced::advanced::renderer::Style,
        layout: iced::advanced::Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &iced::Rectangle,
    ) {
        renderer.fill_quad(
            Quad {
                bounds: layout.bounds(),
                ..Default::default()
            },
            self.style.container_color,
        );

        for (index, item) in self.item_widgets.iter().enumerate() {
            item.draw(
                &tree.children[index],
                renderer,
                theme,
                style,
                layout.child(index),
                cursor,
                viewport,
            );
        }
    }

    fn update(
        &mut self,
        tree: &mut iced::advanced::widget::Tree,
        event: &iced::Event,
        layout: iced::advanced::Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn iced::advanced::Clipboard,
        shell: &mut iced::advanced::Shell<'_, Message>,
        viewport: &iced::Rectangle,
    ) {
        for (index, item) in self.item_widgets.iter_mut().enumerate() {
            (item as &mut dyn Widget<Message, Theme, Renderer>).update(
                &mut tree.children[index],
                event,
                layout.child(index),
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            )
        }
    }

    fn mouse_interaction(
        &self,
        tree: &iced::advanced::widget::Tree,
        layout: iced::advanced::Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &iced::Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        for (index, item) in self.item_widgets.iter().enumerate() {
            let interaction = (item as &dyn Widget<Message, Theme, Renderer>).mouse_interaction(
                &tree.children[index],
                layout.child(index),
                cursor,
                viewport,
                renderer,
            );

            if interaction != mouse::Interaction::None {
                return interaction;
            }
        }

        match cursor.is_over(layout.bounds()) {
            true => mouse::Interaction::Idle,
            false => mouse::Interaction::None,
        }
    }
}

impl<'a, Message, Theme, Renderer> From<NavBar<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a + Clone,
    Theme: 'a,
    Renderer: 'a
        + iced::advanced::text::Renderer
        + iced::advanced::svg::Renderer
        + iced_widget::renderer::geometry::Renderer,
    Renderer::Font: From<iced::Font>,
{
    fn from(mut value: NavBar<'a, Message, Theme, Renderer>) -> Self {
        value.item_widgets = value.get_items(item::Mode::Compact);
        Element::new(value)
    }
}
