use iced::time::Instant;

use crate::animation::{
    Interpolable,
    motion::{SimulatedSpring, Spring, SpringValue},
};

/// Keeps a widget's style spring in sync with its current style.
///
/// A widget may be given a style externally (for example a custom motion
/// scheme). The first time it notices such a change the spring is *reset* so
/// the new style is applied immediately instead of animating from the default
/// one; afterwards the spring simply tracks `target`.
///
/// Returns `true` when the spring was reset, which callers can use to request a
/// redraw.
pub(crate) fn sync_style_spring<S, T>(
    style_spring: &mut SpringValue<T>,
    style_checked: &mut bool,
    last_style: &mut S,
    style: S,
    target: T,
    now: Instant,
) -> bool
where
    S: PartialEq + Clone,
    T: Interpolable + Clone + PartialEq,
{
    if !*style_checked && *last_style != style {
        style_spring.reset(target, now);
        last_style.clone_from(&style);
        *style_checked = true;
        true
    } else {
        style_spring.set_target(target, now);
        false
    }
}

pub(crate) fn sync_spring_model(
    last: &mut Spring,
    target: Option<Spring>,
    springs: &mut [&mut SimulatedSpring],
) {
    let Some(target) = target else {
        return;
    };

    if *last == target {
        return;
    }

    *last = target;
    for spring in springs {
        spring.model = target;
    }
}
