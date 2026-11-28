use iced::{
    Element, Event, Length, Point, Rectangle, Size, Vector,
    advanced::{
        Clipboard, Layout, Renderer as _, Shell, Widget,
        layout::{Limits, Node},
        mouse, overlay,
        renderer::Quad,
        widget::{Operation, Tree, tree},
    },
    color, touch,
};

const SCRIM_ALPHA: f32 = 0.4;

pub struct Modal<'a, Message> {
    content: Element<'a, Message>,
    open: bool,
    fullscreen: bool,
    on_press_outside: Option<Message>,
}

impl<'a, Message> Modal<'a, Message> {
    pub fn new(
        content: impl Into<Element<'a, Message>>,
        open: bool,
        fullscreen: bool,
        on_press_outside: Option<Message>,
    ) -> Self {
        Self {
            content: content.into(),
            open,
            fullscreen,
            on_press_outside,
        }
    }
}

struct State {
    content: Option<Tree>,
}

impl<'a, Message> Widget<Message, iced::Theme, iced::Renderer> for Modal<'a, Message>
where
    Message: 'a + Clone,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State { content: None })
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Shrink, Length::Shrink)
    }

    fn layout(&mut self, _tree: &mut Tree, _renderer: &iced::Renderer, limits: &Limits) -> Node {
        Node::new(limits.resolve(Length::Shrink, Length::Shrink, Size::ZERO))
    }

    fn draw(
        &self,
        _tree: &Tree,
        _renderer: &mut iced::Renderer,
        _theme: &iced::Theme,
        _style: &iced::advanced::renderer::Style,
        _layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        _layout: Layout<'b>,
        _renderer: &iced::Renderer,
        _viewport: &Rectangle,
        _translation: Vector,
    ) -> Option<overlay::Element<'b, Message, iced::Theme, iced::Renderer>> {
        if !self.open {
            tree.state.downcast_mut::<State>().content = None;
            return None;
        }

        let state = tree.state.downcast_mut::<State>();
        if let Some(content_tree) = &mut state.content {
            content_tree.diff(self.content.as_widget());
        } else {
            state.content = Some(Tree::new(self.content.as_widget()));
        }

        Some(overlay::Element::new(Box::new(ModalOverlay {
            content: &mut self.content,
            tree: state.content.as_mut()?,
            fullscreen: self.fullscreen,
            on_press_outside: self.on_press_outside.as_ref(),
        })))
    }
}

struct ModalOverlay<'a, 'b, Message> {
    content: &'b mut Element<'a, Message>,
    tree: &'b mut Tree,
    fullscreen: bool,
    on_press_outside: Option<&'b Message>,
}

impl<'a, 'b, Message> overlay::Overlay<Message, iced::Theme, iced::Renderer>
    for ModalOverlay<'a, 'b, Message>
where
    Message: 'a + Clone,
{
    fn layout(&mut self, renderer: &iced::Renderer, bounds: Size) -> Node {
        let limits = Limits::new(Size::ZERO, bounds);
        let mut content = self
            .content
            .as_widget_mut()
            .layout(self.tree, renderer, &limits);

        let position = if self.fullscreen {
            Point::ORIGIN
        } else {
            Point::new(
                (bounds.width - content.size().width) / 2.0,
                (bounds.height - content.size().height) / 2.0,
            )
        };
        content = content.move_to(position);

        Node::with_children(bounds, vec![content])
    }

    fn draw(
        &self,
        renderer: &mut iced::Renderer,
        theme: &iced::Theme,
        style: &iced::advanced::renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        let bounds = layout.bounds();
        renderer.fill_quad(
            Quad {
                bounds,
                ..Default::default()
            },
            color!(0x000000).scale_alpha(SCRIM_ALPHA),
        );
        self.content.as_widget().draw(
            self.tree,
            renderer,
            theme,
            style,
            layout.child(0),
            cursor,
            &bounds,
        );
    }

    fn update(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) {
        let bounds = layout.bounds();
        self.content.as_widget_mut().update(
            self.tree,
            event,
            layout.child(0),
            cursor,
            renderer,
            clipboard,
            shell,
            &bounds,
        );

        let child_bounds = layout.child(0).bounds();
        if matches!(
            event,
            Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left))
        ) && cursor
            .position()
            .is_some_and(|position| !child_bounds.contains(position))
            && let Some(message) = self.on_press_outside
        {
            shell.publish(message.clone());
        }

        if matches!(
            event,
            Event::Mouse(
                iced::mouse::Event::ButtonPressed(_)
                    | iced::mouse::Event::ButtonReleased(_)
                    | iced::mouse::Event::WheelScrolled { .. }
            ) | Event::Touch(
                touch::Event::FingerPressed { .. }
                    | touch::Event::FingerLifted { .. }
                    | touch::Event::FingerLost { .. }
            )
        ) {
            shell.capture_event();
        }
    }

    fn operate(
        &mut self,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn Operation,
    ) {
        self.content
            .as_widget_mut()
            .operate(self.tree, layout.child(0), renderer, operation);
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        let child = layout.child(0);
        if cursor.is_over(child.bounds()) {
            self.content.as_widget().mouse_interaction(
                self.tree,
                child,
                cursor,
                &layout.bounds(),
                renderer,
            )
        } else {
            mouse::Interaction::Idle
        }
    }

    fn overlay<'c>(
        &'c mut self,
        layout: Layout<'c>,
        renderer: &iced::Renderer,
    ) -> Option<overlay::Element<'c, Message, iced::Theme, iced::Renderer>> {
        let bounds = layout.bounds();
        self.content.as_widget_mut().overlay(
            self.tree,
            layout.child(0),
            renderer,
            &bounds,
            Vector::ZERO,
        )
    }
}

impl<'a, Message> From<Modal<'a, Message>> for Element<'a, Message>
where
    Message: 'a + Clone,
{
    fn from(modal: Modal<'a, Message>) -> Self {
        Element::new(modal)
    }
}
