//! Families with no command enum of their own: `status`, `skill`,
//! `processes`, `usage`, `bell`, `events`, `run`, `commands`, `logs`, and
//! `diagnostics`.
use super::{Args, CliError, ControlCommand, wait_option};

pub(super) const STATUS_USAGE: &str = "spaceport status\n    Whether Spaceport runs, its version, protocol, windows, hosts, and the\n    current Core machine catalog's freshness and count. No machine lookup runs.\n    Exits 3 when it does not run.";

pub(super) fn status(_: &mut Args) -> Result<ControlCommand, CliError> {
    Ok(ControlCommand::Status)
}

pub(super) const SKILL_USAGE: &str =
    "spaceport skill\n    Print the agent skill describing these commands.";

pub(super) fn skill(_: &mut Args) -> Result<ControlCommand, CliError> {
    Ok(ControlCommand::Skill)
}

pub(super) const PROCESSES_USAGE: &str = "spaceport processes [--pane ID]\n    The processes under a pane (default HERDR_PANE_ID), with CPU and\n    memory. Runs here; Spaceport need not run.";

pub(super) fn processes(args: &mut Args) -> Result<ControlCommand, CliError> {
    Ok(ControlCommand::Processes {
        pane: args.text_option("--pane")?,
    })
}

pub(super) const USAGE_USAGE: &str = "spaceport usage\n    Plan usage on the selected host when Core supplies a feed; currently unavailable.";

pub(super) fn usage(_: &mut Args) -> Result<ControlCommand, CliError> {
    Ok(ControlCommand::Usage)
}

pub(super) const BELL_USAGE: &str =
    "spaceport bell\n    Ring the system bell, as a terminal bell does.";

pub(super) fn bell(_: &mut Args) -> Result<ControlCommand, CliError> {
    Ok(ControlCommand::Bell)
}

pub(super) const EVENTS_USAGE: &str = "spaceport events [--after CURSOR] [--wait SECONDS] [--kind KIND]... [--follow]\n    What happened since CURSOR, the INSTANCE:SEQ the last call printed:\n    windows and tabs opening and closing, notes, and host status. A cursor\n    from an earlier app run gets everything this run kept. --follow keeps\n    printing until interrupted. Kinds:\n    window_opened, window_closed, window_focused, tab_opened, tab_closed,\n    note, host.";

pub(super) fn events(args: &mut Args) -> Result<ControlCommand, CliError> {
    let kinds = args
        .options("--kind")?
        .iter()
        .map(|kind| {
            kind.to_str()
                .and_then(|kind| kind.parse().ok())
                .ok_or(CliError::InvalidValue("--kind"))
        })
        .collect::<Result<_, _>>()?;
    Ok(ControlCommand::Events {
        follow: args.flag("--follow"),
        after: args
            .text_option("--after")?
            .map(|after| {
                after
                    .parse()
                    .map_err(|()| CliError::InvalidValue("--after"))
            })
            .transpose()?,
        wait: wait_option(args, "--wait")?,
        kinds,
    })
}

pub(super) const RUN_USAGE: &str = "spaceport run COMMAND\n    Run an app command by its [keybindings] name, as the command palette\n    does, for example toggle_sidebar. See `commands`.";

pub(super) fn run(args: &mut Args) -> Result<ControlCommand, CliError> {
    Ok(ControlCommand::Run {
        name: args.required_text("COMMAND")?,
    })
}

pub(super) const COMMANDS_USAGE: &str =
    "spaceport commands\n    The commands `run` accepts, with their shortcuts.";

pub(super) fn commands(_: &mut Args) -> Result<ControlCommand, CliError> {
    Ok(ControlCommand::Commands)
}

pub(super) const LOGS_USAGE: &str = "spaceport logs [--lines N] [--path]\n    The app's latest log records (default 100, at most 5000), or the log file's path.\n    Runs here; Spaceport need not run. Exits 4 when there is no log yet.";

pub(super) fn logs(args: &mut Args) -> Result<ControlCommand, CliError> {
    Ok(ControlCommand::Logs {
        path: args.flag("--path"),
        lines: args
            .number("--lines", 1..=crate::diagnostics::TAIL_RECORDS)?
            .unwrap_or(100),
    })
}

pub(super) const DIAGNOSTICS_USAGE: &str = "spaceport diagnostics bundle PATH\n    Write build information, settings diagnostics, your settings\n    overrides, and recent logs to a new file at PATH for a bug report,\n    with home paths shortened and secret-looking values redacted. Read\n    it before sharing. Never replaces an existing file. Runs here.";

pub(super) fn diagnostics(args: &mut Args) -> Result<ControlCommand, CliError> {
    match args.verb().as_ref().and_then(|verb| verb.to_str()) {
        Some("bundle") => {}
        _ => return Err(CliError::InvalidValue("bundle")),
    }
    Ok(ControlCommand::Diagnostics {
        output: args.required("PATH")?.into(),
    })
}
