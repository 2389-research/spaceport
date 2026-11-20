// ABOUTME: Keeps closed dialog text visible while its motion finishes.
// ABOUTME: Checks reset consent data without using real clipboard or URL side effects.
use super::*;
use crate::{menu::VersionNotice, sidebar::layout_tests::fixture_window, terminal::ExternalUrl};
use gpui::{Entity, TestAppContext, VisualTestContext};
use sheprd_client::VersionMismatch;

fn draw(cx: &mut VisualTestContext) {
    cx.update(|window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
    });
}

fn close(view: &Entity<SheprdWindow>, cx: &mut VisualTestContext) {
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.menu.appear.live = true;
            view.dismiss_menu(window, cx);
            let later = motion::now() + std::time::Duration::from_secs(3600);
            view.menu.appear.observe(None, None, later);
            let leaving = view.menu.appear.leaving.as_mut().expect("dialog leaves");
            leaving.motion.set(true, later, true);
            leaving.motion.set(false, later, false);
            assert!(view.menu.page.is_none());
            assert!(view.menu.version_notice.is_none());
            assert!(view.menu.open_link.is_none());
        });
    });
}

#[gpui::test]
fn version_notice_body_leaves_after_its_live_notice_is_reset(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(fixture_window);
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.open_menu(window, cx);
            view.menu.version_notice = Some(VersionNotice {
                host: Some("fixture host".into()),
                mismatch: VersionMismatch::DaemonOutdated {
                    server_version: Some("0.8.2".into()),
                },
            });
            view.menu.page = Some(Page::VersionMismatch);
        });
    });
    draw(cx);
    assert!(cx.debug_bounds("version-mismatch-body").is_some());
    close(&view, cx);
    draw(cx);
    assert!(
        cx.debug_bounds("version-mismatch-body").is_some(),
        "the departing notice must keep its body"
    );
}

#[gpui::test]
fn declined_link_keeps_its_address_while_leaving(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(fixture_window);
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.ask_to_open_link(
                ExternalUrl::try_from("spaceport-fixture:declined").expect("fixture URL"),
                window,
                cx,
            );
        });
    });
    draw(cx);
    assert!(cx.debug_bounds("open-link-url").is_some());
    close(&view, cx);
    draw(cx);
    assert!(
        cx.debug_bounds("open-link-url").is_some(),
        "the departing question must keep its address"
    );
    assert!(cx.opened_url().is_none());
}

#[gpui::test]
fn release_note_body_does_not_follow_a_replacement_snapshot_while_leaving(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(fixture_window);
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.live.snapshot = Some(std::sync::Arc::new(
                serde_json::from_str(include_str!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/../sheprd-protocol/tests/fixtures/endpoint-snapshot-v1.json"
                )))
                .expect("synthetic release notes"),
            ));
            std::sync::Arc::make_mut(view.live.snapshot.as_mut().expect("snapshot"))
                .release_notes
                .as_mut()
                .expect("notes")
                .body = (0..100).map(|line| format!("Line {line}\n")).collect();
            view.open_menu(window, cx);
            view.menu.page = Some(Page::Update);
        });
    });
    draw(cx);
    assert!(cx.debug_bounds("release-notes-body").is_some());
    let body = cx.debug_bounds("release-notes-body").expect("scroll body");
    let before = cx.debug_bounds("release-notes-line-0").expect("first line");
    cx.simulate_event(gpui::ScrollEvent {
        position: body.center(),
        delta: gpui::ScrollDelta::Pixels(point(px(0.), px(-120.))),
        ..Default::default()
    });
    draw(cx);
    let scrolled = cx
        .debug_bounds("release-notes-line-0")
        .expect("scrolled line");
    assert!(
        scrolled.top() < before.top(),
        "the live notes actually scroll"
    );
    close(&view, cx);
    view.update(cx, |view, _| view.live.snapshot = None);
    draw(cx);
    assert!(
        cx.debug_bounds("release-notes-body").is_some(),
        "the departing sheet must keep the notes it showed"
    );
    let leaving = cx
        .debug_bounds("release-notes-line-0")
        .expect("departing line");
    assert!((leaving.top() - scrolled.top()).abs() < px(1.));
}
