//! Which copy of a host's live state is current: the window's own for the
//! selected endpoint, the endpoint's for every other. The fixture makes the
//! selected endpoint's own copy differ from the window's, as it may between
//! syncs, so a reader of the wrong copy shows.
use super::*;
use crate::{
    overview::Open,
    sidebar::layout_tests::{fixture_window, snapshot},
};
use gpui::{Entity, TestAppContext, VisualTestContext};

const HOST: &str = "ssh:homelab";

/// A connected state whose daemon booted as `boot` and whose agent panes
/// carry `boot` as a prefix.
fn booted(boot: &str) -> LiveState {
    let mut snapshot = snapshot(1);
    snapshot.boot_id = boot.into();
    for agent in &mut snapshot.agents {
        agent.pane_id = format!("{boot}-{}", agent.pane_id);
    }
    let mut live = LiveState::default();
    live.snapshot = Some(Arc::new(snapshot));
    live.status = ConnectionStatus::Connected;
    live
}

fn boot(live: &LiveState) -> &str {
    live.snapshot.as_deref().map_or("", |s| s.boot_id.as_str())
}

/// Local, selected, and a remote host. Local's own copy is `stale` while the
/// window's is `window`; the remote's own is `remote`.
fn two_hosts(cx: &mut TestAppContext) -> (Entity<SheprdWindow>, &mut VisualTestContext) {
    cx.add_window_view(|window, cx| {
        let mut view = fixture_window(window, cx);
        let mut remote = Endpoint::new(
            HOST.into(),
            "Homelab".into(),
            ConnectTarget::Ssh {
                target: "me@homelab".into(),
                session: "default".into(),
            },
            true,
        );
        remote.never_connect();
        remote.live = booted("remote");
        view.endpoints.push(remote);
        view.endpoints[0].live = booted("stale");
        view.live = booted("window");
        view
    })
}

#[gpui::test]
fn live_of_is_the_windows_copy_for_the_selected_endpoint_only(cx: &mut TestAppContext) {
    let (view, cx) = two_hosts(cx);
    cx.update(|_, cx| {
        let view = view.read(cx);
        assert_eq!(boot(view.live_of(0)), "window");
        assert_eq!(boot(view.live_of(1)), "remote");
    });
    // The choice follows the selection, not the endpoint.
    cx.update(|_, cx| {
        view.update(cx, |view, _| view.selected_endpoint = 1);
        let view = view.read(cx);
        assert_eq!(boot(view.live_of(0)), "stale");
        assert_eq!(boot(view.live_of(1)), "window");
    });
}

#[gpui::test]
fn hosts_yields_every_endpoint_once_in_order_with_its_current_state(cx: &mut TestAppContext) {
    let (view, cx) = two_hosts(cx);
    cx.update(|_, cx| {
        let view = view.read(cx);
        let hosts: Vec<_> = view
            .hosts()
            .map(|(index, endpoint, live)| (index, endpoint.id.as_str(), boot(live)))
            .collect();
        assert_eq!(hosts, [(0, LOCAL, "window"), (1, HOST, "remote")]);
    });
}

#[gpui::test]
fn the_overview_lists_the_selected_host_from_the_windows_copy(cx: &mut TestAppContext) {
    let (view, cx) = two_hosts(cx);
    let panes: Vec<String> = cx.update(|_, cx| {
        view.read(cx)
            .overview_rows(cx)
            .into_iter()
            .map(|row| {
                let Open::Pane { pane, .. } = row.open;
                pane
            })
            .collect()
    });
    assert_eq!(
        panes,
        ["window-p0", "window-p1", "remote-p0", "remote-p1"],
        "the selected host's rows come from the window's copy"
    );
}
