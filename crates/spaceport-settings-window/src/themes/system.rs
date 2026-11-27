//! Themes that follow the system appearance, written as Ghostty's
//! `light:NAME,dark:NAME`: the value turning following on or off builds, the
//! one picking a single theme builds, and the names a pair's sides add to the
//! list.

use spaceport_settings::contrast::luminance;
use spaceport_settings::theme::{FOLLOW_HERDR, ThemeName};

/// What a theme is paired with when following starts: the current theme keeps
/// the side its background suits, and these fill the other.
pub const LIGHT_PAIR: &str = "Catppuccin Latte"; // i18n: data
pub const DARK_PAIR: &str = "Catppuccin Mocha"; // i18n: data

/// The theme value turning following on or off produces. Following stops and
/// keeps the side the system shows; a single theme starts following, and the
/// pair that suits it, read from the `background` it draws, fills the other
/// side.
pub fn toggled(current: &str, background: u32, light: bool) -> String {
    if ThemeName::follows_system(current) {
        ThemeName::side(current, light).to_owned()
    } else if luminance(background) > 0.4 {
        ThemeName::system(current, DARK_PAIR)
    } else {
        ThemeName::system(LIGHT_PAIR, current)
    }
}

/// The theme value picking one brightness's single theme produces, or none
/// when the theme in effect already is that brightness and there is nothing to
/// change. A pair keeps the side picked; a single theme of the other
/// brightness gives way to the default theme of the one picked.
pub fn single(current: &str, light: bool, is_light: bool) -> Option<String> {
    if ThemeName::follows_system(current) {
        return Some(ThemeName::side(current, light).to_owned());
    }
    if is_light == light {
        return None;
    }
    let default = if light { LIGHT_PAIR } else { DARK_PAIR };
    Some(default.to_owned())
}

/// The theme names the list holds even when no theme directory has them: both
/// sides of the configured value, without the one that names Herdr's own
/// theme.
pub fn selected_names(config_theme: &str) -> Vec<String> {
    let mut names = vec![
        ThemeName::side(config_theme, true).to_owned(),
        ThemeName::side(config_theme, false).to_owned(),
    ];
    names.dedup();
    names.retain(|name| name != FOLLOW_HERDR);
    names
}
