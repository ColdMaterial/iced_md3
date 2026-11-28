use iced::{Event, Rectangle, advanced::Shell, mouse, touch};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PressUpdate {
    Ignored,
    Pressed,
    Released,
    ReleasedOutside,
    Lost,
}

pub fn pointer_when(
    enabled: bool,
    cursor: mouse::Cursor,
    bounds: Rectangle,
    interaction: mouse::Interaction,
) -> mouse::Interaction {
    if enabled && cursor.is_over(bounds) {
        interaction
    } else {
        mouse::Interaction::default()
    }
}

pub fn hovered(event: &Event, bounds: Rectangle, cursor: mouse::Cursor) -> bool {
    match event {
        Event::Touch(touch::Event::FingerPressed { position, .. })
        | Event::Touch(touch::Event::FingerLifted { position, .. }) => bounds.contains(*position),
        _ => cursor.is_over(bounds),
    }
}

pub fn update_press(
    event: &Event,
    is_pressed: &mut bool,
    is_allowed: bool,
    is_hovered: bool,
) -> PressUpdate {
    match event {
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
        | Event::Touch(touch::Event::FingerPressed { .. }) => {
            if is_allowed && is_hovered {
                *is_pressed = true;
                PressUpdate::Pressed
            } else {
                PressUpdate::Ignored
            }
        }
        Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
        | Event::Touch(touch::Event::FingerLifted { .. }) => {
            if is_allowed && *is_pressed {
                *is_pressed = false;

                if is_hovered {
                    PressUpdate::Released
                } else {
                    PressUpdate::ReleasedOutside
                }
            } else {
                PressUpdate::Ignored
            }
        }
        Event::Touch(touch::Event::FingerLost { .. }) => {
            *is_pressed = false;
            PressUpdate::Lost
        }
        _ => PressUpdate::Ignored,
    }
}

pub fn toggle_press<Message>(
    event: &Event,
    is_pressed: &mut bool,
    is_hovered: bool,
    enabled: bool,
    on_toggle: Option<&Message>,
    shell: &mut Shell<'_, Message>,
) where
    Message: Clone,
{
    match event {
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) if is_hovered && enabled => {
            shell.capture_event();
            *is_pressed = true;
        }
        Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) if *is_pressed => {
            shell.capture_event();
            *is_pressed = false;
            if is_hovered && let Some(on_toggle) = on_toggle {
                shell.publish(on_toggle.clone());
            }
        }
        _ => {}
    }
}
