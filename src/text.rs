use std::{borrow::Cow, rc::Rc};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Text {
    DialogClose,
    SelectSearch,
    SelectClear,
    SelectClearSearch,
    NavRailExpandMenu,
    NavRailCollapseMenu,
    FabMenuOpenMenu,
    FabMenuCloseMenu,
    ThemeSystem,
    ThemeLight,
    ThemeDark,
    TextInputUndo,
    TextInputRedo,
    TextInputCopy,
    TextInputCut,
    TextInputPaste,
    TextInputSelectWord,
    TextInputSelectAll,
    TextInputClear,
    TextInputShowPassword,
    TextInputHidePassword,
}

impl Text {
    pub const ALL: [Self; 21] = [
        Self::DialogClose,
        Self::SelectSearch,
        Self::SelectClear,
        Self::SelectClearSearch,
        Self::NavRailExpandMenu,
        Self::NavRailCollapseMenu,
        Self::FabMenuOpenMenu,
        Self::FabMenuCloseMenu,
        Self::ThemeSystem,
        Self::ThemeLight,
        Self::ThemeDark,
        Self::TextInputUndo,
        Self::TextInputRedo,
        Self::TextInputCopy,
        Self::TextInputCut,
        Self::TextInputPaste,
        Self::TextInputSelectWord,
        Self::TextInputSelectAll,
        Self::TextInputClear,
        Self::TextInputShowPassword,
        Self::TextInputHidePassword,
    ];

    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::DialogClose => "dialog.close",
            Self::SelectSearch => "select.search",
            Self::SelectClear => "select.clear",
            Self::SelectClearSearch => "select.clear_search",
            Self::NavRailExpandMenu => "navrail.expand_menu",
            Self::NavRailCollapseMenu => "navrail.collapse_menu",
            Self::FabMenuOpenMenu => "fab_menu.open_menu",
            Self::FabMenuCloseMenu => "fab_menu.close_menu",
            Self::ThemeSystem => "theme.system",
            Self::ThemeLight => "theme.light",
            Self::ThemeDark => "theme.dark",
            Self::TextInputUndo => "text_input.undo",
            Self::TextInputRedo => "text_input.redo",
            Self::TextInputCopy => "text_input.copy",
            Self::TextInputCut => "text_input.cut",
            Self::TextInputPaste => "text_input.paste",
            Self::TextInputSelectWord => "text_input.select_word",
            Self::TextInputSelectAll => "text_input.select_all",
            Self::TextInputClear => "text_input.clear",
            Self::TextInputShowPassword => "text_input.show_password",
            Self::TextInputHidePassword => "text_input.hide_password",
        }
    }

    #[must_use]
    pub const fn english(self) -> &'static str {
        match self {
            Self::DialogClose => "Close",
            Self::SelectSearch => "Search",
            Self::SelectClear => "Clear selection",
            Self::SelectClearSearch => "Clear search",
            Self::NavRailExpandMenu => "Expand menu",
            Self::NavRailCollapseMenu => "Collapse menu",
            Self::FabMenuOpenMenu => "Open menu",
            Self::FabMenuCloseMenu => "Close menu",
            Self::ThemeSystem => "System",
            Self::ThemeLight => "Light",
            Self::ThemeDark => "Dark",
            Self::TextInputUndo => "Undo",
            Self::TextInputRedo => "Redo",
            Self::TextInputCopy => "Copy",
            Self::TextInputCut => "Cut",
            Self::TextInputPaste => "Paste",
            Self::TextInputSelectWord => "Select word",
            Self::TextInputSelectAll => "Select all",
            Self::TextInputClear => "Clear",
            Self::TextInputShowPassword => "Show password",
            Self::TextInputHidePassword => "Hide password",
        }
    }
}

pub(crate) type Translator<'a> = Rc<dyn Fn(Text) -> String + 'a>;

pub(crate) fn resolve<'a>(translator: Option<&'a Translator<'_>>, text: Text) -> Cow<'a, str> {
    match translator {
        Some(translate) => Cow::Owned(translate(text)),
        None => Cow::Borrowed(text.english()),
    }
}

#[cfg(test)]
mod tests {
    use super::{Text, resolve};

    #[test]
    fn all_text_keys_have_non_empty_english_defaults() {
        for text in Text::ALL {
            assert!(
                !text.english().is_empty(),
                "{text:?} has no English default"
            );
        }
    }

    #[test]
    fn every_text_key_is_unique() {
        for (index, text) in Text::ALL.iter().enumerate() {
            assert!(
                Text::ALL[index + 1..]
                    .iter()
                    .all(|other| other.key() != text.key()),
                "{text:?} reuses the key {:?}",
                text.key()
            );
        }
    }

    #[test]
    fn resolve_falls_back_to_english() {
        assert_eq!(resolve(None, Text::TextInputCopy), "Copy");
    }

    #[test]
    fn resolve_prefers_the_translator() {
        let translate: super::Translator<'_> =
            std::rc::Rc::new(|text: Text| format!("translated:{}", text.key()));

        assert_eq!(
            resolve(Some(&translate), Text::TextInputCopy),
            "translated:text_input.copy"
        );
    }
}
