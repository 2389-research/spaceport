// ABOUTME: Checks retained native content uses shared tab and terminal layouts.
// ABOUTME: Closing a view restores the terminal without any embedded page UI.
use super::*;
use crate::{browser::Pick, controls::Command};

#[gpui::test]
fn native_devices_tab_closes_back_to_the_existing_terminal(cx: &mut gpui::TestAppContext) {
    let (view, cx) = window(cx);
    draw(cx);
    assert!(cx.debug_bounds("terminal").is_some());
    let before = view.read_with(cx, |view, _| view.live.snapshot.clone().unwrap());
    cx.update(|window, cx| {
        view.update(cx, |view, cx| view.open_devices_overview(window, cx));
    });
    draw(cx);
    assert!(cx.debug_bounds("devices-tab").is_some());
    assert!(cx.debug_bounds("browser-tab-0").is_some());
    assert!(cx.debug_bounds("terminal").is_none());
    for removed in ["browser-toolbar", "browser-address", "browser-placeholder"] {
        assert!(cx.debug_bounds(removed).is_none(), "{removed}");
    }
    cx.update(|window, cx| {
        view.update(cx, |view, cx| view.command(Command::CloseTab, window, cx));
    });
    draw(cx);
    assert!(cx.debug_bounds("devices-tab").is_none());
    assert!(cx.debug_bounds("terminal").is_some());
    view.read_with(cx, |view, cx| {
        assert!(matches!(
            view.group_pick(view.active_group().unwrap()),
            Some(Pick::Herdr(_))
        ));
        assert_eq!(view.live.snapshot.as_ref().unwrap().tabs, before.tabs);
        assert!(view.browser_tab_ids(cx).is_empty());
        assert!(view.menu.page.is_none());
    });
}

#[gpui::test]
fn a_pending_core_tab_uses_the_native_terminal_connection_state(cx: &mut gpui::TestAppContext) {
    use crate::{browser::CoreGroupKey, core_terminals::set_document_for_test_as};
    use sheprd_core_client::wire::Doc;

    let (view, cx) = window(cx);
    let group = uuid::Uuid::from_u128(42);
    let key = CoreGroupKey::new("fixture-core", None, "spaceport", group).unwrap();
    cx.update(|window, cx| {
        set_document_for_test_as(cx, "fixture-core", Doc {
            value: serde_json::json!({"schema_version":1,"workspaces":[{"id":group,"label":"Core"}]}),
            version: 1,
        });
        let id = Store::update(cx, |store| store.open_core(&key, None)).unwrap();
        view.update(cx, |view, cx| {
            assert!(view.show_core_group(&key, cx));
            view.show_browser_tab(id, window, cx);
        });
    });
    draw(cx);
    assert!(cx.debug_bounds("core-terminal-placeholder").is_some());
    assert!(cx.debug_bounds("browser-toolbar").is_none());
    assert!(cx.debug_bounds("browser-address").is_none());
}
