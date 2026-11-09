#![allow(clippy::unwrap_used)]
use super::*;
use crate::{
    browser::{LocalFile, OrchestratorRepo, Scope, WebUrl},
    control::protocol::Request,
    sidebar::layout_tests::{fixture_window, snapshot},
};
use gpui::{Entity, VisualTestContext};
use std::{path::Path, sync::Arc};

mod saved_tabs;
mod windows;

fn web(url: &str) -> Location {
    Location::Web {
        url: WebUrl::try_from(url).unwrap(),
    }
}

#[test]
fn tabs_are_listed_by_kind_with_what_they_show() {
    let scope = Scope::local(Path::new("/tmp/herdr.sock"));
    let mut store = Store::default();
    // Hidden folders are never served, so the folder has a plain name.
    let folder = tempfile::Builder::new().prefix("page").tempdir().unwrap();
    let page = folder.path().join("index.html");
    std::fs::write(&page, "<p>hi</p>").unwrap();
    let locations = [
        None,
        Some(web("https://a.test/docs")),
        Some(Location::Local {
            file: LocalFile::new(&page, None).unwrap(),
        }),
        Some(Location::Orchestrator {
            repo: OrchestratorRepo::new("/src/app".into()).unwrap(),
        }),
        Some(Location::Devices),
    ];
    for location in locations {
        store.open(scope.clone(), "w_1", location, Some("w_1:p1".into()));
    }
    let listed: Vec<TabInfo> = store.tabs().filter_map(info).collect();
    let kinds: Vec<TabKind> = listed.iter().map(|tab| tab.kind).collect();
    assert_eq!(
        kinds,
        [
            TabKind::Blank,
            TabKind::Web,
            TabKind::File,
            TabKind::Orchestrator,
            TabKind::Devices,
        ]
    );
    let locations: Vec<Option<&str>> = listed.iter().map(|tab| tab.location.as_deref()).collect();
    assert_eq!(locations[0], None);
    assert_eq!(locations[1], Some("https://a.test/docs"));
    assert!(locations[2].unwrap().ends_with("index.html"));
    assert_eq!(locations[3], Some("/src/app"));
    assert_eq!(locations[4], None);
    let first = &listed[1];
    assert_eq!(first.workspace, "w_1");
    assert_eq!(first.host, "local:/tmp/herdr.sock");
    assert_eq!(first.opened_by.as_deref(), Some("w_1:p1"));
    assert_eq!(first.title, "a.test");
}

#[test]
fn retired_source_is_never_listed_even_with_an_editor_place() {
    let tab: Tab = serde_json::from_value(serde_json::json!({
        "id": 7,
        "scope": "local",
        "workspace_id": "w",
        "location": {"kind": "code", "file": {"path": "/src/main.rs", "line": 4}},
        "title": "Source",
        "place": "px0"
    }))
    .unwrap();
    assert!(info(&tab).is_none());
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
        Store::update(cx, |store| store.open(scope, workspace, location, None)).unwrap()
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
