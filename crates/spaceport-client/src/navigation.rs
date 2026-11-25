//! Where a navigation target is aimed, and whether a snapshot still holds it.
//!
//! A focus request, a Go To row and a notice's click all name one destination:
//! a workspace, a tab or a pane, as a closed set rather than a boolean-plus-ID
//! pair or a string tag. The rules over a snapshot live here, so the window,
//! the live state's activation fence and the phone answer them the same way:
//! whether the snapshot still holds the destination, whether it has already
//! focused it, and whether a client that focuses one level per request can
//! reach it from what it shows.

use spaceport_protocol::ClientShellSnapshot;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NavigationTarget<T> {
    Workspace(T),
    Tab(T),
    Pane(T),
}

pub type OwnedNavigationTarget = NavigationTarget<String>;

/// Why a snapshot no longer holds a navigation target.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Missing {
    /// The snapshot comes from another boot of the daemon than the one the
    /// destination was listed from.
    Session,
    Workspace,
    Tab,
    Pane,
}

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

    /// Whether `snapshot` comes from `boot` and still holds this destination.
    pub fn exists_in(&self, snapshot: &ClientShellSnapshot, boot: &str) -> Result<(), Missing> {
        if boot.is_empty() || boot != snapshot.boot_id {
            return Err(Missing::Session);
        }
        let missing = match self {
            Self::Workspace(id) => (!snapshot
                .workspaces
                .iter()
                .any(|workspace| workspace.workspace_id == id.as_ref()))
            .then_some(Missing::Workspace),
            Self::Tab(id) => {
                (!snapshot.tabs.iter().any(|tab| tab.tab_id == id.as_ref())).then_some(Missing::Tab)
            }
            Self::Pane(id) => (!snapshot
                .panes
                .iter()
                .any(|pane| pane.pane_id == id.as_ref()))
            .then_some(Missing::Pane),
        };
        missing.map_or(Ok(()), Err)
    }

    /// Whether `snapshot` holds this destination inside the workspace and tab
    /// it has focused, so a client that focuses one level per request reaches
    /// it without leaving what it shows.
    pub fn reachable_in(&self, snapshot: &ClientShellSnapshot) -> bool {
        match self {
            Self::Workspace(id) => snapshot
                .workspaces
                .iter()
                .any(|workspace| workspace.workspace_id == id.as_ref()),
            Self::Tab(id) => snapshot.tabs.iter().any(|tab| {
                tab.tab_id == id.as_ref()
                    && Some(tab.workspace_id.as_str()) == snapshot.focused_workspace_id.as_deref()
            }),
            Self::Pane(id) => snapshot.panes.iter().any(|pane| {
                pane.pane_id == id.as_ref()
                    && Some(pane.tab_id.as_str()) == snapshot.focused_tab_id.as_deref()
                    && Some(pane.workspace_id.as_str()) == snapshot.focused_workspace_id.as_deref()
            }),
        }
    }

    /// Whether `snapshot` has already focused this destination.
    pub fn is_focused_in(&self, snapshot: &ClientShellSnapshot) -> bool {
        match self {
            Self::Workspace(id) => snapshot.focused_workspace_id.as_deref() == Some(id.as_ref()),
            Self::Tab(id) => snapshot.focused_tab_id.as_deref() == Some(id.as_ref()),
            Self::Pane(id) => snapshot.focused_pane_id.as_deref() == Some(id.as_ref()),
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
