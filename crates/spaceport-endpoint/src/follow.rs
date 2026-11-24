//! Whose focus a background host may take.
//!
//! `herdr agent focus` and its kin move every client of the daemon, so the
//! connection a window keeps to a background host sees the move in its
//! snapshot. Herdr sends no event for it, and names no asker, so the snapshot
//! is all there is to go by.

use spaceport_client::protocol::ClientShellSnapshot;

/// Whether going from `old` to `new` shows someone asking for a focus.
///
/// Only a move between two tabs that both still exist counts. Herdr keeps a
/// tab per client, and a background connection may not navigate, so nothing
/// but a public focus moves it there. A closed tab moves it too, and a new
/// tab or workspace takes focus as it opens, which a script or dispatch does
/// on a host nobody is watching; neither should take the window. The pane
/// within a tab is shared by the daemon's clients, so a move inside one may
/// be someone working in a terminal on that host, and is not followed.
pub fn focus_requested(old: &ClientShellSnapshot, new: &ClientShellSnapshot) -> bool {
    let (Some(from), Some(to)) = (old.focused_tab_id.as_deref(), new.focused_tab_id.as_deref())
    else {
        return false;
    };
    let lists =
        |snapshot: &ClientShellSnapshot, id: &str| snapshot.tabs.iter().any(|tab| tab.tab_id == id);
    old.boot_id == new.boot_id && from != to && lists(new, from) && lists(old, to)
}
