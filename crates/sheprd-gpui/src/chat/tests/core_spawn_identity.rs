// ABOUTME: Exercises a focused chat split while its runtime reconnects.
// ABOUTME: Refuses an unscoped restored tab when its Core identity is unknown.

use super::core_panes::{DRIVEN, attached, id, tab};
use crate::{
    controls::Command,
    core_terminals::{self, Hub},
    i18n::fl,
};

#[gpui::test]
fn a_keyboard_split_without_a_runtime_identity_is_refused(cx: &mut gpui::TestAppContext) {
    let (view, _stream, _program, cx) = attached(cx);
    cx.update(|window, cx| {
        let restored = tab(DRIVEN, cx);
        view.update(cx, |view, cx| {
            view.core_terminals.hub = Hub::new(true);
            view.core_terminals
                .hub
                .set_join_for_test(|_| Err(core_terminals::Error::NotInstalled));
            let focus = view
                .core_chat_focus(restored, &id("t1"), window, cx)
                .unwrap();
            window.focus(&focus, cx);
            assert_eq!(
                view.focused_core_chat(window, cx),
                Some((restored, id("t1")))
            );
            assert!(view.core_terminals.hub.backend().is_none());
            assert!(view.core_pane_command(Command::SplitRight, window, cx));
            assert_eq!(
                view.flash.as_ref().map(|(flash, _)| flash.text.to_string()),
                Some(fl!("error-stale-workspace"))
            );
        });
    });
}

#[gpui::test]
fn a_ready_hub_does_not_prove_a_restored_chats_runtime(cx: &mut gpui::TestAppContext) {
    static NEXT: std::sync::Mutex<Option<sheprd_core_client::CoreBackend>> =
        std::sync::Mutex::new(None);
    let (view, _stream, _program, cx) = attached(cx);
    let (backend, _peer) = core_terminals::connect_backend_for_test("core-b");
    *NEXT.lock().unwrap() = Some(backend);
    view.update(cx, |view, _| {
        view.core_terminals.hub = Hub::new(true);
        view.core_terminals
            .hub
            .set_join_for_test(|_| Ok(NEXT.lock().unwrap().take().unwrap()));
        view.core_terminals.hub.connect(core_terminals::Start::Join);
    });
    super::wait_until(|| {
        view.update(cx, |view, _| {
            view.core_terminals.hub.poll();
            view.core_terminals.hub.backend().is_some()
        })
    });
    cx.update(|window, cx| {
        core_terminals::set_document_for_test_as(
            cx,
            "core-a",
            sheprd_core_client::wire::Doc {
                value: serde_json::json!({"schema_version":1,"workspaces":[]}),
                version: 1,
            },
        );
        let restored = tab(DRIVEN, cx);
        view.update(cx, |view, cx| {
            let focus = view
                .core_chat_focus(restored, &id("t1"), window, cx)
                .unwrap();
            window.focus(&focus, cx);
            assert_eq!(
                view.focused_core_chat(window, cx),
                Some((restored, id("t1")))
            );
            assert_eq!(
                view.core_terminals
                    .hub
                    .backend()
                    .unwrap()
                    .core()
                    .hello()
                    .core
                    .id,
                "core-b"
            );
            assert!(view.core_pane_command(Command::SplitRight, window, cx));
            assert_eq!(
                view.flash.as_ref().map(|(flash, _)| flash.text.to_string()),
                Some(fl!("error-stale-workspace"))
            );
        });
    });
    cx.update(|window, cx| {
        let mut current = view
            .read(cx)
            .core_terminals
            .chat_terminals(cx)
            .find(|info| info.id == id("t1"))
            .unwrap()
            .clone();
        current.labels.insert(
            sheprd_chat_protocol::TOKEN.into(),
            "0123456789abcdef0123456789abcdef".into(),
        );
        core_terminals::set_document_for_test_as(
            cx,
            "core-b",
            sheprd_core_client::wire::Doc {
                value: serde_json::json!({"schema_version":1,"workspaces":[]}),
                version: 2,
            },
        );
        core_terminals::set_terminals_for_test(cx, vec![current]);
        view.update(cx, |view, cx| {
            view.flash = None;
            assert!(view.core_pane_command(Command::SplitRight, window, cx));
            assert_eq!(
                view.flash.as_ref().map(|(flash, _)| flash.text.to_string()),
                Some(fl!("error-stale-workspace"))
            );
        });
    });
}

#[gpui::test]
fn a_retained_chat_must_match_the_current_catalog_target(cx: &mut gpui::TestAppContext) {
    let (view, _stream, _program, cx) = attached(cx);
    view.read_with(cx, |view, cx| {
        let mut current = view
            .core_terminals
            .chat_terminals(cx)
            .find(|info| info.id == id("t1"))
            .unwrap()
            .clone();
        assert!(view.core_chat_matches_terminal(&current));
        current.labels.insert(
            sheprd_chat_protocol::TOKEN.into(),
            "0123456789abcdef0123456789abcdef".into(),
        );
        assert!(
            !view.core_chat_matches_terminal(&current),
            "a reused terminal ID must not retain another Core's chat target"
        );
    });
}
