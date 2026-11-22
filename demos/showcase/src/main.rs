use demo_sliders as sliders;
use iced::{
    Alignment, Element, Font, Length, Subscription, Task,
    widget::{Space, column, container, row, text},
};
use iced_m3::{
    theme::{ColorScheme, Mode, Theme},
    widget::{hybrid_icon::Icon, navrail, scrollable, switch},
};
use menu_demo as menu;
use navbar_demo as navbar;
use shapes_demo as shapes;

const APP_NAME: &str = "Material 3 Showcase";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Tab {
    #[default]
    Badge,
    Groups,
    Buttons,
    Cards,
    Checkbox,
    Dialog,
    Fab,
    Inputs,
    Menu,
    Navigation,
    NavRail,
    Progress,
    Shapes,
    Sliders,
    Switch,
}

impl Tab {
    fn index(self) -> usize {
        self as usize
    }

    fn label(self) -> &'static str {
        match self {
            Tab::Badge => "Badge",
            Tab::Groups => "Button groups",
            Tab::Buttons => "Buttons",
            Tab::Cards => "Cards",
            Tab::Checkbox => "Checkbox",
            Tab::Dialog => "Dialog",
            Tab::Fab => "FAB",

            Tab::Inputs => "Inputs",
            Tab::Menu => "Menu",
            Tab::Navigation => "Navigation bar",
            Tab::NavRail => "Navigation rail",
            Tab::Progress => "Progress",
            Tab::Shapes => "Shapes",
            Tab::Sliders => "Sliders",
            Tab::Switch => "Switch",
        }
    }

    fn icon(self) -> &'static str {
        match self {
            Tab::Badge => "notifications",
            Tab::Buttons => "touch_app",
            Tab::Cards => "dashboard",
            Tab::Checkbox => "check_box",
            Tab::Dialog => "chat_bubble",
            Tab::Fab => "add_circle",

            Tab::Groups => "group_work",
            Tab::Inputs => "edit",
            Tab::Menu => "menu",
            Tab::Navigation => "explore",
            Tab::NavRail => "view_sidebar",
            Tab::Progress => "hourglass_empty",
            Tab::Shapes => "interests",
            Tab::Sliders => "tune",
            Tab::Switch => "toggle_on",
        }
    }
}

#[derive(Clone)]
enum Message {
    SwitchTab(Tab),
    ToggleDarkTheme,
    ToggleNavRailExpanded,
    Buttons(buttons::Message),
    Inputs(inputs::Message),
    Sliders(sliders::Message),
    Progress(progress_indicators::Message),
    Cards(card::Message),
    Fab(fab::Message),

    Navigation(navbar::Message),
    Shapes(shapes::Message),
    Dialog(dialog::Message),
    Groups(button_groups::Message),
    Checkbox(checkbox::Message),
    Switch(::switch::Message),
    Menu(menu::Message),
    Badge(badges::Message),
    NavRail(::navrail::Message),
}

struct State {
    theme: Theme,
    tab: Tab,
    navrail_expanded: bool,
    buttons_page: buttons::Page,
    inputs_page: inputs::Page,
    sliders_page: sliders::Page,
    progress_page: progress_indicators::Page,
    card_page: card::Page,
    fab_page: fab::Page,
    navbar_page: navbar::Page,
    dialog_page: dialog::Page,
    groups_page: button_groups::Page,
    checkbox_page: checkbox::Page,
    switch_page: ::switch::Page,
    menu_page: menu::Page,
    badge_page: badges::Page,
    navrail_page: ::navrail::Page,

    shapes_page: shapes::Page,
}

impl Default for State {
    fn default() -> Self {
        Self {
            theme: Theme::default(Mode::Dark),
            tab: Tab::default(),
            navrail_expanded: true,
            buttons_page: buttons::Page::default(),
            inputs_page: inputs::Page::default(),
            sliders_page: sliders::Page::default(),
            progress_page: progress_indicators::Page::default(),
            card_page: card::Page::default(),
            fab_page: fab::Page::default(),
            navbar_page: navbar::Page::default(),
            dialog_page: dialog::Page::default(),
            groups_page: button_groups::Page::default(),
            checkbox_page: checkbox::Page::default(),
            switch_page: ::switch::Page::default(),
            menu_page: menu::Page::default(),
            badge_page: badges::Page::default(),
            navrail_page: ::navrail::Page::default(),

            shapes_page: shapes::Page::default(),
        }
    }
}

impl State {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SwitchTab(tab) => self.tab = tab,
            Message::ToggleDarkTheme => {
                self.theme = Theme::default(match self.theme.mode {
                    Mode::Dark => Mode::Light,
                    Mode::Light => Mode::Dark,
                });
            }
            Message::ToggleNavRailExpanded => self.navrail_expanded = !self.navrail_expanded,
            Message::Buttons(msg) => self.buttons_page.update(msg),
            Message::Inputs(msg) => self.inputs_page.update(msg),
            Message::Sliders(msg) => self.sliders_page.update(msg),
            Message::Progress(msg) => self.progress_page.update(msg),
            Message::Cards(msg) => self.card_page.update(msg),
            Message::Fab(msg) => self.fab_page.update(msg),

            Message::Navigation(msg) => self.navbar_page.update(msg),
            Message::Shapes(msg) => self.shapes_page.update(msg),
            Message::Dialog(msg) => self.dialog_page.update(msg),
            Message::Groups(msg) => self.groups_page.update(msg),
            Message::Checkbox(msg) => self.checkbox_page.update(msg),
            Message::Switch(msg) => self.switch_page.update(msg),
            Message::Menu(msg) => self.menu_page.update(msg),
            Message::Badge(msg) => self.badge_page.update(msg),
            Message::NavRail(msg) => self.navrail_page.update(msg),
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let nav_items: Vec<navrail::Item<Message>> = [
            Tab::Badge,
            Tab::Groups,
            Tab::Buttons,
            Tab::Cards,
            Tab::Checkbox,
            Tab::Dialog,
            Tab::Fab,
            Tab::Inputs,
            Tab::Menu,
            Tab::Navigation,
            Tab::NavRail,
            Tab::Progress,
            Tab::Shapes,
            Tab::Sliders,
            Tab::Switch,
        ]
        .into_iter()
        .map(|tab| navrail::Item {
            icon_active: Icon::Text {
                text: tab.icon().into(),
                font: Some(fonts::icons_filled()),
            },
            icon_inactive: Icon::Text {
                text: tab.icon().into(),
                font: Some(fonts::icons_outlined()),
            },
            badge: None,
            label: tab.label().into(),
            on_press: Message::SwitchTab(tab),
        })
        .collect();

        let navrail_mode = match self.navrail_expanded {
            true => navrail::Mode::ExpandedAuto,
            false => navrail::Mode::Collapsed,
        };

        let navrail_widget =
            iced_m3::widget::navrail(&self.theme, navrail::Style::new(&self.theme), nav_items)
                .active(self.tab.index())
                .mode(navrail_mode)
                .label_font(fonts::text_regular())
                .hide_labels_when_collapsed(false)
                .on_menu_pressed(|_| Message::ToggleNavRailExpanded)
                .menu_visible(true);

        let header = row![
            text(self.tab.label())
                .font(fonts::text_bold())
                .size(24.0)
                .color(self.theme.on_surface()),
            Space::new().width(Length::Fill),
            row![
                text("Dark").color(self.theme.on_surface()).size(14.0),
                switch(&self.theme, self.theme.mode == Mode::Dark)
                    .on_toggle(Message::ToggleDarkTheme),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        ]
        .align_y(Alignment::Center);

        let content: Element<'_, Message> = match self.tab {
            Tab::Badge => self.badge_page.view(&self.theme).map(Message::Badge),
            Tab::Groups => self.groups_page.view(&self.theme).map(Message::Groups),
            Tab::Buttons => self.buttons_page.view(&self.theme).map(Message::Buttons),
            Tab::Cards => self.card_page.view(&self.theme).map(Message::Cards),
            Tab::Checkbox => self.checkbox_page.view(&self.theme).map(Message::Checkbox),
            Tab::Dialog => self.dialog_page.view(&self.theme).map(Message::Dialog),
            Tab::Fab => self.fab_page.view(&self.theme).map(Message::Fab),

            Tab::Inputs => self.inputs_page.view(&self.theme).map(Message::Inputs),
            Tab::Menu => self.menu_page.view(&self.theme).map(Message::Menu),
            Tab::Navigation => self.navbar_page.view(&self.theme).map(Message::Navigation),
            Tab::NavRail => self.navrail_page.view(&self.theme).map(Message::NavRail),
            Tab::Progress => self.progress_page.view(&self.theme).map(Message::Progress),
            Tab::Shapes => self.shapes_page.view(&self.theme).map(Message::Shapes),
            Tab::Sliders => self.sliders_page.view(&self.theme).map(Message::Sliders),
            Tab::Switch => self.switch_page.view(&self.theme).map(Message::Switch),
        };

        let main_area = container(
            column![
                header,
                scrollable(&self.theme, content)
                    .spacing(16)
                    .width(Length::Fill)
            ]
            .spacing(16)
            .padding(16),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_| iced::widget::container::Style::default().background(self.theme.background()));

        row![
            container(navrail_widget).style(|_| {
                iced::widget::container::Style::default().background(self.theme.surface_container())
            }),
            main_area,
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        match self.tab {
            Tab::Progress => self.progress_page.subscription().map(Message::Progress),
            _ => Subscription::none(),
        }
    }
}

fn main() -> iced::Result {
    iced::application(
        || {
            (
                State::default(),
                Task::batch(
                    fonts::get_fonts()
                        .into_iter()
                        .map(|b| iced::font::load(b).discard()),
                ),
            )
        },
        State::update,
        State::view,
    )
    .title(APP_NAME)
    .default_font(Font::with_name("Noto Sans"))
    .subscription(State::subscription)
    .run()
}
