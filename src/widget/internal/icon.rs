use iced::{
    Color, Font, Pixels, Point, Radians, Rectangle, Size,
    advanced::{svg, text},
    alignment::Vertical,
};

use crate::widget::hybrid_icon::Icon;

pub fn draw_svg_icon<Renderer>(
    renderer: &mut Renderer,
    handle: &svg::Handle,
    center: Point,
    size: Pixels,
    rotation: Radians,
    color: Color,
    opacity: f32,
    clip: Rectangle,
) where
    Renderer: svg::Renderer,
{
    let half = size.0 / 2.0;
    let bounds = Rectangle::new(
        Point::new(center.x - half, center.y - half),
        Size::new(size.0, size.0),
    );

    let opacity = opacity * color.a;
    let color = Color { a: 1.0, ..color };

    renderer.draw_svg(
        svg::Svg {
            handle: handle.clone(),
            color: Some(color),
            rotation,
            opacity,
        },
        bounds,
        clip,
    );
}

/// Draws a hybrid icon centered on `center`, clipped to `clip`.
pub fn draw_hybrid_icon<Renderer>(
    renderer: &mut Renderer,
    icon: &Icon<'_>,
    center: Point,
    size: Pixels,
    color: Color,
    clip: Rectangle,
) where
    Renderer: text::Renderer<Font = Font> + svg::Renderer,
{
    match icon {
        Icon::Svg(handle) => draw_svg_icon(
            renderer,
            handle,
            center,
            size,
            Radians(0.0),
            color,
            1.0,
            clip,
        ),
        Icon::Text { text, font } => {
            let half = size.0 / 2.0;
            let bounds = Rectangle::new(
                Point::new(center.x - half, center.y - half),
                Size::new(size.0, size.0),
            );

            renderer.fill_text(
                text::Text {
                    content: text.to_string(),
                    bounds: bounds.size(),
                    size,
                    line_height: text::LineHeight::Absolute(size),
                    font: font.unwrap_or_else(|| renderer.default_font()),
                    align_x: text::Alignment::Center,
                    align_y: Vertical::Center,
                    shaping: text::Shaping::Advanced,
                    wrapping: text::Wrapping::None,
                },
                bounds.center(),
                color,
                clip,
            );
        }
    }
}
