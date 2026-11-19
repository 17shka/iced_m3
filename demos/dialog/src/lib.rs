use iced::{Element, Length, widget::{column, text, Space}};
use iced_m3::theme::{Accent, ColorScheme, Theme};
use iced_m3::widget::{button, dialog};

#[derive(Clone)]
pub enum Message {
    ToggleDialog,
    CloseDialog,
    ToggleCustom,
    CloseCustom,
}

pub struct Page {
    dialog_open: bool,
    dialog_custom_open: bool,
}

impl Default for Page {
    fn default() -> Self {
        Self {
            dialog_open: false,
            dialog_custom_open: false,
        }
    }
}

impl Page {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::ToggleDialog => self.dialog_open = !self.dialog_open,
            Message::CloseDialog => self.dialog_open = false,
            Message::ToggleCustom => self.dialog_custom_open = !self.dialog_custom_open,
            Message::CloseCustom => self.dialog_custom_open = false,
        }
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        iced::Subscription::none()
    }

    pub fn view(&self, theme: &Theme) -> Element<'_, Message> {
        let dialog_element = if self.dialog_open {
            dialog::Dialog::new(
                dialog::Style::new(theme),
                text("This is a simple dialog with standard actions.")
                    .color(theme.on_surface()),
                vec![
                    dialog::Button {
                        on_press: Some(Message::CloseDialog),
                        label: "Cancel".into(),
                        style: button::Style::text(theme, Accent::Primary),
                    },
                    dialog::Button {
                        on_press: Some(Message::CloseDialog),
                        label: "OK".into(),
                        style: button::Style::filled(theme, Accent::Primary),
                    },
                ],
            )
            .title("Simple Dialog")
            .on_press_outside(Message::CloseDialog)
            .into()
        } else {
            Element::from(Space::new().width(Length::Shrink).height(Length::Shrink))
        };

        let custom_dialog_element = if self.dialog_custom_open {
            dialog::Dialog::new(
                dialog::Style::new(theme),
                text("You can put any widget here.").color(theme.on_surface_variant()),
                vec![dialog::Button {
                    on_press: Some(Message::CloseCustom),
                    label: "Dismiss".into(),
                    style: button::Style::text(theme, Accent::Primary),
                }],
            )
            .title("Custom Dialog")
            .on_press_outside(Message::CloseCustom)
            .into()
        } else {
            Element::from(Space::new().width(Length::Shrink).height(Length::Shrink))
        };

        column![
            section_header("Simple dialog", theme),
            iced_m3::widget::button(
                button::Style::filled(theme, Accent::Primary),
                button::Content::Label("Open dialog".into()),
            )
            .on_press(Message::ToggleDialog),
            section_header("Custom content dialog", theme),
            iced_m3::widget::button(
                button::Style::filled(theme, Accent::Primary),
                button::Content::Label("Open custom dialog".into()),
            )
            .on_press(Message::ToggleCustom),
            dialog_element,
            custom_dialog_element,
        ]
        .spacing(24)
        .into()
    }
}

fn section_header<'a>(title: &'a str, theme: &Theme) -> text::Text<'a> {
    text(title)
        .font(fonts::text_bold())
        .size(18.0)
        .color(theme.on_surface())
}
