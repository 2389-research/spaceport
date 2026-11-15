//! `spaceport <command> ...`: commands answered by the running app, or by
//! this process from the app's own files, without starting GPUI. Herdr's own
//! CLI already drives workspaces, tabs, panes, agents, and notifications;
//! these cover what only this client knows or draws.
mod app;
mod chat;
mod hosts;
mod plain;
mod settings;
mod tabs;
mod work;

use super::CliError;
pub(crate) use super::args::Args;
pub use crate::control::{Cursor, EventKind, FontSize, Size, Sound, Switch};
pub use settings::{
    CaffeineCommand, ConfigCommand, IntegrationsCommand, KeybindsCommand, LanguageCommand,
    SoundCommand, ThemeCommand,
};
pub use sheprd_control_protocol::protocol::{
    app::{AppOp, UiOp, WindowOp},
    chat::ChatOp,
    hosts::HostOp,
    tabs::TabOp,
    work::WorkOp,
};
use std::ffi::OsString;
pub use work::WaitFor;

/// What to do, and how the answer is printed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Invocation {
    pub command: ControlCommand,
    /// Print the answer as one JSON document instead of text.
    pub json: bool,
    /// The window to act in, by the ID `window list` prints. Absent, the
    /// window showing the caller's workspace (none: refused), or outside
    /// Herdr the frontmost one.
    pub window: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ControlCommand {
    /// Usage for every command, or for one family.
    Help(Option<&'static str>),
    Status,
    Skill,
    App(AppOp),
    Window(WindowOp),
    Ui(UiOp),
    Tab(TabOp),
    Devices,
    Chat(ChatOp),
    /// `notes` and `worktree` send the same family's operations, as `host`
    /// and `ports` do; each prints its answer its own way.
    Notes(WorkOp),
    Worktree(WorkOp),
    Host(HostOp),
    Ports(HostOp),
    Processes {
        pane: Option<String>,
    },
    Usage,
    Config(ConfigCommand),
    Theme(ThemeCommand),
    Font(ThemeCommand),
    Language(LanguageCommand),
    Keybinds(KeybindsCommand),
    Caffeine(CaffeineCommand),
    Sound(SoundCommand),
    Bell,
    Events {
        after: Option<Cursor>,
        follow: bool,
        wait: u64,
        kinds: Vec<EventKind>,
    },
    Wait {
        what: WaitFor,
        timeout: u64,
    },
    /// Run a command by its `[keybindings]` name, as the palette does.
    Run {
        name: String,
    },
    Commands,
    Logs {
        path: bool,
        lines: usize,
    },
    Diagnostics {
        output: std::path::PathBuf,
    },
    Integrations(IntegrationsCommand),
}

/// One word that starts a control command.
pub(crate) struct Family {
    pub(crate) name: &'static str,
    /// One line for the top-level help.
    pub(crate) summary: &'static str,
    /// What `spaceport NAME --help` prints before the shared text.
    pub(crate) usage: &'static str,
    /// Parses what follows the family word and the shared options.
    parse: fn(&mut Args) -> Result<ControlCommand, CliError>,
}

/// Every control command family, in the order the top-level help lists
/// them. `theme` and `font` share a parser, which reads which one it is
/// from the arguments.
pub(crate) const FAMILIES: &[Family] = &[
    Family {
        name: "status",
        summary: "Whether Spaceport runs, its version, windows, and hosts",
        usage: plain::STATUS_USAGE,
        parse: plain::status,
    },
    Family {
        name: "skill",
        summary: "Print the agent skill describing these commands",
        usage: plain::SKILL_USAGE,
        parse: plain::skill,
    },
    Family {
        name: "app",
        summary: "Bring the app forward, hide it, or quit it",
        usage: app::APP_USAGE,
        parse: |args| app::app(args).map(ControlCommand::App),
    },
    Family {
        name: "window",
        summary: "List, open, focus, resize, detach, or reconnect windows",
        usage: app::WINDOW_USAGE,
        parse: |args| app::window(args).map(ControlCommand::Window),
    },
    Family {
        name: "ui",
        summary: "Sidebar, status bar, and terminal font size",
        usage: app::UI_USAGE,
        parse: |args| app::ui(args).map(ControlCommand::Ui),
    },
    Family {
        name: "tab",
        summary: "List, focus, or close the app's own tabs",
        usage: tabs::TAB_USAGE,
        parse: |args| tabs::tab(args).map(ControlCommand::Tab),
    },
    Family {
        name: "devices",
        summary: "Open the Devices overview",
        usage: tabs::DEVICES_USAGE,
        parse: tabs::devices,
    },
    Family {
        name: "chat",
        summary: "Native views of sheprd-acp chats: start one, list them, show or hide one",
        usage: chat::CHAT_USAGE,
        parse: |args| chat::chat(args).map(ControlCommand::Chat),
    },
    Family {
        name: "notes",
        summary: "Notes kept on worktrees",
        usage: work::NOTES_USAGE,
        parse: |args| work::notes(args).map(ControlCommand::Notes),
    },
    Family {
        name: "worktree",
        summary: "Mute or unmute the caller's worktree",
        usage: work::WORKTREE_USAGE,
        parse: |args| work::worktree(args).map(ControlCommand::Worktree),
    },
    Family {
        name: "host",
        summary: "List the window's hosts, show one, or mute one",
        usage: hosts::HOST_USAGE,
        parse: |args| hosts::host(args).map(ControlCommand::Host),
    },
    Family {
        name: "ports",
        summary: "Listening ports, and forwarding them to this machine",
        usage: hosts::PORTS_USAGE,
        parse: |args| hosts::ports(args).map(ControlCommand::Ports),
    },
    Family {
        name: "processes",
        summary: "The processes running under a pane",
        usage: plain::PROCESSES_USAGE,
        parse: plain::processes,
    },
    Family {
        name: "usage",
        summary: "AI plan usage on the selected host",
        usage: plain::USAGE_USAGE,
        parse: plain::usage,
    },
    Family {
        name: "config",
        summary: "Spaceport settings: path, get, set, reload, check",
        usage: settings::CONFIG_USAGE,
        parse: |args| settings::config(args).map(ControlCommand::Config),
    },
    Family {
        name: "theme",
        summary: "List or choose a color theme",
        usage: settings::THEME_USAGE,
        parse: |args| settings::theme(args).map(ControlCommand::Theme),
    },
    Family {
        name: "font",
        summary: "List or choose the terminal font",
        usage: settings::FONT_USAGE,
        parse: |args| settings::theme(args).map(ControlCommand::Font),
    },
    Family {
        name: "language",
        summary: "The language Spaceport shows, or choose one",
        usage: settings::LANGUAGE_USAGE,
        parse: |args| settings::language(args).map(ControlCommand::Language),
    },
    Family {
        name: "keybinds",
        summary: "List shortcuts and check them for conflicts",
        usage: settings::KEYBINDS_USAGE,
        parse: |args| settings::keybinds(args).map(ControlCommand::Keybinds),
    },
    Family {
        name: "caffeine",
        summary: "Keep the machine awake",
        usage: settings::CAFFEINE_USAGE,
        parse: |args| settings::caffeine(args).map(ControlCommand::Caffeine),
    },
    Family {
        name: "sound",
        summary: "List or play notification sounds",
        usage: settings::SOUND_USAGE,
        parse: |args| settings::sound(args).map(ControlCommand::Sound),
    },
    Family {
        name: "bell",
        summary: "Ring the terminal bell",
        usage: plain::BELL_USAGE,
        parse: plain::bell,
    },
    Family {
        name: "events",
        summary: "What happened in the app, as a stream",
        usage: plain::EVENTS_USAGE,
        parse: plain::events,
    },
    Family {
        name: "wait",
        summary: "Wait for a window or a tab to close",
        usage: work::WAIT_USAGE,
        parse: work::wait,
    },
    Family {
        name: "run",
        summary: "Run any app command by name, as the command palette does",
        usage: plain::RUN_USAGE,
        parse: plain::run,
    },
    Family {
        name: "commands",
        summary: "List the commands `run` accepts",
        usage: plain::COMMANDS_USAGE,
        parse: plain::commands,
    },
    Family {
        name: "logs",
        summary: "Print or locate the app's log",
        usage: plain::LOGS_USAGE,
        parse: plain::logs,
    },
    Family {
        name: "diagnostics",
        summary: "Write a redacted bundle for a bug report",
        usage: plain::DIAGNOSTICS_USAGE,
        parse: plain::diagnostics,
    },
    Family {
        name: "integrations",
        summary: "Agent integrations and the agent skill",
        usage: settings::INTEGRATIONS_USAGE,
        parse: |args| settings::integrations(args).map(ControlCommand::Integrations),
    },
];

/// Whether `args` are a control command asking for JSON, whose errors,
/// usage errors included, are then printed as JSON.
pub fn wants_json(args: &[OsString]) -> bool {
    args.first().is_some_and(is_family)
        && args
            .iter()
            .take_while(|arg| *arg != "--")
            .any(|arg| arg == "--json")
}

pub fn is_family(word: &OsString) -> bool {
    FAMILIES.iter().any(|family| word == family.name)
}

/// The longest wait any command accepts, in seconds.
pub const MAX_WAIT: u64 = 600;

fn switch(word: Option<OsString>) -> Result<Switch, CliError> {
    match word.as_ref().and_then(|word| word.to_str()) {
        None => Ok(Switch::Toggle),
        Some("on" | "show") => Ok(Switch::On),
        Some("off" | "hide") => Ok(Switch::Off),
        Some("toggle") => Ok(Switch::Toggle),
        Some(_) => Err(CliError::InvalidValue("on|off|toggle")),
    }
}

fn wait_option(args: &mut Args, flag: &'static str) -> Result<u64, CliError> {
    Ok(args.number(flag, 0..=MAX_WAIT)?.unwrap_or(0))
}

/// A verb that names no subcommand of `family`.
fn unknown(family: &'static str, verb: OsString) -> CliError {
    CliError::UnknownCommand {
        family,
        command: verb,
    }
}

/// Parses everything after the family word. `--json`, `--window ID`, and
/// `--help` may appear anywhere.
pub fn parse(
    family: OsString,
    rest: impl Iterator<Item = OsString>,
) -> Result<Invocation, CliError> {
    let family = FAMILIES
        .iter()
        .find(|entry| family == entry.name)
        .ok_or(CliError::UnknownOption(family))?;
    let mut args = Args::new(rest, family.name);
    if args.wants_help() {
        return Ok(Invocation {
            command: ControlCommand::Help(Some(family.name)),
            json: false,
            window: None,
        });
    }
    let json = args.flag("--json");
    let window = args.number("--window", 0..=u64::MAX)?;
    args.set_window(window);
    let command = (family.parse)(&mut args)?;
    args.finish()?;
    Ok(Invocation {
        command,
        json,
        window: args.window(),
    })
}

/// `--for 90m`: seconds, minutes, or hours.
fn duration(text: &str) -> Option<u64> {
    let (number, unit) = text.split_at(text.find(|c: char| !c.is_ascii_digit())?);
    let number: u64 = number.parse().ok()?;
    let seconds = match unit {
        "s" => number,
        "m" => number.checked_mul(60)?,
        "h" => number.checked_mul(3600)?,
        _ => return None,
    };
    (seconds > 0).then_some(seconds)
}
