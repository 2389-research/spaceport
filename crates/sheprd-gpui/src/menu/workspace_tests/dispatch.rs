use super::*;

/// A window on a scriptable local daemon showing `snapshot(7)`, with a
/// second connected host, "Box", that has the same repository open.
fn two_hosts(view: &mut SheprdWindow) {
    let snapshot = std::sync::Arc::make_mut(view.live.snapshot.as_mut().unwrap());
    snapshot.workspaces = sidebar::layout_tests::snapshot(7).workspaces;
    view.live.status = crate::state::ConnectionStatus::Connected;
    view.endpoints[0].connection.target = sheprd_client::ConnectTarget::Local;
    add_host(view, "box", "Box");
}

/// A connected SSH host showing the window's own snapshot.
fn add_host(view: &mut SheprdWindow, name: &str, label: &str) {
    let mut remote = crate::endpoint::Endpoint::new(
        format!("ssh:{name}"),
        label.into(),
        sheprd_client::ConnectTarget::Ssh {
            target: format!("nobody@{name}.invalid"),
            session: "default".into(),
        },
        true,
    );
    remote.live = view.live.clone();
    view.endpoints.push(remote);
}

fn setup_for(endpoint: &str, workspace: &str, now: std::time::Instant) -> crate::dispatch::Setup {
    crate::dispatch::Setup::new(
        endpoint.into(),
        workspace.into(),
        "agent-launcher".into(),
        crate::teleport::NewCheckout {
            repo_key: "/repo/.git".into(),
            path: "/nonexistent/herdr-dispatch-test".into(),
            root: None,
        },
        now,
    )
}

/// A worktree made elsewhere runs its setup once the window shows its host
/// and workspace, as one made here does, and gives up rather than wait.
#[gpui::test]
fn a_dispatched_worktree_sets_up_once_its_host_is_shown(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(sidebar::layout_tests::fixture_window);
    cx.update(|_, cx| {
        view.update(cx, |view, cx| {
            two_hosts(view);
            let now = std::time::Instant::now();
            // Box is not the host shown, and w-new is not listed: both wait.
            view.queue_dispatch_setup(setup_for("ssh:box", "w3", now), cx);
            view.queue_dispatch_setup(setup_for(crate::endpoint::LOCAL, "w-new", now), cx);
            view.poll_dispatch_setup(now, cx);
            assert_eq!(view.dispatch_setups.len(), 2);
            assert!(view.worktree_script.is_none());
            // A later one that is ready starts without dropping those waiting.
            view.queue_dispatch_setup(setup_for(crate::endpoint::LOCAL, "w3", now), cx);
            view.poll_dispatch_setup(now, cx);
            assert!(view.worktree_script.is_some());
            assert_eq!(view.dispatch_setups.len(), 2, "earlier setups keep waiting");
            assert!(view.flash.is_none());
            // Hosts never shown are given up on, with a word.
            view.worktree_script = None;
            view.poll_dispatch_setup(now + std::time::Duration::from_secs(121), cx);
            assert!(view.dispatch_setups.is_empty());
            assert!(view.worktree_script.is_none());
            assert!(view.flash.is_some());
        })
    });
}

#[gpui::test]
fn waiting_setups_are_bounded(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(sidebar::layout_tests::fixture_window);
    cx.update(|_, cx| {
        view.update(cx, |view, cx| {
            let now = std::time::Instant::now();
            for _ in 0..8 {
                view.queue_dispatch_setup(setup_for("ssh:box", "w3", now), cx);
            }
            assert!(view.flash.is_none());
            view.queue_dispatch_setup(setup_for("ssh:box", "w4", now), cx);
            assert_eq!(view.dispatch_setups.len(), 8);
            assert_eq!(view.dispatch_setups.back().unwrap().workspace_id, "w4");
            assert!(
                view.flash.is_some(),
                "the oldest is given up on, with a word"
            );
        })
    });
}
