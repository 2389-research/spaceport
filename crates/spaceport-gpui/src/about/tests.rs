#![allow(clippy::unwrap_used, clippy::expect_used)]
use super::*;
use core::prelude::v1::test;
use gpui::VisualTestContext;

mod attribution;
mod en_xa;
mod scene;
mod selection;

/// Opens the box with `demo` pinned: [`SpaceportWindow::open_about`] picks one
/// at random, and a test reads one arrangement.
fn open_about_with(view: &Entity<SpaceportWindow>, demo: Demo, cx: &mut VisualTestContext) {
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.open_about(window, cx);
            view.about_demo = demo;
        })
    });
}

/// The window text size this window sets its document at.
fn text_size(view: &Entity<SpaceportWindow>, cx: &mut VisualTestContext) -> f32 {
    view.read_with(cx, |view, _| view.config.ui.size)
}

/// The height the wordmark's rows stack to at a window text size of `text`:
/// its cell size, a whole number of pixels, times the rows [`nfo::MARK`]
/// spells.
fn wordmark_height(text: f32) -> f32 {
    nfo::MARK.len() as f32 * nfo::mark_size(text)
}

/// The visible copyright must not drift from the repository's NOTICE.
#[test]
fn copyright_tracks_the_repository_notice() {
    let notice = include_str!("../../../../NOTICE");
    for holder in [PUBLISHER, AUTHOR] {
        let copyright = format!(
            "Copyright {} {holder}",
            COPYRIGHT_YEAR.trim_start_matches("© ")
        );
        assert!(
            notice.lines().any(|line| line == copyright),
            "missing {copyright}"
        );
    }
    assert!(notice.contains("Apache License, Version 2.0"));
    assert!(LICENSE.contains("Apache-2.0"));
}

/// About and update checks lead the application menu as one group.
#[test]
fn about_leads_the_application_menu() {
    let menus = crate::menus(&crate::config::Config::default());
    let application = menus.first().unwrap();
    assert_eq!(application.name.as_ref(), "Spaceport");
    let MenuItem::Action { name, action, .. } = application.items.first().unwrap() else {
        panic!("the application menu must start with an action");
    };
    assert_eq!(name.as_ref(), "About Spaceport");
    assert!(action.partial_eq(&crate::RunCommand {
        command: crate::controls::Command::About,
    }));
    assert!(matches!(
        &application.items[1],
        MenuItem::Action { action, .. } if action.partial_eq(&crate::RunCommand {
            command: crate::controls::Command::CheckForUpdates,
        })
    ));
    assert!(matches!(application.items[2], MenuItem::Separator));
    assert!(matches!(
        application.items.last().unwrap(),
        MenuItem::Action { name, .. } if name.as_ref() == "Quit Spaceport"
    ));
}

#[gpui::test]
fn about_box_shows_build_identity_and_opens_links(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(crate::sidebar::layout_tests::fixture_window);
    for width in [800., 360.] {
        cx.simulate_resize(size(px(width), px(600.)));
        open_about_with(&view, Demo::Starfield, cx);
        cx.update(|window, cx| window.draw(cx).clear(cx));
        let text = text_size(&view, cx);
        let about = cx.debug_bounds("about").unwrap();
        let margin = super::scene::margin(Demo::Starfield, text);
        let mark = cx.debug_bounds("about-icon").unwrap();
        let name = cx.debug_bounds("about-name").unwrap();
        let version = cx.debug_bounds("about-version").unwrap();
        let disclaimer = cx.debug_bounds("about-unaffiliated").unwrap();
        let credits = cx.debug_bounds("about-upstream-credits").unwrap();
        let copyright = cx.debug_bounds("about-copyright").unwrap();
        let close = cx.debug_bounds("about-close").unwrap();
        // The mark is the block wordmark itself, as tall as its rows.
        assert_eq!(mark.size.height, px(wordmark_height(text)));
        for row in [mark, name, version, credits, disclaimer, copyright, close] {
            assert!(row.top() >= about.top() && row.bottom() <= about.bottom());
            assert!(row.left() >= about.left() && row.right() <= about.right());
        }
        assert_eq!(mark.top() - about.top(), px(margin));
        assert!(mark.bottom() <= name.top());
        assert!(name.bottom() <= version.top());
        assert!(version.bottom() <= disclaimer.top());
        assert!(disclaimer.bottom() <= copyright.top());
        assert!(copyright.bottom() <= close.top());
        assert!(about.right() <= px(width));
        assert!(about.bottom() <= px(600.), "{width}: {about:?}");
        assert_eq!(
            cx.debug_bounds("about-build").is_some(),
            env!("HERDR_BUILD_WORKTREE") == "1"
        );

        for (selector, url) in [
            ("about-website", WEBSITE.to_owned()),
            ("about-repository", REPOSITORY.to_owned()),
            ("about-upstream-repository", UPSTREAM_REPOSITORY.to_owned()),
            ("about-license", LICENSE_URL.to_owned()),
            ("about-notices", NOTICE.to_owned()),
        ] {
            let link = cx.debug_bounds(selector).unwrap();
            cx.simulate_click(link.center(), Modifiers::default());
            assert_eq!(cx.opened_url(), Some(url));
            view.read_with(cx, |view, _| {
                assert_eq!(view.menu.page, Some(Page::About));
            });
        }

        // Both the OK button and Escape close the box without other effects.
        cx.simulate_click(close.center(), Modifiers::default());
        view.read_with(cx, |view, _| assert!(view.menu.page.is_none()));
        open_about_with(&view, Demo::Starfield, cx);
        cx.update(|window, cx| window.draw(cx).clear(cx));
        cx.simulate_keystrokes("escape");
        view.read_with(cx, |view, _| assert!(view.menu.page.is_none()));
    }
}

#[gpui::test]
fn the_in_app_menu_opens_the_about_box(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(crate::sidebar::layout_tests::fixture_window);
    cx.simulate_resize(size(px(800.), px(600.)));
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            // Anchor the popup to the status bar so the row is on screen.
            view.menu.anchor = point(px(60.), px(560.));
            view.open_menu(window, cx);
        })
    });
    cx.update(|window, cx| window.draw(cx).clear(cx));
    let row = cx.debug_bounds("menu-about").unwrap();
    cx.simulate_click(row.center(), Modifiers::default());
    view.read_with(cx, |view, _| assert_eq!(view.menu.page, Some(Page::About)));
}
