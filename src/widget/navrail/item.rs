pub const INDICATOR_SMALL_HEIGHT: f32 = 32.0;
pub const INDICATOR_SMALL_WIDTH: f32 = 56.0;
pub const ITEM_SMALL_ICON_LABEL_SPACE: f32 = 4.0;
pub const ITEM_SMALL_LABEL_SIZE: f32 = 12.0;
pub const ITEM_SMALL_LABEL_LINE_HEIGHT: f32 = 16.0;
/// Total height of a collapsed item, matching the M3 spec (`container_height`
/// of 64dp: 6 padding + 32 indicator + 4 space + 16 label + 6 padding).
pub const ITEM_SMALL_TOTAL_HEIGHT: f32 = 64.0;
const ITEM_SMALL_VERTICAL_PADDING: f32 = (ITEM_SMALL_TOTAL_HEIGHT
    - INDICATOR_SMALL_HEIGHT
    - ITEM_SMALL_ICON_LABEL_SPACE
    - ITEM_SMALL_LABEL_LINE_HEIGHT)
    / 2.0;
/// Height of a collapsed item when labels are hidden (icon-only).
pub const ITEM_SMALL_NO_LABEL_HEIGHT: f32 =
    INDICATOR_SMALL_HEIGHT + 2.0 * ITEM_SMALL_VERTICAL_PADDING + ITEM_SMALL_ICON_LABEL_SPACE;

pub const INDICATOR_LARGE_HEIGHT: f32 = 56.0;
pub const INDICATOR_LARGE_PADDING: f32 = 16.0;
pub const ITEM_LARGE_ICON_LABEL_SPACE: f32 = 8.0;
pub const ITEM_LARGE_LABEL_SIZE: f32 = 14.0;
pub const ITEM_LARGE_LABEL_LINE_HEIGHT: f32 = 20.0;

const ITEM_LARGE_LABEL_START_X: f32 = 28.0;

pub const ICON_SIZE: f32 = 24.0;

/// Horizontal breathing room kept on each side of a collapsed item's label.
const ITEM_SMALL_LABEL_MARGIN: f32 = 8.0;
/// Width of an expanded item that is not the label itself: the icon, the space between the icon
/// and the label, and the padding on both sides of the pill. Subtracting it from the available
/// rail width yields the widest label that still fits.
const ITEM_LARGE_LABEL_CHROME: f32 =
    ICON_SIZE + 4.0 * INDICATOR_LARGE_PADDING + ITEM_LARGE_ICON_LABEL_SPACE;

use iced::time::Instant;

use iced::{
    Element, Font, Length, Pixels, Point, Rectangle, Size, Vector,
    advanced::{
        Widget,
        layout::Node,
        text::{self, Paragraph},
        widget::Tree,
    },
    border::Radius,
};

use crate::{
    animation::motion::{self, Spring, default_effects, fast_effects},
    widget::{
        internal::navigation_common::{
            build_badge, colors_for, draw_icon, draw_indicator, draw_label, handle_press,
            icon_paragraph, paragraph, update_springs,
        },
        tooltip,
    },
};

pub use crate::widget::internal::navigation_common::NavigationItemContent as Content;
pub use crate::widget::internal::navigation_common::NavigationItemStyle as Style;
pub use crate::widget::internal::navigation_common::State as CommonState;

pub(crate) struct State {
    pub common: CommonState,
    pub expansion: f32,
    /// Hover state of the tooltip that spells out a label that does not fit.
    tooltip: tooltip::Hover,
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Fits `content` into `max_width`, trimming it and appending an ellipsis when it overflows.
///
/// Returns the text to shape together with whether it had to be shortened.
fn fit_label<Renderer>(
    renderer: &Renderer,
    content: &str,
    max_width: f32,
    size: Pixels,
    line_height: text::LineHeight,
    font: Option<Font>,
) -> (String, bool)
where
    Renderer: text::Renderer,
    Renderer::Font: From<Font>,
{
    let width_of = |text: &str| {
        paragraph(
            renderer,
            text,
            Size::INFINITE,
            size,
            line_height,
            font,
            iced::alignment::Vertical::Center,
        )
        .min_bounds()
        .width
    };

    if !max_width.is_finite() || width_of(content) <= max_width {
        return (content.to_owned(), false);
    }

    const ELLIPSIS: char = '…';
    let ellipsis_width = width_of("…");

    if ellipsis_width > max_width {
        return (String::new(), true);
    }

    let mut end = content.len();

    // Walk backwards over char boundaries until the prefix plus the ellipsis fits.
    while end > 0 {
        end = content[..end]
            .char_indices()
            .next_back()
            .map_or(0, |(index, _)| index);

        let candidate = content[..end].trim_end();

        if width_of(candidate) + ellipsis_width <= max_width {
            let mut truncated = String::with_capacity(candidate.len() + ELLIPSIS.len_utf8());
            truncated.push_str(candidate);
            truncated.push(ELLIPSIS);
            return (truncated, true);
        }
    }

    (ELLIPSIS.to_string(), true)
}

pub struct Item<'a, Message, Theme, Renderer>
where
    Message: Clone,
    Renderer: iced::advanced::text::Renderer
        + iced::advanced::svg::Renderer
        + iced_widget::renderer::geometry::Renderer,
    Renderer::Font: From<iced::Font>,
{
    expanded: bool,
    style: Style,
    content: Content<'a, Message>,
    label_font: Option<Font>,
    active: bool,
    active_transition_spring: Option<Spring>,
    color_spring: Option<Spring>,
    hide_labels_when_collapsed: bool,
    /// Width available to the labels, or `None` when the rail grows with its content.
    max_label_width: Option<f32>,
    tooltip_style: tooltip::Style,

    // Cache
    small_label_paragraph: Option<Renderer::Paragraph>,
    large_label_paragraph: Option<Renderer::Paragraph>,
    small_label_text: Option<String>,
    large_label_text: Option<String>,
    small_label_truncated: bool,
    large_label_truncated: bool,
    icon_paragraph: Option<Renderer::Paragraph>,
    badge: Option<Element<'a, Message, Theme, Renderer>>,
}

impl<'a, Message, Theme, Renderer> Item<'a, Message, Theme, Renderer>
where
    Message: 'a + Clone,
    Theme: 'a,
    Renderer: 'a
        + iced::advanced::text::Renderer
        + iced::advanced::svg::Renderer
        + iced_widget::renderer::geometry::Renderer,
    Renderer::Font: From<iced::Font>,
{
    #[must_use]
    pub fn new(
        style: Style,
        content: Content<'a, Message>,
        expanded: bool,
        label_font: Option<Font>,
        active: bool,
        hide_labels_when_collapsed: bool,
        max_label_width: Option<f32>,
        tooltip_style: tooltip::Style,
    ) -> Self {
        let badge =
            build_badge::<Message, Theme, Renderer>(&style, content.badge.as_ref(), ICON_SIZE);

        Self {
            expanded,
            style,
            content,
            label_font,
            active,
            active_transition_spring: None,
            color_spring: None,
            hide_labels_when_collapsed,
            max_label_width,
            tooltip_style,
            small_label_paragraph: None,
            large_label_paragraph: None,
            small_label_text: None,
            large_label_text: None,
            small_label_truncated: false,
            large_label_truncated: false,
            icon_paragraph: None,
            badge,
        }
    }

    pub fn motion_scheme_maybe(mut self, scheme: Option<motion::Scheme>) -> Self {
        if let Some(scheme) = scheme {
            self.color_spring = Some(fast_effects(scheme));
            self.active_transition_spring = Some(default_effects(scheme));
        }
        self
    }

    #[must_use]
    pub fn active_transition_spring_maybe(mut self, spring: Option<Spring>) -> Self {
        self.active_transition_spring = spring;
        self
    }

    #[must_use]
    pub fn color_spring_maybe(mut self, spring: Option<Spring>) -> Self {
        self.color_spring = spring;
        self
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Item<'a, Message, Theme, Renderer>
where
    Message: Clone,
    Renderer: iced::advanced::text::Renderer
        + iced::advanced::svg::Renderer
        + iced_widget::renderer::geometry::Renderer,
    Renderer::Font: From<iced::Font>,
{
    fn size(&self) -> Size<Length> {
        let height = if self.expanded {
            INDICATOR_LARGE_HEIGHT
        } else if self.hide_labels_when_collapsed {
            ITEM_SMALL_NO_LABEL_HEIGHT
        } else {
            ITEM_SMALL_TOTAL_HEIGHT
        };
        Size {
            width: Length::Shrink,
            height: Length::Fixed(height),
        }
    }

    fn tag(&self) -> iced::advanced::widget::tree::Tag {
        iced::advanced::widget::tree::Tag::of::<State>()
    }

    fn state(&self) -> iced::advanced::widget::tree::State {
        let colors = colors_for(self.active, &self.style);
        let common = CommonState::new(
            self.active,
            colors,
            self.active_transition_spring,
            self.color_spring,
            Instant::now(),
        );
        let state = State {
            common,
            expansion: self.expanded as usize as f32,
            tooltip: tooltip::Hover::default(),
        };

        iced::advanced::widget::tree::State::new(state)
    }

    fn children(&self) -> Vec<Tree> {
        match &self.badge {
            Some(badge) => vec![Tree::new(badge.as_widget())],
            None => Vec::new(),
        }
    }

    fn diff(&self, tree: &mut Tree) {
        let children = match &self.badge {
            Some(badge) => vec![badge.as_widget()],
            None => Vec::new(),
        };

        tree.diff_children(&children);
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> iced::advanced::layout::Node {
        let expansion = tree.state.downcast_ref::<State>().expansion.clamp(0.0, 1.0);

        let icon = match self.active {
            true => &self.content.icon_active,
            false => &self.content.icon_inactive,
        };
        self.icon_paragraph = icon_paragraph(renderer, icon, ICON_SIZE);

        // A label that does not fit the rail is ellipsized; hovering the item then shows the full
        // text as a tooltip. Only the label that is actually visible for the current mode is
        // shortened, so the one that is fading out during the expansion animation keeps its size.
        let small_max_width = match (self.expanded, self.max_label_width) {
            (false, Some(width)) => (width - 2.0 * ITEM_SMALL_LABEL_MARGIN).max(0.0),
            _ => f32::INFINITY,
        };
        let large_max_width = match (self.expanded, self.max_label_width) {
            (true, Some(width)) => (width - ITEM_LARGE_LABEL_CHROME).max(0.0),
            _ => f32::INFINITY,
        };

        let (small_text, small_truncated) = fit_label(
            renderer,
            &self.content.label,
            small_max_width,
            Pixels(ITEM_SMALL_LABEL_SIZE),
            text::LineHeight::Absolute(Pixels(ITEM_SMALL_LABEL_LINE_HEIGHT)),
            self.label_font,
        );
        let (large_text, large_truncated) = fit_label(
            renderer,
            &self.content.label,
            large_max_width,
            Pixels(ITEM_LARGE_LABEL_SIZE),
            text::LineHeight::Absolute(Pixels(ITEM_LARGE_LABEL_LINE_HEIGHT)),
            self.label_font,
        );

        // Labels hidden while collapsed are spelled out by the wrapping `Tooltip` instead.
        self.small_label_truncated = small_truncated && !self.hide_labels_when_collapsed;
        self.large_label_truncated = large_truncated;

        if self.small_label_paragraph.is_none()
            || self.small_label_text.as_deref() != Some(small_text.as_str())
        {
            self.small_label_paragraph = Some(paragraph(
                renderer,
                &small_text,
                Size::INFINITE,
                Pixels(ITEM_SMALL_LABEL_SIZE),
                text::LineHeight::Absolute(Pixels(ITEM_SMALL_LABEL_LINE_HEIGHT)),
                self.label_font,
                iced::alignment::Vertical::Center,
            ));
            self.small_label_text = Some(small_text);
        }
        if self.large_label_paragraph.is_none()
            || self.large_label_text.as_deref() != Some(large_text.as_str())
        {
            self.large_label_paragraph = Some(paragraph(
                renderer,
                &large_text,
                Size::INFINITE,
                Pixels(ITEM_LARGE_LABEL_SIZE),
                text::LineHeight::Absolute(Pixels(ITEM_LARGE_LABEL_LINE_HEIGHT)),
                self.label_font,
                iced::alignment::Vertical::Center,
            ));
            self.large_label_text = Some(large_text);
        }

        let small_bounds = self
            .small_label_paragraph
            .as_ref()
            .map(|paragraph| paragraph.min_bounds())
            .unwrap_or(Size::ZERO);
        let large_bounds = self
            .large_label_paragraph
            .as_ref()
            .map(|paragraph| paragraph.min_bounds())
            .unwrap_or(Size::ZERO);

        let hug_width = large_bounds.width
            + ICON_SIZE
            + 2.0 * INDICATOR_LARGE_PADDING
            + ITEM_LARGE_ICON_LABEL_SPACE;

        let node_width = if self.expanded {
            hug_width + 2.0 * INDICATOR_LARGE_PADDING
        } else {
            let max = limits.max().width;
            match max.is_finite() {
                true => max,
                false => hug_width + 2.0 * super::constants::ITEM_OFFSET,
            }
        };

        let small_height = if self.hide_labels_when_collapsed {
            ITEM_SMALL_NO_LABEL_HEIGHT
        } else {
            ITEM_SMALL_TOTAL_HEIGHT
        };
        let small_pill_y = if self.hide_labels_when_collapsed {
            (small_height - INDICATOR_SMALL_HEIGHT) / 2.0
        } else {
            ITEM_SMALL_VERTICAL_PADDING
        };

        let pill_height = lerp(INDICATOR_SMALL_HEIGHT, INDICATOR_LARGE_HEIGHT, expansion).round();
        let pill_width = lerp(INDICATOR_SMALL_WIDTH, hug_width, expansion).round();
        let pill_x = if self.expanded {
            INDICATOR_LARGE_PADDING
        } else {
            super::constants::ITEM_OFFSET
        };
        let pill_y = lerp(small_pill_y, 0.0, expansion).round();
        let node_height = lerp(small_height, INDICATOR_LARGE_HEIGHT, expansion).round();

        let indicator_node =
            Node::new(Size::new(pill_width, pill_height)).move_to(Point::new(pill_x, pill_y));

        // Leading padding of 16dp keeps the icon centered in the collapsed
        // pill (56/2 - 24/2) and leading-aligned in the expanded pill.
        let icon_x = pill_x + INDICATOR_LARGE_PADDING;
        let icon_y = (pill_y + (pill_height - ICON_SIZE) / 2.0).round();
        let icon_node =
            Node::new(Size::new(ICON_SIZE, ICON_SIZE)).move_to(Point::new(icon_x, icon_y));

        let small_label_y = pill_y + pill_height + ITEM_SMALL_ICON_LABEL_SPACE;
        let small_label_node = Node::new(small_bounds).move_to(Point::new(
            (pill_x + (INDICATOR_SMALL_WIDTH - small_bounds.width) / 2.0).round(),
            small_label_y,
        ));

        let large_label_node = Node::new(large_bounds).move_to(Point::new(
            (pill_x
                + lerp(
                    ITEM_LARGE_LABEL_START_X,
                    INDICATOR_LARGE_PADDING + ICON_SIZE + ITEM_LARGE_ICON_LABEL_SPACE,
                    expansion,
                ))
            .round(),
            (pill_y + (pill_height - large_bounds.height) / 2.0).round(),
        ));

        let has_label = self
            .content
            .badge
            .as_ref()
            .is_some_and(|badge| badge.label.is_some());

        // Right edge of the badge in the fully expanded state. Used to size the
        // item, so the width does not depend on the current expansion and stays
        // stable while the rail animates.
        let mut expanded_badge_right = 0.0f32;

        let badge_node = self.badge.as_mut().map(|badge| {
            let node = badge
                .as_widget_mut()
                .layout(&mut tree.children[0], renderer, limits);

            let large_label_end = large_label_node.bounds().x + large_bounds.width;
            let icon_center_y = icon_y + ICON_SIZE / 2.0;
            let label_center_y = large_label_node.bounds().y + large_bounds.height / 2.0;

            // The badge is drawn from the top-right corner of its base, so when
            // collapsed it sits on the icon's top-right corner and when expanded
            // it trails the end of the label.
            let (collapsed_x, expanded_x) = if has_label {
                (icon_x, large_label_end - 8.0)
            } else {
                (icon_x, large_label_end - 14.0)
            };
            let (collapsed_y, expanded_y) = if has_label {
                (icon_center_y - 12.0, label_center_y - 8.0)
            } else {
                (icon_center_y - ICON_SIZE / 2.0, label_center_y - 3.0)
            };

            let badge_x = lerp(collapsed_x, expanded_x, expansion).round();
            let badge_y = lerp(collapsed_y, expanded_y, expansion).round();

            expanded_badge_right = expanded_x + node.bounds().width;

            // Collapsed items are as wide as the rail itself, so keep the badge
            // inside that width instead of letting it spill over the edge.
            let badge_x = if self.expanded {
                badge_x
            } else {
                badge_x
                    .min((node_width - node.bounds().width).max(0.0))
                    .round()
            };

            node.move_to(Point::new(badge_x, badge_y))
        });

        let mut children = vec![
            indicator_node,
            icon_node,
            small_label_node,
            large_label_node,
        ];

        let mut node_size = Size::new(node_width, node_height);
        if let Some(node) = badge_node {
            if self.expanded {
                // The badge trails the label and may be wider than the label's
                // trailing padding. Grow the item (and therefore the auto-sized
                // rail) so the badge is never drawn past the rail's edge.
                if expanded_badge_right > node_size.width {
                    node_size.width = expanded_badge_right + INDICATOR_LARGE_PADDING;
                }
            }

            children.push(node);
        }

        Node::with_children(node_size, children)
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &iced::advanced::renderer::Style,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        viewport: &iced::Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State>();
        let expansion = state.expansion;

        let indicator_bounds = layout.child(0).bounds();
        let icon_bounds = layout.child(1).bounds();
        let small_label_bounds = layout.child(2).bounds();
        let large_label_bounds = layout.child(3).bounds();
        let badge_layout = self.badge.as_ref().map(|_| layout.child(4));

        // Active indicator pill.
        draw_indicator(
            renderer,
            indicator_bounds,
            state.common.active_transition_spring.value(),
            self.style.container_color,
        );

        let colors = state.common.color_spring.value();

        draw_icon(
            renderer,
            self.icon_paragraph.as_ref(),
            match self.active {
                true => &self.content.icon_active,
                false => &self.content.icon_inactive,
            },
            icon_bounds,
            colors.icon,
        );

        let large_label_alpha = if self.hide_labels_when_collapsed {
            // When labels are hidden in collapsed state, smoothly fade the
            // large label in/out alongside the expansion animation.
            expansion.clamp(0.0, 1.0)
        } else {
            let show_labels = expansion > 0.001;
            if show_labels {
                ((expansion - 0.6) / 0.4).clamp(0.0, 1.0)
            } else {
                0.0
            }
        };
        let small_label_alpha = if self.hide_labels_when_collapsed {
            0.0
        } else {
            let show_small = expansion < 0.999;
            if show_small {
                (1.0 - expansion * 3.0).clamp(0.0, 1.0)
            } else {
                0.0
            }
        };

        if small_label_alpha > 0.0
            && let Some(label) = &self.small_label_paragraph
        {
            draw_label(
                renderer,
                label,
                small_label_bounds,
                colors.label.scale_alpha(small_label_alpha),
            );
        }

        if large_label_alpha > 0.0
            && let Some(label) = &self.large_label_paragraph
        {
            draw_label(
                renderer,
                label,
                large_label_bounds,
                colors.label.scale_alpha(large_label_alpha),
            );
        }

        if let Some(badge) = &self.badge {
            badge.as_widget().draw(
                &tree.children[0],
                renderer,
                theme,
                style,
                badge_layout.unwrap(),
                cursor,
                viewport,
            );
        }

        state.common.ripple.draw(
            renderer,
            indicator_bounds,
            Radius::from(f32::MAX),
            self.style.state_layer,
            true,
        );
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &iced::Event,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn iced::advanced::Clipboard,
        shell: &mut iced::advanced::Shell<'_, Message>,
        _viewport: &iced::Rectangle,
    ) {
        let state = tree.state.downcast_mut::<State>();

        let bounds = layout.bounds();
        let indicator_bounds = layout.child(0).bounds();

        handle_press(
            event,
            &mut state.common,
            indicator_bounds,
            bounds,
            cursor,
            &self.content.on_press,
            shell,
        );
        update_springs(
            event,
            &mut state.common,
            self.active,
            &self.style,
            self.active_transition_spring,
            self.color_spring,
            shell,
        );

        // Spell out a label that had to be shortened, like the rail's other tooltips.
        let truncated = if self.expanded {
            self.large_label_truncated
        } else {
            self.small_label_truncated
        };

        state.tooltip.update(
            event,
            cursor,
            Some(bounds),
            truncated,
            tooltip::DEFAULT_DELAY,
            shell,
        );
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        _viewport: &iced::Rectangle,
        _renderer: &Renderer,
    ) -> iced::advanced::mouse::Interaction {
        match cursor.is_over(layout.bounds()) {
            true => iced::advanced::mouse::Interaction::Pointer,
            false => iced::advanced::mouse::Interaction::None,
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: iced::advanced::Layout<'b>,
        _renderer: &Renderer,
        _viewport: &Rectangle,
        translation: Vector,
    ) -> Option<iced::advanced::overlay::Element<'b, Message, Theme, Renderer>> {
        let state = tree.state.downcast_ref::<State>();
        let truncated = if self.expanded {
            self.large_label_truncated
        } else {
            self.small_label_truncated
        };

        if !truncated || !state.tooltip.is_open() {
            return None;
        }

        // The label nodes are pushed in a fixed order at the end of `layout`.
        let label_bounds = layout.child(if self.expanded { 3 } else { 2 }).bounds();
        let anchor = Rectangle::new(label_bounds.position() + translation, label_bounds.size());

        Some(tooltip::label_overlay(
            anchor,
            text::Fragment::Borrowed(self.content.label.as_ref()),
            self.label_font.map(Renderer::Font::from),
            self.tooltip_style,
            tooltip::Position::Right,
            tooltip::DEFAULT_GAP,
            tooltip::DEFAULT_MAX_WIDTH,
            None,
        ))
    }
}

impl<'a, Message, Theme, Renderer> From<Item<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a + Clone,
    Theme: 'a,
    Renderer: 'a
        + iced::advanced::text::Renderer
        + iced::advanced::svg::Renderer
        + iced_widget::renderer::geometry::Renderer,
    Renderer::Font: From<iced::Font>,
{
    fn from(value: Item<'a, Message, Theme, Renderer>) -> Self {
        Element::new(value)
    }
}
