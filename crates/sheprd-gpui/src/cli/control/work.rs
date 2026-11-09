//! `notes`, `scripts`, and `wait`.
use super::{Args, CliError, Script, unknown};

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
    Feedback,
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

pub(super) fn script_kind(args: &mut Args) -> Result<Option<Script>, CliError> {
    let Some(verb) = args.verb() else {
        return Ok(None);
    };
    Ok(match verb.to_str() {
        Some("list") => None,
        Some("setup") => Some(Script::Setup),
        Some("run") => Some(Script::Run),
        _ => return Err(unknown("scripts", verb)),
    })
}

pub(super) fn wait_for(args: &mut Args) -> Result<WaitFor, CliError> {
    let verb = args.verb().ok_or(CliError::MissingArgument {
        command: "wait",
        what: "feedback|window|tab-closed|online",
    })?;
    Ok(match verb.to_str() {
        Some("feedback") => WaitFor::Feedback,
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
