use iced::{
    Element, Font, Length, Task,
    widget::{column, container, text},
};
use iced_m3::theme::{ColorScheme, Mode, Theme};

const APP_NAME: &str = "Badges Demo";

#[derive(Clone)]
enum Message {
    Page(badges::Message),
    SetTheme(Mode),
}

struct State {
    theme: Theme,
    page: badges::Page,
}

impl Default for State {
    fn default() -> Self {
        Self {
            theme: Theme::default(Mode::Dark),
            page: badges::Page::default(),
        }
    }
}

impl State {
    fn view(&self) -> Element<'_, Message> {
        let theme_buttons = common::theme_switcher(&self.theme, Message::SetTheme);

        container(
            column![
                text::Text::new(APP_NAME)
                    .font(fonts::text_bold())
                    .size(28.0)
                    .color(self.theme.on_surface()),
                theme_buttons,
                self.page.view(&self.theme).map(Message::Page),
            ]
            .spacing(24),
        )
        .padding(24.0)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_| iced::widget::container::Style::default().background(self.theme.background()))
        .into()
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Page(msg) => self.page.update(msg),
            Message::SetTheme(mode) => self.theme.mode = mode,
        }
        Task::none()
    }

    fn subscription(&self) -> iced::Subscription<Message> {
        iced::Subscription::none()
    }
}

fn main() -> iced::Result {
    iced::application(
        || {
            (
                State::default(),
                Task::batch(
                    fonts::get_fonts()
                        .into_iter()
                        .map(|b| iced::font::load(b).discard()),
                ),
            )
        },
        State::update,
        State::view,
    )
    .title(APP_NAME)
    .default_font(Font::with_name("Noto Sans"))
    .subscription(State::subscription)
    .run()
}
