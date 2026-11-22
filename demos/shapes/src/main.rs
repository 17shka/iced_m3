use iced::{Element, Font, Length, Task, widget::container};
use iced_m3::theme::{ColorScheme, Mode, Theme};

const APP_NAME: &str = "Shapes Demo";

struct State {
    theme: Theme,
    page: shapes_demo::Page,
}

impl Default for State {
    fn default() -> Self {
        Self {
            theme: Theme::default(Mode::Dark),
            page: shapes_demo::Page::default(),
        }
    }
}

impl State {
    fn update(&mut self, message: shapes_demo::Message) -> Task<shapes_demo::Message> {
        self.page.update(message);
        Task::none()
    }

    fn view(&self) -> Element<'_, shapes_demo::Message> {
        container(self.page.view(&self.theme))
            .padding(10.0)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_| {
                iced::widget::container::Style::default().background(self.theme.background())
            })
            .into()
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
                        .map(|bytes| iced::font::load(bytes).discard()),
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
