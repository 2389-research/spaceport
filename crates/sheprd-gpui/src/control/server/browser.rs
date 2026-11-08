//! Browser tabs, page notes, and the agents waiting for them.
use super::super::{
    Target, main_windows, plain, plain_path,
    protocol::{
        Caller, ErrorCode, FeedbackRequest, MAX_WAIT_SECONDS, NotesRequest, NotesTo, Response,
    },
    socket, valid_caller,
};
use crate::browser::{Feedback, FeedbackKey, Scope};
use gpui::App;
use std::path::Path;

/// Reloads every page the calling pane opened, in every window.
pub(super) fn reload(caller: &Caller, cx: &mut App) -> Response {
    let Some(pane) = caller.pane_id.as_deref().filter(|pane| plain(pane, 256)) else {
        return Response::error(
            ErrorCode::InvalidRequest,
            "browser reload runs in the Herdr pane that opened the page",
        );
    };
    let tabs: Vec<_> = cx
        .try_global::<crate::browser::Store>()
        .map(|store| store.opened_by(pane).map(|tab| tab.id).collect())
        .unwrap_or_default();
    for handle in main_windows(cx) {
        let _ = handle.update(cx, |view, _, _| view.reload_browser_tabs(&tabs));
    }
    Response::Reloaded { tabs: tabs.len() }
}

/// Notes are pasted into an agent's prompt, so they keep their line breaks
/// and lose every other control character: none can end the paste early.
fn pastable(text: &str) -> String {
    text.chars()
        .filter(|c| matches!(c, '\n' | '\t') || !crate::notifications::unsafe_char(*c))
        .collect()
}

/// Hands notes from outside a window to the agent in the caller's pane: the
/// window showing that pane delivers them as it does page notes, and with
/// none they are kept for `browser feedback`.
pub(super) fn notes(request: &NotesRequest, cx: &mut App) -> Response {
    let caller = &request.caller;
    let Some(pane) = caller.pane_id.as_deref().filter(|_| valid_caller(caller)) else {
        return Response::error(
            ErrorCode::InvalidRequest,
            "Notes are sent from the Herdr pane of the agent they are for",
        );
    };
    let Some(daemon) = caller.daemon_socket.as_deref() else {
        return Response::error(
            ErrorCode::InvalidRequest,
            "Notes require the caller's daemon socket",
        );
    };
    let recipient = FeedbackKey {
        scope: Scope::local(Path::new(daemon)),
        pane_id: pane.to_owned(),
    };
    let text = pastable(&request.text);
    if text.trim().is_empty() {
        return Response::error(ErrorCode::InvalidRequest, "There are no notes to send");
    }
    // One take hands out at most this much, so a larger batch could never
    // be fetched.
    if crate::browser::json_size(&text) > crate::browser::TAKE_BUDGET {
        return Response::error(
            ErrorCode::InvalidRequest,
            "These notes are too long to send",
        );
    }
    let target = Target {
        daemon: caller.daemon_socket.as_deref().map(Path::new),
        workspace: None,
        pane: Some(pane),
    };
    for handle in main_windows(cx) {
        let taken = handle.update(cx, |view, _, cx| {
            view.deliver_requested_notes(&target, &text, cx)
        });
        if let Ok(Some(to)) = taken {
            return Response::NotesSent { to };
        }
    }
    let feedback = cx.default_global::<Feedback>();
    let to = if feedback.is_waiting(&recipient) {
        NotesTo::Agent
    } else {
        NotesTo::Kept
    };
    feedback.keep(
        crate::browser::Batch {
            target: recipient,
            text,
        },
        None,
    );
    Response::NotesSent { to }
}

/// An agent waiting in `browser feedback --wait`.
pub(super) struct Waiter {
    target: FeedbackKey,
    until: std::time::Instant,
    incoming: socket::Incoming,
}

/// At most this many agents wait at once; the socket bounds connections too.
const MAX_WAITERS: usize = 8;

pub(super) fn feedback(
    request: &FeedbackRequest,
    incoming: socket::Incoming,
    waiters: &mut Vec<Waiter>,
    cx: &mut App,
) {
    if !plain(&request.pane_id, 256) {
        incoming.respond(Response::error(ErrorCode::InvalidRequest, "Invalid pane"));
        return;
    }
    let Some(daemon) = request
        .daemon_socket
        .as_deref()
        .filter(|path| plain_path(path))
    else {
        incoming.respond(Response::error(
            ErrorCode::InvalidRequest,
            "Feedback requires the caller's daemon socket",
        ));
        return;
    };
    let target = FeedbackKey {
        scope: Scope::local(Path::new(daemon)),
        pane_id: request.pane_id.clone(),
    };
    let kept = take_feedback(&target, cx);
    if kept.is_some() || request.wait_seconds == 0 {
        respond_feedback(incoming, target, kept, cx);
        return;
    }
    if waiters.len() >= MAX_WAITERS {
        incoming.respond(Response::error(
            ErrorCode::Busy,
            "Too many agents are waiting for notes",
        ));
        return;
    }
    // A newer wait from the same pane replaces the older one.
    if let Some(index) = waiters.iter().position(|waiter| waiter.target == target) {
        waiters
            .remove(index)
            .incoming
            .respond(Response::Feedback { text: None });
    }
    waiters.push(Waiter {
        target,
        until: std::time::Instant::now()
            + std::time::Duration::from_secs(request.wait_seconds.min(MAX_WAIT_SECONDS)),
        incoming,
    });
}

/// Takes what is kept for `target`. A tab shows its batch as kept until an
/// agent collects it, so the windows redraw once one does.
fn take_feedback(target: &FeedbackKey, cx: &mut App) -> Option<String> {
    let text = cx.default_global::<Feedback>().take(target);
    if text.is_some() {
        cx.refresh_windows();
    }
    text
}

fn respond_feedback(
    incoming: socket::Incoming,
    target: FeedbackKey,
    text: Option<String>,
    cx: &mut App,
) {
    if let Err(Response::Feedback { text: Some(text) }) =
        incoming.try_respond(Response::Feedback { text })
    {
        // Back first in line: it was the oldest when it was taken.
        cx.default_global::<Feedback>()
            .keep_again(crate::browser::Batch { target, text });
    }
}

/// Hands sent notes to waiting agents and releases the ones whose wait ran
/// out, then tells the windows who is still waiting.
pub(super) fn serve_waiters(waiters: &mut Vec<Waiter>, cx: &mut App) {
    let now = std::time::Instant::now();
    let mut still = Vec::with_capacity(waiters.len());
    for waiter in waiters.drain(..) {
        if !waiter.incoming.is_live() {
            continue;
        }
        let has = cx
            .try_global::<Feedback>()
            .is_some_and(|feedback| feedback.has(&waiter.target));
        if has {
            let text = take_feedback(&waiter.target, cx);
            respond_feedback(waiter.incoming, waiter.target, text, cx);
        } else if now >= waiter.until {
            waiter.incoming.respond(Response::Feedback { text: None });
        } else {
            still.push(waiter);
        }
    }
    *waiters = still;
    let panes: Vec<FeedbackKey> = waiters.iter().map(|waiter| waiter.target.clone()).collect();
    let current = cx.try_global::<Feedback>().map(Feedback::waiting);
    if current.is_none_or(|current| current != panes.as_slice()) {
        cx.default_global::<Feedback>().set_waiting(panes);
    }
}

#[cfg(test)]
mod tests;
