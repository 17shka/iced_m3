use iced::{
    Element,
    widget::{column, row, stack, text},
};
use iced_m3::animation::motion::Speed;
use iced_m3::theme::{Accent, ColorScheme, Theme};
use iced_m3::widget::{button, dialog};

#[derive(Clone)]
pub enum Message {
    ToggleDialog,
    CloseDialog,
    ToggleCustom,
    CloseCustom,
    SetSpeed(Speed),
}

pub struct Page {
    dialog_open: bool,
    dialog_custom_open: bool,
    speed: Speed,
}

impl Default for Page {
    fn default() -> Self {
        Self {
            dialog_open: false,
            dialog_custom_open: false,
            speed: Speed::Default,
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
            Message::SetSpeed(speed) => self.speed = speed,
        }
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        iced::Subscription::none()
    }

    pub fn view(&self, theme: &Theme) -> Element<'_, Message> {
        let content = column![
            section_header("Dialog speed", theme),
            row![
                speed_button(theme, "Fast", Speed::Fast, self.speed),
                speed_button(theme, "Default", Speed::Default, self.speed),
                speed_button(theme, "Slow", Speed::Slow, self.speed),
            ]
            .spacing(8),
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
        ]
        .spacing(24);

        // Dialogs are kept in the tree while closed so they can animate out,
        // and they are stacked on top of the content so they cover the whole
        // window. The speed is applied through a ready-made spring preset.
        let dialog_element = dialog::Dialog::new(
            dialog::Style::new(theme),
            text("This is a simple dialog with standard actions.").color(theme.on_surface()),
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
        .open(self.dialog_open)
        .speed(self.speed);

        let custom_dialog_element = dialog::Dialog::new(
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
        .open(self.dialog_custom_open)
        .speed(self.speed);

        stack![content, dialog_element, custom_dialog_element].into()
    }
}

fn speed_button<'a>(
    theme: &Theme,
    label: &'a str,
    speed: Speed,
    selected: Speed,
) -> Element<'a, Message> {
    let style = if speed == selected {
        button::Style::filled(theme, Accent::Primary)
    } else {
        button::Style::outlined(theme, Accent::Primary)
    };

    iced_m3::widget::button(style, button::Content::Label(label.into()))
        .on_press(Message::SetSpeed(speed))
        .into()
}

fn section_header<'a>(title: &'a str, theme: &Theme) -> text::Text<'a> {
    text(title)
        .font(fonts::text_bold())
        .size(18.0)
        .color(theme.on_surface())
}

common::demo!(Page, Message, "Dialog Demo");
