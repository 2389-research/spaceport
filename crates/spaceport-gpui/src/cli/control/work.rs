//! `notes`, `worktree`, and `wait`.
use super::{Args, CliError, ControlCommand, MAX_WAIT, WorkOp, unknown};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WaitFor {
    Window,
    TabClosed(u64),
    Online,
}

/// Longest text accepted for a note before the app applies its own limits.
const MAX_TEXT: usize = 16 * 1024;

fn bounded(text: String) -> Result<String, CliError> {
    if text.len() > MAX_TEXT {
        return Err(CliError::InvalidValue("text over 16 KiB"));
    }
    Ok(text)
}

pub(super) const WORKTREE_USAGE: &str = "spaceport worktree [status]\n    Whether the caller's worktree is muted.\nspaceport worktree mute|unmute\n    Silence its notifications, or let them through again. Mutes stay in\n    this app; nothing reaches the daemon.";

pub(super) fn worktree(args: &mut Args) -> Result<WorkOp, CliError> {
    let Some(verb) = args.verb() else {
        return Ok(WorkOp::MuteStatus);
    };
    Ok(match verb.to_str() {
        Some("status") => WorkOp::MuteStatus,
        Some("mute") => WorkOp::Mute { muted: true },
        Some("unmute") => WorkOp::Mute { muted: false },
        _ => return Err(unknown("worktree", verb)),
    })
}

pub(super) const NOTES_USAGE: &str = "spaceport notes [list]\n    Every worktree note, most recently edited first.\nspaceport notes get|clear\nspaceport notes set TEXT...\n    The note on the caller's checkout (its repository and branch), or the\n    focused workspace's outside a Herdr pane. Notes are one line of at most\n    500 characters. `list` and `get` exit 4 when there is none.";

pub(super) fn notes(args: &mut Args) -> Result<WorkOp, CliError> {
    let Some(verb) = args.verb() else {
        return Ok(WorkOp::Notes);
    };
    Ok(match verb.to_str() {
        Some("list") => WorkOp::Notes,
        Some("get") => WorkOp::Note,
        Some("set") => WorkOp::SetNote {
            text: bounded(args.words()?.ok_or(CliError::MissingArgument {
                command: "notes set",
                what: "TEXT",
            })?)?,
        },
        // Empty text clears the note.
        Some("clear") => WorkOp::SetNote {
            text: String::new(),
        },
        _ => return Err(unknown("notes", verb)),
    })
}

pub(super) const WAIT_USAGE: &str = "spaceport wait window|online|tab-closed ID [--timeout SECONDS]\n    Wait (default 60, at most 600 seconds). Exits 4 when time runs out.";

pub(super) fn wait(args: &mut Args) -> Result<ControlCommand, CliError> {
    Ok(ControlCommand::Wait {
        timeout: args.number("--timeout", 1..=MAX_WAIT)?.unwrap_or(60),
        what: wait_for(args)?,
    })
}

fn wait_for(args: &mut Args) -> Result<WaitFor, CliError> {
    let verb = args.verb().ok_or(CliError::MissingArgument {
        command: "wait",
        what: "window|tab-closed|online",
    })?;
    Ok(match verb.to_str() {
        Some("window") => WaitFor::Window,
        Some("online") => WaitFor::Online,
        Some("tab-closed") => WaitFor::TabClosed(
            args.required_text("ID")?
                .parse()
                .map_err(|_| CliError::InvalidValue("ID"))?,
        ),
        _ => return Err(unknown("wait", verb)),
    })
}
