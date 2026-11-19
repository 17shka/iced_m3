use iced::Element;
use iced::widget::scrollable::{AutoScroll, Catalog, Rail, Scroller, Style};
use iced::{Background, Border, Color, Shadow, widget::container};

use crate::theme::ColorScheme;

/// Creates a new Material Design 3 styled [`Scrollable`].
#[must_use]
pub fn scrollable<'a, Message, Theme, Renderer>(
    theme: &impl ColorScheme,
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> iced::widget::Scrollable<'a, Message, Theme, Renderer>
where
    Theme: 'a + Catalog,
    <Theme as Catalog>::Class<'a>: From<iced::widget::scrollable::StyleFn<'a, Theme>>,
    Renderer: 'a + iced::advanced::text::Renderer,
{
    let scrollbar_color = theme.on_surface_variant();

    iced::widget::scrollable(content).style(move |_theme: &Theme, status| {
        let color = match status {
            iced::widget::scrollable::Status::Active { .. } => scrollbar_color.scale_alpha(0.4),
            iced::widget::scrollable::Status::Hovered { .. } => scrollbar_color.scale_alpha(0.7),
            iced::widget::scrollable::Status::Dragged { .. } => scrollbar_color,
        };
        Style {
            container: container::Style::default(),
            vertical_rail: Rail {
                background: Some(Background::Color(Color::TRANSPARENT)),
                border: Border::default(),
                scroller: Scroller {
                    background: Background::Color(color),
                    border: Border::default().rounded(u32::MAX),
                },
            },
            horizontal_rail: Rail {
                background: Some(Background::Color(Color::TRANSPARENT)),
                border: Border::default(),
                scroller: Scroller {
                    background: Background::Color(color),
                    border: Border::default().rounded(u32::MAX),
                },
            },
            gap: None,
            auto_scroll: AutoScroll {
                background: Background::Color(Color::TRANSPARENT),
                border: Border::default(),
                shadow: Shadow::default(),
                icon: Color::TRANSPARENT,
            },
        }
    })
}
