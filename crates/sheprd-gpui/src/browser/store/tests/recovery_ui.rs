// ABOUTME: Checks saved-state failures reach the actual native tab-opening action.
// ABOUTME: Prevents ID recovery refusals from being mislabeled as tab capacity limits.
use super::*;
use crate::sidebar::layout_tests::{fixture_core_window, fixture_window, snapshot};
use std::sync::Arc;

#[gpui::test]
fn unreadable_saved_references_explain_why_devices_cannot_open(cx: &mut gpui::TestAppContext) {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("browser-tabs.json");
    std::fs::write(folder.path().join("editor-groups.json"), "{\"layouts\":[").unwrap();
    let store = Store::load_from(Some(path));
    assert!(store.restoration_blocked());
    cx.update(|cx| cx.set_global(store));
    let (view, cx) = cx.add_window_view(|window, cx| {
        let mut view = fixture_window(window, cx);
        let mut shown = snapshot(40);
        shown.focused_workspace_id = Some("w0".into());
        shown.focused_tab_id = Some("t0".into());
        view.live.snapshot = Some(Arc::new(shown));
        view
    });
    cx.update(|window, cx| {
        view.update(cx, |view, cx| view.open_devices_overview(window, cx));
    });
    view.read_with(cx, |view, cx| {
        let text = &view.flash.as_ref().unwrap().0.text;
        assert!(
            text.contains("Repair the saved tabs and layout files"),
            "{text}"
        );
        assert!(text.contains("restart Spaceport"), "{text}");
        assert!(cx.global::<Store>().tabs().next().is_none());
    });
}

#[gpui::test]
fn unreadable_saved_references_refuse_new_core_terminals_before_connecting(
    cx: &mut gpui::TestAppContext,
) {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("browser-tabs.json");
    std::fs::write(folder.path().join("editor-groups.json"), "{\"layouts\":[").unwrap();
    let store = Store::load_from(Some(path));
    assert!(store.restoration_blocked());
    cx.update(|cx| cx.set_global(store));
    let (view, cx) = cx.add_window_view(|window, cx| {
        let mut view = fixture_core_window(window, cx);
        let mut shown = snapshot(40);
        shown.focused_workspace_id = Some("w0".into());
        shown.focused_tab_id = Some("t0".into());
        view.live.snapshot = Some(Arc::new(shown));
        view.core_terminals
            .hub
            .set_join_for_test(|_| Err(crate::core_terminals::Error::NotInstalled));
        view
    });
    cx.update(|window, cx| {
        view.update(cx, |view, cx| view.new_core_terminal(window, cx));
    });
    view.read_with(cx, |view, _| {
        let text = view.flash.as_ref().map(|(flash, _)| flash.text.as_str());
        assert!(
            text.is_some_and(
                |text| text.contains("Repair the saved tabs and layout files")
                    && text.contains("restart Spaceport")
            ),
            "{text:?}"
        );
    });
}

#[gpui::test]
fn exhausted_tab_ids_refuse_core_creation_before_connecting(cx: &mut gpui::TestAppContext) {
    let store =
        Store::restored_for_test(format!(r#"{{"tabs":[],"next":{}}}"#, u64::MAX).as_bytes())
            .unwrap();
    cx.update(|cx| cx.set_global(store));
    let (view, cx) = cx.add_window_view(|window, cx| {
        let mut view = fixture_core_window(window, cx);
        let mut shown = snapshot(40);
        shown.focused_workspace_id = Some("w0".into());
        view.live.snapshot = Some(Arc::new(shown));
        view.core_terminals
            .hub
            .set_join_for_test(|_| Err(crate::core_terminals::Error::NotInstalled));
        view
    });
    cx.update(|window, cx| view.update(cx, |view, cx| view.new_core_terminal(window, cx)));
    view.read_with(cx, |view, _| {
        assert!(
            view.flash
                .as_ref()
                .is_some_and(|(flash, _)| flash.text == crate::i18n::fl!("browser-too-many-tabs")),
            "{:?}",
            view.flash
        )
    });
}
