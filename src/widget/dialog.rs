use iced::{
    Alignment, Border, Color, Element, Event, Font, Length, Padding, Pixels, Point, Rectangle,
    Size, Transformation, Vector,
    advanced::{
        Clipboard, Layout, Renderer as _, Shell, Widget,
        layout::{Limits, Node},
        mouse, overlay,
        renderer::{self, Quad},
        text,
        widget::{Operation, Tree, tree},
    },
    color,
    time::Instant,
    touch,
};
use iced_widget::{column, container, row, space, text::IntoFragment};

use crate::{
    animation::motion::{self, Spring, SpringValue},
    style::{Elevation, StateLayer},
    theme::{Accent, ColorScheme},
    widget::{button, common_icons, hybrid_icon::Icon, icon, internal::placement, tooltip},
};

mod constants {
    pub const MIN_WIDTH: f32 = 280.0;
    pub const MAX_WIDTH: f32 = 560.0;
}

const DIALOG_RADIUS: f32 = 24.0;
const ICON_SIZE: f32 = 24.0;
const TITLE_SIZE: f32 = 24.0;
const PADDING: f32 = 24.0;
const ICON_TITLE_SPACING: f32 = 16.0;
const TITLE_BODY_SPACING: f32 = 16.0;
const BODY_BUTTONS_SPACING: f32 = 24.0;
const BUTTON_SPACING: f32 = 8.0;

/// Opacity of the scrim behind an open dialog.
const SCRIM_ALPHA: f32 = 0.4;
const CLOSE_BUTTON_SIZE: f32 = 48.0;
const CLOSE_ICON_SIZE: f32 = 24.0;

/// Progress below which a closed dialog is considered fully hidden.
///
/// Springs come to rest slightly off their target, so the dialog must be
/// dropped before its progress reaches exactly `0.0` to avoid an invisible
/// overlay that would keep blocking input.
const HIDDEN_PROGRESS: f32 = 0.001;

#[cfg(feature = "pub-internal-const")]
pub use constants::*;

pub struct Button<Message> {
    pub on_press: Option<Message>,
    pub label: String,
    pub style: crate::widget::button::Style,
}

pub struct Style {
    pub icon_color: Color,
    pub title_color: Color,
    pub container_color: Color,
    pub container_border: Border,
    pub elevation: Elevation,
    pub shadow_color: Color,
    /// Style of the close button shown in the header of a fullscreen dialog.
    pub close_button: button::Style,
    /// Style of the tooltip shown for the fullscreen close button.
    pub tooltip_style: tooltip::Style,
}

impl Style {
    pub fn new(theme: &(impl ColorScheme + ?Sized)) -> Self {
        let mut close_button = button::Style::text(theme, Accent::Primary);
        let close_icon = theme.on_surface_variant();
        close_button.regular.icon = close_icon;
        close_button.unselected.icon = close_icon;
        close_button.selected.icon = close_icon;
        let close_layer = StateLayer::new(close_icon);
        close_button.state_layer = close_layer;
        close_button.state_layer_unselected = close_layer;
        close_button.state_layer_selected = close_layer;

        Self {
            icon_color: theme.secondary(),
            title_color: theme.on_surface(),
            container_color: theme.surface_container_high(),
            container_border: Border::default().rounded(DIALOG_RADIUS),
            elevation: Elevation::Level3,
            shadow_color: theme.shadow(),
            close_button,
            tooltip_style: tooltip::Style::new(theme),
        }
    }
}

pub struct Dialog<'a, Message> {
    style: Style,
    icon: Option<char>,
    icon_font: Option<Font>,
    title: text::Fragment<'a>,
    title_font: Option<Font>,
    body: Element<'a, Message>,
    buttons: Vec<Button<Message>>,
    button_label_font: Option<Font>,
    width: Option<f32>,
    height: Option<f32>,
    on_press_outside: Option<Message>,
    close: Option<Message>,
    fullscreen: bool,
    open: bool,
    spring: Option<Spring>,
    scheme: motion::Scheme,
    speed: motion::Speed,
}

impl<'a, Message> Dialog<'a, Message> {
    #[must_use]
    pub fn new(
        style: Style,
        body: impl Into<Element<'a, Message>>,
        buttons: Vec<Button<Message>>,
    ) -> Self {
        Self {
            style,
            icon: None,
            icon_font: None,
            title: crate::i18n::dialog_title(),
            title_font: None,
            body: body.into(),
            buttons,
            button_label_font: None,
            width: None,
            height: None,
            on_press_outside: None,
            close: None,
            fullscreen: false,
            open: true,
            spring: None,
            scheme: motion::Scheme::default(),
            speed: motion::Speed::default(),
        }
    }

    #[must_use]
    pub fn icon(mut self, icon: char) -> Self {
        self.icon = Some(icon);
        self
    }

    #[must_use]
    pub fn icon_maybe(mut self, maybe_icon: Option<char>) -> Self {
        self.icon = maybe_icon;
        self
    }

    #[must_use]
    pub fn icon_font(mut self, font: Font) -> Self {
        self.icon_font = Some(font);
        self
    }

    #[must_use]
    pub fn icon_font_maybe(mut self, font: Option<Font>) -> Self {
        self.icon_font = font;
        self
    }

    #[must_use]
    pub fn title(mut self, title: impl IntoFragment<'a>) -> Self {
        self.title = title.into_fragment();
        self
    }

    #[must_use]
    pub fn title_font(mut self, font: Font) -> Self {
        self.title_font = Some(font);
        self
    }

    #[must_use]
    pub fn title_font_maybe(mut self, font: Option<Font>) -> Self {
        self.title_font = font;
        self
    }

    #[must_use]
    pub fn button_label_font(mut self, font: Font) -> Self {
        self.button_label_font = Some(font);
        self
    }

    #[must_use]
    pub fn button_label_font_maybe(mut self, font: Option<Font>) -> Self {
        self.button_label_font = font;
        self
    }

    #[must_use]
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    #[must_use]
    pub fn width_maybe(mut self, maybe_width: Option<f32>) -> Self {
        self.width = maybe_width;
        self
    }

    #[must_use]
    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }

    #[must_use]
    pub fn height_maybe(mut self, maybe_height: Option<f32>) -> Self {
        self.height = maybe_height;
        self
    }

    #[must_use]
    pub fn on_press_outside(mut self, on_press: Message) -> Self {
        self.on_press_outside = Some(on_press);
        self
    }

    #[must_use]
    pub fn on_press_outside_maybe(mut self, on_press: Option<Message>) -> Self {
        self.on_press_outside = on_press;
        self
    }

    /// Sets the message produced by the close button of a fullscreen dialog.
    ///
    /// When unset, the close button falls back to [`Dialog::on_press_outside`].
    #[must_use]
    pub fn on_close(mut self, on_close: Message) -> Self {
        self.close = Some(on_close);
        self
    }

    /// Sets the message produced by the close button, if `Some`.
    #[must_use]
    pub fn on_close_maybe(mut self, on_close: Option<Message>) -> Self {
        self.close = on_close;
        self
    }

    /// Makes the dialog cover the whole window.
    ///
    /// A fullscreen dialog has no rounded corners, keeps its title in a header
    /// row and shows a close button on the right of that row.
    #[must_use]
    pub fn fullscreen(mut self, fullscreen: bool) -> Self {
        self.fullscreen = fullscreen;
        self
    }

    #[must_use]
    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    #[must_use]
    pub fn motion_scheme(mut self, scheme: motion::Scheme) -> Self {
        self.scheme = scheme;
        self
    }

    #[must_use]
    pub fn speed(mut self, speed: motion::Speed) -> Self {
        self.speed = speed;
        self
    }

    #[must_use]
    pub fn spring(mut self, spring: Spring) -> Self {
        self.spring = Some(spring);
        self
    }

    /// Sets the spring driving the appear and disappear animations, if any.
    #[must_use]
    pub fn spring_maybe(mut self, spring: Option<Spring>) -> Self {
        self.spring = spring;
        self
    }
}

impl<'a, Message> From<Dialog<'a, Message>> for Element<'a, Message>
where
    Message: 'a + Clone,
{
    fn from(value: Dialog<'a, Message>) -> Self {
        let Dialog {
            style,
            icon: maybe_icon,
            icon_font,
            title,
            title_font,
            body,
            buttons,
            button_label_font,
            width,
            height,
            on_press_outside,
            close,
            fullscreen,
            open,
            spring,
            scheme,
            speed,
        } = value;

        let Style {
            icon_color,
            title_color,
            container_color,
            container_border,
            elevation,
            shadow_color,
            close_button: close_button_style,
            tooltip_style,
        } = style;

        let icon_element =
            maybe_icon.map(|i| icon(i, ICON_SIZE).color(icon_color).font_maybe(icon_font));

        let title_element = iced_widget::text(title)
            .size(TITLE_SIZE)
            .color(title_color)
            .font_maybe(title_font);

        let buttons_present = !buttons.is_empty();
        let buttons = row({
            let mut content = Vec::with_capacity(buttons.len() + 1);
            content.push(space().width(Length::Fill).into());
            content.extend(buttons.into_iter().map(|b| {
                crate::widget::button(b.style, button::Content::Label(b.label.into()))
                    .label_font_maybe(button_label_font)
                    .on_press_maybe(b.on_press)
                    .into()
            }));
            content
        })
        .spacing(BUTTON_SPACING);

        let surface: Element<'a, Message> = if fullscreen {
            // A fullscreen dialog covers the whole window, so it has no rounded
            // corners and keeps its title and close button in a header row.
            let close_message = close.or_else(|| on_press_outside.clone());
            let close_button = close_message.map(|on_press| {
                crate::widget::button(
                    close_button_style,
                    button::Content::Icon(Icon::Svg(common_icons::CLOSE.clone())),
                )
                .size(
                    button::Size::extra_small()
                        .width(Length::Fixed(CLOSE_BUTTON_SIZE))
                        .height(Length::Fixed(CLOSE_BUTTON_SIZE))
                        .padding(Padding::ZERO)
                        .icon_size(Pixels(CLOSE_ICON_SIZE)),
                )
                .on_press(on_press)
            });

            let mut header_items: Vec<Element<'a, Message>> = Vec::new();
            if let Some(icon_element) = icon_element {
                header_items.push(icon_element.into());
            }
            header_items.push(title_element.into());
            header_items.push(space().width(Length::Fill).into());
            if let Some(close_button) = close_button {
                header_items.push(
                    crate::widget::tooltip(tooltip_style, close_button)
                        .label(text::Fragment::Owned(
                            crate::i18n::dialog_close().into_owned(),
                        ))
                        .into(),
                );
            }

            let header = row(header_items)
                .width(Length::Fill)
                .align_y(Alignment::Center)
                .spacing(ICON_TITLE_SPACING);

            let mut content = column![header, space().height(TITLE_BODY_SPACING), body]
                .width(Length::Fill)
                .height(Length::Fill);

            if buttons_present {
                // Push the actions to the bottom of the window.
                content = content.push(space().height(Length::Fill));
            }
            content = content.push(buttons);

            container(content)
                .padding(PADDING)
                .width(Length::Fill)
                .height(Length::Fill)
                .style(move |_| {
                    iced_widget::container::Style::default()
                        .background(container_color)
                        .border(Border::default().rounded(0.0))
                })
                .into()
        } else {
            let alignment = if icon_element.is_some() {
                Alignment::Center
            } else {
                Alignment::Start
            };

            let column = column![
                // A little hack to enforce the min width
                space().width(constants::MIN_WIDTH),
                icon_element.map(|icon| column![icon, space().height(ICON_TITLE_SPACING)]),
                column![title_element, space().height(TITLE_BODY_SPACING)],
                body,
                buttons_present.then_some(space().height(BODY_BUTTONS_SPACING)),
                buttons
            ]
            .align_x(alignment);

            let dialog = container(column)
                .padding(PADDING)
                .style(move |_| {
                    iced_widget::container::Style::default()
                        .background(container_color)
                        .border(container_border)
                        .shadow(crate::style::shadow(shadow_color, elevation))
                })
                .max_width(constants::MAX_WIDTH);

            let dialog = match width {
                Some(width) => {
                    dialog.width(width.clamp(constants::MIN_WIDTH, constants::MAX_WIDTH))
                }
                None => dialog,
            };

            let dialog = match height {
                Some(height) => dialog.height(height),
                None => dialog,
            };

            dialog.into()
        };

        // An explicit spring wins; otherwise pick a ready-made one from the
        // configured speed and motion scheme.
        let spring = spring.unwrap_or_else(|| speed.spatial(scheme));

        Element::new(DialogWidget {
            surface,
            fullscreen,
            open,
            on_press_outside,
            spring,
        })
    }
}

struct State {
    progress: SpringValue<f32>,
    surface_tree: Option<Tree>,
}

struct DialogWidget<'a, Message> {
    surface: Element<'a, Message>,
    fullscreen: bool,
    open: bool,
    on_press_outside: Option<Message>,
    spring: Spring,
}

impl<'a, Message> Widget<Message, iced::Theme, iced::Renderer> for DialogWidget<'a, Message>
where
    Message: 'a + Clone,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        let now = Instant::now();
        let mut progress = SpringValue::new(0.0, self.spring, now);
        progress.reset(f32::from(self.open), now);

        tree::State::new(State {
            progress,
            surface_tree: None,
        })
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Shrink, Length::Shrink)
    }

    fn layout(&mut self, _tree: &mut Tree, _renderer: &iced::Renderer, limits: &Limits) -> Node {
        Node::new(limits.resolve(Length::Shrink, Length::Shrink, Size::ZERO))
    }

    fn draw(
        &self,
        _tree: &Tree,
        _renderer: &mut iced::Renderer,
        _theme: &iced::Theme,
        _style: &renderer::Style,
        _layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        _layout: Layout<'b>,
        _renderer: &iced::Renderer,
        _viewport: &Rectangle,
        _translation: Vector,
    ) -> Option<overlay::Element<'b, Message, iced::Theme, iced::Renderer>> {
        let state = tree.state.downcast_mut::<State>();

        if !self.open && state.progress.value() <= HIDDEN_PROGRESS {
            state.surface_tree = None;
            return None;
        }

        if let Some(surface_tree) = &mut state.surface_tree {
            surface_tree.diff(self.surface.as_widget());
        } else {
            state.surface_tree = Some(Tree::new(self.surface.as_widget()));
        }

        let spring = self.spring;
        let DialogWidget {
            surface,
            fullscreen,
            open,
            on_press_outside,
            ..
        } = self;
        let State {
            progress,
            surface_tree,
        } = state;

        Some(overlay::Element::new(Box::new(DialogOverlay {
            surface,
            tree: surface_tree.as_mut()?,
            progress,
            spring,
            fullscreen: *fullscreen,
            open: *open,
            on_press_outside: on_press_outside.as_ref(),
        })))
    }
}

struct DialogOverlay<'a, 'b, Message> {
    surface: &'b mut Element<'a, Message>,
    tree: &'b mut Tree,
    progress: &'b mut SpringValue<f32>,
    spring: Spring,
    fullscreen: bool,
    open: bool,
    on_press_outside: Option<&'b Message>,
}

impl<Message> DialogOverlay<'_, '_, Message> {
    fn is_visible(&self) -> bool {
        self.open || self.progress.value() > HIDDEN_PROGRESS
    }
}

impl<'a, 'b, Message> overlay::Overlay<Message, iced::Theme, iced::Renderer>
    for DialogOverlay<'a, 'b, Message>
where
    Message: 'a + Clone,
{
    fn layout(&mut self, renderer: &iced::Renderer, bounds: Size) -> Node {
        if self.fullscreen {
            let limits = Limits::new(Size::ZERO, bounds);
            let surface = self
                .surface
                .as_widget_mut()
                .layout(self.tree, renderer, &limits)
                .move_to(Point::new(0.0, 0.0));

            return Node::with_children(bounds, vec![surface]);
        }

        let available = placement::inset(bounds, PADDING);
        let limits = Limits::new(Size::ZERO, available);

        let node = self
            .surface
            .as_widget_mut()
            .layout(self.tree, renderer, &limits);
        let size = node.size();

        let surface = node.move_to(placement::centered_in(bounds, size));

        // The overlay spans the whole window so that the scrim and the
        // dialog's shadow are not clipped; the surface is the only child.
        Node::with_children(bounds, vec![surface])
    }

    fn draw(
        &self,
        renderer: &mut iced::Renderer,
        theme: &iced::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        let progress = self.progress.value();

        if progress <= HIDDEN_PROGRESS {
            return;
        }

        // NOTE: The viewport must be derived from the passed layout, not from
        // `layout()`, because `UserInterface::draw` reuses the layout computed
        // during `update` without calling `layout()` again.
        let viewport = layout.bounds();
        let progress = progress.clamp(0.0, 1.0);

        renderer.fill_quad(
            Quad {
                bounds: viewport,
                ..Default::default()
            },
            color!(0x000000).scale_alpha(SCRIM_ALPHA * progress),
        );

        let surface_layout = layout.child(0);
        // A fullscreen surface already covers the whole window, so growing it
        // from zero looks exactly like a regular dialog opening.
        let bounds = surface_layout.bounds();
        let center = bounds.center();
        // Animating the scale all the way to zero is what makes the dialog
        // disappear smoothly, as the renderer cannot fade arbitrary content.
        let scale = progress.max(0.01);

        let transformation = Transformation::translate(center.x, center.y)
            * Transformation::scale(scale)
            * Transformation::translate(-center.x, -center.y);

        renderer.with_transformation(transformation, |renderer| {
            self.surface.as_widget().draw(
                self.tree,
                renderer,
                theme,
                style,
                surface_layout,
                cursor,
                &viewport,
            );
        });
    }

    fn update(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) {
        let now = Instant::now();

        self.progress.set_model(self.spring);
        self.progress.set_target(f32::from(self.open), now);

        if !self.progress.is_at_rest() {
            shell.request_redraw();
        }
        self.progress.step(now);

        if !self.is_visible() {
            return;
        }

        let viewport = layout.bounds();
        let surface_layout = layout.child(0);

        self.surface.as_widget_mut().update(
            self.tree,
            event,
            surface_layout,
            cursor,
            renderer,
            clipboard,
            shell,
            &viewport,
        );

        let bounds = surface_layout.bounds();

        let pressed_outside = match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                cursor.position().map(|position| !bounds.contains(position))
            }
            Event::Touch(touch::Event::FingerPressed { position, .. }) => {
                Some(!bounds.contains(*position))
            }
            _ => None,
        };

        if self.open
            && pressed_outside == Some(true)
            && let Some(on_press_outside) = self.on_press_outside
        {
            shell.publish(on_press_outside.clone());
        }

        let is_pointer_event = matches!(
            event,
            Event::Mouse(
                mouse::Event::ButtonPressed(_)
                    | mouse::Event::ButtonReleased(_)
                    | mouse::Event::WheelScrolled { .. }
            ) | Event::Touch(
                touch::Event::FingerPressed { .. }
                    | touch::Event::FingerLifted { .. }
                    | touch::Event::FingerLost { .. }
            )
        );

        // The dialog is modal: interactions must not reach the content below
        // the scrim.
        if is_pointer_event {
            shell.capture_event();
        }
    }

    fn operate(
        &mut self,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn Operation,
    ) {
        self.surface
            .as_widget_mut()
            .operate(self.tree, layout.child(0), renderer, operation);
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        if !self.is_visible() {
            return mouse::Interaction::None;
        }

        let surface_layout = layout.child(0);

        let interaction = if cursor.is_over(surface_layout.bounds()) {
            self.surface.as_widget().mouse_interaction(
                self.tree,
                surface_layout,
                cursor,
                &layout.bounds(),
                renderer,
            )
        } else {
            mouse::Interaction::None
        };

        // A visible dialog is modal, so it must never report `None` for the
        // whole window. Otherwise `UserInterface` falls back to the content
        // below the scrim and shows its cursor (for instance a pointer over a
        // button that the dialog is drawn on top of). `Idle` is the neutral
        // arrow cursor used over the scrim and the dialog's own background.
        if interaction == mouse::Interaction::None {
            mouse::Interaction::Idle
        } else {
            interaction
        }
    }

    fn overlay<'c>(
        &'c mut self,
        layout: Layout<'c>,
        renderer: &iced::Renderer,
    ) -> Option<overlay::Element<'c, Message, iced::Theme, iced::Renderer>> {
        let viewport = layout.bounds();

        self.surface.as_widget_mut().overlay(
            self.tree,
            layout.child(0),
            renderer,
            &viewport,
            Vector::ZERO,
        )
    }
}
