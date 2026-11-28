use iced::{Point, Rectangle, Size};

pub(crate) fn centered_in(container: Size, size: Size) -> Point {
    Point::new(
        (container.width - size.width) / 2.0,
        (container.height - size.height) / 2.0,
    )
}

pub(crate) fn clamp_into_viewport(position: Point, size: Size, viewport: Size) -> Point {
    Point::new(
        position
            .x
            .clamp(0.0, (viewport.width - size.width).max(0.0)),
        position
            .y
            .clamp(0.0, (viewport.height - size.height).max(0.0)),
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DropDirection {
    Down,
    Up,
    Center,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct VerticalFit {
    pub y: f32,
    pub height: f32,
}

pub(crate) fn fit_vertically(
    anchor: Rectangle,
    viewport_height: f32,
    want: f32,
    gap: f32,
    margin: f32,
    direction: DropDirection,
) -> VerticalFit {
    let above = (anchor.y - gap - margin).max(0.0);
    let below = (viewport_height - margin - anchor.y - anchor.height - gap).max(0.0);

    let (y, height) = match direction {
        DropDirection::Down => {
            if want <= below {
                (anchor.y + anchor.height + gap, want)
            } else if want <= above {
                (anchor.y - gap - want, want)
            } else if below >= above {
                (anchor.y + anchor.height + gap, below)
            } else {
                (anchor.y - gap - above, above)
            }
        }
        DropDirection::Up => {
            if want <= above {
                (anchor.y - gap - want, want)
            } else if want <= below {
                (anchor.y + anchor.height + gap, want)
            } else if above >= below {
                (anchor.y - gap - above, above)
            } else {
                (anchor.y + anchor.height + gap, below)
            }
        }
        DropDirection::Center => {
            let available = (viewport_height - 2.0 * margin).max(0.0);
            let height = want.min(available);
            let max_y = (viewport_height - margin - height).max(margin);
            let anchor_center = anchor.y + anchor.height / 2.0;

            ((anchor_center - height / 2.0).clamp(margin, max_y), height)
        }
    };

    VerticalFit { y, height }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn centered_in_centers_the_box() {
        assert_eq!(
            centered_in(Size::new(800.0, 600.0), Size::new(200.0, 100.0)),
            Point::new(300.0, 250.0)
        );
    }

    #[test]
    fn clamp_into_viewport_keeps_surface_inside() {
        let viewport = Size::new(800.0, 600.0);
        let surface = Size::new(192.0, 200.0);

        assert_eq!(
            clamp_into_viewport(Point::new(100.0, 100.0), surface, viewport),
            Point::new(100.0, 100.0)
        );
        assert_eq!(
            clamp_into_viewport(Point::new(780.0, 580.0), surface, viewport),
            Point::new(608.0, 400.0)
        );
        assert_eq!(
            clamp_into_viewport(Point::new(-20.0, -20.0), surface, viewport),
            Point::new(0.0, 0.0)
        );
    }

    #[test]
    fn clamp_into_viewport_pins_to_origin_when_too_large() {
        let viewport = Size::new(100.0, 80.0);
        let surface = Size::new(192.0, 200.0);

        assert_eq!(
            clamp_into_viewport(Point::new(10.0, 10.0), surface, viewport),
            Point::new(0.0, 0.0)
        );
    }

    #[test]
    fn fit_vertically_opens_down_when_it_fits() {
        let anchor = Rectangle::new(Point::new(0.0, 100.0), Size::new(100.0, 40.0));

        let fit = fit_vertically(anchor, 800.0, 100.0, 8.0, 8.0, DropDirection::Down);

        assert_eq!(
            fit,
            VerticalFit {
                y: 148.0,
                height: 100.0
            }
        );
    }

    #[test]
    fn fit_vertically_flips_up_when_below_is_full() {
        let anchor = Rectangle::new(Point::new(0.0, 150.0), Size::new(100.0, 40.0));

        let fit = fit_vertically(anchor, 200.0, 50.0, 8.0, 8.0, DropDirection::Down);

        assert_eq!(
            fit,
            VerticalFit {
                y: 92.0,
                height: 50.0
            }
        );
    }

    #[test]
    fn fit_vertically_shrinks_to_the_larger_side() {
        let anchor = Rectangle::new(Point::new(0.0, 150.0), Size::new(100.0, 40.0));

        let fit = fit_vertically(anchor, 200.0, 200.0, 8.0, 8.0, DropDirection::Down);

        assert_eq!(
            fit,
            VerticalFit {
                y: 8.0,
                height: 134.0
            }
        );
    }

    #[test]
    fn fit_vertically_centers_on_the_anchor() {
        let anchor = Rectangle::new(Point::new(0.0, 300.0), Size::new(100.0, 40.0));

        let fit = fit_vertically(anchor, 800.0, 100.0, 8.0, 8.0, DropDirection::Center);

        assert_eq!(
            fit,
            VerticalFit {
                y: 270.0,
                height: 100.0
            }
        );
    }
}
