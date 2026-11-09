#![allow(clippy::unwrap_used)]
use crate::sidebar::layout_tests::REPO_KEY;
use gpui::TestAppContext;
use std::sync::Arc;

#[gpui::test]
fn only_a_local_daemon_checkout_is_tracked(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(crate::sidebar::layout_tests::fixture_window);
    cx.update(|_, cx| {
        view.update(cx, |view, _| {
            assert!(view.git_input().is_none(), "no connection, no checkout");
            view.live.status = crate::state::ConnectionStatus::Connected;
            view.live.local_daemon_peer = true;
            let mut snapshot = crate::sidebar::layout_tests::snapshot(6);
            snapshot.focused_workspace_id = Some("w3".into());
            view.live.snapshot = Some(Arc::new(snapshot));
            let input = view.git_input().unwrap();
            assert_eq!(input.repo_key.as_deref(), Some(REPO_KEY));
            assert_eq!(input.branch, "develop");
            assert_eq!(input.checkout, None, "the checkout is resolved by Git");
            // A folder the daemon reports no branch for is not a checkout.
            if let Some(snapshot) = view.live.snapshot.as_ref().map(Arc::clone) {
                let mut snapshot = (*snapshot).clone();
                snapshot.focused_workspace_id = Some("w0".into());
                snapshot.workspaces[0].branch = None;
                view.live.snapshot = Some(Arc::new(snapshot));
            }
            assert!(view.git_input().is_none());
            if let Some(snapshot) = view.live.snapshot.as_ref().map(Arc::clone) {
                let mut snapshot = (*snapshot).clone();
                snapshot.focused_workspace_id = Some("w3".into());
                view.live.snapshot = Some(Arc::new(snapshot));
            }
            assert!(view.git_input().is_some());
            view.live.local_daemon_peer = false;
            assert!(view.git_input().is_none(), "remote peers run no local Git");
            view.live.local_daemon_peer = true;
            view.selected_endpoint = 0;
            view.live.status = crate::state::ConnectionStatus::Disconnected;
            assert!(view.git_input().is_none());
        })
    });
}
