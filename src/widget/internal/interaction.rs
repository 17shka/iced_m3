use iced::{Event, mouse, touch};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PressUpdate {
    Ignored,
    Pressed,
    Released,
    ReleasedOutside,
    Lost,
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
