//! Worktree notes and notification mutes.
//!
//! Per-checkout requests resolve the caller's workspace on its named daemon,
//! or the focused workspace when the caller names none. Notes use endpoint,
//! repository and branch identity so a restarted daemon cannot retarget them.
mod mute;
mod notes;
#[cfg(test)]
mod tests;

use super::{Watches, in_window, not_found, socket, target};
use crate::{
    SheprdWindow,
    control::protocol::{Call, Caller, ErrorCode, Response, work::WorkOp},
};
use gpui::App;
use sheprd_client::protocol::{ClientShellSnapshot, ClientShellWorkspace};

pub(super) fn serve(call: Call<WorkOp>, incoming: socket::Incoming, _: &mut Watches, cx: &mut App) {
    let Call { caller, op } = call;
    let response = match op {
        WorkOp::Notes => notes::list(cx),
        WorkOp::Note => notes::get(&caller, cx),
        WorkOp::SetNote { text } => notes::set(&caller, &text, cx),
        WorkOp::MuteStatus => in_window(&caller, cx, |view, _, cx| mute::status(view, &caller, cx)),
        WorkOp::Mute { muted } => in_window(&caller, cx, |view, _, cx| {
            mute::set(view, &caller, muted, cx)
        }),
    };
    incoming.respond(response);
}

/// The caller's workspace as one window shows it.
pub(super) struct Found<'a> {
    /// The endpoint showing it, by index into the window's endpoints.
    pub index: usize,
    pub snapshot: &'a ClientShellSnapshot,
    pub workspace: &'a ClientShellWorkspace,
}

/// Finds the caller's workspace in `view`: on the caller's own daemon only
/// when it names one, else on any endpoint showing a workspace of that ID.
pub(super) fn find<'a>(view: &'a SheprdWindow, caller: &Caller) -> Result<Found<'a>, Response> {
    let target = target(caller);
    let (index, id) = target
        .strictness()
        .iter()
        .find_map(|strict| view.control_target(&target, *strict))
        .ok_or_else(|| match &caller.workspace_id {
            Some(id) => Response::error(
                ErrorCode::WorkspaceNotFound,
                format!("No window shows workspace {id}"),
            ),
            None => not_found("The window shows no workspace"),
        })?;
    let live = if index == view.selected_endpoint {
        &view.live
    } else {
        &view.endpoints[index].live
    };
    let snapshot = live
        .snapshot
        .as_deref()
        .ok_or_else(|| not_found("The window has no state from that host yet"))?;
    let workspace = snapshot
        .workspaces
        .iter()
        .find(|workspace| workspace.workspace_id == id)
        .ok_or_else(|| not_found(format!("No window shows workspace {id}")))?;
    Ok(Found {
        index,
        snapshot,
        workspace,
    })
}
