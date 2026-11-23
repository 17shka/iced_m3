use iced::{
    Alignment, Element,
    widget::{column, row, text},
};
use iced_m3::theme::{Accent, ColorScheme, Theme};
use iced_m3::widget::{button, hybrid_icon::Icon};

#[derive(Clone)]
pub enum Message {
    ToggleSelected,
    ToggleCornerStyle,
    Noop,
}

pub struct Page {
    button_selected: bool,
    corner_style: button::CornerStyle,
}

impl Default for Page {
    fn default() -> Self {
        Self {
            button_selected: false,
            corner_style: button::CornerStyle::Rounded,
        }
    }
}

impl Page {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::ToggleSelected => self.button_selected = !self.button_selected,
            Message::ToggleCornerStyle => {
                self.corner_style = match self.corner_style {
                    button::CornerStyle::Rounded => button::CornerStyle::Square,
                    button::CornerStyle::Square => button::CornerStyle::Rounded,
                };
            }
            Message::Noop => {}
        }
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        iced::Subscription::none()
    }

    pub fn view(&self, theme: &Theme) -> Element<'_, Message> {
        let add_icon = Icon::Text {
            text: "add".into(),
            font: Some(fonts::icons_filled()),
        };
        let btn = |style: button::Style, label: &'static str| {
            iced_m3::widget::button(style, button::Content::Label(label.into()))
                .on_press(Message::Noop)
        };
        let icon_btn = |style: button::Style, icon: Icon<'static>| {
            iced_m3::widget::button(style, button::Content::Icon(icon)).on_press(Message::Noop)
        };
        let full_btn = |style: button::Style, label: &'static str, icon: Icon<'static>| {
            iced_m3::widget::button(
                style,
                button::Content::Full {
                    icon,
                    label: label.into(),
                },
            )
            .on_press(Message::Noop)
        };

        column![
            section_header("Filled buttons", theme),
            row![
                btn(button::Style::filled(theme, Accent::Primary), "Primary"),
                btn(button::Style::filled(theme, Accent::Secondary), "Secondary"),
                btn(button::Style::filled(theme, Accent::Tertiary), "Tertiary"),
            ]
            .spacing(8),
            section_header("Elevated buttons", theme),
            row![
                btn(button::Style::elevated(theme, Accent::Primary), "Primary"),
                btn(
                    button::Style::elevated(theme, Accent::Secondary),
                    "Secondary"
                ),
                btn(button::Style::elevated(theme, Accent::Tertiary), "Tertiary"),
            ]
            .spacing(8),
            section_header("Outlined buttons", theme),
            row![
                btn(button::Style::outlined(theme, Accent::Primary), "Primary"),
                btn(
                    button::Style::outlined(theme, Accent::Secondary),
                    "Secondary"
                ),
                btn(button::Style::outlined(theme, Accent::Tertiary), "Tertiary"),
            ]
            .spacing(8),
            section_header("Text buttons", theme),
            row![
                btn(button::Style::text(theme, Accent::Primary), "Primary"),
                btn(button::Style::text(theme, Accent::Secondary), "Secondary"),
                btn(button::Style::text(theme, Accent::Tertiary), "Tertiary"),
            ]
            .spacing(8),
            section_header("Disabled buttons", theme),
            row![
                iced_m3::widget::button(
                    button::Style::filled(theme, Accent::Primary),
                    button::Content::Label("Filled".into())
                ),
                iced_m3::widget::button(
                    button::Style::elevated(theme, Accent::Primary),
                    button::Content::Label("Elevated".into())
                ),
                iced_m3::widget::button(
                    button::Style::outlined(theme, Accent::Primary),
                    button::Content::Label("Outlined".into())
                ),
                iced_m3::widget::button(
                    button::Style::text(theme, Accent::Primary),
                    button::Content::Label("Text".into())
                ),
            ]
            .spacing(8),
            section_header("Icon buttons", theme),
            row![
                icon_btn(
                    button::Style::filled(theme, Accent::Primary),
                    add_icon.clone()
                ),
                icon_btn(
                    button::Style::elevated(theme, Accent::Primary),
                    add_icon.clone()
                ),
                icon_btn(
                    button::Style::outlined(theme, Accent::Primary),
                    add_icon.clone()
                ),
                icon_btn(
                    button::Style::text(theme, Accent::Primary),
                    add_icon.clone()
                ),
            ]
            .spacing(8),
            section_header("Full buttons (icon + label)", theme),
            row![
                full_btn(
                    button::Style::filled(theme, Accent::Primary),
                    "Add",
                    add_icon.clone()
                ),
                full_btn(
                    button::Style::elevated(theme, Accent::Primary),
                    "Add",
                    add_icon.clone()
                ),
                full_btn(
                    button::Style::outlined(theme, Accent::Primary),
                    "Add",
                    add_icon.clone()
                ),
                full_btn(button::Style::text(theme, Accent::Primary), "Add", add_icon),
            ]
            .spacing(8),
            section_header("Button sizes", theme),
            row![
                btn(button::Style::filled(theme, Accent::Primary), "Extra Small")
                    .size(button::Size::extra_small()),
                btn(button::Style::filled(theme, Accent::Primary), "Small")
                    .size(button::Size::small()),
                btn(button::Style::filled(theme, Accent::Primary), "Medium")
                    .size(button::Size::medium()),
                btn(button::Style::filled(theme, Accent::Primary), "Large")
                    .size(button::Size::large()),
                btn(button::Style::filled(theme, Accent::Primary), "Extra Large")
                    .size(button::Size::extra_large()),
            ]
            .spacing(8),
            section_header("Selected button", theme),
            row![
                iced_m3::widget::button(
                    if self.button_selected {
                        button::Style::filled(theme, Accent::Primary)
                    } else {
                        button::Style::outlined(theme, Accent::Primary)
                    },
                    button::Content::Label("Toggle selection".into()),
                )
                .on_press(Message::ToggleSelected)
                .selected(self.button_selected),
            ]
            .spacing(8),
            section_header("Buttons with corner styles", theme),
            row![
                iced_m3::widget::button(
                    button::Style::filled(theme, Accent::Primary),
                    button::Content::Label("Rounded".into()),
                )
                .on_press(Message::Noop),
                iced_m3::widget::button(
                    button::Style::filled(theme, Accent::Primary),
                    button::Content::Label("Square".into()),
                )
                .on_press(Message::Noop)
                .size(button::Size {
                    corner_radius: button::CornerRadius::default()
                        .style(button::CornerStyle::Square),
                    ..button::Size::default()
                }),
                iced_m3::widget::button(
                    button::Style::elevated(theme, Accent::default()),
                    button::Content::Label("Toggle style".into()),
                )
                .on_press(Message::ToggleCornerStyle),
                text(format!("{:?}", self.corner_style)).color(theme.on_surface_variant()),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
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

common::demo!(Page, Message, "Buttons Demo");
