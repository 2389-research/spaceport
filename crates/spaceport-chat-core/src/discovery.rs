// ABOUTME: Decides which daemon's chat programs the window may attach to, and how.
// ABOUTME: The route from a connection to its program, and the trust boundary that allows it.
//! Which chat programs a window may attach to, and how it reaches them. A
//! chat's endpoint is derived in this user's own directory, never read from
//! daemon data, so only a daemon on this machine that this user owns — or one
//! an SSH connection runs, reached as that host's user — has a route at all.
//! The connection's live state says whether it is still there; the panes
//! themselves are the app's to name.

use spaceport_client::ConnectTarget;
use spaceport_live::LiveState;

/// How a chat's program is reached.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Route {
    /// Its socket in this user's private directory on this machine.
    Local,
    /// `herdr-acp connect` on the SSH host the daemon runs on, carried over
    /// `ssh`; the program there is reached as that host's user.
    Ssh(String),
}

/// Whether the daemon behind `target` runs on this machine as this user, so
/// its panes' chat endpoints are this user's own: the trust boundary local Git
/// uses too. A socket given by path may be a forward from another
/// host, and every remote or cloud daemon's programs listen elsewhere.
///
/// Windows cannot verify the daemon's peer, so there the check falls to the
/// chat pipe itself, which is refused unless this user owns it.
fn local(target: &ConnectTarget, live: &LiveState) -> bool {
    match target {
        ConnectTarget::Local | ConnectTarget::Session { .. } => {
            (live.local_daemon_peer || cfg!(windows)) && live.status.is_connected()
        }
        _ => false,
    }
}

/// How the window reaches the chat programs of the daemon behind `target`:
/// on this machine when the daemon is this user's own, or over SSH on the host
/// an SSH connection runs the daemon on. A socket given by path, a WSL
/// distribution, and a cloud machine have no route yet.
pub fn route(target: &ConnectTarget, live: &LiveState) -> Option<Route> {
    match target {
        _ if local(target, live) => Some(Route::Local),
        ConnectTarget::Ssh { target, .. } if live.status.is_connected() => {
            Some(Route::Ssh(target.clone()))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests;
