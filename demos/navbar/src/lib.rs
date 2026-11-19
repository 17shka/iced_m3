use iced::{
    Element,
    widget::{column, text},
};
use iced_m3::theme::{ColorScheme, Theme};
use iced_m3::widget::{Badge, hybrid_icon::Icon, navbar};

#[derive(Clone)]
pub enum Message {
    TabChanged(usize),
}

pub struct Page {
    active_index: usize,
}

impl Default for Page {
    fn default() -> Self {
        Self { active_index: 0 }
    }
}

impl Page {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::TabChanged(index) => self.active_index = index,
        }
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        iced::Subscription::none()
    }

    pub fn view(&self, theme: &Theme) -> Element<'_, Message> {
        let navbar_items = vec![
            navbar::Item {
                icon_active: Icon::Text {
                    text: "star".into(),
                    font: Some(fonts::icons_filled()),
                },
                icon_inactive: Icon::Text {
                    text: "star".into(),
                    font: Some(fonts::icons_outlined()),
                },
                badge: None,
                label: "Star".into(),
                on_press: Message::TabChanged(0),
            },
            navbar::Item {
                icon_active: Icon::Text {
                    text: "favorite".into(),
                    font: Some(fonts::icons_filled()),
                },
                icon_inactive: Icon::Text {
                    text: "favorite".into(),
                    font: Some(fonts::icons_outlined()),
                },
                badge: Some(Badge { label: None }),
                label: "Fav".into(),
                on_press: Message::TabChanged(1),
            },
            navbar::Item {
                icon_active: Icon::Text {
                    text: "search".into(),
                    font: Some(fonts::icons_filled()),
                },
                icon_inactive: Icon::Text {
                    text: "search".into(),
                    font: Some(fonts::icons_outlined()),
                },
                badge: Some(Badge {
                    label: Some("3".into()),
                }),
                label: "Search".into(),
                on_press: Message::TabChanged(2),
            },
        ];

        column![
            section_header("Compact", theme),
            navbar(navbar::Style::new(theme), navbar_items.clone())
                .active_index(self.active_index)
                .mode(navbar::Mode::Compact),
            section_header("Large", theme),
            navbar(navbar::Style::new(theme), navbar_items.clone())
                .active_index(self.active_index)
                .mode(navbar::Mode::Large),
            section_header("Auto", theme),
            navbar(navbar::Style::new(theme), navbar_items.clone())
                .active_index(self.active_index)
                .mode(navbar::Mode::Auto),
            section_header("NavBar expressive motion", theme),
            navbar(navbar::Style::new(theme), navbar_items.clone())
                .active_index(self.active_index)
                .mode(navbar::Mode::Compact)
                .motion_scheme(iced_m3::animation::motion::Scheme::Expressive),
            section_header("NavBar custom springs", theme),
            navbar(navbar::Style::new(theme), navbar_items.clone())
                .active_index(self.active_index)
                .mode(navbar::Mode::Compact)
                .motion_scheme(iced_m3::animation::motion::Scheme::Expressive)
                .effects_spring(iced_m3::animation::motion::Spring {
                    damping: 0.5,
                    stiffness: 200.0,
                })
                .spatial_spring(iced_m3::animation::motion::Spring {
                    damping: 0.7,
                    stiffness: 150.0,
                }),
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
