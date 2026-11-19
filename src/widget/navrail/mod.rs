mod item;

use std::time::Instant;

use iced::{
    Color, Element, Font, Length, Padding, Pixels, Rectangle, Size, Vector,
    advanced::{Widget, layout::Node, mouse, text, widget::Tree},
};

use crate::{
    animation::motion::{self, Spring, SpringValue, default_spatial},
    style::StateLayer,
    theme::ColorScheme,
    widget::{
        self, Action, badge,
        button::{self, ElevationStates, Outline},
        common_icons::{self},
        hybrid_icon::Icon,
        navrail::item::State as ItemState,
    },
};

pub use item::Content as Item;
pub use item::Style as ItemStyle;

const MENU_ICON_SIZE: Pixels = Pixels(24.0);
const MENU_BUTTON_PADDING: f32 = 4.0;

const MENU_SECTION_SPACE: f32 = 16.0;
const ITEMS_SECTION_SPACE: f32 = 40.0;
// Small FAB container size
const FAB_OFFSET: f32 = (constants::CONTAINER_COLLAPSED_WIDTH - 56.0) / 2.0;
const MENU_OFFSET: f32 =
    (constants::CONTAINER_COLLAPSED_WIDTH - MENU_ICON_SIZE.0 - MENU_BUTTON_PADDING * 2.0) / 2.0;

pub(crate) mod constants {
    use crate::widget::navrail::item::INDICATOR_SMALL_WIDTH;

    pub const CONTAINER_COLLAPSED_WIDTH: f32 = 96.0;
    #[allow(dead_code)]
    pub const CONTAINER_EXPANDED_MIN_WIDTH: f32 = 220.0;
    pub const CONTAINER_EXPANDED_MAX_WIDTH: f32 = 360.0;
    pub const ITEM_OFFSET: f32 = (CONTAINER_COLLAPSED_WIDTH - INDICATOR_SMALL_WIDTH) / 2.0;
    pub const CONTAINER_VERTICAL_PADDING: f32 = 44.0;
}

#[cfg(feature = "pub-internal-const")]
pub use constants::*;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ItemAlignment {
    #[default]
    Top,
    Center,
    Bottom,
}

pub struct Fab<'a, Message>
where
    Message: Clone,
{
    pub icon: Icon<'a>,
    pub label: text::Fragment<'a>,
    pub style: crate::widget::fab::Style,
    pub on_press: Action<'a, Message>,
}

#[derive(Default, Clone, Copy, PartialEq, PartialOrd)]
pub enum Mode {
    #[default]
    Collapsed,
    Expanded {
        /// The width of the navigation rail container.
        ///
        /// This value will be clamped between the minimum and maximum container width.
        width: Pixels,
    },
    /// Width is automatically determined by the widest item.
    ExpandedAuto,
}

impl Mode {
    fn width(&self) -> Pixels {
        match self {
            Mode::Collapsed => Pixels(constants::CONTAINER_COLLAPSED_WIDTH),
            Mode::Expanded { width } => Pixels(Into::<f32>::into(*width).clamp(
                constants::CONTAINER_EXPANDED_MIN_WIDTH,
                constants::CONTAINER_EXPANDED_MAX_WIDTH,
            )),
            Mode::ExpandedAuto => Pixels(constants::CONTAINER_EXPANDED_MIN_WIDTH),
        }
    }

    fn is_expanded(&self) -> bool {
        matches!(self, Mode::Expanded { .. } | Mode::ExpandedAuto)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    pub menu_button_style: button::Style,
    pub item_style: ItemStyle,
    pub badge_style: badge::Style,
}

impl Style {
    pub fn new(theme: &(impl ColorScheme + ?Sized)) -> Self {
        let style = button::StateStyle {
            container: Color::TRANSPARENT,
            label: Color::TRANSPARENT,
            icon: theme.on_surface(),
            outline: Outline::default(),
        };
        Self {
            menu_button_style: button::Style {
                regular: style,
                unselected: style,
                selected: style,
                disabled: style,
                disabled_unselected: style,
                disabled_selected: style,
                elevation: ElevationStates::new(theme.shadow()),
                state_layer: StateLayer::new(theme.on_surface()),
                state_layer_unselected: StateLayer::new(theme.on_surface()),
                state_layer_selected: StateLayer::new(theme.on_surface()),
            },
            item_style: {
                let mut item_style = ItemStyle::new(theme);
                // Active icon uses `on_secondary_container` while the active
                // label stays `secondary`, per the M3 expressive spec.
                item_style.content_active = theme.on_secondary_container();
                item_style
            },
            badge_style: badge::Style::new(theme),
        }
    }
}

pub struct NavRail<'a, Message, Theme, Renderer>
where
    Message: 'a + Clone,
    Renderer: iced::advanced::text::Renderer
        + iced::advanced::svg::Renderer
        + iced_widget::renderer::geometry::Renderer,
    Renderer::Font: From<iced::Font>,
{
    style: Style,
    mode: Mode,
    on_menu_pressed: Option<Box<dyn Fn(Mode) -> Message + 'a>>,
    fab: Option<Fab<'a, Message>>,
    items: Vec<Item<'a, Message>>,
    label_font: Option<Font>,
    active_index: usize,
    item_alignment: ItemAlignment,
    container_vertical_padding: Option<f32>,
    motion_scheme: Option<motion::Scheme>,
    item_active_transition_spring: Option<Spring>,
    item_color_spring: Option<Spring>,
    expand_spring: Option<Spring>,
    hide_labels_when_collapsed: bool,
    menu_visible: bool,
    scrollbar_color: Option<Color>,

    menu_button: Option<Element<'a, Message, Theme, Renderer>>,
    fab_element: Option<Element<'a, Message, Theme, Renderer>>,
    item_elements: Vec<Element<'a, Message, Theme, Renderer>>,
    items_container: Option<Element<'a, Message, Theme, Renderer>>,
}

impl<'a, Message, Theme, Renderer> NavRail<'a, Message, Theme, Renderer>
where
    Message: 'a + Clone,
    Renderer: iced::advanced::text::Renderer
        + iced::advanced::svg::Renderer
        + iced_widget::renderer::geometry::Renderer,
    Renderer::Font: From<iced::Font>,
{
    #[must_use]
    pub fn new(style: Style, items: Vec<Item<'a, Message>>) -> Self {
        Self {
            style,
            mode: Mode::default(),
            on_menu_pressed: None,
            fab: None,
            items,
            label_font: None,
            active_index: 0,
            item_alignment: ItemAlignment::default(),
            container_vertical_padding: None,
            motion_scheme: None,
            item_active_transition_spring: None,
            item_color_spring: None,
            expand_spring: None,
            hide_labels_when_collapsed: false,
            menu_visible: true,
            scrollbar_color: None,
            menu_button: None,
            fab_element: None,
            item_elements: Vec::new(),
            items_container: None,
        }
    }

    #[must_use]
    pub fn hide_labels_when_collapsed(mut self, hide: bool) -> Self {
        self.hide_labels_when_collapsed = hide;
        self
    }

    #[must_use]
    pub fn menu_visible(mut self, visible: bool) -> Self {
        self.menu_visible = visible;
        self
    }

    #[must_use]
    pub fn mode(mut self, mode: Mode) -> Self {
        self.mode = mode;
        self
    }

    #[must_use]
    pub fn mode_maybe(self, maybe_mode: Option<Mode>) -> Self {
        match maybe_mode {
            Some(mode) => self.mode(mode),
            None => self,
        }
    }

    #[must_use]
    pub fn scrollbar_color(mut self, color: Color) -> Self {
        self.scrollbar_color = Some(color);
        self
    }

    #[must_use]
    pub fn scrollbar_color_maybe(mut self, maybe_color: Option<Color>) -> Self {
        if let Some(color) = maybe_color {
            self.scrollbar_color = Some(color);
        }
        self
    }

    #[must_use]
    pub fn fab(mut self, fab: Fab<'a, Message>) -> Self {
        self.fab = Some(fab);
        self
    }

    #[must_use]
    pub fn fab_maybe(mut self, maybe_fab: Option<Fab<'a, Message>>) -> Self {
        self.fab = maybe_fab;
        self
    }

    #[must_use]
    pub fn on_menu_pressed(mut self, on_press: impl Fn(Mode) -> Message + 'a) -> Self {
        self.on_menu_pressed = Some(Box::new(on_press));
        self
    }

    #[must_use]
    pub fn on_menu_pressed_maybe(
        mut self,
        on_press: Option<impl Fn(Mode) -> Message + 'a>,
    ) -> Self {
        self.on_menu_pressed =
            on_press.map(|callback| Box::new(callback) as Box<dyn Fn(Mode) -> Message + 'a>);
        self
    }

    #[must_use]
    pub fn active(mut self, index: usize) -> Self {
        self.active_index = index;
        self
    }

    #[must_use]
    pub fn label_font(mut self, font: iced::Font) -> Self {
        self.label_font = Some(font);
        self
    }

    #[must_use]
    pub fn label_font_maybe(mut self, maybe_font: Option<iced::Font>) -> Self {
        self.label_font = maybe_font;
        self
    }

    #[must_use]
    pub fn item_alignment(mut self, alignment: ItemAlignment) -> Self {
        self.item_alignment = alignment;
        self
    }

    #[must_use]
    pub fn item_alignment_maybe(self, maybe_alignment: Option<ItemAlignment>) -> Self {
        match maybe_alignment {
            Some(alignment) => self.item_alignment(alignment),
            None => self,
        }
    }

    #[must_use]
    pub fn container_vertical_padding(mut self, padding: f32) -> Self {
        self.container_vertical_padding = Some(padding);
        self
    }

    #[must_use]
    pub fn container_vertical_padding_maybe(mut self, maybe_padding: Option<f32>) -> Self {
        self.container_vertical_padding = maybe_padding;
        self
    }

    #[must_use]
    pub fn motion_scheme(mut self, scheme: motion::Scheme) -> Self {
        self.motion_scheme = Some(scheme);
        self
    }

    #[must_use]
    pub fn motion_scheme_maybe(mut self, scheme: Option<motion::Scheme>) -> Self {
        self.motion_scheme = scheme;
        self
    }

    #[must_use]
    pub fn item_active_transition_spring(mut self, spring: Spring) -> Self {
        self.item_active_transition_spring = Some(spring);
        self
    }

    #[must_use]
    pub fn item_active_transition_spring_maybe(mut self, spring: Option<Spring>) -> Self {
        self.item_active_transition_spring = spring;
        self
    }

    #[must_use]
    pub fn item_color_spring(mut self, spring: Spring) -> Self {
        self.item_color_spring = Some(spring);
        self
    }

    #[must_use]
    pub fn item_color_spring_maybe(mut self, spring: Option<Spring>) -> Self {
        self.item_color_spring = spring;
        self
    }

    #[must_use]
    pub fn expand_spring(mut self, spring: Spring) -> Self {
        self.expand_spring = Some(spring);
        self
    }

    #[must_use]
    pub fn expand_spring_maybe(mut self, spring: Option<Spring>) -> Self {
        self.expand_spring = spring;
        self
    }

    fn resolve_expand_spring(&self) -> Option<Spring> {
        match (self.expand_spring, self.motion_scheme) {
            (Some(spring), _) => Some(spring),
            (None, Some(scheme)) => Some(default_spatial(scheme)),
            (None, None) => None,
        }
    }

    fn mode_is_expanded(&self) -> bool {
        self.mode.is_expanded()
    }
}

struct State {
    width: SpringValue<f32>,
    /// Normalized expansion progress: 0 = collapsed, 1 = expanded
    expansion: SpringValue<f32>,
    menu_visible: SpringValue<f32>,
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for NavRail<'a, Message, Theme, Renderer>
where
    Message: Clone,
    Theme: 'a,
    Renderer: 'a
        + iced::advanced::text::Renderer
        + iced::advanced::svg::Renderer
        + iced_widget::renderer::geometry::Renderer,
    Renderer::Font: From<iced::Font>,
{
    fn tag(&self) -> iced::advanced::widget::tree::Tag {
        iced::advanced::widget::tree::Tag::of::<State>()
    }

    fn state(&self) -> iced::advanced::widget::tree::State {
        let now = Instant::now();
        let spring = self
            .resolve_expand_spring()
            .unwrap_or_else(|| default_spatial(motion::Scheme::default()));
        let width = self.mode.width().0;
        let expansion = self.mode_is_expanded() as usize as f32;
        let menu_visible_initial =
            (self.menu_button.is_some() && self.menu_visible) as usize as f32;
        let mut state = State {
            width: SpringValue::new(width, spring, now),
            expansion: SpringValue::new(expansion, spring, now),
            menu_visible: SpringValue::new(menu_visible_initial, spring, now),
        };
        state.width.reset(width, now);
        state.expansion.reset(expansion, now);
        state.menu_visible.reset(menu_visible_initial, now);

        iced::advanced::widget::tree::State::new(state)
    }

    fn diff(&self, tree: &mut Tree) {
        let now = Instant::now();
        let state = tree.state.downcast_mut::<State>();

        if let Some(spring) = self.resolve_expand_spring()
            && state.width.spring.model != spring
        {
            state.width.spring.model = spring;
            state.expansion.spring.model = spring;
            state.menu_visible.spring.model = spring;
        }
        state
            .expansion
            .set_target(self.mode_is_expanded() as usize as f32, now);
        state.menu_visible.set_target(
            (self.menu_button.is_some() && self.menu_visible) as usize as f32,
            now,
        );
        // For ExpandedAuto the real width target is computed in layout() from
        // measured item widths. Setting it here to mode.width() (220.0) and
        // then overriding it in layout() causes the spring to be reset twice
        // per frame, breaking the animation. Only set it in diff() for modes
        // with a fixed known width.
        if !matches!(self.mode, Mode::ExpandedAuto) {
            state.width.set_target(self.mode.width().0, now);
        }

        let target_len = 3;
        if tree.children.len() != target_len {
            tree.children = self.children();
        } else {
            let menu_placeholder: Element<'a, Message, Theme, Renderer> = Element::new(
                iced::widget::Space::new()
                    .width(Length::Shrink)
                    .height(Length::Shrink),
            );
            let fab_placeholder: Element<'a, Message, Theme, Renderer> = Element::new(
                iced::widget::Space::new()
                    .width(Length::Shrink)
                    .height(Length::Shrink),
            );
            let items_placeholder: Element<'a, Message, Theme, Renderer> = Element::new(
                iced::widget::Space::new()
                    .width(Length::Shrink)
                    .height(Length::Shrink),
            );

            if let Some(menu) = self.menu_button.as_ref() {
                tree.children[0].diff(menu.as_widget());
            } else {
                tree.children[0].diff(menu_placeholder.as_widget());
            }
            if let Some(fab) = self.fab_element.as_ref() {
                tree.children[1].diff(fab.as_widget());
            } else {
                tree.children[1].diff(fab_placeholder.as_widget());
            }
            if let Some(container) = self.items_container.as_ref() {
                tree.children[2].diff(container.as_widget());
            } else {
                tree.children[2].diff(items_placeholder.as_widget());
            }
        }
    }

    fn children(&self) -> Vec<Tree> {
        let mut children = Vec::with_capacity(3);

        if let Some(menu) = self.menu_button.as_ref() {
            children.push(Tree::new(menu.as_widget()));
        } else {
            let menu_placeholder: Element<'a, Message, Theme, Renderer> = Element::new(
                iced::widget::Space::new()
                    .width(Length::Shrink)
                    .height(Length::Shrink),
            );
            children.push(Tree::new(menu_placeholder.as_widget()));
        }

        if let Some(fab) = self.fab_element.as_ref() {
            children.push(Tree::new(fab.as_widget()));
        } else {
            let fab_placeholder: Element<'a, Message, Theme, Renderer> = Element::new(
                iced::widget::Space::new()
                    .width(Length::Shrink)
                    .height(Length::Shrink),
            );
            children.push(Tree::new(fab_placeholder.as_widget()));
        }

        if let Some(container) = self.items_container.as_ref() {
            children.push(Tree::new(container.as_widget()));
        } else {
            let placeholder: Element<'a, Message, Theme, Renderer> = Element::new(
                iced::widget::Space::new()
                    .width(Length::Shrink)
                    .height(Length::Shrink),
            );
            children.push(Tree::new(placeholder.as_widget()));
        }

        children
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Shrink, Length::Fill)
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> iced::advanced::layout::Node {
        let rail_state = tree.state.downcast_mut::<State>();
        let collapsed_width = constants::CONTAINER_COLLAPSED_WIDTH;
        let expansion = rail_state.expansion.value().clamp(0.0, 1.0);
        let menu_visible = rail_state.menu_visible.value().clamp(0.0, 1.0);

        // Pre-layout pass to determine content width for ExpandedAuto
        let effective_target = match self.mode {
            Mode::ExpandedAuto => {
                let temp_limits =
                    iced::advanced::layout::Limits::new(Size::ZERO, Size::new(f32::MAX, f32::MAX));
                let mut max_item_width = 0.0f32;
                for (index, content) in self.items.iter().enumerate() {
                    let active = index == self.active_index;
                    let mut item_element: Element<'a, Message, Theme, Renderer> = item::Item::new(
                        self.style.item_style,
                        content.clone(),
                        self.mode.is_expanded(),
                        self.label_font,
                        active,
                        self.hide_labels_when_collapsed,
                    )
                    .motion_scheme_maybe(self.motion_scheme)
                    .active_transition_spring_maybe(self.item_active_transition_spring)
                    .color_spring_maybe(self.item_color_spring)
                    .into();
                    let mut temp_tree = Tree::new(item_element.as_widget());
                    temp_tree.state.downcast_mut::<ItemState>().expansion = expansion;
                    let item_layout =
                        item_element
                            .as_widget_mut()
                            .layout(&mut temp_tree, renderer, &temp_limits);
                    max_item_width = max_item_width.max(item_layout.bounds().width);
                }
                max_item_width.max(collapsed_width)
            }
            _ => self.mode.width().0,
        };

        let width = rail_state.width.value().max(collapsed_width);
        let container_bounds = limits.resolve(Length::Fixed(width), Length::Fill, Size::ZERO);
        let container_vertical_padding = self
            .container_vertical_padding
            .unwrap_or(constants::CONTAINER_VERTICAL_PADDING);
        let mut offset = container_vertical_padding;

        let mut children = Vec::with_capacity(3);
        let mut tree_index = 0;

        // Menu
        let mut menu_placeholder: Element<'a, Message, Theme, Renderer> = Element::new(
            iced::widget::Space::new()
                .width(Length::Shrink)
                .height(Length::Shrink),
        );
        let menu_layout = if let Some(menu) = self.menu_button.as_mut() {
            menu.as_widget_mut()
                .layout(&mut tree.children[tree_index], renderer, limits)
        } else {
            menu_placeholder.as_widget_mut().layout(
                &mut tree.children[tree_index],
                renderer,
                limits,
            )
        }
        .translate(Vector::new(MENU_OFFSET, offset));
        let menu_natural_height = menu_layout.bounds().height;
        let menu_effective_height = if self.menu_button.is_some() {
            menu_natural_height * menu_visible
        } else {
            0.0
        };
        children.push(menu_layout);
        tree_index += 1;
        offset += menu_effective_height;
        if self.menu_button.is_some() && menu_visible > 0.001 {
            offset += match self.fab_element.is_some() {
                true => MENU_SECTION_SPACE,
                false => ITEMS_SECTION_SPACE,
            } * menu_visible;
        }
        offset = offset.round();

        // Fab
        let mut fab_placeholder: Element<'a, Message, Theme, Renderer> = Element::new(
            iced::widget::Space::new()
                .width(Length::Shrink)
                .height(Length::Shrink),
        );
        let fab_layout = if let Some(fab) = self.fab_element.as_mut() {
            fab.as_widget_mut()
                .layout(&mut tree.children[tree_index], renderer, limits)
        } else {
            fab_placeholder
                .as_widget_mut()
                .layout(&mut tree.children[tree_index], renderer, limits)
        }
        .translate(Vector::new(FAB_OFFSET, offset));
        let fab_height = fab_layout.bounds().height;
        children.push(fab_layout);
        tree_index += 1;
        if self.fab_element.is_some() {
            offset += fab_height + ITEMS_SECTION_SPACE;
            offset = offset.round();
        }

        let item_start_offset = offset;
        let available_height =
            (container_bounds.height - item_start_offset - container_vertical_padding).max(0.0);

        // Set expansion on real items inside the scrollable container
        if let Some(container) = self.items_container.as_mut() {
            let scrollable_tree = &mut tree.children[tree_index];
            if !scrollable_tree.children.is_empty() {
                let inner_scrollable_tree = &mut scrollable_tree.children[0];
                if !inner_scrollable_tree.children.is_empty() {
                    let column_tree = &mut inner_scrollable_tree.children[0];
                    for item_tree in column_tree.children.iter_mut() {
                        item_tree.state.downcast_mut::<ItemState>().expansion = expansion;
                    }
                }
            }

            let item_limits =
                iced::advanced::layout::Limits::new(Size::ZERO, Size::new(width, available_height));
            let scrollable_layout = container
                .as_widget_mut()
                .layout(scrollable_tree, renderer, &item_limits)
                .translate(Vector::new(0.0, item_start_offset));
            children.push(scrollable_layout);
        }

        let now = Instant::now();
        rail_state.width.set_target(effective_target, now);

        Node::with_children(container_bounds, children)
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
        let mut layouts = layout.children();
        let mut tree_index = 0;
        let container_vertical_padding = self
            .container_vertical_padding
            .unwrap_or(constants::CONTAINER_VERTICAL_PADDING);
        let rail_state = tree.state.downcast_ref::<State>();
        let menu_visible_v = rail_state.menu_visible.value().clamp(0.0, 1.0);

        // Menu
        let menu_layout = layouts.next().unwrap();
        if self.menu_button.is_some() {
            if menu_visible_v > 0.001 {
                let clip = Rectangle {
                    x: menu_layout.bounds().x,
                    y: menu_layout.bounds().y,
                    width: menu_layout.bounds().width,
                    height: menu_layout.bounds().height * menu_visible_v,
                };
                renderer.with_layer(clip, |renderer| {
                    self.menu_button.as_ref().unwrap().as_widget().draw(
                        &tree.children[0],
                        renderer,
                        theme,
                        style,
                        menu_layout,
                        cursor,
                        viewport,
                    );
                });
            }
        }
        tree_index += 1;

        // Fab
        let fab_layout = layouts.next().unwrap();
        if self.fab_element.is_some() {
            self.fab_element.as_ref().unwrap().as_widget().draw(
                &tree.children[1],
                renderer,
                theme,
                style,
                fab_layout,
                cursor,
                viewport,
            );
        }
        tree_index += 1;

        // Items area starts right after the fab (or after the menu if no fab)
        let fab_space = if self.fab_element.is_some() {
            ITEMS_SECTION_SPACE
        } else {
            0.0
        };
        let start_y = fab_layout.bounds().y + fab_layout.bounds().height + fab_space;
        let clip_height =
            (layout.bounds().y + layout.bounds().height - container_vertical_padding - start_y)
                .max(0.0);

        if self.items_container.is_some() {
            let items_clip = Rectangle {
                x: layout.bounds().x,
                y: start_y,
                width: layout.bounds().width,
                height: clip_height,
            };

            let scrollable_layout = layouts.next().unwrap();
            renderer.with_layer(items_clip, |renderer| {
                self.items_container.as_ref().unwrap().as_widget().draw(
                    &tree.children[tree_index],
                    renderer,
                    theme,
                    style,
                    scrollable_layout,
                    cursor,
                    viewport,
                );
            });
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        viewport: &iced::Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let mut tree_index = 0;
        let mut layouts = layout.children();
        let rail_state = tree.state.downcast_ref::<State>();

        // Menu
        let menu_layout = layouts.next().unwrap();
        if self.menu_button.is_some() {
            let menu_visible_v = rail_state.menu_visible.value().clamp(0.0, 1.0);
            if menu_visible_v > 0.001 {
                let visible_menu_bounds = Rectangle {
                    x: menu_layout.bounds().x,
                    y: menu_layout.bounds().y,
                    width: menu_layout.bounds().width,
                    height: menu_layout.bounds().height * menu_visible_v,
                };
                if cursor.is_over(visible_menu_bounds) {
                    let interaction = self
                        .menu_button
                        .as_ref()
                        .unwrap()
                        .as_widget()
                        .mouse_interaction(
                            &tree.children[tree_index],
                            menu_layout,
                            cursor,
                            viewport,
                            renderer,
                        );
                    if interaction != mouse::Interaction::None {
                        return interaction;
                    }
                }
            }
        }
        tree_index += 1;

        // Fab
        let fab_layout = layouts.next().unwrap();
        if self.fab_element.is_some() {
            let interaction = self
                .fab_element
                .as_ref()
                .unwrap()
                .as_widget()
                .mouse_interaction(
                    &tree.children[tree_index],
                    fab_layout,
                    cursor,
                    viewport,
                    renderer,
                );
            if interaction != mouse::Interaction::None {
                return interaction;
            }
        }
        tree_index += 1;

        if let Some(container) = self.items_container.as_ref() {
            let scrollable_layout = layouts.next().unwrap();
            let interaction = container.as_widget().mouse_interaction(
                &tree.children[tree_index],
                scrollable_layout,
                cursor,
                viewport,
                renderer,
            );
            if interaction != mouse::Interaction::None {
                return interaction;
            }
        }

        mouse::Interaction::None
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &iced::Event,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn iced::advanced::Clipboard,
        shell: &mut iced::advanced::Shell<'_, Message>,
        viewport: &iced::Rectangle,
    ) {
        let mut layouts = layout.children();
        let mut tree_index = 0;

        // Menu
        let menu_layout = layouts.next().unwrap();
        let menu_visible_v = tree
            .state
            .downcast_ref::<State>()
            .menu_visible
            .value()
            .clamp(0.0, 1.0);
        if let Some(menu) = &mut self.menu_button {
            if menu_visible_v > 0.001 {
                let visible_menu_bounds = Rectangle {
                    x: menu_layout.bounds().x,
                    y: menu_layout.bounds().y,
                    width: menu_layout.bounds().width,
                    height: menu_layout.bounds().height * menu_visible_v,
                };
                let should_forward = match event {
                    iced::Event::Mouse(_) | iced::Event::Touch(_) => {
                        cursor.is_over(visible_menu_bounds)
                    }
                    _ => true,
                };
                if should_forward {
                    menu.as_widget_mut().update(
                        &mut tree.children[tree_index],
                        event,
                        menu_layout,
                        cursor,
                        renderer,
                        clipboard,
                        shell,
                        viewport,
                    );
                }
            }
        }
        tree_index += 1;

        // Fab
        let fab_layout = layouts.next().unwrap();
        if let Some(fab) = &mut self.fab_element {
            fab.as_widget_mut().update(
                &mut tree.children[tree_index],
                event,
                fab_layout,
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );
        }
        tree_index += 1;

        let state = tree.state.downcast_mut::<State>();

        // Delegate events to scrollable items container
        if let Some(container) = self.items_container.as_mut() {
            let scrollable_layout = layouts.next().unwrap();
            container.as_widget_mut().update(
                &mut tree.children[tree_index],
                event,
                scrollable_layout,
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );
        }

        let now = match event {
            iced::Event::Window(iced::window::Event::RedrawRequested(at)) => *at,
            _ => Instant::now(),
        };

        if let Some(spring) = self.resolve_expand_spring()
            && state.width.spring.model != spring
        {
            state.width.spring.model = spring;
            state.expansion.spring.model = spring;
            state.menu_visible.spring.model = spring;
        }

        state.width.step(now);
        state.expansion.step(now);
        state.menu_visible.step(now);

        if !state.width.is_at_rest()
            || !state.expansion.is_at_rest()
            || !state.menu_visible.is_at_rest()
        {
            shell.request_redraw();
            shell.invalidate_layout();
        }
    }
}

impl<'a, Message, Theme, Renderer> From<NavRail<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: Clone,
    Theme: 'a + iced_widget::scrollable::Catalog,
    <Theme as iced_widget::scrollable::Catalog>::Class<'a>:
        From<iced_widget::scrollable::StyleFn<'a, Theme>>,
    Renderer: 'a
        + iced::advanced::text::Renderer
        + iced::advanced::svg::Renderer
        + iced_widget::renderer::geometry::Renderer,
    Renderer::Font: From<iced::Font>,
{
    fn from(mut value: NavRail<'a, Message, Theme, Renderer>) -> Self {
        value.menu_button = value.on_menu_pressed.as_ref().map(|on_press| {
            let icon = if value.mode.is_expanded() {
                common_icons::MENU_OPEN.clone()
            } else {
                common_icons::MENU.clone()
            };
            let button_size = MENU_ICON_SIZE + MENU_BUTTON_PADDING * 2.0;
            crate::widget::button(
                value.style.menu_button_style,
                button::Content::Icon(Icon::Svg(icon)),
            )
            .size(button::Size {
                width: Length::Fixed(button_size.0),
                height: Length::Fixed(button_size.0),
                spacing: Pixels::ZERO,
                padding: Padding::default(),
                icon_size: MENU_ICON_SIZE,
                label_size: Pixels(0.0),
                corner_radius: button::CornerRadius {
                    style: button::CornerStyle::Rounded,
                    shape_morph: false,
                    rounded: f32::MAX.into(),
                    square: f32::MAX.into(),
                    pressed: f32::MAX.into(),
                },
            })
            .on_press((on_press)(value.mode))
            .into()
        });

        let fab = std::mem::take(&mut value.fab);
        value.fab_element = fab.map(|fab| {
            let content = if value.mode.is_expanded() {
                widget::fab::Content::Extended {
                    icon: fab.icon,
                    label: fab.label,
                }
            } else {
                widget::fab::Content::Reguar { icon: fab.icon }
            };

            widget::fab(fab.style, content, fab.on_press)
                .label_font_maybe(value.label_font)
                .into()
        });

        let items = std::mem::take(&mut value.items);
        let expanded = value.mode.is_expanded();
        value.item_elements = items
            .iter()
            .enumerate()
            .map(|(index, content)| {
                let active = index == value.active_index;
                item::Item::new(
                    value.style.item_style,
                    content.clone(),
                    expanded,
                    value.label_font,
                    active,
                    value.hide_labels_when_collapsed,
                )
                .motion_scheme_maybe(value.motion_scheme)
                .active_transition_spring_maybe(value.item_active_transition_spring)
                .color_spring_maybe(value.item_color_spring)
                .into()
            })
            .collect();

        // Build scrollable container from items
        let scrollbar_color = value
            .scrollbar_color
            .unwrap_or(value.style.item_style.content_inactive);
        let taken_items = std::mem::take(&mut value.item_elements);
        let mut col = iced::widget::Column::new().spacing(0.0).padding(0.0);
        for item in taken_items {
            col = col.push(item);
        }
        let scroller = crate::widget::scrollable::scrollable_with_color(scrollbar_color, col);
        value.items_container = Some(scroller.into());

        // Restore raw items for potential future layout measurements
        value.items = items;

        Element::new(value)
    }
}
