rust_i18n::i18n!("locales", fallback = "en");

use iced::{
    Element, Length,
    widget::{column, stack, text},
};
use iced_m3::theme::{Accent, ColorScheme, Theme};
use iced_m3::widget::{
    self, button, dialog,
    select::{self, Item, Select},
    text_input::{self, ActionLabels, TextInput},
};
use rust_i18n::t;

pub fn init() {
    rust_i18n::set_locale("en");
}

#[derive(Clone)]
pub enum Message {
    SetLocale(Option<String>),
    TextChanged(String),
    PasswordChanged(String),
    TogglePasswordVisible,
    OptionSelected(Option<u8>),
    ToggleDialog,
    CloseDialog,
}

pub struct Page {
    locale: String,
    text: String,
    password: String,
    password_visible: bool,
    option: Option<u8>,
    dialog_open: bool,
}

impl Default for Page {
    fn default() -> Self {
        Self {
            locale: rust_i18n::locale().to_string(),
            text: String::new(),
            password: String::new(),
            password_visible: false,
            option: None,
            dialog_open: false,
        }
    }
}

impl Page {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::SetLocale(Some(locale)) => {
                rust_i18n::set_locale(&locale);
                self.locale = locale;
            }
            Message::SetLocale(None) => {}
            Message::TextChanged(text) => self.text = text,
            Message::PasswordChanged(password) => self.password = password,
            Message::TogglePasswordVisible => self.password_visible = !self.password_visible,
            Message::OptionSelected(option) => self.option = option,
            Message::ToggleDialog => self.dialog_open = true,
            Message::CloseDialog => self.dialog_open = false,
        }
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        iced::Subscription::none()
    }

    /// Locales are read from `locales/*.yml`, each listed in its own language.
    fn language_select<'a>(&'a self, theme: &Theme) -> Select<'a, String, Message> {
        widget::select(
            theme,
            language_items(),
            Some(self.locale.clone()),
            Message::SetLocale,
        )
        .placeholder(t!("app.language_placeholder").into_owned())
        .searchable(false)
        .width(Length::Fixed(280.0))
    }

    pub fn view<'a>(&'a self, theme: &'a Theme) -> Element<'a, Message> {
        let text_placeholder = t!("app.text_placeholder").into_owned();
        let password_placeholder = t!("app.password_placeholder").into_owned();

        let options = vec![
            Item::labelled(0u8, t!("app.option.alpha").into_owned()),
            Item::labelled(1u8, t!("app.option.beta").into_owned()),
            Item::labelled(2u8, t!("app.option.gamma").into_owned()),
        ];

        let actions = action_labels();

        let input_labels = text_input::Labels {
            clear: t!("iced_m3.text_input.clear"),
            show_password: t!("iced_m3.text_input.show_password"),
            hide_password: t!("iced_m3.text_input.hide_password"),
            actions: actions.clone(),
        };

        let select_labels = select::Labels {
            search: t!("iced_m3.select.search"),
            clear: t!("iced_m3.select.clear"),
            clear_search: t!("iced_m3.select.clear_search"),
            actions,
        };

        let content = column![
            heading(theme, t!("app.language")),
            self.language_select(theme),
            heading(theme, t!("app.text_input")),
            TextInput::new(&text_placeholder, &self.text, theme)
                .on_input(Message::TextChanged)
                .clearable(true)
                .labels(input_labels.clone())
                .width(Length::Fixed(320.0)),
            heading(theme, t!("app.password")),
            TextInput::new(&password_placeholder, &self.password, theme)
                .on_input(Message::PasswordChanged)
                .secure(!self.password_visible)
                .on_toggle_secure(Message::TogglePasswordVisible)
                .labels(input_labels)
                .width(Length::Fixed(320.0)),
            heading(theme, t!("app.select")),
            widget::select(theme, options, self.option, Message::OptionSelected)
                .placeholder(t!("app.select_placeholder").into_owned())
                .searchable(true)
                .clearable(true)
                .labels(select_labels)
                .width(Length::Fixed(280.0)),
            heading(theme, t!("app.dialog")),
            filled_button(theme, t!("app.open_dialog"), Message::ToggleDialog),
        ]
        .spacing(16);

        let dialog = dialog::Dialog::new(
            dialog::Style::new(theme),
            body_text(theme, t!("app.dialog_body")),
            vec![dialog::Button {
                on_press: Some(Message::CloseDialog),
                label: t!("app.dialog_close").into_owned(),
                style: button::Style::filled(theme, Accent::Primary),
            }],
        )
        .title(t!("app.dialog_title").into_owned())
        .labels(dialog::Labels {
            close: t!("iced_m3.dialog.close"),
        })
        .on_press_outside(Message::CloseDialog)
        .open(self.dialog_open);

        stack![content, dialog].into()
    }
}

fn heading<'a>(theme: &Theme, label: impl Into<String>) -> Element<'a, Message> {
    text(label.into())
        .font(fonts::text_bold())
        .size(18.0)
        .color(theme.on_surface())
        .into()
}

fn body_text<'a>(theme: &Theme, label: impl Into<String>) -> Element<'a, Message> {
    text(label.into())
        .font(fonts::text_regular())
        .size(16.0)
        .color(theme.on_surface_variant())
        .into()
}

fn filled_button<'a>(
    theme: &Theme,
    label: impl Into<String>,
    message: Message,
) -> Element<'a, Message> {
    widget::button(
        button::Style::filled(theme, Accent::Primary),
        button::Content::Label(label.into().into()),
    )
    .on_press(message)
    .into()
}

fn action_labels<'a>() -> ActionLabels<'a> {
    ActionLabels {
        undo: t!("iced_m3.actions.undo"),
        redo: t!("iced_m3.actions.redo"),
        copy: t!("iced_m3.actions.copy"),
        cut: t!("iced_m3.actions.cut"),
        paste: t!("iced_m3.actions.paste"),
        select_word: t!("iced_m3.actions.select_word"),
        select_all: t!("iced_m3.actions.select_all"),
    }
}

/// Every bundled locale, labelled in its own language.
fn language_items() -> Vec<Item<String>> {
    rust_i18n::available_locales!()
        .into_iter()
        .map(|locale| {
            let name = t!("app.language_name", locale = locale.as_ref()).into_owned();
            Item::labelled(locale.into_owned(), name)
        })
        .collect()
}

common::demo!(Page, Message, "Localization Demo");

#[cfg(test)]
mod tests {
    use super::*;
    use iced_m3::widget::text_input::Action;
    use std::sync::Once;

    static INIT: Once = Once::new();

    #[test]
    fn widget_text_follows_the_application_locales() {
        INIT.call_once(init);

        assert_eq!(t!("iced_m3.select.search").as_ref(), "Find");
        assert_eq!(t!("iced_m3.text_input.clear").as_ref(), "Reset");
        assert_eq!(t!("iced_m3.dialog.close").as_ref(), "Close");

        rust_i18n::set_locale("ru");
        assert_eq!(t!("iced_m3.actions.copy").as_ref(), "Копировать");
        assert_eq!(t!("iced_m3.select.search").as_ref(), "Поиск");
        assert_eq!(t!("iced_m3.dialog.close").as_ref(), "Закрыть");

        let labels = action_labels();
        assert_eq!(labels.get(Action::Copy), "Копировать");
        assert_eq!(labels.get(Action::SelectAll), "Выделить всё");

        rust_i18n::set_locale("en");
        assert_eq!(t!("iced_m3.select.search").as_ref(), "Find");

        let labels = action_labels();
        assert_eq!(labels.get(Action::Copy), "Copy");
    }

    #[test]
    fn language_select_lists_every_available_locale_in_its_own_language() {
        INIT.call_once(init);

        let labels: std::collections::HashMap<_, _> = language_items()
            .into_iter()
            .map(|item| {
                (
                    item.value().cloned().expect("a locale"),
                    item.label().expect("a name").to_owned(),
                )
            })
            .collect();

        assert_eq!(labels.get("en").map(String::as_str), Some("English"));
        assert_eq!(labels.get("ru").map(String::as_str), Some("Русский"));
    }
}
