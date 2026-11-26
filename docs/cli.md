# Command line

This development branch builds one `spaceport` GUI and CLI executable. The
[Spaceport task graph](spaceport/plan/tasks.json) tracks routes that still
need work.

`spaceport` is also a command line for the running app. Herdr's own
`herdr` CLI drives the daemon: workspaces, terminal tabs, panes, agents,
worktrees, and notifications. `spaceport COMMAND` covers what only this
client knows or draws: its windows, its own tabs (Devices and Core
terminals), the activity bar's sections, worktree notes, forwarded
ports, and its settings.

Commands reach the app over a socket of its own,
`$XDG_STATE_HOME/spaceport/control.sock` (default `~/.local/state/spaceport/`), which
only your user can open; on Windows, over a named pipe (see
[Windows](#windows)). An agent on an SSH host has no path to this socket.
A few commands work from files alone and
need no running app; they say so below.

On Windows, `spaceport.exe` is a console program, so a shell waits for it and
sees its exit code; the installers put its directory on `PATH`. The Start menu
opens `spaceport-launcher.exe` instead, which starts it without a console
window.

```sh
spaceport --help              # every command
spaceport COMMAND --help      # one command
spaceport --build-info        # the executable's build identity
```

`spaceport --build-info` prints the executable's build identity without
starting the GUI, and `spaceport --socket CLIENT_SOCKET` starts it attached
only to that existing daemon, discovering and starting no other.

## Options and output

Every command accepts, after its command word:

- `--json`: print one JSON document instead of text, the reply as the control
  protocol defines it (`crates/spaceport-control-protocol/src/protocol` holds each
  family's types; an answer the app sends is bounded to the same 256 KiB as
  every response). Errors become
  `{"error": {"code", "message"}}` on standard output. `events --follow`
  is the one stream: it writes one JSON object per line, each with its
  `cursor`, so a follower can resume after any event. `--help` and `skill`
  print text whatever the options say.
- `--window ID`: act in that window (IDs come from `window list`). By default
  a command acts in the window showing the caller's workspace
  (`HERDR_WORKSPACE_ID`, which Herdr sets in every pane). When no window shows
  it, as for a session the app does not show, the command exits 5 rather than
  act on another session's window. Outside Herdr, it acts in the frontmost
  window. A command this process answers from files alone, marked "no app
  needed" below, accepts the ID and ignores it, so it never refuses one. A
  command about the whole app rather than one window — `status`, `window
  list`, `tab list`, `events`, `notes list`, `caffeine`, `font`, `language`,
  `app hide`, and `app quit` — likewise changes nothing window by window, but
  the app still refuses an ID no window has.

Both options follow the command word: `spaceport --json status` is an
unknown option, not `status` asked for JSON.

| Exit | Meaning |
| --- | --- |
| 0 | Done |
| 1 | Refused or failed |
| 2 | Invalid arguments, or an address or file that cannot be shown |
| 3 | Spaceport is not running, or the control connection closed without an answer |
| 4 | Nothing yet: no notes, no events, no chat panes, no ports, no log, a key not set, or the wait not met |
| 5 | The window, workspace, tab, host, file, or `run` command name is not there |
| 6 | A dialog is open in the window, or the app is busy |
| 7 | The app did not answer in time |
| 8 | Not available on this platform or in this version of the app |

With `--json`, an error the app returns keeps the app's exact code, such as
`workspace_not_found`, `no_window`, `not_found`, `invalid_request`, or
`failed`; the exit status stays the coarser category above. Errors the
command line finds itself use the category's name: `usage`, `not_running`,
`timeout`, `unsupported`, or `refused`.

An answer this command line cannot read, or a `status` from another control
protocol version, exits 8. A read or write timeout on the control connection
exits 7. A lost answer does not prove a change failed: check the app's state
before you retry a change. Commands never retry on their own.

## Commands

Herdr Core is disabled until explicitly enabled in Settings > General > Experimental.
The equivalent saved setting is `spaceport config set herdr_core_enabled true`.
Restart Spaceport to apply either an enable or disable change.
`config reload` keeps the current app session's Core mode unchanged.
While disabled, Core-only commands return exit 8 (`Unsupported`) with an explanation.
Saved Core tabs stay stored but do not appear in `tab list`.
Changing this preference never stops Core or its terminals.

### App and windows

| Command | What it does |
| --- | --- |
| `status` | Whether the app runs, its version and control protocol, windows and hosts, plus the observed Core machine catalog's state, epoch and count |
| `app focus\|hide\|quit` | Bring the app forward, hide it, or quit it (Herdr keeps running) |
| `window [list]` | Windows with their IDs, size, selected legacy workspace or exact Core presentation group, and hosts; stale Core group data is marked |
| `window new` | Another window on the exact Core group or legacy launch target of the window selected by `--window`, the caller's host, or the frontmost/default target |
| `window focus ID` | Bring a window forward |
| `window fullscreen [on\|off\|toggle]` | `show` and `hide` work as `on` and `off` |
| `window size [WIDTHxHEIGHT]` | Print or set the window's size, WIDTH 200-16384 and HEIGHT 150-16384; windows cannot be moved |
| `window detach\|reconnect` | Detach from, or reconnect to, the selected host |
| `ui sidebar\|statusbar [on\|off\|toggle]` | For this window and session; `status-bar` is the same verb, and `show`/`hide` are `on`/`off` |
| `ui font-size +\|-\|reset\|POINTS` | The terminal font size for this session; POINTS is 8-48, and `larger`/`smaller` are `+`/`-` |
| `ui panel workspaces\|overview` | Show an activity bar section: Workspaces with the sidebar expanded, or Overview of what needs you on every host. Exits 8 when the activity bar is off (`[sidebar] activity_bar = false`) |
| `ui agent-idle [--pane ID]` | Read this client’s idle estimate for a pane in the targeted workspace. Defaults to the caller’s pane, then the focused pane. JSON includes seconds and a lower-bound flag. Exits 4 when the client has no usable age. |
| `run check-for-updates` | Open Sparkle’s native update check in an installed signed macOS build, preview or production; development, worktree and other-platform builds refuse it |
| `run COMMAND` | Any app command by its `[keybindings]` name, as the palette runs it; an unknown name exits 5 |
| `commands` | The names `run` accepts (no app needed) |

When a Core presentation group is selected, `window list` reports its exact
hub, document, group and optional machine identity. It marks the group stale
when it is unavailable in a fresh observation of the matching hub. The host
rows still describe the legacy endpoint catalog. `window new` joins the
selected Core hub and group when its current observation still confirms that
exact target. It refuses a stale selection before asking for a new window.
Legacy windows keep their existing launch target.

### The app's own tabs

| Command | What it does |
| --- | --- |
| `tab [list]` | Current Devices and Core terminal tabs with IDs; retired page and specialized tabs are omitted |
| `tab focus\|close ID` | |
| `devices [open]` | The Devices overview |

### Worktrees and agents

| Command | What it does |
| --- | --- |
| `notes [list]` | Every worktree note |
| `notes get\|clear`, `notes set TEXT...` | The note on your checkout (its repository and branch): one line of at most 500 characters, refused over 16 KiB. `list` and `get` exit 4 when there is none |
| `worktree [status]`, `worktree mute\|unmute` | Whether your worktree's notifications are muted; silence them or let them through again |
| `chat [status]`, `chat list` | The `sheprd-acp` chat panes of the window's host, this machine or an SSH host: agent, status, model, context window use, and whether the chat view or the terminal shows |
| `chat show\|hide\|toggle [--pane ID]` | Show the native chat view over your pane (or ID, else the focused one), or its terminal |
| `chat new AGENT [--pane ID]` | Open a tab running a `sheprd-acp` chat with a built-in agent (`claude`, `codex`, `copilot`, `gemini`, `opencode`, `pi`) in the workspace and folder of your pane (or ID). On an SSH host the tab runs that host's `sheprd-acp` (see [Chats on SSH hosts](#chats-on-ssh-hosts)). With no Herdr pane to go by, as when called from a Herdr 2 pane, a Herdr 2 tab in the active group splits its pane in use with the chat instead (see [Chats in Herdr 2 panes](rich-chat.md#chats-in-herdr-2-panes)); its pane id in `chat` is `core:` and the terminal's. Waits up to 30 seconds for Herdr or Herdr 2 to create the tab and pane, then prints the pane id and window (see [Chat creation](#chat-creation)). Exits 8 when `sheprd-acp` is missing here or the agent is not installed on it (`sheprd-acp agents` lists those that are), 1 when the host cannot be reached, and 7 or 1 when the outcome is unknown |

### Hosts

A focus asked for on a connected host the window is not showing, such as
`herdr --machine Homelab agent focus NAME`, switches the window to that host
when it moves the host to another tab. The daemon reports nothing when the
target is already its focused tab, or when the focus only moves between panes
of one tab, so a script that must end up on the host runs `spaceport host select`
as well. A focus waits until you stop typing or composing, so keys meant for
one host do not reach another, and choosing a host yourself in the meantime
cancels it. It leaves the host restored at the next launch as it was.
It does not take the window while a dialog is open, or from another window
already showing the host. A tab or workspace opening on a background host does
not take it either.

| Command | What it does |
| --- | --- |
| `host [list]`, `host select ID` | The window's hosts, with their tags; show another one |
| `host mute\|unmute [HOST]` | Silence a host's notifications as a whole, or let them through again, by ID or label; absent, your host. Works while it is offline |
| `host tag ID [TAG...]` | Replace the local tags that group a host in the sidebar; with no tags, clear them. At most 8 tags of 24 characters each, as the sidebar keeps them |
| `ports [list]` | Ports your workspace listens on, and forwards |
| `ports forward\|unforward PORT` | Forward a port of the SSH host to this machine |
| `ports open PORT` | Open `localhost:PORT` with the system handler; forwards a remote port before opening |
| `processes [--pane ID]` | The processes under a pane, with CPU and memory (no app needed) |

### Settings and services

| Command | What it does |
| --- | --- |
| `config [path]` | The Spaceport settings files, including the private `view.toml` path (no app needed) |
| `config get\|set\|unset KEY [VALUE]` | Read a key's effective value (your override, else the default), or change `spaceport/config.local.toml`, keeping its comments. A value that would not load, or an unknown key, is refused and nothing is written; most changes apply at once; `herdr_core_enabled` requires restart (no app needed) |
| `config check` | Report settings that do not load and unknown keys, and, when the app runs, configured fonts that are not installed; exits 1 if there are any. Without a running app it says the fonts were not checked |
| `config reload` | Reread Spaceport's settings files |
| `theme [list]`, `theme set NAME...` | The app theme, or `light:NAME,dark:NAME` to follow the system appearance. A device's terminal theme is `config set devices.ID.theme NAME`, with the ID from `host list` minus its `ssh:` prefix. `list` needs no app; `set` saves the setting whether or not the app runs |
| `font [list]`, `font set NAME...`, `font reset` | The terminal font of every face, or back to the default. Needs the running app, which knows the installed fonts |
| `language [status]`, `language list`, `language set system\|TAG` | The language set and the locale Spaceport shows, the languages there are, each with its name in itself as Settings > General lists it, or save the `language` setting (a BCP 47 tag; `en-XA` and `en-XB` are pseudo-locales). Only `status` asks the app, and without it reports what it would show |
| `keybinds [list]`, `keybinds check` | This window's shortcuts, and Herdr's custom commands other than a plugin's, each with whether its keys reach this window; `check` says which do not and exits 1 if any |
| `caffeine [status]`, `caffeine on\|off\|toggle [--for 30m]` | Keep the machine awake. `status` reports `off`, `pending`, `on`, or `until_agents_finish` (the status bar cup's agents mode, which no command sets); `--for` takes `90s`, `30m`, or `2h`, from 1 second to 24 hours, and turns it off then |
| `sound [list]`, `sound play [done\|request]`, `bell` | Sounds play at `sound.volume` percent (`config set sound.volume 30`) |
| `integrations [list]` | What Herdr offers the selected host |
| `integrations install-skill` | Install the agent skill (no app needed) |
| `skill` | Print the agent skill `integrations install-skill` installs (no app needed) |
| `logs [--lines N] [--path]` | The app's log, or the path of its file: 100 records by default, at most 5000. Exits 4 when there is no log yet (no app needed) |
| `diagnostics bundle PATH` | A redacted report for a bug, with the newest crash reports (no app needed) |


Focus-on-hover between split terminal panes is on by default. Disable it with
`spaceport config set focus_follows_mouse false`.
The change applies live; `config get focus_follows_mouse` reads it back.

Set the idle threshold for the Priority sidebar with `spaceport config set quiet_after 48h`.
Choose `24h` (the default), `48h`, or `7d`. Quiet starts folded.

An unnamed tab shows its number. To show its agent's terminal title instead,
run `spaceport config set tab_labels.agent_title true`.

### Default shortcuts

`keybinds list` prints each command's shortcuts as this build binds them, and
the catalog in `crates/spaceport-commands` is their one declaration. macOS takes
Cmd; Linux and Windows take Ctrl, Ctrl+Shift for the primary actions and
Ctrl+Alt+Shift for their partners, so plain Ctrl still reaches the terminal
program. A few platform facts decide the rest:

- A shifted digit or punctuation key arrives as the symbol it types, so Linux
  and Windows bind both spellings of each: `ctrl-shift-1` and `ctrl-!`,
  `ctrl-shift-[` and `ctrl-{`.
- Windows reports AltGr as Ctrl+Alt, so no Windows default pairs Ctrl+Alt with
  a letter alone. New window is Ctrl+Shift+N there, zen mode Ctrl+Shift+Z, and
  the status bar Ctrl+Alt+Shift+B. A window closes with Alt+F4 itself.
- Bare F6, F11, Ctrl+PageUp, and Ctrl+PageDown belong to the programs a pane
  runs, so region focus, full screen, and tab cycling carry the app's Ctrl
  prefix: Ctrl+F6 and Ctrl+Shift+F6, Ctrl+Shift+F11, Ctrl+Shift+PageDown, and
  Ctrl+Shift+PageUp.

### Events

`events` prints what happened in the app: windows opening, closing, and
gaining focus; the app's tabs opening and closing; a worktree note changing;
and a host's connection status changing. Each event has a sequence
number; pass back the `INSTANCE:SEQ` cursor a call prints with `--after` to
get only newer events. A cursor from an earlier run of the app gets every
event the new run has kept, so a restart never hides events. `--wait SECONDS`
holds the call until an event arrives or the wait runs out (at most 600), so
a poll can ask for the next event rather than ask again in a loop; an empty
answer then exits 4, as it does without `--wait`.

| Command | What it does |
| --- | --- |
| `wait window\|online\|tab-closed ID [--timeout SECONDS]` | Block until the app has a window, the selected host comes online, or app tab `ID` closes: 60 seconds unless `--timeout` says otherwise, and at most 600. Exits 4 when the time runs out. `wait online` is about the window's host, so it exits 5 at once when no window shows the caller's workspace |

```sh
spaceport events --follow --kind tab_closed
spaceport wait tab-closed 12 --timeout 600
spaceport wait window            # also waits for the app to start
```

## What stays in the app

Commands never bypass a confirmation. Spaceport no longer offers repository
worktree scripts in the workspace menu or control CLI.

The development build still has an inherited Herdr 1 startup path; removing it
is an open Spaceport task. The Core path may start only a separately installed,
trusted Core when needed. Spaceport packages do not bundle or upgrade Core.
The removed `update` command is rejected before app startup.

Adding an SSH or local network device has no command: the Add Device dialog
runs it.

Herdr 1 scrollback search is not repeated here: `herdr pane read` covers it. Moving a
window and capturing a whole window have no command because the app cannot
do them.

The other way round, a few commands do what no window, menu, or palette entry
does, and exist for a shell: `window size WIDTHxHEIGHT` gives a window an
exact size no drag produces, `bell` rings the bell on demand, `config path`
and `config get` read the settings files rather than a settings page, and
`status`, `events`, and `wait` report or wait on the app itself.

## Links

`spaceport URL`, with a `spaceport://` link as the only argument, hands the
link to the running app, or starts the app and follows it once it is
connected. The development build does not register an OS-wide handler; its
Windows jump-list entries launch the executable with the link directly.
Protocol-activated notification clicks require a handler command pointing
exactly at the running executable and have not been verified on Windows.

| Link | What the app does |
| --- | --- |
| `spaceport://device?id=ID` | Show the device `host list` names `ID` |
| `spaceport://notification?tag=TAG` | Open the pane of a notification this app posted |

A link is untrusted input, checked by the command and again by the app: only
those shapes, each parameter once, bounded lengths, and no control characters.
A link only ever shows something; it never runs a command. A
refused link exits 2, an unknown device or notification 5, as for commands.
The request is `{"method": "link", "caller", "op": {"url"}}`.

## The protocol

One JSON request line, one JSON response line, then the socket closes. Every
request but `status` is `{"method", "caller", "op"}`, where `caller` carries
the pane, workspace, daemon socket, and window, and `op` is tagged with
`"op"`. `status` carries those caller fields beside `"method": "status"`
instead, with no `op`. It reports the protocol version, currently 39.
Requests and responses are bounded to 256 KiB. A request that waits says for
how long (at most 600 seconds), and the app answers at most a few requests at
a time.

### Chat creation

`chat new` waits up to 30 seconds for the runtime to create the chat's tab.
For Herdr 2, the wait also covers placing the new pane in its tab. Success
replies `created` with `agent`, `window_id`, `tab`, and `pane_id`. The agent
may still be starting, and the chat view attaches once it reports.

The `tab` field names its runtime, because each runtime owns its own tab IDs.
Herdr uses `{"runtime":"herdr","tab_id":"w1:t9"}`. Herdr 2 uses
`{"runtime":"core","tab_id":9,"terminal_id":"t9"}`, with a tab ID that
Spaceport owns. Use each ID only with its own runtime's commands. Pass the
`pane_id` to `chat show --pane ID --window WINDOW_ID`.

If the wait expires, the answer is lost, or the window closes, the reply is
an error saying the outcome is unknown. The chat may exist anyway. Inspect
the window with `spaceport chat` before you retry. Spaceport never sends the
request again on its own. The wait takes one of the bounded waiting slots.

### Windows

On Windows the same lines travel over a named pipe,
`\\.\pipe\spaceport-control-SID`, where `SID` is your Windows user's
security identifier. When `XDG_STATE_HOME` is set, a hash of the state
directory is appended, so a test or side-by-side install gets a pipe of its
own; without it, every shell of yours reaches the same app whatever `HOME` it
has. Pipe names are shared by all users of the machine, so:

- The app creates the pipe owned by you, with a DACL that grants only your
  SID and inherits nothing, so no other user can connect or add an instance.
- It creates it as the first instance and rejects remote clients
  (`PIPE_REJECT_REMOTE_CLIENTS`), so it never joins a pipe someone else made
  and nothing off this machine can reach it. If another user made the name
  first, the app does not listen, and logs why.
- `spaceport COMMAND` opens the pipe at identification level, so the server
  can learn who called but cannot act as you, and checks that the pipe's
  owner is you before it sends anything. A pipe someone else owns is refused
  with exit 1.

A pipe has no receive or send timeout. Reads poll with `PeekNamedPipe`, as
the daemon client does; writes have no deadline, so the app's out buffer
holds a whole answer and the connection bound limits what a stalled caller
can hold.

## Chats on SSH hosts

Native chat views are on by default: turn off **ACP chats** in Settings >
General (`acp_chats = false` under `[features]`) to go without them. While it
is off, chat panes show their terminal and `chat new` exits 8.

Set the reading width with `spaceport config set chat.max_width 1200`, or use
`0` for the full available width. Bounded widths range from 320 to 4000 logical
pixels; the default is 800. Settings > General offers the same width slider and
**Full-width chat** switch.

The inherited SSH chat route remains in this development build, but it does
not prove Spaceport's required Core-owned remote native chat. That route
runs `sheprd-acp connect ID` on the host over the legacy SSH connection;
[G-CHAT](spaceport/plan/gaps.json) records the missing Core adapter contract.
Starting a chat there (`chat new`, or the workspace menu) runs the host's own
`sheprd-acp`, looked for in `~/.herdr/bin`, `~/.cargo/bin`, `~/.local/bin`, and
then the remote `PATH`. Install it on the host with:

```sh
cargo install --locked --git https://github.com/2389-research/spaceport spaceport-acp --bin sheprd-acp --root ~/.herdr
```

The workspace menu offers only the agents whose own ACP command is installed
on the host, as `sheprd-acp agents` lists them; an agent `npx` would have to
download first is left out.

Files dropped on a remote chat are copied to the host first, into private
temporary files under its `${TMPDIR:-/tmp}`, and the copies are attached.

### Existing agent rendering

Set `chat.render_existing = true`, with `features.acp_chats` left on, to
render existing Linux Codex/Claude conversations automatically. The original
terminal input remains interactive in the same pane. `chat status --json`
reports `input_mode` (`chat`, `terminal`, or `read_only`) and `session_id`;
`blocked` status means the running agent needs input. `chat show/hide/toggle`
still switches the pane presentation, without starting or stopping its agent.
