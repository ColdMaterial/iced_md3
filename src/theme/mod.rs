#[cfg(test)]
mod tests;

use iced::{Color, color};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Accent {
    #[default]
    Primary,
    Secondary,
    Tertiary,
}

impl Accent {
    pub fn color(&self, theme: &(impl ColorScheme + ?Sized)) -> Color {
        match self {
            Accent::Primary => theme.primary(),
            Accent::Secondary => theme.secondary(),
            Accent::Tertiary => theme.tertiary(),
        }
    }

    pub fn on_color(&self, theme: &(impl ColorScheme + ?Sized)) -> Color {
        match self {
            Accent::Primary => theme.on_primary(),
            Accent::Secondary => theme.on_secondary(),
            Accent::Tertiary => theme.on_tertiary(),
        }
    }

    pub fn color_container(&self, theme: &(impl ColorScheme + ?Sized)) -> Color {
        match self {
            Accent::Primary => theme.primary_container(),
            Accent::Secondary => theme.secondary_container(),
            Accent::Tertiary => theme.tertiary_container(),
        }
    }

    pub fn on_color_container(&self, theme: &(impl ColorScheme + ?Sized)) -> Color {
        match self {
            Accent::Primary => theme.on_primary_container(),
            Accent::Secondary => theme.on_secondary_container(),
            Accent::Tertiary => theme.on_tertiary_container(),
        }
    }

    pub fn color_fixed(&self, theme: &(impl ColorScheme + ?Sized)) -> Color {
        match self {
            Accent::Primary => theme.primary_fixed(),
            Accent::Secondary => theme.secondary_fixed(),
            Accent::Tertiary => theme.tertiary_fixed(),
        }
    }

    pub fn on_color_fixed(&self, theme: &(impl ColorScheme + ?Sized)) -> Color {
        match self {
            Accent::Primary => theme.on_primary_fixed(),
            Accent::Secondary => theme.on_secondary_fixed(),
            Accent::Tertiary => theme.on_tertiary_fixed(),
        }
    }

    pub fn color_fixed_dim(&self, theme: &(impl ColorScheme + ?Sized)) -> Color {
        match self {
            Accent::Primary => theme.primary_fixed_dim(),
            Accent::Secondary => theme.secondary_fixed_dim(),
            Accent::Tertiary => theme.tertiary_fixed_dim(),
        }
    }

    pub fn on_color_fixed_variant(&self, theme: &(impl ColorScheme + ?Sized)) -> Color {
        match self {
            Accent::Primary => theme.on_primary_fixed_variant(),
            Accent::Secondary => theme.on_secondary_fixed_variant(),
            Accent::Tertiary => theme.on_tertiary_fixed_variant(),
        }
    }
}

pub trait ColorScheme {
    fn primary(&self) -> Color;
    fn on_primary(&self) -> Color;
    fn primary_container(&self) -> Color;
    fn on_primary_container(&self) -> Color;
    fn primary_fixed(&self) -> Color;
    fn on_primary_fixed(&self) -> Color;
    fn primary_fixed_dim(&self) -> Color;
    fn on_primary_fixed_variant(&self) -> Color;
    fn inverse_primary(&self) -> Color;

    fn secondary(&self) -> Color;
    fn on_secondary(&self) -> Color;
    fn secondary_container(&self) -> Color;
    fn on_secondary_container(&self) -> Color;
    fn secondary_fixed(&self) -> Color;
    fn on_secondary_fixed(&self) -> Color;
    fn secondary_fixed_dim(&self) -> Color;
    fn on_secondary_fixed_variant(&self) -> Color;

    fn tertiary(&self) -> Color;
    fn on_tertiary(&self) -> Color;
    fn tertiary_container(&self) -> Color;
    fn on_tertiary_container(&self) -> Color;
    fn tertiary_fixed(&self) -> Color;
    fn on_tertiary_fixed(&self) -> Color;
    fn tertiary_fixed_dim(&self) -> Color;
    fn on_tertiary_fixed_variant(&self) -> Color;

    fn error(&self) -> Color;
    fn on_error(&self) -> Color;
    fn error_container(&self) -> Color;
    fn on_error_container(&self) -> Color;

    fn surface(&self) -> Color;
    fn on_surface(&self) -> Color;
    fn surface_variant(&self) -> Color;
    fn on_surface_variant(&self) -> Color;
    fn surface_container_highest(&self) -> Color;
    fn surface_container_high(&self) -> Color;
    fn surface_container(&self) -> Color;
    fn surface_container_low(&self) -> Color;
    fn surface_container_lowest(&self) -> Color;
    fn inverse_surface(&self) -> Color;
    fn inverse_on_surface(&self) -> Color;

    fn background(&self) -> Color;
    fn on_background(&self) -> Color;
    fn surface_bright(&self) -> Color;
    fn surface_dim(&self) -> Color;
    fn scrim(&self) -> Color;
    fn shadow(&self) -> Color;
    fn outline(&self) -> Color;
    fn outline_variant(&self) -> Color;
}

#[derive(Debug, Clone, Copy)]
pub struct Palette {
    pub primary: Color,
    pub on_primary: Color,
    pub primary_container: Color,
    pub on_primary_container: Color,
    pub primary_fixed: Color,
    pub on_primary_fixed: Color,
    pub primary_fixed_dim: Color,
    pub on_primary_fixed_variant: Color,
    pub inverse_primary: Color,

    pub secondary: Color,
    pub on_secondary: Color,
    pub secondary_container: Color,
    pub on_secondary_container: Color,
    pub secondary_fixed: Color,
    pub on_secondary_fixed: Color,
    pub secondary_fixed_dim: Color,
    pub on_secondary_fixed_variant: Color,

    pub tertiary: Color,
    pub on_tertiary: Color,
    pub tertiary_container: Color,
    pub on_tertiary_container: Color,
    pub tertiary_fixed: Color,
    pub on_tertiary_fixed: Color,
    pub tertiary_fixed_dim: Color,
    pub on_tertiary_fixed_variant: Color,

    pub error: Color,
    pub on_error: Color,
    pub error_container: Color,
    pub on_error_container: Color,

    pub surface: Color,
    pub on_surface: Color,
    pub surface_variant: Color,
    pub on_surface_variant: Color,
    pub surface_container_highest: Color,
    pub surface_container_high: Color,
    pub surface_container: Color,
    pub surface_container_low: Color,
    pub surface_container_lowest: Color,
    pub inverse_surface: Color,
    pub inverse_on_surface: Color,
    pub background: Color,
    pub on_background: Color,
    pub surface_bright: Color,
    pub surface_dim: Color,
    pub scrim: Color,
    pub shadow: Color,

    pub outline: Color,
    pub outline_variant: Color,
}

impl Palette {
    pub const fn default_dark() -> Self {
        Self {
            primary: color!(0xD0BCFF),
            on_primary: color!(0x381E72),
            primary_container: color!(0x4F378B),
            on_primary_container: color!(0xEADDFF),
            primary_fixed: color!(0xEADDFF),
            on_primary_fixed: color!(0x21005D),
            primary_fixed_dim: color!(0xD0BCFF),
            on_primary_fixed_variant: color!(0x4F378B),
            inverse_primary: color!(0x6750A4),

            secondary: color!(0xCCC2DC),
            on_secondary: color!(0x332D41),
            secondary_container: color!(0x4A4458),
            on_secondary_container: color!(0xE8DEF8),
            secondary_fixed: color!(0xE8DEF8),
            on_secondary_fixed: color!(0x1D192B),
            secondary_fixed_dim: color!(0xCCC2DC),
            on_secondary_fixed_variant: color!(0x4A4458),

            tertiary: color!(0xEFB8C8),
            on_tertiary: color!(0x492532),
            tertiary_container: color!(0x633B48),
            on_tertiary_container: color!(0xFFD8E4),
            tertiary_fixed: color!(0xFFD8E4),
            on_tertiary_fixed: color!(0x31111D),
            tertiary_fixed_dim: color!(0xEFB8C8),
            on_tertiary_fixed_variant: color!(0x633B48),

            error: color!(0xF2B8B5),
            on_error: color!(0x601410),
            error_container: color!(0x8C1D18),
            on_error_container: color!(0xF9DEDC),

            surface: color!(0x141218),
            on_surface: color!(0xE6E0E9),
            surface_variant: color!(0x49454F),
            on_surface_variant: color!(0xCAC4D0),
            surface_container_highest: color!(0x36343B),
            surface_container_high: color!(0x2B2930),
            surface_container: color!(0x211F26),
            surface_container_low: color!(0x1D1B20),
            surface_container_lowest: color!(0x0F0D13),
            inverse_surface: color!(0xE6E0E9),
            inverse_on_surface: color!(0x322F35),
            background: color!(0x141218),
            on_background: color!(0xE6E0E9),
            surface_bright: color!(0x3B383E),
            surface_dim: color!(0x141218),
            scrim: color!(0x000000),
            shadow: color!(0x000000),

            outline: color!(0x938F99),
            outline_variant: color!(0x49454F),
        }
    }

    pub const fn default_light() -> Self {
        Self {
            primary: color!(0x6750A4),
            on_primary: color!(0xFFFFFF),
            primary_container: color!(0xEADDFF),
            on_primary_container: color!(0x4F378B),
            primary_fixed: color!(0xEADDFF),
            on_primary_fixed: color!(0x21005D),
            primary_fixed_dim: color!(0xD0BCFF),
            on_primary_fixed_variant: color!(0x4F378B),
            inverse_primary: color!(0xD0BCFF),

            secondary: color!(0x625B71),
            on_secondary: color!(0xFFFFFF),
            secondary_container: color!(0xE8DEF8),
            on_secondary_container: color!(0x4A4458),
            secondary_fixed: color!(0xE8DEF8),
            on_secondary_fixed: color!(0x1D192B),
            secondary_fixed_dim: color!(0xCCC2DC),
            on_secondary_fixed_variant: color!(0x4A4458),

            tertiary: color!(0x7D5260),
            on_tertiary: color!(0xFFFFFF),
            tertiary_container: color!(0xFFD8E4),
            on_tertiary_container: color!(0x633B48),
            tertiary_fixed: color!(0xFFD8E4),
            on_tertiary_fixed: color!(0x31111D),
            tertiary_fixed_dim: color!(0xEFB8C8),
            on_tertiary_fixed_variant: color!(0x633B48),

            error: color!(0xB3261E),
            on_error: color!(0xFFFFFF),
            error_container: color!(0xF9DEDC),
            on_error_container: color!(0x8C1D18),

            surface: color!(0xFEF7FF),
            on_surface: color!(0x1D1B20),
            surface_variant: color!(0xE7E0EC),
            on_surface_variant: color!(0x49454F),
            surface_container_highest: color!(0xE6E0E9),
            surface_container_high: color!(0xECE6F0),
            surface_container: color!(0xF3EDF7),
            surface_container_low: color!(0xF7F2FA),
            surface_container_lowest: color!(0xFFFFFF),
            inverse_surface: color!(0x322F35),
            inverse_on_surface: color!(0xF5EFF7),
            background: color!(0xFEF7FF),
            on_background: color!(0x1D1B20),
            surface_bright: color!(0xFEF7FF),
            surface_dim: color!(0xDED8E1),
            scrim: color!(0x000000),
            shadow: color!(0x000000),

            outline: color!(0x79747E),
            outline_variant: color!(0xCAC4D0),
        }
    }
}

macro_rules! impl_color_scheme {
    ($($name:ident),* $(,)?) => {
        impl ColorScheme for Palette {
            $(fn $name(&self) -> Color {
                self.$name
            })*
        }

        impl ColorScheme for Theme {
            $(fn $name(&self) -> Color {
                self.current().$name
            })*
        }
    };
}

impl_color_scheme! {
    primary,
    on_primary,
    primary_container,
    on_primary_container,
    primary_fixed,
    on_primary_fixed,
    primary_fixed_dim,
    on_primary_fixed_variant,
    inverse_primary,
    secondary,
    on_secondary,
    secondary_container,
    on_secondary_container,
    secondary_fixed,
    on_secondary_fixed,
    secondary_fixed_dim,
    on_secondary_fixed_variant,
    tertiary,
    on_tertiary,
    tertiary_container,
    on_tertiary_container,
    tertiary_fixed,
    on_tertiary_fixed,
    tertiary_fixed_dim,
    on_tertiary_fixed_variant,
    error,
    on_error,
    error_container,
    on_error_container,
    surface,
    on_surface,
    surface_variant,
    on_surface_variant,
    surface_container_highest,
    surface_container_high,
    surface_container,
    surface_container_low,
    surface_container_lowest,
    inverse_surface,
    inverse_on_surface,
    background,
    on_background,
    surface_bright,
    surface_dim,
    scrim,
    shadow,
    outline,
    outline_variant,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Mode {
    #[default]
    System,
    Light,
    Dark,
}

impl Mode {
    pub fn is_system(self) -> bool {
        matches!(self, Self::System)
    }

    pub fn resolve(self, system: Mode) -> Mode {
        match (self, system) {
            (Mode::System, Mode::Dark) => Mode::Dark,
            (Mode::System, Mode::Light | Mode::System) => Mode::Light,
            (mode, _) => mode,
        }
    }
}

impl From<iced::theme::Mode> for Mode {
    fn from(value: iced::theme::Mode) -> Self {
        match value {
            iced::theme::Mode::None => Self::System,
            iced::theme::Mode::Light => Self::Light,
            iced::theme::Mode::Dark => Self::Dark,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Theme {
    pub dark: Palette,
    pub light: Palette,
    pub mode: Mode,
    pub system_mode: Mode,
}

impl Theme {
    pub fn current(&self) -> &Palette {
        match self.resolved_mode() {
            Mode::Light | Mode::System => &self.light,
            Mode::Dark => &self.dark,
        }
    }

    pub fn resolved_mode(&self) -> Mode {
        self.mode.resolve(self.system_mode)
    }

    pub fn default(mode: Mode) -> Self {
        Self {
            dark: Palette::default_dark(),
            light: Palette::default_light(),
            mode,
            system_mode: Mode::Light,
        }
    }
}

impl iced_widget::scrollable::Catalog for Theme {
    type Class<'a> = iced_widget::scrollable::StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(|_theme, _status| iced_widget::scrollable::Style {
            container: iced::widget::container::Style::default(),
            vertical_rail: iced_widget::scrollable::Rail {
                background: None,
                border: iced::Border::default(),
                scroller: iced_widget::scrollable::Scroller {
                    background: iced::Background::Color(iced::Color::TRANSPARENT),
                    border: iced::Border::default(),
                },
            },
            horizontal_rail: iced_widget::scrollable::Rail {
                background: None,
                border: iced::Border::default(),
                scroller: iced_widget::scrollable::Scroller {
                    background: iced::Background::Color(iced::Color::TRANSPARENT),
                    border: iced::Border::default(),
                },
            },
            gap: None,
            auto_scroll: iced_widget::scrollable::AutoScroll {
                background: iced::Background::Color(iced::Color::TRANSPARENT),
                border: iced::Border::default(),
                shadow: iced::Shadow::default(),
                icon: iced::Color::TRANSPARENT,
            },
        })
    }

    fn style(
        &self,
        class: &Self::Class<'_>,
        status: iced_widget::scrollable::Status,
    ) -> iced_widget::scrollable::Style {
        class(self, status)
    }
}
