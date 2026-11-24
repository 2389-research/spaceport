//! A focus asked for on a background host is told from its snapshot, and
//! nothing else a background host does may ask for one.
use super::*;
use spaceport_client::protocol::ClientShellSnapshot;

/// A snapshot on tab `tab` of one workspace, with both of its tabs open.
fn on_tab(tab: &str, revision: u64) -> ClientShellSnapshot {
    let mut snapshot = snapshot();
    let mut second = snapshot.tabs[0].clone();
    snapshot.tabs[0].workspace_id = "w0".into();
    snapshot.tabs[0].tab_id = "w0:t0".into();
    second.workspace_id = "w0".into();
    second.tab_id = "w0:t1".into();
    snapshot.tabs.push(second);
    snapshot.focused_workspace_id = Some("w0".into());
    snapshot.focused_tab_id = Some(tab.into());
    snapshot.revision = revision;
    snapshot
}

#[test]
fn a_move_between_existing_tabs_is_a_focus_asked_for() {
    assert!(focus_requested(&on_tab("w0:t0", 1), &on_tab("w0:t1", 2)));
}

#[test]
fn staying_on_a_tab_asks_for_nothing() {
    assert!(!focus_requested(&on_tab("w0:t0", 1), &on_tab("w0:t0", 2)));
    // The daemon shares a tab's focused pane among its clients: this may be
    // someone at a terminal on the host.
    let mut other_pane = on_tab("w0:t0", 2);
    other_pane.focused_pane_id = Some("w0:p2".into());
    assert!(!focus_requested(&on_tab("w0:t0", 1), &other_pane));
}

#[test]
fn closing_the_focused_tab_is_not_a_focus_asked_for() {
    let mut closed = on_tab("w0:t1", 2);
    closed.tabs.retain(|tab| tab.tab_id != "w0:t0");
    assert!(!focus_requested(&on_tab("w0:t0", 1), &closed));
}

#[test]
fn a_tab_opening_with_focus_does_not_take_the_window() {
    let mut before = on_tab("w0:t0", 1);
    before.tabs.retain(|tab| tab.tab_id != "w0:t1");
    assert!(!focus_requested(&before, &on_tab("w0:t1", 2)));
}

#[test]
fn a_restarted_daemon_is_not_a_focus_asked_for() {
    let mut restarted = on_tab("w0:t1", 2);
    restarted.boot_id = "another-boot".into();
    assert!(!focus_requested(&on_tab("w0:t0", 1), &restarted));
}

#[test]
fn a_snapshot_without_a_focused_tab_asks_for_nothing() {
    let mut none = on_tab("w0:t0", 1);
    none.focused_tab_id = None;
    assert!(!focus_requested(&none, &on_tab("w0:t1", 2)));
    let mut gone = none;
    gone.revision = 3;
    assert!(!focus_requested(&on_tab("w0:t0", 2), &gone));
}
