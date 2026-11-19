use iced::{
    Alignment, Element, Font, Length, Subscription, Task,
    widget::{Space, center, column, container, row, text},
};
use iced_m3::{
    theme::{Accent, ColorScheme, Mode, Theme},
    widget::{
        Action, Badge, badge, button, button_group, card, connected_button_group, dialog, fab,
        fab_menu, hybrid_icon::Icon, loading_indicator, menu, navbar, navrail, progress_bar,
        range_slider, scrollable, slider, standard_button_group, text_input::TextInput,
    },
};

const APP_NAME: &str = "Iced M3 Components Showcase";

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
#[repr(usize)]
enum Tab {
    #[default]
    Buttons = 0,
    Inputs = 1,
    Sliders = 2,
    Progress = 3,
    Cards = 4,
    Fab = 5,
    Navigation = 6,
    Dialog = 7,
    Groups = 8,
    Checkbox = 9,
    Switch = 10,
    Menu = 11,
    Badge = 12,
    NavRail = 13,
}

impl Tab {
    fn index(self) -> usize {
        self as usize
    }

    fn label(self) -> &'static str {
        match self {
            Tab::Buttons => "Buttons",
            Tab::Inputs => "Inputs",
            Tab::Sliders => "Sliders",
            Tab::Progress => "Progress",
            Tab::Cards => "Cards",
            Tab::Fab => "FAB",
            Tab::Navigation => "Navigation",
            Tab::Dialog => "Dialog",
            Tab::Groups => "Groups",
            Tab::Checkbox => "Checkbox",
            Tab::Switch => "Switch",
            Tab::Menu => "Menu",
            Tab::Badge => "Badge",
            Tab::NavRail => "NavRail",
        }
    }

    fn icon(self) -> &'static str {
        match self {
            Tab::Buttons => "touch_app",
            Tab::Inputs => "edit",
            Tab::Sliders => "tune",
            Tab::Progress => "hourglass_empty",
            Tab::Cards => "dashboard",
            Tab::Fab => "add_circle",
            Tab::Navigation => "explore",
            Tab::Dialog => "chat_bubble",
            Tab::Groups => "group_work",
            Tab::Checkbox => "check_box",
            Tab::Switch => "toggle_on",
            Tab::Menu => "menu",
            Tab::Badge => "notifications",
            Tab::NavRail => "view_sidebar",
        }
    }
}

#[derive(Clone)]
enum Message {
    SwitchTab(Tab),
    ToggleDarkTheme,
    // Inputs
    ToggleSwitch,
    TextInputChanged(String),
    SecureInputChanged(String),
    TextInputErrorToggle,
    // Sliders
    SliderValueXs(f32),
    SliderValueS(f32),
    SliderValueM(f32),
    SliderValueL(f32),
    SliderValueXl(f32),
    SliderCentered(f32),
    SliderStepped(f32),
    SliderTicks(f32),
    SliderVolume(f32),
    SliderPercent(f32),
    SliderDecimals(f32),
    SliderRange(f32, f32),
    SliderRangeStepped(f32, f32),
    SliderRangeSmooth(f32, f32),
    SliderRangeSeparated(f32, f32),
    SliderRangeOneHandle(f32, f32),
    SliderVertical(f32),
    SliderDisabled,
    // Dialog
    ToggleDialog,
    CloseDialog,
    // Navigation
    NavBarTabChanged(usize),
    // Groups
    SelectGroupIndex(usize),
    ToggleGroupButton(usize),
    // Cards
    CardPressed,
    // Reuse demos
    Checkbox(checkbox::Message),
    SwitchDemo(switch::Message),
    // Progress
    ProgressTick,
    // Text input extras
    TextInputSubmitted,
    TextInputWithIconChanged(String),
    TextInputWithLabelChanged(String),
    TextInputAlignedChanged(String),
    // Button extras
    ToggleButtonSelected,
    // Dialog extras
    ToggleDialogCustom,
    CloseDialogCustom,
    // NavRail
    ToggleNavRailExpanded,
    ToggleNavRailHideLabels,
    ToggleNavRailShowMenu,
    // FAB extras
    ToggleFabCornerStyle,
    // Progress extras
    ToggleProgressMaybe,
    ToggleProgressHeightMaybe,
    // General
    Noop,
}

struct State {
    theme: Theme,
    tab: Tab,
    navrail_expanded: bool,

    // Inputs tab
    switch_value: bool,
    text_input_value: String,
    secure_text_input_value: String,
    text_input_error: bool,

    // Sliders tab
    slider_value_xs: f32,
    slider_value_s: f32,
    slider_value_m: f32,
    slider_value_l: f32,
    slider_value_xl: f32,
    slider_centered: f32,
    slider_stepped: f32,
    slider_ticks: f32,
    slider_volume: f32,
    slider_percent: f32,
    slider_decimals: f32,
    slider_range: (f32, f32),
    slider_range_stepped: (f32, f32),
    slider_range_smooth: (f32, f32),
    slider_range_separated: (f32, f32),
    slider_range_one_handle: (f32, f32),
    slider_vertical: f32,

    // Dialog tab
    dialog_open: bool,

    // Navigation tab
    navbar_index: usize,

    // Groups tab
    group_selected: usize,
    group_buttons: [bool; 3],

    // Cards tab
    card_press_count: u32,

    // Reuse demos
    checkbox_demo: checkbox::Demo,
    switch_demo: switch::Demo,

    // Progress tab
    progress_value: f32,

    // Text input extras
    text_input_submitted: String,
    text_input_with_icon: String,
    text_input_with_label: String,
    text_input_aligned: String,

    // Buttons extras
    button_selected: bool,

    // Dialog extras
    dialog_custom_open: bool,

    // NavRail
    navrail_hide_labels: bool,
    navrail_show_menu: bool,

    // FAB extras
    fab_corner_style: button::CornerStyle,

    // Progress extras
    progress_maybe: Option<f32>,
    progress_height_maybe: Option<f32>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            theme: Theme::default(Mode::Dark),
            tab: Tab::default(),
            navrail_expanded: false,
            switch_value: false,
            text_input_value: String::new(),
            secure_text_input_value: String::new(),
            text_input_error: false,
            slider_value_xs: 20.0,
            slider_value_s: 40.0,
            slider_value_m: 60.0,
            slider_value_l: 80.0,
            slider_value_xl: 65.0,
            slider_centered: 0.0,
            slider_stepped: 50.0,
            slider_ticks: 30.0,
            slider_volume: 70.0,
            slider_percent: 42.0,
            slider_decimals: 42.25,
            slider_range: (25.0, 75.0),
            slider_range_stepped: (25.0, 75.0),
            slider_range_smooth: (30.0, 70.0),
            slider_range_separated: (20.0, 70.0),
            slider_range_one_handle: (20.0, 60.0),
            slider_vertical: 40.0,
            dialog_open: false,
            navbar_index: 0,
            group_selected: 0,
            group_buttons: [false, false, false],
            card_press_count: 0,
            checkbox_demo: checkbox::Demo::default(),
            switch_demo: switch::Demo::default(),
            progress_value: 0.0,
            text_input_submitted: String::new(),
            text_input_with_icon: String::new(),
            text_input_with_label: String::new(),
            text_input_aligned: String::new(),
            button_selected: false,
            dialog_custom_open: false,
            navrail_hide_labels: false,
            navrail_show_menu: true,
            fab_corner_style: button::CornerStyle::Rounded,
            progress_maybe: Some(0.5),
            progress_height_maybe: Some(12.0),
        }
    }
}

fn section_header<'a>(title: &'a str, theme: &Theme) -> text::Text<'a> {
    text(title)
        .font(fonts::text_bold())
        .size(18.0)
        .color(theme.on_surface())
}

impl State {
    fn view(&self) -> Element<'_, Message> {
        let nav_items: Vec<navrail::Item<Message>> = [
            Tab::Buttons,
            Tab::Inputs,
            Tab::Sliders,
            Tab::Progress,
            Tab::Cards,
            Tab::Fab,
            Tab::Navigation,
            Tab::Dialog,
            Tab::Groups,
            Tab::Checkbox,
            Tab::Switch,
            Tab::Menu,
            Tab::Badge,
            Tab::NavRail,
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

        let navrail_widget = navrail(navrail::Style::new(&self.theme), nav_items)
            .active(self.tab.index())
            .mode(navrail_mode)
            .label_font(fonts::text_regular())
            .hide_labels_when_collapsed(self.navrail_hide_labels)
            .on_menu_pressed(|_| Message::ToggleNavRailExpanded)
            .menu_visible(self.navrail_show_menu);

        let header = row![
            text(self.tab.label())
                .font(fonts::text_bold())
                .size(24.0)
                .color(self.theme.on_surface()),
            Space::new().width(Length::Fill),
            row![
                text("Dark").color(self.theme.on_surface()).size(14.0),
                iced_m3::widget::switch(&self.theme, self.theme.mode == Mode::Dark)
                    .on_toggle(Message::ToggleDarkTheme),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        ]
        .align_y(Alignment::Center);

        let content: Element<'_, Message> = match self.tab {
            Tab::Buttons => self.view_buttons(),
            Tab::Inputs => self.view_inputs(),
            Tab::Sliders => self.view_sliders(),
            Tab::Progress => self.view_progress(),
            Tab::Cards => self.view_cards(),
            Tab::Fab => self.view_fab(),
            Tab::Navigation => self.view_navigation(),
            Tab::Dialog => self.view_dialog(),
            Tab::Groups => self.view_groups(),
            Tab::Checkbox => self.view_checkbox(),
            Tab::Switch => self.view_switch(),
            Tab::Menu => self.view_menu(),
            Tab::Badge => self.view_badge(),
            Tab::NavRail => self.view_navrail(),
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

    fn view_buttons(&self) -> Element<'_, Message> {
        let add_icon = Icon::Text {
            text: "add".into(),
            font: Some(fonts::icons_filled()),
        };
        let btn = |style: button::Style, label: &'static str| {
            iced_m3::widget::button(style, button::Content::Label(label.into()))
                .on_press(Message::Noop)
        };
        let icon_btn = |style: button::Style,
                        icon: Icon<'static>|
         -> iced_m3::widget::button::Button<'static, Message> {
            iced_m3::widget::button(style, button::Content::Icon(icon)).on_press(Message::Noop)
        };
        let full_btn = |style: button::Style,
                        label: &'static str,
                        icon: Icon<'static>|
         -> iced_m3::widget::button::Button<'static, Message> {
            iced_m3::widget::button(
                style,
                button::Content::Full {
                    icon,
                    label: label.into(),
                },
            )
            .on_press(Message::Noop)
        };

        column![
            section_header("Filled buttons", &self.theme),
            row![
                btn(
                    button::Style::filled(&self.theme, Accent::Primary),
                    "Primary"
                ),
                btn(
                    button::Style::filled(&self.theme, Accent::Secondary),
                    "Secondary"
                ),
                btn(
                    button::Style::filled(&self.theme, Accent::Tertiary),
                    "Tertiary"
                ),
            ]
            .spacing(8),
            section_header("Elevated buttons", &self.theme),
            row![
                btn(
                    button::Style::elevated(&self.theme, Accent::Primary),
                    "Primary"
                ),
                btn(
                    button::Style::elevated(&self.theme, Accent::Secondary),
                    "Secondary"
                ),
                btn(
                    button::Style::elevated(&self.theme, Accent::Tertiary),
                    "Tertiary"
                ),
            ]
            .spacing(8),
            section_header("Outlined buttons", &self.theme),
            row![
                btn(button::Style::outlined(&self.theme), "Primary"),
                btn(button::Style::outlined(&self.theme), "Secondary"),
                btn(button::Style::outlined(&self.theme), "Tertiary"),
            ]
            .spacing(8),
            section_header("Text buttons", &self.theme),
            row![
                btn(button::Style::text(&self.theme, Accent::Primary), "Primary"),
                btn(
                    button::Style::text(&self.theme, Accent::Secondary),
                    "Secondary"
                ),
                btn(
                    button::Style::text(&self.theme, Accent::Tertiary),
                    "Tertiary"
                ),
            ]
            .spacing(8),
            section_header("Disabled buttons", &self.theme),
            row![
                iced_m3::widget::button(
                    button::Style::filled(&self.theme, Accent::Primary),
                    button::Content::Label("Filled".into())
                ),
                iced_m3::widget::button(
                    button::Style::elevated(&self.theme, Accent::Primary),
                    button::Content::Label("Elevated".into())
                ),
                iced_m3::widget::button(
                    button::Style::outlined(&self.theme),
                    button::Content::Label("Outlined".into())
                ),
                iced_m3::widget::button(
                    button::Style::text(&self.theme, Accent::Primary),
                    button::Content::Label("Text".into())
                ),
            ]
            .spacing(8),
            section_header("Icon buttons", &self.theme),
            row![
                icon_btn(
                    button::Style::filled(&self.theme, Accent::Primary),
                    add_icon.clone()
                ),
                icon_btn(
                    button::Style::elevated(&self.theme, Accent::Primary),
                    add_icon.clone()
                ),
                icon_btn(button::Style::outlined(&self.theme), add_icon.clone()),
                icon_btn(
                    button::Style::text(&self.theme, Accent::Primary),
                    add_icon.clone()
                ),
            ]
            .spacing(8),
            section_header("Full buttons (icon + label)", &self.theme),
            row![
                full_btn(
                    button::Style::filled(&self.theme, Accent::Primary),
                    "Add",
                    add_icon.clone()
                ),
                full_btn(
                    button::Style::elevated(&self.theme, Accent::Primary),
                    "Add",
                    add_icon.clone()
                ),
                full_btn(
                    button::Style::outlined(&self.theme),
                    "Add",
                    add_icon.clone()
                ),
                full_btn(
                    button::Style::text(&self.theme, Accent::Primary),
                    "Add",
                    add_icon
                ),
            ]
            .spacing(8),
            section_header("Button sizes", &self.theme),
            row![
                btn(button::Style::filled(&self.theme, Accent::Primary), "XS")
                    .size(button::Size::extra_small()),
                btn(button::Style::filled(&self.theme, Accent::Primary), "S")
                    .size(button::Size::small()),
                btn(button::Style::filled(&self.theme, Accent::Primary), "M")
                    .size(button::Size::medium()),
                btn(button::Style::filled(&self.theme, Accent::Primary), "L")
                    .size(button::Size::large()),
                btn(button::Style::filled(&self.theme, Accent::Primary), "XL")
                    .size(button::Size::extra_large()),
            ]
            .spacing(8),
            section_header("Selected button", &self.theme),
            row![
                iced_m3::widget::button(
                    if self.button_selected {
                        button::Style::filled(&self.theme, Accent::Primary)
                    } else {
                        button::Style::outlined(&self.theme)
                    },
                    button::Content::Label("Toggle selection".into()),
                )
                .on_press(Message::ToggleButtonSelected)
                .selected(self.button_selected),
            ]
            .spacing(8),
            section_header("Buttons with corner styles", &self.theme),
            row![
                iced_m3::widget::button(
                    button::Style::filled(&self.theme, Accent::Primary),
                    button::Content::Label("Rounded".into()),
                )
                .on_press(Message::Noop),
                iced_m3::widget::button(
                    button::Style::filled(&self.theme, Accent::Primary),
                    button::Content::Label("Square".into()),
                )
                .on_press(Message::Noop)
                .size(button::Size {
                    corner_radius: button::CornerRadius::default()
                        .style(button::CornerStyle::Square),
                    ..button::Size::default()
                }),
                iced_m3::widget::button(
                    button::Style::elevated(&self.theme, Accent::default()),
                    button::Content::Label("Toggle style".into()),
                )
                .on_press(Message::ToggleFabCornerStyle),
                text(format!("{:?}", self.fab_corner_style)).color(self.theme.on_surface_variant()),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        ]
        .spacing(24)
        .into()
    }

    fn view_inputs(&self) -> Element<'_, Message> {
        column![
            section_header("Switch", &self.theme),
            row![
                text("Toggle me").color(self.theme.on_surface()),
                iced_m3::widget::switch(&self.theme, self.switch_value)
                    .on_toggle(Message::ToggleSwitch),
            ]
            .spacing(12)
            .align_y(Alignment::Center),
            section_header("Text Input", &self.theme),
            TextInput::new("Type here...", &self.text_input_value, &self.theme)
                .on_input(Message::TextInputChanged)
                .width(300.0),
            section_header("Secure Input", &self.theme),
            TextInput::new("Password...", &self.secure_text_input_value, &self.theme)
                .on_input(Message::SecureInputChanged)
                .secure(true)
                .width(300.0),
            section_header("Error Input", &self.theme),
            row![
                TextInput::new("Type to trigger...", &self.text_input_value, &self.theme)
                    .on_input(Message::TextInputChanged)
                    .error(self.text_input_error)
                    .width(300.0),
                iced_m3::widget::button(
                    button::Style::filled(&self.theme, Accent::Primary),
                    button::Content::Label("Toggle Error".into()),
                )
                .on_press(Message::TextInputErrorToggle),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
            section_header("Text input with icon", &self.theme),
            TextInput::new("Search...", &self.text_input_with_icon, &self.theme)
                .on_input(Message::TextInputWithIconChanged)
                .icon(iced_m3::widget::text_input::Icon {
                    font: fonts::icons_filled(),
                    code_point: '\u{e8b6}',
                    size: None,
                    spacing: 8.0,
                    side: iced_m3::widget::text_input::Side::Left,
                })
                .width(300.0),
            section_header("Text input with label", &self.theme),
            TextInput::new(
                "Labelled input...",
                &self.text_input_with_label,
                &self.theme
            )
            .on_input(Message::TextInputWithLabelChanged)
            .with_label_text("Label", self.theme.primary_container())
            .width(300.0),
            section_header("Text input on submit", &self.theme),
            TextInput::new("Press Enter...", &self.text_input_value, &self.theme)
                .on_input(Message::TextInputChanged)
                .on_submit(Message::TextInputSubmitted)
                .width(300.0),
            text(format!("Submitted: {}", self.text_input_submitted))
                .color(self.theme.on_surface()),
            section_header("Text input aligned", &self.theme),
            TextInput::new("Centered...", &self.text_input_aligned, &self.theme)
                .on_input(Message::TextInputAlignedChanged)
                .align_x(iced::alignment::Horizontal::Center)
                .size(18.0)
                .line_height(1.5)
                .padding(16.0)
                .width(300.0),
        ]
        .spacing(24)
        .into()
    }

    fn slider_section<'a>(&'a self, title: &'a str) -> Element<'a, Message> {
        text(title)
            .font(fonts::text_bold())
            .size(16.0)
            .color(self.theme.on_surface())
            .into()
    }

    fn labeled_slider<'a>(
        &'a self,
        label: &'a str,
        value: f32,
        on_change: impl 'a + Fn(f32) -> Message,
        style: slider::Style,
        size: slider::Size,
    ) -> iced::widget::Row<'a, Message> {
        row![
            text(label).width(40).color(self.theme.on_surface_variant()),
            slider(0.0..=100.0, value, on_change, style)
                .value_indicator(slider::ValueIndicator::Value)
                .stops(false)
                .size(size),
        ]
        .spacing(16)
        .align_y(Alignment::Center)
    }

    fn view_sliders(&self) -> Element<'_, Message> {
        let style = slider::Style::new(&self.theme);

        let sizes = column![
            self.labeled_slider(
                "XS",
                self.slider_value_xs,
                Message::SliderValueXs,
                style,
                slider::Size::extra_small(),
            ),
            self.labeled_slider(
                "S",
                self.slider_value_s,
                Message::SliderValueS,
                style,
                slider::Size::small(),
            ),
            self.labeled_slider(
                "M",
                self.slider_value_m,
                Message::SliderValueM,
                style,
                slider::Size::medium(),
            ),
            self.labeled_slider(
                "L",
                self.slider_value_l,
                Message::SliderValueL,
                style,
                slider::Size::large(),
            ),
            self.labeled_slider(
                "XL",
                self.slider_value_xl,
                Message::SliderValueXl,
                style,
                slider::Size::extra_large(),
            ),
        ]
        .spacing(20);

        let centered = slider(
            -100.0..=100.0,
            self.slider_centered,
            Message::SliderCentered,
            style,
        )
        .centered(true)
        .step(0.0_f32)
        .value_indicator(slider::ValueIndicator::Value);

        let steps = column![
            slider(
                0.0..=100.0,
                self.slider_stepped,
                Message::SliderStepped,
                style
            )
            .step(25.0_f32)
            .continuous_tick_count(4)
            .value_indicator(slider::ValueIndicator::Value),
            slider(0.0..=100.0, self.slider_ticks, Message::SliderTicks, style)
                .step(0.0_f32)
                .continuous_tick_count(8)
                .value_indicator(slider::ValueIndicator::Value),
            slider(
                0.0..=100.0,
                self.slider_volume,
                Message::SliderVolume,
                style
            )
            .stops(false)
            .inset_icon_start(slider::InsetIcon::dual(
                Icon::Text {
                    text: "volume_down".into(),
                    font: Some(fonts::icons_filled()),
                },
                Icon::Text {
                    text: "volume_up".into(),
                    font: Some(fonts::icons_filled()),
                },
            ))
            .inset_icon_end(slider::InsetIcon::new(Icon::Text {
                text: "volume_up".into(),
                font: Some(fonts::icons_filled()),
            }))
            .size(slider::Size::medium())
            .value_indicator(slider::ValueIndicator::Value),
        ]
        .spacing(20);

        let percent = slider(
            0.0..=100.0,
            self.slider_percent,
            Message::SliderPercent,
            style,
        )
        .step(0.1_f32)
        .value_indicator(slider::ValueIndicator::Format(Box::new(|value: f32| {
            format!("{value:.1}%")
        })));

        let decimals = slider(
            0.0..=100.0,
            self.slider_decimals,
            Message::SliderDecimals,
            style,
        )
        .step(0.25_f32)
        .decimal_places(2)
        .value_indicator(slider::ValueIndicator::Value);

        let disabled = slider(0.0..=100.0, 70.0, |_| Message::SliderDisabled, style)
            .on_change_maybe(None as Option<fn(f32) -> Message>)
            .value_indicator(slider::ValueIndicator::Value);

        let ranges = column![
            range_slider(
                0.0..=100.0,
                self.slider_range.0,
                self.slider_range.1,
                Message::SliderRange,
                style,
            )
            .value_indicator(slider::ValueIndicator::Value),
            range_slider(
                0.0..=100.0,
                self.slider_range_stepped.0,
                self.slider_range_stepped.1,
                Message::SliderRangeStepped,
                style,
            )
            .step(25.0_f32)
            .value_indicator(slider::ValueIndicator::Value),
            range_slider(
                0.0..=100.0,
                self.slider_range_smooth.0,
                self.slider_range_smooth.1,
                Message::SliderRangeSmooth,
                style,
            )
            .step(1.0_f32)
            .value_indicator(slider::ValueIndicator::Value),
            range_slider(
                0.0..=100.0,
                self.slider_range_separated.0,
                self.slider_range_separated.1,
                Message::SliderRangeSeparated,
                style,
            )
            .min_separation(20.0_f32)
            .value_indicator(slider::ValueIndicator::Value),
        ]
        .spacing(20);

        let range_one_handle = range_slider(
            0.0..=100.0,
            self.slider_range_one_handle.0,
            self.slider_range_one_handle.1,
            Message::SliderRangeOneHandle,
            style,
        )
        .lower_enabled(false)
        .value_indicator(slider::ValueIndicator::Value);

        let vertical = row![
            slider(
                0.0..=100.0,
                self.slider_vertical,
                Message::SliderVertical,
                style
            )
            .vertical()
            .height(300)
            .stops(false)
            .value_indicator(slider::ValueIndicator::Value),
            range_slider(
                0.0..=100.0,
                self.slider_range.0,
                self.slider_range.1,
                Message::SliderRange,
                style,
            )
            .vertical()
            .height(300)
            .stops(false)
            .value_indicator(slider::ValueIndicator::Value),
        ]
        .spacing(24)
        .align_y(Alignment::End);

        column![
            self.slider_section("Sizes"),
            sizes,
            self.slider_section("Centered (the active track grows from the middle)"),
            centered,
            self.slider_section("Steps, continuous ticks and inset icons"),
            steps,
            self.slider_section("Formatted value indicator"),
            percent,
            self.slider_section("Value precision (step 0.25, two decimal places)"),
            decimals,
            self.slider_section("Disabled"),
            disabled,
            self.slider_section("Range sliders (the handles can pass each other)"),
            ranges,
            self.slider_section("Range slider with a disabled lower handle"),
            range_one_handle,
            self.slider_section("Vertical sliders"),
            vertical,
        ]
        .spacing(24)
        .into()
    }

    fn view_progress(&self) -> Element<'_, Message> {
        column![
            section_header("Linear progress bars", &self.theme),
            row![
                progress_bar(progress_bar::Style::new(&self.theme)).progress(0.0),
                progress_bar(progress_bar::Style::new(&self.theme)).progress(0.25),
                progress_bar(progress_bar::Style::new(&self.theme)).progress(0.5),
                progress_bar(progress_bar::Style::new(&self.theme)).progress(0.75),
                progress_bar(progress_bar::Style::new(&self.theme)).progress(1.0),
            ]
            .spacing(8),
            section_header("Animated progress", &self.theme),
            progress_bar(progress_bar::Style::new(&self.theme)).progress(self.progress_value),
            section_header("With custom height", &self.theme),
            progress_bar(progress_bar::Style::new(&self.theme))
                .progress(0.6)
                .height(24.0),
            section_header("Indeterminate", &self.theme),
            progress_bar(progress_bar::Style::new(&self.theme)),
            progress_bar(progress_bar::Style::new(&self.theme)).height(16.0),
            section_header("Circular loading indicators", &self.theme),
            row![
                loading_indicator(loading_indicator::Style::contained(&self.theme)),
                loading_indicator(loading_indicator::Style::uncontained(&self.theme)),
            ]
            .spacing(16)
            .align_y(Alignment::Center),
            section_header("Maybe value", &self.theme),
            row![
                progress_bar(progress_bar::Style::new(&self.theme))
                    .progress_maybe(self.progress_maybe),
                iced_m3::widget::button(
                    button::Style::elevated(&self.theme, Accent::default()),
                    button::Content::Label("Toggle value".into())
                )
                .on_press(Message::ToggleProgressMaybe),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
            section_header("Maybe height", &self.theme),
            row![
                progress_bar(progress_bar::Style::new(&self.theme))
                    .progress(0.6)
                    .height_maybe(self.progress_height_maybe),
                iced_m3::widget::button(
                    button::Style::elevated(&self.theme, Accent::default()),
                    button::Content::Label("Toggle height".into())
                )
                .on_press(Message::ToggleProgressHeightMaybe),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        ]
        .spacing(24)
        .into()
    }

    fn view_cards(&self) -> Element<'_, Message> {
        let card_width = 250.0;
        let card_height = 150.0;

        column![
            section_header("Elevated card", &self.theme),
            iced_m3::widget::card(
                card::Style::elevated(&self.theme),
                center(text("Elevated").size(24.0).color(self.theme.on_surface())),
            )
            .interaction(card::Interaction::Press(Action::Direct(
                Message::CardPressed
            )))
            .width(card_width)
            .height(card_height),
            section_header("Filled card", &self.theme),
            iced_m3::widget::card(
                card::Style::filled(&self.theme),
                center(text("Filled").size(24.0).color(self.theme.on_surface())),
            )
            .width(card_width)
            .height(card_height),
            section_header("Outlined card", &self.theme),
            iced_m3::widget::card(
                card::Style::outlined(&self.theme),
                center(text("Outlined").size(24.0).color(self.theme.on_surface())),
            )
            .width(card_width)
            .height(card_height),
            section_header("Custom card", &self.theme),
            iced_m3::widget::card(
                card::Style {
                    container_color: self.theme.tertiary_container(),
                    state_layer: iced_m3::style::StateLayer::new(
                        self.theme.on_tertiary_container()
                    ),
                    ..card::Style::filled(&self.theme)
                },
                center(
                    text("Custom")
                        .size(24.0)
                        .color(self.theme.on_tertiary_container())
                ),
            )
            .width(card_width)
            .height(card_height),
            section_header("Disabled card", &self.theme),
            iced_m3::widget::card(
                card::Style::filled(&self.theme),
                center(text("Disabled").size(24.0).color(self.theme.on_surface())),
            )
            .interaction(card::Interaction::Disabled)
            .width(card_width)
            .height(card_height),
            section_header("Card press count", &self.theme),
            text(format!("Pressed {} times", self.card_press_count)).color(self.theme.on_surface()),
        ]
        .spacing(24)
        .into()
    }

    fn view_fab(&self) -> Element<'_, Message> {
        let add_icon = Icon::Text {
            text: "add".into(),
            font: Some(fonts::icons_filled()),
        };
        let edit_icon = Icon::Text {
            text: "edit".into(),
            font: Some(fonts::icons_filled()),
        };

        column![
            section_header("FAB sizes", &self.theme),
            row![
                fab(
                    fab::Style::fab_tonal(&self.theme, Accent::Secondary),
                    fab::Content::Reguar {
                        icon: add_icon.clone()
                    },
                    Action::Direct(Message::Noop),
                )
                .size(fab::Size::Regular),
                fab(
                    fab::Style::fab_tonal(&self.theme, Accent::Secondary),
                    fab::Content::Reguar {
                        icon: add_icon.clone()
                    },
                    Action::Direct(Message::Noop),
                )
                .size(fab::Size::Medium),
                fab(
                    fab::Style::fab_tonal(&self.theme, Accent::Secondary),
                    fab::Content::Reguar {
                        icon: add_icon.clone()
                    },
                    Action::Direct(Message::Noop),
                )
                .size(fab::Size::Large),
            ]
            .spacing(16)
            .align_y(Alignment::Center),
            section_header("FAB styles", &self.theme),
            row![
                fab(
                    fab::Style::fab_vibrant(&self.theme, Accent::Primary),
                    fab::Content::Reguar {
                        icon: add_icon.clone()
                    },
                    Action::Direct(Message::Noop),
                ),
                fab(
                    fab::Style::fab_tonal(&self.theme, Accent::Secondary),
                    fab::Content::Reguar {
                        icon: add_icon.clone()
                    },
                    Action::Direct(Message::Noop),
                ),
                fab(
                    fab::Style::fab_vibrant(&self.theme, Accent::Tertiary),
                    fab::Content::Reguar {
                        icon: add_icon.clone()
                    },
                    Action::Direct(Message::Noop),
                ),
            ]
            .spacing(16)
            .align_y(Alignment::Center),
            section_header("Extended FAB", &self.theme),
            row![
                fab(
                    fab::Style::fab_vibrant(&self.theme, Accent::Primary),
                    fab::Content::Extended {
                        icon: add_icon.clone(),
                        label: "Create".into(),
                    },
                    Action::Direct(Message::Noop),
                ),
                fab(
                    fab::Style::fab_tonal(&self.theme, Accent::Secondary),
                    fab::Content::Extended {
                        icon: edit_icon,
                        label: "Edit".into(),
                    },
                    Action::Direct(Message::Noop),
                ),
            ]
            .spacing(16)
            .align_y(Alignment::Center),
            section_header("FAB Menu", &self.theme),
            fab_menu(
                fab_menu::Style::new(&self.theme, Accent::default()),
                vec![
                    fab_menu::Item {
                        icon: Icon::Text {
                            text: "email".into(),
                            font: Some(fonts::icons_filled()),
                        },
                        label: "Email".into(),
                        on_press: Action::Direct(Message::Noop),
                    },
                    fab_menu::Item {
                        icon: Icon::Text {
                            text: "chat".into(),
                            font: Some(fonts::icons_filled()),
                        },
                        label: "Chat".into(),
                        on_press: Action::Direct(Message::Noop),
                    },
                    fab_menu::Item {
                        icon: Icon::Text {
                            text: "call".into(),
                            font: Some(fonts::icons_filled()),
                        },
                        label: "Call".into(),
                        on_press: Action::Direct(Message::Noop),
                    },
                ],
                None,
            ),
            section_header("FAB corner styles", &self.theme),
            row![
                fab(
                    fab::Style::fab_vibrant(&self.theme, Accent::Primary),
                    fab::Content::Reguar {
                        icon: add_icon.clone()
                    },
                    Action::Direct(Message::Noop),
                )
                .corner_style(self.fab_corner_style),
                fab(
                    fab::Style::fab_tonal(&self.theme, Accent::Secondary),
                    fab::Content::Reguar {
                        icon: add_icon.clone()
                    },
                    Action::Direct(Message::Noop),
                )
                .corner_style(self.fab_corner_style),
                iced_m3::widget::button(
                    button::Style::elevated(&self.theme, Accent::default()),
                    button::Content::Label("Toggle".into()),
                )
                .on_press(Message::ToggleFabCornerStyle),
                text(format!("{:?}", self.fab_corner_style)).color(self.theme.on_surface_variant()),
            ]
            .spacing(16)
            .align_y(Alignment::Center),
        ]
        .spacing(24)
        .into()
    }

    fn view_navigation(&self) -> Element<'_, Message> {
        let navbar_items = vec![
            navbar::Item {
                icon_active: Icon::Text {
                    text: "star".into(),
                    font: Some(fonts::icons_filled()),
                },
                icon_inactive: Icon::Text {
                    text: "star".into(),
                    font: Some(fonts::icons_outlined()),
                },
                badge: None,
                label: "Star".into(),
                on_press: Message::NavBarTabChanged(0),
            },
            navbar::Item {
                icon_active: Icon::Text {
                    text: "favorite".into(),
                    font: Some(fonts::icons_filled()),
                },
                icon_inactive: Icon::Text {
                    text: "favorite".into(),
                    font: Some(fonts::icons_outlined()),
                },
                badge: Some(Badge { label: None }),
                label: "Fav".into(),
                on_press: Message::NavBarTabChanged(1),
            },
            navbar::Item {
                icon_active: Icon::Text {
                    text: "search".into(),
                    font: Some(fonts::icons_filled()),
                },
                icon_inactive: Icon::Text {
                    text: "search".into(),
                    font: Some(fonts::icons_outlined()),
                },
                badge: Some(Badge {
                    label: Some("3".into()),
                }),
                label: "Search".into(),
                on_press: Message::NavBarTabChanged(2),
            },
        ];

        column![
            section_header("Compact", &self.theme),
            navbar(navbar::Style::new(&self.theme), navbar_items.clone())
                .active_index(self.navbar_index)
                .mode(navbar::Mode::Compact),
            section_header("Large", &self.theme),
            navbar(navbar::Style::new(&self.theme), navbar_items.clone())
                .active_index(self.navbar_index)
                .mode(navbar::Mode::Large),
            section_header("Auto", &self.theme),
            navbar(navbar::Style::new(&self.theme), navbar_items.clone())
                .active_index(self.navbar_index)
                .mode(navbar::Mode::Auto),
            section_header("NavBar expressive motion", &self.theme),
            navbar(navbar::Style::new(&self.theme), navbar_items.clone())
                .active_index(self.navbar_index)
                .mode(navbar::Mode::Compact)
                .motion_scheme(iced_m3::animation::motion::Scheme::Expressive),
            section_header("NavBar custom springs", &self.theme),
            navbar(navbar::Style::new(&self.theme), navbar_items.clone())
                .active_index(self.navbar_index)
                .mode(navbar::Mode::Compact)
                .motion_scheme(iced_m3::animation::motion::Scheme::Expressive)
                .effects_spring(iced_m3::animation::motion::Spring {
                    damping: 0.5,
                    stiffness: 200.0
                })
                .spatial_spring(iced_m3::animation::motion::Spring {
                    damping: 0.7,
                    stiffness: 150.0
                }),
        ]
        .spacing(24)
        .into()
    }

    fn view_navrail(&self) -> Element<'_, Message> {
        column![
            section_header("Main Navigation Rail Settings", &self.theme),
            row![
                text("Hide labels when collapsed").color(self.theme.on_surface()),
                iced_m3::widget::switch(&self.theme, self.navrail_hide_labels)
                    .on_toggle(Message::ToggleNavRailHideLabels),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
            row![
                text("Show menu button").color(self.theme.on_surface()),
                iced_m3::widget::switch(&self.theme, self.navrail_show_menu)
                    .on_toggle(Message::ToggleNavRailShowMenu),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
            iced_m3::widget::button(
                button::Style::filled(&self.theme, Accent::Primary),
                button::Content::Label("Toggle expanded".into()),
            )
            .on_press(Message::ToggleNavRailExpanded),
            text("These settings apply to the main navigation rail on the left.")
                .color(self.theme.on_surface_variant())
                .size(12),
        ]
        .spacing(16)
        .into()
    }

    fn view_dialog(&self) -> Element<'_, Message> {
        let dialog_element = if self.dialog_open {
            dialog::Dialog::new(
                dialog::Style::new(&self.theme),
                text("This is a simple dialog with standard actions.")
                    .color(self.theme.on_surface()),
                vec![
                    dialog::Button {
                        on_press: Some(Message::CloseDialog),
                        label: "Cancel".into(),
                        style: button::Style::text(&self.theme, Accent::Primary),
                    },
                    dialog::Button {
                        on_press: Some(Message::CloseDialog),
                        label: "OK".into(),
                        style: button::Style::filled(&self.theme, Accent::Primary),
                    },
                ],
            )
            .title("Simple Dialog")
            .on_press_outside(Message::CloseDialog)
            .into()
        } else {
            Element::from(Space::new().width(Length::Shrink).height(Length::Shrink))
        };

        let custom_dialog_element = if self.dialog_custom_open {
            dialog::Dialog::new(
                dialog::Style::new(&self.theme),
                text("You can put any widget here.").color(self.theme.on_surface_variant()),
                vec![dialog::Button {
                    on_press: Some(Message::CloseDialogCustom),
                    label: "Dismiss".into(),
                    style: button::Style::text(&self.theme, Accent::Primary),
                }],
            )
            .title("Custom Dialog")
            .on_press_outside(Message::CloseDialogCustom)
            .into()
        } else {
            Element::from(Space::new().width(Length::Shrink).height(Length::Shrink))
        };

        column![
            section_header("Simple dialog", &self.theme),
            iced_m3::widget::button(
                button::Style::filled(&self.theme, Accent::Primary),
                button::Content::Label("Open dialog".into()),
            )
            .on_press(Message::ToggleDialog),
            section_header("Custom content dialog", &self.theme),
            iced_m3::widget::button(
                button::Style::filled(&self.theme, Accent::Primary),
                button::Content::Label("Open custom dialog".into()),
            )
            .on_press(Message::ToggleDialogCustom),
            dialog_element,
            custom_dialog_element,
        ]
        .spacing(24)
        .into()
    }

    fn view_groups(&self) -> Element<'_, Message> {
        let single_select = connected_button_group(
            button::Style::filled(&self.theme, Accent::Primary),
            (0..4)
                .map(|i| button_group::connected::Button {
                    content: button::Content::Label(format!("Option {}", i + 1).into()),
                    on_press: Some(Message::SelectGroupIndex(i)),
                    selected: self.group_selected == i,
                })
                .collect(),
        );

        let items = self.group_buttons.iter().enumerate().map(|(i, &selected)| {
            let btn = iced_m3::widget::button(
                button::Style::filled(&self.theme, Accent::Secondary),
                button::Content::Label(format!("Toggle {}", i + 1).into()),
            )
            .on_press(Message::ToggleGroupButton(i))
            .selected(selected);
            button_group::standard::GroupItem::new(btn, true)
        });
        let multi_select =
            standard_button_group(items).spacing(button_group::standard::SPACING_MEDIUM);

        column![
            section_header("Connected button group (single select)", &self.theme),
            single_select,
            text(format!("Selected index: {}", self.group_selected)).color(self.theme.on_surface()),
            section_header("Standard button group (multi select)", &self.theme),
            multi_select,
            text(format!("Selected: {:?}", self.group_buttons)).color(self.theme.on_surface()),
        ]
        .spacing(24)
        .into()
    }

    fn view_checkbox(&self) -> Element<'_, Message> {
        self.checkbox_demo.view(&self.theme).map(Message::Checkbox)
    }

    fn view_switch(&self) -> Element<'_, Message> {
        self.switch_demo.view(&self.theme).map(Message::SwitchDemo)
    }

    fn view_menu(&self) -> Element<'_, Message> {
        let groups = vec![menu::Group::new(vec![
            menu::Item::Button(menu::button::Button {
                icon: Some(Icon::Text {
                    text: "content_copy".into(),
                    font: Some(fonts::icons_filled()),
                }),
                label: "Copy".into(),
                supporting_text: Some("Ctrl+C".into()),
                action: menu::button::MenuAction::Message {
                    is_selected: false,
                    on_press: Some(Action::Direct(Message::Noop)),
                },
            }),
            menu::Item::Button(menu::button::Button {
                icon: Some(Icon::Text {
                    text: "content_cut".into(),
                    font: Some(fonts::icons_filled()),
                }),
                label: "Cut".into(),
                supporting_text: Some("Ctrl+X".into()),
                action: menu::button::MenuAction::Message {
                    is_selected: false,
                    on_press: Some(Action::Direct(Message::Noop)),
                },
            }),
            menu::Item::Separator,
            menu::Item::Button(menu::button::Button {
                icon: Some(Icon::Text {
                    text: "content_paste".into(),
                    font: Some(fonts::icons_filled()),
                }),
                label: "Paste".into(),
                supporting_text: Some("Ctrl+V".into()),
                action: menu::button::MenuAction::Message {
                    is_selected: false,
                    on_press: Some(Action::Direct(Message::Noop)),
                },
            }),
        ])];

        let groups_with_label = vec![menu::Group {
            label: Some("Actions".into()),
            items: vec![
                menu::Item::Button(menu::button::Button {
                    icon: None,
                    label: "Undo".into(),
                    supporting_text: None,
                    action: menu::button::MenuAction::Message {
                        is_selected: false,
                        on_press: Some(Action::Direct(Message::Noop)),
                    },
                }),
                menu::Item::Button(menu::button::Button {
                    icon: None,
                    label: "Redo".into(),
                    supporting_text: None,
                    action: menu::button::MenuAction::Message {
                        is_selected: false,
                        on_press: None,
                    },
                }),
            ],
        }];

        column![
            section_header("Menu", &self.theme),
            iced_m3::widget::menu(menu::Style::new(&self.theme, false), groups),
            section_header("Menu with label", &self.theme),
            iced_m3::widget::menu(menu::Style::new(&self.theme, false), groups_with_label,),
        ]
        .spacing(24)
        .into()
    }

    fn view_badge(&self) -> Element<'_, Message> {
        let style = badge::Style::new(&self.theme);
        let mail_icon = text("mail")
            .font(fonts::icons_filled())
            .size(24.0)
            .color(self.theme.on_surface());
        let notif_icon = text("notifications")
            .font(fonts::icons_filled())
            .size(24.0)
            .color(self.theme.on_surface());

        column![
            section_header("Badges", &self.theme),
            row![
                badge(
                    style,
                    Space::new().width(Length::Shrink).height(Length::Shrink)
                ),
                badge(
                    style,
                    Space::new().width(Length::Shrink).height(Length::Shrink)
                )
                .label("1"),
                badge(
                    style,
                    Space::new().width(Length::Shrink).height(Length::Shrink)
                )
                .label("99"),
                badge(
                    style,
                    Space::new().width(Length::Shrink).height(Length::Shrink)
                )
                .label("999+"),
            ]
            .spacing(16)
            .align_y(Alignment::Center),
            section_header("Badges on icons", &self.theme),
            row![
                center(
                    row![
                        mail_icon,
                        badge(
                            style,
                            Space::new().width(Length::Shrink).height(Length::Shrink)
                        )
                        .label("3")
                        .bounds_mode(badge::BoundsMode::Symmetrical),
                    ]
                    .align_y(Alignment::Center)
                    .spacing(0),
                )
                .width(48)
                .height(48),
                center(
                    row![
                        notif_icon,
                        badge(
                            style,
                            Space::new().width(Length::Shrink).height(Length::Shrink)
                        )
                        .label("99+")
                        .bounds_mode(badge::BoundsMode::Symmetrical),
                    ]
                    .align_y(Alignment::Center)
                    .spacing(0),
                )
                .width(48)
                .height(48),
            ]
            .spacing(16),
        ]
        .spacing(24)
        .into()
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SwitchTab(tab) => {
                self.tab = tab;
            }
            Message::ToggleDarkTheme => {
                self.theme.mode = match self.theme.mode {
                    Mode::Dark => Mode::Light,
                    Mode::Light => Mode::Dark,
                };
            }
            Message::ToggleSwitch => self.switch_value = !self.switch_value,
            Message::TextInputChanged(value) => self.text_input_value = value,
            Message::SecureInputChanged(value) => self.secure_text_input_value = value,
            Message::TextInputErrorToggle => self.text_input_error = !self.text_input_error,
            // Sliders
            Message::SliderValueXs(v) => self.slider_value_xs = v,
            Message::SliderValueS(v) => self.slider_value_s = v,
            Message::SliderValueM(v) => self.slider_value_m = v,
            Message::SliderValueL(v) => self.slider_value_l = v,
            Message::SliderValueXl(v) => self.slider_value_xl = v,
            Message::SliderCentered(v) => self.slider_centered = v,
            Message::SliderStepped(v) => self.slider_stepped = v,
            Message::SliderTicks(v) => self.slider_ticks = v,
            Message::SliderVolume(v) => self.slider_volume = v,
            Message::SliderPercent(v) => self.slider_percent = v,
            Message::SliderDecimals(v) => self.slider_decimals = v,
            Message::SliderRange(start, end) => self.slider_range = (start, end),
            Message::SliderRangeStepped(start, end) => self.slider_range_stepped = (start, end),
            Message::SliderRangeSmooth(start, end) => self.slider_range_smooth = (start, end),
            Message::SliderRangeSeparated(start, end) => self.slider_range_separated = (start, end),
            Message::SliderRangeOneHandle(start, end) => {
                self.slider_range_one_handle = (start, end)
            }
            Message::SliderVertical(v) => self.slider_vertical = v,
            Message::SliderDisabled => {}
            // Dialog
            Message::ToggleDialog => self.dialog_open = !self.dialog_open,
            Message::CloseDialog => self.dialog_open = false,
            // Navigation
            Message::NavBarTabChanged(index) => self.navbar_index = index,
            // Groups
            Message::SelectGroupIndex(index) => self.group_selected = index,
            Message::ToggleGroupButton(index) => {
                self.group_buttons[index] = !self.group_buttons[index]
            }
            // Cards
            Message::CardPressed => self.card_press_count += 1,
            // Reuse demos
            Message::Checkbox(msg) => self.checkbox_demo.update(msg),
            Message::SwitchDemo(msg) => self.switch_demo.update(msg),
            // Progress
            Message::ProgressTick => {
                self.progress_value += 0.005;
                if self.progress_value > 1.0 {
                    self.progress_value = 0.0;
                }
            }
            // Text input extras
            Message::TextInputSubmitted => {
                self.text_input_submitted = self.text_input_value.clone();
            }
            Message::TextInputWithIconChanged(v) => self.text_input_with_icon = v,
            Message::TextInputWithLabelChanged(v) => self.text_input_with_label = v,
            Message::TextInputAlignedChanged(v) => self.text_input_aligned = v,
            // Button extras
            Message::ToggleButtonSelected => self.button_selected = !self.button_selected,
            // Dialog extras
            Message::ToggleDialogCustom => self.dialog_custom_open = !self.dialog_custom_open,
            Message::CloseDialogCustom => self.dialog_custom_open = false,
            // NavRail
            Message::ToggleNavRailExpanded => {
                self.navrail_expanded = !self.navrail_expanded;
            }
            Message::ToggleNavRailHideLabels => {
                self.navrail_hide_labels = !self.navrail_hide_labels;
            }
            Message::ToggleNavRailShowMenu => {
                self.navrail_show_menu = !self.navrail_show_menu;
            }
            // FAB extras
            Message::ToggleFabCornerStyle => {
                self.fab_corner_style = match self.fab_corner_style {
                    button::CornerStyle::Rounded => button::CornerStyle::Square,
                    button::CornerStyle::Square => button::CornerStyle::Rounded,
                };
            }
            // Progress extras
            Message::ToggleProgressMaybe => {
                self.progress_maybe = if self.progress_maybe.is_some() {
                    None
                } else {
                    Some(0.5)
                };
            }
            Message::ToggleProgressHeightMaybe => {
                self.progress_height_maybe = if self.progress_height_maybe.is_some() {
                    None
                } else {
                    Some(12.0)
                };
            }
            Message::Noop => {}
        }
        Task::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        if self.tab == Tab::Progress {
            iced::time::every(std::time::Duration::from_millis(50)).map(|_| Message::ProgressTick)
        } else {
            Subscription::none()
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
