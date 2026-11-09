use super::*;

#[gpui::test]
fn profile_slot_opens_local_controls_without_github_auth(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(header_window);
    cx.update(|window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
    });
    let slot = cx.debug_bounds("titlebar-account-slot").unwrap();
    assert_eq!(slot.size.width, px(40.));
    let hit = cx.debug_bounds("titlebar-avatar").unwrap();
    cx.simulate_click(hit.center(), Modifiers::default());
    cx.update(|window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
    });
    cx.update(|window, cx| {
        view.read_with(cx, |view, _| {
            assert_eq!(view.menu.page, Some(Page::Profile));
            assert!(view.menu.focus.is_focused(window));
        });
    });
    assert!(cx.debug_bounds("profile-heading").is_some());
    for selector in [
        "profile-settings",
        "profile-shortcuts",
        "profile-themes",
        "profile-about",
    ] {
        assert!(cx.debug_bounds(selector).is_some(), "{selector}");
    }
    assert!(cx.debug_bounds("github-start").is_none());
    assert!(cx.debug_bounds("github-sign-out").is_none());
}

#[gpui::test]
fn profile_rows_open_their_local_views(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(header_window);
    for (selector, destination) in [
        ("profile-shortcuts", Page::Keybinds),
        ("profile-themes", Page::Themes),
        ("profile-about", Page::About),
    ] {
        cx.update(|window, cx| {
            view.update(cx, |view, cx| view.open_profile(window, cx));
            window.refresh();
            window.draw(cx).clear(cx);
        });
        let row = cx.debug_bounds(selector).unwrap();
        cx.simulate_click(row.center(), Modifiers::default());
        view.read_with(cx, |view, _| assert_eq!(view.menu.page, Some(destination)));
        cx.update(|window, cx| view.update(cx, |view, cx| view.dismiss_menu(window, cx)));
    }

    let before = cx.update(|_, cx| cx.windows().len());
    cx.update(|window, cx| {
        view.update(cx, |view, cx| view.open_profile(window, cx));
        window.refresh();
        window.draw(cx).clear(cx);
    });
    let row = cx.debug_bounds("profile-settings").unwrap();
    cx.simulate_click(row.center(), Modifiers::default());
    cx.run_until_parked();
    view.read_with(cx, |view, _| assert!(view.menu.page.is_none()));
    assert_eq!(cx.update(|_, cx| cx.windows().len()), before + 1);
}
