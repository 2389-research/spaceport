//! Usage text for every control command.
use crate::cli::control::FAMILIES;

const COMMON: &str = "\
Every command accepts --json, printing one JSON document, and --window ID,
acting in that window (IDs come from `window list`). By default a command
acts in the window showing the caller's workspace (HERDR_WORKSPACE_ID), and
exits 5 when no window shows it; outside Herdr, in the frontmost window.
Exit status: 0 done, 1 refused, 2 invalid arguments, 3 Spaceport not running,
4 nothing yet (no event), 5 not found, 6 busy (a dialog is open),
7 timed out, 8 unsupported on this platform or by this app version.";

fn family(name: &str) -> &'static str {
    match name {
        "status" => {
            "spaceport status\n    Whether Spaceport runs, its version, protocol, windows, hosts, and the\n    current Core machine catalog's freshness and count. Also reports Ask Spaceport\n    configuration, loading, active answer, retained history, agent, and error.\n    No machine lookup runs. No question is submitted. Exits 3 when it does not run."
        }
        "skill" => "spaceport skill\n    Print the agent skill describing these commands.",
        "app" => {
            "spaceport app focus|hide|quit\n    Bring the app forward, hide it, or quit it. Quitting leaves Herdr\n    and its terminals running."
        }
        "window" => {
            "spaceport window [list]\n    The app's windows, with IDs for --window.\nspaceport window new\n    Open another window on the selected window's exact Core group or legacy\n    launch target, the caller's host, or the frontmost/default target. A\n    stale Core group is refused before the window opens.\nspaceport window focus ID\nspaceport window fullscreen [on|off|toggle]\nspaceport window size [WIDTHxHEIGHT]\n    Print the window's size, or resize it. Windows cannot be moved.\nspaceport window detach|reconnect\n    Detach from, or reconnect to, the window's selected host."
        }
        "ui" => {
            "spaceport ui sidebar [on|off|toggle]\nspaceport ui statusbar [on|off|toggle]\nspaceport ui font-size +|-|reset|POINTS\nspaceport ui panel workspaces|overview|ask|automations|usage\n    For this window and session; settings are changed with `config`."
        }
        "tab" => {
            "spaceport tab [list]\n    The app's own tabs: pages, Devices, Core terminals.\n    Terminal tabs belong to Herdr: use `herdr tab`.\nspaceport tab focus|close ID\n    Show a tab, switching to its workspace, or close it."
        }
        "devices" => {
            "spaceport devices [open]\n    Open the Devices overview tab in the caller's workspace."
        }
        "chat" => {
            "spaceport chat [status]\n    The window's sheprd-acp chat panes on this machine or an SSH host: pane, agent, status,\n    and whether the native chat view or the terminal shows. Exits 4 when\n    there are none.\nspaceport chat show|hide|toggle [--pane ID]\n    Show the native chat view over the pane, or its terminal, for this\n    window. The pane is the caller's own, else the focused one. Exits 5 when\n    it runs no chat the window can show.\nspaceport chat new AGENT [--pane ID]\n    Open a tab running a sheprd-acp chat with AGENT (claude, codex, copilot,\n    gemini, opencode, or pi) in the workspace and folder of the pane, else the\n    caller's own. On an SSH host the tab runs that host's sheprd-acp, from\n    ~/.herdr/bin or PATH. With no Herdr pane to go by, a Herdr 2 tab in the\n    active group splits its pane in use with the chat instead. The chat view\n    attaches once the program starts.\n    Exits 8 when ACP chats are off in Settings, sheprd-acp is missing here,\n    the host cannot be reached, or the agent is not installed there."
        }
        "observation" => {
            "spaceport observation sessions [--offset N] [--limit N] [--status idle|working|blocked|done|unknown]\nspaceport observation conversation TARGET [--offset N] [--limit N] [--max-bytes N]\nspaceport observation terminal TARGET [--offset N] [--limit N] [--max-bytes N]\n    Read prepared connected session state across all app windows. TARGET comes\n    from sessions and names one exact connection and daemon boot. No new I/O\n    or session startup runs. Limits clamp to 64 sessions, 200 text items, and\n    16384 excerpt bytes. Conversation offsets count text entries; terminal\n    offsets count viewport rows. Truncated items are consumed. Coverage\n    reports unavailable routes; app read time is not daemon freshness.\n    --window validates that window exists; observation scope remains app-wide."
        }
        "notes" => {
            "spaceport notes [list]\n    Every worktree note, most recently edited first.\nspaceport notes get|clear\nspaceport notes set TEXT...\n    The note on the caller's checkout (its repository and branch), or the\n    focused workspace's outside a Herdr pane. Notes are one line of at most\n    500 characters. `list` and `get` exit 4 when there is none."
        }
        "worktree" => {
            "spaceport worktree [status]\n    Whether the caller's worktree is muted.\nspaceport worktree mute|unmute\n    Silence its notifications, or let them through again. Mutes stay in\n    this app; nothing reaches the daemon."
        }
        "host" => {
            "spaceport host [list]\n    The window's hosts and their status. Herdr's `machine` command\n    manages saved hosts.\nspaceport host mute|unmute [HOST]\n    Silence a host's notifications as a whole, or let them through again,\n    by ID or label; absent, the caller's host. Works while it is offline.\nspaceport host select ID\n    Show that host in the window, as choosing it in the sidebar does.\nspaceport host tag ID [TAG...]\n    Replace the local tags that group that host in the sidebar; with no\n    tags, clear them."
        }
        "ports" => {
            "spaceport ports [list]\n    Ports the caller's workspace's processes listen on, and forwards.\n    Listening ports are known only while show_listening_ports is on.\nspaceport ports forward|unforward PORT\n    Forward a port of the workspace's SSH host to this machine, or stop.\nspaceport ports open PORT\n    Open localhost:PORT with the system URL handler, forwarding it first if remote."
        }
        "processes" => {
            "spaceport processes [--pane ID]\n    The processes under a pane (default HERDR_PANE_ID), with CPU and\n    memory. Runs here; Spaceport need not run."
        }
        "usage" => {
            "spaceport usage\n    Plan usage on the selected host when Core supplies a feed; currently unavailable."
        }
        "config" => {
            "spaceport config [path]\n    The settings files.\nspaceport config get KEY\n    The setting's value, as TOML: your override, else the default.\nspaceport config set KEY VALUE\nspaceport config unset KEY\n    Change a setting in spaceport/config.local.toml. VALUE is read as TOML\n    (14, true, [\"a\"], { show = true }), else as text. A value or key\n    Spaceport would not load is refused and the file left as it was.\n    A running Spaceport applies the change at once. KEY is dotted, such\n    as terminal.size. Unsetting a key that is not set exits 4.\nspaceport config check\n    Report settings that do not load and unknown keys, and, when Spaceport\n    runs, fonts that are not installed; exits 1 when there are any.\nspaceport config reload\n    Reread Spaceport's settings files.\nOnly reload needs Spaceport running."
        }
        "theme" => {
            "spaceport theme [list]\n    The themes there are; * marks the selected one.\nspaceport theme set NAME...\n    Choose and save a theme, or light:NAME,dark:NAME to follow the\n    system appearance. Saved even when Spaceport is not running."
        }
        "font" => {
            "spaceport font [list]\n    The installed font families.\nspaceport font set NAME...\nspaceport font reset\n    The font of every face; reset goes back to the default.\nThese need Spaceport running, which knows the installed fonts."
        }
        "language" => {
            "spaceport language [status]\n    The language set, and the locale Spaceport shows: the best match for\n    the setting among the languages there are, else en-US.\nspaceport language list\n    The languages there are, each named in itself, and the en-XA and en-XB\n    pseudo-locales.\nspaceport language set system|TAG\n    Save the language setting: system follows the OS's languages; TAG is\n    a BCP 47 tag such as fr-CA. A running Spaceport switches at once.\nOnly status asks Spaceport; without it, status reports what it would show."
        }
        "keybinds" => {
            "spaceport keybinds [list]\n    The window's shortcuts.\nspaceport keybinds check\n    Herdr's custom commands this client cannot reach; exits 1 if any."
        }
        "caffeine" => {
            "spaceport caffeine [status]\nspaceport caffeine on|off|toggle [--for 90s|30m|2h]\n    Keep the display and the machine awake. With --for (at most 24h),\n    caffeine turns off then, unless it was changed in the meantime."
        }
        "sound" => {
            "spaceport sound [list]\n    The notification sounds.\nspaceport sound play [done|request]\n    Play one now, from the file Herdr's sound settings name or the\n    built-in one."
        }
        "bell" => "spaceport bell\n    Ring the system bell, as a terminal bell does.",
        "events" => {
            "spaceport events [--after CURSOR] [--wait SECONDS] [--kind KIND]... [--follow]\n    What happened since CURSOR, the INSTANCE:SEQ the last call printed:\n    windows and tabs opening and closing, notes, and host status. A cursor\n    from an earlier app run gets everything this run kept. --follow keeps\n    printing until interrupted. Kinds:\n    window_opened, window_closed, window_focused, tab_opened, tab_closed,\n    note, host."
        }
        "wait" => {
            "spaceport wait window|online|tab-closed ID [--timeout SECONDS]\n    Wait (default 60, at most 600 seconds). Exits 4 when time runs out."
        }
        "run" => {
            "spaceport run COMMAND\n    Run an app command by its [keybindings] name, as the command palette\n    does, for example toggle_sidebar. See `commands`."
        }
        "commands" => "spaceport commands\n    The commands `run` accepts, with their shortcuts.",
        "logs" => {
            "spaceport logs [--lines N] [--path]\n    The app's latest log records (default 100, at most 5000), or the log file's path.\n    Runs here; Spaceport need not run. Exits 4 when there is no log yet."
        }
        "diagnostics" => {
            "spaceport diagnostics bundle PATH\n    Write build information, settings diagnostics, your settings\n    overrides, and recent logs to a new file at PATH for a bug report,\n    with home paths shortened and secret-looking values redacted. Read\n    it before sharing. Never replaces an existing file. Runs here."
        }
        "integrations" => {
            "spaceport integrations [list]\n    The agent integrations Herdr offers the window's host. Install them\n    from Spaceport's integrations settings.\nspaceport integrations install-skill\n    Install or update the agent skill for Claude Code and other agents.\n    Runs here; Spaceport need not run."
        }
        _ => "",
    }
}

/// Usage for one family, or the list of families.
pub(super) fn usage(name: Option<&str>) -> String {
    match name {
        Some(name) => format!("{}\n\n{COMMON}", family(name)),
        None => {
            let mut text = String::from("spaceport COMMAND ... [--json] [--window ID]\n\n");
            for (name, summary) in FAMILIES {
                text.push_str(&format!("  {name:<14}{summary}\n"));
            }
            text.push_str("\nRun `spaceport COMMAND --help` for one command's usage.\n\n");
            text.push_str(COMMON);
            text
        }
    }
}
