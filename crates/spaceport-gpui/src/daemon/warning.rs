//! A usable terminal connection can still fail the local Git trust checks.
//! Keep that diagnosis separate from transport failure until the UI presents it.
//! The class and the socket it refused are `spaceport-client`'s, read here in
//! the words the window shows.
use super::{LocalPeerWarning, UntrustedEndpoint};
use crate::i18n::fl;

pub(crate) fn notice(
    warning: &LocalPeerWarning,
    now: std::time::Instant,
) -> crate::notifications::Notice {
    use spaceport_client::protocol::{SemanticNotification, SemanticNotificationKind};
    let (title, body) = (fl!("daemon-warning-title"), body(warning));
    crate::notifications::Notice::local_feedback_multiline(
        SemanticNotification {
            kind: SemanticNotificationKind::NeedsAttention,
            title: title.clone(),
            body: Some(body.clone()),
            sound: None,
            agent: None,
            workspace_id: None,
            tab_id: None,
            pane_id: None,
            position: None,
        },
        now,
        &crate::state::labels(),
    )
    // The cleanup above is for daemon text. This is the app's own: Fluent
    // messages, whose direction marks a right-to-left language needs, and
    // a command whose path `body` already refused when it was unsafe.
    .with_message(title, body)
}

fn body(warning: &LocalPeerWarning) -> String {
    use UntrustedEndpoint::*;
    let reason = warning.reason();
    let socket = warning.socket();
    let path = match reason {
        DirectoryPermissions => socket.parent().unwrap_or(socket),
        _ => socket,
    };
    // Do not turn a lossy, sanitized, or truncated path into a command.
    let quoted = path
        .to_str()
        .filter(|path| path.len() <= 200 && !path.chars().any(crate::notifications::unsafe_char))
        .map(spaceport_client::shell_quote)
        .filter(|quoted| quoted.len() <= 200);
    let command = if quoted.is_some() { "yes" } else { "no" };
    let advice = match reason {
        DirectoryPermissions => fl!("daemon-warning-directory-permissions", command = command),
        SocketPermissions => fl!("daemon-warning-socket-permissions", command = command),
        PeerUser => fl!("daemon-warning-peer-user"),
        NotSocket => fl!("daemon-warning-not-socket"),
        OtherSocket => fl!("daemon-warning-other-socket"),
    };
    // The command is the shell's, the same in every language.
    let mode = match reason {
        DirectoryPermissions => Some("o-w"),
        SocketPermissions => Some("go-w"),
        PeerUser | NotSocket | OtherSocket => None,
    };
    match (mode, quoted) {
        (Some(mode), Some(path)) => format!("{advice}\nchmod -- {mode} {path}"),
        _ => advice,
    }
}

#[cfg(test)]
mod tests;
