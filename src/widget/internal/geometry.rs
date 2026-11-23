use iced::Rectangle;

/// Centers a square of edge `size` inside `bounds`.
///
/// Used to place the circular state layer (or the control glyph) inside the
/// larger touch target of widgets like [`Checkbox`](crate::widget::Checkbox)
/// and [`Radio`](crate::widget::Radio).
pub(crate) fn centered_square(bounds: Rectangle, size: f32) -> Rectangle {
    Rectangle {
        x: bounds.x + (bounds.width - size) / 2.0,
        y: bounds.y + (bounds.height - size) / 2.0,
        width: size,
        height: size,
    }
}
