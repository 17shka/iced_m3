rust_i18n::i18n!("locales", fallback = "en");

use iced::{
    Element, Length,
    widget::{column, stack, text},
};
use iced_m3::theme::{Accent, ColorScheme, Theme};
use iced_m3::widget::{self, button, dialog, select::Item, text_input::TextInput};
use rust_i18n::t;

pub fn init() {
    rust_i18n::extend!(iced_m3);
    rust_i18n::set_locale("en");
}

#[derive(Clone)]
pub enum Message {
    SetLocale(Option<String>),
    TextChanged(String),
    OptionSelected(Option<u8>),
    ToggleDialog,
    CloseDialog,
}

pub struct Page {
    locale: String,
    text: String,
    option: Option<u8>,
    dialog_open: bool,
}

impl Default for Page {
    fn default() -> Self {
        Self {
            locale: rust_i18n::locale().to_string(),
            text: String::new(),
            option: None,
            dialog_open: false,
        }
    }
}

impl Page {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::SetLocale(Some(locale)) => {
                // Shared by the application and the library.
                rust_i18n::set_locale(&locale);
                self.locale = locale;
            }
            Message::SetLocale(None) => {}
            Message::TextChanged(text) => self.text = text,
            Message::OptionSelected(option) => self.option = option,
            Message::ToggleDialog => self.dialog_open = true,
            Message::CloseDialog => self.dialog_open = false,
        }
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        iced::Subscription::none()
    }

    pub fn view<'a>(&'a self, theme: &'a Theme) -> Element<'a, Message> {
        let placeholder = t!("app.text_placeholder");

        let options = vec![
            Item::labelled(0u8, t!("app.option.alpha").into_owned()),
            Item::labelled(1u8, t!("app.option.beta").into_owned()),
            Item::labelled(2u8, t!("app.option.gamma").into_owned()),
        ];

        let content = column![
            heading(theme, t!("app.language")),
            widget::language_select(theme, Some(self.locale.clone()), Message::SetLocale)
                .placeholder(t!("app.language_placeholder").into_owned())
                .width(Length::Fixed(280.0)),
            body_text(theme, t!("app.intro")),
            heading(theme, t!("app.text_input")),
            body_text(theme, t!("app.text_input_hint")),
            TextInput::new(&placeholder, &self.text, theme)
                .on_input(Message::TextChanged)
                .width(Length::Fixed(320.0)),
            heading(theme, t!("app.select")),
            widget::select(theme, options, self.option, Message::OptionSelected)
                .placeholder(t!("app.select_placeholder").into_owned())
                .searchable(true)
                .width(Length::Fixed(280.0)),
            heading(theme, t!("app.dialog")),
            filled_button(theme, t!("app.open_dialog"), Message::ToggleDialog),
        ]
        .spacing(16);

        let dialog_element = dialog::Dialog::new(
            dialog::Style::new(theme),
            body_text(theme, t!("app.dialog_body")),
            vec![dialog::Button {
                on_press: Some(Message::CloseDialog),
                label: t!("app.dialog_close").into_owned(),
                style: button::Style::filled(theme, Accent::Primary),
            }],
        )
        .on_press_outside(Message::CloseDialog)
        .open(self.dialog_open);

        stack![content, dialog_element].into()
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

common::demo!(Page, Message, "Localization Demo");

#[cfg(test)]
mod tests {
    use super::*;
    use iced_m3::widget::text_input::Action;
    use std::sync::Once;

    static INIT: Once = Once::new();

    #[test]
    fn application_and_library_keys_follow_the_locale() {
        INIT.call_once(init);

        let locales = iced_m3::i18n::available_locales();
        assert!(
            locales
                .iter()
                .any(|(code, name)| code == "en" && name == "English"),
            "English should be listed: {locales:?}"
        );
        assert!(
            locales
                .iter()
                .any(|(code, name)| code == "ru" && name == "Русский"),
            "Russian should be listed: {locales:?}"
        );

        rust_i18n::set_locale("ru");
        assert_eq!(&*t!("app.open_dialog"), "Открыть диалог");
        assert_eq!(&*Action::Copy.label(), "Копировать");
        assert_eq!(&*Action::SelectAll.label(), "Выделить всё");
        assert_eq!(&*t!("iced_m3.widget.theme.dark"), "Тёмная");

        rust_i18n::set_locale("en");
        assert_eq!(&*t!("app.open_dialog"), "Open dialog");
        assert_eq!(&*Action::Copy.label(), "Copy");
    }
}
