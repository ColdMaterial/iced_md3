//! Catalog of the widget pages demonstrated by the showcase.
//!
//! Both the showcase navigation and the select demo read from this single source of truth.

/// A widget page showcased by the demo app.
///
/// Variants are declared in navigation order, so [`Tab::index`] can be used directly as the
/// active index of the navigation rail.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum Tab {
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
    Radio,
    Select,
    Shapes,
    Sliders,
    Switch,
}

impl Tab {
    /// Every page, in navigation order.
    pub const ALL: &[Self] = &[
        Self::Badge,
        Self::Groups,
        Self::Buttons,
        Self::Cards,
        Self::Checkbox,
        Self::Dialog,
        Self::Fab,
        Self::Inputs,
        Self::Menu,
        Self::Navigation,
        Self::NavRail,
        Self::Progress,
        Self::Radio,
        Self::Select,
        Self::Shapes,
        Self::Sliders,
        Self::Switch,
    ];

    /// Position of the page in [`Tab::ALL`].
    pub const fn index(self) -> usize {
        self as usize
    }

    /// Human readable page name, used for navigation labels and lists.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Badge => "Badge",
            Self::Groups => "Button groups",
            Self::Buttons => "Buttons",
            Self::Cards => "Cards",
            Self::Checkbox => "Checkbox",
            Self::Dialog => "Dialog",
            Self::Fab => "FAB",
            Self::Inputs => "Inputs",
            Self::Menu => "Menu",
            Self::Navigation => "Navigation bar",
            Self::NavRail => "Navigation rail",
            Self::Progress => "Progress",
            Self::Radio => "Radio",
            Self::Select => "Select",
            Self::Shapes => "Shapes",
            Self::Sliders => "Sliders",
            Self::Switch => "Switch",
        }
    }

    /// Material icon name used by the showcase navigation.
    pub const fn icon(self) -> &'static str {
        match self {
            Self::Badge => "notifications",
            Self::Groups => "group_work",
            Self::Buttons => "touch_app",
            Self::Cards => "dashboard",
            Self::Checkbox => "check_box",
            Self::Dialog => "chat_bubble",
            Self::Fab => "add_circle",
            Self::Inputs => "edit",
            Self::Menu => "menu",
            Self::Navigation => "explore",
            Self::NavRail => "view_sidebar",
            Self::Progress => "hourglass_empty",
            Self::Radio => "radio_button_checked",
            Self::Select => "arrow_drop_down",
            Self::Shapes => "interests",
            Self::Sliders => "tune",
            Self::Switch => "toggle_on",
        }
    }
}
