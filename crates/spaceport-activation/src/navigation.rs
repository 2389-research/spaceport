// ABOUTME: Names where a focus request is aimed, as a closed set of targets.
// ABOUTME: One type for the activation fence, a notice's click and a Go To row.
//! Where a focus request is aimed. A closed set of targets, so navigation is
//! never a boolean-plus-ID pair or a string tag.
//!
//! A surface activation carries one, a notice resolves its click to one, and a
//! palette row for a Go To destination carries one. The desktop reaches it
//! through its `crate::navigation` façade, the phone through its projection.

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NavigationTarget<T> {
    Workspace(T),
    Tab(T),
    Pane(T),
}

pub type OwnedNavigationTarget = NavigationTarget<String>;

impl<T: AsRef<str>> NavigationTarget<T> {
    /// Borrow the payload, as `Option::as_deref` does. Not `AsRef`, which
    /// returns a reference to the same value rather than a new target.
    pub fn as_deref(&self) -> NavigationTarget<&str> {
        match self {
            Self::Workspace(id) => NavigationTarget::Workspace(id.as_ref()),
            Self::Tab(id) => NavigationTarget::Tab(id.as_ref()),
            Self::Pane(id) => NavigationTarget::Pane(id.as_ref()),
        }
    }
}

/// Take ownership of a borrowed target. `ToOwned` is not usable here: its
/// blanket `Clone` impl already covers this type and would hand back a
/// borrowing target instead of an owning one.
impl<T: AsRef<str>> From<&NavigationTarget<T>> for OwnedNavigationTarget {
    fn from(target: &NavigationTarget<T>) -> Self {
        match target.as_deref() {
            NavigationTarget::Workspace(id) => Self::Workspace(id.to_owned()),
            NavigationTarget::Tab(id) => Self::Tab(id.to_owned()),
            NavigationTarget::Pane(id) => Self::Pane(id.to_owned()),
        }
    }
}

use spaceport_client::protocol::ClientShellSnapshot;

impl<T: AsRef<str>> NavigationTarget<T> {
    /// Whether the snapshot has already focused this destination.
    pub fn is_focused_in(&self, snapshot: &ClientShellSnapshot) -> bool {
        match self.as_deref() {
            NavigationTarget::Workspace(id) => snapshot.focused_workspace_id.as_deref() == Some(id),
            NavigationTarget::Tab(id) => snapshot.focused_tab_id.as_deref() == Some(id),
            NavigationTarget::Pane(id) => snapshot.focused_pane_id.as_deref() == Some(id),
        }
    }

    /// Whether the destination sits in the focused workspace and tab, so that
    /// focusing one level per request reaches it.
    pub fn reachable_in(&self, snapshot: &ClientShellSnapshot) -> bool {
        let workspace = snapshot.focused_workspace_id.as_deref();
        let tab = snapshot.focused_tab_id.as_deref();
        match self.as_deref() {
            NavigationTarget::Workspace(id) => snapshot
                .workspaces
                .iter()
                .any(|candidate| candidate.workspace_id == id),
            NavigationTarget::Tab(id) => snapshot.tabs.iter().any(|candidate| {
                candidate.tab_id == id && Some(candidate.workspace_id.as_str()) == workspace
            }),
            NavigationTarget::Pane(id) => snapshot.panes.iter().any(|candidate| {
                candidate.pane_id == id
                    && Some(candidate.tab_id.as_str()) == tab
                    && Some(candidate.workspace_id.as_str()) == workspace
            }),
        }
    }
}
