use iced::{
    Alignment, Element,
    widget::{column, row, text},
};
use iced_m3::{
    animation::motion,
    theme::{ColorScheme, Theme},
    widget::{self, button, switch::IconMode},
};

#[derive(Clone)]
pub enum Message {
    Toggle1,
    Toggle2,
    SetIconMode(IconMode),
}

pub struct Demo {
    toggled1: bool,
    toggled2: bool,
    icon_mode: IconMode,
}

impl Default for Demo {
    fn default() -> Self {
        Self {
            toggled1: false,
            toggled2: false,
            icon_mode: IconMode::default(),
        }
    }
}

impl Demo {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::Toggle1 => self.toggled1 = !self.toggled1,
            Message::Toggle2 => self.toggled2 = !self.toggled2,
            Message::SetIconMode(icon_mode) => self.icon_mode = icon_mode,
        }
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        iced::Subscription::none()
    }

    pub fn view(&self, theme: &Theme) -> Element<'_, Message> {
        column![
            row![
                text("With expressive animation").color(theme.on_surface()),
                widget::switch(theme, self.toggled1)
                    .motion_scheme(motion::Scheme::Expressive)
                    .icon_mode(self.icon_mode)
                    .on_toggle(Message::Toggle1)
            ]
            .spacing(8)
            .align_y(Alignment::Center),
            row![
                text("With standard animation").color(theme.on_surface()),
                widget::switch(theme, self.toggled2)
                    .motion_scheme(motion::Scheme::Standard)
                    .icon_mode(self.icon_mode)
                    .on_toggle(Message::Toggle2)
            ]
            .spacing(8)
            .align_y(Alignment::Center),
            row![
                text("Disabled selected").color(theme.on_surface()),
                widget::switch(theme, true).icon_mode(self.icon_mode),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
            row![
                text("Disabled unselected").color(theme.on_surface()),
                widget::switch(theme, false).icon_mode(self.icon_mode),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
            column![
                text("Icon mode").color(theme.on_surface()),
                row![
                    widget::button(
                        button::Style::filled(theme, iced_m3::theme::Accent::default()),
                        button::Content::Label("Never".into()),
                    )
                    .on_press(Message::SetIconMode(IconMode::Never)),
                    widget::button(
                        button::Style::filled(theme, iced_m3::theme::Accent::default()),
                        button::Content::Label("When Selected".into()),
                    )
                    .on_press(Message::SetIconMode(IconMode::WhenSelected)),
                    widget::button(
                        button::Style::filled(theme, iced_m3::theme::Accent::default()),
                        button::Content::Label("Always".into()),
                    )
                    .on_press(Message::SetIconMode(IconMode::Always))
                ]
                .spacing(8)
            ]
            .spacing(8)
        ]
        .spacing(20.0)
        .into()
    }
}

pub type Page = Demo;
