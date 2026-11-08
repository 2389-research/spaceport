//! Usage text for every control command.
use crate::cli::control::FAMILIES;

const COMMON: &str = "\
Every command accepts --json, printing one JSON document, and --window ID,
acting in that window (IDs come from `window list`). By default a command
acts in the window showing the caller's workspace (HERDR_WORKSPACE_ID), and
exits 5 when no window shows it; outside Herdr, in the frontmost window.
Exit status: 0 done, 1 refused, 2 invalid arguments, 3 Sheprd not running,
4 nothing yet (no notes, no event), 5 not found, 6 busy (a dialog is open),
7 timed out, 8 unsupported on this platform or by this app version.";

const BROWSER: &str = "\
sheprd browser reload
    Reload the pages this pane opened, for example after editing the file.
sheprd browser feedback [--wait SECONDS]
    Print the notes the user sent about this pane's pages and have not been
    delivered yet. With --wait, wait up to SECONDS (at most 600) for them.
    Notes are otherwise typed into this pane as a prompt once it is idle.
sheprd browser list
    List page tabs with their IDs.
sheprd browser close|back|forward|text TAB
    Close a page tab, go back or forward in it, or print its visible text.
sheprd browser navigate TAB URL
    Load another web address in a page tab.
sheprd browser zoom TAB +|-|reset|PERCENT
    Zoom a page tab in or out a step, back to 100%, or to PERCENT (50-300),
    as the font size keys do while it is focused. The tab keeps its zoom.
sheprd browser screenshot TAB PATH
    Save a PNG of the page (macOS).
sheprd browser skill
    Print the agent skill that explains these commands.
sheprd browser example
    Print an example explainer page built on the kit Sheprd serves to local
    pages at /.herdr/kit.css and /.herdr/kit.js.
Exit status: 0 done, 1 refused, 2 invalid arguments, 3 Sheprd not running,
4 no notes (feedback), 5 no such tab or workspace, 6 busy, 7 timed out,
8 unsupported on this platform.";

fn family(name: &str) -> &'static str {
    match name {
        "status" => {
            "sheprd status\n    Whether Sheprd runs, its version, protocol, windows, hosts, and the\n    current Core machine catalog's freshness and count. No machine lookup runs.\n    Exits 3 when it does not run."
        }
        "skill" => "sheprd skill\n    Print the agent skill describing these commands.",
        "app" => {
            "sheprd app focus|hide|quit\n    Bring the app forward, hide it, or quit it. Quitting leaves Herdr\n    and its terminals running."
        }
        "open" => {
            "sheprd open DIR\n    Open the folder as Open Project in the command palette does: focus its\n    workspace, or start one there, and bring the window forward. DIR must\n    be an existing folder. With no Sheprd running, starts it and waits up\n    to 20 seconds for its window to reach the local Herdr (exit 7 past that)."
        }
        "window" => {
            "sheprd window [list]\n    The app's windows, with IDs for --window.\nsheprd window new\n    Open another window on the target selected by --window, the caller's\n    host, or the frontmost/default target.\nsheprd window focus ID\nsheprd window fullscreen [on|off|toggle]\nsheprd window size [WIDTHxHEIGHT]\n    Print the window's size, or resize it. Windows cannot be moved.\nsheprd window detach|reconnect\n    Detach from, or reconnect to, the window's selected host."
        }
        "ui" => {
            "sheprd ui sidebar [on|off|toggle]\nsheprd ui statusbar [on|off|toggle]\nsheprd ui font-size +|-|reset|POINTS\nsheprd ui panel workspaces|overview|pull-requests|issues|tasks|security|automations|usage\n    For this window and session; settings are changed with `config`."
        }
        "tab" => {
            "sheprd tab [list]\n    The app's own tabs: pages, code, reviews, Devices.\n    Terminal tabs belong to Herdr: use `herdr tab`.\nsheprd tab focus|close ID\n    Show a tab, switching to its workspace, or close it."
        }
        "devices" => {
            "sheprd devices [open]\n    Open the Devices overview tab in the caller's workspace."
        }
        "orchestrator" => {
            "sheprd orchestrator [open] [--repo PATH]\n    Show the Issues section narrowed to the caller's project, or PATH's.\nsheprd orchestrator items|issues|prs|tasks [--repo PATH]\n    List what it last synced, from its local database (no network).\n    Runs here; Sheprd need not run. Exits 4 before its first sync."
        }
        "review" => {
            "sheprd review [open]\n    Open the review of the caller's workspace's uncommitted changes."
        }
        "chat" => {
            "sheprd chat [status]\n    The window's sheprd-acp chat panes on this machine or an SSH host: pane, agent, status,\n    and whether the native chat view or the terminal shows. Exits 4 when\n    there are none.\nsheprd chat show|hide|toggle [--pane ID]\n    Show the native chat view over the pane, or its terminal, for this\n    window. The pane is the caller's own, else the focused one. Exits 5 when\n    it runs no chat the window can show.\nsheprd chat new AGENT [--pane ID]\n    Open a tab running a sheprd-acp chat with AGENT (claude, codex, copilot,\n    gemini, opencode, or pi) in the workspace and folder of the pane, else the\n    caller's own. On an SSH host the tab runs that host's sheprd-acp, from\n    ~/.herdr/bin or PATH. With no Herdr pane to go by, a Herdr 2 tab in the\n    active group splits its pane in use with the chat instead. The chat view\n    attaches once the program starts.\n    Exits 8 when ACP chats are off in Settings, sheprd-acp is missing here,\n    the host cannot be reached, or the agent is not installed there.\nsheprd chat convert [--pane ID]\n    Move the terminal agent in the pane, else the caller's own, else the\n    focused one, into a sheprd-acp chat in the same pane: stops its CLI\n    with Ctrl+C and continues its session, with its messages, in the chat.\n    Claude, Codex, Copilot, and OpenCode, once idle; a draft in its input\n    is lost. Exits 6 while the agent works and 8 for an agent or pane that\n    cannot move."
        }
        "px0" => {
            "sheprd px0 [open]\n    Open or focus the caller's px0 editor tab on its local checkout.\n    Discovers a running workbench or starts installed px0 lazily.\n    px0.url configures an existing loopback HTTP workbench.\n    Success means the tab opened; server startup errors appear there.\n    Embedded pages require macOS or Windows; remote workspaces are unsupported."
        }
        "notes" => {
            "sheprd notes [list]\n    Every worktree note, most recently edited first.\nsheprd notes get|clear\nsheprd notes set TEXT...\n    The note on the caller's checkout (its repository and branch), or the\n    focused workspace's outside a Herdr pane. Notes are one line of at most\n    500 characters. `list` and `get` exit 4 when there is none."
        }
        "worktree" => {
            "sheprd worktree [status]\n    Whether the caller's worktree is muted.\nsheprd worktree mute|unmute\n    Silence its notifications, or let them through again. Mutes stay in\n    this app; nothing reaches the daemon."
        }
        "feedback" => {
            "sheprd feedback [--wait SECONDS]\n    Page and review notes sent to this pane that were not typed into it.\n    Exits 4 when there are none."
        }
        "checkpoint" => {
            "sheprd checkpoint [list]\n    The current directory's checkout's checkpoints, newest first.\nsheprd checkpoint create [LABEL...]\n    Take one now.\nsheprd checkpoint diff CHECKPOINT [CHECKPOINT]\n    What changed between two, or since one in the working tree (untracked\n    files included).\n    These run here, on the current branch; Sheprd need not run.\nsheprd checkpoint restore [CHECKPOINT]\n    Open the caller's checkout's checkpoints dialog in Sheprd with\n    CHECKPOINT chosen; the user confirms the restore there."
        }
        "scripts" => {
            "sheprd scripts [list]\n    The current directory's repository's worktree scripts\n    (.herdr/worktree.toml). Runs here; Sheprd need not run.\nsheprd scripts setup|run\n    Run one in a new tab of the caller's workspace (setup: linked worktrees\n    only). Sheprd asks before running a script it has not seen. The\n    archive script runs only when a worktree is removed in Sheprd."
        }
        "fan-out" => {
            "sheprd fan-out PROMPT... [--agent KIND[:COUNT]]...\n    Open the fan-out dialog for the caller's repository with PROMPT and\n    these agents (claude, codex, ...; at most 6 in all) chosen once the\n    host's agents are known. The user starts it there."
        }
        "host" => {
            "sheprd host [list]\n    The window's hosts and their status. Herdr's `machine` command\n    manages saved hosts.\nsheprd host mute|unmute [HOST]\n    Silence a host's notifications as a whole, or let them through again,\n    by ID or label; absent, the caller's host. Works while it is offline.\nsheprd host select ID\n    Show that host in the window, as choosing it in the sidebar does.\nsheprd host tag ID [TAG...]\n    Replace the local tags that group that host in the sidebar; with no\n    tags, clear them."
        }
        "ports" => {
            "sheprd ports [list]\n    Ports the caller's workspace's processes listen on, and forwards.\n    Listening ports are known only while show_listening_ports is on.\nsheprd ports forward|unforward PORT\n    Forward a port of the workspace's SSH host to this machine, or stop.\nsheprd ports open PORT\n    Open localhost:PORT with the system URL handler, forwarding it first if remote."
        }
        "processes" => {
            "sheprd processes [--pane ID]\n    The processes under a pane (default HERDR_PANE_ID), with CPU and\n    memory. Runs here; Sheprd need not run."
        }
        "usage" => "sheprd usage\n    AI plan usage on the window's selected host.",
        "system" => "sheprd system\n    The window's selected host's load, when Sheprd samples it.",
        "config" => {
            "sheprd config [path]\n    The settings files.\nsheprd config get KEY\n    The setting's value, as TOML: your override, else the default.\nsheprd config set KEY VALUE\nsheprd config unset KEY\n    Change a setting in spaceport/config.local.toml. VALUE is read as TOML\n    (14, true, [\"a\"], { show = true }), else as text. A value or key\n    Spaceport would not load is refused and the file left as it was.\n    A running Spaceport applies the change at once. KEY is dotted, such\n    as terminal.size. Unsetting a key that is not set exits 4.\nsheprd config check\n    Report settings that do not load and unknown keys, and, when Spaceport\n    runs, fonts that are not installed; exits 1 when there are any.\nsheprd config reload\n    Reread Spaceport's settings files.\nOnly reload needs Spaceport running."
        }
        "theme" => {
            "sheprd theme [list]\n    The themes there are; * marks the selected one.\nsheprd theme set NAME...\n    Choose and save a theme, or light:NAME,dark:NAME to follow the\n    system appearance. Saved even when Sheprd is not running."
        }
        "font" => {
            "sheprd font [list]\n    The installed font families.\nsheprd font set NAME...\nsheprd font reset\n    The font of every face; reset goes back to the default.\nThese need Sheprd running, which knows the installed fonts."
        }
        "language" => {
            "sheprd language [status]\n    The language set, and the locale Sheprd shows: the best match for\n    the setting among the languages there are, else en-US.\nsheprd language list\n    The languages there are, each named in itself, and the en-XA and en-XB\n    pseudo-locales.\nsheprd language set system|TAG\n    Save the language setting: system follows the OS's languages; TAG is\n    a BCP 47 tag such as fr-CA. A running Sheprd switches at once.\nOnly status asks Sheprd; without it, status reports what it would show."
        }
        "keybinds" => {
            "sheprd keybinds [list]\n    The window's shortcuts.\nsheprd keybinds check\n    Herdr's custom commands this client cannot reach; exits 1 if any."
        }
        "caffeine" => {
            "sheprd caffeine [status]\nsheprd caffeine on|off|toggle [--for 90s|30m|2h]\n    Keep the display and the machine awake. With --for (at most 24h),\n    caffeine turns off then, unless it was changed in the meantime."
        }
        "sound" => {
            "sheprd sound [list]\n    The notification sounds.\nsheprd sound play [done|request]\n    Play one now, from the file Herdr's sound settings name or the\n    built-in one."
        }
        "bell" => "sheprd bell\n    Ring the system bell, as a terminal bell does.",
        "events" => {
            "sheprd events [--after CURSOR] [--wait SECONDS] [--kind KIND]... [--follow]\n    What happened since CURSOR, the INSTANCE:SEQ the last call printed:\n    windows and tabs opening and closing, notes, and host status. A cursor\n    from an earlier app run gets everything this run kept. --follow keeps\n    printing until interrupted. Kinds:\n    window_opened, window_closed, window_focused, tab_opened, tab_closed,\n    feedback, note, host."
        }
        "wait" => {
            "sheprd wait feedback|window|online|tab-closed ID [--timeout SECONDS]\n    Wait (default 60, at most 600 seconds). Exits 4 when time runs out."
        }
        "run" => {
            "sheprd run COMMAND\n    Run an app command by its [keybindings] name, as the command palette\n    does, for example toggle_sidebar. See `commands`."
        }
        "commands" => "sheprd commands\n    The commands `run` accepts, with their shortcuts.",
        "logs" => {
            "sheprd logs [--lines N] [--path]\n    The app's latest log records (default 100, at most 5000), or the log file's path.\n    Runs here; Sheprd need not run. Exits 4 when there is no log yet."
        }
        "update" => {
            "sheprd update [status]\nsheprd update check\n    Ask for the latest release. Never downloads or installs. Local\n    builds report updates as off."
        }
        "diagnostics" => {
            "sheprd diagnostics bundle PATH\n    Write build information, settings diagnostics, your settings\n    overrides, and recent logs to a new file at PATH for a bug report,\n    with home paths shortened and secret-looking values redacted. Read\n    it before sharing. Never replaces an existing file. Runs here."
        }
        "integrations" => {
            "sheprd integrations [list]\n    The agent integrations Herdr offers the window's host. Install them\n    from Sheprd's integrations settings.\nsheprd integrations install-skill\n    Install or update the agent skill for Claude Code and other agents.\n    Runs here; Sheprd need not run."
        }
        _ => "",
    }
}

/// Usage for one family, or the list of families.
pub(super) fn usage(name: Option<&str>) -> String {
    match name {
        Some("browser") => BROWSER.to_owned(),
        Some(name) => format!("{}\n\n{COMMON}", family(name)),
        None => {
            let mut text = String::from("sheprd COMMAND ... [--json] [--window ID]\n\n");
            for (name, summary) in FAMILIES {
                text.push_str(&format!("  {name:<14}{summary}\n"));
            }
            text.push_str("\nRun `sheprd COMMAND --help` for one command's usage.\n\n");
            text.push_str(COMMON);
            text
        }
    }
}
