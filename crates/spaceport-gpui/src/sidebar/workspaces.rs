//! How each workspace row is named and marked, and which rows a folded group
//! shows: the grouping and the row order are `spaceport-sidebar`'s, and what a
//! row says — its label, its note, its mute — is the app's, which words it for
//! the reader's language.
//!
//! Collapsing changes this client's view only, never the daemon's state, so a
//! checkout the daemon restored while it was unavailable keeps its group.

use super::{RowBadge, row::first_text};
use spaceport_client::protocol::ClientShellWorkspace;

pub(crate) use spaceport_sidebar::workspaces::{
    displayed_workspace_status, visible_workspace_entries, workspace_entries,
};

/// Note and mute marks for a workspace row. Notes belong to a checkout;
/// every workspace can carry a mute.
pub(super) fn workspace_badge((note, muted): (Option<&str>, bool)) -> Option<RowBadge> {
    RowBadge::new(note.is_some(), muted)
}

/// The note the user keeps on a workspace's checkout, if it has one.
pub(super) fn workspace_note<'a>(
    workspace: &ClientShellWorkspace,
    notes: Option<&'a crate::worktree_notes::Notes>,
    endpoint: &str,
) -> Option<&'a str> {
    notes?.get(
        endpoint,
        &workspace.worktree.as_ref()?.key,
        workspace.branch.as_deref()?,
    )
}

pub(crate) fn workspace_label(workspace: &ClientShellWorkspace, indented: bool) -> &str {
    let branch = (indented && !workspace.custom_label)
        .then_some(workspace.branch.as_deref())
        .flatten()
        .map(|branch| branch.strip_prefix("worktree/").unwrap_or(branch));
    // The fallback, for a workspace with no label at all, stays English
    // while other areas borrow the label as `&str`.
    // i18n: data
    first_text([branch, Some(&workspace.label)], "workspace")
}
