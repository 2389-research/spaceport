//! Settings follows Herdr's theme by the side of a `light:…,dark:…` pair
//! the system appearance shows. The test platform's appearance is light.
use super::*;

fn shared(name: &str) -> herdr_settings::Settings {
    herdr_settings::Settings::parse_text(&format!("[theme]\nname = '{name}'")).unwrap()
}

/// Settings after a load of `value` whose theme was resolved for another
/// appearance: the theme that load carried, the theme shown, and Herdr's.
fn loaded_for_another_appearance(
    cx: &mut TestAppContext,
    value: &'static str,
) -> (Theme, Theme, Theme) {
    let source = cx.add_window(crate::sidebar::layout_tests::fixture_window);
    cx.update(|cx| {
        let weak = source.update(cx, |_, _, cx| cx.weak_entity()).unwrap();
        open_fixture(weak, cx);
        cx.global::<SettingsWindowHandle>()
            .window
            .unwrap()
            .update(cx, |view, _, cx| {
                assert!(crate::app::light_appearance(cx));
                let stale = Theme::builtin("Catppuccin Latte").unwrap();
                let mut loaded = fixture();
                loaded.config.theme = value.into();
                loaded.theme = stale.clone();
                loaded.shared = Some(shared("dracula"));
                view.apply_loaded(Ok(loaded), cx);
                let followed = shared("dracula")
                    .theme(true)
                    .unwrap()
                    .styled_for(&view.config);
                assert_ne!(followed, stale);
                (stale, view.theme.clone(), followed)
            })
            .unwrap()
    })
}

#[gpui::test]
fn a_load_resolves_the_shown_side_that_follows_herdr(cx: &mut TestAppContext) {
    let (_, shown, followed) = loaded_for_another_appearance(cx, "light:Follow Herdr,dark:Dracula");
    assert_eq!(shown, followed, "the light side follows Herdr");
}

#[gpui::test]
fn a_load_leaves_a_shown_side_that_names_a_theme(cx: &mut TestAppContext) {
    let (loaded, shown, _) = loaded_for_another_appearance(cx, "light:Dracula,dark:Follow Herdr");
    assert_eq!(shown, loaded, "only the dark side follows Herdr");
}

#[gpui::test]
fn a_load_resolves_a_whole_value_following_herdr(cx: &mut TestAppContext) {
    let (_, shown, followed) = loaded_for_another_appearance(cx, "Follow Herdr");
    assert_eq!(shown, followed);
}

#[gpui::test]
fn a_load_leaves_a_named_theme(cx: &mut TestAppContext) {
    let (loaded, shown, _) = loaded_for_another_appearance(cx, "Nord");
    assert_eq!(shown, loaded);
}

/// A changed shared theme reaches Settings through the appearance pass.
#[gpui::test]
fn the_appearance_pass_follows_herdr_for_the_shown_side(cx: &mut TestAppContext) {
    let source = cx.add_window(crate::sidebar::layout_tests::fixture_window);
    cx.update(|cx| {
        let weak = source.update(cx, |_, _, cx| cx.weak_entity()).unwrap();
        open_fixture(weak, cx);
        cx.global::<SettingsWindowHandle>()
            .window
            .unwrap()
            .update(cx, |view, _, cx| {
                view.config.theme = "light:Follow Herdr,dark:Dracula".into();
                view.theme = Theme::builtin("Catppuccin Latte").unwrap();
                view.shared = Some(shared("nord"));
                view.apply_window_appearance(cx);
                let followed = shared("nord").theme(true).unwrap();
                assert_eq!(view.theme, followed.styled_for(&view.config));
            })
            .unwrap();
    });
}
