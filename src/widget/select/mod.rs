use std::{
    cell::RefCell,
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    rc::Rc,
};

use iced::{
    Border, Color, Element, Event, Font, Length, Padding, Pixels, Point, Radians, Rectangle, Size,
    Vector,
    advanced::{
        Clipboard, Layout, Shell, Widget, clipboard,
        layout::{self, Limits},
        mouse, overlay,
        renderer::Quad,
        svg,
        text::{self, Text},
        widget::tree::{self, Tree},
    },
    alignment::Vertical,
    keyboard,
    time::{Duration, Instant},
    touch, window,
};
use iced_widget::text::IntoFragment;

use crate::{
    animation::motion::{self, Spring, SpringValue, default_spatial},
    style::{DIM_ALPHA, Elevation, mix_colors, shadow},
    theme::ColorScheme,
    widget::{
        common_icons,
        hybrid_icon::Icon,
        internal::{
            elide, focus::Focus as Blink, fuzzy::score as fuzzy_score, icon, interaction,
            placement, scrollbar, text as internal_text,
        },
        menu,
        text_input::{
            Action, ActionLabels, ContextMenu, ContextMenuHost, ContextMenuOverlay, Cursor,
            EditKind, Editor, History, Value, cursor, edit_value,
        },
        tooltip,
    },
};

pub(crate) mod constants {
    use iced::Pixels;

    use crate::widget::internal::scrollbar;

    pub const FIELD_HEIGHT: Pixels = Pixels(56.0);
    pub const FIELD_RADIUS: f32 = 4.0;
    pub const FIELD_OUTLINE_WIDTH: f32 = 1.0;
    pub const FIELD_OUTLINE_WIDTH_OPEN: f32 = 2.0;
    pub const FIELD_TEXT_PADDING: f32 = 16.0;
    pub const FIELD_TEXT_SIZE: Pixels = Pixels(16.0);

    pub const ICON_SIZE: Pixels = Pixels(24.0);
    pub const ICON_PADDING: f32 = 16.0;
    pub const CLEAR_GAP: f32 = 8.0;
    pub const CLEAR_ICON_SIZE: Pixels = Pixels(18.0);
    pub const CLEAR_TOUCH_TARGET: f32 = 32.0;

    pub const GAP: f32 = 8.0;
    pub const VIEWPORT_MARGIN: f32 = 8.0;

    pub const MENU_RADIUS: f32 = 16.0;
    pub const MENU_PADDING: f32 = 4.0;
    pub const ITEM_SPACING: f32 = 2.0;
    pub const ITEM_HEIGHT: f32 = 44.0;
    pub const ITEM_RADIUS: f32 = 8.0;
    pub const ITEM_PADDING: f32 = 12.0;
    pub const ITEM_TEXT_SIZE: Pixels = Pixels(14.0);
    pub const ITEM_ICON_SIZE: Pixels = Pixels(18.0);
    pub const ITEM_ICON_SLOT: f32 = 20.0;
    pub const ITEM_ICON_SPACING: f32 = 8.0;

    pub const SEPARATOR_HEIGHT: Pixels = Pixels(9.0);
    pub const SEPARATOR_RULE_HEIGHT: f32 = 1.0;

    pub const SEARCH_HEIGHT: f32 = 44.0;

    pub const DEFAULT_MAX_MENU_HEIGHT: Pixels = Pixels(320.0);
    pub const DEFAULT_SEARCH_THRESHOLD: usize = 10;

    pub const SCROLLBAR_GUTTER: f32 = scrollbar::GUTTER;

    pub const STATE_LAYER_OPACITY: f32 = crate::style::HOVER_STATE_LAYER_OPACITY;
}

#[cfg(feature = "pub-internal-const")]
pub use constants::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum OpenDirection {
    #[default]
    Down,
    Up,
    Center,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    pub field_background: Color,
    pub field_outline: Color,
    pub field_outline_hovered: Color,
    pub field_outline_open: Color,
    pub field_text: Color,
    pub field_placeholder: Color,
    pub field_icon: Color,
    pub menu_background: Color,
    pub menu_shadow: Color,
    pub elevation: Elevation,
    pub item_text: Color,
    pub item_disabled_text: Color,
    pub item_selected_text: Color,
    pub item_selected_background: Color,
    pub item_state_layer: Color,
    pub separator: Color,
    pub scrollbar: Color,
}

impl Style {
    #[must_use]
    pub fn new(theme: &(impl ColorScheme + ?Sized)) -> Self {
        Self {
            field_background: Color::TRANSPARENT,
            field_outline: theme.outline(),
            field_outline_hovered: theme.on_surface(),
            field_outline_open: theme.primary(),
            field_text: theme.on_surface(),
            field_placeholder: theme.on_surface_variant(),
            field_icon: theme.on_surface_variant(),
            menu_background: theme.surface_container_low(),
            menu_shadow: theme.shadow(),
            elevation: Elevation::Level2,
            item_text: theme.on_surface(),
            item_disabled_text: theme.on_surface().scale_alpha(DIM_ALPHA),
            item_selected_text: theme.on_secondary_container(),
            item_selected_background: theme.secondary_container(),
            item_state_layer: theme.on_surface(),
            separator: theme.outline_variant(),
            scrollbar: theme.on_surface_variant(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Labels<'a> {
    pub search: text::Fragment<'a>,
    pub clear: text::Fragment<'a>,
    pub clear_search: text::Fragment<'a>,
    pub actions: ActionLabels<'a>,
}

impl Default for Labels<'_> {
    fn default() -> Self {
        Self {
            search: "Search".into(),
            clear: "Clear selection".into(),
            clear_search: "Clear search".into(),
            actions: ActionLabels::default(),
        }
    }
}

pub struct Select<'a, T, Message>
where
    T: Clone + PartialEq,
    Message: Clone,
{
    items: Rc<[Item<T>]>,
    selected: Option<T>,
    on_selected: Box<dyn Fn(Option<T>) -> Message + 'a>,
    placeholder: String,
    direction: OpenDirection,
    clearable: bool,
    close_on_select: bool,
    searchable: bool,
    motion_spring: Option<Spring>,
    style: Style,
    menu_style: menu::Style,
    tooltip_style: tooltip::Style,
    width: Length,
    padding: Padding,
    text_size: Pixels,
    item_text_size: Pixels,
    item_height: f32,
    max_menu_height: f32,
    font: Option<Font>,
    labels: Labels<'a>,
    tooltips_enabled: bool,
}

/// A single entry of a [`Select`]: either a selectable option or a separator.
#[derive(Debug, Clone)]
pub enum Item<T> {
    /// A selectable option with its value, label and optional icon.
    Option {
        value: T,
        label: String,
        icon: Option<Icon<'static>>,
    },
    /// A horizontal rule separating groups of options.
    ///
    /// Separators are neither selectable nor searchable, and they are hidden while a search
    /// query is active.
    Separator,
}

impl<T: ToString> Item<T> {
    /// Creates an [`Item::Option`] whose label is derived from the value.
    pub fn new(value: T) -> Self {
        let label = value.to_string();

        Self::Option {
            value,
            label,
            icon: None,
        }
    }
}

impl<T> Item<T> {
    /// Creates an [`Item::Option`] with an explicit label.
    pub fn labelled(value: T, label: impl Into<String>) -> Self {
        Self::Option {
            value,
            label: label.into(),
            icon: None,
        }
    }

    /// Creates a horizontal rule separating groups of options.
    pub fn separator() -> Self {
        Self::Separator
    }

    /// Sets the icon shown next to the label.
    ///
    /// Accepts either an SVG handle or a text (font) icon.
    #[must_use]
    pub fn icon(self, icon: impl Into<Icon<'static>>) -> Self {
        self.icon_maybe(Some(icon.into()))
    }

    /// Sets the icon shown next to the label, if `Some`.
    #[must_use]
    pub fn icon_maybe(self, icon: Option<Icon<'static>>) -> Self {
        match self {
            Self::Option { value, label, .. } => Self::Option { value, label, icon },
            Self::Separator => Self::Separator,
        }
    }

    /// Returns whether this entry is a separator.
    pub fn is_separator(&self) -> bool {
        matches!(self, Self::Separator)
    }

    /// Returns the value of the option, or `None` for a separator.
    pub fn value(&self) -> Option<&T> {
        match self {
            Self::Option { value, .. } => Some(value),
            Self::Separator => None,
        }
    }

    /// Returns the label of the option, or `None` for a separator.
    pub fn label(&self) -> Option<&str> {
        match self {
            Self::Option { label, .. } => Some(label),
            Self::Separator => None,
        }
    }

    fn icon_ref(&self) -> Option<&Icon<'static>> {
        match self {
            Self::Option { icon, .. } => icon.as_ref(),
            Self::Separator => None,
        }
    }
}

impl<'a, T, Message> Select<'a, T, Message>
where
    T: Clone + PartialEq,
    Message: Clone,
{
    pub fn new(
        theme: &(impl ColorScheme + ?Sized),
        items: impl IntoIterator<Item = Item<T>>,
        selected: Option<T>,
        on_selected: impl Fn(Option<T>) -> Message + 'a,
    ) -> Self {
        let items: Vec<Item<T>> = items.into_iter().collect();

        let searchable = items.len() >= constants::DEFAULT_SEARCH_THRESHOLD;

        Self {
            items: items.into(),
            selected,
            on_selected: Box::new(on_selected),
            placeholder: String::new(),
            direction: OpenDirection::Down,
            clearable: false,
            close_on_select: true,
            searchable,
            motion_spring: None,
            style: Style::new(theme),
            menu_style: menu::Style::new(theme, false),
            tooltip_style: tooltip::Style::new(theme),
            width: Length::Fill,
            padding: Padding {
                top: 0.0,
                right: 0.0,
                bottom: 0.0,
                left: constants::FIELD_TEXT_PADDING,
            },
            text_size: constants::FIELD_TEXT_SIZE,
            item_text_size: constants::ITEM_TEXT_SIZE,
            item_height: constants::ITEM_HEIGHT,
            max_menu_height: constants::DEFAULT_MAX_MENU_HEIGHT.0,
            font: None,
            labels: Labels::default(),
            tooltips_enabled: false,
        }
    }

    #[must_use]
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    #[must_use]
    pub fn labels(mut self, labels: Labels<'a>) -> Self {
        self.labels = labels;
        self
    }

    #[must_use]
    pub fn search_label(mut self, label: impl IntoFragment<'a>) -> Self {
        self.labels.search = label.into_fragment();
        self
    }

    #[must_use]
    pub fn clear_label(mut self, label: impl IntoFragment<'a>) -> Self {
        self.labels.clear = label.into_fragment();
        self
    }

    #[must_use]
    pub fn clear_search_label(mut self, label: impl IntoFragment<'a>) -> Self {
        self.labels.clear_search = label.into_fragment();
        self
    }

    #[must_use]
    pub fn action_label(mut self, action: Action, label: impl IntoFragment<'a>) -> Self {
        self.labels.actions.set(action, label);
        self
    }

    #[must_use]
    pub fn open_direction(mut self, direction: OpenDirection) -> Self {
        self.direction = direction;
        self
    }

    #[must_use]
    pub fn clearable(mut self, clearable: bool) -> Self {
        self.clearable = clearable;
        self
    }

    #[must_use]
    pub fn close_on_select(mut self, close: bool) -> Self {
        self.close_on_select = close;
        self
    }

    #[must_use]
    pub fn searchable(mut self, searchable: bool) -> Self {
        self.searchable = searchable;
        self
    }

    #[must_use]
    pub fn motion_scheme(mut self, scheme: motion::Scheme) -> Self {
        self.motion_spring = Some(default_spatial(scheme));
        self
    }

    #[must_use]
    pub fn motion_spring(mut self, spring: Spring) -> Self {
        self.motion_spring = Some(spring);
        self
    }

    #[must_use]
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    #[must_use]
    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.padding = padding.into();
        self
    }

    #[must_use]
    pub fn text_size(mut self, size: impl Into<Pixels>) -> Self {
        self.text_size = size.into();
        self
    }

    #[must_use]
    pub fn item_text_size(mut self, size: impl Into<Pixels>) -> Self {
        self.item_text_size = size.into();
        self
    }

    #[must_use]
    pub fn item_height(mut self, height: impl Into<Pixels>) -> Self {
        self.item_height = height.into().0;
        self
    }

    #[must_use]
    pub fn max_menu_height(mut self, height: impl Into<Pixels>) -> Self {
        self.max_menu_height = height.into().0;
        self
    }

    #[must_use]
    pub fn font(mut self, font: Font) -> Self {
        self.font = Some(font);
        self
    }

    #[must_use]
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    #[must_use]
    pub fn tooltip_style(mut self, style: tooltip::Style) -> Self {
        self.tooltip_style = style;
        self
    }

    #[must_use]
    pub fn tooltips_enabled(mut self, enabled: bool) -> Self {
        self.tooltips_enabled = enabled;
        self
    }

    fn selected_index(&self) -> Option<usize> {
        let selected = self.selected.as_ref()?;

        self.items
            .iter()
            .position(|item| item.value() == Some(selected))
    }

    fn selected_item(&self) -> Option<&Item<T>> {
        self.items.get(self.selected_index()?)
    }

    /// Whether any option carries an icon.
    ///
    /// The field reserves a leading slot whenever an icon can be selected, so its label stays
    /// aligned with the menu rows instead of jumping between picks.
    fn has_icons(&self) -> bool {
        self.items.iter().any(|item| item.icon_ref().is_some())
    }

    /// Leading slot taken by the selected option's icon inside the field.
    fn icon_slot(&self) -> f32 {
        if self.has_icons() {
            constants::ITEM_ICON_SLOT + constants::ITEM_ICON_SPACING
        } else {
            0.0
        }
    }

    fn arrow_center(bounds: Rectangle) -> Point {
        Point::new(
            bounds.x + bounds.width - constants::ICON_PADDING - constants::ICON_SIZE.0 / 2.0,
            bounds.center_y(),
        )
    }

    fn clear_center(bounds: Rectangle) -> Point {
        let arrow = Self::arrow_center(bounds);

        Point::new(
            arrow.x
                - constants::ICON_SIZE.0 / 2.0
                - constants::CLEAR_GAP
                - constants::CLEAR_ICON_SIZE.0 / 2.0,
            arrow.y,
        )
    }

    fn clear_bounds(bounds: Rectangle) -> Rectangle {
        let center = Self::clear_center(bounds);
        let size = constants::CLEAR_TOUCH_TARGET;

        Rectangle::new(
            Point::new(center.x - size / 2.0, center.y - size / 2.0),
            Size::new(size, size),
        )
    }

    fn show_clear(&self) -> bool {
        self.clearable && self.selected.is_some()
    }

    fn reserved_width(&self) -> f32 {
        let clear = if self.show_clear() {
            constants::CLEAR_ICON_SIZE.0 + constants::CLEAR_GAP
        } else {
            0.0
        };

        constants::ICON_PADDING + constants::ICON_SIZE.0 + clear
    }

    fn width_fingerprint(&self, font: Font) -> u64 {
        let mut hasher = DefaultHasher::new();

        self.text_size.0.to_bits().hash(&mut hasher);
        font.hash(&mut hasher);
        self.placeholder.hash(&mut hasher);
        self.padding.left.to_bits().hash(&mut hasher);
        self.padding.right.to_bits().hash(&mut hasher);
        self.icon_slot().to_bits().hash(&mut hasher);
        self.reserved_width().to_bits().hash(&mut hasher);

        for label in self.items.iter().filter_map(Item::label) {
            label.hash(&mut hasher);
        }

        hasher.finish()
    }
}

fn text_width<Renderer>(
    renderer: &Renderer,
    content: &str,
    size: Pixels,
    font: Renderer::Font,
) -> f32
where
    Renderer: text::Renderer,
{
    internal_text::width(
        renderer,
        content,
        size,
        text::LineHeight::default(),
        font,
        Vertical::Center,
    )
}

const FIT_CACHE_CAPACITY: usize = 64;

#[derive(Debug, Default)]
struct FitCache {
    entries: RefCell<Vec<FitEntry>>,
}

#[derive(Debug)]
struct FitEntry {
    content: String,
    fitted: String,
    max_width: f32,
    size: Pixels,
    font: Font,
}

impl FitCache {
    fn fit<Renderer>(
        &self,
        renderer: &Renderer,
        content: &str,
        max_width: f32,
        size: Pixels,
        font: Font,
    ) -> String
    where
        Renderer: text::Renderer<Font = Font>,
    {
        let mut entries = self.entries.borrow_mut();

        let cached = entries.iter().find(|entry| {
            entry.max_width == max_width
                && entry.size == size
                && entry.font == font
                && entry.content == content
        });

        if let Some(entry) = cached {
            return entry.fitted.clone();
        }

        let line_height = text::LineHeight::default();
        let (fitted, _) = elide::elide(content, max_width, |candidate| {
            internal_text::width(
                renderer,
                candidate,
                size,
                line_height,
                font,
                Vertical::Center,
            )
        });

        if entries.len() >= FIT_CACHE_CAPACITY {
            entries.remove(0);
        }

        entries.push(FitEntry {
            content: content.to_owned(),
            fitted: fitted.clone(),
            max_width,
            size,
            font,
        });

        fitted
    }
}

fn order_options<T>(query: &str, items: &[Item<T>]) -> Vec<(usize, bool)> {
    let mut scored: Vec<(usize, i32, bool)> = items
        .iter()
        .enumerate()
        .filter_map(|(index, item)| {
            let label = item.label()?;

            Some(match fuzzy_score(query, label) {
                Some(score) => (index, score, true),
                None => (index, i32::MIN, false),
            })
        })
        .collect();

    scored.sort_by_key(|&(_, score, _)| std::cmp::Reverse(score));

    scored
        .into_iter()
        .map(|(index, _, matched)| (index, matched))
        .collect()
}

impl<'a, T, Message, Theme, Renderer> Widget<Message, Theme, Renderer> for Select<'a, T, Message>
where
    T: Clone + PartialEq + 'a,
    Message: Clone + 'a,
    Theme: 'a,
    Renderer: text::Renderer<Font = iced::Font> + svg::Renderer + 'a,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn size(&self) -> Size<Length> {
        Size::new(self.width, Length::Fixed(constants::FIELD_HEIGHT.0))
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &Renderer, limits: &Limits) -> layout::Node {
        let font = self.font.unwrap_or_else(|| renderer.default_font());

        let fingerprint = self.width_fingerprint(font);
        let state = tree.state.downcast_mut::<State>();

        let content_width = match state.intrinsic_width {
            Some((cached, width)) if cached == fingerprint => width,
            _ => {
                let labels_width = self
                    .items
                    .iter()
                    .filter_map(Item::label)
                    .map(|label| text_width(renderer, label, self.text_size, font))
                    .fold(0.0_f32, f32::max);

                let placeholder_width = if self.placeholder.is_empty() {
                    0.0
                } else {
                    text_width(renderer, &self.placeholder, self.text_size, font)
                };

                let width = labels_width.max(placeholder_width);
                state.intrinsic_width = Some((fingerprint, width));

                width
            }
        };

        let intrinsic = Size::new(
            content_width
                + self.padding.left
                + self.padding.right
                + self.icon_slot()
                + self.reserved_width(),
            constants::FIELD_HEIGHT.0,
        );

        let size = limits.width(self.width).resolve(
            self.width,
            Length::Fixed(constants::FIELD_HEIGHT.0),
            intrinsic,
        );

        layout::Node::new(size)
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &iced::advanced::renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State>();
        let bounds = layout.bounds();

        let progress = state.progress.value().clamp(0.0, 1.0);

        // Animate the outline along with the menu instead of snapping it on open/close.
        let rest_outline = if state.hovered {
            self.style.field_outline_hovered
        } else {
            self.style.field_outline
        };
        let outline_color = mix_colors(rest_outline, self.style.field_outline_open, progress);
        let outline_width = constants::FIELD_OUTLINE_WIDTH
            + (constants::FIELD_OUTLINE_WIDTH_OPEN - constants::FIELD_OUTLINE_WIDTH) * progress;

        renderer.fill_quad(
            Quad {
                bounds,
                border: Border {
                    color: outline_color,
                    width: outline_width,
                    radius: constants::FIELD_RADIUS.into(),
                },
                ..Quad::default()
            },
            self.style.field_background,
        );

        let font = self.font.unwrap_or_else(|| renderer.default_font());
        let line_height = text::LineHeight::default();

        let selected = self.selected_item();
        let icon_slot = self.icon_slot();
        let text_left = bounds.x + self.padding.left + icon_slot;
        let text_width = (bounds.width
            - self.padding.left
            - self.padding.right
            - icon_slot
            - self.reserved_width())
        .max(0.0);

        if icon_slot > 0.0
            && let Some(icon) = selected.and_then(Item::icon_ref)
        {
            icon::draw_hybrid_icon(
                renderer,
                icon,
                Point::new(
                    bounds.x + self.padding.left + constants::ITEM_ICON_SLOT / 2.0,
                    bounds.center_y(),
                ),
                constants::ITEM_ICON_SIZE,
                self.style.field_icon,
                bounds,
            );
        }

        let label = selected
            .and_then(Item::label)
            .unwrap_or(self.placeholder.as_str());

        if !label.is_empty() && text_width > 0.0 {
            let clip = Rectangle::new(
                Point::new(text_left, bounds.y),
                Size::new(text_width, bounds.height),
            );

            renderer.fill_text(
                Text {
                    content: state
                        .fit
                        .fit(renderer, label, text_width, self.text_size, font),
                    bounds: Size::new(text_width, line_height.to_absolute(self.text_size).0),
                    size: self.text_size,
                    line_height,
                    font,
                    align_x: text::Alignment::Left,
                    align_y: Vertical::Center,
                    shaping: text::Shaping::Advanced,
                    wrapping: text::Wrapping::None,
                },
                Point::new(text_left, bounds.center_y()),
                if selected.is_some() {
                    self.style.field_text
                } else {
                    self.style.field_placeholder
                },
                clip,
            );
        }

        let chevron_color = mix_colors(
            self.style.field_icon,
            self.style.field_outline_open,
            progress,
        );

        icon::draw_svg_icon(
            renderer,
            &common_icons::ARROW_RIGHT,
            Self::arrow_center(bounds),
            constants::ICON_SIZE,
            // Rotate the chevron from pointing down (closed) to pointing up (open).
            Radians(std::f32::consts::FRAC_PI_2 - progress * std::f32::consts::PI),
            chevron_color,
            1.0,
            bounds,
        );

        if self.show_clear() {
            icon::draw_svg_icon(
                renderer,
                &common_icons::CLOSE,
                Self::clear_center(bounds),
                constants::CLEAR_ICON_SIZE,
                Radians(0.0),
                if state.clear_hovered {
                    self.style.field_text
                } else {
                    self.style.field_icon
                },
                1.0,
                bounds,
            );
        }
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<State>();
        let bounds = layout.bounds();

        let is_hovered = cursor.is_over(bounds);
        let is_clear_hovered = self.show_clear()
            && cursor
                .position()
                .is_some_and(|position| Self::clear_bounds(bounds).contains(position));

        if state.hovered != is_hovered || state.clear_hovered != is_clear_hovered {
            state.hovered = is_hovered;
            state.clear_hovered = is_clear_hovered;
            shell.request_redraw();
        }

        // Both field tooltips (the clear button and the truncated value) share the field's hover
        // delay. They are suppressed while the menu is open. The cross is a target of its own, so
        // hovering it always waits for the delay instead of reusing the timer of the field body.
        let tooltip_target = if self.tooltips_enabled && is_clear_hovered {
            Self::clear_bounds(bounds)
        } else {
            bounds
        };

        state.field_tooltip.update(
            event,
            cursor,
            Some(tooltip_target),
            !state.is_open,
            tooltip::DEFAULT_DELAY,
            shell,
        );

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                if is_hovered && is_clear_hovered {
                    // Clearing wins over toggling the menu: the clear button is always visible.
                    if state.is_open {
                        state.close();
                    }

                    shell.publish((self.on_selected)(None));
                    shell.request_redraw();
                    shell.capture_event();
                } else if state.is_open {
                    // The overlay did not capture this press, so it happened either outside of
                    // the menu or on the field itself. Either way, close the menu.
                    state.close();
                    shell.request_redraw();
                    shell.capture_event();
                } else if is_hovered {
                    state.open(self.searchable);
                    shell.request_redraw();
                    shell.capture_event();
                }
            }
            _ => {}
        }

        state.progress.set_model_if_some(self.motion_spring);
        state.animate(shell);
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        interaction::pointer_when(true, cursor, layout.bounds(), mouse::Interaction::Pointer)
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        _viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        let state = tree.state.downcast_mut::<State>();

        let mut anchor_bounds = layout.bounds();
        anchor_bounds.x += translation.x;
        anchor_bounds.y += translation.y;

        let menu_open = state.is_open || !state.progress.is_at_rest();

        let font = self.font.unwrap_or_else(|| renderer.default_font());

        let tooltip = if menu_open || !state.field_tooltip.is_open() {
            None
        } else if self.tooltips_enabled && state.clear_hovered {
            // The clear button is icon-only, so spell out what it does.
            let center = Self::clear_center(anchor_bounds);
            let size = constants::CLEAR_ICON_SIZE.0;
            let anchor = Rectangle::new(
                Point::new(center.x - size / 2.0, center.y - size / 2.0),
                Size::new(size, size),
            );

            Some(tooltip::label_overlay(
                anchor,
                self.labels.clear.clone(),
                Some(font),
                self.tooltip_style,
                tooltip::Position::Above,
                constants::GAP,
                tooltip::DEFAULT_MAX_WIDTH,
                None,
            ))
        } else if state.hovered {
            // When the field has to ellipsize its value, show the full text on hover.
            let label = self
                .selected_item()
                .and_then(Item::label)
                .unwrap_or(self.placeholder.as_str());

            let available = (anchor_bounds.width
                - self.padding.left
                - self.padding.right
                - self.icon_slot()
                - self.reserved_width())
            .max(0.0);

            if !label.is_empty()
                && available > 0.0
                && text_width(renderer, label, self.text_size, font) > available
            {
                Some(tooltip::label_overlay(
                    anchor_bounds,
                    text::Fragment::Owned(label.to_owned()),
                    Some(font),
                    self.tooltip_style,
                    tooltip::Position::Above,
                    constants::GAP,
                    tooltip::DEFAULT_MAX_WIDTH,
                    None,
                ))
            } else {
                None
            }
        } else {
            None
        };

        let menu = if menu_open {
            // Freeze the highlighted option while closing so that picking an item does not make the
            // highlight (or the surface position) jump to another row during the fade-out.
            if state.is_open {
                state.shown_selected = self.selected_index();
            }

            let selected = state.shown_selected;

            let items = Rc::clone(&self.items);
            let on_selected = &self.on_selected;
            let on_select: Box<dyn Fn(Option<usize>) -> Message + 'b> = Box::new(move |index| {
                let value = index.and_then(|index| items[index].value().cloned());

                (on_selected)(value)
            });

            let menu = Overlay {
                state,
                items: Rc::clone(&self.items),
                selected,
                on_select,
                style: self.style,
                menu_style: self.menu_style,
                direction: self.direction,
                clearable: self.clearable,
                close_on_select: self.close_on_select,
                searchable: self.searchable,
                anchor_bounds,
                item_height: self.item_height,
                item_text_size: self.item_text_size,
                max_menu_height: self.max_menu_height,
                font: self.font,
                labels: self.labels.clone(),
                tooltips_enabled: self.tooltips_enabled,
            };

            // The menu's tooltip is a sibling of the surface and not one of its nested overlays:
            // the runtime reports no mouse interaction for an overlay with a nested child, which
            // would let the widgets below the popup receive the pointer and open their own
            // tooltips through the menu.
            let menu_tooltip = menu.hovered_tooltip(renderer).map(|menu_tooltip| {
                tooltip::label_overlay(
                    menu_tooltip.anchor,
                    text::Fragment::Owned(menu_tooltip.label),
                    Some(font),
                    self.tooltip_style,
                    menu_tooltip.position,
                    constants::GAP,
                    menu_tooltip.max_width,
                    None,
                )
            });

            let menu = overlay::Element::new(Box::new(menu));

            Some(match menu_tooltip {
                Some(menu_tooltip) => {
                    overlay::Group::with_children(vec![menu, menu_tooltip]).overlay()
                }
                None => menu,
            })
        } else {
            None
        };

        menu.or(tooltip)
    }
}

crate::macros::impl_element_from!(
    ['a, T, Message] Select<'a, T, Message> => Element<'a, Message>
    where T: Clone + PartialEq + 'a, Message: Clone + 'a,
);

#[derive(Debug)]
struct State {
    is_open: bool,
    hovered: bool,
    clear_hovered: bool,
    scroll_offset: f32,
    scroll_initialized: bool,
    hovered_item: Option<usize>,
    /// Whether the pointer is over the scrollbar rail.
    scrollbar_hovered: bool,
    /// Whether the pointer is anywhere over the menu surface.
    menu_hovered: bool,
    /// Active scrollbar drag, storing the grab offset inside the thumb.
    scrollbar_drag: Option<f32>,
    /// Open/close progress driven by a spring: `0.0` is fully closed, `1.0` fully open.
    progress: SpringValue<f32>,
    /// Options highlighted by the menu. Kept while closing so the highlight does not jump when
    /// the selection changes right after a pick.
    shown_selected: Option<usize>,
    /// Current search query typed into the menu.
    query: Value,
    /// Caret and selection inside the search query.
    cursor: Cursor,
    /// Undo/redo history of the search query, shared with [`TextInput`](crate::widget::TextInput).
    history: History,
    /// Whether the search field currently has keyboard focus.
    search_focused: bool,
    /// Whether the pointer hovers the clear button inside the search field.
    search_clear_hovered: bool,
    /// Modifier keys held down, tracked to mirror the text input shortcuts.
    keyboard_modifiers: keyboard::Modifiers,
    /// Last click inside the search field, used for double and triple click selection.
    last_click: Option<mouse::Click>,
    /// Active mouse selection drag inside the search field.
    dragging: Option<SearchDrag>,
    /// Surface geometry captured on the first layout after opening. Reused while the menu
    /// animates so that model updates cannot move it.
    geometry: Option<MenuGeometry>,
    /// Open right-click context menu for the search field.
    context_menu: Option<ContextMenu>,
    /// Caret blink bookkeeping for the search field.
    blink: Blink,
    /// Hover delay of the tooltips shown for the field itself (clear button, truncated value).
    field_tooltip: tooltip::Hover,
    /// Hover delay of the tooltips shown inside the open menu (clear button, truncated option).
    menu_tooltip: tooltip::Hover,
    fit: FitCache,
    intrinsic_width: Option<(u64, f32)>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum SearchDrag {
    Select,
    SelectWords { anchor: usize },
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct MenuGeometry {
    viewport: Size,
    rect: Rectangle,
}

impl Default for State {
    fn default() -> Self {
        let now = Instant::now();
        let mut progress = SpringValue::new(0.0, default_spatial(motion::Scheme::default()), now);
        progress.reset(0.0, now);

        Self {
            is_open: false,
            hovered: false,
            clear_hovered: false,
            scroll_offset: 0.0,
            scroll_initialized: false,
            hovered_item: None,
            scrollbar_hovered: false,
            menu_hovered: false,
            scrollbar_drag: None,
            progress,
            shown_selected: None,
            query: Value::new(""),
            cursor: Cursor::default(),
            history: History::default(),
            search_focused: false,
            search_clear_hovered: false,
            keyboard_modifiers: keyboard::Modifiers::default(),
            last_click: None,
            dragging: None,
            geometry: None,
            context_menu: None,
            blink: Blink::new(now),
            field_tooltip: tooltip::Hover::default(),
            menu_tooltip: tooltip::Hover::default(),
            fit: FitCache::default(),
            intrinsic_width: None,
        }
    }
}

impl State {
    fn open(&mut self, searchable: bool) {
        self.is_open = true;
        self.scroll_initialized = false;
        self.scroll_offset = 0.0;
        self.hovered_item = None;
        self.scrollbar_hovered = false;
        self.menu_hovered = false;
        self.scrollbar_drag = None;
        self.shown_selected = None;
        self.menu_tooltip = tooltip::Hover::default();
        self.query = Value::new("");
        self.cursor = Cursor::default();
        self.history.reset("");
        self.search_focused = searchable;
        self.search_clear_hovered = false;
        self.last_click = None;
        self.dragging = None;
        self.geometry = None;
        self.context_menu = None;
        let now = Instant::now();
        self.blink = Blink::new(now);
        self.progress.set_target(1.0, now);
    }

    fn close(&mut self) {
        self.is_open = false;
        self.hovered_item = None;
        self.scrollbar_hovered = false;
        self.menu_hovered = false;
        self.scrollbar_drag = None;
        self.search_focused = false;
        self.search_clear_hovered = false;
        self.dragging = None;
        self.context_menu = None;
        self.menu_tooltip = tooltip::Hover::default();
        self.history.break_group();
        // Start the close animation immediately. The overlay only exists while the spring is not
        // at rest, and it is created *before* `update` runs, so waiting for the next animation
        // tick would drop the menu for a frame and make it flicker.
        self.progress.set_target(0.0, Instant::now());
    }

    fn animate<Message>(&mut self, shell: &mut Shell<'_, Message>) {
        let now = Instant::now();
        let target = if self.is_open { 1.0 } else { 0.0 };

        self.progress.set_target(target, now);

        if !self.progress.is_at_rest() {
            shell.request_redraw();
        }

        self.progress.step(now);
    }

    /// Clears the search query and resets its caret, selection and history.
    fn reset_query(&mut self, now: Instant) {
        self.query = Value::new("");
        self.cursor = Cursor::default();
        self.history.reset("");
        self.last_click = None;
        self.dragging = None;
        self.hovered_item = None;
        self.search_clear_hovered = false;

        self.restart_blink(now);
    }

    /// Restarts the caret blink so it is visible right after an interaction, like a text input.
    fn restart_blink(&mut self, now: Instant) {
        self.blink.restart(now);
    }

    /// Applies an edit to the query through the same pipeline as the text input.
    ///
    /// Returns `true` when the query changed.
    fn edit_query(
        &mut self,
        now: Instant,
        kind: EditKind,
        change: impl FnOnce(&mut Editor<'_>),
    ) -> bool {
        let before = self.query.to_string();

        let contents = edit_value(
            &mut self.query,
            &mut self.cursor,
            &mut self.history,
            kind,
            change,
        );

        if contents == before {
            return false;
        }

        // Keep the scroll position where the user left it: reordering the options must not yank
        // the list back to the top while typing. The row under the pointer no longer holds the
        // same option, so drop the stale hover.
        self.hovered_item = None;
        self.restart_blink(now);

        true
    }
}

struct Overlay<'a, T, Message> {
    state: &'a mut State,
    items: Rc<[Item<T>]>,
    selected: Option<usize>,
    on_select: Box<dyn Fn(Option<usize>) -> Message + 'a>,
    style: Style,
    menu_style: menu::Style,
    direction: OpenDirection,
    clearable: bool,
    close_on_select: bool,
    searchable: bool,
    anchor_bounds: Rectangle,
    item_height: f32,
    item_text_size: Pixels,
    max_menu_height: f32,
    font: Option<Font>,
    labels: Labels<'a>,
    tooltips_enabled: bool,
}

/// A tooltip the open menu wants to show, described so `Select::overlay` can draw it outside the
/// menu surface.
struct MenuTooltip {
    anchor: Rectangle,
    label: String,
    position: tooltip::Position,
    max_width: f32,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Row {
    Separator,
    Option { index: usize, matched: bool },
}

impl<T, Message> Overlay<'_, T, Message> {
    fn is_selected(&self, index: usize) -> bool {
        self.selected == Some(index)
    }

    /// Height of the search field (including the gap below it), or `0` when search is disabled.
    fn header_height(&self) -> f32 {
        if self.searchable {
            constants::SEARCH_HEIGHT + constants::ITEM_SPACING
        } else {
            0.0
        }
    }

    /// Height of a single menu row.
    fn row_height(&self, row: Row) -> f32 {
        match row {
            Row::Separator => constants::SEPARATOR_HEIGHT.0,
            Row::Option { .. } => self.item_height,
        }
    }

    /// Total height of the given stacked rows, including the spacing between them.
    fn rows_height(&self, rows: &[Row]) -> f32 {
        if rows.is_empty() {
            0.0
        } else {
            rows.iter().map(|&row| self.row_height(row)).sum::<f32>()
                + (rows.len() as f32 - 1.0) * constants::ITEM_SPACING
        }
    }

    /// Vertical offset of `row` from the top of the list.
    fn row_top(&self, rows: &[Row], row: usize) -> f32 {
        rows[..row]
            .iter()
            .map(|&row| self.row_height(row) + constants::ITEM_SPACING)
            .sum()
    }

    /// The rows to display, in visual order.
    ///
    /// Without a query (or when search is disabled) the original order is kept, separators
    /// included and every option matches. Otherwise the options are sorted by fuzzy match score,
    /// best match first, options that do not match are pushed to the bottom in their original
    /// order and flagged as disabled, and separators are hidden because grouping no longer makes
    /// sense once the rows are reordered.
    fn rows(&self) -> Vec<Row> {
        let query = if self.searchable {
            self.state.query.to_string()
        } else {
            String::new()
        };

        if query.is_empty() {
            self.items
                .iter()
                .enumerate()
                .map(|(index, item)| match item {
                    Item::Separator => Row::Separator,
                    Item::Option { .. } => Row::Option {
                        index,
                        matched: true,
                    },
                })
                .collect()
        } else {
            order_options(&query, &self.items)
                .into_iter()
                .map(|(index, matched)| Row::Option { index, matched })
                .collect()
        }
    }

    /// The row the selected option occupies in the current list, if visible.
    fn selected_row(&self) -> Option<usize> {
        let index = self.selected?;

        self.rows()
            .iter()
            .position(|row| matches!(row, Row::Option { index: item, .. } if *item == index))
    }

    fn natural_height(&self) -> f32 {
        2.0 * constants::MENU_PADDING + self.header_height() + self.rows_height(&self.rows())
    }

    fn max_offset(&self, surface_height: f32) -> f32 {
        let viewport =
            (surface_height - 2.0 * constants::MENU_PADDING - self.header_height()).max(0.0);

        (self.rows_height(&self.rows()) - viewport).max(0.0)
    }

    /// Scroll offset used for drawing and hit-testing.
    ///
    /// The stored offset is clamped to the current content so filtering the options down to a
    /// shorter list cannot leave the viewport scrolled past the end.
    fn scroll_offset(&self, surface_height: f32) -> f32 {
        self.state
            .scroll_offset
            .clamp(0.0, self.max_offset(surface_height))
    }

    /// Scrollable list area inside the menu surface.
    fn list_bounds(&self, bounds: Rectangle) -> Rectangle {
        let padding = constants::MENU_PADDING;
        let header = self.header_height();
        let content_height = (bounds.height - 2.0 * padding).max(0.0);

        Rectangle::new(
            Point::new(bounds.x + padding, bounds.y + padding + header),
            Size::new(
                (bounds.width - 2.0 * padding).max(0.0),
                (content_height - header).max(0.0),
            ),
        )
    }

    /// Scrollbar geometry when the list overflows, if any.
    fn scrollbar(&self, bounds: Rectangle) -> Option<scrollbar::Rail> {
        let list = self.list_bounds(bounds);

        scrollbar::Rail::vertical(
            list,
            self.rows_height(&self.rows()),
            self.scroll_offset(bounds.height),
        )
    }

    /// Maps a thumb top position to the matching scroll offset.
    fn scroll_to_thumb(&mut self, rail: &scrollbar::Rail, thumb_top: f32) {
        self.state.scroll_offset = rail.offset_at(thumb_top);
    }

    /// Bounds of the search field inside the menu surface.
    fn search_bounds(&self, bounds: Rectangle) -> Rectangle {
        let padding = constants::MENU_PADDING;

        Rectangle::new(
            Point::new(bounds.x + padding, bounds.y + padding),
            Size::new(
                (bounds.width - 2.0 * padding).max(0.0),
                constants::SEARCH_HEIGHT,
            ),
        )
    }

    /// Whether the search field shows a clear button (search is enabled and the query is not
    /// empty).
    fn search_clear_visible(&self) -> bool {
        self.searchable && !self.state.query.is_empty()
    }

    /// Horizontal space reserved at the trailing edge of the search field for the clear button.
    fn search_clear_reserved(&self) -> f32 {
        if self.search_clear_visible() {
            constants::CLEAR_ICON_SIZE.0 + constants::CLEAR_GAP
        } else {
            0.0
        }
    }

    /// Center of the clear button inside the search field.
    fn search_clear_center(&self, bounds: Rectangle) -> Point {
        let search = self.search_bounds(bounds);

        Point::new(
            search.x + search.width - constants::ITEM_PADDING - constants::CLEAR_ICON_SIZE.0 / 2.0,
            search.center_y(),
        )
    }

    /// Hit area of the clear button inside the search field, when it is visible.
    fn search_clear_bounds(&self, bounds: Rectangle) -> Option<Rectangle> {
        if !self.search_clear_visible() {
            return None;
        }

        let center = self.search_clear_center(bounds);
        let size = constants::CLEAR_TOUCH_TARGET;

        Some(Rectangle::new(
            Point::new(center.x - size / 2.0, center.y - size / 2.0),
            Size::new(size, size),
        ))
    }

    /// Bounds of the option row at `index` inside the menu surface, factoring in the scroll
    /// offset.
    fn row_bounds(&self, index: usize, bounds: Rectangle) -> Option<Rectangle> {
        let rows = self.rows();
        let row = rows
            .iter()
            .position(|row| matches!(row, Row::Option { index: item, .. } if *item == index))?;
        let list = self.list_bounds(bounds);
        let offset = self.scroll_offset(bounds.height);
        let top: f32 = rows[..row]
            .iter()
            .map(|&row| self.row_height(row) + constants::ITEM_SPACING)
            .sum();
        let item_width = if self.scrollbar(bounds).is_some() {
            (list.width - constants::SCROLLBAR_GUTTER).max(0.0)
        } else {
            list.width
        };

        Some(Rectangle::new(
            Point::new(list.x, list.y - offset + top),
            Size::new(item_width, self.row_height(rows[row])),
        ))
    }

    /// Width available to an option's label inside the menu surface.
    fn row_text_width(&self, bounds: Rectangle) -> f32 {
        let list = self.list_bounds(bounds);
        let item_width = if self.scrollbar(bounds).is_some() {
            (list.width - constants::SCROLLBAR_GUTTER).max(0.0)
        } else {
            list.width
        };
        let text_x_offset =
            constants::ITEM_PADDING + constants::ITEM_ICON_SLOT + constants::ITEM_ICON_SPACING;

        (item_width - constants::ITEM_PADDING - text_x_offset).max(0.0)
    }

    /// The menu element under the pointer that can show a tooltip.
    ///
    /// The hover delay is tracked per element, so this is the exact row or button rather than the
    /// whole surface: otherwise the first tooltip would arm the timer for every other element and
    /// they would appear without a delay.
    fn tooltip_target<Renderer>(&self, renderer: &Renderer, bounds: Rectangle) -> Option<Rectangle>
    where
        Renderer: text::Renderer<Font = Font>,
    {
        if self.state.search_clear_hovered && self.tooltips_enabled {
            return self.search_clear_bounds(bounds);
        }

        let index = self.state.hovered_item?;
        let label = self.items.get(index)?.label()?;

        self.label_is_truncated(renderer, label, bounds)
            .then(|| self.row_bounds(index, bounds))
            .flatten()
    }

    /// Whether an option label does not fit in its row.
    fn label_is_truncated<Renderer>(
        &self,
        renderer: &Renderer,
        label: &str,
        bounds: Rectangle,
    ) -> bool
    where
        Renderer: text::Renderer<Font = Font>,
    {
        let font = self.font.unwrap_or_else(|| renderer.default_font());

        !label.is_empty()
            && text_width(renderer, label, self.item_text_size, font) > self.row_text_width(bounds)
    }

    /// Anchor and label of the tooltip for the hovered menu entry, when one should be shown.
    ///
    /// Used by the `Select` overlay to place the tooltip next to the menu surface instead of
    /// nesting it inside, which would let the pointer reach the widgets behind the popup.
    fn hovered_tooltip<Renderer>(&self, renderer: &Renderer) -> Option<MenuTooltip>
    where
        Renderer: text::Renderer<Font = Font>,
    {
        if !self.state.is_open || !self.state.menu_tooltip.is_open() {
            return None;
        }

        let bounds = self.state.geometry?.rect;

        // The search field's clear button is icon-only, so spell out what it does.
        if self.tooltips_enabled
            && self.state.search_clear_hovered
            && let Some(clear) = self.search_clear_bounds(bounds)
        {
            return Some(MenuTooltip {
                anchor: clear,
                label: self.labels.clear_search.to_string(),
                position: tooltip::Position::Above,
                max_width: tooltip::DEFAULT_MAX_WIDTH,
            });
        }

        // Show the full label of a hovered option when it is truncated. The tooltip is placed next
        // to the menu surface, never on top of the option: covering the option would blend the full
        // label with the shortened text drawn underneath and spill over the neighbouring rows.
        let index = self.state.hovered_item?;
        let label = self.items.get(index)?.label()?;

        if !self.label_is_truncated(renderer, label, bounds) {
            return None;
        }

        let row = self.row_bounds(index, bounds)?;

        // Span the whole surface so the tooltip lands beside the menu instead of on its border,
        // while staying vertically aligned with the hovered row.
        let anchor = Rectangle::new(
            Point::new(bounds.x, row.y),
            Size::new(bounds.width, row.height),
        );

        Some(MenuTooltip {
            anchor,
            label: label.to_owned(),
            position: tooltip::Position::auto(),
            max_width: tooltip::DEFAULT_MAX_WIDTH,
        })
    }

    fn item_at(&self, position: Option<Point>, bounds: Rectangle) -> Option<usize> {
        let position = position?;

        if !bounds.contains(position) {
            return None;
        }

        let list_top = bounds.y + constants::MENU_PADDING + self.header_height();

        if position.y < list_top {
            return None;
        }

        let rows = self.rows();
        let viewport =
            (bounds.height - 2.0 * constants::MENU_PADDING - self.header_height()).max(0.0);

        // Let the reserved scrollbar gutter behave like a dead zone instead of highlighting the
        // option behind it.
        if self.rows_height(&rows) > viewport {
            let list_right = bounds.x + bounds.width - constants::MENU_PADDING;

            if position.x >= list_right - constants::SCROLLBAR_GUTTER {
                return None;
            }
        }

        let local = position.y - list_top + self.scroll_offset(bounds.height);
        let mut top = 0.0;

        for row in rows {
            let height = self.row_height(row);

            if local < top + height {
                return match row {
                    Row::Option { index, matched } => matched.then_some(index),
                    Row::Separator => None,
                };
            }

            if local < top + height + constants::ITEM_SPACING {
                return None;
            }

            top += height + constants::ITEM_SPACING;
        }

        None
    }

    /// Bounds of the editable text slot inside the search field, where the caret lives.
    fn search_text_bounds(&self, bounds: Rectangle) -> Rectangle {
        let search = self.search_bounds(bounds);
        let x = search.x
            + constants::ITEM_PADDING
            + constants::ITEM_ICON_SLOT
            + constants::ITEM_ICON_SPACING;

        Rectangle::new(
            Point::new(x, search.y),
            Size::new(
                (search.x + search.width
                    - constants::ITEM_PADDING
                    - self.search_clear_reserved()
                    - x)
                    .max(0.0),
                search.height,
            ),
        )
    }

    /// Copies the selected part of the query to the clipboard, if any.
    fn copy_query(&self, clipboard: &mut dyn Clipboard) {
        if let Some((start, end)) = self.state.cursor.selection(&self.state.query) {
            clipboard.write(
                clipboard::Kind::Standard,
                self.state.query.select(start, end).to_string(),
            );
        }
    }

    /// Handles a key press while the search field is focused.
    ///
    /// Mirrors the keyboard behavior of the text input widget: editing shortcuts (undo, redo,
    /// copy, cut, paste, select all), word-wise motion with the "jump" key and shift-extended
    /// selections.
    ///
    /// Returns `true` when the field consumed the event, so the menu must stay open.
    fn handle_search_key(
        &mut self,
        key: &keyboard::Key,
        physical_key: keyboard::key::Physical,
        text: Option<&str>,
        modifiers: keyboard::Modifiers,
        clipboard: &mut dyn Clipboard,
        now: Instant,
    ) -> bool {
        use keyboard::key::Named;

        let latin = key.to_latin(physical_key);

        if modifiers.command() {
            match latin.map(|character| character.to_ascii_lowercase()) {
                Some('z' | 'y') => {
                    let is_redo = latin
                        .is_some_and(|character| character.eq_ignore_ascii_case(&'y'))
                        || modifiers.shift();

                    let current = self.state.query.to_string();
                    let cursor = self.state.cursor;

                    let snapshot = if is_redo {
                        self.state.history.redo(&current, cursor)
                    } else {
                        self.state.history.undo(&current, cursor)
                    };

                    if let Some(snapshot) = snapshot {
                        self.state.query = Value::new(&snapshot.value);
                        self.state.cursor = snapshot.cursor;
                        self.state.hovered_item = None;
                    }

                    return true;
                }
                Some('c') => {
                    self.copy_query(clipboard);

                    return true;
                }
                Some('x') => {
                    self.copy_query(clipboard);
                    self.state
                        .edit_query(now, EditKind::Other, |editor| editor.delete());

                    return true;
                }
                Some('v') if !modifiers.alt() => {
                    let content: String = clipboard
                        .read(clipboard::Kind::Standard)
                        .unwrap_or_default()
                        .chars()
                        .filter(|character| !character.is_control())
                        .collect();

                    self.state.edit_query(now, EditKind::Other, |editor| {
                        editor.paste(Value::new(&content))
                    });

                    return true;
                }
                Some('a') => {
                    self.state.history.break_group();
                    self.state.cursor.select_all(&self.state.query);

                    return true;
                }
                _ => {}
            }
        }

        match key {
            keyboard::Key::Named(Named::Escape) => {
                if self.state.query.is_empty() {
                    return false;
                }

                self.state.reset_query(now);
            }
            keyboard::Key::Named(Named::Backspace) => {
                if self.state.cursor.selection(&self.state.query).is_none() {
                    if modifiers.macos_command() {
                        let start = self.state.cursor.start(&self.state.query);
                        self.state.cursor.select_range(start, 0);
                    } else if modifiers.jump() {
                        self.state.cursor.select_left_by_words(&self.state.query);
                    }
                }

                self.state
                    .edit_query(now, EditKind::Delete, |editor| editor.backspace());
            }
            keyboard::Key::Named(Named::Delete) => {
                if self.state.cursor.selection(&self.state.query).is_none() {
                    if modifiers.macos_command() {
                        let start = self.state.cursor.start(&self.state.query);
                        let end = self.state.query.len();
                        self.state.cursor.select_range(start, end);
                    } else if modifiers.jump() {
                        self.state.cursor.select_right_by_words(&self.state.query);
                    }
                }

                self.state
                    .edit_query(now, EditKind::Delete, |editor| editor.delete());
            }
            keyboard::Key::Named(Named::Home) => {
                self.state.history.break_group();

                if modifiers.shift() {
                    let start = self.state.cursor.start(&self.state.query);
                    self.state.cursor.select_range(start, 0);
                } else {
                    self.state.cursor.move_to(0);
                }
            }
            keyboard::Key::Named(Named::End) => {
                self.state.history.break_group();

                let end = self.state.query.len();

                if modifiers.shift() {
                    let start = self.state.cursor.start(&self.state.query);
                    self.state.cursor.select_range(start, end);
                } else {
                    self.state.cursor.move_to(end);
                }
            }
            keyboard::Key::Named(Named::ArrowLeft) => {
                self.state.history.break_group();

                if modifiers.macos_command() {
                    if modifiers.shift() {
                        let start = self.state.cursor.start(&self.state.query);
                        self.state.cursor.select_range(start, 0);
                    } else {
                        self.state.cursor.move_to(0);
                    }
                } else if modifiers.jump() {
                    if modifiers.shift() {
                        self.state.cursor.select_left_by_words(&self.state.query);
                    } else {
                        self.state.cursor.move_left_by_words(&self.state.query);
                    }
                } else if modifiers.shift() {
                    self.state.cursor.select_left(&self.state.query);
                } else {
                    self.state.cursor.move_left(&self.state.query);
                }
            }
            keyboard::Key::Named(Named::ArrowRight) => {
                self.state.history.break_group();

                if modifiers.macos_command() {
                    let end = self.state.query.len();

                    if modifiers.shift() {
                        let start = self.state.cursor.start(&self.state.query);
                        self.state.cursor.select_range(start, end);
                    } else {
                        self.state.cursor.move_to(end);
                    }
                } else if modifiers.jump() {
                    if modifiers.shift() {
                        self.state.cursor.select_right_by_words(&self.state.query);
                    } else {
                        self.state.cursor.move_right_by_words(&self.state.query);
                    }
                } else if modifiers.shift() {
                    self.state.cursor.select_right(&self.state.query);
                } else {
                    self.state.cursor.move_right(&self.state.query);
                }
            }
            keyboard::Key::Named(Named::Space) => {
                self.state
                    .edit_query(now, EditKind::Insert, |editor| editor.insert(' '));
            }
            keyboard::Key::Character(_) => {
                if modifiers.control() || modifiers.alt() || modifiers.logo() {
                    // Leave other shortcuts (copy, cut, …) to the rest of the app.
                    return false;
                }

                match text.and_then(|text| text.chars().find(|character| !character.is_control())) {
                    Some(character) => {
                        self.state
                            .edit_query(now, EditKind::Insert, |editor| editor.insert(character));
                    }
                    None => return false,
                }
            }
            _ => return false,
        }

        true
    }

    /// Width of the first `index` graphemes of the search query.
    fn query_width<Renderer>(&self, renderer: &Renderer, index: usize, font: iced::Font) -> f32
    where
        Renderer: text::Renderer<Font = iced::Font>,
    {
        text_width(
            renderer,
            &self.state.query.until(index).to_string(),
            self.item_text_size,
            font,
        )
    }

    /// Horizontal offset used to keep the caret inside the visible search field.
    fn search_scroll<Renderer>(&self, renderer: &Renderer, available: f32, font: iced::Font) -> f32
    where
        Renderer: text::Renderer<Font = iced::Font>,
    {
        let caret = self.state.cursor.end(&self.state.query);

        if caret == 0 {
            return 0.0;
        }

        (self.query_width(renderer, caret, font) + 1.0 - available).max(0.0)
    }

    /// The query index closest to the given `x` position inside the search field.
    fn query_index_at<Renderer>(
        &self,
        renderer: &Renderer,
        x: f32,
        text_x: f32,
        available: f32,
        font: iced::Font,
    ) -> usize
    where
        Renderer: text::Renderer<Font = iced::Font>,
    {
        let local = x - (text_x - self.search_scroll(renderer, available, font));

        for index in 0..self.state.query.len() {
            let start = self.query_width(renderer, index, font);
            let end = self.query_width(renderer, index + 1, font);

            if local < (start + end) / 2.0 {
                return index;
            }
        }

        self.state.query.len()
    }
}

impl<'a, T, Message, Theme, Renderer> overlay::Overlay<Message, Theme, Renderer>
    for Overlay<'a, T, Message>
where
    Message: Clone,
    Renderer: text::Renderer<Font = iced::Font> + svg::Renderer + 'a,
{
    fn layout(&mut self, _renderer: &Renderer, bounds: Size) -> layout::Node {
        // Reuse the geometry captured on the first layout after opening. Recomputing it later
        // (for example when the model updates right after a pick) would make the surface jump
        // while it is still visible.
        if let Some(geometry) = self.state.geometry
            && geometry.viewport == bounds
        {
            // Keep the stored offset inside the current content: hiding separators while a query
            // is active can shorten the list and leave the offset past its end.
            self.state.scroll_offset = self
                .state
                .scroll_offset
                .clamp(0.0, self.max_offset(geometry.rect.height));

            return layout::Node::new(geometry.rect.size()).move_to(geometry.rect.position());
        }

        let anchor = self.anchor_bounds;
        let padding = constants::MENU_PADDING;
        let gap = constants::GAP;
        let margin = constants::VIEWPORT_MARGIN;

        let natural = self.natural_height();
        let want = natural.min(self.max_menu_height);

        let fit = placement::fit_vertically(
            anchor,
            bounds.height,
            want,
            gap,
            margin,
            match self.direction {
                OpenDirection::Down => placement::DropDirection::Down,
                OpenDirection::Up => placement::DropDirection::Up,
                OpenDirection::Center => placement::DropDirection::Center,
            },
        );
        let y = fit.y;
        let height = fit.height;

        let max_offset = self.max_offset(height);

        // Open with the selected option centered in the viewport. Selections near either end of a
        // long list clamp to the scroll bounds, which is unavoidable.
        let initial_offset = self.selected_row().map_or(0.0, |row| {
            let rows = self.rows();
            let viewport = (height - 2.0 * padding - self.header_height()).max(0.0);
            let center = self.row_top(&rows, row) + self.row_height(rows[row]) / 2.0;

            (center - viewport / 2.0).clamp(0.0, max_offset)
        });

        if self.state.scroll_initialized {
            self.state.scroll_offset = self.state.scroll_offset.clamp(0.0, max_offset);
        } else {
            self.state.scroll_offset = initial_offset;
        }

        self.state.scroll_initialized = true;

        let rect = Rectangle::new(Point::new(anchor.x, y), Size::new(anchor.width, height));
        self.state.geometry = Some(MenuGeometry {
            viewport: bounds,
            rect,
        });

        layout::Node::new(rect.size()).move_to(rect.position())
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &iced::advanced::renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
    ) {
        let bounds = layout.bounds();

        if bounds.width <= 0.0 || bounds.height <= 0.0 {
            return;
        }

        let percentage = self.state.progress.value().clamp(0.0, 1.0);

        if percentage <= 0.0 {
            return;
        }

        let fade = |color: Color| Color {
            a: color.a * percentage,
            ..color
        };

        // Fade the surface in and out in place; only the opacity is animated.
        let visible = bounds;

        renderer.fill_quad(
            Quad {
                bounds: visible,
                border: Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: constants::MENU_RADIUS.into(),
                },
                shadow: shadow(fade(self.style.menu_shadow), self.style.elevation),
                ..Quad::default()
            },
            fade(self.style.menu_background),
        );

        if bounds.width <= 2.0 * constants::MENU_PADDING
            || bounds.height <= 2.0 * constants::MENU_PADDING
        {
            return;
        }

        let font = self.font.unwrap_or_else(|| renderer.default_font());
        let line_height = text::LineHeight::default();
        let text_x_offset =
            constants::ITEM_PADDING + constants::ITEM_ICON_SLOT + constants::ITEM_ICON_SPACING;

        if self.searchable {
            let search = self.search_bounds(bounds);

            // Thin divider separating the search field from the options.
            renderer.fill_quad(
                Quad {
                    bounds: Rectangle::new(
                        Point::new(
                            search.x + constants::ITEM_PADDING,
                            search.y + search.height - 1.0,
                        ),
                        Size::new((search.width - 2.0 * constants::ITEM_PADDING).max(0.0), 1.0),
                    ),
                    ..Quad::default()
                },
                fade(self.style.separator),
            );

            icon::draw_svg_icon(
                renderer,
                &common_icons::SEARCH,
                Point::new(
                    search.x + constants::ITEM_PADDING + constants::ITEM_ICON_SLOT / 2.0,
                    search.center_y(),
                ),
                constants::ITEM_ICON_SIZE,
                Radians(0.0),
                fade(self.style.field_icon),
                1.0,
                search,
            );

            let text_x_base = search.x + text_x_offset;
            let available = (search.x + search.width
                - constants::ITEM_PADDING
                - self.search_clear_reserved()
                - text_x_base)
                .max(0.0);

            if available > 0.0 {
                // Clip the editable text to its own slot so a scrolled query cannot run under the
                // leading search icon.
                let viewport = Rectangle::new(
                    Point::new(text_x_base, search.y),
                    Size::new(available, search.height),
                );
                let scroll = self.search_scroll(renderer, available, font);
                let text_x = text_x_base - scroll;
                let caret = self.state.cursor.end(&self.state.query);
                let caret_x = text_x + self.query_width(renderer, caret, font);

                if let Some((start, end)) = self.state.cursor.selection(&self.state.query) {
                    let start_x = text_x + self.query_width(renderer, start, font);
                    let end_x = text_x + self.query_width(renderer, end, font);

                    renderer.with_layer(viewport, |renderer| {
                        renderer.fill_quad(
                            Quad {
                                bounds: Rectangle::new(
                                    Point::new(start_x, search.center_y() - 9.0),
                                    Size::new((end_x - start_x).max(0.0), 18.0),
                                ),
                                ..Quad::default()
                            },
                            fade(self.style.field_outline_open.scale_alpha(0.35)),
                        );
                    });
                }

                let (content, color, position) = if self.state.query.is_empty() {
                    (
                        self.labels.search.to_string(),
                        self.style.field_placeholder,
                        text_x_base,
                    )
                } else {
                    (self.state.query.to_string(), self.style.field_text, text_x)
                };

                renderer.fill_text(
                    Text {
                        content,
                        bounds: Size::new(
                            available,
                            line_height.to_absolute(self.item_text_size).0,
                        ),
                        size: self.item_text_size,
                        line_height,
                        font,
                        align_x: text::Alignment::Left,
                        align_y: Vertical::Center,
                        shaping: text::Shaping::Advanced,
                        wrapping: text::Wrapping::None,
                    },
                    Point::new(position, search.center_y()),
                    fade(color),
                    viewport,
                );

                let caret_visible = self.state.search_focused
                    && matches!(
                        self.state.cursor.state(&self.state.query),
                        cursor::State::Index(_)
                    )
                    && self.state.blink.is_caret_visible();

                if caret_visible {
                    let caret_max = (text_x_base + available - 1.0).max(text_x_base);
                    let caret_x = caret_x.clamp(text_x_base, caret_max);

                    renderer.with_layer(viewport, |renderer| {
                        renderer.fill_quad(
                            Quad {
                                bounds: Rectangle::new(
                                    Point::new(caret_x, search.center_y() - 9.0),
                                    Size::new(1.5, 18.0),
                                ),
                                ..Quad::default()
                            },
                            fade(self.style.field_text),
                        );
                    });
                }
            }

            if self.search_clear_visible() {
                icon::draw_svg_icon(
                    renderer,
                    &common_icons::CLOSE,
                    self.search_clear_center(bounds),
                    constants::CLEAR_ICON_SIZE,
                    Radians(0.0),
                    fade(if self.state.search_clear_hovered {
                        self.style.field_text
                    } else {
                        self.style.field_icon
                    }),
                    1.0,
                    search,
                );
            }
        }

        let list = self.list_bounds(bounds);

        if list.width <= 0.0 || list.height <= 0.0 {
            return;
        }

        let offset = self.scroll_offset(bounds.height);
        let rows = self.rows();
        let rail = self.scrollbar(bounds);

        // Keep options clear of the scrollbar: reserve a gutter at the trailing edge only when the
        // list actually scrolls, so short menus keep their full width.
        let item_width = if rail.is_some() {
            (list.width - constants::SCROLLBAR_GUTTER).max(0.0)
        } else {
            list.width
        };

        let mut top = list.y - offset;

        renderer.with_layer(list, |renderer| {
            for &row in &rows {
                let height = self.row_height(row);
                let item = Rectangle::new(Point::new(list.x, top), Size::new(item_width, height));

                top += height + constants::ITEM_SPACING;

                if item.y + item.height <= list.y || item.y >= list.y + list.height {
                    continue;
                }

                let Row::Option { index, matched } = row else {
                    // Separators are a thin, inset rule centered in their row.
                    renderer.fill_quad(
                        Quad {
                            bounds: Rectangle::new(
                                Point::new(
                                    item.x + constants::ITEM_PADDING,
                                    item.center_y() - constants::SEPARATOR_RULE_HEIGHT / 2.0,
                                ),
                                Size::new(
                                    (item.width - 2.0 * constants::ITEM_PADDING).max(0.0),
                                    constants::SEPARATOR_RULE_HEIGHT,
                                ),
                            ),
                            border: Border::default()
                                .rounded(constants::SEPARATOR_RULE_HEIGHT / 2.0),
                            ..Quad::default()
                        },
                        fade(self.style.separator),
                    );

                    continue;
                };

                let label = self.items[index].label().unwrap_or_default();

                // Options that do not match the search stay visible but are disabled. The option
                // that is currently selected still reads as selected, even while disabled.
                let highlighted = self.is_selected(index);
                let is_hovered = matched && self.state.hovered_item == Some(index);
                let color = if highlighted {
                    self.style.item_selected_text
                } else if !matched {
                    self.style.item_disabled_text
                } else {
                    self.style.item_text
                };

                if highlighted || is_hovered {
                    let background = if highlighted {
                        self.style.item_selected_background
                    } else {
                        self.style
                            .item_state_layer
                            .scale_alpha(constants::STATE_LAYER_OPACITY)
                    };

                    renderer.fill_quad(
                        Quad {
                            bounds: item,
                            border: Border::default().rounded(constants::ITEM_RADIUS),
                            ..Quad::default()
                        },
                        fade(background),
                    );
                }

                let slot_center = Point::new(
                    item.x + constants::ITEM_PADDING + constants::ITEM_ICON_SLOT / 2.0,
                    item.center_y(),
                );

                if let Some(icon) = self.items[index].icon_ref() {
                    icon::draw_hybrid_icon(
                        renderer,
                        icon,
                        slot_center,
                        constants::ITEM_ICON_SIZE,
                        fade(color),
                        list,
                    );
                }

                let text_x = item.x + text_x_offset;
                let text_width = item.x + item.width - constants::ITEM_PADDING - text_x;

                if text_width > 0.0 {
                    renderer.fill_text(
                        Text {
                            content: self.state.fit.fit(
                                renderer,
                                label,
                                text_width,
                                self.item_text_size,
                                font,
                            ),
                            bounds: Size::new(
                                text_width,
                                line_height.to_absolute(self.item_text_size).0,
                            ),
                            size: self.item_text_size,
                            line_height,
                            font,
                            align_x: text::Alignment::Left,
                            align_y: Vertical::Center,
                            shaping: text::Shaping::Advanced,
                            wrapping: text::Wrapping::None,
                        },
                        Point::new(text_x, item.center_y()),
                        fade(color),
                        list,
                    );
                }
            }

            if let Some(rail) = rail {
                // Same scroller as the page scrollables: a thin pill that fades with the pointer
                // interaction instead of an always-visible bar.
                renderer.fill_quad(
                    Quad {
                        bounds: rail.scroller,
                        border: scrollbar::scroller_border(),
                        ..Quad::default()
                    },
                    fade(scrollbar::scroller_color(
                        self.style.scrollbar,
                        self.state.menu_hovered,
                        self.state.scrollbar_hovered,
                        self.state.scrollbar_drag.is_some(),
                    )),
                );
            }
        });
    }

    fn update(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) {
        let now = Instant::now();
        let target = if self.state.is_open { 1.0 } else { 0.0 };

        self.state.progress.set_target(target, now);

        if !self.state.progress.is_at_rest() {
            shell.request_redraw();
        }

        self.state.progress.step(now);

        // `Select::overlay` drops the popup as soon as this spring comes to rest, but the runtime
        // only rebuilds overlay layouts during `update`. Ask for a relayout while we still can, so
        // the frame that finishes the fade-out does not draw the remaining overlays with the
        // layout computed while the popup still existed.
        if !self.state.is_open && self.state.progress.is_at_rest() {
            shell.invalidate_layout();
        }

        if matches!(
            event,
            Event::Window(
                window::Event::Unfocused
                    | window::Event::Moved(_)
                    | window::Event::Resized(_)
                    | window::Event::Rescaled(_)
                    | window::Event::Closed,
            )
        ) && self.state.context_menu.take().is_some()
        {
            shell.request_redraw();
        }

        match event {
            Event::Window(window::Event::Focused) => {
                self.state.blink.is_window_focused = true;

                if self.state.search_focused {
                    self.state.restart_blink(now);
                    shell.request_redraw();
                }
            }
            Event::Window(window::Event::Unfocused) => {
                self.state.blink.is_window_focused = false;
            }
            _ => {}
        }

        if !self.state.is_open {
            return;
        }

        // Keep the caret blinking like a text input by scheduling a redraw on each phase change.
        if let Event::Window(window::Event::RedrawRequested(now)) = event
            && self.searchable
            && self.state.search_focused
            && self.state.blink.is_window_focused
            && matches!(
                self.state.cursor.state(&self.state.query),
                cursor::State::Index(_)
            )
        {
            self.state.blink.now = *now;

            let millis_until_redraw = self.state.blink.millis_until_blink();

            shell.request_redraw_at(*now + Duration::from_millis(millis_until_redraw as u64));
        }

        let bounds = layout.bounds();

        let pointer = match event {
            Event::Mouse(mouse::Event::CursorMoved { .. })
            | Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => cursor.position(),
            Event::Touch(touch::Event::FingerPressed { position, .. })
            | Event::Touch(touch::Event::FingerMoved { position, .. }) => Some(*position),
            _ => None,
        };

        if let Some(position) = pointer {
            let rail = self.scrollbar(bounds);

            // Track an active scrollbar drag before anything else so it follows the pointer even
            // when it leaves the menu bounds.
            if let Some(grab) = self.state.scrollbar_drag
                && let Some(rail) = rail
            {
                self.scroll_to_thumb(&rail, position.y - grab);
                shell.request_redraw();
                shell.capture_event();
            }

            let scrollbar_hovered = rail.is_some_and(|rail| rail.bounds.contains(position));
            let menu_hovered = bounds.contains(position);
            let dragging = self.state.scrollbar_drag.is_some();
            let hovered = if dragging {
                None
            } else {
                self.item_at(Some(position), bounds)
            };
            let clear_hovered = !dragging
                && self
                    .search_clear_bounds(bounds)
                    .is_some_and(|clear| clear.contains(position));

            if self.state.hovered_item != hovered
                || self.state.search_clear_hovered != clear_hovered
                || self.state.scrollbar_hovered != scrollbar_hovered
                || self.state.menu_hovered != menu_hovered
            {
                self.state.hovered_item = hovered;
                self.state.search_clear_hovered = clear_hovered;
                self.state.scrollbar_hovered = scrollbar_hovered;
                self.state.menu_hovered = menu_hovered;
                shell.request_redraw();
            }
        }

        // Each row and button is a target of its own, so the delay restarts when the pointer moves
        // to another option instead of opening the tooltip instantly.
        let tooltip_target = self.tooltip_target(renderer, bounds);

        self.state.menu_tooltip.update(
            event,
            cursor,
            tooltip_target,
            self.state.is_open,
            tooltip::DEFAULT_DELAY,
            shell,
        );

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)) => {
                let Some(position) = cursor.position() else {
                    if self.state.context_menu.take().is_some() {
                        shell.request_redraw();
                    }

                    return;
                };

                if self.searchable && self.search_bounds(bounds).contains(position) {
                    let text = self.search_text_bounds(bounds);
                    let font = self.font.unwrap_or_else(|| renderer.default_font());

                    if self.state.cursor.selection(&self.state.query).is_none() {
                        let index =
                            self.query_index_at(renderer, position.x, text.x, text.width, font);
                        self.state.cursor.move_to(index);
                    }

                    let is_already_opened = self
                        .state
                        .context_menu
                        .is_some_and(|menu| menu.position == position);

                    if !is_already_opened {
                        self.state.context_menu = Some(ContextMenu::new(position));
                    }

                    self.state.search_focused = true;
                    self.state.history.break_group();
                    self.state.restart_blink(now);

                    shell.request_redraw();
                    shell.capture_event();
                } else if self.state.context_menu.take().is_some() {
                    shell.request_redraw();
                }
            }
            Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                if cursor.is_over(bounds) {
                    let movement = scrollbar::wheel_offset(*delta);
                    let max_offset = self.max_offset(bounds.height);
                    let current = self.scroll_offset(bounds.height);
                    let next = (current + movement).clamp(0.0, max_offset);

                    if (next - self.state.scroll_offset).abs() > f32::EPSILON {
                        self.state.scroll_offset = next;
                        shell.request_redraw();
                    }

                    shell.capture_event();
                }
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                let position = pointer.unwrap_or(Point::new(0.0, 0.0));
                let search = self.search_bounds(bounds);

                if self.searchable && search.contains(position) {
                    // The clear button resets the query without losing keyboard focus.
                    if self
                        .search_clear_bounds(bounds)
                        .is_some_and(|clear| clear.contains(position))
                    {
                        self.state.reset_query(now);
                        self.state.search_focused = true;
                        self.state.history.break_group();
                        shell.request_redraw();
                        shell.capture_event();
                        return;
                    }

                    // Clicking the search field focuses it. Like a text input, it places the caret
                    // on a single click, selects a word on a double click and everything on a
                    // triple click.
                    let text = self.search_text_bounds(bounds);
                    let font = self.font.unwrap_or_else(|| renderer.default_font());
                    let index = self.query_index_at(renderer, position.x, text.x, text.width, font);
                    let click =
                        mouse::Click::new(position, mouse::Button::Left, self.state.last_click);

                    match click.kind() {
                        mouse::click::Kind::Single => {
                            if self.state.keyboard_modifiers.shift() {
                                let start = self.state.cursor.start(&self.state.query);
                                self.state.cursor.select_range(start, index);
                            } else {
                                self.state.cursor.move_to(index);
                            }

                            self.state.dragging = Some(SearchDrag::Select);
                        }
                        mouse::click::Kind::Double => {
                            let start = self.state.query.previous_start_of_word(index);
                            let end = self.state.query.next_end_of_word(index);
                            self.state.cursor.select_range(start, end);

                            self.state.dragging = Some(SearchDrag::SelectWords { anchor: index });
                        }
                        mouse::click::Kind::Triple => {
                            self.state.cursor.select_all(&self.state.query);
                            self.state.dragging = None;
                        }
                    }

                    self.state.search_focused = true;
                    self.state.last_click = Some(click);
                    self.state.history.break_group();
                    self.state.restart_blink(now);
                    shell.request_redraw();
                    shell.capture_event();
                } else if let Some(rail) = self.scrollbar(bounds)
                    && rail.bounds.contains(position)
                {
                    // Pressing the rail grabs the scrollbar: clicking the scroller keeps the grab
                    // offset, clicking the track centers the scroller under the pointer.
                    let grab = rail.grab(position);

                    self.scroll_to_thumb(&rail, position.y - grab);
                    self.state.scrollbar_drag = Some(grab);
                    self.state.scrollbar_hovered = true;
                    shell.request_redraw();
                    shell.capture_event();
                } else if let Some(index) = self.item_at(Some(position), bounds) {
                    let selection = if self.clearable && self.selected == Some(index) {
                        None
                    } else {
                        Some(index)
                    };

                    shell.publish((self.on_select)(selection));
                    shell.request_redraw();
                    shell.capture_event();

                    if self.close_on_select {
                        self.state.close();
                    } else {
                        // Keep the menu open and highlight the new pick right away instead of
                        // waiting for the next frame.
                        self.state.shown_selected = selection;
                        self.selected = selection;
                    }
                } else if bounds.contains(position) {
                    // Press inside the menu surface but not on an option: swallow it so the
                    // menu stays open instead of closing through the anchor field.
                    shell.capture_event();
                }
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key,
                physical_key,
                text,
                modifiers,
                ..
            }) => {
                self.state.keyboard_modifiers = *modifiers;

                if self.searchable && self.state.search_focused {
                    let consumed = self.handle_search_key(
                        key,
                        *physical_key,
                        text.as_deref(),
                        *modifiers,
                        clipboard,
                        now,
                    );

                    if consumed {
                        self.state.restart_blink(now);
                        shell.request_redraw();
                        shell.capture_event();
                        return;
                    }
                }

                if matches!(key, keyboard::Key::Named(keyboard::key::Named::Escape)) {
                    self.state.close();
                    shell.request_redraw();
                    shell.capture_event();
                }
            }
            Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) => {
                self.state.keyboard_modifiers = *modifiers;
            }
            Event::Mouse(mouse::Event::CursorMoved { position })
            | Event::Touch(touch::Event::FingerMoved { position, .. }) => {
                let Some(dragging) = self.state.dragging else {
                    return;
                };

                let text = self.search_text_bounds(bounds);
                let font = self.font.unwrap_or_else(|| renderer.default_font());
                let index = self.query_index_at(renderer, position.x, text.x, text.width, font);

                match dragging {
                    SearchDrag::Select => {
                        let start = self.state.cursor.start(&self.state.query);
                        self.state.cursor.select_range(start, index);
                    }
                    SearchDrag::SelectWords { anchor } => {
                        let (start, end) = if index < anchor {
                            (
                                self.state.query.previous_start_of_word(index),
                                self.state.query.next_end_of_word(anchor),
                            )
                        } else {
                            (
                                self.state.query.previous_start_of_word(anchor),
                                self.state.query.next_end_of_word(index),
                            )
                        };

                        self.state.cursor.select_range(start, end);
                    }
                }

                self.state.history.break_group();
                self.state.restart_blink(now);
                shell.request_redraw();
                shell.capture_event();
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerLifted { .. })
            | Event::Touch(touch::Event::FingerLost { .. }) => {
                self.state.dragging = None;

                if self.state.scrollbar_drag.take().is_some() {
                    shell.request_redraw();
                }
            }
            _ => {}
        }
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        if self.state.scrollbar_drag.is_some() {
            return mouse::Interaction::Grabbing;
        }

        let bounds = layout.bounds();
        let Some(position) = cursor.position() else {
            return mouse::Interaction::None;
        };

        if !bounds.contains(position) {
            return mouse::Interaction::None;
        }

        if !self.state.is_open {
            return mouse::Interaction::None;
        }

        if self
            .scrollbar(bounds)
            .is_some_and(|rail| rail.bounds.contains(position))
        {
            return mouse::Interaction::Pointer;
        }

        if self
            .search_clear_bounds(bounds)
            .is_some_and(|clear| clear.contains(position))
        {
            return mouse::Interaction::Pointer;
        }

        if self.searchable && self.search_bounds(bounds).contains(position) {
            return mouse::Interaction::Text;
        }

        if self.item_at(Some(position), bounds).is_some() {
            return mouse::Interaction::Pointer;
        }

        // Inside the menu but over nothing interactive (an option filtered out by the query, or
        // padding): keep the plain arrow and stop the widget underneath from leaking its own
        // cursor.
        mouse::Interaction::Idle
    }

    fn overlay<'b>(
        &'b mut self,
        _layout: Layout<'b>,
        _renderer: &Renderer,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        if self.state.context_menu.is_some() {
            let style = self.menu_style;
            let font = self.font;

            return Some(overlay::Element::new(Box::new(ContextMenuOverlay {
                host: SearchContext {
                    state: &mut *self.state,
                    actions: self.labels.actions.clone(),
                },
                style,
                font,
                translation: Vector::ZERO,
            })));
        }

        // Any tooltip for the menu is created by `Select::overlay`, next to the surface instead of
        // nested inside it. See `hovered_tooltip`.
        None
    }
}

/// Adapts the search field of the select menu to the shared [`ContextMenuHost`].
struct SearchContext<'a> {
    state: &'a mut State,
    actions: ActionLabels<'a>,
}

impl<Message, Renderer> ContextMenuHost<Message, Renderer> for SearchContext<'_> {
    fn value(&self) -> &Value {
        &self.state.query
    }

    fn cursor(&self) -> Cursor {
        self.state.cursor
    }

    fn is_editable(&self) -> bool {
        true
    }

    fn can_undo(&self) -> bool {
        self.state.history.can_undo()
    }

    fn can_redo(&self) -> bool {
        self.state.history.can_redo()
    }

    fn delete_selection(&mut self, _renderer: &Renderer) -> Option<Message> {
        self.state
            .edit_query(Instant::now(), EditKind::Other, |editor| editor.delete());

        None
    }

    fn paste(&mut self, content: Value, _renderer: &Renderer) -> Option<Message> {
        self.state
            .edit_query(Instant::now(), EditKind::Other, |editor| {
                editor.paste(content)
            });

        None
    }

    fn undo(&mut self, _renderer: &Renderer) -> Option<Message> {
        self.history_step(Instant::now(), false);

        None
    }

    fn redo(&mut self, _renderer: &Renderer) -> Option<Message> {
        self.history_step(Instant::now(), true);

        None
    }

    fn select_word(&mut self) {
        let position = self.state.cursor.end(&self.state.query);
        let start = self.state.query.previous_start_of_word(position);
        let end = self.state.query.next_end_of_word(position);

        self.state.cursor.select_range(start, end);
    }

    fn select_all(&mut self) {
        self.state.cursor.select_all(&self.state.query);
    }

    fn menu(&self) -> Option<ContextMenu> {
        self.state.context_menu
    }

    fn set_menu(&mut self, menu: Option<ContextMenu>) {
        self.state.context_menu = menu;
    }

    fn label(&self, action: Action) -> std::borrow::Cow<'_, str> {
        std::borrow::Cow::Borrowed(self.actions.get(action))
    }
}

impl SearchContext<'_> {
    fn history_step(&mut self, now: Instant, is_redo: bool) {
        let current = self.state.query.to_string();
        let cursor = self.state.cursor;

        let snapshot = if is_redo {
            self.state.history.redo(&current, cursor)
        } else {
            self.state.history.undo(&current, cursor)
        };

        if let Some(snapshot) = snapshot {
            self.state.query = Value::new(&snapshot.value);
            self.state.cursor = snapshot.cursor;
            self.state.hovered_item = None;
            self.state.restart_blink(now);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Item, fuzzy_score, order_options};

    fn options<'a>(candidates: &[&'a str]) -> Vec<Item<&'a str>> {
        candidates
            .iter()
            .map(|candidate| Item::new(*candidate))
            .collect()
    }

    fn ordered<'a>(query: &str, candidates: &[&'a str]) -> Vec<&'a str> {
        order_options(query, &options(candidates))
            .into_iter()
            .map(|(index, _)| candidates[index])
            .collect()
    }

    fn disabled<'a>(query: &str, candidates: &[&'a str]) -> Vec<&'a str> {
        order_options(query, &options(candidates))
            .into_iter()
            .filter(|&(_, matched)| !matched)
            .map(|(index, _)| candidates[index])
            .collect()
    }

    fn ranked<'a>(query: &str, candidates: &[&'a str]) -> Vec<&'a str> {
        let mut scored: Vec<(&str, i32)> = candidates
            .iter()
            .filter_map(|candidate| fuzzy_score(query, candidate).map(|score| (*candidate, score)))
            .collect();

        scored.sort_by_key(|&(_, score)| std::cmp::Reverse(score));

        scored.into_iter().map(|(candidate, _)| candidate).collect()
    }

    #[test]
    fn substring_matches_before_subsequences() {
        assert_eq!(
            ranked("item", &["one item", "I t e m", "item"]),
            ["item", "one item", "I t e m"]
        );
    }

    #[test]
    fn prefix_and_earlier_matches_rank_higher() {
        assert_eq!(
            ranked("op", &["opaque", "stop", "top"]),
            ["opaque", "top", "stop"]
        );
    }

    #[test]
    fn fuzzy_score_rejects_non_matches() {
        assert_eq!(
            ranked("zzz", &["Option 01", "Option 02"]),
            Vec::<&str>::new()
        );
    }

    #[test]
    fn subsequence_match_is_found() {
        assert!(fuzzy_score("o1", "Option 01").is_some());
        assert!(fuzzy_score("x1", "Option 01").is_none());
    }

    #[test]
    fn empty_query_matches_everything() {
        assert_eq!(fuzzy_score("", "anything"), Some(0));
    }

    #[test]
    fn ordering_keeps_every_option() {
        assert_eq!(
            ordered("op", &["opaque", "stop", "top", "zoom"]),
            ["opaque", "top", "stop", "zoom"]
        );
    }

    #[test]
    fn non_matches_keep_their_order_at_the_end() {
        assert_eq!(
            ordered("zzz", &["Option 01", "Option 02"]),
            ["Option 01", "Option 02"]
        );
    }

    #[test]
    fn empty_query_keeps_the_original_order() {
        assert_eq!(
            ordered("", &["third", "first", "second"]),
            ["third", "first", "second"]
        );
    }

    #[test]
    fn non_matching_options_are_disabled() {
        assert_eq!(disabled("op", &["opaque", "stop", "top", "zoom"]), ["zoom"]);
        assert!(disabled("", &["opaque", "zoom"]).is_empty());
    }

    #[test]
    fn separators_have_no_value_or_label() {
        let separator = Item::<&str>::separator();

        assert!(separator.is_separator());
        assert_eq!(separator.value(), None);
        assert_eq!(separator.label(), None);
    }

    #[test]
    fn ordering_skips_separators() {
        let items = vec![Item::new("alpha"), Item::separator(), Item::new("beta")];

        let ordered: Vec<usize> = order_options("", &items)
            .into_iter()
            .map(|(index, _)| index)
            .collect();

        assert_eq!(ordered, [0, 2]);
    }
}
