// ABOUTME: Exercises client-only popups and captured batches without a legacy endpoint.
// ABOUTME: Preserves explicit consent and exact Core document ownership.
use crate::{
    SheprdWindow,
    browser::{CoreGroupKey, Pick, Store, TabId},
    close_modal::batch::{Batch, Inspection},
    core_terminals,
    menu::Page,
    sidebar::layout_tests::fixture_core_window,
};
use gpui::{Entity, TestAppContext, VisualTestContext};
use sheprd_core_client::wire::Doc;
use uuid::Uuid;

fn core_window(cx: &mut TestAppContext) -> (Entity<SheprdWindow>, TabId, &mut VisualTestContext) {
    let (view, cx) = cx.add_window_view(fixture_core_window);
    let tab = cx.update(|_, cx| {
        let group = Uuid::from_u128(901);
        let key = CoreGroupKey::new("batch-core", None, "spaceport", group).unwrap();
        core_terminals::set_document_for_test_as(
            cx,
            "batch-core",
            Doc {
                value: serde_json::json!({"schema_version":1,"workspaces":[
                    {"id":group,"label":"Core batch"}
                ]}),
                version: 1,
            },
        );
        view.update(cx, |view, cx| {
            assert!(view.show_core_group(&key, cx));
            view.endpoints.clear();
            view.live.snapshot = None;
        });
        Store::update(cx, |store| store.open_core(&key, None)).unwrap()
    });
    (view, tab, cx)
}

#[gpui::test]
fn core_only_window_opens_messages_without_authorizing_a_legacy_target(cx: &mut TestAppContext) {
    let (view, _, cx) = core_window(cx);
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.open_messages(window, cx);
            assert_eq!(view.menu.page, Some(Page::Messages));
            assert!(!view.menu_target_current());
            view.dismiss_menu(window, cx);
        })
    });
}

#[gpui::test]
fn core_only_tab_shows_gesture_warnings_outside_the_legacy_terminal(cx: &mut TestAppContext) {
    let (view, tab, cx) = core_window(cx);
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.show_browser_tab(tab, window, cx);
            view.show_flash(crate::window::Flash::warning("Could not copy"), cx);
            assert!(view.endpoints.is_empty());
        });
    });
    cx.update(|window, cx| crate::sidebar::layout_tests::full_draw(window, cx).clear(cx));
    assert!(cx.debug_bounds("core-terminal-placeholder").is_some());
    assert!(cx.debug_bounds("flash").is_some());
}

#[gpui::test]
fn core_only_batch_captures_and_validates_its_view_without_a_legacy_connection(
    cx: &mut TestAppContext,
) {
    let (view, tab, cx) = core_window(cx);
    view.update(cx, |view, cx| {
        let batch = Batch::capture(view, vec![Pick::Page(tab)], cx).unwrap();
        assert!(matches!(
            batch.validate(view, &batch.items, cx),
            Ok(Inspection::Current)
        ));
        assert_eq!(batch.view_count(), 1);
        assert_eq!(batch.pane_count(), 0);
        view.selection_epoch += 1;
        assert!(matches!(
            batch.validate(view, &batch.items, cx),
            Err(crate::Error::StaleConnection)
        ));
    });
}

#[gpui::test]
fn core_only_batch_cancel_keeps_the_view_and_confirm_detaches_only_that_view(
    cx: &mut TestAppContext,
) {
    let (view, tab, cx) = core_window(cx);
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.close_tabs(vec![Pick::Page(tab)], window, cx);
            assert_eq!(view.menu.page, Some(Page::ConfirmClose));
            assert!(cx.global::<Store>().get(tab).is_some());
            view.dismiss_menu(window, cx);
            assert!(cx.global::<Store>().get(tab).is_some());
            view.close_tabs(vec![Pick::Page(tab)], window, cx);
            view.send_close(window, cx);
            assert!(cx.global::<Store>().get(tab).is_none());
            assert!(view.browser.closing.is_none());
            assert!(view.endpoints.is_empty());
        })
    });
}
