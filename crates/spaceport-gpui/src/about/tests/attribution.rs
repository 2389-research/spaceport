// ABOUTME: Keep Spaceport ownership distinct from the retained upstream credits.
// ABOUTME: Check notices, project links and readable attribution at narrow sizes.
use super::*;

#[test]
fn notices_credit_spaceport_modifications_and_original_work() {
    let notice = include_str!("../../../../../NOTICE");
    assert!(notice.contains("Copyright 2026 2389 Research, Inc."));
    assert!(notice.contains("Copyright 2026 Fabien Penso"));
    assert!(notice.contains("Spaceport modifications"));
    assert!(notice.contains("Apache License, Version 2.0"));
}

#[test]
fn spaceport_is_the_primary_repository() {
    assert_eq!(REPOSITORY, "https://github.com/2389-research/spaceport");
}

#[gpui::test]
fn ownership_rows_and_project_links_fit_the_about_box(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(crate::sidebar::layout_tests::fixture_window);
    for width in [800., 360.] {
        cx.update(|window, cx| view.update(cx, |view, cx| view.open_about(window, cx)));
        cx.simulate_resize(size(px(width), px(600.)));
        cx.update(|window, cx| window.draw(cx).clear(cx));
        let about = cx.debug_bounds("about").unwrap();
        assert!(about.bottom() <= px(600.), "{width}: {about:?}");
        for selector in [
            "about-publisher",
            "about-modifications",
            "about-original-copyright",
        ] {
            let row = cx.debug_bounds(selector).unwrap();
            assert!(
                row.left() >= about.left() && row.right() <= about.right(),
                "{selector}"
            );
            assert!(
                row.top() >= about.top() && row.bottom() <= about.bottom(),
                "{selector}"
            );
        }
        let project = cx.debug_bounds("about-repository").unwrap();
        assert!(project.bottom() <= cx.debug_bounds("about-upstream-credits").unwrap().top());
        for (selector, url) in [
            (
                "about-repository",
                "https://github.com/2389-research/spaceport",
            ),
            (
                "about-upstream-repository",
                "https://github.com/sheprd-labs/sheprd",
            ),
            ("about-website", "https://herdr.dev/"),
            (
                "about-license",
                "https://www.apache.org/licenses/LICENSE-2.0",
            ),
            (
                "about-notices",
                "https://github.com/2389-research/spaceport/blob/main/NOTICE",
            ),
        ] {
            let link = cx.debug_bounds(selector).unwrap();
            cx.simulate_click(link.center(), Modifiers::default());
            assert_eq!(cx.opened_url().as_deref(), Some(url));
            view.read_with(cx, |view, _| assert_eq!(view.menu.page, Some(Page::About)));
        }
        assert!(cx.debug_bounds("about-twitter").is_none());
    }
}
