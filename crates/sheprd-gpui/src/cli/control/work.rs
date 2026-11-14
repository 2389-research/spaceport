//! `notes`, `worktree`, and `wait`.
use super::{Args, CliError, ControlCommand, MAX_WAIT, unknown};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WorktreeCommand {
    /// Whether the caller's worktree is muted.
    Status,
    Mute,
    Unmute,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NotesCommand {
    List,
    Get,
    Set(String),
    Clear,
}

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

pub(super) fn worktree(args: &mut Args) -> Result<WorktreeCommand, CliError> {
    let Some(verb) = args.verb() else {
        return Ok(WorktreeCommand::Status);
    };
    Ok(match verb.to_str() {
        Some("status") => WorktreeCommand::Status,
        Some("mute") => WorktreeCommand::Mute,
        Some("unmute") => WorktreeCommand::Unmute,
        _ => return Err(unknown("worktree", verb)),
    })
}

pub(super) const NOTES_USAGE: &str = "spaceport notes [list]\n    Every worktree note, most recently edited first.\nspaceport notes get|clear\nspaceport notes set TEXT...\n    The note on the caller's checkout (its repository and branch), or the\n    focused workspace's outside a Herdr pane. Notes are one line of at most\n    500 characters. `list` and `get` exit 4 when there is none.";

pub(super) fn notes(args: &mut Args) -> Result<NotesCommand, CliError> {
    let Some(verb) = args.verb() else {
        return Ok(NotesCommand::List);
    };
    Ok(match verb.to_str() {
        Some("list") => NotesCommand::List,
        Some("get") => NotesCommand::Get,
        Some("set") => NotesCommand::Set(bounded(args.words()?.ok_or(
            CliError::MissingArgument {
                command: "notes set",
                what: "TEXT",
            },
        )?)?),
        Some("clear") => NotesCommand::Clear,
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
