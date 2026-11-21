use std::{fmt::Display, mem, ops::RangeInclusive, time::Instant};

use iced::{
    Border, Color, Element, Event, Length, Point, Rectangle,
    advanced::{
        Widget, layout, mouse, overlay,
        renderer::Quad,
        widget::{Id, Tree, operation},
    },
    keyboard::{self, Key, key},
    touch,
};
use iced_widget::core::text::{self, paragraph};

use super::{
    Axis, DRAG_CHASE_DECAY, HANDLE_GAP, HANDLE_OVERLAP_WIDTH, HANDLE_WIDTH_DRAGGED,
    HANDLE_WIDTH_IDLE, LABEL_HORIZONTAL_PADDING, LABEL_MIN_HEIGHT, RippleLayer,
    STOP_INDICATOR_MIN_SEGMENT, STOP_INDICATOR_SIZE, STOP_INDICATOR_TRAILING_SPACE, Segment, Size,
    SliderOverlay, Style, ValueIndicator, ValueLabel, center_at, cursor_main, decrement,
    draw_stop_indicator, effective_step, factor_at, fill_track_segment, focus_ring_size,
    handle_circle, handle_rect, huge_bounds, increment, keyboard_step, label_anchor, label_text,
    main_len, range_f64, rect_at, slider_layout, slider_mouse_interaction, slider_size,
    step_decimal_places, value_at, value_factor,
};
use crate::{
    animation::motion::{self, Spring, SpringValue, fast_effects, fast_spatial},
    widget::ripple::Ripple,
};

/// Which handle of a [`RangeSlider`] is interacted with.
///
/// The handles are identified by position: the lower one sits at the smaller
/// value. When the handles pass each other, they swap identities and the
/// dragged one keeps following the pointer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Thumb {
    /// The handle at the lower value.
    Lower,
    /// The handle at the upper value.
    Upper,
}

/// The internal state of a [`RangeSlider`].
struct State<P: text::Paragraph> {
    /// The dragged thumb, and whether it tracks the pointer directly yet:
    /// `None` from the press until the first cursor move, so that the handle
    /// glides to the pressed position instead of teleporting.
    drag: Option<(Thumb, Option<f32>)>,
    /// The place the dragged handle chases, trailing the pointer by a small
    /// delay. The chase mirrors itself into the chased handle's position
    /// spring, which stays the single source the track fill and the label
    /// follow.
    drag_display: f32,
    active: Thumb,
    keyboard_modifiers: keyboard::Modifiers,
    is_focused: bool,
    /// Whether the focus was granted by the keyboard: the focus ring and the
    /// resting value indicator are shown for keyboard focus only, pointer
    /// focus just enables the keyboard shortcuts.
    keyboard_focus: bool,
    ripples: [Ripple; 2],
    lower_width_spring: SpringValue<f32>,
    upper_width_spring: SpringValue<f32>,
    lower_position_spring: SpringValue<f32>,
    upper_position_spring: SpringValue<f32>,
    /// The remaining travel of the value indicator towards the active
    /// handle, after the active handle has changed.
    label_anchor_offset_spring: SpringValue<f32>,
    label_scale_spring: SpringValue<f32>,
    focus_ring_spring: SpringValue<f32>,
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
        self.style_spring.spring.model = effects;
    }

    fn set_spatial_spring(&mut self, spatial: Spring) {
        self.lower_width_spring.spring.model = spatial;
        self.upper_width_spring.spring.model = spatial;
        self.lower_position_spring.spring.model = spatial;
        self.upper_position_spring.spring.model = spatial;
        self.label_anchor_offset_spring.spring.model = spatial;
    }

    fn position_spring(&self, thumb: Thumb) -> f32 {
        match thumb {
            Thumb::Lower => self.lower_position_spring.value(),
            Thumb::Upper => self.upper_position_spring.value(),
        }
    }

    fn position_spring_mut(&mut self, thumb: Thumb) -> &mut SpringValue<f32> {
        match thumb {
            Thumb::Lower => &mut self.lower_position_spring,
            Thumb::Upper => &mut self.upper_position_spring,
        }
    }

    fn width_spring(&self, thumb: Thumb) -> f32 {
        match thumb {
            Thumb::Lower => self.lower_width_spring.value(),
            Thumb::Upper => self.upper_width_spring.value(),
        }
    }

    fn is_animating(&self) -> bool {
        !self.lower_width_spring.is_at_rest()
            || !self.upper_width_spring.is_at_rest()
            || !self.lower_position_spring.is_at_rest()
            || !self.upper_position_spring.is_at_rest()
            || !self.label_anchor_offset_spring.is_at_rest()
            || !self.label_scale_spring.is_at_rest()
            || !self.focus_ring_spring.is_at_rest()
            || !self.style_spring.is_at_rest()
    }

    fn step(&mut self, now: Instant, continuous_drag: bool) {
        // In continuous mode the drag chases the pointer: the chase drives
        // the dragged handle's position spring through itself, resetting it
        // to the chased place every frame so the handle trails the pointer
        // by a fixed, frame-rate-independent delay.
        // In discrete mode the spring is allowed to step normally so the
        // handle animates between snapped positions.
        if continuous_drag {
            if let Some((thumb, Some(target))) = self.drag {
                let spring = match thumb {
                    Thumb::Lower => &mut self.lower_position_spring,
                    Thumb::Upper => &mut self.upper_position_spring,
                };
                let delta = now.duration_since(spring.spring.time).as_secs_f32();
                let decay = 1.0 - (-delta / DRAG_CHASE_DECAY).exp();
                self.drag_display += (target - self.drag_display) * decay;
                spring.reset(self.drag_display, now);
            }
        }

        self.lower_width_spring.step(now);
        self.upper_width_spring.step(now);
        self.lower_position_spring.step(now);
        self.upper_position_spring.step(now);
        self.label_anchor_offset_spring.step(now);
        self.label_scale_spring.step(now);
        self.focus_ring_spring.step(now);
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

/// A slider with two handles.
///
/// Range sliders react to pointer, touch and keyboard interaction: clicking
/// the track moves the nearest handle there and starts a drag, the arrow keys
/// move the active handle by one step while the slider is focused, and the
/// Home, End, PageUp and PageDown keys jump to well-known values. While
/// dragged, the handle trails the pointer slightly and settles with a soft
/// wobble when released.
///
/// Either handle can be disabled, acting as a fixed bound for the other one;
/// see [`RangeSlider::lower_enabled`] and [`RangeSlider::upper_enabled`].
///
/// The values snap to whole numbers by default; see [`RangeSlider::step`] and
/// [`RangeSlider::decimal_places`].
pub struct RangeSlider<'a, T, Message> {
    id: Option<Id>,
    range: RangeInclusive<T>,
    step: T,
    shift_step: Option<T>,
    lower: T,
    upper: T,
    min_separation: Option<T>,
    lower_enabled: bool,
    upper_enabled: bool,
    on_change: Option<Box<dyn Fn(T, T) -> Message + 'a>>,
    on_release: Option<Message>,
    width: Length,
    height: Length,
    style: Style,
    size: Size,
    axis: Axis,
    stops: bool,
    continuous_tick_count: Option<u32>,
    value_indicator: ValueIndicator<'a, T>,
    decimal_places: usize,
    effects_spring: Option<Spring>,
    spatial_spring: Option<Spring>,
}

impl<'a, T, Message> RangeSlider<'a, T, Message>
where
    T: Copy + From<u8> + PartialOrd,
{
    #[must_use]
    pub fn new<F>(range: RangeInclusive<T>, start: T, end: T, on_change: F, style: Style) -> Self
    where
        F: 'a + Fn(T, T) -> Message,
    {
        let (start, end) = if start <= end {
            (start, end)
        } else {
            (end, start)
        };
        let clamp = |mut value: T| {
            if value < *range.start() {
                value = *range.start();
            }
            if value > *range.end() {
                value = *range.end();
            }
            value
        };
        let (lower, upper) = (clamp(start), clamp(end));
        let (lower, upper) = if lower <= upper {
            (lower, upper)
        } else {
            (upper, lower)
        };

        RangeSlider {
            id: None,
            range,
            // One by default, so the values snap to whole numbers and the
            // value indicator shows them without fractional noise
            step: T::from(1),
            shift_step: None,
            lower,
            upper,
            min_separation: None,
            lower_enabled: true,
            upper_enabled: true,
            on_change: Some(Box::new(on_change)),
            on_release: None,
            width: Length::Fill,
            height: Length::Fill,
            style,
            size: Size::default(),
            axis: Axis::default(),
            stops: true,
            continuous_tick_count: None,
            value_indicator: ValueIndicator::None,
            decimal_places: 1,
            effects_spring: None,
            spatial_spring: None,
        }
    }

    /// Sets the [`Id`] of the [`RangeSlider`], used by focus operations.
    #[must_use]
    pub fn id(mut self, id: impl Into<Id>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Sets the message that replaces `on_change` if `Some`, disabling the
    /// [`RangeSlider`] when `None`.
    #[must_use]
    pub fn on_change_maybe<F>(mut self, on_change: Option<F>) -> Self
    where
        F: 'a + Fn(T, T) -> Message,
    {
        self.on_change = on_change.map(|on_change| Box::new(on_change) as _);
        self
    }

    /// Sets the release message of the [`RangeSlider`].
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

    /// Sets the width of the [`RangeSlider`].
    #[must_use]
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of a vertical [`RangeSlider`].
    #[must_use]
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Sets the step size of the [`RangeSlider`].
    ///
    /// The values snap to multiples of the step. A step of zero makes the
    /// [`RangeSlider`] continuous.
    ///
    /// Defaults to 1.
    #[must_use]
    pub fn step(mut self, step: impl Into<T>) -> Self {
        self.step = step.into();
        self
    }

    /// Sets the optional "shift" step for the [`RangeSlider`].
    ///
    /// If set, this value is used as the step while the shift key is pressed.
    #[must_use]
    pub fn shift_step(mut self, shift_step: impl Into<T>) -> Self {
        self.shift_step = Some(shift_step.into());
        self
    }

    /// Sets the minimum separation between the two handles, in value units.
    ///
    /// If set, the handles cannot come closer to each other than this and
    /// cannot pass each other.
    #[must_use]
    pub fn min_separation(mut self, min_separation: impl Into<T>) -> Self {
        self.min_separation = Some(min_separation.into());
        self
    }

    /// Sets whether the lower handle is enabled.
    ///
    /// A disabled handle keeps its value and cannot be grabbed, hovered,
    /// moved or focused: it acts as a fixed bound that the other handle
    /// cannot pass.
    #[must_use]
    pub fn lower_enabled(mut self, enabled: bool) -> Self {
        self.lower_enabled = enabled;
        self
    }

    /// Sets whether the upper handle is enabled.
    ///
    /// A disabled handle keeps its value and cannot be grabbed, hovered,
    /// moved or focused: it acts as a fixed bound that the other handle
    /// cannot pass.
    #[must_use]
    pub fn upper_enabled(mut self, enabled: bool) -> Self {
        self.upper_enabled = enabled;
        self
    }

    /// Sets the size of the [`RangeSlider`].
    #[must_use]
    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }

    /// Sets the orientation of the [`RangeSlider`].
    #[must_use]
    pub fn axis(mut self, axis: Axis) -> Self {
        self.axis = axis;
        self
    }

    /// Makes the [`RangeSlider`] vertical.
    #[must_use]
    pub fn vertical(self) -> Self {
        self.axis(Axis::Vertical)
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

    /// Sets how the value indicator of the active handle is shown while it is
    /// dragged, hovered or focused.
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

    /// Sets the motion scheme used by the [`RangeSlider`].
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

    /// Sets the spring used for spatial motion, like the handles.
    #[must_use]
    pub fn spatial_spring(mut self, spring: Spring) -> Self {
        self.spatial_spring = Some(spring);
        self
    }
}

impl<T, Message> RangeSlider<'_, T, Message> {
    /// Whether a handle responds to interaction: the slider has an
    /// `on_change` handler and the handle is enabled.
    fn enabled_of(&self, thumb: Thumb) -> bool {
        self.on_change.is_some()
            && match thumb {
                Thumb::Lower => self.lower_enabled,
                Thumb::Upper => self.upper_enabled,
            }
    }

    /// Whether the slider as a whole responds to interaction: it has an
    /// `on_change` handler and at least one enabled handle.
    fn is_enabled(&self) -> bool {
        self.enabled_of(Thumb::Lower) || self.enabled_of(Thumb::Upper)
    }
}

impl<T, Message> RangeSlider<'_, T, Message>
where
    T: Copy + Display + PartialOrd,
{
    fn value_of(&self, thumb: Thumb) -> T {
        match thumb {
            Thumb::Lower => self.lower,
            Thumb::Upper => self.upper,
        }
    }

    fn format_value(&self, thumb: Thumb) -> Option<String>
    where
        T: Into<f64>,
    {
        let step_f64: f64 = self.step.into();
        let min_decimal_places = step_decimal_places(step_f64);
        let decimal_places = self.decimal_places.max(min_decimal_places);
        self.value_indicator
            .format(self.value_of(thumb), decimal_places, min_decimal_places)
    }
}

impl<T, Message> RangeSlider<'_, T, Message>
where
    T: Copy + Into<f64> + PartialOrd + num_traits::FromPrimitive,
{
    /// Moves a handle to a value, keeping the pair sorted and separated.
    ///
    /// Returns the handle the value ended up on: when a handle crosses the
    /// other one, the pair flips and the moved handle changes identity. A
    /// disabled handle is an immovable bound: the other handle cannot pass
    /// it and it never moves itself.
    fn move_thumb(&mut self, thumb: Thumb, value: T) -> Thumb {
        if !self.enabled_of(thumb) {
            return thumb;
        }

        let lower: f64 = self.lower.into();
        let upper: f64 = self.upper.into();
        let value: f64 = value.into();
        let full_range = range_f64(&self.range);
        let separation = self
            .min_separation
            .map(|separation: T| separation.into())
            .unwrap_or(0.0)
            .min(full_range.end - full_range.start)
            .max(0.0);

        // Without a minimum separation the handles can pass each other, which
        // flips the moved handle to the other end of the pair. A disabled
        // handle cannot be passed, so the pair never flips over it.
        let (new_lower, new_upper, thumb) = match thumb {
            Thumb::Lower if separation <= 0.0 && self.upper_enabled && value >= upper => {
                (upper, value, Thumb::Upper)
            }
            Thumb::Lower => (
                value.min(upper - separation).max(full_range.start),
                upper,
                Thumb::Lower,
            ),
            Thumb::Upper if separation <= 0.0 && self.lower_enabled && value <= lower => {
                (value, lower, Thumb::Lower)
            }
            Thumb::Upper => (
                lower,
                value.max(lower + separation).min(full_range.end),
                Thumb::Upper,
            ),
        };

        self.lower = T::from_f64(new_lower).unwrap_or(self.lower);
        self.upper = T::from_f64(new_upper).unwrap_or(self.upper);
        thumb
    }
}

impl<T, Message, Theme, Renderer> Widget<Message, Theme, Renderer> for RangeSlider<'_, T, Message>
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
        let label_content = self.format_value(Thumb::Upper).unwrap_or_default();

        let state: State<Renderer::Paragraph> = State {
            drag: None,
            drag_display: value_factor(self.upper, &self.range),
            active: Thumb::Upper,
            keyboard_modifiers: keyboard::Modifiers::default(),
            is_focused: false,
            keyboard_focus: false,
            ripples: [Ripple::new(), Ripple::new()],
            lower_width_spring: SpringValue::new(HANDLE_WIDTH_IDLE, spatial_spring, now),
            upper_width_spring: SpringValue::new(HANDLE_WIDTH_IDLE, spatial_spring, now),
            lower_position_spring: SpringValue::new(
                value_factor(self.lower, &self.range),
                spatial_spring,
                now,
            ),
            upper_position_spring: SpringValue::new(
                value_factor(self.upper, &self.range),
                spatial_spring,
                now,
            ),
            label_anchor_offset_spring: SpringValue::new(0.0, spatial_spring, now),
            label_scale_spring: SpringValue::new(0.0, effects_spring, now),
            focus_ring_spring: SpringValue::new(0.0, effects_spring, now),
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
        if !self.is_enabled() {
            state.drag = None;
        }

        // A disabled handle cannot stay active or stay dragged
        for thumb in [Thumb::Lower, Thumb::Upper] {
            if !self.enabled_of(thumb) {
                if state.active == thumb {
                    state.active = match thumb {
                        Thumb::Lower => Thumb::Upper,
                        Thumb::Upper => Thumb::Lower,
                    };
                }
                if state.drag.is_some_and(|(dragged, _)| dragged == thumb) {
                    state.drag = None;
                }
            }
        }
    }

    fn size(&self) -> iced::Size<Length> {
        slider_size(self.axis, self.width, self.height, self.size.handle_height)
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> layout::Node {
        slider_layout(
            self.axis,
            self.width,
            self.height,
            self.size.handle_height,
            limits,
        )
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
        let enabled = self.is_enabled();
        let now = Instant::now();
        let len = main_len(self.axis, bounds);
        // Remember which handle was active at entry, so the value indicator
        // can travel when the active handle changes below.
        let active_at_entry = state.active;

        // --- Ripples on the handle state layers
        let lower_circle = handle_circle(
            self.axis,
            bounds,
            self.size.handle_height,
            center_at(
                len,
                state.width_spring(Thumb::Lower),
                state.position_spring(Thumb::Lower),
            ),
        );
        let upper_circle = handle_circle(
            self.axis,
            bounds,
            self.size.handle_height,
            center_at(
                len,
                state.width_spring(Thumb::Upper),
                state.position_spring(Thumb::Upper),
            ),
        );

        // While dragging, keep the state layer hovered even if the pointer
        // leaves the handle.
        let [lower_ripple, upper_ripple] = &mut state.ripples;
        for (ripple, circle, thumb, thumb_enabled) in [
            (
                lower_ripple,
                lower_circle,
                Thumb::Lower,
                self.enabled_of(Thumb::Lower),
            ),
            (
                upper_ripple,
                upper_circle,
                Thumb::Upper,
                self.enabled_of(Thumb::Upper),
            ),
        ] {
            let interaction_bounds = if state.drag.is_some_and(|(t, _)| t == thumb) {
                huge_bounds(bounds)
            } else {
                circle
            };
            ripple.handle_in(
                event,
                interaction_bounds,
                circle,
                cursor,
                thumb_enabled,
                shell,
            );
        }

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

        // --- Continuous vs discrete drag mode
        let axis = self.axis;
        let range = self.range.clone();
        let range_step = effective_step(self.step);
        let full_range = range_f64(&range);
        let pixels_per_step = if range_step > 0.0 {
            len as f64 * range_step / (full_range.end - full_range.start)
        } else {
            0.0
        };
        let continuous_drag = pixels_per_step < 12.0;

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

        let lower_target = value_factor(self.lower, &self.range);
        let upper_target = value_factor(self.upper, &self.range);
        // While dragging, `State::step` pins the chased handle's spring;
        // otherwise each handle glides to its value. In discrete mode
        // the target stays current so the spring steps normally.
        if !matches!(state.drag, Some((Thumb::Lower, Some(_)))) || !continuous_drag {
            state.lower_position_spring.set_target(lower_target, now);
        }
        if !matches!(state.drag, Some((Thumb::Upper, Some(_)))) || !continuous_drag {
            state.upper_position_spring.set_target(upper_target, now);
        }
        state.lower_width_spring.set_target(
            match state.drag {
                Some((Thumb::Lower, _)) => HANDLE_WIDTH_DRAGGED,
                _ => HANDLE_WIDTH_IDLE,
            },
            now,
        );
        state.upper_width_spring.set_target(
            match state.drag {
                Some((Thumb::Upper, _)) => HANDLE_WIDTH_DRAGGED,
                _ => HANDLE_WIDTH_IDLE,
            },
            now,
        );

        // Label visibility for the entry active handle (decides whether the
        // value indicator travels when the active handle changes).
        let active_circle_at_entry = match active_at_entry {
            Thumb::Lower => lower_circle,
            Thumb::Upper => upper_circle,
        };
        let cursor_over_active_at_entry = cursor.is_over(bounds)
            && self.enabled_of(active_at_entry)
            && cursor.is_over(active_circle_at_entry);
        let label_visible_at_entry = !matches!(self.value_indicator, ValueIndicator::None)
            && enabled
            && (state.drag.is_some() || cursor_over_active_at_entry || state.keyboard_focus);

        // Hovered handle becomes active
        if state.drag.is_none() && cursor.is_over(bounds) {
            let lower_hit = self.enabled_of(Thumb::Lower) && cursor.is_over(lower_circle);
            let upper_hit = self.enabled_of(Thumb::Upper) && cursor.is_over(upper_circle);

            match (lower_hit, upper_hit) {
                (true, false) => state.active = Thumb::Lower,
                (false, true) => state.active = Thumb::Upper,
                (true, true) => {
                    // Circles overlap: pick the nearer handle center.
                    let cursor_pos =
                        cursor_main(self.axis, bounds, cursor.position().unwrap_or_default());
                    let lower_center = center_at(
                        len,
                        state.width_spring(Thumb::Lower),
                        state.position_spring(Thumb::Lower),
                    );
                    let upper_center = center_at(
                        len,
                        state.width_spring(Thumb::Upper),
                        state.position_spring(Thumb::Upper),
                    );
                    state.active =
                        if (cursor_pos - lower_center).abs() <= (cursor_pos - upper_center).abs() {
                            Thumb::Lower
                        } else {
                            Thumb::Upper
                        };
                }
                (false, false) => {}
            }
        }

        state
            .label_scale_spring
            .set_target(label_visible_at_entry as u32 as f32, now);
        state
            .focus_ring_spring
            .set_target((state.keyboard_focus && enabled) as u32 as f32, now);

        // --- Value indicator content
        let label_content = self.format_value(state.active).unwrap_or_default();
        if state.label_content != label_content {
            state.label_content = label_content;
            // Re-measure the paragraph when label content changes.
            let _ = state.label_paragraph.update(label_text(
                renderer.default_font(),
                state.label_content.as_str(),
            ));
        }

        let drag_is_chasing = state.drag.is_some_and(|(_, target)| {
            target.is_some_and(|target| (target - state.drag_display).abs() > 0.001)
        });
        if state.is_animating() || drag_is_chasing {
            shell.request_redraw();
        }
        // step() must run every frame to keep springs animating.
        state.step(now, continuous_drag);

        // --- Interaction
        let locate = move |position: Point| -> Option<T> {
            value_at(axis, bounds, position, &range, range_step)
        };

        let publish =
            |slider: &Self, shell: &mut iced::advanced::Shell<'_, Message>, previous: (T, T)| {
                let changed = (previous.0.into() - slider.lower.into()).abs() > f64::EPSILON
                    || (previous.1.into() - slider.upper.into()).abs() > f64::EPSILON;
                if changed && let Some(on_change) = &slider.on_change {
                    shell.publish(on_change(slider.lower, slider.upper));
                }
            };

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                if enabled && let Some(position) = cursor.position_over(bounds) {
                    if !state.is_focused {
                        state.is_focused = true;
                        shell.request_redraw();
                    }
                    // Pointer focus enables the keyboard shortcuts, but hides
                    // the keyboard-only focus ring and resting value indicator
                    state.keyboard_focus = false;

                    // Grab the nearest enabled handle
                    let press_main = cursor_main(self.axis, bounds, position);
                    let lower_center = center_at(
                        len,
                        state.width_spring(Thumb::Lower),
                        state.position_spring(Thumb::Lower),
                    );
                    let upper_center = center_at(
                        len,
                        state.width_spring(Thumb::Upper),
                        state.position_spring(Thumb::Upper),
                    );
                    let thumb = if !self.enabled_of(Thumb::Lower) {
                        Thumb::Upper
                    } else if !self.enabled_of(Thumb::Upper)
                        || (press_main - lower_center).abs() <= (press_main - upper_center).abs()
                    {
                        Thumb::Lower
                    } else {
                        Thumb::Upper
                    };

                    if let Some(new_value) = locate(position) {
                        let previous = (self.lower, self.upper);
                        state.active = self.move_thumb(thumb, new_value);
                        // No drag factor yet: glide to pressed value instead of
                        // teleporting; tracking starts on first move.
                        state.drag = Some((state.active, None));
                        publish(self, shell, previous);
                    }

                    shell.capture_event();
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerLifted { .. })
            | Event::Touch(touch::Event::FingerLost { .. }) => {
                if let Some((thumb, chase)) = state.drag.take() {
                    let drag_display = state.drag_display;
                    if continuous_drag && range_step > 0.0 {
                        // Smooth fine-step mode: settle from continuous
                        // cursor position to the snapped model value.
                        let snapped_factor = value_factor(self.value_of(thumb), &self.range) as f32;
                        let spring = state.position_spring_mut(thumb);
                        if (snapped_factor - drag_display).abs() > f32::EPSILON {
                            spring.set_target(snapped_factor, now);
                            if let Some(target) = chase {
                                let gap = target - drag_display;
                                if gap.abs() > f32::EPSILON {
                                    spring.spring.velocity = 1.0 / DRAG_CHASE_DECAY;
                                }
                            }
                        }
                    } else if continuous_drag {
                        // Continuous slider without step: hand the chase
                        // over as-is.
                        if let Some(target) = chase {
                            let gap = target - drag_display;
                            if gap.abs() > f32::EPSILON {
                                let spring = state.position_spring_mut(thumb);
                                spring.set_target(target, now);
                                spring.spring.velocity = 1.0 / DRAG_CHASE_DECAY;
                            }
                        }
                    }
                    // Discrete mode: the spring is already animating toward
                    // the snapped value, nothing to do.

                    if let Some(on_release) = self.on_release.clone() {
                        shell.publish(on_release);
                    }
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { .. })
            | Event::Touch(touch::Event::FingerMoved { .. }) => {
                if let Some((thumb, _)) = state.drag
                    && enabled
                {
                    if let Some(position) = cursor.land().position()
                        && let Some(new_value) = locate(position)
                    {
                        let current_value = self.value_of(thumb);
                        let value_changed = (current_value.into() as f64 - new_value.into() as f64)
                            .abs()
                            > f64::EPSILON;

                        let previous = (self.lower, self.upper);
                        if value_changed {
                            state.active = self.move_thumb(thumb, new_value);
                            // The handles crossed and swapped identities: swap
                            // their springs too, so each circle keeps its own
                            // motion instead of jerking to the other one's place.
                            if state.active != thumb {
                                mem::swap(
                                    &mut state.lower_position_spring,
                                    &mut state.upper_position_spring,
                                );
                                mem::swap(
                                    &mut state.lower_width_spring,
                                    &mut state.upper_width_spring,
                                );
                            }
                            // Keep spring target in sync for discrete mode so the handle
                            // animates immediately instead of lagging.
                            if !continuous_drag {
                                let new_target =
                                    value_factor(self.value_of(state.active), &self.range) as f32;
                                state
                                    .position_spring_mut(state.active)
                                    .set_target(new_target, now);
                            }
                        }
                        // Chase continues from wherever the press glide brought the handle.
                        if state.drag.is_some_and(|(_, factor)| factor.is_none()) {
                            state.drag_display = state.position_spring(state.active);
                        }
                        // For pointer tracking, use the handle's current width so its
                        // visual center stays aligned with the cursor.
                        let current_width = state.width_spring(state.active);
                        let continuous_factor =
                            factor_at(len, current_width, cursor_main(axis, bounds, position));
                        let drag_factor = if continuous_drag {
                            continuous_factor
                        } else {
                            value_factor(new_value, &self.range) as f32
                        };
                        if !continuous_drag {
                            state.drag_display = drag_factor;
                        }
                        state.drag = Some((state.active, Some(drag_factor)));
                        if value_changed {
                            publish(self, shell, previous);
                        }
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

                    let current: f64 = self.value_of(state.active).into();
                    let new_value = if *delta < 0.0 {
                        decrement(current, &range, step)
                    } else {
                        increment(current, &range, step)
                    };

                    if let Some(new_value) = T::from_f64(new_value) {
                        let previous = (self.lower, self.upper);
                        state.active = self.move_thumb(state.active, new_value);
                        publish(self, shell, previous);
                    }

                    shell.capture_event();
                }
            }
            Event::Keyboard(keyboard::Event::KeyPressed { key, .. }) => {
                if state.is_focused && enabled {
                    let range = range_f64(&self.range);
                    let step = keyboard_step(
                        self.step,
                        self.shift_step,
                        state.keyboard_modifiers.shift(),
                        &range,
                    );
                    let current: f64 = self.value_of(state.active).into();
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
                        let previous = (self.lower, self.upper);
                        state.active = self.move_thumb(state.active, new_value);
                        publish(self, shell, previous);
                        shell.capture_event();
                    }
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

        // Recompute label visibility for the final active handle after all
        // events have been processed, so the target is correct for the
        // next frame and the travel check below uses the final state.
        let active_circle = match state.active {
            Thumb::Lower => lower_circle,
            Thumb::Upper => upper_circle,
        };
        let cursor_over_active = cursor.is_over(bounds)
            && self.enabled_of(state.active)
            && cursor.is_over(active_circle);
        let label_visible = !matches!(self.value_indicator, ValueIndicator::None)
            && enabled
            && (state.drag.is_some() || cursor_over_active || state.keyboard_focus);
        state
            .label_scale_spring
            .set_target(label_visible as u32 as f32, now);

        // The value indicator travels to the newly active handle instead
        // of teleporting: animate the difference along the axis.
        if state.active != active_at_entry {
            let label_should_travel = label_visible_at_entry && label_visible;
            if label_should_travel {
                let previous_center = center_at(
                    len,
                    state.width_spring(active_at_entry),
                    state.position_spring(active_at_entry),
                );
                let current_center = center_at(
                    len,
                    state.width_spring(state.active),
                    state.position_spring(state.active),
                );
                let offset = state.label_anchor_offset_spring.value();
                let travel = match self.axis {
                    Axis::Horizontal => previous_center - current_center + offset,
                    // Screen Y grows downward; slider coordinates grow upward.
                    Axis::Vertical => current_center - previous_center + offset,
                };
                state.label_anchor_offset_spring.reset(travel, now);
            } else {
                // Label was hidden: appear directly at the new handle.
                state.label_anchor_offset_spring.reset(0.0, now);
            }
        }
        state.label_anchor_offset_spring.set_target(0.0, now);
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &iced::advanced::renderer::Style,
        layout: layout::Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &iced::Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State<Renderer::Paragraph>>();
        let style = state.style_spring.value();
        let enabled = self.is_enabled();
        let bounds = layout.bounds();
        let len = main_len(self.axis, bounds);
        let track_height = self.size.track_height;

        let lower_width = state.lower_width_spring.value();
        let lower_center = center_at(len, lower_width, state.lower_position_spring.value());
        let upper_width = state.upper_width_spring.value();
        let upper_center = center_at(len, upper_width, state.upper_position_spring.value());

        let cross_center = match self.axis {
            Axis::Horizontal => bounds.y + bounds.height / 2.0,
            Axis::Vertical => bounds.x + bounds.width / 2.0,
        };

        // --- Track
        let active = Segment::new(
            lower_center + lower_width / 2.0 + HANDLE_GAP,
            upper_center - upper_width / 2.0 - HANDLE_GAP,
            false,
            false,
        );
        let inactive = [
            Segment::new(
                0.0,
                lower_center - lower_width / 2.0 - HANDLE_GAP,
                true,
                false,
            ),
            Segment::new(
                upper_center + upper_width / 2.0 + HANDLE_GAP,
                len,
                false,
                true,
            ),
        ];

        if let Some(active) = active.valid() {
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
        for inactive in inactive.iter().filter_map(Segment::valid) {
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
                } else if active.valid().is_some_and(|active| active.contains(&main)) {
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
            for inactive in inactive.iter().filter_map(Segment::valid) {
                if inactive.start <= 0.0
                    && inactive.end - inactive.start > STOP_INDICATOR_MIN_SEGMENT
                {
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
                if inactive.end >= len && inactive.end - inactive.start > STOP_INDICATOR_MIN_SEGMENT
                {
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

        // --- Handles
        // When handles overlap, the inactive one gets an outline to stay
        // visible on top.
        let overlap =
            (upper_center - lower_center).abs() <= (lower_width + upper_width) / 2.0 + f32::EPSILON;
        let lower = handle_rect(
            self.axis,
            bounds,
            self.size.handle_height,
            lower_width,
            lower_center,
        );
        let upper = handle_rect(
            self.axis,
            bounds,
            self.size.handle_height,
            upper_width,
            upper_center,
        );

        let draw_handle =
            |renderer: &mut Renderer, handle: Rectangle, thumb: Thumb, outline: bool| {
                renderer.fill_quad(
                    Quad {
                        bounds: handle,
                        border: if outline {
                            Border::default()
                                .rounded(f32::MAX)
                                .width(HANDLE_OVERLAP_WIDTH)
                                .color(style.handle_overlap_outline)
                        } else {
                            Border::default().rounded(f32::MAX)
                        },
                        ..Quad::default()
                    },
                    if self.enabled_of(thumb) {
                        style.handle
                    } else {
                        style.handle_disabled
                    },
                );
            };

        match state.active {
            Thumb::Upper => {
                draw_handle(renderer, lower, Thumb::Lower, overlap);
                draw_handle(renderer, upper, Thumb::Upper, false);
            }
            Thumb::Lower => {
                draw_handle(renderer, upper, Thumb::Upper, overlap);
                draw_handle(renderer, lower, Thumb::Lower, false);
            }
        }
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
        slider_mouse_interaction(
            !self.is_enabled(),
            state.drag.is_some(),
            cursor.is_over(layout.bounds()),
        )
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
        let enabled = self.is_enabled();
        // Shift bounds by the scrollable translation.
        let widget_bounds = layout.bounds();
        let bounds = Rectangle {
            x: widget_bounds.x + translation.x,
            y: widget_bounds.y + translation.y,
            ..widget_bounds
        };
        let len = main_len(self.axis, bounds);

        let lower_circle = handle_circle(
            self.axis,
            bounds,
            self.size.handle_height,
            center_at(
                len,
                state.width_spring(Thumb::Lower),
                state.position_spring(Thumb::Lower),
            ),
        );
        let upper_circle = handle_circle(
            self.axis,
            bounds,
            self.size.handle_height,
            center_at(
                len,
                state.width_spring(Thumb::Upper),
                state.position_spring(Thumb::Upper),
            ),
        );

        let style = state.style_spring.value();
        let focus = state.focus_ring_spring.value();
        let label_scale = state.label_scale_spring.value();

        let mut ripples = [None, None];
        if state.ripples[0].is_visible(lower_circle.size(), self.enabled_of(Thumb::Lower)) {
            ripples[0] = Some(RippleLayer {
                ripple: state.ripples[0],
                layer: style.state_layer,
                bounds: lower_circle,
                enabled: self.enabled_of(Thumb::Lower),
            });
        }
        if state.ripples[1].is_visible(upper_circle.size(), self.enabled_of(Thumb::Upper)) {
            ripples[1] = Some(RippleLayer {
                ripple: state.ripples[1],
                layer: style.state_layer,
                bounds: upper_circle,
                enabled: self.enabled_of(Thumb::Upper),
            });
        }

        // The focus ring and the value indicator follow the active handle
        let active_circle = match state.active {
            Thumb::Lower => lower_circle,
            Thumb::Upper => upper_circle,
        };
        let focus_ring = (focus > 0.01 && enabled && state.keyboard_focus).then(|| {
            let size = focus_ring_size(self.size.handle_height) * focus;
            let full = focus_ring_size(self.size.handle_height);
            (
                Rectangle {
                    x: active_circle.x + (active_circle.width - full) / 2.0 + (full - size) / 2.0,
                    y: active_circle.y + (active_circle.height - full) / 2.0 + (full - size) / 2.0,
                    width: size,
                    height: size,
                },
                Color {
                    a: style.focus_ring.a * focus,
                    ..style.focus_ring
                },
            )
        });

        let active_factor = if state.drag.is_some() {
            state.position_spring(state.active)
        } else {
            value_factor(self.value_of(state.active), &self.range) as f32
        };
        let active_handle = handle_rect(
            self.axis,
            bounds,
            self.size.handle_height,
            state.width_spring(state.active),
            center_at(len, state.width_spring(state.active), active_factor),
        );
        let label = (label_scale > 0.01).then(|| {
            // While the label travels, shift it along the axis by remaining distance.
            let mut anchor = label_anchor(self.axis, &active_handle);
            let travel = state.label_anchor_offset_spring.value();
            match self.axis {
                Axis::Horizontal => anchor.x += travel,
                Axis::Vertical => anchor.y += travel,
            }
            ValueLabel {
                axis: self.axis,
                anchor,
                width: state.label_paragraph.min_width() + 2.0 * LABEL_HORIZONTAL_PADDING,
                height: LABEL_MIN_HEIGHT,
                scale: label_scale,
                background: style.label_container,
                text_color: style.label_text,
                content: state.label_content.clone(),
            }
        });

        if ripples.iter().all(Option::is_none) && focus_ring.is_none() && label.is_none() {
            return None;
        }

        let overlay_element = SliderOverlay {
            ripples,
            focus_ring,
            label,
        };

        Some(overlay::Element::new(Box::new(overlay_element)))
    }
}

impl<'a, T, Message, Theme, Renderer> From<RangeSlider<'a, T, Message>>
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
    fn from(slider: RangeSlider<'a, T, Message>) -> Element<'a, Message, Theme, Renderer> {
        Element::new(slider)
    }
}
