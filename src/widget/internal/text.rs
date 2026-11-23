use iced::{
    Pixels, Size,
    advanced::text::{self, Paragraph},
    alignment::Vertical,
};

pub fn paragraph<Renderer>(
    renderer: &Renderer,
    content: &str,
    bounds: Size,
    size: Pixels,
    line_height: text::LineHeight,
    font: Option<Renderer::Font>,
    align_y: Vertical,
) -> Renderer::Paragraph
where
    Renderer: text::Renderer,
{
    <Renderer as text::Renderer>::Paragraph::with_text(text::Text {
        content,
        bounds,
        size,
        line_height,
        font: font.unwrap_or_else(|| renderer.default_font()),
        shaping: text::Shaping::Advanced,
        wrapping: text::Wrapping::None,
        align_x: text::Alignment::Left,
        align_y,
    })
}
