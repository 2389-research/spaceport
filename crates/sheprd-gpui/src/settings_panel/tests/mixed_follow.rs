//! Which theme values take the daemon's shared theme when it changes: the
//! side of a `light:…,dark:…` pair the window shows decides, not the whole
//! value.
use super::*;
use crate::config::Theme;

fn shared(name: &str) -> Settings {
    match Settings::parse_text(&format!("[theme]\nname = '{name}'")) {
        Ok(shared) => shared,
        Err(error) => panic!("shared settings: {error}"),
    }
}

/// A window showing `value` while Herdr names Nord, after Herdr's shared
/// theme changes to Dracula: the theme it started with, the theme it shows
/// now, and Herdr's new theme for the appearance.
///
/// The test platform's appearance is always light, so a pair shows its
/// light side; a dark window cannot be built here.
fn after_the_shared_theme_changes(cx: &mut TestAppContext, value: &str) -> (Theme, Theme, Theme) {
    let (view, cx) = cx.add_window_view(crate::sidebar::layout_tests::fixture_window);
    let start = Theme::builtin("Catppuccin Latte").unwrap_or_default();
    view.update(cx, |view, cx| {
        assert!(crate::app::light_appearance(cx));
        view.config.theme = value.into();
        view.settings.shared = Some(shared("nord"));
        view.theme = start.clone();
        view.load_shared_settings_with(|| Ok(shared("dracula")), cx);
    });
    cx.run_until_parked();
    view.read_with(cx, |view, _| {
        assert!(view.settings.error.is_none(), "{:?}", view.settings.error);
        let followed = match shared("dracula").theme(true) {
            Ok(theme) => theme.styled_for(&view.config),
            Err(error) => panic!("shared theme: {error}"),
        };
        assert_ne!(followed, start);
        (start, view.theme.clone(), followed)
    })
}

#[gpui::test]
fn a_pair_whose_shown_side_follows_herdr_takes_a_changed_shared_theme(cx: &mut TestAppContext) {
    let (_, shown, followed) =
        after_the_shared_theme_changes(cx, "light:Follow Herdr,dark:Dracula");
    assert_eq!(shown, followed, "the light side follows Herdr");
}

#[gpui::test]
fn a_pair_whose_hidden_side_follows_herdr_keeps_its_theme(cx: &mut TestAppContext) {
    let (start, shown, _) = after_the_shared_theme_changes(cx, "light:Dracula,dark:Follow Herdr");
    assert_eq!(shown, start, "only the dark side follows Herdr");
}

#[gpui::test]
fn a_whole_value_following_herdr_takes_a_changed_shared_theme(cx: &mut TestAppContext) {
    let (_, shown, followed) = after_the_shared_theme_changes(cx, "Follow Herdr");
    assert_eq!(shown, followed);
}

#[gpui::test]
fn a_named_theme_ignores_a_changed_shared_theme(cx: &mut TestAppContext) {
    let (start, shown, _) = after_the_shared_theme_changes(cx, "Nord");
    assert_eq!(shown, start);
}
