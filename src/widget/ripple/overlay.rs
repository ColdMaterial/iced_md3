use iced::{
    Rectangle, Vector,
    advanced::{Layout, Overlay, layout, mouse, overlay, renderer},
    border::Radius,
};
use iced_widget::renderer::geometry;

use super::Ripple;
use crate::style::StateLayer;

pub(crate) struct StateLayerOverlay {
    pub ripple: Ripple,
    pub layer: StateLayer,
    pub bounds: Rectangle,
    pub clip: Rectangle,
}

impl StateLayerOverlay {
    pub(crate) fn clipped(mut self, viewport: &Rectangle) -> Option<Self> {
        self.clip = visible_clip(self.bounds, viewport)?;
        Some(self)
    }
}

/// Builds the clipped overlay that paints `ripple`'s state layer at `bounds`
/// (offset by `translation`).
///
/// Returns `None` when the widget is disabled or nothing would be drawn.
pub(crate) fn state_layer_overlay<'a, Message, Theme, Renderer>(
    ripple: Ripple,
    layer: StateLayer,
    bounds: Rectangle,
    translation: Vector,
    viewport: &Rectangle,
    enabled: bool,
) -> Option<overlay::Element<'a, Message, Theme, Renderer>>
where
    Renderer: geometry::Renderer,
{
    if !(enabled && ripple.is_visible(bounds.size(), enabled)) {
        return None;
    }

    let overlay = StateLayerOverlay {
        ripple,
        layer,
        bounds: Rectangle {
            x: bounds.x + translation.x,
            y: bounds.y + translation.y,
            ..bounds
        },
        clip: Rectangle::default(),
    }
    .clipped(viewport)?;

    Some(overlay::Element::new(Box::new(overlay)))
}

pub(crate) fn visible_clip(bounds: Rectangle, viewport: &Rectangle) -> Option<Rectangle> {
    bounds.intersection(viewport)
}

impl<Message, Theme, Renderer> Overlay<Message, Theme, Renderer> for StateLayerOverlay
where
    Renderer: geometry::Renderer,
{
    fn layout(&mut self, _renderer: &Renderer, _bounds: iced::Size) -> layout::Node {
        layout::Node::new(self.bounds.size()).move_to(self.bounds.position())
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        _layout: Layout<'_>,
        _cursor: mouse::Cursor,
    ) {
        renderer.with_layer(self.clip, |renderer| {
            self.ripple.draw(
                renderer,
                self.bounds,
                Radius::from(f32::MAX),
                self.layer,
                true,
            );
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn visible_clip_is_none_when_scrolled_out_of_view() {
        let bounds = Rectangle {
            x: 10.0,
            y: 10.0,
            width: 40.0,
            height: 40.0,
        };
        let viewport = Rectangle {
            x: 0.0,
            y: 200.0,
            width: 400.0,
            height: 300.0,
        };

        assert_eq!(visible_clip(bounds, &viewport), None);
    }

    #[test]
    fn visible_clip_is_intersected_with_viewport() {
        let bounds = Rectangle {
            x: 10.0,
            y: 10.0,
            width: 40.0,
            height: 40.0,
        };
        let viewport = Rectangle {
            x: 0.0,
            y: 30.0,
            width: 400.0,
            height: 300.0,
        };

        assert_eq!(
            visible_clip(bounds, &viewport),
            Some(Rectangle {
                x: 10.0,
                y: 30.0,
                width: 40.0,
                height: 20.0,
            })
        );
    }
}
