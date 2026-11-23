pub use iced;
pub use iced_m3;

pub mod catalog;

use iced::{
    Element, Font, Length, Subscription, Task,
    widget::{column, container, row, text::Text},
};
use iced_m3::{
    theme::{Accent, ColorScheme, Mode, Theme},
    widget::{button, scrollable},
};

pub trait Demo: Default + 'static {
    type Message: Clone + Send + 'static;

    const TITLE: &'static str;

    const PADDING: f32 = 24.0;
    const TITLE_SIZE: f32 = 28.0;
    const CHROME: bool = true;
    const SCROLLABLE: bool = true;

    fn view<'a>(&'a self, theme: &'a Theme) -> Element<'a, Self::Message>;
    fn update(&mut self, message: Self::Message);
    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::none()
    }
}

struct App<D: Demo> {
    theme: Theme,
    page: D,
}

#[derive(Clone)]
enum Message<M> {
    Page(M),
    SetTheme(Mode),
}

impl<D: Demo> App<D> {
    fn boot() -> (Self, Task<Message<D::Message>>) {
        (
            Self {
                theme: Theme::default(Mode::Dark),
                page: D::default(),
            },
            load_fonts(),
        )
    }

    fn view(&self) -> Element<'_, Message<D::Message>> {
        let background = self.theme.background();
        let page = self.page.view(&self.theme).map(Message::Page);

        let content: Element<'_, Message<D::Message>> = if D::CHROME {
            column![
                Text::new(D::TITLE)
                    .font(fonts::text_bold())
                    .size(D::TITLE_SIZE)
                    .color(self.theme.on_surface()),
                theme_switcher(&self.theme, Message::SetTheme),
                page,
            ]
            .spacing(24.0)
            .into()
        } else {
            page
        };

        let content: Element<'_, Message<D::Message>> = if D::SCROLLABLE {
            scrollable(&self.theme, content)
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        } else {
            content
        };

        container(content)
            .padding(D::PADDING)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |_| iced::widget::container::Style::default().background(background))
            .into()
    }

    fn update(&mut self, message: Message<D::Message>) -> Task<Message<D::Message>> {
        match message {
            Message::Page(message) => self.page.update(message),
            Message::SetTheme(mode) => self.theme.mode = mode,
        }
        Task::none()
    }

    fn subscription(&self) -> Subscription<Message<D::Message>> {
        self.page.subscription().map(Message::Page)
    }
}

pub fn run<D: Demo>() -> iced::Result {
    iced::application(App::<D>::boot, App::<D>::update, App::<D>::view)
        .title(D::TITLE)
        .default_font(Font::with_name(fonts::NAME))
        .subscription(App::<D>::subscription)
        .run()
}

pub fn run_custom<State, Message, Update, View>(
    title: &'static str,
    boot: impl Fn() -> State + 'static,
    update: Update,
    view: View,
) -> iced::Result
where
    State: 'static,
    Message: Clone + Send + 'static,
    Update: Fn(&mut State, Message) -> Task<Message> + 'static,
    View: Fn(&State) -> Element<'_, Message> + 'static,
{
    iced::application(move || (boot(), load_fonts()), update, view)
        .title(title)
        .default_font(Font::with_name(fonts::NAME))
        .run()
}

fn load_fonts<Message: Send + 'static>() -> Task<Message> {
    Task::batch(
        fonts::get_fonts()
            .into_iter()
            .map(|bytes| iced::font::load(bytes).discard()),
    )
}

#[must_use]
pub fn theme_switcher<'a, Message>(
    theme: &impl ColorScheme,
    on_change: impl Fn(Mode) -> Message,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    row![
        button(
            button::Style::filled(theme, Accent::Primary),
            button::Content::Label("Light".into()),
        )
        .on_press(on_change(Mode::Light)),
        button(
            button::Style::filled(theme, Accent::Secondary),
            button::Content::Label("Dark".into()),
        )
        .on_press(on_change(Mode::Dark)),
    ]
    .spacing(8)
    .into()
}

/// A bold section heading used by the demo pages.
#[must_use]
pub fn section_header<'a>(content: &'a str, theme: &impl ColorScheme) -> Text<'a> {
    Text::new(content)
        .font(fonts::text_bold())
        .size(18.0)
        .color(theme.on_surface())
}

/// A regular body label used by the demo pages.
#[must_use]
pub fn label<'a>(content: &'a str, theme: &impl ColorScheme) -> Text<'a> {
    Text::new(content)
        .font(fonts::text_regular())
        .size(16.0)
        .color(theme.on_surface())
}

#[macro_export]
macro_rules! demo {
    (@impl $page:ty, $message:ty, $title:expr, { $($consts:tt)* }) => {
        impl $crate::Demo for $page {
            type Message = $message;

            const TITLE: &'static str = $title;
            $($consts)*

            fn view<'a>(
                &'a self,
                theme: &'a $crate::iced_m3::theme::Theme,
            ) -> $crate::iced::Element<'a, $message> {
                <$page>::view(self, theme)
            }

            fn update(&mut self, message: $message) {
                <$page>::update(self, message);
            }

            fn subscription(&self) -> $crate::iced::Subscription<$message> {
                <$page>::subscription(self)
            }
        }
    };
    ($page:ty, $message:ty, $title:expr) => {
        $crate::demo!(@impl $page, $message, $title, {
            const PADDING: f32 = 24.0;
            const TITLE_SIZE: f32 = 28.0;
            const CHROME: bool = true;
        });
    };
    ($page:ty, $message:ty, $title:expr, bare) => {
        $crate::demo!(@impl $page, $message, $title, {
            const PADDING: f32 = 10.0;
            const TITLE_SIZE: f32 = 28.0;
            const CHROME: bool = false;
        });
    };
    ($page:ty, $message:ty, $title:expr, fill) => {
        $crate::demo!(@impl $page, $message, $title, {
            const PADDING: f32 = 24.0;
            const TITLE_SIZE: f32 = 28.0;
            const CHROME: bool = true;
            const SCROLLABLE: bool = false;
        });
    };
}
