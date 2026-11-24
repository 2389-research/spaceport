//! Which copy of a host's live state is the current one.
use super::Host;
use spaceport_live::LiveState;

/// The current state of host `index`: the window's own copy for the selected
/// host, the host's own otherwise. The window copies the selected host's state
/// into its own when the selection or the connection moves on; from then on
/// only the window's copy follows the selected connection, because the window
/// stamps it with what it has queued there since.
///
/// `host` reads host `index`, which a caller with a list of hosts passes as
/// `|index| &hosts[index].host`. Panics when `index` is not a host, as indexing
/// the list does.
pub fn live_of<'a>(
    selected: usize,
    selected_live: &'a LiveState,
    index: usize,
    host: impl FnOnce(usize) -> &'a Host,
) -> &'a LiveState {
    if index == selected {
        selected_live
    } else {
        host(index).live()
    }
}
