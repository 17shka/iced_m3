use iced::{
    Element,
    widget::{column, text},
};
use iced_m3::theme::{ColorScheme, Theme};
use iced_m3::widget::{Action, hybrid_icon::Icon, menu};

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
        let groups = vec![menu::Group::new(vec![
            menu::Item::Button(menu::button::Button {
                icon: Some(Icon::Text {
                    text: "content_copy".into(),
                    font: Some(fonts::icons_filled()),
                }),
                label: "Copy".into(),
                supporting_text: Some("Ctrl+C".into()),
                action: menu::button::MenuAction::Message {
                    is_selected: false,
                    on_press: Some(Action::Direct(Message::Noop)),
                },
            }),
            menu::Item::Button(menu::button::Button {
                icon: Some(Icon::Text {
                    text: "content_cut".into(),
                    font: Some(fonts::icons_filled()),
                }),
                label: "Cut".into(),
                supporting_text: Some("Ctrl+X".into()),
                action: menu::button::MenuAction::Message {
                    is_selected: false,
                    on_press: Some(Action::Direct(Message::Noop)),
                },
            }),
            menu::Item::Separator,
            menu::Item::Button(menu::button::Button {
                icon: Some(Icon::Text {
                    text: "content_paste".into(),
                    font: Some(fonts::icons_filled()),
                }),
                label: "Paste".into(),
                supporting_text: Some("Ctrl+V".into()),
                action: menu::button::MenuAction::Message {
                    is_selected: false,
                    on_press: Some(Action::Direct(Message::Noop)),
                },
            }),
        ])];

        let groups_with_label = vec![menu::Group {
            label: Some("Actions".into()),
            items: vec![
                menu::Item::Button(menu::button::Button {
                    icon: None,
                    label: "Undo".into(),
                    supporting_text: None,
                    action: menu::button::MenuAction::Message {
                        is_selected: false,
                        on_press: Some(Action::Direct(Message::Noop)),
                    },
                }),
                menu::Item::Button(menu::button::Button {
                    icon: None,
                    label: "Redo".into(),
                    supporting_text: None,
                    action: menu::button::MenuAction::Message {
                        is_selected: false,
                        on_press: None,
                    },
                }),
            ],
        }];

        column![
            section_header("Menu", theme),
            iced_m3::widget::menu(menu::Style::new(theme, false), groups),
            section_header("Menu with label", theme),
            iced_m3::widget::menu(menu::Style::new(theme, false), groups_with_label,),
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
