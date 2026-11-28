use std::borrow::Cow;

use iced::{
    Border, Color, Event, Font, Point, Radians, Rectangle, Size, Vector,
    advanced::{
        Clipboard, Layout, Shell, clipboard, layout, mouse, overlay, renderer, svg,
        text::{self, Paragraph as _},
    },
    alignment, keyboard, touch,
};

use crate::{
    style::shadow,
    widget::{
        internal::placement,
        menu,
        text_input::{Action, Cursor, Value},
    },
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct ContextMenu {
    pub(crate) position: Point,
    pub(crate) pressed: Option<Action>,
    pub(crate) is_pressing: bool,
    pub(crate) hovered: Option<Action>,
}

impl ContextMenu {
    pub(crate) fn new(position: Point) -> Self {
        Self {
            position,
            pressed: None,
            is_pressing: false,
            hovered: None,
        }
    }
}

pub(crate) trait ContextMenuHost<Message, Renderer> {
    fn value(&self) -> &Value;
    fn cursor(&self) -> Cursor;
    fn is_editable(&self) -> bool;
    fn is_secure(&self) -> bool {
        false
    }
    fn can_undo(&self) -> bool;
    fn can_redo(&self) -> bool;

    fn selection(&self) -> Option<(usize, usize)> {
        self.cursor().selection(self.value())
    }

    fn selected_text(&self) -> Option<String> {
        self.selection()
            .map(|(start, end)| self.value().select(start, end).to_string())
    }

    fn delete_selection(&mut self, renderer: &Renderer) -> Option<Message>;
    fn paste(&mut self, content: Value, renderer: &Renderer) -> Option<Message>;
    fn undo(&mut self, renderer: &Renderer) -> Option<Message>;
    fn redo(&mut self, renderer: &Renderer) -> Option<Message>;
    fn select_word(&mut self);
    fn select_all(&mut self);

    fn menu(&self) -> Option<ContextMenu>;
    fn set_menu(&mut self, menu: Option<ContextMenu>);

    fn label(&self, action: Action) -> Cow<'_, str> {
        Cow::Borrowed(action.label())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ContextMenuEntry {
    Item(Action),
    Separator,
}

const CONTEXT_MENU_ENTRIES: [ContextMenuEntry; 9] = [
    ContextMenuEntry::Item(Action::Copy),
    ContextMenuEntry::Item(Action::Cut),
    ContextMenuEntry::Item(Action::Paste),
    ContextMenuEntry::Separator,
    ContextMenuEntry::Item(Action::SelectWord),
    ContextMenuEntry::Item(Action::SelectAll),
    ContextMenuEntry::Separator,
    ContextMenuEntry::Item(Action::Undo),
    ContextMenuEntry::Item(Action::Redo),
];

pub(crate) struct ContextMenuOverlay<H> {
    pub(crate) host: H,
    pub(crate) style: menu::Style,
    pub(crate) font: Option<Font>,
    pub(crate) translation: Vector,
}

fn is_enabled<H, Message, Renderer>(host: &H, item: Action) -> bool
where
    H: ContextMenuHost<Message, Renderer>,
    Renderer: text::Renderer,
{
    match item {
        Action::Undo => host.is_editable() && host.can_undo(),
        Action::Redo => host.is_editable() && host.can_redo(),
        Action::Copy | Action::Cut => !host.is_secure() && host.selection().is_some(),
        Action::Paste => host.is_editable(),
        Action::SelectWord | Action::SelectAll => !host.value().is_empty() && host.is_editable(),
    }
}

fn entry_height(entry: ContextMenuEntry) -> f32 {
    match entry {
        ContextMenuEntry::Item(_) => menu::constants::ITEM_HEIGHT.0,
        ContextMenuEntry::Separator => menu::constants::SPACER_HEIGHT.0,
    }
}

fn menu_size() -> Size {
    let padding = menu::constants::MENU_GROUP_PADDING.0;
    let spacing = menu::constants::MENU_ITEM_SPACING.0;

    let rows = CONTEXT_MENU_ENTRIES
        .iter()
        .map(|entry| entry_height(*entry) + spacing)
        .sum::<f32>();

    Size::new(
        menu::constants::DEFAULT_WIDTH.0,
        padding + rows - spacing + padding,
    )
}

fn menu_children() -> Vec<layout::Node> {
    let padding = menu::constants::MENU_GROUP_PADDING.0;
    let spacing = menu::constants::MENU_ITEM_SPACING.0;
    let width = menu_size().width - 2.0 * padding;

    let mut offset_y = padding;

    CONTEXT_MENU_ENTRIES
        .iter()
        .map(|entry| {
            let height = entry_height(*entry);

            let node =
                layout::Node::new(Size::new(width, height)).move_to(Point::new(padding, offset_y));

            offset_y += height + spacing;
            node
        })
        .collect()
}

fn is_menu_layout(layout: Layout<'_>) -> bool {
    layout.children().len() == CONTEXT_MENU_ENTRIES.len() && layout.bounds().size() == menu_size()
}

fn item_at(layout: Layout<'_>, position: Point) -> Option<Action> {
    layout
        .children()
        .zip(CONTEXT_MENU_ENTRIES)
        .find_map(|(child, entry)| match entry {
            ContextMenuEntry::Item(item) if child.bounds().contains(position) => Some(item),
            _ => None,
        })
}

fn close<H, Message, Renderer>(host: &mut H, shell: &mut Shell<'_, Message>)
where
    H: ContextMenuHost<Message, Renderer>,
{
    host.set_menu(None);
    shell.request_redraw();

    // The overlay disappears outside of the runtime's relayout pass. Ask for one so the other
    // overlays sharing the group are not drawn with our layout node for this frame.
    shell.invalidate_layout();
}

fn perform<H, Message, Renderer>(
    host: &mut H,
    item: Action,
    clipboard: &mut dyn Clipboard,
    renderer: &Renderer,
    shell: &mut Shell<'_, Message>,
) where
    H: ContextMenuHost<Message, Renderer>,
    Renderer: text::Renderer,
{
    let message = match item {
        Action::Undo => host.undo(renderer),
        Action::Redo => host.redo(renderer),
        Action::Copy => {
            if let Some(text) = host.selected_text() {
                clipboard.write(clipboard::Kind::Standard, text);
            }

            None
        }
        Action::Cut => {
            if let Some(text) = host.selected_text() {
                clipboard.write(clipboard::Kind::Standard, text);
            }

            host.delete_selection(renderer)
        }
        Action::Paste => {
            let content: String = clipboard
                .read(clipboard::Kind::Standard)
                .unwrap_or_default()
                .chars()
                .filter(|character| !character.is_control())
                .collect();

            if content.is_empty() {
                None
            } else {
                host.paste(Value::new(&content), renderer)
            }
        }
        Action::SelectWord => {
            host.select_word();

            None
        }
        Action::SelectAll => {
            host.select_all();

            None
        }
    };

    if let Some(message) = message {
        shell.publish(message);
    }

    shell.request_redraw();
}

impl<H, Message, Theme, Renderer> overlay::Overlay<Message, Theme, Renderer>
    for ContextMenuOverlay<H>
where
    Message: Clone,
    H: ContextMenuHost<Message, Renderer>,
    Renderer: text::Renderer + svg::Renderer,
    Renderer::Font: From<Font>,
{
    fn layout(&mut self, _renderer: &Renderer, bounds: Size) -> layout::Node {
        let size = menu_size();
        let position = self
            .host
            .menu()
            .map(|menu| {
                placement::clamp_into_viewport(menu.position + self.translation, size, bounds)
            })
            .unwrap_or(Point::ORIGIN);

        layout::Node::with_children(size, menu_children()).move_to(position)
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        if !is_menu_layout(layout) {
            // A stale node from another overlay; see `is_menu_layout`.
            return;
        }

        let bounds = layout.bounds();

        let font = self
            .font
            .map(Renderer::Font::from)
            .unwrap_or_else(|| renderer.default_font());
        let pressed = self.host.menu().and_then(|menu| menu.pressed);

        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: self.style.container_border,
                shadow: shadow(self.style.shadow_color, self.style.elevation),
                ..Default::default()
            },
            self.style.container_color,
        );

        for (entry, child) in CONTEXT_MENU_ENTRIES.into_iter().zip(layout.children()) {
            let child_bounds = child.bounds();

            match entry {
                ContextMenuEntry::Separator => {
                    renderer.fill_quad(
                        renderer::Quad {
                            bounds: child_bounds,
                            border: Border::default().rounded(f32::MAX),
                            ..Default::default()
                        },
                        self.style.separator_color,
                    );
                }
                ContextMenuEntry::Item(item) => {
                    let enabled = is_enabled(&self.host, item);
                    let hovered = enabled && cursor.is_over(child_bounds);
                    let opacity = if enabled {
                        1.0
                    } else {
                        menu::constants::ITEM_CONTENT_DISABLED_OPACITY
                    };

                    let state_layer = if hovered && pressed == Some(item) {
                        self.style.item.state_layer.pressed
                    } else if hovered {
                        self.style.item.state_layer.hovered
                    } else {
                        Color::TRANSPARENT
                    };

                    if state_layer.a > 0.0 {
                        renderer.fill_quad(
                            renderer::Quad {
                                bounds: child_bounds,
                                border: Border::default()
                                    .rounded(menu::constants::ITEM_CORNER_RADIUS.0),
                                ..Default::default()
                            },
                            state_layer,
                        );
                    }

                    let padding = menu::constants::ITEM_CONTENT_PADDING.0;
                    let icon_size = menu::constants::ITEM_ICON_SIZE.0;
                    let spacing = menu::constants::ITEM_CONTENT_SPACING.0;

                    let icon_bounds = Rectangle {
                        x: child_bounds.x + padding,
                        y: child_bounds.y + (child_bounds.height - icon_size) / 2.0,
                        width: icon_size,
                        height: icon_size,
                    };

                    renderer.draw_svg(
                        svg::Svg {
                            handle: item.icon(),
                            color: Some(self.style.item.icon_color),
                            rotation: Radians(0.0),
                            opacity,
                        },
                        icon_bounds,
                        icon_bounds,
                    );

                    let shortcut = item.shortcut_text();
                    let shortcut_line_height = menu::constants::ITEM_SUPPORTING_TEXT_LINE_HEIGHT.0;

                    let shortcut_width = shortcut.as_ref().map_or(0.0, |shortcut| {
                        crate::widget::internal::text::paragraph(
                            renderer,
                            shortcut.as_str(),
                            Size::new(child_bounds.width, shortcut_line_height),
                            menu::constants::ITEM_SUPPORTING_TEXT_SIZE,
                            text::LineHeight::Absolute(
                                menu::constants::ITEM_SUPPORTING_TEXT_LINE_HEIGHT,
                            ),
                            Some(font),
                            alignment::Vertical::Top,
                        )
                        .min_bounds()
                        .width
                    });

                    let shortcut_gap = if shortcut_width > 0.0 {
                        spacing + shortcut_width
                    } else {
                        0.0
                    };

                    let content_right = child_bounds.x + child_bounds.width - padding;
                    let label_height = menu::constants::ITEM_LABEL_LINE_HEIGHT.0;
                    let label_x = icon_bounds.x + icon_size + spacing;
                    let label_bounds = Rectangle {
                        x: label_x,
                        y: child_bounds.y + (child_bounds.height - label_height) / 2.0,
                        width: (content_right - label_x - shortcut_gap).max(0.0),
                        height: label_height,
                    };

                    renderer.fill_text(
                        text::Text {
                            content: self.host.label(item).into_owned(),
                            bounds: label_bounds.size(),
                            size: menu::constants::ITEM_LABEL_FONT_SIZE,
                            line_height: text::LineHeight::Absolute(
                                menu::constants::ITEM_LABEL_LINE_HEIGHT,
                            ),
                            font,
                            align_x: text::Alignment::Left,
                            align_y: alignment::Vertical::Top,
                            shaping: text::Shaping::Advanced,
                            wrapping: text::Wrapping::None,
                        },
                        label_bounds.position(),
                        self.style.item.label_color.scale_alpha(opacity),
                        label_bounds,
                    );

                    if let Some(shortcut) = shortcut {
                        let shortcut_bounds = Rectangle {
                            x: label_x,
                            y: child_bounds.y + (child_bounds.height - shortcut_line_height) / 2.0,
                            width: (content_right - label_x).max(0.0),
                            height: shortcut_line_height,
                        };

                        renderer.fill_text(
                            text::Text {
                                content: shortcut,
                                bounds: shortcut_bounds.size(),
                                size: menu::constants::ITEM_SUPPORTING_TEXT_SIZE,
                                line_height: text::LineHeight::Absolute(
                                    menu::constants::ITEM_SUPPORTING_TEXT_LINE_HEIGHT,
                                ),
                                font,
                                align_x: text::Alignment::Right,
                                align_y: alignment::Vertical::Top,
                                shaping: text::Shaping::Advanced,
                                wrapping: text::Wrapping::None,
                            },
                            Point::new(content_right, shortcut_bounds.y),
                            self.style.item.supporting_text_color.scale_alpha(opacity),
                            shortcut_bounds,
                        );
                    }
                }
            }
        }
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
        let bounds = layout.bounds();

        let Some(mut menu) = self.host.menu() else {
            return;
        };

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                let Some(position) = cursor.position() else {
                    return;
                };

                if bounds.contains(position) {
                    let item = item_at(layout, position);

                    menu.pressed = item;
                    menu.is_pressing = true;
                    self.host.set_menu(Some(menu));

                    shell.capture_event();
                    shell.request_redraw();
                } else {
                    close(&mut self.host, shell);
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerLifted { .. }) => {
                if !menu.is_pressing {
                    return;
                }

                let pressed = menu.pressed;
                let released_over = cursor
                    .position()
                    .filter(|position| bounds.contains(*position))
                    .and_then(|position| item_at(layout, position));

                shell.capture_event();

                match pressed {
                    Some(item) if !is_enabled(&self.host, item) => {
                        // Disabled items are not actionable, so treat the press as a no-op and
                        // keep the menu open.
                        menu.pressed = None;
                        menu.is_pressing = false;
                        self.host.set_menu(Some(menu));

                        shell.request_redraw();
                    }
                    Some(item) if Some(item) == released_over => {
                        perform(&mut self.host, item, clipboard, renderer, shell);
                        close(&mut self.host, shell);
                    }
                    _ => close(&mut self.host, shell),
                }
            }
            Event::Touch(touch::Event::FingerLost { .. }) => {
                if menu.is_pressing {
                    menu.is_pressing = false;
                    menu.pressed = None;
                    self.host.set_menu(Some(menu));

                    shell.request_redraw();
                }
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)) => {
                let is_inside = cursor
                    .position()
                    .is_some_and(|position| bounds.contains(position));

                if is_inside {
                    shell.capture_event();
                } else {
                    // Let the text widget handle the press so it can reopen the menu elsewhere.
                    close(&mut self.host, shell);
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { .. })
            | Event::Touch(touch::Event::FingerMoved { .. }) => {
                let hovered = cursor
                    .position()
                    .filter(|position| bounds.contains(*position))
                    .and_then(|position| item_at(layout, position));

                if menu.hovered != hovered {
                    menu.hovered = hovered;
                    self.host.set_menu(Some(menu));

                    shell.request_redraw();
                }
            }
            Event::Keyboard(keyboard::Event::KeyPressed { key, .. })
                if matches!(
                    key.as_ref(),
                    keyboard::Key::Named(keyboard::key::Named::Escape)
                ) =>
            {
                close(&mut self.host, shell);
                shell.capture_event();
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
        let Some(item) = cursor
            .position_over(layout.bounds())
            .and_then(|position| item_at(layout, position))
        else {
            return mouse::Interaction::None;
        };

        if is_enabled(&self.host, item) {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn menu_layout_is_recognised() {
        let menu = layout::Node::with_children(menu_size(), menu_children());

        assert!(is_menu_layout(Layout::new(&menu)));

        // A leaf node is what a foreign overlay (like a select popup) provides.
        assert!(!is_menu_layout(Layout::new(
            &layout::Node::new(menu_size())
        )));
    }

    #[test]
    fn context_menu_lists_edit_and_selection_items() {
        assert_eq!(
            CONTEXT_MENU_ENTRIES,
            [
                ContextMenuEntry::Item(Action::Copy),
                ContextMenuEntry::Item(Action::Cut),
                ContextMenuEntry::Item(Action::Paste),
                ContextMenuEntry::Separator,
                ContextMenuEntry::Item(Action::SelectWord),
                ContextMenuEntry::Item(Action::SelectAll),
                ContextMenuEntry::Separator,
                ContextMenuEntry::Item(Action::Undo),
                ContextMenuEntry::Item(Action::Redo),
            ]
        );
    }
}
