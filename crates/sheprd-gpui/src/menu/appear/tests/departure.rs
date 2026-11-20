// ABOUTME: Checks every popup family can depart without a visibility jump.
// ABOUTME: Uses explicit motion time to cover interrupted entry, exit, and reopening.
use super::*;

#[gpui::test]
fn close_confirmation_keeps_its_content_after_consent_is_cleared(cx: &mut gpui::TestAppContext) {
    use crate::{close_modal::CloseConfirmation, sidebar::layout_tests::fixture_window};
    let (view, cx) = cx.add_window_view(fixture_window);
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.open_menu(window, cx);
            let snapshot = view.live.snapshot.as_ref().expect("fixture snapshot");
            view.menu.close = CloseConfirmation::capture_tab(snapshot, &snapshot.tabs[0].tab_id);
            assert!(view.menu.close.is_some());
            view.menu.page = Some(Page::ConfirmClose);
        });
        window.refresh();
        window.draw(cx).clear(cx);
    });
    assert!(cx.debug_bounds("close-confirm").is_some());
    assert_eq!(
        view.read_with(cx, |view, _| view.menu.cover.get()),
        crate::menu::Cover::All
    );
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.menu.appear.live = true;
            view.dismiss_menu(window, cx);
            let later = motion::now() + std::time::Duration::from_secs(3600);
            view.menu.appear.observe(None, None, later);
            let leaving = view
                .menu
                .appear
                .leaving
                .as_mut()
                .expect("confirmation leaves");
            leaving.motion.set(true, later, true);
            leaving.motion.set(false, later, false);
            assert!(view.menu.close.is_none(), "closing ends the active consent");
        });
        window.refresh();
        window.draw(cx).clear(cx);
    });
    assert!(
        cx.debug_bounds("close-confirm").is_some(),
        "the panel must retain its captured content"
    );
    assert_eq!(
        view.read_with(cx, |view, _| view.menu.cover.get()),
        crate::menu::Cover::Unknown,
        "departing content reclaimed the active cover"
    );
}

#[gpui::test]
fn palette_departure_preserves_the_visible_scrolled_rows(cx: &mut gpui::TestAppContext) {
    use crate::{palette::Filter, sidebar::layout_tests::fixture_window};
    let (view, cx) = cx.add_window_view(fixture_window);
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.open_palette(Filter::Commands, window, cx)
        });
        window.refresh();
        window.draw(cx).clear(cx);
    });
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            let event = gpui::KeyDownEvent {
                keystroke: gpui::Keystroke::parse("down").expect("valid key"),
                is_held: false,
                prefer_character_input: false,
            };
            for _ in 0..20 {
                view.palette_key(&event, window, cx);
            }
        });
        window.refresh();
        window.draw(cx).clear(cx);
    });
    let before = cx
        .debug_bounds("palette-row-20")
        .expect("the selected row is visible");
    assert!(cx.debug_bounds("palette-row-0").is_none());
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.menu.appear.live = true;
            view.dismiss_menu(window, cx);
            let later = motion::now() + std::time::Duration::from_secs(3600);
            view.menu.appear.observe(None, None, later);
            let leaving = view.menu.appear.leaving.as_mut().expect("palette leaves");
            leaving.motion.set(true, later, true);
            leaving.motion.set(false, later, false);
        });
        window.refresh();
        window.draw(cx).clear(cx);
    });
    assert_eq!(cx.debug_bounds("palette-row-20"), Some(before));
    assert!(cx.debug_bounds("palette-row-0").is_none());
}

#[gpui::test]
fn core_only_footer_keeps_client_actions_without_legacy_connection_rows(
    cx: &mut gpui::TestAppContext,
) {
    use crate::{
        browser::CoreGroupKey, core_terminals, sidebar::layout_tests::fixture_core_window,
    };
    use sheprd_core_client::wire::Doc;
    let (view, cx) = cx.add_window_view(fixture_core_window);
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            let group = uuid::Uuid::from_u128(982);
            core_terminals::set_document_for_test_as(cx, "footer-core", Doc {
                value: serde_json::json!({"schema_version":1,"workspaces":[{"id":group,"label":"Core footer"}]}),
                version: 1,
            });
            let key = CoreGroupKey::new("footer-core", None, "spaceport", group).expect("valid Core group");
            assert!(view.show_core_group(&key, cx));
            view.endpoints.clear();
            view.live.snapshot = None;
            assert!(view.open_menu(window, cx));
        });
        window.refresh();
        window.draw(cx).clear(cx);
    });
    assert!(cx.debug_bounds("menu-settings").is_some());
    assert!(cx.debug_bounds("menu-about").is_some());
    for selector in ["menu-detach", "menu-reconnect", "menu-reload daemon config"] {
        assert!(
            cx.debug_bounds(selector).is_none(),
            "{selector} has no Herdr TUI target"
        );
    }
}

#[gpui::test]
fn messages_departure_keeps_the_captured_history(cx: &mut gpui::TestAppContext) {
    use crate::{notice::Cause, sidebar::layout_tests::fixture_window};
    let (view, cx) = cx.add_window_view(fixture_window);
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.notices.show(Cause::Command, "Captured warning".into());
            view.open_messages(window, cx);
        });
        window.refresh();
        window.draw(cx).clear(cx);
    });
    assert!(cx.debug_bounds("message-text: Captured warning").is_some());
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.menu.appear.live = true;
            view.dismiss_menu(window, cx);
            view.notices
                .show(Cause::Command, "New hidden warning".into());
            let later = motion::now() + std::time::Duration::from_secs(3600);
            view.menu.appear.observe(None, None, later);
            let leaving = view.menu.appear.leaving.as_mut().expect("messages leave");
            leaving.motion.set(true, later, true);
            leaving.motion.set(false, later, false);
        });
        window.refresh();
        window.draw(cx).clear(cx);
    });
    assert!(cx.debug_bounds("message-text: Captured warning").is_some());
    assert!(
        cx.debug_bounds("message-text: New hidden warning")
            .is_none(),
        "new history must not replace departing content"
    );
}

#[gpui::test]
fn footer_departure_keeps_its_captured_rows_when_the_snapshot_changes(
    cx: &mut gpui::TestAppContext,
) {
    use crate::sidebar::layout_tests::fixture_window;
    let (view, cx) = cx.add_window_view(fixture_window);
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            std::sync::Arc::make_mut(view.live.snapshot.as_mut().expect("fixture snapshot"))
                .update_available = Some("999.0.0".into());
            view.open_menu(window, cx);
        });
        window.refresh();
        window.draw(cx).clear(cx);
    });
    assert!(cx.debug_bounds("menu-update ready").is_some());
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.menu.appear.live = true;
            view.dismiss_menu(window, cx);
            std::sync::Arc::make_mut(view.live.snapshot.as_mut().expect("fixture snapshot"))
                .update_available = None;
            let later = motion::now() + std::time::Duration::from_secs(3600);
            view.menu.appear.observe(None, None, later);
            let leaving = view.menu.appear.leaving.as_mut().expect("menu leaves");
            leaving.motion.set(true, later, true);
            leaving.motion.set(false, later, false);
        });
        window.refresh();
        window.draw(cx).clear(cx);
    });
    assert!(
        cx.debug_bounds("menu-update ready").is_some(),
        "closing content must not follow new snapshots"
    );
}

#[gpui::test]
fn palette_departure_preserves_rows_after_active_state_is_dropped(cx: &mut gpui::TestAppContext) {
    use crate::{palette::Filter, sidebar::layout_tests::fixture_window};
    let (view, cx) = cx.add_window_view(fixture_window);
    cx.update(|window, cx| {
        view.update(cx, |view, cx| view.open_palette(Filter::All, window, cx));
        window.refresh();
        window.draw(cx).clear(cx);
    });
    assert!(cx.debug_bounds("palette-row-0").is_some());
    let search = view.update(cx, |view, _| {
        view.menu
            .palette
            .as_ref()
            .expect("open palette")
            .search
            .downgrade()
    });
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.menu.appear.live = true;
            view.dismiss_menu(window, cx);
            let later = motion::now() + std::time::Duration::from_secs(3600);
            view.menu.appear.observe(None, None, later);
            let leaving = view.menu.appear.leaving.as_mut().expect("palette leaves");
            leaving.motion.set(true, later, true);
            leaving.motion.set(false, later, false);
            assert!(
                view.menu.palette.is_none(),
                "readers and search must drop now"
            );
            assert!(view.menu.input.is_none());
        });
        window.refresh();
        window.draw(cx).clear(cx);
    });
    assert!(
        cx.debug_bounds("palette-row-0").is_some(),
        "visible rows must leave with the panel"
    );
    assert!(
        search.upgrade().is_none(),
        "the leaving field must not retain its input entity"
    );
}

#[test]
fn dialogs_popovers_palette_and_messages_leave_after_closing() {
    let start = motion::now();
    for page in [
        Page::About,
        Page::ConfirmClose,
        Page::Palette,
        Page::Messages,
        Page::Tab,
        Page::Menu,
    ] {
        let mut appear = live();
        let shape = if page.pointer_anchored() || page == Page::Menu {
            Shown::Popover(point(px(20.), px(40.)))
        } else {
            Shown::Dialog
        };
        appear.observe(Some(shape), Some(page), start);
        appear.observe(None, None, start + ENTER);
        assert_eq!(appear.leaving(start + ENTER), Some((page, 1.)), "{page:?}");
        assert!(appear.leaving(start + ENTER + LEAVE).is_none());
    }
}

#[test]
fn closing_a_half_entered_popup_starts_at_its_current_position() {
    let start = motion::now();
    let mut appear = live();
    appear.observe(Some(Shown::Dialog), Some(Page::Palette), start);
    let close = start + ENTER / 4;
    let before = appear.panel(close);
    appear.observe(None, None, close);
    let (_, left) = appear.leaving(close).expect("palette leaves");
    assert!((left - before).abs() < 0.001, "{left} jumped from {before}");
}

#[test]
fn reopening_the_same_popup_reverses_its_departure() {
    let start = motion::now();
    let mut appear = live();
    appear.observe(Some(Shown::Dialog), Some(Page::Messages), start);
    let close = start + ENTER;
    appear.observe(None, None, close);
    let reopen = close + LEAVE / 4;
    let (_, before) = appear.leaving(reopen).expect("messages leave");
    appear.observe(Some(Shown::Dialog), Some(Page::Messages), reopen);
    assert!((appear.panel(reopen) - before).abs() < 0.001);
    assert!(appear.leaving(reopen).is_none());
}
