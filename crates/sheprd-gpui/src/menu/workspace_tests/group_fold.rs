// ABOUTME: Pins that the workspace menu folds a repository on the selected host only.
// ABOUTME: Reads the fold back through the rows the sidebar shortcuts step through.
use super::*;
use crate::{sidebar::layout_tests::REPO_KEY, state::ConnectionStatus};
use sheprd_client::ConnectTarget;
use std::sync::Arc;

/// The hosts whose repository group hides its linked worktrees.
fn folded(view: &SheprdWindow) -> Vec<String> {
    let rows = view.sidebar_workspaces();
    view.endpoints
        .iter()
        .enumerate()
        .filter(|(index, _)| !rows.contains(&(*index, "w4")))
        .map(|(_, endpoint)| endpoint.id.clone())
        .collect()
}

#[gpui::test]
fn the_menu_folds_the_group_on_the_selected_host_only(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(sidebar::layout_tests::fixture_window);
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.live.snapshot = Some(Arc::new(sidebar::layout_tests::snapshot(6)));
            view.live.status = ConnectionStatus::Connected;
            view.endpoints[0].live = view.live.clone();
            let mut remote = crate::endpoint::Endpoint::new(
                "ssh:a".into(),
                "ssh:a".into(),
                ConnectTarget::Ssh {
                    target: "example.invalid".into(),
                    session: "default".into(),
                },
                true,
            );
            remote.live = view.live.clone();
            remote.never_connect();
            view.endpoints.push(remote);
            let mut toggle = |view: &mut SheprdWindow| {
                view.open_workspace_menu("w3", Default::default(), window, cx);
                assert!(view.menu.target.is_some());
                view.toggle_selected_group(cx);
            };
            assert!(folded(view).is_empty());

            toggle(view);
            assert_eq!(folded(view), ["local"]);
            assert!(view.collapsed_repos_for_selection().contains(REPO_KEY));

            // Another host has its own folds, whichever is selected.
            view.selected_endpoint = 1;
            view.live = view.endpoints[1].live.clone();
            assert!(!view.collapsed_repos_for_selection().contains(REPO_KEY));
            toggle(view);
            assert_eq!(folded(view), ["local", "ssh:a"]);
            toggle(view);
            assert_eq!(folded(view), ["local"]);

            view.selected_endpoint = 0;
            view.live = view.endpoints[0].live.clone();
            toggle(view);
            assert!(folded(view).is_empty());
        });
    });
}
