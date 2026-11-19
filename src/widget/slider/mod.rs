use std::{fmt::Display, ops::RangeInclusive, time::Instant};

use iced::{
    Border, Color, Element, Event, Length, Point, Rectangle,
    advanced::{
        Overlay, Widget, layout, mouse, overlay,
        renderer::Quad,
        widget::{Id, Tree, operation},
    },
    border::Radius,
    keyboard::{self, Key, key},
    touch,
};
use iced_widget::core::{
    Svg, alignment,
    text::{self, Text, paragraph},
};

use crate::{
    animation::{
        Interpolable,
        motion::{self, Spring, SpringValue, fast_effects, fast_spatial},
    },
    style::StateLayer,
    theme::ColorScheme,
    widget::{hybrid_icon::Icon, ripple::Ripple},
};

// The range slider lives in its own submodule named after the widget it
// holds; the module mirrors `slider/mod.rs` on purpose.
#[allow(clippy::module_inception)]
pub mod slider;
pub use slider::RangeSlider;

const HANDLE_WIDTH_IDLE: f32 = 4.0;
const HANDLE_WIDTH_DRAGGED: f32 = 2.0;
const HANDLE_GAP: f32 = 6.0;
/// The outline around the inactive handle of a range slider when both
/// handles overlap.
const HANDLE_OVERLAP_WIDTH: f32 = 2.0;
const INNER_CORNER_RADIUS: f32 = 2.0;
const STOP_INDICATOR_SIZE: f32 = 4.0;
const STOP_INDICATOR_TRAILING_SPACE: f32 = 4.0;
/// The smallest stop indicator gap around a handle that still fits the
/// trailing stop indicator.
const STOP_INDICATOR_MIN_SEGMENT: f32 = 12.0;

const STATE_LAYER_MIN_SIZE: f32 = 48.0;
const FOCUS_RING_MIN_SIZE: f32 = 48.0;
const FOCUS_RING_WIDTH: f32 = 2.0;

const LABEL_MIN_HEIGHT: f32 = 32.0;
const LABEL_HORIZONTAL_PADDING: f32 = 10.0;
const LABEL_TEXT_SIZE: f32 = 12.0;
const LABEL_GAP: f32 = 8.0;
/// Value indicator text fades in during the last part of the scale animation.
const LABEL_TEXT_FADE_START: f32 = 0.85;

const DISABLED_ACTIVE_OPACITY: f32 = 0.38;
const DISABLED_INACTIVE_OPACITY: f32 = 0.12;
const DISABLED_INDICATOR_OPACITY: f32 = 0.38;

/// The time constant, in seconds, of the exponential chase of the pointer
/// while dragging: the handle trails by this delay, and the gap it keeps
/// becomes the momentum the release hands over to the spatial spring.
const DRAG_CHASE_DECAY: f32 = 0.008;

/// A huge rectangle used as a viewport for text rendered from an overlay,
/// where no natural viewport exists.
const HUGE_VIEWPORT: Rectangle = Rectangle {
    x: -1.0e6,
    y: -1.0e6,
    width: 2.0e6,
    height: 2.0e6,
};

/// The track and handle metrics of a [`Slider`] for each Material size.
pub struct Size {
    pub track_height: f32,
    pub handle_height: f32,
    pub corner_radius: f32,
}

impl Size {
    pub fn extra_small() -> Self {
        Self {
            track_height: 16.0,
            handle_height: 44.0,
            corner_radius: 8.0,
        }
    }

    pub fn small() -> Self {
        Self {
            track_height: 24.0,
            handle_height: 44.0,
            corner_radius: 8.0,
        }
    }

    pub fn medium() -> Self {
        Self {
            track_height: 40.0,
            handle_height: 52.0,
            corner_radius: 12.0,
        }
    }

    pub fn large() -> Self {
        Self {
            track_height: 56.0,
            handle_height: 68.0,
            corner_radius: 16.0,
        }
    }

    pub fn extra_large() -> Self {
        Self {
            track_height: 96.0,
            handle_height: 108.0,
            corner_radius: 28.0,
        }
    }
}

impl Default for Size {
    fn default() -> Self {
        Self::extra_small()
    }
}

/// The orientation of a [`Slider`].
///
/// Vertical sliders have their zero value at the bottom.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Axis {
    /// A horizontal slider, with its zero value on the left.
    #[default]
    Horizontal,
    /// A vertical slider, with its zero value at the bottom.
    Vertical,
}

/// Whether, and how, a [`Slider`] shows a value indicator while its handle is
/// interacted with.
pub enum ValueIndicator<'a, T> {
    /// The value indicator is never shown.
    None,
    /// The value is rounded to the slider's `decimal_places` digits after
    /// the decimal point, with trailing zeros trimmed.
    Value,
    /// The value is formatted with the given function.
    Format(Box<dyn Fn(T) -> String + 'a>),
}

impl<'a, T> ValueIndicator<'a, T> {
    fn format(&self, value: T, decimal_places: usize, min_decimal_places: usize) -> Option<String>
    where
        T: Display + Into<f64>,
    {
        match self {
            ValueIndicator::None => None,
            ValueIndicator::Value => Some(format_decimal(
                value.into(),
                decimal_places,
                min_decimal_places,
            )),
            ValueIndicator::Format(format) => Some(format(value)),
        }
    }
}

/// Formats a value with the given maximum number of digits after the decimal
/// point, trimming trailing zeros while keeping at least `min_decimal_places`.
fn format_decimal(value: f64, decimal_places: usize, min_decimal_places: usize) -> String {
    let formatted = if value.abs() < 0.5 / 10_f64.powi(decimal_places as i32) {
        format!("{0:.1$}", 0.0, decimal_places)
    } else {
        format!("{value:.decimal_places$}")
    };
    trim_trailing_zeros(formatted, min_decimal_places)
}

fn trim_trailing_zeros(formatted: String, min_decimal_places: usize) -> String {
    if let Some(dot_pos) = formatted.find('.') {
        let integer = &formatted[..dot_pos];
        let fraction = &formatted[dot_pos + 1..];
        let trimmed = fraction.trim_end_matches('0');
        let kept_len = trimmed.len().max(min_decimal_places).min(fraction.len());
        if kept_len > 0 {
            format!("{integer}.{}", &fraction[..kept_len])
        } else {
            integer.to_string()
        }
    } else {
        formatted
    }
}

fn step_decimal_places(step: f64) -> usize {
    const EPS: f64 = 1e-9;
    if step <= 0.0 {
        return 1;
    }
    let mut f = step.fract().abs();
    if f < EPS {
        return 0;
    }
    let mut count = 0;
    while f > EPS && count < 10 {
        f = (f * 10.0).fract();
        count += 1;
    }
    count
}

/// An icon inset in the track of a [`Slider`].
///
/// The icon switches between its two variants as the active track grows past
/// it: while it sits on the active track the `on_active` icon is drawn, once
/// the active track recedes past it the `on_inactive` icon takes its place.
///
/// Inset icons are only meant for medium, large and extra-large sliders.
pub struct InsetIcon<'a> {
    /// The icon drawn while sitting on the active track.
    pub on_active: Icon<'a>,
    /// The icon drawn while sitting on the inactive track.
    pub on_inactive: Icon<'a>,
}

impl<'a> InsetIcon<'a> {
    #[must_use]
    pub fn new(icon: Icon<'a>) -> Self {
        Self {
            on_active: icon.clone(),
            on_inactive: icon,
        }
    }

    #[must_use]
    pub fn dual(on_active: Icon<'a>, on_inactive: Icon<'a>) -> Self {
        Self {
            on_active,
            on_inactive,
        }
    }
}

/// The colors of a [`Slider`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// The active part of the track.
    pub active_track: Color,
    /// The inactive part of the track.
    pub inactive_track: Color,
    /// The handle.
    pub handle: Color,
    /// The stop indicator at the end of the track.
    pub stop_indicator: Color,
    /// Stop indicators sitting on the active track.
    pub stop_indicators_active: Color,
    /// Stop indicators sitting on the inactive track.
    pub stop_indicators_inactive: Color,
    /// The container of the value indicator.
    pub label_container: Color,
    /// The text of the value indicator.
    pub label_text: Color,
    /// The state layer of the handle.
    pub state_layer: StateLayer,
    /// The focus ring of the handle.
    pub focus_ring: Color,
    /// Inset icons sitting on the active track.
    pub icon_active: Color,
    /// Inset icons sitting on the inactive track.
    pub icon_inactive: Color,
    /// The outline separating the handles of a [`RangeSlider`] when they
    /// overlap.
    pub handle_overlap_outline: Color,
    /// The active part of the track while disabled.
    pub active_track_disabled: Color,
    /// The inactive part of the track while disabled.
    pub inactive_track_disabled: Color,
    /// The handle while disabled.
    pub handle_disabled: Color,
    /// Stop indicators while disabled.
    pub stop_indicators_disabled: Color,
    /// Inset icons while disabled.
    pub icon_disabled: Color,
}

impl Style {
    #[must_use]
    pub fn new(theme: &(impl ColorScheme + ?Sized)) -> Self {
        Self {
            active_track: theme.primary(),
            inactive_track: theme.surface_container_highest(),
            handle: theme.primary(),
            stop_indicator: theme.primary(),
            stop_indicators_active: theme.surface_container_highest(),
            stop_indicators_inactive: theme.primary(),
            label_container: theme.primary(),
            label_text: theme.on_primary(),
            state_layer: StateLayer::new(theme.primary()),
            focus_ring: theme.primary(),
            icon_active: theme.on_primary(),
            icon_inactive: theme.on_surface_variant(),
            handle_overlap_outline: theme.on_primary(),
            active_track_disabled: theme.on_surface().scale_alpha(DISABLED_ACTIVE_OPACITY),
            inactive_track_disabled: theme.on_surface().scale_alpha(DISABLED_INACTIVE_OPACITY),
            handle_disabled: theme.on_surface().scale_alpha(DISABLED_ACTIVE_OPACITY),
            stop_indicators_disabled: theme.on_surface().scale_alpha(DISABLED_INDICATOR_OPACITY),
            icon_disabled: theme.on_surface().scale_alpha(DISABLED_INDICATOR_OPACITY),
        }
    }

    /// The style while the slider is disabled: only the track, handle, stop
    /// indicators and inset icons change appearance.
    pub(crate) fn disabled(self) -> Self {
        Self {
            active_track: self.active_track_disabled,
            inactive_track: self.inactive_track_disabled,
            handle: self.handle_disabled,
            stop_indicator: self.stop_indicators_disabled,
            stop_indicators_active: self.stop_indicators_disabled,
            stop_indicators_inactive: self.stop_indicators_disabled,
            icon_active: self.icon_disabled,
            icon_inactive: self.icon_disabled,
            ..self
        }
    }
}

impl Interpolable for Style {
    fn interpolate(self, other: Self, t: f32) -> Self {
        Self {
            active_track: self.active_track.interpolate(other.active_track, t),
            inactive_track: self.inactive_track.interpolate(other.inactive_track, t),
            handle: self.handle.interpolate(other.handle, t),
            stop_indicator: self.stop_indicator.interpolate(other.stop_indicator, t),
            stop_indicators_active: self
                .stop_indicators_active
                .interpolate(other.stop_indicators_active, t),
            stop_indicators_inactive: self
                .stop_indicators_inactive
                .interpolate(other.stop_indicators_inactive, t),
            label_container: self.label_container.interpolate(other.label_container, t),
            label_text: self.label_text.interpolate(other.label_text, t),
            state_layer: self.state_layer.interpolate(other.state_layer, t),
            focus_ring: self.focus_ring.interpolate(other.focus_ring, t),
            icon_active: self.icon_active.interpolate(other.icon_active, t),
            icon_inactive: self.icon_inactive.interpolate(other.icon_inactive, t),
            handle_overlap_outline: self
                .handle_overlap_outline
                .interpolate(other.handle_overlap_outline, t),
            active_track_disabled: self
                .active_track_disabled
                .interpolate(other.active_track_disabled, t),
            inactive_track_disabled: self
                .inactive_track_disabled
                .interpolate(other.inactive_track_disabled, t),
            handle_disabled: self.handle_disabled.interpolate(other.handle_disabled, t),
            stop_indicators_disabled: self
                .stop_indicators_disabled
                .interpolate(other.stop_indicators_disabled, t),
            icon_disabled: self.icon_disabled.interpolate(other.icon_disabled, t),
        }
    }
}

/// The internal state of a [`Slider`].
struct State<P: text::Paragraph> {
    is_dragging: bool,
    drag_factor: Option<f32>,
    /// The place the handle chases while dragging, trailing the pointer by a
    /// small delay. The chase mirrors itself into the position spring, which
    /// stays the single source the track fill and the label follow.
    drag_display: f32,
    keyboard_modifiers: keyboard::Modifiers,
    is_focused: bool,
    /// Whether the focus was granted by the keyboard: the focus ring and the
    /// resting value indicator are shown for keyboard focus only, pointer
    /// focus just enables the keyboard shortcuts.
    keyboard_focus: bool,
    ripple: Ripple,
    handle_width_spring: SpringValue<f32>,
    handle_position_spring: SpringValue<f32>,
    label_scale_spring: SpringValue<f32>,
    focus_ring_spring: SpringValue<f32>,
    icon_start_visibility_spring: SpringValue<f32>,
    icon_end_visibility_spring: SpringValue<f32>,
    style_spring: SpringValue<Style>,
    label_paragraph: paragraph::Plain<P>,
    label_content: String,
    last_effects_spring: Spring,
    last_spatial_spring: Spring,
    last_style: Style,
}

impl<P: text::Paragraph> State<P> {
    fn set_effects_spring(&mut self, effects: Spring) {
        self.label_scale_spring.spring.model = effects;
        self.focus_ring_spring.spring.model = effects;
        self.icon_start_visibility_spring.spring.model = effects;
        self.icon_end_visibility_spring.spring.model = effects;
        self.style_spring.spring.model = effects;
    }

    fn set_spatial_spring(&mut self, spatial: Spring) {
        self.handle_width_spring.spring.model = spatial;
        self.handle_position_spring.spring.model = spatial;
    }

    fn is_animating(&self) -> bool {
        !self.handle_width_spring.is_at_rest()
            || !self.handle_position_spring.is_at_rest()
            || !self.label_scale_spring.is_at_rest()
            || !self.focus_ring_spring.is_at_rest()
            || !self.icon_start_visibility_spring.is_at_rest()
            || !self.icon_end_visibility_spring.is_at_rest()
            || !self.style_spring.is_at_rest()
    }

    fn step(&mut self, now: Instant) {
        // While a drag chases the pointer, the chase drives the position
        // spring through itself: the spring is reset to the chased place
        // every frame, so the handle trails the pointer by a fixed,
        // frame rate independent delay.
        if self.is_dragging
            && let Some(target) = self.drag_factor
        {
            let spring = &mut self.handle_position_spring;
            let delta = now.duration_since(spring.spring.time).as_secs_f32();
            let decay = 1.0 - (-delta / DRAG_CHASE_DECAY).exp();
            self.drag_display += (target - self.drag_display) * decay;
            spring.reset(self.drag_display, now);
        }

        self.handle_width_spring.step(now);
        self.handle_position_spring.step(now);
        self.label_scale_spring.step(now);
        self.focus_ring_spring.step(now);
        self.icon_start_visibility_spring.step(now);
        self.icon_end_visibility_spring.step(now);
        self.style_spring.step(now);
    }
}

impl<P: text::Paragraph> operation::Focusable for State<P> {
    fn is_focused(&self) -> bool {
        self.is_focused
    }

    fn focus(&mut self) {
        self.is_focused = true;
        self.keyboard_focus = true;
    }

    fn unfocus(&mut self) {
        self.is_focused = false;
        self.keyboard_focus = false;
    }
}

/// A slider with a single handle.
///
/// Sliders react to pointer, touch and keyboard interaction: clicking the
/// track moves the handle there and starts a drag, the arrow keys move it by
/// one step while the slider is focused, and the Home, End, PageUp and
/// PageDown keys jump to well-known values. While dragged, the handle trails
/// the pointer slightly and settles with a soft wobble when released.
///
/// The value snaps to whole numbers by default; see [`Slider::step`] and
/// [`Slider::decimal_places`].
pub struct Slider<'a, T, Message> {
    id: Option<Id>,
    range: RangeInclusive<T>,
    step: T,
    shift_step: Option<T>,
    value: T,
    default: Option<T>,
    on_change: Option<Box<dyn Fn(T) -> Message + 'a>>,
    on_release: Option<Message>,
    width: Length,
    height: Length,
    style: Style,
    size: Size,
    axis: Axis,
    centered: bool,
    stops: bool,
    continuous_tick_count: Option<u32>,
    value_indicator: ValueIndicator<'a, T>,
    decimal_places: usize,
    inset_icon_start: Option<InsetIcon<'a>>,
    inset_icon_end: Option<InsetIcon<'a>>,
    effects_spring: Option<Spring>,
    spatial_spring: Option<Spring>,
}

impl<'a, T, Message> Slider<'a, T, Message>
where
    T: Copy + From<u8> + PartialOrd,
{
    #[must_use]
    pub fn new<F>(range: RangeInclusive<T>, value: T, on_change: F, style: Style) -> Self
    where
        F: 'a + Fn(T) -> Message,
    {
        let value = if value >= *range.start() {
            value
        } else {
            *range.start()
        };

        let value = if value <= *range.end() {
            value
        } else {
            *range.end()
        };

        Slider {
            id: None,
            value,
            default: None,
            range,
            // One by default, so the values snap to whole numbers and the
            // value indicator shows them without fractional noise
            step: T::from(1),
            shift_step: None,
            on_change: Some(Box::new(on_change)),
            on_release: None,
            width: Length::Fill,
            height: Length::Fill,
            style,
            size: Size::default(),
            axis: Axis::default(),
            centered: false,
            stops: true,
            continuous_tick_count: None,
            value_indicator: ValueIndicator::None,
            decimal_places: 1,
            inset_icon_start: None,
            inset_icon_end: None,
            effects_spring: None,
            spatial_spring: None,
        }
    }

    /// Sets the [`Id`] of the [`Slider`], used by focus operations.
    #[must_use]
    pub fn id(mut self, id: impl Into<Id>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Sets the message that replaces `on_change` if `Some`, disabling the
    /// [`Slider`] when `None`.
    #[must_use]
    pub fn on_change_maybe<F>(mut self, on_change: Option<F>) -> Self
    where
        F: 'a + Fn(T) -> Message,
    {
        self.on_change = on_change.map(|on_change| Box::new(on_change) as _);
        self
    }

    /// Sets the optional default value for the [`Slider`].
    ///
    /// If set, the [`Slider`] will reset to this value when ctrl-clicked or command-clicked.
    #[must_use]
    pub fn default(mut self, default: impl Into<T>) -> Self {
        self.default = Some(default.into());
        self
    }

    /// Sets the release message of the [`Slider`].
    /// This is called when the mouse is released from the slider.
    ///
    /// Typically, the user's interaction with the slider is finished when this message is produced.
    /// This is useful if you need to spawn a long-running task from the slider's result, where
    /// the default on_change message could create too many events.
    #[must_use]
    pub fn on_release(mut self, on_release: Message) -> Self {
        self.on_release = Some(on_release);
        self
    }

    /// Sets the width of the [`Slider`].
    #[must_use]
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of a vertical [`Slider`].
    #[must_use]
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Sets the step size of the [`Slider`].
    ///
    /// The value snaps to multiples of the step. A step of zero makes the
    /// [`Slider`] continuous.
    ///
    /// Defaults to 1.
    #[must_use]
    pub fn step(mut self, step: impl Into<T>) -> Self {
        self.step = step.into();
        self
    }

    /// Sets the optional "shift" step for the [`Slider`].
    ///
    /// If set, this value is used as the step while the shift key is pressed.
    #[must_use]
    pub fn shift_step(mut self, shift_step: impl Into<T>) -> Self {
        self.shift_step = Some(shift_step.into());
        self
    }

    /// Sets the size of the [`Slider`].
    #[must_use]
    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }

    /// Sets the orientation of the [`Slider`].
    #[must_use]
    pub fn axis(mut self, axis: Axis) -> Self {
        self.axis = axis;
        self
    }

    /// Makes the [`Slider`] vertical.
    #[must_use]
    pub fn vertical(self) -> Self {
        self.axis(Axis::Vertical)
    }

    /// Makes the [`Slider`] grow its active track from the middle of the
    /// range instead of its start, like a balance control.
    #[must_use]
    pub fn centered(mut self, centered: bool) -> Self {
        self.centered = centered;
        self
    }

    /// Sets whether the stop indicators along the track are drawn.
    #[must_use]
    pub fn stops(mut self, stops: bool) -> Self {
        self.stops = stops;
        self
    }

    /// Draws the given number of evenly spaced tick marks along the track,
    /// without snapping the value to them.
    #[must_use]
    pub fn continuous_tick_count(mut self, count: u32) -> Self {
        self.continuous_tick_count = Some(count);
        self
    }

    /// Sets how the value indicator is shown while the handle is dragged,
    /// hovered or focused.
    #[must_use]
    pub fn value_indicator(mut self, value_indicator: ValueIndicator<'a, T>) -> Self {
        self.value_indicator = value_indicator;
        self
    }

    /// Sets how many digits after the decimal point [`ValueIndicator::Value`]
    /// shows, rounding the excess away and trimming trailing zeros.
    ///
    /// Defaults to 1.
    #[must_use]
    pub fn decimal_places(mut self, decimal_places: usize) -> Self {
        self.decimal_places = decimal_places;
        self
    }

    /// Sets the icon inset at the start of the track.
    #[must_use]
    pub fn inset_icon_start(mut self, icon: InsetIcon<'a>) -> Self {
        self.inset_icon_start = Some(icon);
        self
    }

    /// Sets the icon inset at the end of the track.
    #[must_use]
    pub fn inset_icon_end(mut self, icon: InsetIcon<'a>) -> Self {
        self.inset_icon_end = Some(icon);
        self
    }

    /// Sets the motion scheme used by the [`Slider`].
    #[must_use]
    pub fn motion_scheme(mut self, scheme: motion::Scheme) -> Self {
        self.effects_spring = Some(fast_effects(scheme));
        self.spatial_spring = Some(fast_spatial(scheme));
        self
    }

    /// Sets the spring used for effects, like the value indicator.
    #[must_use]
    pub fn effects_spring(mut self, spring: Spring) -> Self {
        self.effects_spring = Some(spring);
        self
    }

    /// Sets the spring used for spatial motion, like the handle.
    #[must_use]
    pub fn spatial_spring(mut self, spring: Spring) -> Self {
        self.spatial_spring = Some(spring);
        self
    }
}

impl<'a, T, Message> Slider<'a, T, Message>
where
    T: Copy + Display + Into<f64> + PartialOrd,
{
    fn format_value(&self) -> Option<String> {
        let step_f64: f64 = self.step.into();
        let min_decimal_places = step_decimal_places(step_f64);
        let decimal_places = self.decimal_places.max(min_decimal_places);
        self.value_indicator
            .format(self.value, decimal_places, min_decimal_places)
    }
}

impl<T, Message, Theme, Renderer> Widget<Message, Theme, Renderer> for Slider<'_, T, Message>
where
    T: Copy + Display + Into<f64> + num_traits::FromPrimitive + PartialOrd,
    Message: Clone,
    Renderer: iced::advanced::text::Renderer
        + iced::advanced::svg::Renderer
        + iced_widget::renderer::geometry::Renderer,
    Renderer::Font: From<iced::Font>,
{
    fn tag(&self) -> iced::advanced::widget::tree::Tag {
        iced::advanced::widget::tree::Tag::of::<State<Renderer::Paragraph>>()
    }

    fn state(&self) -> iced::advanced::widget::tree::State {
        let now = Instant::now();
        let effects_spring = self
            .effects_spring
            .unwrap_or(fast_effects(motion::Scheme::default()));
        let spatial_spring = self
            .spatial_spring
            .unwrap_or(fast_spatial(motion::Scheme::default()));
        let factor = value_factor(self.value, &self.range);
        let label_content = self.format_value().unwrap_or_default();

        let state: State<Renderer::Paragraph> = State {
            is_dragging: false,
            drag_factor: None,
            drag_display: factor,
            keyboard_modifiers: keyboard::Modifiers::default(),
            is_focused: false,
            keyboard_focus: false,
            ripple: Ripple::new(),
            handle_width_spring: SpringValue::new(HANDLE_WIDTH_IDLE, spatial_spring, now),
            handle_position_spring: SpringValue::new(factor, spatial_spring, now),
            label_scale_spring: SpringValue::new(0.0, effects_spring, now),
            focus_ring_spring: SpringValue::new(0.0, effects_spring, now),
            icon_start_visibility_spring: SpringValue::new(0.0, effects_spring, now),
            icon_end_visibility_spring: SpringValue::new(0.0, effects_spring, now),
            style_spring: SpringValue::new(self.style, effects_spring, now),
            label_paragraph: paragraph::Plain::new(label_text(
                Renderer::ICON_FONT,
                label_content.clone(),
            )),
            label_content,
            last_effects_spring: effects_spring,
            last_spatial_spring: spatial_spring,
            last_style: self.style,
        };

        iced::advanced::widget::tree::State::new(state)
    }

    fn diff(&self, tree: &mut Tree) {
        let state = tree.state.downcast_mut::<State<Renderer::Paragraph>>();

        // Stop dragging if the slider becomes disabled
        if self.on_change.is_none() {
            state.is_dragging = false;
            state.drag_factor = None;
        }
    }

    fn size(&self) -> iced::Size<Length> {
        match self.axis {
            Axis::Horizontal => iced::Size {
                width: self.width,
                height: Length::Fixed(self.size.handle_height),
            },
            Axis::Vertical => iced::Size {
                width: Length::Fixed(self.size.handle_height),
                height: self.height,
            },
        }
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> layout::Node {
        match self.axis {
            Axis::Horizontal => layout::atomic(limits, self.width, self.size.handle_height),
            Axis::Vertical => layout::atomic(limits, self.size.handle_height, self.height),
        }
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: layout::Layout<'_>,
        _renderer: &Renderer,
        operation: &mut dyn operation::Operation,
    ) {
        let state = tree.state.downcast_mut::<State<Renderer::Paragraph>>();
        operation.focusable(self.id.as_ref(), layout.bounds(), state);
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: layout::Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        _clipboard: &mut dyn iced::advanced::Clipboard,
        shell: &mut iced::advanced::Shell<'_, Message>,
        _viewport: &iced::Rectangle,
    ) {
        let state = tree.state.downcast_mut::<State<Renderer::Paragraph>>();
        let bounds = layout.bounds();
        let enabled = self.on_change.is_some();
        let now = Instant::now();

        // --- Ripple on the handle state layer
        let len = main_len(self.axis, bounds);
        let handle_width = state.handle_width_spring.value();
        let handle_center = center_at(len, handle_width, state.handle_position_spring.value());
        let circle = handle_circle(self.axis, bounds, self.size.handle_height, handle_center);
        // While a drag is in progress the pointer can leave the handle, but
        // the state layer has to stay hovered until the button is released
        let interaction_bounds = if state.is_dragging {
            huge_bounds(bounds)
        } else {
            circle
        };
        state
            .ripple
            .handle_in(event, interaction_bounds, circle, cursor, enabled, shell);

        if let Some(effects_spring) = self.effects_spring
            && effects_spring != state.last_effects_spring
        {
            state.last_effects_spring = effects_spring;
            state.set_effects_spring(effects_spring);
        }
        if let Some(spatial_spring) = self.spatial_spring
            && spatial_spring != state.last_spatial_spring
        {
            state.last_spatial_spring = spatial_spring;
            state.set_spatial_spring(spatial_spring);
        }

        // --- Springs
        let resolved_style = if enabled {
            self.style
        } else {
            self.style.disabled()
        };
        if resolved_style != state.last_style {
            state.last_style = resolved_style;
            state.style_spring.reset(resolved_style, now);
        } else {
            state.style_spring.set_target(resolved_style, now);
        }

        // While a drag chases the pointer, `State::step` keeps the handle's
        // spring pinned to the chase; otherwise it glides to the value.
        if state.drag_factor.is_none() {
            state
                .handle_position_spring
                .set_target(value_factor(self.value, &self.range), now);
        }
        state.handle_width_spring.set_target(
            match state.is_dragging {
                true => HANDLE_WIDTH_DRAGGED,
                false => HANDLE_WIDTH_IDLE,
            },
            now,
        );

        let cursor_over_handle = enabled && cursor.is_over(bounds) && cursor.is_over(circle);
        let label_visible = !matches!(self.value_indicator, ValueIndicator::None)
            && enabled
            && (state.is_dragging || cursor_over_handle || state.keyboard_focus);
        state
            .label_scale_spring
            .set_target(label_visible as u32 as f32, now);
        state
            .focus_ring_spring
            .set_target((state.keyboard_focus && enabled) as u32 as f32, now);

        // --- Value indicator content
        let label_content = self.format_value().unwrap_or_default();
        if state.label_content != label_content {
            state.label_content = label_content;
            // The paragraph is bootstrapped with the renderer's icon font;
            // refresh it so that it settles on the renderer's default font and
            // re-measures when the content changes.
            let _ = state.label_paragraph.update(label_text(
                renderer.default_font(),
                state.label_content.as_str(),
            ));
        }

        // --- Inset icon crossfades
        if self.inset_icon_start.is_some() || self.inset_icon_end.is_some() {
            let track_height = self.size.track_height;
            let active = self
                .active_main_range(len, handle_width, handle_center)
                .valid();
            if self.inset_icon_start.is_some() {
                let center = track_height / 2.0;
                state.icon_start_visibility_spring.set_target(
                    active.is_some_and(|active| active.contains(&center)) as u32 as f32,
                    now,
                );
            }
            if self.inset_icon_end.is_some() {
                let center = len - track_height / 2.0;
                state.icon_end_visibility_spring.set_target(
                    active.is_some_and(|active| active.contains(&center)) as u32 as f32,
                    now,
                );
            }
        }

        if state.is_animating() {
            shell.request_redraw();
        }
        // Those DO need to be updated every frame, otherwise wacky things happen
        state.step(now);

        // --- Interaction
        let axis = self.axis;
        let range = self.range.clone();
        let range_step = effective_step(self.step);
        let locate = move |position: Point| -> Option<T> {
            value_at(axis, bounds, position, &range, range_step)
        };

        let change =
            |slider: &mut Self, shell: &mut iced::advanced::Shell<'_, Message>, new_value: T| {
                if (slider.value.into() - new_value.into()).abs() > f64::EPSILON {
                    if let Some(on_change) = &slider.on_change {
                        shell.publish(on_change(new_value));
                    }
                    slider.value = new_value;
                }
            };

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                if !enabled {
                    return;
                }

                if let Some(position) = cursor.position_over(bounds) {
                    if !state.is_focused {
                        state.is_focused = true;
                        shell.request_redraw();
                    }
                    // Pointer focus enables the keyboard shortcuts, but hides
                    // the keyboard-only focus ring and resting value indicator
                    state.keyboard_focus = false;

                    if state.keyboard_modifiers.command() {
                        let _ = self.default.map(|default| change(self, shell, default));
                        state.is_dragging = false;
                        state.drag_factor = None;
                    } else if let Some(new_value) = locate(position) {
                        change(self, shell, new_value);
                        state.is_dragging = true;
                        // No drag factor yet: the handle glides from its
                        // current position to the pressed one instead of
                        // teleporting. Direct pointer tracking starts with
                        // the first cursor move.
                        state.drag_factor = None;
                    }

                    shell.capture_event();
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerLifted { .. })
            | Event::Touch(touch::Event::FingerLost { .. }) => {
                if state.is_dragging {
                    // Hand the chase over to the spatial spring: the gap the
                    // chase still carries turns into the momentum of a damped
                    // settling wobble around the value.
                    if let Some(target) = state.drag_factor {
                        let gap = target - state.drag_display;
                        if gap.abs() > f32::EPSILON {
                            state.handle_position_spring.set_target(target, now);
                            state.handle_position_spring.spring.velocity = 1.0 / DRAG_CHASE_DECAY;
                        }
                    }

                    if let Some(on_release) = self.on_release.clone() {
                        shell.publish(on_release);
                    }
                    state.is_dragging = false;
                    state.drag_factor = None;
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { .. })
            | Event::Touch(touch::Event::FingerMoved { .. }) => {
                if state.is_dragging && enabled {
                    if let Some(position) = cursor.land().position()
                        && let Some(new_value) = locate(position)
                    {
                        change(self, shell, new_value);
                        // The chase continues from wherever the press glide
                        // has brought the handle when the pointer first moves.
                        if state.drag_factor.is_none() {
                            state.drag_display = state.handle_position_spring.value();
                        }
                        state.drag_factor = Some(pointer_factor(axis, bounds, position));
                    }

                    shell.capture_event();
                }
            }
            Event::Mouse(mouse::Event::WheelScrolled { delta })
                if state.keyboard_modifiers.control() =>
            {
                if cursor.is_over(bounds) && enabled {
                    let delta = match delta {
                        mouse::ScrollDelta::Lines { x: _, y } => y,
                        mouse::ScrollDelta::Pixels { x: _, y } => y,
                    };

                    let range = range_f64(&self.range);
                    let step = keyboard_step(
                        self.step,
                        self.shift_step,
                        state.keyboard_modifiers.shift(),
                        &range,
                    );

                    let new_value = if *delta < 0.0 {
                        decrement(self.value.into(), &range, step)
                    } else {
                        increment(self.value.into(), &range, step)
                    };

                    if let Some(new_value) = T::from_f64(new_value) {
                        change(self, shell, new_value);
                    }

                    shell.capture_event();
                }
            }
            Event::Keyboard(keyboard::Event::KeyPressed { key, .. }) => {
                if !state.is_focused || !enabled {
                    return;
                }

                let range = range_f64(&self.range);
                let step = keyboard_step(
                    self.step,
                    self.shift_step,
                    state.keyboard_modifiers.shift(),
                    &range,
                );
                let current: f64 = self.value.into();
                let page = (range.end - range.start) / 10.0;

                let new_value = match key {
                    Key::Named(key::Named::ArrowUp | key::Named::ArrowRight) => {
                        Some(increment(current, &range, step))
                    }
                    Key::Named(key::Named::ArrowDown | key::Named::ArrowLeft) => {
                        Some(decrement(current, &range, step))
                    }
                    Key::Named(key::Named::Home) => Some(range.start),
                    Key::Named(key::Named::End) => Some(range.end),
                    Key::Named(key::Named::PageUp) => Some((current + page).min(range.end)),
                    Key::Named(key::Named::PageDown) => Some((current - page).max(range.start)),
                    _ => None,
                };

                if let Some(new_value) = new_value
                    && let Some(new_value) = T::from_f64(new_value)
                {
                    change(self, shell, new_value);
                    shell.capture_event();
                }
            }
            Event::Keyboard(keyboard::Event::KeyReleased {
                key: Key::Named(key::Named::Escape),
                ..
            }) => {
                if state.is_focused {
                    state.is_focused = false;
                    state.keyboard_focus = false;
                    shell.capture_event();
                }
            }
            Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) => {
                state.keyboard_modifiers = *modifiers;
            }
            _ => {}
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &iced::advanced::renderer::Style,
        layout: layout::Layout<'_>,
        _cursor: mouse::Cursor,
        viewport: &iced::Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State<Renderer::Paragraph>>();
        let style = state.style_spring.value();
        let enabled = self.on_change.is_some();
        let bounds = layout.bounds();
        let len = main_len(self.axis, bounds);
        let track_height = self.size.track_height;

        let handle_width = state.handle_width_spring.value();
        let handle_center = center_at(len, handle_width, state.handle_position_spring.value());

        let cross_center = match self.axis {
            Axis::Horizontal => bounds.y + bounds.height / 2.0,
            Axis::Vertical => bounds.x + bounds.width / 2.0,
        };

        // --- Track
        let active_range = self.active_main_range(len, handle_width, handle_center);
        let inactive_ranges = self.inactive_main_ranges(len, handle_width, handle_center);

        if let Some(active) = active_range.valid() {
            fill_track_segment(
                renderer,
                self.axis,
                bounds,
                active,
                cross_center,
                track_height,
                self.size.corner_radius,
                if enabled {
                    style.active_track
                } else {
                    style.active_track_disabled
                },
            );
        }

        let inactive_color = if enabled {
            style.inactive_track
        } else {
            style.inactive_track_disabled
        };
        for inactive in inactive_ranges.iter().filter_map(Segment::valid) {
            fill_track_segment(
                renderer,
                self.axis,
                bounds,
                inactive,
                cross_center,
                track_height,
                self.size.corner_radius,
                inactive_color,
            );
        }

        // --- Stop indicators
        if self.stops {
            let track = rect_at(
                self.axis,
                bounds,
                0.0..=len,
                cross_center - track_height / 2.0..=cross_center + track_height / 2.0,
            );
            let dot_color = |main: f32| -> Color {
                if !enabled {
                    style.stop_indicators_disabled
                } else if active_range
                    .valid()
                    .is_some_and(|active| active.contains(&main))
                {
                    style.stop_indicators_active
                } else {
                    style.stop_indicators_inactive
                }
            };

            // Continuous ticks are bounded by the available track length.
            if let Some(count) = self.continuous_tick_count.filter(|count| *count > 1) {
                let stride = (count.saturating_sub(1) as f32 / (len / 8.0).floor().max(1.0))
                    .ceil()
                    .max(1.0) as usize;
                for i in (1..count).step_by(stride) {
                    let fraction = f64::from(i) / f64::from(count);
                    let main = len * fraction as f32;
                    draw_stop_indicator(
                        renderer,
                        self.axis,
                        bounds,
                        cross_center,
                        main,
                        dot_color(main),
                        track,
                    );
                }
            }

            // End stop indicators on the inactive track edges
            for inactive in inactive_ranges.iter().filter_map(Segment::valid) {
                if inactive.start <= 0.0 {
                    // Segment touching the start of the track
                    if inactive.end - inactive.start > STOP_INDICATOR_MIN_SEGMENT {
                        let main = STOP_INDICATOR_TRAILING_SPACE + STOP_INDICATOR_SIZE / 2.0;
                        draw_stop_indicator(
                            renderer,
                            self.axis,
                            bounds,
                            cross_center,
                            main,
                            if enabled {
                                style.stop_indicator
                            } else {
                                style.stop_indicators_disabled
                            },
                            track,
                        );
                    }
                }
                if inactive.end >= len {
                    // Segment touching the end of the track
                    if inactive.end - inactive.start > STOP_INDICATOR_MIN_SEGMENT {
                        let main = len - STOP_INDICATOR_TRAILING_SPACE - STOP_INDICATOR_SIZE / 2.0;
                        draw_stop_indicator(
                            renderer,
                            self.axis,
                            bounds,
                            cross_center,
                            main,
                            if enabled {
                                style.stop_indicator
                            } else {
                                style.stop_indicators_disabled
                            },
                            track,
                        );
                    }
                }
            }
        }

        // --- Inset icons
        if self.inset_icon_start.is_some() || self.inset_icon_end.is_some() {
            let icon_size = inset_icon_size(track_height);
            let icon_color = |active: bool| -> Color {
                match (active, enabled) {
                    (true, true) => style.icon_active,
                    (false, true) => style.icon_inactive,
                    (_, false) => style.icon_disabled,
                }
            };

            if let Some(icon) = &self.inset_icon_start {
                let center = track_height / 2.0;
                draw_inset_icon(
                    renderer,
                    self.axis,
                    bounds,
                    cross_center,
                    center,
                    icon_size,
                    &icon.on_active,
                    &icon.on_inactive,
                    icon_color(true),
                    icon_color(false),
                    state.icon_start_visibility_spring.value(),
                    viewport,
                );
            }
            if let Some(icon) = &self.inset_icon_end {
                let center = len - track_height / 2.0;
                draw_inset_icon(
                    renderer,
                    self.axis,
                    bounds,
                    cross_center,
                    center,
                    icon_size,
                    &icon.on_active,
                    &icon.on_inactive,
                    icon_color(true),
                    icon_color(false),
                    state.icon_end_visibility_spring.value(),
                    viewport,
                );
            }
        }

        // --- Handle
        let handle = handle_rect(
            self.axis,
            bounds,
            self.size.handle_height,
            handle_width,
            handle_center,
        );
        renderer.fill_quad(
            Quad {
                bounds: handle,
                border: Border::default().rounded(f32::MAX),
                ..Quad::default()
            },
            if enabled {
                style.handle
            } else {
                style.handle_disabled
            },
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: layout::Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &iced::Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        let state = tree.state.downcast_ref::<State<Renderer::Paragraph>>();

        if self.on_change.is_none() {
            return mouse::Interaction::default();
        }

        if state.is_dragging {
            // FIXME: Fall back to `Pointer` on Windows
            // See https://github.com/rust-windowing/winit/issues/1043
            if cfg!(target_os = "windows") {
                mouse::Interaction::Pointer
            } else {
                mouse::Interaction::Grabbing
            }
        } else if cursor.is_over(layout.bounds()) {
            if cfg!(target_os = "windows") {
                mouse::Interaction::Pointer
            } else {
                mouse::Interaction::Grab
            }
        } else {
            mouse::Interaction::default()
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: layout::Layout<'b>,
        _renderer: &Renderer,
        _viewport: &iced::Rectangle,
        translation: iced::Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        let state = tree.state.downcast_ref::<State<Renderer::Paragraph>>();
        let enabled = self.on_change.is_some();
        // The overlay draws in window coordinates: shift the widget bounds by
        // the translation of the enclosing scrollable, if any
        let widget_bounds = layout.bounds();
        let bounds = Rectangle {
            x: widget_bounds.x + translation.x,
            y: widget_bounds.y + translation.y,
            ..widget_bounds
        };

        let len = main_len(self.axis, bounds);
        let handle_width = state.handle_width_spring.value();
        let handle_center = center_at(len, handle_width, state.handle_position_spring.value());
        let circle = handle_circle(self.axis, bounds, self.size.handle_height, handle_center);

        let style = state.style_spring.value();
        let focus = state.focus_ring_spring.value();
        let label_scale = state.label_scale_spring.value();

        let ripple_visible = state.ripple.is_visible(circle.size(), enabled);

        if !ripple_visible && focus <= 0.01 && label_scale <= 0.01 {
            return None;
        }

        let focus_ring = (focus > 0.01 && enabled && state.keyboard_focus).then(|| {
            let size = focus_ring_size(self.size.handle_height) * focus;
            let full = focus_ring_size(self.size.handle_height);
            (
                Rectangle {
                    x: circle.x + (circle.width - full) / 2.0 + (full - size) / 2.0,
                    y: circle.y + (circle.height - full) / 2.0 + (full - size) / 2.0,
                    width: size,
                    height: size,
                },
                Color {
                    a: style.focus_ring.a * focus,
                    ..style.focus_ring
                },
            )
        });

        let handle = handle_rect(
            self.axis,
            bounds,
            self.size.handle_height,
            handle_width,
            handle_center,
        );
        let label = (label_scale > 0.01).then(|| ValueLabel {
            axis: self.axis,
            anchor: label_anchor(self.axis, &handle),
            width: state.label_paragraph.min_width() + 2.0 * LABEL_HORIZONTAL_PADDING,
            height: LABEL_MIN_HEIGHT,
            scale: label_scale,
            background: style.label_container,
            text_color: style.label_text,
            content: state.label_content.clone(),
        });

        let overlay_element = SliderOverlay {
            ripples: [
                Some(RippleLayer {
                    ripple: state.ripple,
                    layer: style.state_layer,
                    bounds: circle,
                    enabled,
                }),
                None,
            ],
            focus_ring,
            label,
        };

        Some(overlay::Element::new(Box::new(overlay_element)))
    }
}

impl<'a, T, Message, Theme, Renderer> From<Slider<'a, T, Message>>
    for Element<'a, Message, Theme, Renderer>
where
    T: Copy + Display + Into<f64> + num_traits::FromPrimitive + PartialOrd + 'a,
    Message: Clone + 'a,
    Renderer: iced::advanced::text::Renderer
        + iced::advanced::svg::Renderer
        + iced_widget::renderer::geometry::Renderer
        + 'a,
    Renderer::Font: From<iced::Font>,
{
    fn from(slider: Slider<'a, T, Message>) -> Element<'a, Message, Theme, Renderer> {
        Element::new(slider)
    }
}

// --- Geometry helpers

/// A segment of the track along the main axis, in pixels from the start of
/// the track. Both ends remember whether they sit on an edge of the track, in
/// which case they get the outer corner radius instead of the inner one.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Segment {
    pub start: f32,
    pub end: f32,
    start_at_edge: bool,
    end_at_edge: bool,
}

impl Segment {
    fn new(start: f32, end: f32, start_at_edge: bool, end_at_edge: bool) -> Self {
        Self {
            start,
            end,
            start_at_edge,
            end_at_edge,
        }
    }

    /// The segment, if it is long enough to be visible.
    fn valid(&self) -> Option<Self> {
        (self.end - self.start > 0.5).then_some(*self)
    }

    fn contains(&self, main: &f32) -> bool {
        *main >= self.start && *main <= self.end
    }

    fn radii(&self, axis: Axis, corner_radius: f32) -> Radius {
        // `start` is the radius at the main-start end of the segment (left for
        // a horizontal slider, bottom for a vertical one), `end` at the
        // main-end (right, top).
        let (start, end) = (
            if self.start_at_edge {
                corner_radius
            } else {
                INNER_CORNER_RADIUS
            },
            if self.end_at_edge {
                corner_radius
            } else {
                INNER_CORNER_RADIUS
            },
        );

        match axis {
            Axis::Horizontal => Radius {
                top_left: start,
                bottom_left: start,
                top_right: end,
                bottom_right: end,
            },
            Axis::Vertical => Radius {
                bottom_left: start,
                bottom_right: start,
                top_left: end,
                top_right: end,
            },
        }
    }
}

impl<T, Message> Slider<'_, T, Message>
where
    T: Copy + Into<f64> + PartialOrd,
{
    /// The main-axis segment covered by the active track, in pixels from the
    /// start of the track.
    pub(crate) fn active_main_range(
        &self,
        len: f32,
        handle_width: f32,
        handle_center: f32,
    ) -> Segment {
        if self.centered {
            // The active track grows from the middle of the range towards the
            // handle, in either direction.
            let mid = center_at(len, handle_width, 0.5);
            let (start, end) = if handle_center < mid {
                (handle_center, mid)
            } else {
                (mid, handle_center)
            };
            Segment::new(start, end, false, false)
        } else {
            Segment::new(
                0.0,
                handle_center - handle_width / 2.0 - HANDLE_GAP,
                true,
                false,
            )
        }
    }

    /// The main-axis segments covered by the inactive track. For a centered
    /// slider both sides of the middle can be inactive; otherwise only the
    /// part past the handle is. Invalid segments are filtered out by
    /// [`Segment::valid`].
    pub(crate) fn inactive_main_ranges(
        &self,
        len: f32,
        handle_width: f32,
        handle_center: f32,
    ) -> [Segment; 2] {
        if self.centered {
            let mid = center_at(len, handle_width, 0.5);
            [
                Segment::new(0.0, mid.min(handle_center), true, false),
                Segment::new(mid.max(handle_center), len, false, true),
            ]
        } else {
            [
                Segment::new(0.0, 0.0, false, false),
                Segment::new(
                    handle_center + handle_width / 2.0 + HANDLE_GAP,
                    len,
                    false,
                    true,
                ),
            ]
        }
    }
}

/// The length of the widget along its main axis.
fn main_len(axis: Axis, bounds: Rectangle) -> f32 {
    match axis {
        Axis::Horizontal => bounds.width,
        Axis::Vertical => bounds.height,
    }
}

/// The handle center, in pixels from the start of the track, for a value
/// factor in `0..=1`.
fn center_at(len: f32, handle_width: f32, factor: f32) -> f32 {
    let len = len.max(handle_width);
    handle_width / 2.0 + (len - handle_width) * factor.clamp(0.0, 1.0)
}

/// The value factor in `0..=1` for a handle center in pixels from the start
/// of the track.
fn factor_at(len: f32, handle_width: f32, center: f32) -> f32 {
    let len = len.max(handle_width + 1.0);
    ((center - handle_width / 2.0) / (len - handle_width)).clamp(0.0, 1.0)
}

/// The main-axis coordinate of a pointer position, with the zero of a
/// vertical slider at the bottom.
fn cursor_main(axis: Axis, bounds: Rectangle, position: Point) -> f32 {
    match axis {
        Axis::Horizontal => position.x - bounds.x,
        Axis::Vertical => bounds.y + bounds.height - position.y,
    }
}

/// A screen-space rectangle for a main-axis range and a cross-axis range.
fn rect_at(
    axis: Axis,
    bounds: Rectangle,
    main: RangeInclusive<f32>,
    cross: RangeInclusive<f32>,
) -> Rectangle {
    let (main_start, main_end) = main.into_inner();
    let (cross_start, cross_end) = cross.into_inner();

    match axis {
        Axis::Horizontal => Rectangle {
            x: bounds.x + main_start,
            width: main_end - main_start,
            y: cross_start,
            height: cross_end - cross_start,
        },
        Axis::Vertical => Rectangle {
            y: bounds.y + bounds.height - main_end,
            height: main_end - main_start,
            x: cross_start,
            width: cross_end - cross_start,
        },
    }
}

/// A huge rectangle covering the whole screen area around `bounds`.
pub(crate) fn huge_bounds(bounds: Rectangle) -> Rectangle {
    Rectangle {
        x: bounds.x - 1.0e6,
        y: bounds.y - 1.0e6,
        width: bounds.width + 2.0e6,
        height: bounds.height + 2.0e6,
    }
}

/// The screen-space rectangle of a handle.
pub(crate) fn handle_rect(
    axis: Axis,
    bounds: Rectangle,
    handle_height: f32,
    handle_width: f32,
    handle_center: f32,
) -> Rectangle {
    let cross_center = match axis {
        Axis::Horizontal => bounds.y + bounds.height / 2.0,
        Axis::Vertical => bounds.x + bounds.width / 2.0,
    };

    rect_at(
        axis,
        bounds,
        handle_center - handle_width / 2.0..=handle_center + handle_width / 2.0,
        cross_center - handle_height / 2.0..=cross_center + handle_height / 2.0,
    )
}

/// The state layer circle centered on the handle.
fn handle_circle(
    axis: Axis,
    bounds: Rectangle,
    handle_height: f32,
    handle_center: f32,
) -> Rectangle {
    let diameter = state_circle_size(handle_height);
    let cross_center = match axis {
        Axis::Horizontal => bounds.y + bounds.height / 2.0,
        Axis::Vertical => bounds.x + bounds.width / 2.0,
    };

    rect_at(
        axis,
        bounds,
        handle_center - diameter / 2.0..=handle_center + diameter / 2.0,
        cross_center - diameter / 2.0..=cross_center + diameter / 2.0,
    )
}

fn state_circle_size(handle_height: f32) -> f32 {
    handle_height.max(STATE_LAYER_MIN_SIZE)
}

fn focus_ring_size(handle_height: f32) -> f32 {
    handle_height.max(FOCUS_RING_MIN_SIZE)
}

fn inset_icon_size(track_height: f32) -> f32 {
    (track_height / 2.0).clamp(16.0, 24.0)
}

/// The anchor of the value indicator pill: its bottom center for a horizontal
/// slider, the center of its left edge for a vertical one.
fn label_anchor(axis: Axis, handle_rect: &Rectangle) -> Point {
    match axis {
        Axis::Horizontal => Point::new(
            handle_rect.x + handle_rect.width / 2.0,
            handle_rect.y - LABEL_GAP,
        ),
        Axis::Vertical => Point::new(
            handle_rect.x + handle_rect.width + LABEL_GAP,
            handle_rect.y + handle_rect.height / 2.0,
        ),
    }
}

// --- Value helpers

fn range_f64<T>(range: &RangeInclusive<T>) -> std::ops::Range<f64>
where
    T: Copy + Into<f64>,
{
    let (start, end) = range.clone().into_inner();
    start.into()..end.into()
}

fn fraction_of(value: f64, range: &std::ops::Range<f64>) -> f32 {
    let span = range.end - range.start;
    if span <= 0.0 {
        0.0
    } else {
        ((value - range.start) / span).clamp(0.0, 1.0) as f32
    }
}

/// The value factor in `0..=1` of a value.
fn value_factor<T>(value: T, range: &RangeInclusive<T>) -> f32
where
    T: Copy + Into<f64>,
{
    fraction_of(value.into(), &range_f64(range))
}

/// Snaps a value to the closest multiple of the step.
fn snap_to_step(value: f64, range: &std::ops::Range<f64>, step: f64) -> f64 {
    let steps = ((value - range.start) / step).round();
    (range.start + steps * step).clamp(range.start, range.end)
}

/// The step used to locate the pointer, in value units: the configured step,
/// or zero for a continuous slider.
fn effective_step<T>(step: T) -> f64
where
    T: Copy + Into<f64>,
{
    let step = step.into();
    if step > 0.0 { step } else { 0.0 }
}

/// Keeps discrete tick marks far enough apart to avoid excessive drawing on
/// long ranges while retaining every step when the track has room for them.

/// The unsnapped pointer position along the slider's value axis.
fn pointer_factor(axis: Axis, bounds: Rectangle, position: Point) -> f32 {
    factor_at(
        main_len(axis, bounds),
        HANDLE_WIDTH_IDLE,
        cursor_main(axis, bounds, position),
    )
}

/// The step used by the keyboard and wheel: the configured step, or a
/// fraction of the range for continuous sliders.
fn effective_keyboard_step(step: f64, range: &std::ops::Range<f64>) -> f64 {
    if step > 0.0 {
        step
    } else {
        (range.end - range.start) / 20.0
    }
}

/// The step used by the keyboard and wheel while shift is held, matching the
/// Material "larger interval" behavior.
fn keyboard_step<T>(
    step: T,
    shift_step: Option<T>,
    shift: bool,
    range: &std::ops::Range<f64>,
) -> f64
where
    T: Copy + Into<f64>,
{
    if !shift {
        return effective_keyboard_step(step.into(), range);
    }

    match shift_step {
        Some(shift_step) => {
            let shift_step = shift_step.into();
            if shift_step > 0.0 {
                shift_step
            } else {
                effective_keyboard_step(step.into(), range)
            }
        }
        None => effective_keyboard_step(step.into(), range) * 10.0,
    }
}

fn increment(value: f64, range: &std::ops::Range<f64>, step: f64) -> f64 {
    let steps = ((value - range.start) / step).round();
    (range.start + (steps + 1.0) * step).min(range.end)
}

fn decrement(value: f64, range: &std::ops::Range<f64>, step: f64) -> f64 {
    let steps = ((value - range.start) / step).round();
    (range.start + (steps - 1.0) * step).max(range.start)
}

/// Locates the value under a pointer position.
fn value_at<T>(
    axis: Axis,
    bounds: Rectangle,
    position: Point,
    range: &RangeInclusive<T>,
    step: f64,
) -> Option<T>
where
    T: Copy + Into<f64> + num_traits::FromPrimitive,
{
    let full_range = range_f64(range);
    let factor = factor_at(
        main_len(axis, bounds),
        HANDLE_WIDTH_IDLE,
        cursor_main(axis, bounds, position),
    ) as f64;
    let value = full_range.start + factor * (full_range.end - full_range.start);

    let value = if step > 0.0 {
        snap_to_step(value, &full_range, step)
    } else {
        value.clamp(full_range.start, full_range.end)
    };

    T::from_f64(value)
}

fn label_text<C, F>(font: F, content: C) -> Text<C, F> {
    Text {
        font,
        line_height: text::LineHeight::default(),
        content,
        bounds: iced::Size::new(f32::INFINITY, LABEL_MIN_HEIGHT),
        size: LABEL_TEXT_SIZE.into(),
        align_x: text::Alignment::Center,
        align_y: alignment::Vertical::Center,
        shaping: text::Shaping::Advanced,
        wrapping: text::Wrapping::Word,
    }
}

// --- Shared drawing helpers

fn fill_track_segment<R>(
    renderer: &mut R,
    axis: Axis,
    bounds: Rectangle,
    segment: Segment,
    cross_center: f32,
    track_height: f32,
    corner_radius: f32,
    color: Color,
) where
    R: iced_widget::renderer::geometry::Renderer + ?Sized,
{
    let rect = rect_at(
        axis,
        bounds,
        segment.start..=segment.end,
        cross_center - track_height / 2.0..=cross_center + track_height / 2.0,
    );

    renderer.fill_quad(
        Quad {
            bounds: rect,
            border: Border::default().rounded(segment.radii(axis, corner_radius)),
            ..Quad::default()
        },
        color,
    );
}

fn draw_stop_indicator<R>(
    renderer: &mut R,
    axis: Axis,
    bounds: Rectangle,
    cross_center: f32,
    main: f32,
    color: Color,
    track: Rectangle,
) where
    R: iced_widget::renderer::geometry::Renderer + ?Sized,
{
    let rect = rect_at(
        axis,
        bounds,
        main - STOP_INDICATOR_SIZE / 2.0..=main + STOP_INDICATOR_SIZE / 2.0,
        cross_center - STOP_INDICATOR_SIZE / 2.0..=cross_center + STOP_INDICATOR_SIZE / 2.0,
    );

    // Keep the indicator inside the rounded track
    renderer.with_layer(track, |renderer| {
        renderer.fill_quad(
            Quad {
                bounds: rect,
                border: Border::default().rounded(f32::MAX),
                ..Quad::default()
            },
            color,
        );
    });
}

fn draw_inset_icon<R>(
    renderer: &mut R,
    axis: Axis,
    bounds: Rectangle,
    cross_center: f32,
    main: f32,
    size: f32,
    on_active: &Icon<'_>,
    on_inactive: &Icon<'_>,
    active_color: Color,
    inactive_color: Color,
    visibility: f32,
    viewport: &iced::Rectangle,
) where
    R: iced::advanced::text::Renderer
        + iced::advanced::svg::Renderer
        + iced_widget::renderer::geometry::Renderer,
    R::Font: From<iced::Font>,
{
    let rect = rect_at(
        axis,
        bounds,
        main - size / 2.0..=main + size / 2.0,
        cross_center - size / 2.0..=cross_center + size / 2.0,
    );

    let crossfade = |renderer: &mut R, icon: &Icon<'_>, color: Color, alpha: f32| match icon {
        Icon::Text { text, font } => {
            let font = font
                .map(R::Font::from)
                .unwrap_or_else(|| renderer.default_font());

            renderer.fill_text(
                Text {
                    font,
                    line_height: text::LineHeight::default(),
                    content: text.to_string(),
                    bounds: rect.size(),
                    size: size.into(),
                    align_x: text::Alignment::Center,
                    align_y: alignment::Vertical::Center,
                    shaping: text::Shaping::Advanced,
                    wrapping: text::Wrapping::Word,
                },
                rect.position(),
                Color {
                    a: color.a * alpha,
                    ..color
                },
                *viewport,
            );
        }
        Icon::Svg(handle) => {
            renderer.draw_svg(
                Svg::new(handle.clone()).color(color).opacity(alpha),
                rect,
                *viewport,
            );
        }
    };

    crossfade(renderer, on_inactive, inactive_color, 1.0 - visibility);
    crossfade(renderer, on_active, active_color, visibility);
}

// --- Overlay

/// The value indicator pill shown next to the handle of a slider.
pub(crate) struct ValueLabel {
    axis: Axis,
    /// The bottom center of the pill for a horizontal slider, the center of
    /// its left edge for a vertical one.
    anchor: Point,
    width: f32,
    height: f32,
    scale: f32,
    background: Color,
    text_color: Color,
    content: String,
}

impl ValueLabel {
    /// The on-screen rectangle of the pill at its current scale.
    fn pill_rect(&self) -> Rectangle {
        let width = self.width * self.scale;
        let height = self.height * self.scale;

        match self.axis {
            Axis::Horizontal => Rectangle {
                x: self.anchor.x - width / 2.0,
                y: self.anchor.y - height,
                width,
                height,
            },
            Axis::Vertical => Rectangle {
                x: self.anchor.x,
                y: self.anchor.y - height / 2.0,
                width,
                height,
            },
        }
    }
}

/// The state layer, focus ring and value indicator of a slider, drawn as an
/// overlay so they can grow past the bounds of the widget.
pub(crate) struct RippleLayer {
    pub ripple: Ripple,
    pub layer: StateLayer,
    pub bounds: Rectangle,
    pub enabled: bool,
}

/// The ripples, focus ring and value indicator of a slider, drawn as an
/// overlay so they can grow past the bounds of the widget.
///
/// A single-handle slider has one ripple layer, a range slider two.
pub(crate) struct SliderOverlay {
    pub ripples: [Option<RippleLayer>; 2],
    pub focus_ring: Option<(Rectangle, Color)>,
    pub label: Option<ValueLabel>,
}

impl SliderOverlay {
    /// The smallest rectangle covering everything the overlay draws. The
    /// overlay node is positioned and sized by it, and clipped to it.
    fn content_bounds(&self) -> Rectangle {
        self.ripples
            .iter()
            .flatten()
            .map(|layer| layer.bounds)
            .chain(self.focus_ring.map(|(bounds, _)| bounds))
            .chain(self.label.as_ref().map(ValueLabel::pill_rect))
            .reduce(|a, b| a.union(&b))
            .unwrap_or(Rectangle::default())
    }
}

impl<Message, Theme, Renderer> Overlay<Message, Theme, Renderer> for SliderOverlay
where
    Renderer: iced::advanced::text::Renderer + iced_widget::renderer::geometry::Renderer,
{
    fn layout(
        &mut self,
        _renderer: &Renderer,
        _bounds: iced::Size,
    ) -> iced::advanced::layout::Node {
        let bounds = self.content_bounds();

        layout::Node::new(bounds.size()).move_to(bounds.position())
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &iced::advanced::renderer::Style,
        _layout: layout::Layout<'_>,
        _cursor: mouse::Cursor,
    ) {
        for layer in self.ripples.iter().flatten() {
            layer.ripple.draw(
                renderer,
                layer.bounds,
                Radius::from(f32::MAX),
                layer.layer,
                layer.enabled,
            );
        }

        if let Some((bounds, color)) = self.focus_ring {
            renderer.fill_quad(
                Quad {
                    bounds,
                    border: Border::default()
                        .rounded(f32::MAX)
                        .width(FOCUS_RING_WIDTH)
                        .color(color),
                    ..Quad::default()
                },
                Color::TRANSPARENT,
            );
        }

        if let Some(label) = &self.label {
            let pill = label.pill_rect();

            renderer.fill_quad(
                Quad {
                    bounds: pill,
                    border: Border::default().rounded(Radius::from(f32::MAX)),
                    ..Quad::default()
                },
                label.background,
            );

            let text_alpha = ((label.scale - LABEL_TEXT_FADE_START)
                / (1.0 - LABEL_TEXT_FADE_START))
                .clamp(0.0, 1.0);
            if text_alpha > 0.0 {
                renderer.fill_text(
                    Text {
                        font: renderer.default_font(),
                        line_height: text::LineHeight::default(),
                        content: label.content.clone(),
                        bounds: pill.size(),
                        size: LABEL_TEXT_SIZE.into(),
                        align_x: text::Alignment::Center,
                        align_y: alignment::Vertical::Center,
                        shaping: text::Shaping::Advanced,
                        wrapping: text::Wrapping::Word,
                    },
                    pill.center(),
                    Color {
                        a: label.text_color.a * text_alpha,
                        ..label.text_color
                    },
                    HUGE_VIEWPORT,
                );
            }
        }
    }
}
