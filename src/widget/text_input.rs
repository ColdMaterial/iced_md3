mod action;
mod context_menu;
mod editor;
mod value;

pub mod cursor;

pub use action::{Action, ActionLabels, Shortcut};
pub use cursor::Cursor;
use iced::advanced::svg;
use iced::border::Radius;
pub use value::Value;

pub(crate) use context_menu::{ContextMenu, ContextMenuHost, ContextMenuOverlay};
pub(crate) use editor::Editor;

use iced_widget::core::{
    Alignment, Background, Border, Color, Element, Event, InputMethod, Layout, Length, Padding,
    Pixels, Point, Radians, Rectangle, Shell, Size, Theme, Vector, Widget, alignment,
    clipboard::{self, Clipboard},
    input_method,
    keyboard::{self, key},
    layout,
    mouse::{self, click},
    overlay, renderer,
    text::{
        self, Text,
        paragraph::{self, Paragraph as _},
    },
    time::{Duration, Instant},
    touch,
    widget::{
        self,
        operation::{self, Operation},
        tree::{self, Tree},
    },
    window,
};

use crate::animation::motion::{self, SpringValue};
use crate::style::DIM_ALPHA;
use crate::theme::ColorScheme;
use crate::widget::internal::focus::Focus;
use crate::widget::{common_icons, menu, tooltip};
use iced_widget::text::IntoFragment;
use std::borrow::Cow;

#[derive(Debug, Clone)]
pub struct Labels<'a> {
    pub clear: text::Fragment<'a>,
    pub show_password: text::Fragment<'a>,
    pub hide_password: text::Fragment<'a>,
    pub actions: ActionLabels<'a>,
}

impl Default for Labels<'_> {
    fn default() -> Self {
        Self {
            clear: "Clear".into(),
            show_password: Cow::Borrowed("Show password"),
            hide_password: Cow::Borrowed("Hide password"),
            actions: ActionLabels::default(),
        }
    }
}

pub struct TextInput<'a, Message, Theme = iced_widget::Theme, Renderer = iced_widget::Renderer>
where
    Theme: Catalog,
    Renderer: text::Renderer + svg::Renderer,
{
    id: Option<widget::Id>,
    placeholder: String,
    value: Value,
    is_secure: bool,
    font: Option<Renderer::Font>,
    width: Length,
    padding: Padding,
    size: Option<Pixels>,
    line_height: text::LineHeight,
    alignment: alignment::Horizontal,
    on_input: Option<Box<dyn Fn(String) -> Message + 'a>>,
    on_paste: Option<Box<dyn Fn(String) -> Message + 'a>>,
    on_submit: Option<Message>,
    on_toggle_secure: Option<Message>,
    icon: Option<Icon<Renderer::Font>>,
    class: Theme::Class<'a>,
    last_status: Option<Status>,
    theme: &'a dyn ColorScheme,
    label_text: Option<&'a str>,
    background_color: Option<Color>,
    error: bool,
    clearable: bool,
    tooltips_enabled: bool,
    tooltip_style: tooltip::Style,
    icon_tooltip: Option<text::Fragment<'a>>,
    labels: Labels<'a>,
}

/// The default [`Padding`] of a [`TextInput`].
pub const DEFAULT_PADDING: Padding = Padding::new(12.0);
const LABEL_BACKGROUND_PADDING: f32 = 4.0;
const LABEL_TEXT_SIZE: Pixels = Pixels(12.0);
const LABEL_TEXT_OFFSET: Point = Point::new(
    12.0 + LABEL_BACKGROUND_PADDING,
    LABEL_TEXT_SIZE.0 / 4.0 - BORDER_WIDTH / 2.0,
);
const BORDER_WIDTH: f32 = 2.0;
const EYE_ICON_SIZE: f32 = 24.0;
const EYE_ICON_SPACING: f32 = 8.0;
const CLEAR_ICON_SIZE: f32 = 20.0;
const CLEAR_ICON_SPACING: f32 = 8.0;

/// The maximum amount of undo steps kept by a [`TextInput`].
const MAX_UNDO_STEPS: usize = 64;

/// The time window used to group consecutive insertions or deletions into a
/// single undo step.
const UNDO_GROUPING_WINDOW: Duration = Duration::from_millis(500);

#[derive(Debug, Default, Clone)]
pub(crate) struct History {
    undo: Vec<Snapshot>,
    redo: Vec<Snapshot>,
    last_edit: Option<EditKind>,
    last_edit_at: Option<Instant>,
    last_value: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct Snapshot {
    pub(crate) value: String,
    pub(crate) cursor: Cursor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EditKind {
    Insert,
    Delete,
    Other,
}

impl History {
    pub(crate) fn record(&mut self, value: &str, cursor: Cursor, kind: EditKind) {
        let now = Instant::now();

        let is_grouped = matches!(kind, EditKind::Insert | EditKind::Delete)
            && self.last_edit == Some(kind)
            && self
                .last_edit_at
                .is_some_and(|last| now.duration_since(last) < UNDO_GROUPING_WINDOW);

        self.redo.clear();

        if !is_grouped {
            if self.undo.len() == MAX_UNDO_STEPS {
                self.undo.remove(0);
            }

            self.undo.push(Snapshot {
                value: value.to_owned(),
                cursor,
            });
        }

        self.last_edit = Some(kind);
        self.last_edit_at = Some(now);
    }

    pub(crate) fn commit(&mut self, value: &str) {
        self.last_value = Some(value.to_owned());
    }

    pub(crate) fn break_group(&mut self) {
        self.last_edit = None;
        self.last_edit_at = None;
    }

    pub(crate) fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub(crate) fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    pub(crate) fn undo(&mut self, value: &str, cursor: Cursor) -> Option<Snapshot> {
        self.break_group();

        let snapshot = self.undo.pop()?;

        self.redo.push(Snapshot {
            value: value.to_owned(),
            cursor,
        });

        self.last_value = Some(snapshot.value.clone());

        Some(snapshot)
    }

    pub(crate) fn redo(&mut self, value: &str, cursor: Cursor) -> Option<Snapshot> {
        self.break_group();

        let snapshot = self.redo.pop()?;

        if self.undo.len() == MAX_UNDO_STEPS {
            self.undo.remove(0);
        }

        self.undo.push(Snapshot {
            value: value.to_owned(),
            cursor,
        });

        self.last_value = Some(snapshot.value.clone());

        Some(snapshot)
    }

    pub(crate) fn reset(&mut self, value: &str) {
        self.undo.clear();
        self.redo.clear();
        self.break_group();
        self.last_value = Some(value.to_owned());
    }
}

pub(crate) fn edit_value(
    value: &mut Value,
    cursor: &mut Cursor,
    history: &mut History,
    kind: EditKind,
    change: impl FnOnce(&mut Editor<'_>),
) -> String {
    let before = value.to_string();
    let cursor_before = *cursor;

    let contents = {
        let mut editor = Editor::new(value, cursor);
        change(&mut editor);
        editor.contents()
    };

    if contents != before {
        history.record(&before, cursor_before, kind);
        history.commit(&contents);
    }

    contents
}

impl<'a, Message, Theme, Renderer> TextInput<'a, Message, Theme, Renderer>
where
    Message: Clone,
    Theme: Catalog,
    Renderer: text::Renderer + svg::Renderer,
{
    /// Creates a new [`TextInput`] with the given placeholder and
    /// its current value.
    pub fn new(placeholder: &str, value: &str, theme: &'a impl ColorScheme) -> Self {
        TextInput {
            id: None,
            placeholder: String::from(placeholder),
            value: Value::new(value),
            is_secure: false,
            font: None,
            width: Length::Fill,
            padding: DEFAULT_PADDING,
            size: None,
            line_height: text::LineHeight::default(),
            alignment: alignment::Horizontal::Left,
            on_input: None,
            on_paste: None,
            on_submit: None,
            on_toggle_secure: None,
            icon: None,
            class: Theme::default(),
            last_status: None,
            theme,
            label_text: None,
            background_color: None,
            error: false,
            clearable: false,
            tooltips_enabled: false,
            tooltip_style: tooltip::Style::new(theme),
            icon_tooltip: None,
            labels: Labels::default(),
        }
    }

    pub fn error(mut self, error: bool) -> Self {
        self.error = error;
        self
    }

    /// Shows a button that clears the contents of the [`TextInput`] when pressed.
    ///
    /// The button is only shown when the [`TextInput`] has an
    /// [`on_input`](Self::on_input) callback and its value is not empty.
    pub fn clearable(mut self, clearable: bool) -> Self {
        self.clearable = clearable;
        self
    }

    /// Enables tooltips for the secure and clear buttons.
    #[must_use]
    pub fn tooltips_enabled(mut self, enabled: bool) -> Self {
        self.tooltips_enabled = enabled;
        self
    }

    /// Sets the [`tooltip::Style`] used for the built-in tooltips.
    #[must_use]
    pub fn tooltip_style(mut self, style: tooltip::Style) -> Self {
        self.tooltip_style = style;
        self
    }

    /// Sets the label shown in a tooltip when the [`Icon`] is hovered.
    #[must_use]
    pub fn icon_tooltip(mut self, label: impl IntoFragment<'a>) -> Self {
        self.icon_tooltip = Some(label.into_fragment());
        self
    }

    #[must_use]
    pub fn labels(mut self, labels: Labels<'a>) -> Self {
        self.labels = labels;
        self
    }

    #[must_use]
    pub fn clear_label(mut self, label: impl IntoFragment<'a>) -> Self {
        self.labels.clear = label.into_fragment();
        self
    }

    #[must_use]
    pub fn show_password_label(mut self, label: impl IntoFragment<'a>) -> Self {
        self.labels.show_password = label.into_fragment();
        self
    }

    #[must_use]
    pub fn hide_password_label(mut self, label: impl IntoFragment<'a>) -> Self {
        self.labels.hide_password = label.into_fragment();
        self
    }

    #[must_use]
    pub fn action_label(mut self, action: Action, label: impl IntoFragment<'a>) -> Self {
        self.labels.actions.set(action, label);
        self
    }

    pub fn with_label_text(mut self, label_text: &'a str, background: Color) -> Self {
        self.label_text = Some(label_text);
        self.background_color = Some(background);
        self
    }

    /// Sets the [`widget::Id`] of the [`TextInput`].
    pub fn id(mut self, id: impl Into<widget::Id>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Converts the [`TextInput`] into a secure password input.
    ///
    /// Combine with [`TextInput::on_toggle_secure`] to show a visibility icon.
    pub fn secure(mut self, is_secure: bool) -> Self {
        self.is_secure = is_secure;
        self
    }

    /// Sets the message produced when the visibility toggle icon is pressed.
    ///
    /// Toggle [`TextInput::secure`] in response to reveal or hide the contents.
    pub fn on_toggle_secure(mut self, on_toggle_secure: Message) -> Self {
        self.on_toggle_secure = Some(on_toggle_secure);
        self
    }

    crate::macros::maybe_assign!(on_toggle_secure_maybe, on_toggle_secure, Option<Message>);

    /// Sets the message that should be produced when some text is typed into
    /// the [`TextInput`].
    ///
    /// If this method is not called, the [`TextInput`] will be disabled.
    pub fn on_input(mut self, on_input: impl Fn(String) -> Message + 'a) -> Self {
        self.on_input = Some(Box::new(on_input));
        self
    }

    /// Sets the message that should be produced when some text is typed into
    /// the [`TextInput`], if `Some`.
    ///
    /// If `None`, the [`TextInput`] will be disabled.
    pub fn on_input_maybe(mut self, on_input: Option<impl Fn(String) -> Message + 'a>) -> Self {
        self.on_input = on_input.map(|f| Box::new(f) as _);
        self
    }

    /// Sets the message that should be produced when the [`TextInput`] is
    /// focused and the enter key is pressed.
    pub fn on_submit(mut self, message: Message) -> Self {
        self.on_submit = Some(message);
        self
    }

    crate::macros::maybe_assign!(on_submit_maybe, on_submit, Option<Message>);

    /// Sets the message that should be produced when some text is pasted into
    /// the [`TextInput`].
    pub fn on_paste(mut self, on_paste: impl Fn(String) -> Message + 'a) -> Self {
        self.on_paste = Some(Box::new(on_paste));
        self
    }

    /// Sets the message that should be produced when some text is pasted into
    /// the [`TextInput`], if `Some`.
    pub fn on_paste_maybe(mut self, on_paste: Option<impl Fn(String) -> Message + 'a>) -> Self {
        self.on_paste = on_paste.map(|f| Box::new(f) as _);
        self
    }

    /// Sets the [`Font`] of the [`TextInput`].
    ///
    /// [`Font`]: text::Renderer::Font
    pub fn font(mut self, font: Renderer::Font) -> Self {
        self.font = Some(font);
        self
    }

    /// Sets the [`Icon`] of the [`TextInput`].
    pub fn icon(mut self, icon: Icon<Renderer::Font>) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Sets the width of the [`TextInput`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the [`Padding`] of the [`TextInput`].
    pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets the text size of the [`TextInput`].
    pub fn size(mut self, size: impl Into<Pixels>) -> Self {
        self.size = Some(size.into());
        self
    }

    /// Sets the [`text::LineHeight`] of the [`TextInput`].
    pub fn line_height(mut self, line_height: impl Into<text::LineHeight>) -> Self {
        self.line_height = line_height.into();
        self
    }

    /// Sets the horizontal alignment of the [`TextInput`].
    pub fn align_x(mut self, alignment: impl Into<alignment::Horizontal>) -> Self {
        self.alignment = alignment.into();
        self
    }

    /// Sets the style of the [`TextInput`].
    #[must_use]
    pub fn style(mut self, style: impl Fn(&Theme, Status) -> Style + 'a) -> Self
    where
        Theme::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        self.class = (Box::new(style) as StyleFn<'a, Theme>).into();
        self
    }

    /// Sets the style class of the [`TextInput`].
    #[must_use]
    pub fn class(mut self, class: impl Into<Theme::Class<'a>>) -> Self {
        self.class = class.into();
        self
    }

    fn eye_bounds(&self, layout: Layout<'_>) -> Option<Rectangle> {
        self.on_toggle_secure.as_ref()?;

        let mut children = layout.children();
        children.next()?;

        if self.icon.is_some() {
            children.next()?;
        }

        if self.label_text.is_some() {
            children.next()?;
        }

        children.next().map(|eye| eye.bounds())
    }

    fn show_clear(&self, value: &Value) -> bool {
        self.clearable && self.on_input.is_some() && !value.is_empty()
    }

    fn clear_bounds(&self, layout: Layout<'_>) -> Option<Rectangle> {
        if !self.show_clear(&self.value) {
            return None;
        }

        let mut children = layout.children();
        children.next()?;

        if self.icon.is_some() {
            children.next()?;
        }

        if self.label_text.is_some() {
            children.next()?;
        }

        if self.on_toggle_secure.is_some() {
            children.next()?;
        }

        children.next().map(|clear| clear.bounds())
    }

    fn icon_bounds(&self, layout: Layout<'_>) -> Option<Rectangle> {
        self.icon.as_ref()?;

        let mut children = layout.children();
        children.next()?;

        children.next().map(|icon| icon.bounds())
    }

    /// Lays out the [`TextInput`], overriding its [`Value`] if provided.
    ///
    /// [`Renderer`]: text::Renderer
    pub fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
        value: Option<&Value>,
    ) -> layout::Node {
        let state = tree.state.downcast_mut::<State<Renderer::Paragraph>>();
        let value = value.unwrap_or(&self.value);

        let font = self.font.unwrap_or_else(|| renderer.default_font());
        let text_size = self.size.unwrap_or_else(|| renderer.default_size());
        let padding = self.padding.fit(Size::ZERO, limits.max());
        let height = self.line_height.to_absolute(text_size);

        let limits = limits.width(self.width).shrink(padding);
        let text_bounds = limits.resolve(self.width, height, Size::ZERO);

        let placeholder_text = Text {
            font,
            line_height: self.line_height,
            content: self.placeholder.as_str(),
            bounds: Size::new(f32::INFINITY, text_bounds.height),
            size: text_size,
            align_x: text::Alignment::Default,
            align_y: alignment::Vertical::Center,
            shaping: text::Shaping::Advanced,
            wrapping: text::Wrapping::default(),
        };

        let _ = state.placeholder.update(placeholder_text);

        let secure_value = self.is_secure.then(|| value.secure());
        let value = secure_value.as_ref().unwrap_or(value);

        let _ = state.value.update(Text {
            content: &value.to_string(),
            ..placeholder_text
        });

        let eye_size = EYE_ICON_SIZE.min(text_bounds.height + padding.top + padding.bottom);
        let has_eye = self.on_toggle_secure.is_some();
        let eye_reserved = if has_eye {
            eye_size + EYE_ICON_SPACING
        } else {
            0.0
        };

        let clear_size = CLEAR_ICON_SIZE.min(text_bounds.height + padding.top + padding.bottom);
        let has_clear = self.show_clear(value);
        let clear_reserved = if has_clear {
            clear_size + CLEAR_ICON_SPACING
        } else {
            0.0
        };

        let icon_width = if let Some(icon) = &self.icon {
            let mut content = [0; 4];

            let icon_text = Text {
                line_height: self.line_height,
                content: icon.code_point.encode_utf8(&mut content) as &_,
                font: icon.font,
                size: icon.size.unwrap_or_else(|| renderer.default_size()),
                bounds: Size::new(f32::INFINITY, text_bounds.height),
                align_x: text::Alignment::Center,
                align_y: alignment::Vertical::Center,
                shaping: text::Shaping::Advanced,
                wrapping: text::Wrapping::default(),
            };

            let _ = state.icon.update(icon_text);

            Some(state.icon.min_width())
        } else {
            None
        };

        let mut text_position = Point::new(padding.left, padding.top);
        let mut text_width = text_bounds.width;
        let mut children = Vec::with_capacity(6);

        let left_icon_node = match (&self.icon, icon_width) {
            (Some(icon), Some(icon_width)) if matches!(icon.side, Side::Left) => {
                text_width -= icon_width + icon.spacing;
                text_position.x += icon_width + icon.spacing;

                Some(
                    layout::Node::new(Size::new(icon_width, text_bounds.height))
                        .move_to(Point::new(padding.left, padding.top)),
                )
            }
            _ => None,
        };

        let right_icon_node = match (&self.icon, icon_width) {
            (Some(icon), Some(icon_width)) if matches!(icon.side, Side::Right) => {
                text_width -= icon_width + icon.spacing;

                Some(
                    layout::Node::new(Size::new(icon_width, text_bounds.height)).move_to(
                        Point::new(
                            padding.left + text_bounds.width
                                - icon_width
                                - eye_reserved
                                - clear_reserved,
                            padding.top,
                        ),
                    ),
                )
            }
            _ => None,
        };

        let eye_node = if has_eye {
            text_width -= eye_reserved;

            let eye_y = padding.top + (text_bounds.height - eye_size) / 2.0;

            Some(
                layout::Node::new(Size::new(eye_size, eye_size)).move_to(Point::new(
                    padding.left + text_bounds.width - eye_size,
                    eye_y,
                )),
            )
        } else {
            None
        };

        let clear_node = if has_clear {
            text_width -= clear_reserved;

            let clear_y = padding.top + (text_bounds.height - clear_size) / 2.0;

            Some(
                layout::Node::new(Size::new(clear_size, clear_size)).move_to(Point::new(
                    padding.left + text_bounds.width - eye_reserved - clear_size,
                    clear_y,
                )),
            )
        } else {
            None
        };

        let text_node = layout::Node::new(Size::new(text_width.max(0.0), text_bounds.height))
            .move_to(text_position);
        state.text_bounds = Some(text_node.bounds());
        children.push(text_node);

        if let Some(icon_node) = left_icon_node {
            children.push(icon_node);
        }

        if let Some(icon_node) = right_icon_node {
            children.push(icon_node);
        }

        if let Some(label_text) = self.label_text {
            let text = Text {
                font: self.font.unwrap_or_else(|| renderer.default_font()),
                line_height: self.line_height,
                content: label_text,
                bounds: Size::new(f32::INFINITY, LABEL_TEXT_SIZE.into()),
                size: LABEL_TEXT_SIZE,
                align_x: text::Alignment::Default,
                align_y: alignment::Vertical::Center,
                shaping: text::Shaping::Advanced,
                wrapping: text::Wrapping::default(),
            };
            let _ = state.label.update(text);
            let label_bounds = state.label.min_bounds();
            children.push(layout::Node::new(label_bounds).move_to(LABEL_TEXT_OFFSET));
        }

        if let Some(eye_node) = eye_node {
            children.push(eye_node);
        }

        if let Some(clear_node) = clear_node {
            children.push(clear_node);
        }

        layout::Node::with_children(text_bounds.expand(padding), children)
    }

    fn input_method<'b>(
        &self,
        state: &'b State<Renderer::Paragraph>,
        layout: Layout<'_>,
        value: &Value,
    ) -> InputMethod<&'b str> {
        let Some(Focus {
            is_window_focused: true,
            ..
        }) = &state.is_focused
        else {
            return InputMethod::Disabled;
        };

        let secure_value = self.is_secure.then(|| value.secure());
        let value = secure_value.as_ref().unwrap_or(value);

        let text_bounds = layout.children().next().unwrap().bounds();

        let caret_index = match state.cursor.state(value) {
            cursor::State::Index(position) => position,
            cursor::State::Selection { start, end } => start.min(end),
        };

        let text = state.value.raw();
        let (cursor_x, scroll_offset) =
            measure_cursor_and_scroll_offset(text, text_bounds, caret_index);

        let alignment_offset =
            alignment_offset(text_bounds.width, text.min_width(), self.alignment);

        let x = (text_bounds.x + cursor_x).floor() - scroll_offset + alignment_offset;

        InputMethod::Enabled {
            cursor: Rectangle::new(
                Point::new(x, text_bounds.y),
                Size::new(1.0, text_bounds.height),
            ),
            purpose: if self.is_secure {
                input_method::Purpose::Secure
            } else {
                input_method::Purpose::Normal
            },
            preedit: state.preedit.as_ref().map(input_method::Preedit::as_ref),
        }
    }

    /// Draws the [`TextInput`] with the given [`Renderer`], overriding its
    /// [`Value`] if provided.
    ///
    /// [`Renderer`]: text::Renderer
    pub fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        value: Option<&Value>,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State<Renderer::Paragraph>>();
        let value = value.unwrap_or(&self.value);
        let is_disabled = self.on_input.is_none();

        let secure_value = self.is_secure.then(|| value.secure());
        let value = secure_value.as_ref().unwrap_or(value);

        let bounds = layout.bounds();

        let mut children_layout = layout.children();
        let text_bounds = children_layout.next().unwrap().bounds();

        let style = theme.style(&self.class, self.last_status.unwrap_or(Status::Disabled));

        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: iced::Border {
                    color: match self.last_status.unwrap_or(Status::Disabled) {
                        Status::Active | Status::Hovered => {
                            if self.error {
                                self.theme.error()
                            } else {
                                self.theme.outline()
                            }
                        }
                        Status::Focused { is_hovered: _ } => {
                            if self.error {
                                self.theme.error()
                            } else {
                                self.theme.primary()
                            }
                        }
                        Status::Disabled => self.theme.outline().scale_alpha(DIM_ALPHA),
                    },
                    width: BORDER_WIDTH,
                    radius: Radius::from(4.0),
                },
                ..renderer::Quad::default()
            },
            Color::TRANSPARENT,
        );

        if self.icon.is_some() {
            let icon_layout = children_layout.next().unwrap();

            let icon = state.icon.raw();

            renderer.fill_paragraph(
                icon,
                icon_layout.bounds().anchor(
                    icon.min_bounds(),
                    Alignment::Center,
                    Alignment::Center,
                ),
                style.icon,
                *viewport,
            );
        }

        if self.label_text.is_some() {
            let label_bounds = children_layout.next().unwrap().bounds();

            if let Some(Status::Focused { is_hovered: _ }) = self.last_status {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle {
                            height: BORDER_WIDTH,
                            width: label_bounds.width + 2.0 * LABEL_BACKGROUND_PADDING,
                            x: label_bounds.x - LABEL_BACKGROUND_PADDING,
                            y: label_bounds.y - LABEL_TEXT_OFFSET.y,
                        },
                        ..Default::default()
                    },
                    self.background_color.unwrap(),
                );

                renderer.fill_text(
                    Text {
                        font: self.font.unwrap_or_else(|| renderer.default_font()),
                        line_height: self.line_height,
                        content: self.label_text.unwrap_or_default().to_string(),
                        bounds: Size::new(f32::INFINITY, LABEL_TEXT_SIZE.into()),
                        size: LABEL_TEXT_SIZE,
                        align_x: text::Alignment::Default,
                        align_y: alignment::Vertical::Center,
                        shaping: text::Shaping::Advanced,
                        wrapping: text::Wrapping::default(),
                    },
                    label_bounds.position(),
                    if self.error {
                        self.theme.error()
                    } else {
                        self.theme.primary()
                    },
                    *viewport,
                );
            }
        }

        if self.on_toggle_secure.is_some() {
            let eye_bounds = children_layout.next().unwrap().bounds();

            let eye_progress = state
                .eye_spring
                .as_ref()
                .map(|spring| spring.value().clamp(0.0, 1.0))
                .unwrap_or(if self.is_secure { 0.0 } else { 1.0 });

            let eye_color = if is_disabled {
                style.icon.scale_alpha(DIM_ALPHA)
            } else {
                style.icon
            };

            if eye_progress < 0.999 {
                renderer.draw_svg(
                    svg::Svg {
                        handle: common_icons::VISIBILITY_OFF.clone(),
                        color: Some(eye_color),
                        rotation: Radians(0.0),
                        opacity: 1.0 - eye_progress,
                    },
                    eye_bounds,
                    eye_bounds,
                );
            }

            if eye_progress > 0.001 {
                renderer.draw_svg(
                    svg::Svg {
                        handle: common_icons::VISIBILITY.clone(),
                        color: Some(eye_color),
                        rotation: Radians(0.0),
                        opacity: eye_progress,
                    },
                    eye_bounds,
                    eye_bounds,
                );
            }
        }

        if self.show_clear(value) {
            let clear_bounds = children_layout.next().unwrap().bounds();

            let clear_color = if is_disabled {
                style.icon.scale_alpha(DIM_ALPHA)
            } else {
                style.icon
            };

            renderer.draw_svg(
                svg::Svg {
                    handle: common_icons::CLOSE.clone(),
                    color: Some(clear_color),
                    rotation: Radians(0.0),
                    opacity: 1.0,
                },
                clear_bounds,
                clear_bounds,
            );
        }

        let text = value.to_string();

        let (cursor, offset, is_selecting) = if let Some(focus) = state
            .is_focused
            .as_ref()
            .filter(|focus| focus.is_window_focused)
        {
            match state.cursor.state(value) {
                cursor::State::Index(position) => {
                    let (text_value_width, offset) =
                        measure_cursor_and_scroll_offset(state.value.raw(), text_bounds, position);

                    let is_cursor_visible = !is_disabled && focus.is_caret_visible();

                    let cursor = if is_cursor_visible {
                        Some((
                            renderer::Quad {
                                bounds: Rectangle {
                                    x: (text_bounds.x + text_value_width).floor(),
                                    y: text_bounds.y,
                                    width: 1.0,
                                    height: text_bounds.height,
                                },
                                ..renderer::Quad::default()
                            },
                            if self.error {
                                self.theme.error()
                            } else {
                                self.theme.primary()
                            },
                        ))
                    } else {
                        None
                    };

                    (cursor, offset, false)
                }
                cursor::State::Selection { start, end } => {
                    let left = start.min(end);
                    let right = end.max(start);

                    let (left_position, left_offset) =
                        measure_cursor_and_scroll_offset(state.value.raw(), text_bounds, left);

                    let (right_position, right_offset) =
                        measure_cursor_and_scroll_offset(state.value.raw(), text_bounds, right);

                    let width = right_position - left_position;

                    (
                        Some((
                            renderer::Quad {
                                bounds: Rectangle {
                                    x: text_bounds.x + left_position,
                                    y: text_bounds.y,
                                    width,
                                    height: text_bounds.height,
                                },
                                ..renderer::Quad::default()
                            },
                            if self.error {
                                self.theme.error_container()
                            } else {
                                self.theme.inverse_primary()
                            },
                        )),
                        if end == right {
                            right_offset
                        } else {
                            left_offset
                        },
                        true,
                    )
                }
            }
        } else {
            (None, 0.0, false)
        };

        let draw = |renderer: &mut Renderer, viewport| {
            let paragraph = if text.is_empty()
                && state
                    .preedit
                    .as_ref()
                    .map(|preedit| preedit.content.is_empty())
                    .unwrap_or(true)
            {
                state.placeholder.raw()
            } else {
                state.value.raw()
            };

            let alignment_offset =
                alignment_offset(text_bounds.width, paragraph.min_width(), self.alignment);

            if let Some((cursor, color)) = cursor {
                renderer.with_translation(
                    Vector::new(alignment_offset - offset, 0.0),
                    |renderer| {
                        renderer.fill_quad(cursor, color);
                    },
                );
            } else {
                renderer.with_translation(Vector::ZERO, |_| {});
            }

            renderer.fill_paragraph(
                paragraph,
                text_bounds.anchor(paragraph.min_bounds(), Alignment::Start, Alignment::Center)
                    + Vector::new(alignment_offset - offset, 0.0),
                if text.is_empty() {
                    if self.error {
                        self.theme.error().scale_alpha(DIM_ALPHA)
                    } else {
                        self.theme.on_surface_variant()
                    }
                } else {
                    self.theme.on_surface()
                },
                viewport,
            );
        };

        if is_selecting {
            renderer.with_layer(text_bounds, |renderer| draw(renderer, *viewport));
        } else {
            draw(renderer, text_bounds);
        }
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for TextInput<'a, Message, Theme, Renderer>
where
    Message: Clone,
    Theme: Catalog,
    Renderer: text::Renderer + svg::Renderer + 'a,
    Renderer::Font: From<iced::Font>,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State<Renderer::Paragraph>>()
    }

    fn state(&self) -> tree::State {
        let mut state = State::<Renderer::Paragraph>::new();

        if self.on_toggle_secure.is_some() {
            let now = Instant::now();
            let value = if self.is_secure { 0.0 } else { 1.0 };

            state.eye_spring = Some(SpringValue::new(
                value,
                motion::fast_effects(motion::Scheme::default()),
                now,
            ));
        }

        tree::State::new(state)
    }

    fn diff(&self, tree: &mut Tree) {
        let state = tree.state.downcast_mut::<State<Renderer::Paragraph>>();

        let value = self.value.to_string();

        if state.history.last_value.as_deref() != Some(value.as_str()) {
            state.history.reset(&value);
        }

        // Stop pasting if input becomes disabled
        if self.on_input.is_none() {
            state.is_pasting = None;
            state.context_menu = None;
        }

        if state.eye_spring.is_none() && self.on_toggle_secure.is_some() {
            let now = Instant::now();
            let value = if self.is_secure { 0.0 } else { 1.0 };

            state.eye_spring = Some(SpringValue::new(
                value,
                motion::fast_effects(motion::Scheme::default()),
                now,
            ));
        }
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: Length::Shrink,
        }
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.layout(tree, renderer, limits, None)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        _renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        let state = tree.state.downcast_mut::<State<Renderer::Paragraph>>();

        operation.text_input(self.id.as_ref(), layout.bounds(), state);
        operation.focusable(self.id.as_ref(), layout.bounds(), state);
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
        _viewport: &Rectangle,
    ) {
        {
            let now = Instant::now();
            let state = state::<Renderer>(tree);

            if let Some(spring) = &mut state.eye_spring {
                let target = if self.is_secure { 0.0 } else { 1.0 };
                spring.set_target(target, now);
                spring.step(now);

                if !spring.is_at_rest() {
                    shell.request_redraw();
                }
            }
        }

        {
            let state = state::<Renderer>(tree);

            // The eye, clear and icon buttons are drawn by hand, so their tooltips are driven
            // manually instead of wrapping them in a `Tooltip`.
            state.eye_hover.update(
                event,
                cursor,
                self.eye_bounds(layout),
                self.tooltips_enabled && self.on_toggle_secure.is_some(),
                tooltip::DEFAULT_DELAY,
                shell,
            );
            state.clear_hover.update(
                event,
                cursor,
                self.clear_bounds(layout),
                self.tooltips_enabled && self.show_clear(&self.value),
                tooltip::DEFAULT_DELAY,
                shell,
            );
            state.icon_hover.update(
                event,
                cursor,
                self.icon_bounds(layout),
                self.icon.is_some() && self.icon_tooltip.is_some(),
                tooltip::DEFAULT_DELAY,
                shell,
            );
        }

        let text_bounds = layout
            .children()
            .next()
            .map(|text_layout| text_layout.bounds());
        let update_cache = |state, value| {
            replace_paragraph(
                renderer,
                state,
                text_bounds,
                value,
                self.font,
                self.size,
                self.line_height,
            );
        };

        match &event {
            Event::Window(
                window::Event::Moved(_)
                | window::Event::Resized(_)
                | window::Event::Rescaled(_)
                | window::Event::Closed,
            ) => {
                if state::<Renderer>(tree).context_menu.take().is_some() {
                    shell.request_redraw();
                }
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                let state = state::<Renderer>(tree);

                if self
                    .eye_bounds(layout)
                    .is_some_and(|eye_bounds| cursor.is_over(eye_bounds))
                {
                    if let Some(message) = self.on_toggle_secure.clone() {
                        shell.publish(message);
                    }

                    shell.capture_event();
                    return;
                }

                if self
                    .clear_bounds(layout)
                    .is_some_and(|clear_bounds| cursor.is_over(clear_bounds))
                {
                    let now = Instant::now();

                    state.is_focused = Some(Focus::new(now));

                    state.cursor.select_all(&self.value);

                    let contents = edit_value(
                        &mut self.value,
                        &mut state.cursor,
                        &mut state.history,
                        EditKind::Other,
                        |editor| editor.delete(),
                    );

                    if let Some(on_input) = &self.on_input {
                        shell.publish((on_input)(contents));
                    }

                    state.is_pasting = None;
                    state.history.break_group();

                    update_cache(state, &self.value);

                    shell.request_redraw();
                    shell.capture_event();
                    return;
                }

                let cursor_before = state.cursor;

                let click_position = cursor.position_over(layout.bounds());

                state.is_focused = if click_position.is_some() {
                    let now = Instant::now();

                    Some(Focus::new(now))
                } else {
                    None
                };

                if let Some(cursor_position) = click_position {
                    let text_layout = layout.children().next().unwrap();

                    let target = {
                        let text_bounds = text_layout.bounds();

                        let alignment_offset = alignment_offset(
                            text_bounds.width,
                            state.value.raw().min_width(),
                            self.alignment,
                        );

                        cursor_position.x - text_bounds.x - alignment_offset
                    };

                    let click =
                        mouse::Click::new(cursor_position, mouse::Button::Left, state.last_click);

                    match click.kind() {
                        click::Kind::Single => {
                            let position = if target > 0.0 {
                                let value = if self.is_secure {
                                    self.value.secure()
                                } else {
                                    self.value.clone()
                                };

                                find_cursor_position(text_layout.bounds(), &value, state, target)
                            } else {
                                None
                            }
                            .unwrap_or(0);

                            if state.keyboard_modifiers.shift() {
                                state
                                    .cursor
                                    .select_range(state.cursor.start(&self.value), position);
                            } else {
                                state.cursor.move_to(position);
                            }

                            state.is_dragging = Some(Drag::Select);
                        }
                        click::Kind::Double => {
                            if self.is_secure {
                                state.cursor.select_all(&self.value);

                                state.is_dragging = None;
                            } else {
                                let position = find_cursor_position(
                                    text_layout.bounds(),
                                    &self.value,
                                    state,
                                    target,
                                )
                                .unwrap_or(0);

                                state.cursor.select_range(
                                    self.value.previous_start_of_word(position),
                                    self.value.next_end_of_word(position),
                                );

                                state.is_dragging = Some(Drag::SelectWords { anchor: position });
                            }
                        }
                        click::Kind::Triple => {
                            state.cursor.select_all(&self.value);
                            state.is_dragging = None;
                        }
                    }

                    state.last_click = Some(click);

                    if cursor_before != state.cursor {
                        shell.request_redraw();
                    }

                    shell.capture_event();
                }

                state.history.break_group();
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)) => {
                let state = state::<Renderer>(tree);

                if self.on_input.is_none() {
                    return;
                }

                let Some(position) = cursor.position_over(layout.bounds()) else {
                    if state.context_menu.take().is_some() {
                        shell.request_redraw();
                    }

                    return;
                };

                let now = Instant::now();
                state.is_focused = Some(Focus::new(now));

                if state.cursor.selection(&self.value).is_none() {
                    let text_layout = layout.children().next().unwrap();
                    let text_bounds = text_layout.bounds();

                    let alignment_offset = alignment_offset(
                        text_bounds.width,
                        state.value.raw().min_width(),
                        self.alignment,
                    );

                    let target = position.x - text_bounds.x - alignment_offset;

                    if target > 0.0 {
                        let value = if self.is_secure {
                            self.value.secure()
                        } else {
                            self.value.clone()
                        };

                        if let Some(position) =
                            find_cursor_position(text_bounds, &value, state, target)
                        {
                            state.cursor.move_to(position);
                        }
                    }
                }

                let is_already_opened = state
                    .context_menu
                    .is_some_and(|menu| menu.position == position);

                if !is_already_opened {
                    state.context_menu = Some(ContextMenu::new(position));
                }

                state.history.break_group();

                shell.request_redraw();
                shell.capture_event();
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerLifted { .. })
            | Event::Touch(touch::Event::FingerLost { .. }) => {
                state::<Renderer>(tree).is_dragging = None;
            }
            Event::Mouse(mouse::Event::CursorMoved { position })
            | Event::Touch(touch::Event::FingerMoved { position, .. }) => {
                let state = state::<Renderer>(tree);

                if let Some(is_dragging) = &state.is_dragging {
                    let text_layout = layout.children().next().unwrap();

                    let target = {
                        let text_bounds = text_layout.bounds();

                        let alignment_offset = alignment_offset(
                            text_bounds.width,
                            state.value.raw().min_width(),
                            self.alignment,
                        );

                        position.x - text_bounds.x - alignment_offset
                    };

                    let value = if self.is_secure {
                        self.value.secure()
                    } else {
                        self.value.clone()
                    };

                    let position =
                        find_cursor_position(text_layout.bounds(), &value, state, target)
                            .unwrap_or(0);

                    let selection_before = state.cursor.selection(&value);

                    match is_dragging {
                        Drag::Select => {
                            state
                                .cursor
                                .select_range(state.cursor.start(&value), position);
                        }
                        Drag::SelectWords { anchor } => {
                            if position < *anchor {
                                state.cursor.select_range(
                                    self.value.previous_start_of_word(position),
                                    self.value.next_end_of_word(*anchor),
                                );
                            } else {
                                state.cursor.select_range(
                                    self.value.previous_start_of_word(*anchor),
                                    self.value.next_end_of_word(position),
                                );
                            }
                        }
                    }

                    if let Some(focus) = &mut state.is_focused {
                        focus.updated_at = Instant::now();
                    }

                    if selection_before != state.cursor.selection(&value) {
                        shell.request_redraw();
                    }

                    state.history.break_group();

                    shell.capture_event();
                }
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key,
                text,
                modified_key,
                physical_key,
                ..
            }) => {
                let state = state::<Renderer>(tree);

                if let Some(focus) = &mut state.is_focused {
                    let modifiers = state.keyboard_modifiers;

                    match key.to_latin(*physical_key) {
                        Some(c)
                            if state.keyboard_modifiers.command()
                                && matches!(c.to_ascii_lowercase(), 'z' | 'y') =>
                        {
                            let Some(on_input) = &self.on_input else {
                                return;
                            };

                            let is_redo =
                                c.eq_ignore_ascii_case(&'y') || state.keyboard_modifiers.shift();

                            let current = self.value.to_string();
                            let cursor = state.cursor;

                            let target = if is_redo {
                                state.history.redo(&current, cursor)
                            } else {
                                state.history.undo(&current, cursor)
                            };

                            if let Some(snapshot) = target {
                                self.value = Value::new(&snapshot.value);
                                state.cursor = snapshot.cursor;
                                state.is_pasting = None;

                                shell.publish((on_input)(snapshot.value));
                                shell.request_redraw();

                                focus.updated_at = Instant::now();
                                update_cache(state, &self.value);
                            }

                            shell.capture_event();
                            return;
                        }
                        Some('c') if state.keyboard_modifiers.command() && !self.is_secure => {
                            if let Some((start, end)) = state.cursor.selection(&self.value) {
                                clipboard.write(
                                    clipboard::Kind::Standard,
                                    self.value.select(start, end).to_string(),
                                );
                            }

                            shell.capture_event();
                            return;
                        }
                        Some('x') if state.keyboard_modifiers.command() && !self.is_secure => {
                            let Some(on_input) = &self.on_input else {
                                return;
                            };

                            if let Some((start, end)) = state.cursor.selection(&self.value) {
                                clipboard.write(
                                    clipboard::Kind::Standard,
                                    self.value.select(start, end).to_string(),
                                );
                            }

                            let contents = edit_value(
                                &mut self.value,
                                &mut state.cursor,
                                &mut state.history,
                                EditKind::Other,
                                |editor| editor.delete(),
                            );

                            shell.publish((on_input)(contents));
                            shell.capture_event();

                            focus.updated_at = Instant::now();
                            update_cache(state, &self.value);
                            return;
                        }
                        Some('v')
                            if state.keyboard_modifiers.command()
                                && !state.keyboard_modifiers.alt() =>
                        {
                            let Some(on_input) = &self.on_input else {
                                return;
                            };

                            let content = match state.is_pasting.take() {
                                Some(content) => content,
                                None => {
                                    let content: String = clipboard
                                        .read(clipboard::Kind::Standard)
                                        .unwrap_or_default()
                                        .chars()
                                        .filter(|c| !c.is_control())
                                        .collect();

                                    Value::new(&content)
                                }
                            };

                            let contents = edit_value(
                                &mut self.value,
                                &mut state.cursor,
                                &mut state.history,
                                EditKind::Other,
                                |editor| editor.paste(content.clone()),
                            );

                            let message = if let Some(paste) = &self.on_paste {
                                (paste)(contents)
                            } else {
                                (on_input)(contents)
                            };
                            shell.publish(message);
                            shell.capture_event();

                            state.is_pasting = Some(content);
                            focus.updated_at = Instant::now();
                            update_cache(state, &self.value);
                            return;
                        }
                        Some('a') if state.keyboard_modifiers.command() => {
                            let cursor_before = state.cursor;

                            state.history.break_group();
                            state.cursor.select_all(&self.value);

                            if cursor_before != state.cursor {
                                focus.updated_at = Instant::now();

                                shell.request_redraw();
                            }

                            shell.capture_event();
                            return;
                        }
                        _ => {}
                    }

                    if let Some(text) = text {
                        let Some(on_input) = &self.on_input else {
                            return;
                        };

                        state.is_pasting = None;

                        if let Some(c) = text.chars().next().filter(|c| !c.is_control()) {
                            let contents = edit_value(
                                &mut self.value,
                                &mut state.cursor,
                                &mut state.history,
                                EditKind::Insert,
                                |editor| editor.insert(c),
                            );

                            shell.publish((on_input)(contents));
                            shell.capture_event();

                            focus.updated_at = Instant::now();
                            update_cache(state, &self.value);
                            return;
                        }
                    }

                    #[cfg(target_os = "macos")]
                    let macos_shortcut = convert_macos_shortcut(key, modifiers);

                    #[cfg(target_os = "macos")]
                    let modified_key = macos_shortcut.as_ref().unwrap_or(modified_key);

                    match modified_key.as_ref() {
                        keyboard::Key::Named(key::Named::Enter) => {
                            if let Some(on_submit) = self.on_submit.clone() {
                                shell.publish(on_submit);
                                shell.capture_event();
                            }
                        }
                        keyboard::Key::Named(key::Named::Backspace) => {
                            let Some(on_input) = &self.on_input else {
                                return;
                            };

                            if state.cursor.selection(&self.value).is_none() {
                                if (self.is_secure && modifiers.jump()) || modifiers.macos_command()
                                {
                                    state
                                        .cursor
                                        .select_range(state.cursor.start(&self.value), 0);
                                } else if modifiers.jump() {
                                    state.cursor.select_left_by_words(&self.value);
                                }
                            }

                            let contents = edit_value(
                                &mut self.value,
                                &mut state.cursor,
                                &mut state.history,
                                EditKind::Delete,
                                |editor| editor.backspace(),
                            );

                            shell.publish((on_input)(contents));
                            shell.capture_event();

                            focus.updated_at = Instant::now();
                            update_cache(state, &self.value);
                        }
                        keyboard::Key::Named(key::Named::Delete) => {
                            let Some(on_input) = &self.on_input else {
                                return;
                            };

                            if state.cursor.selection(&self.value).is_none() {
                                if (self.is_secure && modifiers.jump()) || modifiers.macos_command()
                                {
                                    state.cursor.select_range(
                                        state.cursor.start(&self.value),
                                        self.value.len(),
                                    );
                                } else if modifiers.jump() {
                                    state.cursor.select_right_by_words(&self.value);
                                }
                            }

                            let contents = edit_value(
                                &mut self.value,
                                &mut state.cursor,
                                &mut state.history,
                                EditKind::Delete,
                                |editor| editor.delete(),
                            );

                            shell.publish((on_input)(contents));
                            shell.capture_event();

                            focus.updated_at = Instant::now();
                            update_cache(state, &self.value);
                        }
                        keyboard::Key::Named(key::Named::Home) => {
                            let cursor_before = state.cursor;

                            state.history.break_group();

                            if modifiers.shift() {
                                state
                                    .cursor
                                    .select_range(state.cursor.start(&self.value), 0);
                            } else {
                                state.cursor.move_to(0);
                            }

                            if cursor_before != state.cursor {
                                focus.updated_at = Instant::now();

                                shell.request_redraw();
                            }

                            shell.capture_event();
                        }
                        keyboard::Key::Named(key::Named::End) => {
                            let cursor_before = state.cursor;

                            state.history.break_group();

                            if modifiers.shift() {
                                state.cursor.select_range(
                                    state.cursor.start(&self.value),
                                    self.value.len(),
                                );
                            } else {
                                state.cursor.move_to(self.value.len());
                            }

                            if cursor_before != state.cursor {
                                focus.updated_at = Instant::now();

                                shell.request_redraw();
                            }

                            shell.capture_event();
                        }
                        keyboard::Key::Named(key::Named::ArrowLeft) => {
                            let cursor_before = state.cursor;

                            state.history.break_group();

                            if (self.is_secure && modifiers.jump()) || modifiers.macos_command() {
                                if modifiers.shift() {
                                    state
                                        .cursor
                                        .select_range(state.cursor.start(&self.value), 0);
                                } else {
                                    state.cursor.move_to(0);
                                }
                            } else if modifiers.jump() {
                                if modifiers.shift() {
                                    state.cursor.select_left_by_words(&self.value);
                                } else {
                                    state.cursor.move_left_by_words(&self.value);
                                }
                            } else if modifiers.shift() {
                                state.cursor.select_left(&self.value);
                            } else {
                                state.cursor.move_left(&self.value);
                            }

                            if cursor_before != state.cursor {
                                focus.updated_at = Instant::now();

                                shell.request_redraw();
                            }

                            shell.capture_event();
                        }
                        keyboard::Key::Named(key::Named::ArrowRight) => {
                            let cursor_before = state.cursor;

                            state.history.break_group();

                            if (self.is_secure && modifiers.jump()) || modifiers.macos_command() {
                                if modifiers.shift() {
                                    state.cursor.select_range(
                                        state.cursor.start(&self.value),
                                        self.value.len(),
                                    );
                                } else {
                                    state.cursor.move_to(self.value.len());
                                }
                            } else if modifiers.jump() {
                                if modifiers.shift() {
                                    state.cursor.select_right_by_words(&self.value);
                                } else {
                                    state.cursor.move_right_by_words(&self.value);
                                }
                            } else if modifiers.shift() {
                                state.cursor.select_right(&self.value);
                            } else {
                                state.cursor.move_right(&self.value);
                            }

                            if cursor_before != state.cursor {
                                focus.updated_at = Instant::now();

                                shell.request_redraw();
                            }

                            shell.capture_event();
                        }
                        keyboard::Key::Named(key::Named::Escape) => {
                            state.history.break_group();
                            state.is_focused = None;
                            state.is_dragging = None;
                            state.is_pasting = None;
                            state.context_menu = None;

                            state.keyboard_modifiers = keyboard::Modifiers::default();

                            shell.request_redraw();
                            shell.capture_event();
                        }
                        _ => {}
                    }
                }
            }
            Event::Keyboard(keyboard::Event::KeyReleased { key, .. }) => {
                let state = state::<Renderer>(tree);

                if state.is_focused.is_some()
                    && let keyboard::Key::Character("v") = key.as_ref()
                {
                    state.is_pasting = None;

                    shell.capture_event();
                }

                state.is_pasting = None;
            }
            Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) => {
                let state = state::<Renderer>(tree);

                state.keyboard_modifiers = *modifiers;
            }
            Event::InputMethod(event) => match event {
                input_method::Event::Opened | input_method::Event::Closed => {
                    let state = state::<Renderer>(tree);

                    state.preedit = matches!(event, input_method::Event::Opened)
                        .then(input_method::Preedit::new);

                    shell.request_redraw();
                }
                input_method::Event::Preedit(content, selection) => {
                    let state = state::<Renderer>(tree);

                    if state.is_focused.is_some() {
                        state.preedit = Some(input_method::Preedit {
                            content: content.to_owned(),
                            selection: selection.clone(),
                            text_size: self.size,
                        });

                        shell.request_redraw();
                    }
                }
                input_method::Event::Commit(text) => {
                    let state = state::<Renderer>(tree);

                    if let Some(focus) = &mut state.is_focused {
                        let Some(on_input) = &self.on_input else {
                            return;
                        };

                        let contents = edit_value(
                            &mut self.value,
                            &mut state.cursor,
                            &mut state.history,
                            EditKind::Other,
                            |editor| editor.paste(Value::new(text)),
                        );

                        focus.updated_at = Instant::now();
                        state.is_pasting = None;

                        shell.publish((on_input)(contents));
                        shell.capture_event();

                        update_cache(state, &self.value);
                    }
                }
            },
            Event::Window(window::Event::Unfocused) => {
                let state = state::<Renderer>(tree);

                state.history.break_group();

                if let Some(focus) = &mut state.is_focused {
                    focus.is_window_focused = false;
                }

                if state.context_menu.take().is_some() {
                    shell.request_redraw();
                }
            }
            Event::Window(window::Event::Focused) => {
                let state = state::<Renderer>(tree);

                if let Some(focus) = &mut state.is_focused {
                    focus.is_window_focused = true;
                    focus.updated_at = Instant::now();

                    shell.request_redraw();
                }
            }
            Event::Window(window::Event::RedrawRequested(now)) => {
                let state = state::<Renderer>(tree);

                if let Some(focus) = &mut state.is_focused
                    && focus.is_window_focused
                {
                    if matches!(state.cursor.state(&self.value), cursor::State::Index(_)) {
                        focus.now = *now;

                        let millis_until_redraw = focus.millis_until_blink();

                        shell.request_redraw_at(
                            *now + Duration::from_millis(millis_until_redraw as u64),
                        );
                    }

                    shell.request_input_method(&self.input_method(state, layout, &self.value));
                }
            }
            _ => {}
        }

        let state = state::<Renderer>(tree);
        let is_disabled = self.on_input.is_none();

        let status = if is_disabled {
            Status::Disabled
        } else if state.is_focused() {
            Status::Focused {
                is_hovered: cursor.is_over(layout.bounds()),
            }
        } else if cursor.is_over(layout.bounds()) {
            Status::Hovered
        } else {
            Status::Active
        };

        if let Event::Window(window::Event::RedrawRequested(_now)) = event {
            self.last_status = Some(status);
        } else if self
            .last_status
            .is_some_and(|last_status| status != last_status)
        {
            shell.request_redraw();
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.draw(tree, renderer, theme, layout, cursor, None, viewport);
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        _viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        let state = tree.state.downcast_mut::<State<Renderer::Paragraph>>();

        if state.context_menu.is_some() {
            let style = menu::Style::new(self.theme, false);
            return Some(overlay::Element::new(Box::new(ContextMenuOverlay {
                host: TextInputContext { input: self, state },
                style,
                font: None,
                translation,
            })));
        }

        let font = self.font.unwrap_or_else(|| renderer.default_font());
        let translate =
            |rect: Rectangle| Rectangle::new(rect.position() + translation, rect.size());

        // The eye button is icon-only, so spell out what it does.
        if self.tooltips_enabled
            && state.eye_hover.is_open()
            && let Some(bounds) = self.eye_bounds(layout)
        {
            let label = if self.is_secure {
                self.labels.show_password.clone()
            } else {
                self.labels.hide_password.clone()
            };

            return Some(tooltip::label_overlay(
                translate(bounds),
                label,
                Some(font),
                self.tooltip_style,
                tooltip::Position::Above,
                8.0,
                tooltip::DEFAULT_MAX_WIDTH,
                None,
            ));
        }

        if self.tooltips_enabled
            && state.clear_hover.is_open()
            && let Some(bounds) = self.clear_bounds(layout)
        {
            let label = self.labels.clear.clone();

            return Some(tooltip::label_overlay(
                translate(bounds),
                label,
                Some(font),
                self.tooltip_style,
                tooltip::Position::Above,
                8.0,
                tooltip::DEFAULT_MAX_WIDTH,
                None,
            ));
        }

        if state.icon_hover.is_open()
            && let Some(bounds) = self.icon_bounds(layout)
            && let Some(label) = self.icon_tooltip.clone()
        {
            return Some(tooltip::label_overlay(
                translate(bounds),
                label,
                Some(font),
                self.tooltip_style,
                tooltip::Position::Above,
                8.0,
                tooltip::DEFAULT_MAX_WIDTH,
                None,
            ));
        }

        None
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        if self
            .eye_bounds(layout)
            .is_some_and(|eye_bounds| cursor.is_over(eye_bounds))
        {
            return mouse::Interaction::Pointer;
        }

        if self
            .clear_bounds(layout)
            .is_some_and(|clear_bounds| cursor.is_over(clear_bounds))
        {
            return mouse::Interaction::Pointer;
        }

        if cursor.is_over(layout.bounds()) {
            if self.on_input.is_none() {
                mouse::Interaction::Idle
            } else {
                mouse::Interaction::Text
            }
        } else {
            mouse::Interaction::default()
        }
    }
}

crate::macros::impl_element_from!(
    ['a, Message, Theme, Renderer]
    TextInput<'a, Message, Theme, Renderer> => Element<'a, Message, Theme, Renderer>
    where
        Message: Clone + 'a,
        Theme: Catalog + 'a,
        Renderer: text::Renderer + svg::Renderer + 'a,
        Renderer::Font: From<iced::Font>,
);

/// The content of the [`Icon`].
#[derive(Debug, Clone)]
pub struct Icon<Font> {
    /// The font that will be used to display the `code_point`.
    pub font: Font,
    /// The unicode code point that will be used as the icon.
    pub code_point: char,
    /// The font size of the content.
    pub size: Option<Pixels>,
    /// The spacing between the [`Icon`] and the text in a [`TextInput`].
    pub spacing: f32,
    /// The side of a [`TextInput`] where to display the [`Icon`].
    pub side: Side,
}

/// The side of a [`TextInput`].
#[derive(Debug, Clone)]
pub enum Side {
    /// The left side of a [`TextInput`].
    Left,
    /// The right side of a [`TextInput`].
    Right,
}

/// The state of a [`TextInput`].
#[derive(Debug, Default, Clone)]
pub struct State<P: text::Paragraph> {
    value: paragraph::Plain<P>,
    placeholder: paragraph::Plain<P>,
    label: paragraph::Plain<P>,
    icon: paragraph::Plain<P>,
    is_focused: Option<Focus>,
    is_dragging: Option<Drag>,
    is_pasting: Option<Value>,
    preedit: Option<input_method::Preedit>,
    last_click: Option<mouse::Click>,
    cursor: Cursor,
    keyboard_modifiers: keyboard::Modifiers,
    eye_spring: Option<SpringValue<f32>>,
    context_menu: Option<ContextMenu>,
    text_bounds: Option<Rectangle>,
    history: History,
    eye_hover: tooltip::Hover,
    clear_hover: tooltip::Hover,
    icon_hover: tooltip::Hover,
    // TODO: Add stateful horizontal scrolling offset
}

fn state<Renderer: text::Renderer>(tree: &mut Tree) -> &mut State<Renderer::Paragraph> {
    tree.state.downcast_mut::<State<Renderer::Paragraph>>()
}

#[derive(Debug, Clone)]
enum Drag {
    Select,
    SelectWords { anchor: usize },
}

impl<P: text::Paragraph> State<P> {
    /// Creates a new [`State`], representing an unfocused [`TextInput`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns whether the [`TextInput`] is currently focused or not.
    pub fn is_focused(&self) -> bool {
        self.is_focused.is_some()
    }

    /// Returns the [`Cursor`] of the [`TextInput`].
    pub fn cursor(&self) -> Cursor {
        self.cursor
    }

    /// Focuses the [`TextInput`].
    pub fn focus(&mut self) {
        let now = Instant::now();

        self.is_focused = Some(Focus::new(now));

        self.move_cursor_to_end();
    }

    /// Unfocuses the [`TextInput`].
    pub fn unfocus(&mut self) {
        self.is_focused = None;
        self.history.break_group();
    }

    /// Moves the [`Cursor`] of the [`TextInput`] to the front of the input text.
    pub fn move_cursor_to_front(&mut self) {
        self.move_cursor_to(0);
    }

    /// Moves the [`Cursor`] of the [`TextInput`] to the end of the input text.
    pub fn move_cursor_to_end(&mut self) {
        self.move_cursor_to(usize::MAX);
    }

    /// Moves the [`Cursor`] of the [`TextInput`] to an arbitrary location.
    pub fn move_cursor_to(&mut self, position: usize) {
        self.history.break_group();
        self.cursor.move_to(position);
    }

    /// Selects all the content of the [`TextInput`].
    pub fn select_all(&mut self) {
        self.history.break_group();
        self.cursor.select_range(0, usize::MAX);
    }

    /// Selects the given range of the content of the [`TextInput`].
    pub fn select_range(&mut self, start: usize, end: usize) {
        self.history.break_group();
        self.cursor.select_range(start, end);
    }
}

impl<P: text::Paragraph> operation::Focusable for State<P> {
    fn is_focused(&self) -> bool {
        State::is_focused(self)
    }

    fn focus(&mut self) {
        State::focus(self);
    }

    fn unfocus(&mut self) {
        State::unfocus(self);
    }
}

impl<P: text::Paragraph> operation::TextInput for State<P> {
    fn text(&self) -> &str {
        if self.value.content().is_empty() {
            self.placeholder.content()
        } else {
            self.value.content()
        }
    }

    fn move_cursor_to_front(&mut self) {
        State::move_cursor_to_front(self);
    }

    fn move_cursor_to_end(&mut self) {
        State::move_cursor_to_end(self);
    }

    fn move_cursor_to(&mut self, position: usize) {
        State::move_cursor_to(self, position);
    }

    fn select_all(&mut self) {
        State::select_all(self);
    }

    fn select_range(&mut self, start: usize, end: usize) {
        State::select_range(self, start, end);
    }
}

fn offset<P: text::Paragraph>(text_bounds: Rectangle, value: &Value, state: &State<P>) -> f32 {
    if state.is_focused() {
        let cursor = state.cursor();

        let focus_position = match cursor.state(value) {
            cursor::State::Index(i) => i,
            cursor::State::Selection { end, .. } => end,
        };

        let (_, offset) =
            measure_cursor_and_scroll_offset(state.value.raw(), text_bounds, focus_position);

        offset
    } else {
        0.0
    }
}

fn measure_cursor_and_scroll_offset(
    paragraph: &impl text::Paragraph,
    text_bounds: Rectangle,
    cursor_index: usize,
) -> (f32, f32) {
    let grapheme_position = paragraph
        .grapheme_position(0, cursor_index)
        .unwrap_or(Point::ORIGIN);

    let offset = ((grapheme_position.x + 5.0) - text_bounds.width).max(0.0);

    (grapheme_position.x, offset)
}

/// Computes the position of the text cursor at the given X coordinate of
/// a [`TextInput`].
fn find_cursor_position<P: text::Paragraph>(
    text_bounds: Rectangle,
    value: &Value,
    state: &State<P>,
    x: f32,
) -> Option<usize> {
    let offset = offset(text_bounds, value, state);
    let value = value.to_string();

    let char_offset = state
        .value
        .raw()
        .hit_test(Point::new(x + offset, text_bounds.height / 2.0))
        .map(text::Hit::cursor)?;

    Some(
        unicode_segmentation::UnicodeSegmentation::graphemes(
            &value[..char_offset.min(value.len())],
            true,
        )
        .count(),
    )
}

fn replace_paragraph<Renderer>(
    renderer: &Renderer,
    state: &mut State<Renderer::Paragraph>,
    text_bounds: Option<Rectangle>,
    value: &Value,
    font: Option<Renderer::Font>,
    text_size: Option<Pixels>,
    line_height: text::LineHeight,
) where
    Renderer: text::Renderer,
{
    let font = font.unwrap_or_else(|| renderer.default_font());
    let text_size = text_size.unwrap_or_else(|| renderer.default_size());

    let height = text_bounds.map_or(0.0, |text_bounds| text_bounds.height);

    state.value = paragraph::Plain::new(Text {
        font,
        line_height,
        content: value.to_string(),
        bounds: Size::new(f32::INFINITY, height),
        size: text_size,
        align_x: text::Alignment::Default,
        align_y: alignment::Vertical::Center,
        shaping: text::Shaping::Advanced,
        wrapping: text::Wrapping::default(),
    });
}

/// The possible status of a [`TextInput`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// The [`TextInput`] can be interacted with.
    Active,
    /// The [`TextInput`] is being hovered.
    Hovered,
    /// The [`TextInput`] is focused.
    Focused {
        /// Whether the [`TextInput`] is hovered, while focused.
        is_hovered: bool,
    },
    /// The [`TextInput`] cannot be interacted with.
    Disabled,
}

/// The appearance of a text input.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// The [`Background`] of the text input.
    pub background: Background,
    /// The [`Border`] of the text input.
    pub border: Border,
    /// The [`Color`] of the icon of the text input.
    pub icon: Color,
    /// The [`Color`] of the placeholder of the text input.
    pub placeholder: Color,
    /// The [`Color`] of the value of the text input.
    pub value: Color,
    /// The [`Color`] of the selection of the text input.
    pub selection: Color,
}

/// The theme catalog of a [`TextInput`].
pub trait Catalog: Sized {
    /// The item class of the [`Catalog`].
    type Class<'a>;

    /// The default class produced by the [`Catalog`].
    fn default<'a>() -> Self::Class<'a>;

    /// The [`Style`] of a class with the given status.
    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style;
}

/// A styling function for a [`TextInput`].
///
/// This is just a boxed closure: `Fn(&Theme, Status) -> Style`.
pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme, Status) -> Style + 'a>;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(default)
    }

    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style {
        class(self, status)
    }
}

/// The default style of a [`TextInput`].
pub fn default(theme: &Theme, status: Status) -> Style {
    let palette = theme.extended_palette();

    let active = Style {
        background: Background::Color(palette.background.base.color),
        border: Border {
            radius: 2.0.into(),
            width: 1.0,
            color: palette.background.strong.color,
        },
        icon: palette.background.weak.text,
        placeholder: palette.secondary.base.color,
        value: palette.background.base.text,
        selection: palette.primary.weak.color,
    };

    match status {
        Status::Active => active,
        Status::Hovered => Style {
            border: Border {
                color: palette.background.base.text,
                ..active.border
            },
            ..active
        },
        Status::Focused { .. } => Style {
            border: Border {
                color: palette.primary.strong.color,
                ..active.border
            },
            ..active
        },
        Status::Disabled => Style {
            background: Background::Color(palette.background.weak.color),
            value: active.placeholder,
            placeholder: palette.background.strongest.color,
            ..active
        },
    }
}

fn alignment_offset(
    text_bounds_width: f32,
    text_min_width: f32,
    alignment: alignment::Horizontal,
) -> f32 {
    if text_min_width > text_bounds_width {
        0.0
    } else {
        match alignment {
            alignment::Horizontal::Left => 0.0,
            alignment::Horizontal::Center => (text_bounds_width - text_min_width) / 2.0,
            alignment::Horizontal::Right => text_bounds_width - text_min_width,
        }
    }
}

/// Maps the macOS Emacs-style `Ctrl` shortcuts to the equivalent navigation and deletion keys.
///
/// `iced_widget` keeps its own implementation crate-private, so this module owns a copy.
#[cfg(target_os = "macos")]
fn convert_macos_shortcut(
    key: &keyboard::Key,
    modifiers: keyboard::Modifiers,
) -> Option<keyboard::Key> {
    if modifiers != keyboard::Modifiers::CTRL {
        return None;
    }

    let named = match key.as_ref() {
        keyboard::Key::Character("b") => key::Named::ArrowLeft,
        keyboard::Key::Character("f") => key::Named::ArrowRight,
        keyboard::Key::Character("a") => key::Named::Home,
        keyboard::Key::Character("e") => key::Named::End,
        keyboard::Key::Character("h") => key::Named::Backspace,
        keyboard::Key::Character("d") => key::Named::Delete,
        _ => return None,
    };

    Some(keyboard::Key::Named(named))
}

/// Adapts a [`TextInput`] and its [`State`] to the shared [`ContextMenuHost`].
///
/// This is what lets the generic context menu (see
/// [`context_menu`](self::context_menu)) drive a [`TextInput`] the same way it
/// drives the search field of a select widget.
struct TextInputContext<'a, 'b, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: text::Renderer + svg::Renderer,
{
    input: &'b mut TextInput<'a, Message, Theme, Renderer>,
    state: &'b mut State<Renderer::Paragraph>,
}

impl<Message, Theme, Renderer> TextInputContext<'_, '_, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: text::Renderer + svg::Renderer,
{
    fn touch_focus(&mut self) {
        if let Some(focus) = &mut self.state.is_focused {
            focus.updated_at = Instant::now();
        }
    }

    fn update_cache(&mut self, renderer: &Renderer) {
        let text_bounds = self.state.text_bounds;

        replace_paragraph(
            renderer,
            self.state,
            text_bounds,
            &self.input.value,
            self.input.font,
            self.input.size,
            self.input.line_height,
        );
    }

    fn edit(
        &mut self,
        renderer: &Renderer,
        is_paste: bool,
        change: impl FnOnce(&mut Editor<'_>),
    ) -> Option<Message> {
        let callback = if is_paste {
            self.input
                .on_paste
                .as_ref()
                .or(self.input.on_input.as_ref())
        } else {
            self.input.on_input.as_ref()
        }?;

        let contents = edit_value(
            &mut self.input.value,
            &mut self.state.cursor,
            &mut self.state.history,
            EditKind::Other,
            change,
        );

        let message = (callback)(contents);

        self.state.is_pasting = None;
        self.touch_focus();
        self.update_cache(renderer);

        Some(message)
    }

    fn history_action(&mut self, renderer: &Renderer, is_redo: bool) -> Option<Message> {
        let on_input = self.input.on_input.as_ref()?;

        let current = self.input.value.to_string();
        let cursor = self.state.cursor;

        let target = if is_redo {
            self.state.history.redo(&current, cursor)
        } else {
            self.state.history.undo(&current, cursor)
        }?;

        self.input.value = Value::new(&target.value);
        self.state.cursor = target.cursor;
        self.state.is_pasting = None;

        let message = (on_input)(target.value);

        self.touch_focus();
        self.update_cache(renderer);

        Some(message)
    }
}

impl<Message, Theme, Renderer> ContextMenuHost<Message, Renderer>
    for TextInputContext<'_, '_, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: text::Renderer + svg::Renderer,
{
    fn value(&self) -> &Value {
        &self.input.value
    }

    fn cursor(&self) -> Cursor {
        self.state.cursor
    }

    fn is_editable(&self) -> bool {
        self.input.on_input.is_some()
    }

    fn is_secure(&self) -> bool {
        self.input.is_secure
    }

    fn can_undo(&self) -> bool {
        self.input.on_input.is_some() && self.state.history.can_undo()
    }

    fn can_redo(&self) -> bool {
        self.input.on_input.is_some() && self.state.history.can_redo()
    }

    fn delete_selection(&mut self, renderer: &Renderer) -> Option<Message> {
        self.state.cursor.selection(&self.input.value)?;

        self.edit(renderer, false, |editor| editor.delete())
    }

    fn paste(&mut self, content: Value, renderer: &Renderer) -> Option<Message> {
        self.edit(renderer, true, |editor| editor.paste(content))
    }

    fn undo(&mut self, renderer: &Renderer) -> Option<Message> {
        self.history_action(renderer, false)
    }

    fn redo(&mut self, renderer: &Renderer) -> Option<Message> {
        self.history_action(renderer, true)
    }

    fn select_word(&mut self) {
        let position = self.state.cursor.end(&self.input.value);
        let start = self.input.value.previous_start_of_word(position);
        let end = self.input.value.next_end_of_word(position);

        self.state.cursor.select_range(start, end);
        self.touch_focus();
    }

    fn select_all(&mut self) {
        self.state.cursor.select_all(&self.input.value);
        self.touch_focus();
    }

    fn menu(&self) -> Option<ContextMenu> {
        self.state.context_menu
    }

    fn set_menu(&mut self, menu: Option<ContextMenu>) {
        self.state.context_menu = menu;
    }

    fn label(&self, action: Action) -> std::borrow::Cow<'_, str> {
        std::borrow::Cow::Borrowed(self.input.labels.actions.get(action))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn history_groups_consecutive_typing() {
        let cursor = Cursor::default();
        let mut history = History::default();

        history.record("", cursor, EditKind::Insert);
        history.commit("h");
        history.record("h", cursor, EditKind::Insert);
        history.commit("he");
        history.record("he", cursor, EditKind::Insert);
        history.commit("hel");

        let undone = history.undo("hel", cursor).expect("an undo step");

        assert_eq!(undone.value, "");
        assert!(history.undo("", cursor).is_none());
    }

    #[test]
    fn history_undoes_and_redoes_separate_runs() {
        let cursor = Cursor::default();
        let mut history = History::default();

        history.record("", cursor, EditKind::Insert);
        history.commit("a");

        history.break_group();

        history.record("a", cursor, EditKind::Insert);
        history.commit("ab");

        let undone = history.undo("ab", cursor).expect("an undo step");
        assert_eq!(undone.value, "a");

        let redone = history.redo("a", cursor).expect("a redo step");
        assert_eq!(redone.value, "ab");

        assert!(history.redo("ab", cursor).is_none());
    }

    #[test]
    fn history_does_not_group_other_edits() {
        let cursor = Cursor::default();
        let mut history = History::default();

        history.record("", cursor, EditKind::Other);
        history.commit("a");
        history.record("a", cursor, EditKind::Other);
        history.commit("ab");

        let undone = history.undo("ab", cursor).expect("an undo step");

        assert_eq!(undone.value, "a");
    }

    #[test]
    fn history_forgets_redo_after_new_edit() {
        let cursor = Cursor::default();
        let mut history = History::default();

        history.record("", cursor, EditKind::Insert);
        history.commit("a");

        let _ = history.undo("a", cursor).expect("an undo step");

        history.record("", cursor, EditKind::Insert);

        assert!(history.redo("", cursor).is_none());
    }

    #[test]
    fn history_is_limited_in_size() {
        let cursor = Cursor::default();
        let mut history = History::default();

        for i in 0..=MAX_UNDO_STEPS {
            history.break_group();
            history.record(&i.to_string(), cursor, EditKind::Insert);
        }

        let mut steps = 0;

        while history.undo("", cursor).is_some() {
            steps += 1;
        }

        assert_eq!(steps, MAX_UNDO_STEPS);
    }
}
