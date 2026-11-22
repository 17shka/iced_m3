use std::time::Instant;

use iced::{
    Border, Color, Element, Length, Point, Rectangle, Size, Vector,
    advanced::{
        Overlay, Widget,
        layout::{self, Node},
        mouse, overlay,
        renderer::Quad,
    },
    border::Radius,
    widget::canvas,
};

use crate::{
    animation::{
        Interpolable,
        motion::{self, Spring, SpringValue, fast_effects, fast_spatial},
    },
    style::StateLayer,
    theme::ColorScheme,
    widget::ripple::Ripple,
};

const CHECKBOX_SIZE: f32 = 18.0;
const TOUCH_TARGET_SIZE: f32 = 48.0;
const BORDER_RADIUS: f32 = 2.0;
const OUTLINE_WIDTH: f32 = 2.0;

const STATE_LAYER_SIZE: f32 = 40.0;
const DISABLED_OPACITY: f32 = 0.38;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Value {
    #[default]
    Unchecked,
    Checked,
    Indeterminate,
}

impl Value {
    pub fn is_checked(self) -> bool {
        matches!(self, Self::Checked)
    }

    pub fn is_unchecked(self) -> bool {
        matches!(self, Self::Unchecked)
    }

    pub fn is_indeterminate(self) -> bool {
        matches!(self, Self::Indeterminate)
    }

    pub fn is_selected(self) -> bool {
        matches!(self, Self::Checked | Self::Indeterminate)
    }

    #[must_use]
    // Unchecked -> Checked
    pub fn toggle(self) -> Self {
        if self.is_checked() {
            Self::Unchecked
        } else {
            Self::Checked
        }
    }

    #[must_use]
    // Unchecked -> Checked -> Indeterminate
    pub fn next(self) -> Self {
        match self {
            Self::Unchecked => Self::Checked,
            Self::Checked => Self::Indeterminate,
            Self::Indeterminate => Self::Unchecked,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StateStyle {
    pub background_color: Color,
    pub icon_color: Color,
    pub outline_width: f32,
    pub outline_color: Color,
}

impl Interpolable for StateStyle {
    fn interpolate(self, other: Self, t: f32) -> Self {
        Self {
            background_color: self.background_color.interpolate(other.background_color, t),
            icon_color: self.icon_color.interpolate(other.icon_color, t),
            outline_width: self.outline_width.interpolate(other.outline_width, t),
            outline_color: self.outline_color.interpolate(other.outline_color, t),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    pub state_layer_selected: StateLayer,
    pub state_layer_unselected: StateLayer,
    pub checked: StateStyle,
    pub checked_disabled: StateStyle,
    pub unchecked: StateStyle,
    pub unchecked_disabled: StateStyle,
    pub indeterminate: StateStyle,
    pub indeterminate_disabled: StateStyle,
}

impl Style {
    pub fn new(theme: &(impl ColorScheme + ?Sized)) -> Self {
        Self::with_error_flag(theme, false)
    }

    pub fn error(theme: &(impl ColorScheme + ?Sized)) -> Self {
        Self::with_error_flag(theme, true)
    }

    fn with_error_flag(theme: &(impl ColorScheme + ?Sized), is_error: bool) -> Self {
        let (selected_bg, selected_icon, unselected_outline) = if is_error {
            (theme.error(), theme.on_error(), theme.error())
        } else {
            (
                theme.primary(),
                theme.on_primary(),
                theme.on_surface_variant(),
            )
        };

        let unselected_outline_disabled = if is_error {
            theme.error().scale_alpha(DISABLED_OPACITY)
        } else {
            theme.on_surface().scale_alpha(DISABLED_OPACITY)
        };

        Self {
            state_layer_selected: StateLayer::new(selected_bg),
            state_layer_unselected: StateLayer::new(unselected_outline),
            checked: StateStyle {
                background_color: selected_bg,
                icon_color: selected_icon,
                outline_width: 0.0,
                outline_color: Color::TRANSPARENT,
            },
            checked_disabled: StateStyle {
                background_color: theme.on_surface().scale_alpha(DISABLED_OPACITY),
                icon_color: theme.surface().scale_alpha(DISABLED_OPACITY),
                outline_width: 0.0,
                outline_color: Color::TRANSPARENT,
            },
            unchecked: StateStyle {
                background_color: Color::TRANSPARENT,
                icon_color: Color::TRANSPARENT,
                outline_width: OUTLINE_WIDTH,
                outline_color: unselected_outline,
            },
            unchecked_disabled: StateStyle {
                background_color: Color::TRANSPARENT,
                icon_color: Color::TRANSPARENT,
                outline_width: OUTLINE_WIDTH,
                outline_color: unselected_outline_disabled,
            },
            indeterminate: StateStyle {
                background_color: selected_bg,
                icon_color: selected_icon,
                outline_width: 0.0,
                outline_color: Color::TRANSPARENT,
            },
            indeterminate_disabled: StateStyle {
                background_color: theme.on_surface().scale_alpha(DISABLED_OPACITY),
                icon_color: theme.surface().scale_alpha(DISABLED_OPACITY),
                outline_width: 0.0,
                outline_color: Color::TRANSPARENT,
            },
        }
    }

    fn state_layer(&self, selected: bool) -> &StateLayer {
        if selected {
            &self.state_layer_selected
        } else {
            &self.state_layer_unselected
        }
    }

    fn state(&self, value: Value, enabled: bool) -> &StateStyle {
        match (value, enabled) {
            (Value::Checked, true) => &self.checked,
            (Value::Checked, false) => &self.checked_disabled,
            (Value::Indeterminate, true) => &self.indeterminate,
            (Value::Indeterminate, false) => &self.indeterminate_disabled,
            (Value::Unchecked, true) => &self.unchecked,
            (Value::Unchecked, false) => &self.unchecked_disabled,
        }
    }
}

pub struct Checkbox<Message>
where
    Message: Clone,
{
    style: Style,
    value: Value,
    on_toggle: Option<Message>,
    effects_spring: Option<Spring>,
    spatial_spring: Option<Spring>,
    style_checked: bool,
}

impl<Message> Checkbox<Message>
where
    Message: Clone,
{
    pub fn new(theme: &(impl ColorScheme + ?Sized), value: Value) -> Self {
        Self {
            style: Style::new(theme),
            value,
            on_toggle: None,
            effects_spring: None,
            spatial_spring: None,
            style_checked: false,
        }
    }

    pub fn error(mut self, is_error: bool, theme: &(impl ColorScheme + ?Sized)) -> Self {
        self.style = Style::with_error_flag(theme, is_error);
        self
    }

    pub fn on_toggle(mut self, on_toggle: Message) -> Self {
        self.on_toggle = Some(on_toggle);
        self
    }

    pub fn on_toggle_maybe(mut self, on_toggle: Option<Message>) -> Self {
        self.on_toggle = on_toggle;
        self
    }

    pub fn motion_scheme(mut self, scheme: motion::Scheme) -> Self {
        self.effects_spring = Some(fast_effects(scheme));
        self.spatial_spring = Some(fast_spatial(scheme));
        self
    }

    pub fn effects_spring(mut self, spring: Spring) -> Self {
        self.effects_spring = Some(spring);
        self
    }

    pub fn spatial_spring(mut self, spring: Spring) -> Self {
        self.spatial_spring = Some(spring);
        self
    }
}

struct State {
    ripple: Ripple,
    is_pressed: bool,
    check: SpringValue<f32>,
    dash: SpringValue<f32>,
    style_spring: SpringValue<StateStyle>,
    last_effects_spring: Spring,
    last_spatial_spring: Spring,
    last_style: Style,
    last_value: Value,
}

impl State {
    fn set_spatial_spring(&mut self, spatial: Spring) {
        self.check.spring.model = spatial;
        self.dash.spring.model = spatial;
    }

    fn set_effects_spring(&mut self, effects: Spring) {
        self.style_spring.spring.model = effects;
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for Checkbox<Message>
where
    Message: Clone,
    Renderer: iced_widget::renderer::geometry::Renderer,
{
    fn tag(&self) -> iced::advanced::widget::tree::Tag {
        iced::advanced::widget::tree::Tag::of::<State>()
    }

    fn state(&self) -> iced::advanced::widget::tree::State {
        let enabled = self.on_toggle.is_some();
        let target_style = *self.style.state(self.value, enabled);
        let now = Instant::now();
        let effects_spring = self
            .effects_spring
            .unwrap_or(fast_effects(motion::Scheme::default()));
        let spatial_spring = self
            .spatial_spring
            .unwrap_or(fast_spatial(motion::Scheme::default()));
        let (check, dash) = match self.value {
            Value::Unchecked => (0.0, 0.0),
            Value::Checked => (1.0, 0.0),
            Value::Indeterminate => (0.0, 1.0),
        };

        let state = State {
            ripple: Ripple::new(),
            is_pressed: false,
            check: SpringValue::new(check, spatial_spring, now),
            dash: SpringValue::new(dash, spatial_spring, now),
            style_spring: SpringValue::new(target_style, effects_spring, now),
            last_effects_spring: effects_spring,
            last_spatial_spring: spatial_spring,
            last_style: self.style,
            last_value: self.value,
        };

        iced::advanced::widget::tree::State::new(state)
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: Length::Fixed(TOUCH_TARGET_SIZE),
            height: Length::Fixed(TOUCH_TARGET_SIZE),
        }
    }

    fn layout(
        &mut self,
        _tree: &mut iced::advanced::widget::Tree,
        _renderer: &Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> Node {
        layout::atomic(limits, TOUCH_TARGET_SIZE, TOUCH_TARGET_SIZE)
    }

    fn draw(
        &self,
        tree: &iced::advanced::widget::Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &iced::advanced::renderer::Style,
        layout: iced::advanced::Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &iced::Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State>();
        let style = state.style_spring.value();
        let bounds = layout.bounds();

        let checkbox_bounds = Rectangle {
            x: bounds.x + (TOUCH_TARGET_SIZE - CHECKBOX_SIZE) / 2.0,
            y: bounds.y + (TOUCH_TARGET_SIZE - CHECKBOX_SIZE) / 2.0,
            width: CHECKBOX_SIZE,
            height: CHECKBOX_SIZE,
        };

        renderer.fill_quad(
            Quad {
                bounds: checkbox_bounds,
                border: Border::default()
                    .rounded(BORDER_RADIUS)
                    .color(style.outline_color)
                    .width(style.outline_width),
                ..Default::default()
            },
            style.background_color,
        );

        let check_val = state.check.value().max(0.0).min(1.0);
        let dash_val = state.dash.value().max(0.0).min(1.0);

        let has_icon = check_val > 0.05 || dash_val > 0.05;

        if has_icon {
            let enabled = self.on_toggle.is_some();
            let icon_source = if state.last_value.is_selected() {
                state.last_value
            } else if self.value.is_selected() {
                self.value
            } else {
                Value::Unchecked
            };
            let icon_color = self.style.state(icon_source, enabled).icon_color;

            if icon_color.a > 0.0 {
                let mut frame = canvas::Frame::new(&*renderer, checkbox_bounds.size());

                let left_progress = (check_val * 2.0).clamp(0.0, 1.0);
                if left_progress > 0.05 {
                    let start = Point::new(3.62, 9.0);
                    let end = Point::new(6.75, 12.13);
                    let alpha = icon_color.a * left_progress;
                    let color = Color {
                        r: icon_color.r,
                        g: icon_color.g,
                        b: icon_color.b,
                        a: alpha,
                    };
                    let current_end = Point::new(
                        start.x + (end.x - start.x) * left_progress,
                        start.y + (end.y - start.y) * left_progress,
                    );
                    let path = canvas::Path::line(start, current_end);
                    let stroke = canvas::Stroke::default()
                        .with_color(color)
                        .with_width(2.0)
                        .with_line_cap(canvas::LineCap::Round);
                    frame.stroke(&path, stroke);
                }

                let right_progress = ((check_val - 0.5) * 2.0).clamp(0.0, 1.0);
                if right_progress > 0.05 {
                    let start = Point::new(6.75, 12.13);
                    let end = Point::new(15.75, 5.25);
                    let alpha = icon_color.a * right_progress;
                    let color = Color {
                        r: icon_color.r,
                        g: icon_color.g,
                        b: icon_color.b,
                        a: alpha,
                    };
                    let current_end = Point::new(
                        start.x + (end.x - start.x) * right_progress,
                        start.y + (end.y - start.y) * right_progress,
                    );
                    let path = canvas::Path::line(start, current_end);
                    let stroke = canvas::Stroke::default()
                        .with_color(color)
                        .with_width(2.0)
                        .with_line_cap(canvas::LineCap::Round);
                    frame.stroke(&path, stroke);
                }

                if dash_val > 0.05 {
                    let start = Point::new(3.0, 9.0);
                    let end = Point::new(15.0, 9.0);
                    let alpha = icon_color.a * dash_val;
                    let color = Color {
                        r: icon_color.r,
                        g: icon_color.g,
                        b: icon_color.b,
                        a: alpha,
                    };
                    let current_end = Point::new(
                        start.x + (end.x - start.x) * dash_val,
                        start.y + (end.y - start.y) * dash_val,
                    );
                    let path = canvas::Path::line(start, current_end);
                    let stroke = canvas::Stroke::default()
                        .with_color(color)
                        .with_width(2.0)
                        .with_line_cap(canvas::LineCap::Round);
                    frame.stroke(&path, stroke);
                }

                let geometry = frame.into_geometry();

                renderer.with_translation(
                    Vector::new(checkbox_bounds.x, checkbox_bounds.y),
                    |renderer| {
                        renderer.draw_geometry(geometry);
                    },
                );
            }
        }
    }

    fn mouse_interaction(
        &self,
        _tree: &iced::advanced::widget::Tree,
        layout: iced::advanced::Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &iced::Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        if self.on_toggle.is_none() {
            return mouse::Interaction::default();
        }
        if cursor.is_over(layout.bounds()) {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }

    fn update(
        &mut self,
        tree: &mut iced::advanced::widget::Tree,
        event: &iced::Event,
        layout: iced::advanced::Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn iced::advanced::Clipboard,
        shell: &mut iced::advanced::Shell<'_, Message>,
        _viewport: &iced::Rectangle,
    ) {
        let state = tree.state.downcast_mut::<State>();
        let bounds = layout.bounds();
        let is_over = cursor.is_over(bounds);
        let now = Instant::now();
        let enabled = self.on_toggle.is_some();

        let layer_bounds = Rectangle {
            x: bounds.x + (TOUCH_TARGET_SIZE - STATE_LAYER_SIZE) / 2.0,
            y: bounds.y + (TOUCH_TARGET_SIZE - STATE_LAYER_SIZE) / 2.0,
            width: STATE_LAYER_SIZE,
            height: STATE_LAYER_SIZE,
        };
        state
            .ripple
            .handle_in(event, bounds, layer_bounds, cursor, enabled, shell);

        if let Some(effects_spring) = self.effects_spring
            && effects_spring != state.last_effects_spring
        {
            state.set_effects_spring(effects_spring);
        }
        if let Some(spatial_spring) = self.spatial_spring
            && spatial_spring != state.last_spatial_spring
        {
            state.set_spatial_spring(spatial_spring);
        }

        let target_style = *self.style.state(self.value, enabled);

        if !self.style_checked && self.style != state.last_style {
            state.style_spring.reset(target_style, now);
            state.last_style = self.style;
            self.style_checked = true;
            shell.request_redraw();
        } else {
            state.style_spring.set_target(target_style, now);
        }

        let (check_target, dash_target) = match self.value {
            Value::Unchecked => (0.0, 0.0),
            Value::Checked => (1.0, 0.0),
            Value::Indeterminate => (0.0, 1.0),
        };
        state.check.set_target(check_target, now);
        state.dash.set_target(dash_target, now);

        if !state.check.is_at_rest() || !state.dash.is_at_rest() || !state.style_spring.is_at_rest()
        {
            shell.request_redraw();
        }

        state.check.step(now);
        state.dash.step(now);
        state.style_spring.step(now);

        if state.check.is_at_rest() && state.dash.is_at_rest() && state.style_spring.is_at_rest() {
            state.last_value = self.value;
        }

        match event {
            iced::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                if is_over && enabled =>
            {
                shell.capture_event();
                state.is_pressed = true;
            }
            iced::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
                if state.is_pressed =>
            {
                shell.capture_event();
                state.is_pressed = false;
                if is_over && let Some(on_toggle) = self.on_toggle.clone() {
                    shell.publish(on_toggle);
                }
            }
            _ => {}
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut iced::advanced::widget::Tree,
        layout: iced::advanced::Layout<'b>,
        _renderer: &Renderer,
        _viewport: &iced::Rectangle,
        translation: iced::Vector,
    ) -> Option<iced::advanced::overlay::Element<'b, Message, Theme, Renderer>> {
        let state = tree.state.downcast_ref::<State>();
        let bounds = layout.bounds();
        let enabled = self.on_toggle.is_some();
        let layer_bounds = Rectangle {
            x: bounds.x + translation.x + (TOUCH_TARGET_SIZE - STATE_LAYER_SIZE) / 2.0,
            y: bounds.y + translation.y + (TOUCH_TARGET_SIZE - STATE_LAYER_SIZE) / 2.0,
            width: STATE_LAYER_SIZE,
            height: STATE_LAYER_SIZE,
        };
        (enabled && state.ripple.is_visible(layer_bounds.size(), enabled)).then(|| {
            let overlay = CheckboxOverlay {
                ripple: state.ripple,
                layer: *self.style.state_layer(self.value.is_selected()),
                bounds: layer_bounds,
            };
            overlay::Element::new(Box::new(overlay))
        })
    }
}

struct CheckboxOverlay {
    ripple: Ripple,
    layer: StateLayer,
    bounds: Rectangle,
}

impl<Message, Theme, Renderer> Overlay<Message, Theme, Renderer> for CheckboxOverlay
where
    Renderer: iced_widget::renderer::geometry::Renderer,
{
    fn layout(&mut self, _renderer: &Renderer, _bounds: Size) -> iced::advanced::layout::Node {
        Node::new(Size {
            width: self.bounds.width,
            height: self.bounds.height,
        })
        .move_to(Point::new(self.bounds.x, self.bounds.y))
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &iced::advanced::renderer::Style,
        layout: iced::advanced::Layout<'_>,
        _cursor: mouse::Cursor,
    ) {
        self.ripple.draw(
            renderer,
            layout.bounds(),
            Radius::from(f32::MAX),
            self.layer,
            true,
        );
    }
}

impl<'a, Message> From<Checkbox<Message>> for Element<'a, Message>
where
    Message: 'a + Clone,
{
    fn from(value: Checkbox<Message>) -> Self {
        Element::new(value)
    }
}
