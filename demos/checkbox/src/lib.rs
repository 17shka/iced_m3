use iced::{
    Alignment, Element,
    widget::{column, row, text},
};
use iced_m3::{
    animation::motion,
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
    Toggle7,
    CycleState8,
}

pub struct Demo {
    value1: Value,
    value2: Value,
    value3: Value,
    value4: Value,
    value5: Value,
    value6: Value,
    value7: Value,
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
            value6: Value::Unchecked,
            value7: Value::Checked,
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
            Message::Toggle1 => {
                self.value1 = match self.value1 {
                    Value::Unchecked => Value::Checked,
                    Value::Checked => Value::Unchecked,
                    Value::Indeterminate => Value::Unchecked,
                }
            }
            Message::Toggle2 => {
                self.value2 = match self.value2 {
                    Value::Unchecked => Value::Checked,
                    Value::Checked => Value::Unchecked,
                    Value::Indeterminate => Value::Unchecked,
                }
            }
            Message::Toggle3 => {
                self.value3 = match self.value3 {
                    Value::Unchecked => Value::Checked,
                    Value::Checked => Value::Unchecked,
                    Value::Indeterminate => Value::Unchecked,
                }
            }
            Message::Toggle4 => {
                self.value4 = match self.value4 {
                    Value::Unchecked => Value::Checked,
                    Value::Checked => Value::Unchecked,
                    Value::Indeterminate => Value::Unchecked,
                }
            }
            Message::Toggle5 => {
                self.value5 = match self.value5 {
                    Value::Unchecked => Value::Checked,
                    Value::Checked => Value::Unchecked,
                    Value::Indeterminate => Value::Unchecked,
                }
            }
            Message::Toggle6 => {
                self.value6 = match self.value6 {
                    Value::Unchecked => Value::Checked,
                    Value::Checked => Value::Unchecked,
                    Value::Indeterminate => Value::Unchecked,
                }
            }
            Message::Toggle7 => {
                self.value7 = match self.value7 {
                    Value::Unchecked => Value::Checked,
                    Value::Checked => Value::Unchecked,
                    Value::Indeterminate => Value::Unchecked,
                }
            }
            Message::CycleState8 => {
                self.value8 = match self.value8 {
                    Value::Unchecked => Value::Checked,
                    Value::Checked => Value::Indeterminate,
                    Value::Indeterminate => Value::Unchecked,
                }
            }
        }
    }

    pub fn view(&self, theme: &Theme) -> Element<'_, Message> {
        let interactive_section = column![
            section_header("Interactive", theme),
            row![
                widget::checkbox(theme, self.value1).on_toggle(Message::Toggle1),
                label("Unchecked (click to check)", theme),
            ]
            .spacing(12)
            .align_y(Alignment::Center),
            row![
                widget::checkbox(theme, self.value2).on_toggle(Message::Toggle2),
                label("Checked (click to uncheck)", theme),
            ]
            .spacing(12)
            .align_y(Alignment::Center),
            row![
                widget::checkbox(theme, self.value3).on_toggle(Message::Toggle3),
                label("Indeterminate (click to uncheck)", theme),
            ]
            .spacing(12)
            .align_y(Alignment::Center),
            row![
                widget::checkbox(theme, self.value8).on_toggle(Message::CycleState8),
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
                widget::checkbox(theme, Value::Unchecked)
                    .error(true, theme)
                    .on_toggle(Message::Toggle4),
                label("Error unchecked", theme),
            ]
            .spacing(12)
            .align_y(Alignment::Center),
            row![
                widget::checkbox(theme, Value::Checked)
                    .error(true, theme)
                    .on_toggle(Message::Toggle5),
                label("Error checked", theme),
            ]
            .spacing(12)
            .align_y(Alignment::Center),
            row![
                widget::checkbox(theme, Value::Indeterminate).error(true, theme),
                label("Error indeterminate", theme),
            ]
            .spacing(12)
            .align_y(Alignment::Center),
        ]
        .spacing(12);

        let animation_section = column![
            section_header("Animation", theme),
            row![
                widget::checkbox(theme, self.value6)
                    .motion_scheme(motion::Scheme::Expressive)
                    .on_toggle(Message::Toggle6),
                label("Expressive animation", theme),
            ]
            .spacing(12)
            .align_y(Alignment::Center),
            row![
                widget::checkbox(theme, self.value7)
                    .motion_scheme(motion::Scheme::Standard)
                    .on_toggle(Message::Toggle7),
                label("Standard animation", theme),
            ]
            .spacing(12)
            .align_y(Alignment::Center),
        ]
        .spacing(12);

        column![
            interactive_section,
            disabled_section,
            error_section,
            animation_section,
        ]
        .spacing(24)
        .into()
    }
}
