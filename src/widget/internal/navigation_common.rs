use iced::{Color, advanced::text};

use crate::{
    animation::{Interpolable, motion::SpringValue},
    style::StateLayer,
    theme::ColorScheme,
    widget::{self, Badge, hybrid_icon::Icon, ripple::Ripple},
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NavigationItemStyle {
    pub content_active: Color,
    pub label_content_active: Color,
    pub content_inactive: Color,
    pub container_color: Color,
    pub badge_style: widget::badge::Style,
    pub state_layer: StateLayer,
}

impl NavigationItemStyle {
    pub fn new(theme: &(impl ColorScheme + ?Sized)) -> Self {
        Self {
            content_active: theme.secondary(),
            label_content_active: theme.secondary(),
            content_inactive: theme.on_surface_variant(),
            container_color: theme.secondary_container(),
            badge_style: widget::badge::Style::new(theme),
            state_layer: StateLayer::new(theme.on_secondary_container()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct NavigationItemContent<'a, Message>
where
    Message: Clone,
{
    pub icon_active: Icon<'a>,
    pub icon_inactive: Icon<'a>,
    pub badge: Option<Badge<'a>>,
    pub label: text::Fragment<'a>,
    pub on_press: Message,
}

#[derive(Clone, Copy, PartialEq)]
pub struct Colors {
    pub icon: Color,
    pub label: Color,
}

impl Interpolable for Colors {
    fn interpolate(self, other: Self, t: f32) -> Self {
        Self {
            icon: self.icon.interpolate(other.icon, t),
            label: self.label.interpolate(other.label, t),
        }
    }
}

pub struct State {
    pub is_pressed: bool,
    pub ripple: Ripple,
    pub active_transition_spring: SpringValue<f32>,
    pub color_spring: SpringValue<Colors>,
}
