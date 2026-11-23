use crate::{
    i18n,
    theme::{ColorScheme, Mode},
};

use super::{Item, Select};

#[must_use]
pub fn theme_select<'a, Message>(
    theme: &(impl ColorScheme + ?Sized),
    selected: Mode,
    on_selected: impl Fn(Mode) -> Message + 'a,
) -> Select<'a, Mode, Message>
where
    Message: Clone,
{
    let items = [
        (Mode::System, i18n::theme_system()),
        (Mode::Light, i18n::theme_light()),
        (Mode::Dark, i18n::theme_dark()),
    ]
    .into_iter()
    .map(|(mode, label)| Item::labelled(mode, label))
    .collect::<Vec<_>>();

    Select::new(theme, items, Some(selected), move |mode| {
        on_selected(mode.unwrap_or_default())
    })
}
