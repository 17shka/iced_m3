use std::borrow::Cow;
use std::fmt;

use iced::advanced::svg;
use iced_widget::core::keyboard::{self, key};

use crate::widget::common_icons;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Action {
    Undo,
    Redo,
    Copy,
    Cut,
    Paste,
    SelectWord,
    SelectAll,
}

impl Action {
    pub const ALL: [Self; 7] = [
        Self::Copy,
        Self::Cut,
        Self::Paste,
        Self::SelectWord,
        Self::SelectAll,
        Self::Undo,
        Self::Redo,
    ];


    pub fn label(self) -> Cow<'static, str> {
        use crate::i18n;

        match self {
            Self::Undo => i18n::text_input_undo(),
            Self::Redo => i18n::text_input_redo(),
            Self::Copy => i18n::text_input_copy(),
            Self::Cut => i18n::text_input_cut(),
            Self::Paste => i18n::text_input_paste(),
            Self::SelectWord => i18n::text_input_select_word(),
            Self::SelectAll => i18n::text_input_select_all(),
        }
    }

    pub fn shortcut(self) -> Option<Shortcut> {
        let command = keyboard::Modifiers::COMMAND;

        Some(match self {
            Self::Undo => Shortcut::character(command, 'z'),
            #[cfg(target_os = "macos")]
            Self::Redo => Shortcut::character(command | keyboard::Modifiers::SHIFT, 'z'),
            #[cfg(not(target_os = "macos"))]
            Self::Redo => Shortcut::character(command, 'y'),
            Self::Copy => Shortcut::character(command, 'c'),
            Self::Cut => Shortcut::character(command, 'x'),
            Self::Paste => Shortcut::character(command, 'v'),
            Self::SelectWord => return None,
            Self::SelectAll => Shortcut::character(command, 'a'),
        })
    }

    pub fn shortcut_text(self) -> Option<String> {
        self.shortcut().map(|shortcut| shortcut.to_string())
    }

    pub(super) fn icon(self) -> svg::Handle {
        match self {
            Self::Undo => common_icons::UNDO.clone(),
            Self::Redo => common_icons::REDO.clone(),
            Self::Copy => common_icons::COPY.clone(),
            Self::Cut => common_icons::CUT.clone(),
            Self::Paste => common_icons::PASTE.clone(),
            Self::SelectWord => common_icons::SELECT_WORD.clone(),
            Self::SelectAll => common_icons::SELECT_ALL.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Shortcut {
    pub modifiers: keyboard::Modifiers,
    pub key: keyboard::Key,
}

impl Shortcut {
    pub fn new(modifiers: keyboard::Modifiers, key: keyboard::Key) -> Self {
        Self { modifiers, key }
    }

    pub fn character(modifiers: keyboard::Modifiers, character: char) -> Self {
        Self::new(
            modifiers,
            keyboard::Key::Character(character.to_string().into()),
        )
    }
}

impl fmt::Display for Shortcut {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let modifiers = self.modifiers;

        if modifiers.control() {
            f.write_str(if cfg!(target_os = "macos") {
                "⌃"
            } else {
                "Ctrl+"
            })?;
        }

        if modifiers.alt() {
            f.write_str(if cfg!(target_os = "macos") {
                "⌥"
            } else {
                "Alt+"
            })?;
        }

        if modifiers.shift() {
            f.write_str(if cfg!(target_os = "macos") {
                "⇧"
            } else {
                "Shift+"
            })?;
        }

        if modifiers.logo() {
            f.write_str(if cfg!(target_os = "macos") {
                "⌘"
            } else {
                "Super+"
            })?;
        }

        f.write_str(&key_label(&self.key))
    }
}

fn key_label(key: &keyboard::Key) -> String {
    match key {
        keyboard::Key::Character(character) => character.to_uppercase(),
        keyboard::Key::Named(named) => named_label(*named)
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| format!("{named:?}")),
        keyboard::Key::Unidentified => "?".to_owned(),
    }
}

fn named_label(named: key::Named) -> Option<&'static str> {
    Some(match named {
        key::Named::Enter => "Enter",
        key::Named::Space => "Space",
        key::Named::Tab => "Tab",
        key::Named::Escape => "Esc",
        key::Named::Backspace => "Backspace",
        key::Named::Delete => "Delete",
        key::Named::Insert => "Insert",
        key::Named::Home => "Home",
        key::Named::End => "End",
        key::Named::PageUp => "PageUp",
        key::Named::PageDown => "PageDown",
        key::Named::ArrowUp => "↑",
        key::Named::ArrowDown => "↓",
        key::Named::ArrowLeft => "←",
        key::Named::ArrowRight => "→",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_action_has_a_label() {
        for action in Action::ALL {
            assert!(!action.label().is_empty());
        }
    }

    #[test]
    fn select_word_has_no_shortcut() {
        assert!(Action::SelectWord.shortcut().is_none());
        assert!(Action::SelectWord.shortcut_text().is_none());
    }

    #[test]
    fn shortcuts_match_the_context_menu() {
        for action in Action::ALL
            .into_iter()
            .filter(|action| *action != Action::SelectWord)
        {
            assert!(
                action.shortcut().is_some(),
                "{action:?} should have a shortcut"
            );
        }
    }

    #[test]
    fn shortcuts_are_rendered_for_the_platform() {
        let copy = Action::Copy.shortcut().expect("a shortcut");

        assert_eq!(copy.modifiers, keyboard::Modifiers::COMMAND);

        #[cfg(target_os = "macos")]
        assert_eq!(copy.to_string(), "⌘C");

        #[cfg(not(target_os = "macos"))]
        assert_eq!(copy.to_string(), "Ctrl+C");
    }

    #[test]
    fn named_keys_are_rendered() {
        let shortcut = Shortcut::new(
            keyboard::Modifiers::COMMAND,
            keyboard::Key::Named(key::Named::Enter),
        );

        #[cfg(target_os = "macos")]
        assert_eq!(shortcut.to_string(), "⌘Enter");

        #[cfg(not(target_os = "macos"))]
        assert_eq!(shortcut.to_string(), "Ctrl+Enter");
    }
}
