use iced::{Element, widget::row};
use iced_m3::{
    theme::{Accent, ColorScheme, Mode},
    widget::button,
};

/// A row of buttons to switch between light and dark themes.
#[must_use]
pub fn theme_switcher<'a, Message>(
    theme: &impl ColorScheme,
    on_change: impl Fn(Mode) -> Message,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    row![
        button(
            button::Style::filled(theme, Accent::Primary),
            button::Content::Label("Light".into()),
        )
        .on_press(on_change(Mode::Light)),
        button(
            button::Style::filled(theme, Accent::Secondary),
            button::Content::Label("Dark".into()),
        )
        .on_press(on_change(Mode::Dark)),
    ]
    .spacing(8)
    .into()
}
