//! The section list shows each page under its group, above a search box, and
//! the footer offers Save now only while a draft waits.

use super::*;
use gpui::size;

#[gpui::test]
fn every_section_lists_under_its_group(cx: &mut TestAppContext) {
    let (_, cx) = cx.add_window_view(|_, _| Harness::new());
    cx.simulate_resize(size(px(1200.), px(900.)));
    draw(cx);
    let mut last_bottom = bounds(cx, "settings-nav-search").bottom();
    for section in Section::ALL
        .iter()
        .filter(|section| section.group().is_none())
    {
        let row = bounds(cx, leak(section.selector()));
        assert!(row.top() >= last_bottom, "{section:?} above every group");
        last_bottom = row.bottom();
    }
    for group in Group::ALL {
        let label = bounds(cx, leak(format!("settings-group-{}", group.slug())));
        assert!(
            label.top() >= last_bottom,
            "{group:?} below the previous row"
        );
        last_bottom = label.bottom();
        for section in Section::ALL.iter().filter(|s| s.group() == Some(group)) {
            let row = bounds(cx, leak(section.selector()));
            assert!(row.top() >= last_bottom, "{section:?} under {group:?}");
            last_bottom = row.bottom();
        }
    }
    assert!(
        cx.debug_bounds("settings-footer-save").is_none(),
        "no pending drafts, no Save now"
    );
}

#[gpui::test]
fn a_waiting_draft_offers_save_now_and_the_status_names_it(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(|_, _| Harness {
        drafts: Drafts {
            theme: true,
            ..Default::default()
        },
        ..Harness::new()
    });
    cx.simulate_resize(size(px(1200.), px(900.)));
    draw(cx);
    let save = bounds(cx, "settings-footer-save");
    let reload = bounds(cx, "settings-footer-reload");
    assert!(save.right() <= reload.left(), "{save:?} before {reload:?}");
    assert_eq!(
        view.read_with(cx, |harness, _| status_text(
            &harness.drawn(),
            navigation_labels()
        )),
        "settings-draft-theme unsaved"
    );
}

/// An id this module draws but does not list would go unwritten in a language,
/// so every one of them is listed.
#[test]
fn every_id_the_list_and_footer_name_is_listed_for_the_app() {
    for id in [
        SECTIONS,
        SECTIONS_LABEL,
        "settings-draft-theme",
        "settings-draft-layout",
        "settings-draft-sidebar",
        "settings-loading",
        "settings-saved",
        "settings-unsaved",
        "settings-save-now",
        "settings-reload",
    ] {
        assert!(MESSAGE_IDS.contains(&id), "{id} is not in MESSAGE_IDS");
    }
}
