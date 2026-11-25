// ABOUTME: Builds the snapshots and activations the fence and route tests drive.
// ABOUTME: Keeps the fixtures in one place so each topic file states only its rule.
use spaceport_client::protocol::*;

/// The protocol fixture's snapshot: one workspace, one tab, one pane, focused,
/// at revision 7 on `boot-v1`. `unwrap_or_else` rather than `unwrap`, which this
/// workspace denies in every target, tests included.
pub(super) fn snapshot() -> ClientShellSnapshot {
    serde_json::from_str(include_str!(
        "../../spaceport-protocol/tests/fixtures/endpoint-snapshot-v1.json"
    ))
    .unwrap_or_else(|error| panic!("the protocol fixture must read: {error}"))
}

/// The same snapshot with a second workspace, tab and pane named
/// `infra`/`deploy`/`watch`, still focused on `w1`, so a route to `watch` has
/// three levels to walk.
pub(super) fn two_workspaces() -> ClientShellSnapshot {
    let mut snapshot = snapshot();
    let mut workspace = snapshot.workspaces[0].clone();
    workspace.workspace_id = "infra".into();
    snapshot.workspaces.push(workspace);
    let mut tab = snapshot.tabs[0].clone();
    tab.workspace_id = "infra".into();
    tab.tab_id = "deploy".into();
    snapshot.tabs.push(tab);
    let mut pane = snapshot.panes[0].clone();
    pane.workspace_id = "infra".into();
    pane.tab_id = "deploy".into();
    pane.pane_id = "watch".into();
    snapshot.panes.push(pane);
    snapshot
}

/// A surface frame presenting `revision` on `boot_v1`'s boot, as a daemon
/// answers an activated surface with.
pub(super) fn surface(revision: u64) -> PaneSurfaceFrame {
    let rect = SurfaceRect {
        x: 0,
        y: 0,
        width: 2,
        height: 2,
    };
    PaneSurfaceFrame {
        boot_id: "boot-v1".into(),
        projection_revision: revision,
        surface_revision: 1,
        frame: FrameData {
            cells: vec![CellData {
                symbol: "a".into(),
                fg: 1,
                bg: 0,
                modifier: 0,
                skip: false,
                hyperlink: None,
            }],
            width: 1,
            height: 1,
            cursor: None,
            hyperlinks: vec![],
            graphics: vec![],
        },
        panes: vec![PaneSurfacePane {
            pane_id: "w1:p1".into(),
            content_revision: 1,
            rect,
            inner_rect: rect,
            scrollbar_rect: None,
            scroll: None,
            focused: true,
            mouse_reporting: false,
            sgr_pixel_mouse: false,
            alternate_screen_active: false,
            pixel_width: 16,
            pixel_height: 32,
        }],
        splits: vec![],
        popup: None,
        graphics: Default::default(),
    }
}

/// The acknowledgement a barrier is answered with, for one revision.
pub(super) fn acknowledgement(active: bool, revision: u64) -> serde_json::Value {
    serde_json::json!({
        "type": "client_shell_surface_set",
        "active": active,
        "projection_revision": revision,
    })
}

mod fence;
mod route;
