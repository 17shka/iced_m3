use crate::{i18n, theme::ColorScheme};

use super::{Item, Select};

#[must_use]
pub fn language_select<'a, Message>(
    theme: &(impl ColorScheme + ?Sized),
    selected: Option<String>,
    on_selected: impl Fn(Option<String>) -> Message + 'a,
) -> Select<'a, String, Message>
where
    Message: Clone,
{
    let items = i18n::available_locales()
        .into_iter()
        .map(|(locale, name)| Item::labelled(locale, name))
        .collect::<Vec<_>>();

    Select::new(theme, items, selected, on_selected)
}
