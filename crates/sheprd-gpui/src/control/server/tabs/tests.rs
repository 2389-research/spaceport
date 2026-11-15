// ABOUTME: Shares native tab fixtures for local control requests.
// ABOUTME: Covers exact targets, retained state and unavailable Core actions.
#![allow(clippy::unwrap_used)]
use super::*;
use crate::{
    browser::Scope,
    control::protocol::Request,
    sidebar::layout_tests::{fixture_window, snapshot},
};
use gpui::{Entity, VisualTestContext};
use std::{path::Path, sync::Arc};

mod core_flag;
mod saved_tabs;
mod windows;

#[test]
fn blank_tabs_are_not_listed_but_native_tabs_keep_their_targets() {
    let scope = Scope::local(Path::new("/tmp/herdr.sock"));
    let mut store = Store::default();
    for location in [None, Some(Location::Devices)] {
        store.open(scope.clone(), "w_1", location, Some("w_1:p1".into()));
    }
    let listed: Vec<TabInfo> = store.tabs().filter_map(info).collect();
    let kinds: Vec<TabKind> = listed.iter().map(|tab| tab.kind).collect();
    assert_eq!(kinds, [TabKind::Devices]);
    let first = &listed[0];
    assert_eq!(first.location, None);
    assert_eq!(first.workspace, "w_1");
    assert_eq!(first.host, "local:/tmp/herdr.sock");
    assert_eq!(first.opened_by.as_deref(), Some("w_1:p1"));
    assert_eq!(first.title, "Devices");
}

/// The fixture window, showing workspace `w0` of two as a connected daemon
/// would.
fn window(cx: &mut gpui::TestAppContext) -> (Entity<SheprdWindow>, &mut VisualTestContext) {
    cx.add_window_view(|window, cx| {
        let mut view = fixture_window(window, cx);
        let mut shown = snapshot(2);
        shown.focused_workspace_id = Some("w0".into());
        shown.focused_tab_id = Some("t0".into());
        view.live.snapshot = Some(Arc::new(shown));
        view
    })
}

/// Opens a tab in workspace `workspace` of the window's own daemon.
fn open(
    view: &Entity<SheprdWindow>,
    cx: &mut VisualTestContext,
    workspace: &str,
    location: Option<Location>,
) -> TabId {
    cx.cx.update(|cx| {
        let scope = crate::browser::scope(&view.read(cx).endpoints[0]);
        Store::update(cx, |store| {
            store.open(scope, workspace, location.or(Some(Location::Devices)), None)
        })
        .unwrap()
    })
}

/// A request queued on a private socket, and the caller waiting for it.
fn queued(
    server: &socket::Server,
    op: TabOp,
    caller: Caller,
) -> (
    Call<TabOp>,
    socket::Incoming,
    std::thread::JoinHandle<sheprd_control_protocol::Result<Response>>,
) {
    let path = server.path().to_owned();
    let call = Call { caller, op };
    let request = Request::Tab(call.clone());
    let waiting = std::thread::spawn(move || socket::call(&path, &request));
    (call, server.next().unwrap(), waiting)
}
