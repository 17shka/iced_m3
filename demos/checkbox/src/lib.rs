use iced::{
    Alignment, Element,
    widget::{column, row, text},
};
use iced_m3::{
    theme::{ColorScheme, Theme},
    widget::{self, checkbox::Value},
};

#[derive(Clone)]
pub enum Message {
    Toggle1,
    Toggle2,
    Toggle3,
    Toggle4,
    Toggle5,
    Toggle6,
    CycleState,
}

pub struct Demo {
    value1: Value,
    value2: Value,
    value3: Value,
    value4: Value,
    value5: Value,
    value6: Value,
    value8: Value,
}

impl Default for Demo {
    fn default() -> Self {
        Self {
            value1: Value::Unchecked,
            value2: Value::Checked,
            value3: Value::Indeterminate,
            value4: Value::Unchecked,
            value5: Value::Checked,
            value6: Value::Indeterminate,
            value8: Value::Unchecked,
        }
    }
}

fn label<'a>(text: &'a str, theme: &Theme) -> text::Text<'a> {
    text::Text::new(text)
        .font(fonts::text_regular())
        .size(16.0)
        .color(theme.on_surface())
}

fn section_header<'a>(text: &'a str, theme: &Theme) -> text::Text<'a> {
    text::Text::new(text)
        .font(fonts::text_bold())
        .size(18.0)
        .color(theme.on_surface())
}

impl Demo {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::Toggle1 => self.value1 = self.value1.toggle(),
            Message::Toggle2 => self.value2 = self.value2.toggle(),
            Message::Toggle3 => self.value3 = self.value3.next(),
            Message::Toggle4 => self.value4 = self.value4.toggle(),
            Message::Toggle5 => self.value5 = self.value5.toggle(),
            Message::Toggle6 => self.value6 = self.value6.next(),
            Message::CycleState => self.value8 = self.value8.next(),
        }
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        iced::Subscription::none()
    }

    pub fn view(&self, theme: &Theme) -> Element<'_, Message> {
        let interactive_section = column![
            section_header("Interactive", theme),
            row![
                widget::checkbox(theme, self.value1).on_toggle(Message::Toggle1),
                label("Unchecked", theme),
            ]
            .spacing(12)
            .align_y(Alignment::Center),
            row![
                widget::checkbox(theme, self.value2).on_toggle(Message::Toggle2),
                label("Checked", theme),
            ]
            .spacing(12)
            .align_y(Alignment::Center),
            row![
                widget::checkbox(theme, self.value3).on_toggle(Message::Toggle3),
                label("Indeterminate", theme),
            ]
            .spacing(12)
            .align_y(Alignment::Center),
            row![
                widget::checkbox(theme, self.value8).on_toggle(Message::CycleState),
                label(
                    "State cycler (Unchecked -> Checked -> Indeterminate)",
                    theme
                ),
            ]
            .spacing(12)
            .align_y(Alignment::Center),
        ]
        .spacing(12);

        let disabled_section = column![
            section_header("Disabled", theme),
            row![
                widget::checkbox(theme, Value::Unchecked),
                label("Disabled unchecked", theme),
            ]
            .spacing(12)
            .align_y(Alignment::Center),
            row![
                widget::checkbox(theme, Value::Checked),
                label("Disabled checked", theme),
            ]
            .spacing(12)
            .align_y(Alignment::Center),
            row![
                widget::checkbox(theme, Value::Indeterminate),
                label("Disabled indeterminate", theme),
            ]
            .spacing(12)
            .align_y(Alignment::Center),
        ]
        .spacing(12);

        let error_section = column![
            section_header("Error", theme),
            row![
                widget::checkbox(theme, self.value4)
                    .error(true, theme)
                    .on_toggle(Message::Toggle4),
                label("Error unchecked", theme),
            ]
            .spacing(12)
            .align_y(Alignment::Center),
            row![
                widget::checkbox(theme, self.value5)
                    .error(true, theme)
                    .on_toggle(Message::Toggle5),
                label("Error checked", theme),
            ]
            .spacing(12)
            .align_y(Alignment::Center),
            row![
                widget::checkbox(theme, self.value6)
                    .error(true, theme)
                    .on_toggle(Message::Toggle6),
                label("Error indeterminate", theme),
            ]
            .spacing(12)
            .align_y(Alignment::Center),
        ]
        .spacing(12);

        column![interactive_section, disabled_section, error_section,]
            .spacing(24)
            .into()
    }
}

pub type Page = Demo;
