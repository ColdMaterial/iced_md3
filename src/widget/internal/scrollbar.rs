//! Scrollbar geometry and styling shared by page scrollables and dropdown menus.
//!
//! Keeping the metrics and the interaction states in one place makes every scrollbar of the
//! library look and behave the same, instead of each widget rolling its own.

use iced::{Border, Color, Point, Rectangle, Size, advanced::mouse};

/// Width of the scroller.
pub(crate) const WIDTH: f32 = 4.0;

/// Gap between the scroller and the trailing edge of its viewport.
pub(crate) const MARGIN: f32 = 2.0;

/// Smallest scroller length, keeping it grabbable on very long content.
pub(crate) const MIN_LENGTH: f32 = 2.0;

/// Opacity of the scroller while the pointer is away from the viewport.
const IDLE_OPACITY: f32 = 0.15;

/// Opacity of the scroller while the pointer hovers the viewport.
const HOVERED_OPACITY: f32 = 0.35;

/// Opacity of the scroller while the pointer hovers the scrollbar itself.
const RAIL_HOVERED_OPACITY: f32 = 0.8;

/// Horizontal space reserved at the trailing edge of scrollable content so it never renders
/// underneath the scrollbar.
pub(crate) const GUTTER: f32 = WIDTH + 2.0 * MARGIN;

/// Scroll distance of a single wheel notch, matching the default page scrolling speed.
pub(crate) const LINES_PER_NOTCH: f32 = 60.0;

/// Color of the scroller for the given pointer interaction.
pub(crate) fn scroller_color(
    base: Color,
    hovered: bool,
    rail_hovered: bool,
    dragged: bool,
) -> Color {
    let opacity = if dragged {
        1.0
    } else if rail_hovered {
        RAIL_HOVERED_OPACITY
    } else if hovered {
        HOVERED_OPACITY
    } else {
        IDLE_OPACITY
    };

    base.scale_alpha(opacity)
}

/// Border of the scroller: a pill, whatever its thickness.
pub(crate) fn scroller_border() -> Border {
    Border::default().rounded(u32::MAX)
}

/// Converts a wheel delta into a vertical scroll offset change.
pub(crate) fn wheel_offset(delta: mouse::ScrollDelta) -> f32 {
    match delta {
        mouse::ScrollDelta::Lines { y, .. } => -y * LINES_PER_NOTCH,
        mouse::ScrollDelta::Pixels { y, .. } => -y,
    }
}

/// Geometry of a vertical scrollbar inside a scrollable viewport.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Rail {
    /// Clickable rail, wider than the scroller so it is easy to grab.
    pub(crate) bounds: Rectangle,
    /// Visible scroller.
    pub(crate) scroller: Rectangle,
    /// Scroll distance at the end of the viewport.
    pub(crate) max_offset: f32,
}

impl Rail {
    /// Computes the scrollbar of a viewport showing `content_height` pixels of content.
    ///
    /// Returns `None` when the content fits, so no scrollbar is needed.
    pub(crate) fn vertical(viewport: Rectangle, content_height: f32, offset: f32) -> Option<Self> {
        if viewport.height <= 0.0 || content_height <= viewport.height {
            return None;
        }

        let max_offset = (content_height - viewport.height).max(0.0);
        let ratio = (viewport.height / content_height).clamp(0.0, 1.0);
        let length = (viewport.height * ratio)
            .max(MIN_LENGTH)
            .min(viewport.height);
        let travel = viewport.height - length;
        let progress = (offset / max_offset).clamp(0.0, 1.0);
        let right = viewport.x + viewport.width;

        Some(Self {
            bounds: Rectangle::new(
                Point::new(right - (WIDTH + 2.0 * MARGIN), viewport.y),
                Size::new(WIDTH + 2.0 * MARGIN, viewport.height),
            ),
            scroller: Rectangle::new(
                Point::new(right - MARGIN - WIDTH, viewport.y + progress * travel),
                Size::new(WIDTH, length),
            ),
            max_offset,
        })
    }

    /// Offset inside the scroller when it is grabbed at `position`.
    ///
    /// Grabbing the rail outside the scroller centers the scroller under the pointer.
    pub(crate) fn grab(&self, position: Point) -> f32 {
        if self.scroller.contains(position) {
            position.y - self.scroller.y
        } else {
            self.scroller.height / 2.0
        }
    }

    /// Scroll offset that puts the top of the scroller at `scroller_top`.
    pub(crate) fn offset_at(&self, scroller_top: f32) -> f32 {
        let travel = self.bounds.height - self.scroller.height;

        if travel <= 0.0 {
            return 0.0;
        }

        ((scroller_top - self.bounds.y) / travel).clamp(0.0, 1.0) * self.max_offset
    }
}

#[cfg(test)]
mod tests {
    use super::Rail;
    use iced::{Point, Rectangle, Size};

    fn viewport() -> Rectangle {
        Rectangle::new(Point::new(0.0, 10.0), Size::new(100.0, 200.0))
    }

    #[test]
    fn content_that_fits_has_no_scrollbar() {
        assert!(Rail::vertical(viewport(), 200.0, 0.0).is_none());
        assert!(Rail::vertical(viewport(), 20.0, 0.0).is_none());
    }

    #[test]
    fn scroller_tracks_the_scroll_offset() {
        let start = Rail::vertical(viewport(), 600.0, 0.0).unwrap();
        let middle = Rail::vertical(viewport(), 600.0, start.max_offset / 2.0).unwrap();
        let end = Rail::vertical(viewport(), 600.0, start.max_offset).unwrap();

        assert_eq!(start.scroller.y, viewport().y);
        assert_eq!(
            end.scroller.y + end.scroller.height,
            viewport().y + viewport().height
        );

        let travel = viewport().height - start.scroller.height;
        assert!((middle.scroller.y - (viewport().y + travel / 2.0)).abs() < 0.01);
    }

    #[test]
    fn dragging_the_scroller_maps_back_to_the_offset() {
        let rail = Rail::vertical(viewport(), 600.0, 0.0).unwrap();

        assert_eq!(rail.offset_at(rail.bounds.y), 0.0);
        assert_eq!(
            rail.offset_at(rail.bounds.y + rail.bounds.height),
            rail.max_offset
        );

        // Dragging past either end clamps instead of overshooting.
        assert_eq!(rail.offset_at(rail.bounds.y - 100.0), 0.0);
        assert_eq!(
            rail.offset_at(rail.bounds.y + rail.bounds.height + 100.0),
            rail.max_offset
        );
    }

    #[test]
    fn grabbing_the_track_centers_the_scroller() {
        let rail = Rail::vertical(viewport(), 600.0, 0.0).unwrap();
        let scroller_x = rail.scroller.center_x();

        assert_eq!(
            rail.grab(Point::new(scroller_x, rail.scroller.y + 4.0)),
            4.0
        );
        assert_eq!(
            rail.grab(Point::new(scroller_x, rail.scroller.y + 4.0)),
            4.0
        );
        assert_eq!(
            rail.grab(Point::new(
                scroller_x,
                rail.scroller.y + rail.scroller.height + 10.0
            )),
            rail.scroller.height / 2.0
        );

        // The scroller hugs the trailing edge of the viewport.
        assert!(rail.scroller.x > viewport().center_x());
    }
}
