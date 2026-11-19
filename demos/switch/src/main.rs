use iced::{
    Element, Font, Length, Task,
    widget::{column, container, row, text},
};
use iced_m3::{
    theme::{ColorScheme, Mode, Theme},
    widget,
    widget::{button, button::Content},
};

const APP_NAME: &str = "Switch Demo";

#[derive(Clone)]
enum Message {
    Demo(switch::Message),
    SetTheme(Mode),
}

struct State {
    theme: Theme,
    demo: switch::Demo,
}

impl Default for State {
    fn default() -> Self {
        Self {
            theme: Theme::default(Mode::Dark),
            demo: switch::Demo::default(),
        }
    }
}

impl State {
    fn view(&self) -> Element<'_, Message> {
        let theme_buttons = row![
            widget::button(
                button::Style::filled(&self.theme, iced_m3::theme::Accent::Primary),
                Content::Label("Light".into()),
            )
            .on_press(Message::SetTheme(Mode::Light)),
            widget::button(
                button::Style::filled(&self.theme, iced_m3::theme::Accent::Secondary),
                Content::Label("Dark".into()),
            )
            .on_press(Message::SetTheme(Mode::Dark)),
        ]
        .spacing(8);

        container(
            column![
                text::Text::new(APP_NAME)
                    .font(fonts::text_bold())
                    .size(28.0)
                    .color(self.theme.on_surface()),
                theme_buttons,
                self.demo.view(&self.theme).map(Message::Demo),
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
            Message::Demo(msg) => self.demo.update(msg),
            Message::SetTheme(mode) => self.theme.mode = mode,
        }
        Task::none()
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
    .run()
}
