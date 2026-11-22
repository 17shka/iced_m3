use iced::{
    Alignment, Element,
    alignment::Horizontal,
    widget::{column, row, text},
};
use iced_m3::theme::{Accent, ColorScheme, Theme};
use iced_m3::widget::{button, text_input::TextInput};

#[derive(Clone)]
pub enum Message {
    TextInputChanged(String),
    SecureInputChanged(String),
    SecureVisibilityToggled,
    TextInputErrorToggle,
    TextInputSubmitted,
    TextInputWithIconChanged(String),
    TextInputWithLabelChanged(String),
    TextInputAlignedChanged(String),
}

pub struct Page {
    text_input_value: String,
    secure_text_input_value: String,
    secure_visibility: bool,
    text_input_error: bool,
    text_input_submitted: String,
    text_input_with_icon: String,
    text_input_with_label: String,
    text_input_aligned: String,
}

impl Default for Page {
    fn default() -> Self {
        Self {
            text_input_value: String::new(),
            secure_text_input_value: String::new(),
            secure_visibility: false,
            text_input_error: false,
            text_input_submitted: String::new(),
            text_input_with_icon: String::new(),
            text_input_with_label: String::new(),
            text_input_aligned: String::new(),
        }
    }
}

impl Page {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::TextInputChanged(value) => self.text_input_value = value,
            Message::SecureInputChanged(value) => self.secure_text_input_value = value,
            Message::SecureVisibilityToggled => self.secure_visibility = !self.secure_visibility,
            Message::TextInputErrorToggle => self.text_input_error = !self.text_input_error,
            Message::TextInputSubmitted => {
                self.text_input_submitted = self.text_input_value.clone();
            }
            Message::TextInputWithIconChanged(v) => self.text_input_with_icon = v,
            Message::TextInputWithLabelChanged(v) => self.text_input_with_label = v,
            Message::TextInputAlignedChanged(v) => self.text_input_aligned = v,
        }
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        iced::Subscription::none()
    }

    pub fn view<'a>(&self, theme: &'a Theme) -> Element<'a, Message> {
        column![
            section_header("Text Input", theme),
            TextInput::new("Type here...", &self.text_input_value, theme)
                .on_input(Message::TextInputChanged)
                .width(300.0),
            section_header("Secure Input", theme),
            TextInput::new("Password...", &self.secure_text_input_value, theme)
                .on_input(Message::SecureInputChanged)
                .secure(!self.secure_visibility)
                .on_toggle_secure(Message::SecureVisibilityToggled)
                .width(300.0),
            section_header("Error Input", theme),
            row![
                TextInput::new("Type to trigger...", &self.text_input_value, theme)
                    .on_input(Message::TextInputChanged)
                    .error(self.text_input_error)
                    .width(300.0),
                iced_m3::widget::button(
                    button::Style::filled(theme, Accent::Primary),
                    button::Content::Label("Toggle Error".into()),
                )
                .on_press(Message::TextInputErrorToggle),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
            section_header("Text input with icon", theme),
            TextInput::new("Search...", &self.text_input_with_icon, theme)
                .on_input(Message::TextInputWithIconChanged)
                .icon(iced_m3::widget::text_input::Icon {
                    font: fonts::icons_filled(),
                    code_point: '\u{e8b6}',
                    size: None,
                    spacing: 8.0,
                    side: iced_m3::widget::text_input::Side::Left,
                })
                .width(300.0),
            section_header("Text input with label", theme),
            TextInput::new("Labelled input...", &self.text_input_with_label, theme)
                .on_input(Message::TextInputWithLabelChanged)
                .with_label_text("Label", theme.primary_container())
                .width(300.0),
            section_header("Text input on submit", theme),
            TextInput::new("Press Enter...", &self.text_input_value, theme)
                .on_input(Message::TextInputChanged)
                .on_submit(Message::TextInputSubmitted)
                .width(300.0),
            text(format!("Submitted: {}", self.text_input_submitted)).color(theme.on_surface()),
            section_header("Text input aligned", theme),
            TextInput::new("Centered...", &self.text_input_aligned, theme)
                .on_input(Message::TextInputAlignedChanged)
                .align_x(Horizontal::Center)
                .size(18.0)
                .line_height(1.5)
                .padding(16.0)
                .width(300.0),
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
