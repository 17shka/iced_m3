use iced::{
    Alignment, Element, Length,
    widget::{Space, center, column, row, text},
};
use iced_m3::theme::{ColorScheme, Theme};

#[derive(Clone)]
pub enum Message {
    Noop,
}

pub struct Page;

impl Default for Page {
    fn default() -> Self {
        Self
    }
}

impl Page {
    pub fn update(&mut self, _message: Message) {}

    pub fn subscription(&self) -> iced::Subscription<Message> {
        iced::Subscription::none()
    }

    pub fn view(&self, theme: &Theme) -> Element<'_, Message> {
        use iced_m3::widget::badge;

        let style = badge::Style::new(theme);
        let mail_icon = text("mail")
            .font(fonts::icons_filled())
            .size(24.0)
            .color(theme.on_surface());
        let notif_icon = text("notifications")
            .font(fonts::icons_filled())
            .size(24.0)
            .color(theme.on_surface());

        column![
            section_header("Badges", theme),
            row![
                badge(
                    style,
                    Space::new().width(Length::Shrink).height(Length::Shrink)
                ),
                badge(
                    style,
                    Space::new().width(Length::Shrink).height(Length::Shrink)
                )
                .label("1"),
                badge(
                    style,
                    Space::new().width(Length::Shrink).height(Length::Shrink)
                )
                .label("99"),
                badge(
                    style,
                    Space::new().width(Length::Shrink).height(Length::Shrink)
                )
                .label("999+"),
            ]
            .spacing(16)
            .align_y(Alignment::Center),
            section_header("Badges on icons", theme),
            row![
                center(
                    row![
                        mail_icon,
                        badge(
                            style,
                            Space::new().width(Length::Shrink).height(Length::Shrink)
                        )
                        .label("3")
                        .bounds_mode(badge::BoundsMode::Symmetrical),
                    ]
                    .align_y(Alignment::Center)
                    .spacing(0),
                )
                .width(48)
                .height(48),
                center(
                    row![
                        notif_icon,
                        badge(
                            style,
                            Space::new().width(Length::Shrink).height(Length::Shrink)
                        )
                        .label("99+")
                        .bounds_mode(badge::BoundsMode::Symmetrical),
                    ]
                    .align_y(Alignment::Center)
                    .spacing(0),
                )
                .width(48)
                .height(48),
            ]
            .spacing(16),
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
