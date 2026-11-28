use crate::theme::{ColorScheme, Mode, Palette, Theme};

#[test]
fn default_theme_works() {
    Palette::default_dark();
    Palette::default_light();
}

#[test]
fn system_mode_follows_the_platform_preference() {
    let mut theme = Theme::default(Mode::System);

    theme.system_mode = Mode::Dark;
    assert_eq!(theme.resolved_mode(), Mode::Dark);
    assert_eq!(theme.background(), theme.dark.background);

    theme.system_mode = Mode::Light;
    assert_eq!(theme.resolved_mode(), Mode::Light);
    assert_eq!(theme.background(), theme.light.background);
}

#[test]
fn an_explicit_mode_ignores_the_platform_preference() {
    let mut theme = Theme::default(Mode::Dark);
    theme.system_mode = Mode::Light;
    assert_eq!(theme.resolved_mode(), Mode::Dark);

    theme.mode = Mode::Light;
    theme.system_mode = Mode::Dark;
    assert_eq!(theme.resolved_mode(), Mode::Light);
}
