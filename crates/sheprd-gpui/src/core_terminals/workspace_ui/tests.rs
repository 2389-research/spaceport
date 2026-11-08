use super::{Access, EditGuard, GuardState, VersionedDocument, observation};
use crate::{menu::Page, sidebar};
use sheprd_core_client::{Error, Method, wire::Doc};

#[test]
fn the_reader_never_treats_a_stale_document_as_empty() {
    let mut projection = observation::Projection::default();
    projection.begin(1);
    assert!(matches!(Access::of(Some(&projection)), Access::Missing));
    projection.initial(
        1,
        vec![],
        Ok(VersionedDocument::read(Doc {
            value: serde_json::Value::Null,
            version: 0,
        })
        .unwrap()),
    );
    assert!(matches!(Access::of(Some(&projection)), Access::Ready(_)));
    projection.lost(1);
    assert!(matches!(Access::of(Some(&projection)), Access::Missing));
}

#[gpui::test]
fn core_create_dialog_keeps_a_name_and_refuses_stale_submission(cx: &mut gpui::TestAppContext) {
    assert!(matches!(Access::of(None), Access::Missing));
    let (view, cx) = cx.add_window_view(sidebar::layout_tests::fixture_window);
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.open_core_workspace_dialog(window, cx);
            assert_eq!(view.menu.page, Some(Page::CoreWorkspaceCreate));
            assert!(view.menu.input.is_some());
            view.menu.input.as_mut().unwrap().text = "One".into();
            view.submit_core_workspace_dialog(cx);
            assert!(view.core_terminals.workspace_ui.pending.is_none());
            assert!(matches!(view.core_workspace_access(cx), Access::Missing));
        });
    });
}

#[gpui::test]
fn uncertain_write_blocks_the_shared_version_until_core_observes_a_new_one(
    cx: &mut gpui::TestAppContext,
) {
    let (view, cx) = cx.add_window_view(sidebar::layout_tests::fixture_window);
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.open_core_workspace_dialog(window, cx);
            let (sender, receiver) = std::sync::mpsc::sync_channel(1);
            view.core_terminals.workspace_ui.pending = Some(receiver);
            let shared = EditGuard::begin(cx, 3).unwrap();
            let reason = Error::Timeout(Method::DocSet).to_string();
            let outcome = Err(super::edit::Error::WriteUnknown {
                version: 3,
                source: Error::Timeout(Method::DocSet),
            });
            EditGuard::finish(&shared, 3, &outcome);
            sender.send(outcome).unwrap();
            view.poll_core_workspace_edit(window, cx);
            assert!(
                view.menu
                    .error
                    .as_ref()
                    .is_some_and(|message| message.contains(&reason))
            );
        });
        assert!(EditGuard::blocks(cx, 3));
        let mut projection = observation::Projection::default();
        projection.begin(1);
        projection.initial(
            1,
            vec![],
            Ok(VersionedDocument::read(Doc {
                value: serde_json::Value::Null,
                version: 3,
            })
            .unwrap()),
        );
        let Access::Ready(document) = Access::of(Some(&projection)) else {
            panic!("Core document must be fresh");
        };
        assert!(EditGuard::blocks(cx, document.version));
        projection.initial(
            1,
            vec![],
            Ok(VersionedDocument::read(Doc {
                value: serde_json::Value::Null,
                version: 4,
            })
            .unwrap()),
        );
        let Access::Ready(document) = Access::of(Some(&projection)) else {
            panic!("new Core document must be fresh");
        };
        assert!(!EditGuard::blocks(cx, document.version));
    });
}

#[test]
fn an_uncertain_worker_keeps_the_version_fenced_after_its_window_closes() {
    let shared = std::sync::Arc::new(std::sync::Mutex::new(GuardState::Pending(3)));
    assert!(EditGuard::blocked(GuardState::Pending(3), 8));
    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    drop(receiver);
    let worker_guard = shared.clone();
    std::thread::spawn(move || {
        let outcome = Err(super::edit::Error::WriteUnknown {
            version: 7,
            source: Error::Timeout(Method::DocSet),
        });
        EditGuard::finish(&worker_guard, 3, &outcome);
        let _ = sender.send(outcome);
    })
    .join()
    .unwrap();
    assert_eq!(*shared.lock().unwrap(), GuardState::Uncertain(7));
    assert!(EditGuard::blocked(GuardState::Uncertain(7), 4));
    assert!(EditGuard::blocked(GuardState::Uncertain(7), 7));
    assert!(!EditGuard::blocked(GuardState::Uncertain(7), 8));
}
