//! Worktree notes and notification mutes.
#[cfg(test)]
mod tests;

use super::{Context, Output, unexpected};
use crate::control::protocol::{
    Request, Response,
    work::{MuteInfo, NoteInfo, Reply, WorkOp},
};
use std::fmt::Write as _;

/// A repository's name from its Git common directory: the directory the
/// `.git` is in, or a bare repository's own name.
pub(super) fn repository_name(repo_key: &str) -> &str {
    let key = repo_key.trim_end_matches('/');
    let key = key.strip_suffix("/.git").unwrap_or(key);
    key.rsplit('/')
        .next()
        .filter(|name| !name.is_empty())
        .unwrap_or(repo_key)
}

pub(super) fn describe_note(note: &NoteInfo) -> String {
    format!(
        "{} ({}, {}): {}",
        note.branch,
        repository_name(&note.repository),
        note.host,
        note.text
    )
}

/// Prints what `spaceport notes` answers.
pub(super) fn notes(op: WorkOp, context: &Context) -> crate::Result<Output> {
    match context.call(Request::Work, op)? {
        Response::Done { message } => Ok(Output::message(message)),
        Response::Work {
            reply: Reply::Notes { notes, omitted },
        } => {
            let json = serde_json::json!({ "notes": notes, "omitted": omitted });
            let mut text = String::new();
            for note in &notes {
                let _ = writeln!(text, "{}", describe_note(note));
            }
            if omitted > 0 {
                let _ = writeln!(text, "… and {omitted} older notes that did not fit");
            }
            if notes.is_empty() && omitted == 0 {
                return Ok(Output::data("No worktree notes", &json)?.nothing());
            }
            Output::data(text.trim_end(), &json)
        }
        Response::Work {
            reply: Reply::Note { note: Some(note) },
        } => Output::data(note.text.clone(), &serde_json::json!({ "note": note })),
        Response::Work {
            reply: Reply::Note { note: None },
        } => Ok(Output::data(
            "This checkout has no note",
            &serde_json::json!({ "note": null }),
        )?
        .nothing()),
        response => Err(unexpected(response)),
    }
}

/// What `worktree` says about a mute.
pub(super) fn describe_mute(mute: &MuteInfo) -> String {
    let MuteInfo {
        workspace, host, ..
    } = mute;
    match (mute.muted, mute.host_muted) {
        (true, false) => format!("{workspace} is muted"),
        (true, true) => format!("{workspace} is muted, and so is its host {host}"),
        (false, true) => format!("{workspace} is silent because its host {host} is muted"),
        (false, false) => format!("{workspace} is not muted"),
    }
}

/// Prints what `spaceport worktree` answers.
pub(super) fn worktree(op: WorkOp, context: &Context) -> crate::Result<Output> {
    match context.call(Request::Work, op)? {
        Response::Work {
            reply: Reply::Mute { mute },
        } => Output::data(describe_mute(&mute), &mute),
        response => Err(unexpected(response)),
    }
}
