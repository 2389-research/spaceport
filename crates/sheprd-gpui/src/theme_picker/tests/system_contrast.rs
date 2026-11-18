// ABOUTME: Verifies contrast updates during previews and after cancellation.
// ABOUTME: Uses built-in palettes without reading a personal theme catalog.
use super::*;
use crate::accessibility::policy::{Preferences, tests as policy};
use crate::contrast::Contrast;

fn prepare(view: &mut SheprdWindow, window: &Window, cx: &mut Context<SheprdWindow>) {
    view.config.theme = "Nord".into();
    view.theme = Theme::builtin("Nord").unwrap();
    view.menu.page = Some(Page::Themes);
    view.menu.themes = Some(ThemePicker {
        input: Some(Input::new(cx)),
        names: vec!["Dracula".into()],
        filtered: vec!["Dracula".into()],
        favorites: 0,
        favorites_saving: false,
        favorite_edits: vec![],
        favorites_error: None,
        selected: 0,
        scroll: UniformListScrollHandle::new(),
        error: None,
        query: String::new(),
        discovering: false,
        baseline: Some(view.theme.clone()),
        session: 1,
        request: 0,
        desired: None,
        loaded: None,
        accepting: false,
        saving: false,
        in_flight: None,
        window: window.window_handle(),
        status_scroll: ScrollHandle::new(),
    });
    view.preview_picker_selection(cx);
}

fn set_high(high: bool, cx: &mut VisualTestContext) {
    // The real OS watcher applies from App, with every window available.
    cx.cx.update(|cx| {
        policy::set(
            Preferences {
                high_contrast: Some(high),
                reduce_motion: None,
            },
            cx,
        )
    });
}

#[gpui::test]
fn a_contrast_change_refreshes_preview_and_cancelled_baseline(cx: &mut TestAppContext) {
    let _scope = policy::Scope::new();
    let (view, cx) = cx.add_window_view(crate::sidebar::layout_tests::fixture_window);
    cx.update(|window, cx| view.update(cx, |view, cx| prepare(view, window, cx)));
    for high in [true, false] {
        set_high(high, cx);
        cx.run_until_parked();
        view.read_with(cx, |view, _| {
            let contrast = if high {
                Contrast::High
            } else {
                Contrast::Standard
            };
            assert_eq!(
                view.theme,
                Theme::builtin("Dracula").unwrap().with_contrast(contrast)
            );
            assert_eq!(
                view.menu.themes.as_ref().unwrap().baseline,
                Some(Theme::builtin("Nord").unwrap().with_contrast(contrast))
            );
        });
    }
    set_high(true, cx);
    // Cancel before the background baseline reload finishes.
    view.update(cx, |view, cx| {
        assert!(view.cancel_theme_preview(cx));
        view.menu.page = None;
    });
    cx.run_until_parked();
    view.read_with(cx, |view, _| {
        assert_eq!(
            view.theme,
            Theme::builtin("Nord")
                .unwrap()
                .with_contrast(Contrast::High)
        )
    });
}

#[gpui::test]
fn a_contrast_change_preserves_an_accepted_theme_save(cx: &mut TestAppContext) {
    let _scope = policy::Scope::new();
    let (view, cx) = cx.add_window_view(crate::sidebar::layout_tests::fixture_window);
    cx.update(|window, cx| view.update(cx, |view, cx| prepare(view, window, cx)));
    set_high(true, cx);
    cx.run_until_parked();
    let token = view.update(cx, |view, _| {
        let picker = view.menu.themes.as_mut().unwrap();
        let token = (picker.session, picker.request);
        // Hold the existing write boundary; this fixture writes no personal config.
        picker.accepting = true;
        picker.saving = true;
        picker.in_flight = Some(token);
        token
    });
    set_high(false, cx);
    cx.run_until_parked();
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            let picker = view.menu.themes.as_ref().unwrap();
            assert_eq!(picker.in_flight, Some(token));
            assert!(picker.saving);
            view.finish_picker_load(
                token,
                true,
                Ok(Theme::builtin("Dracula")
                    .unwrap()
                    .with_contrast(Contrast::High)),
                window,
                cx,
            );
            assert_eq!(view.config.theme, "Dracula");
            assert_eq!(view.menu.page, None);
        })
    });
    cx.run_until_parked();
    view.read_with(cx, |view, _| {
        assert_eq!(view.theme, Theme::builtin("Dracula").unwrap())
    });
}
