// ABOUTME: Shares terminal focus and sizing policies between Herdr clients.
// ABOUTME: Classifies snapshot changes and the recent input that permits a resize.
use spaceport_protocol::ClientShellSnapshot;
use std::time::{Duration, Instant};

/// Whether a newer snapshot leaves the displayed terminal safe for input.
pub fn same_input_projection(old: &ClientShellSnapshot, new: &ClientShellSnapshot) -> bool {
    // New protocol fields must make an explicit choice about input invalidation.
    let ClientShellSnapshot {
        boot_id,
        revision: _,
        config_diagnostic: _,
        product_announcement: _,
        update_available: _,
        update_install_command: _,
        server_keybindings_toml,
        latest_release_notes_available: _,
        integration_updates_available: _,
        worktree_directory: _,
        release_notes: _,
        focused_workspace_id,
        focused_tab_id,
        focused_pane_id,
        tab_bar_right: _,
        tab_bar_right_separator: _,
        agent_view_label: _,
        agent_order: _,
        workspaces: _,
        tabs: _,
        panes: _,
        agents: _,
        commands,
    } = new;
    if &old.boot_id != boot_id
        || &old.focused_workspace_id != focused_workspace_id
        || &old.focused_tab_id != focused_tab_id
        || &old.focused_pane_id != focused_pane_id
        || &old.server_keybindings_toml != server_keybindings_toml
        || &old.commands != commands
    {
        return false;
    }
    let zoomed = |snapshot: &ClientShellSnapshot| {
        snapshot
            .tabs
            .iter()
            .find(|tab| Some(&tab.tab_id) == snapshot.focused_tab_id.as_ref())
            .map(|tab| tab.zoomed)
    };
    zoomed(old) == zoomed(new) && input_panes(old).eq(input_panes(new))
}

fn input_panes(
    snapshot: &ClientShellSnapshot,
) -> impl Iterator<Item = (&str, &str, &str, bool, bool)> {
    snapshot
        .panes
        .iter()
        .filter(move |pane| Some(&pane.tab_id) == snapshot.focused_tab_id.as_ref())
        .map(|pane| {
            (
                pane.pane_id.as_str(),
                pane.workspace_id.as_str(),
                pane.tab_id.as_str(),
                pane.focused,
                pane.right_click_passthrough,
            )
        })
}

/// A frame another client took is re-claimed more slowly than a settled
/// window resize, so the request is not resent while the daemon answers it.
pub const RESIZE_REASSERT: Duration = Duration::from_secs(1);

/// How long after its last user input a window still re-claims a frame
/// another client sized. Two windows of different sizes on one tab would
/// otherwise take it from each other every `RESIZE_REASSERT`; the one in use
/// keeps it, and an idle one leaves it alone.
pub const REASSERT_AFTER_INPUT: Duration = Duration::from_secs(5);

/// Whether a window whose last user input was at `last_input` may re-claim,
/// at `now`, a frame another client sized.
pub fn reasserts(last_input: Option<Instant>, now: Instant) -> bool {
    last_input.is_some_and(|last| now.saturating_duration_since(last) < REASSERT_AFTER_INPUT)
}
