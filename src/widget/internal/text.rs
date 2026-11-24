use iced::{
    Pixels, Size,
    advanced::text::{self, Paragraph},
    alignment::Vertical,
};

#[derive(Debug)]
pub struct CachedParagraph<Font, P> {
    cached: Option<Cached<Font, P>>,
}

#[derive(Debug)]
struct Cached<Font, P> {
    content: String,
    bounds: Size,
    size: Pixels,
    line_height: text::LineHeight,
    font: Option<Font>,
    align_y: Vertical,
    paragraph: P,
}

impl<Font, P> Default for CachedParagraph<Font, P> {
    fn default() -> Self {
        Self { cached: None }
    }
}

impl<Font: PartialEq, P> CachedParagraph<Font, P> {
    pub fn clear(&mut self) {
        self.cached = None;
    }

    pub fn as_ref(&self) -> Option<&P> {
        self.cached.as_ref().map(|cached| &cached.paragraph)
    }

    pub fn update(
        &mut self,
        content: &str,
        bounds: Size,
        size: Pixels,
        line_height: text::LineHeight,
        font: Option<Font>,
        align_y: Vertical,
        build: impl FnOnce() -> P,
    ) -> &P {
        let is_stale = self.cached.as_ref().is_none_or(|cached| {
            cached.content != content
                || cached.bounds != bounds
                || cached.size != size
                || cached.line_height != line_height
                || cached.font != font
                || cached.align_y != align_y
        });

        if is_stale {
            self.cached = Some(Cached {
                content: content.to_owned(),
                bounds,
                size,
                line_height,
                font,
                align_y,
                paragraph: build(),
            });
        }

        &self
            .cached
            .as_ref()
            .expect("the paragraph is cached above")
            .paragraph
    }
}

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
