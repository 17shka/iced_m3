use iced::{Event, Rectangle, mouse, touch};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PressUpdate {
    Ignored,
    Pressed,
    Released,
    ReleasedOutside,
    Lost,
}

/// The interaction for an `enabled` interactive widget whose `bounds`
/// contain the cursor: a pointer hand when interactive, otherwise the
/// default interaction.
pub fn pointer_when(enabled: bool, cursor: mouse::Cursor, bounds: Rectangle) -> mouse::Interaction {
    if enabled && cursor.is_over(bounds) {
        mouse::Interaction::Pointer
    } else {
        mouse::Interaction::default()
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
