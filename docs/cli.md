# Command line

This development branch builds one `spaceport` GUI and CLI executable. The
[Spaceport task graph](spaceport/plan/tasks.json) tracks routes that still
need work.

`spaceport` is also a command line for the running app. Herdr's own
`herdr` CLI drives the daemon: workspaces, terminal tabs, panes, agents,
worktrees, and notifications. `spaceport COMMAND` covers what only this
client knows or draws: its windows, its own tabs (pages, Devices and Core
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
```

## Options and output

Every command accepts:

- `--json`: print one JSON document instead of text. Errors become
  `{"error": {"code", "message"}}` on standard output.
- `--window ID`: act in that window (IDs come from `window list`). By default
  a command acts in the window showing the caller's workspace
  (`HERDR_WORKSPACE_ID`, which Herdr sets in every pane). When no window shows
  it, as for a session the app does not show, the command exits 5 rather than
  act on another session's window. Outside Herdr, it acts in the frontmost
  window.

| Exit | Meaning |
| --- | --- |
| 0 | Done |
| 1 | Refused or failed |
| 2 | Invalid arguments, or an address or file that cannot be shown |
| 3 | Spaceport is not running |
| 4 | Nothing yet: no notes, no event, no pull request, condition not met |
| 5 | The window, workspace, tab, host, or file is not there |
| 6 | A dialog is open in the window, or the app is busy |
| 7 | The app did not answer in time |
| 8 | Not available on this platform or in this version of the app |

## Commands

### App and windows

| Command | What it does |
| --- | --- |
| `status` | Whether the app runs, its version and control protocol, windows and hosts, plus the observed Core machine catalog's state, epoch and count |
| `app focus\|hide\|quit` | Bring the app forward, hide it, or quit it (Herdr keeps running) |
| `window [list]` | Windows with their IDs, size, workspace, and hosts |
| `window new` | Another window on the target selected by `--window`, the caller's host, or the frontmost/default target |
| `window focus ID` | Bring a window forward |
| `window fullscreen [on\|off\|toggle]` | |
| `window size [WIDTHxHEIGHT]` | Print or set the window's size; windows cannot be moved |
| `window detach\|reconnect` | Detach from, or reconnect to, the selected host |
| `ui sidebar\|statusbar [on\|off\|toggle]` | For this window and session |
| `ui font-size +\|-\|reset\|POINTS` | The terminal font size for this session |
| `ui panel workspaces\|overview\|pull-requests\|issues\|tasks\|automations\|usage` | Show an activity bar section: Workspaces with the sidebar expanded, Overview of what needs you on every host, the focused workspace's pull requests or issues across the window, Beads tasks across the window's projects, an unavailable job and run feed, or every host's plan usage |
| `run COMMAND` | Any app command by its `[keybindings]` name, as the palette runs it |
| `commands` | The names `run` accepts (no app needed) |
| `open DIR` | Open a folder as **Open Project** in the palette does, starting the app if needed (below) |

`open DIR` checks the folder where you run it and resolves it to an absolute,
canonical path. A path that does not exist exits 5; a file, or a folder in a
device namespace (`\\.\…`, `\\?\GLOBALROOT\…`, and on Unix `/dev`, `/proc`,
`/sys`), exits 1. The app then opens the command palette on
its projects and opens the folder there, which focuses the workspace already
rooted in it or starts one, on the window's local Herdr, and brings the window
forward. A dialog open in the window refuses it (exit 6); the palette shows
anything that goes wrong after that, such as the folder disappearing.

When no Spaceport is running and no `--window` is given, `open` starts the app,
as a plain `spaceport` would, then asks again every quarter second until the
window has connected to the local Herdr and accepts the folder. It waits at
most 20 seconds after starting the app, then exits 7. Explorer's **Open in
Spaceport** entry, which both Windows installers add, runs this command through
`spaceport-launcher.exe`, so no console window appears.

### The app's own tabs

| Command | What it does |
| --- | --- |
| `tab [list]` | Current page, Orchestrator, Devices, Core terminal, and editor tabs with IDs; retired source and review tabs are omitted |
| `tab focus\|close ID` | |
| `browser reload\|feedback` | See [browser tabs](../crates/sheprd-gpui/README.md#browser-tabs) |
| `browser list` | Page tabs only |
| `browser navigate ID URL` | Load another address in a page tab |
| `browser back\|forward ID` | |
| `browser zoom ID +\|-\|reset\|PERCENT` | Zoom a page tab a step, back to 100%, or to 50-300%; the tab keeps it |
| `browser text ID` | The page's visible text |
| `browser screenshot ID PATH` | A PNG of the visible page, at most 2000 px a side (macOS) |
| `devices` | The Devices overview |

### Worktrees and agents

| Command | What it does |
| --- | --- |
| `notes [list]` | Every worktree note |
| `notes get\|clear`, `notes set TEXT` | The note on your checkout (its repository and branch) |
| `worktree [status]`, `worktree mute\|unmute` | Whether your worktree's notifications are muted; silence them or let them through again |
| `feedback [--wait S]` | Page notes the user sent your pane |
| `scripts [list]` | The repository's `.herdr/worktree.toml` scripts (no app needed) |
| `scripts setup\|run` | Run one in a new tab; an unseen script still asks first. Archive removes the worktree, so it stays behind the app's delete dialog |
| `chat [status]` | The `sheprd-acp` chat panes of the window's host, this machine or an SSH host: agent, status, model, context window use, and whether the chat view or the terminal shows |
| `chat convert [--pane ID]` | Move the terminal agent in your pane (or ID, else the focused one) into a `sheprd-acp` chat in the same pane, keeping its conversation (see [Moving a terminal agent into a chat](rich-chat.md#moving-a-terminal-agent-into-a-chat)). Returns once the move has started; `chat` shows the chat when it attaches. Exits 6 while the agent works, 5 when the pane has no terminal agent, and 8 for an agent or pane that cannot move |
| `chat show\|hide\|toggle [--pane ID]` | Show the native chat view over your pane (or ID, else the focused one), or its terminal |
| `chat new AGENT [--pane ID]` | Open a tab running a `sheprd-acp` chat with a built-in agent (`claude`, `codex`, `copilot`, `gemini`, `opencode`, `pi`) in the workspace and folder of your pane (or ID). On an SSH host the tab runs that host's `sheprd-acp` (see [Chats on SSH hosts](#chats-on-ssh-hosts)). With no Herdr pane to go by, as when called from a Herdr 2 pane, a Herdr 2 tab in the active group splits its pane in use with the chat instead (see [Chats in Herdr 2 panes](rich-chat.md#chats-in-herdr-2-panes)); its pane id in `chat` is `core:` and the terminal's. Exits 8 when `sheprd-acp` is missing here, the host cannot be reached, or the agent is not installed on it (`sheprd-acp agents` lists those that are) |

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
| `host tag ID [TAG...]` | Replace the local tags that group a host in the sidebar; with no tags, clear them |
| `ports [list]` | Ports your workspace listens on, and forwards |
| `ports forward\|unforward PORT` | Forward a port of the SSH host to this machine |
| `ports open PORT` | Open `localhost:PORT` with the system handler; forwards a remote port before opening |
| `usage` | Plan usage on the selected host; reports unavailable until Core supplies a usage feed |
| `system` | Load of the selected host: CPU, memory, disk, uptime, and its GPU and video memory when enabled |
| `processes [--pane ID]` | The processes under a pane, with CPU and memory (no app needed) |

### Settings and services

| Command | What it does |
| --- | --- |
| `config [path]` | The Spaceport settings files, including the private `view.toml` path (no app needed) |
| `config get\|set\|unset KEY [VALUE]` | Read or change `spaceport/config.local.toml`, keeping its comments. A value that would not load, or an unknown key, is refused and nothing is written; the app applies the rest at once (no app needed) |
| `config check` | Report invalid settings, exit 1 if any |
| `config reload` | Reread Spaceport's settings files |
| `theme [list]`, `theme set NAME` | The app theme. A device's terminal theme is `config set devices.ID.theme NAME`, with the ID from `host list` minus its `ssh:` prefix |
| `font [list]`, `font set NAME`, `font reset` | |
| `language [status]`, `language list`, `language set system\|TAG` | The language set and the locale Spaceport shows, the languages there are, each with its name in itself as Settings > General lists it, or save the `language` setting (a BCP 47 tag; `en-XA` and `en-XB` are pseudo-locales). Only `status` asks the app, and without it reports what it would show |
| `keybinds [list]`, `keybinds check` | Shortcuts and reachable non-plugin custom Herdr commands |
| `caffeine [status]`, `caffeine on\|off\|toggle [--for 30m]` | Keep the machine awake |
| `sound [list]`, `sound play [done\|request]`, `bell` | Sounds play at `sound.volume` percent (`config set sound.volume 30`) |
| `update [status]`, `update check` | Never downloads or installs |
| `integrations [list]` | What Herdr offers the selected host |
| `integrations install-skill` | Install the agent skill (no app needed) |
| `logs [--lines N] [--path]` | The app's log (no app needed) |
| `diagnostics bundle PATH` | A redacted report for a bug, with the newest crash reports (no app needed) |


Focus-on-hover between split terminal panes is on by default. Disable it with
`spaceport config set focus_follows_mouse false`.
The change applies live; `config get focus_follows_mouse` reads it back.

An unnamed tab shows its number. To show its agent's terminal title instead,
run `spaceport config set tab_labels.agent_title true`.

### Events

`events` prints what happened in the app: windows opening, closing, and
gaining focus; the app's tabs opening and closing; notes kept for a pane;
worktree notes changing; and host status changes. Each event has a sequence
number; pass back the `INSTANCE:SEQ` cursor a call prints with `--after` to
get only newer events. A cursor from an earlier run of the app gets every
event the new run has kept, so a restart never hides events.

```sh
spaceport events --follow --kind tab_closed
spaceport wait tab-closed 12 --timeout 600
spaceport wait window            # also waits for the app to start
```

## What stays in the app

Commands never bypass a confirmation. Fanning out a prompt and running a
worktree script the app has not seen open the app's
own dialog, pointed at your checkout; the user confirms there.
The development build still has an inherited Herdr 1 startup path; removing it
is an open Spaceport task. The Core path may start only a separately installed,
trusted Core when needed. Spaceport packages do not bundle or upgrade Core.
`update` never installs the app.

Adding an SSH or local network device has no command: the Add Device dialog
runs it. Its setup workspace runs `spaceport __device-setup-attach SOCKET`,
an internal helper that shows the setup the app runs (see the Native Shell
README's "Local Network access"); it is not a command to call yourself, and
the socket it names is single use.

Herdr 1 scrollback search is not repeated here: `herdr pane read` covers it. Moving a
window and capturing a whole window have no command because the app cannot
do them.

## Links

`spaceport URL`, with a `spaceport://` link as the only argument, hands the
link to the running app, or starts the app and follows it once it is
connected. The development build does not register an OS-wide handler; its
Windows jump-list entries launch the executable with the link directly.
Protocol-activated notification clicks require a handler command pointing
exactly at the running executable and have not been verified on Windows.

| Link | What the app does |
| --- | --- |
| `spaceport://open?path=PATH` | Focus the workspace showing the Git checkout or worktree at `PATH`, or ask in the window before opening one there |
| `spaceport://device?id=ID` | Show the device `host list` names `ID` |
| `spaceport://notification?tag=TAG` | Open the pane of a notification this app posted |

A link is untrusted input, checked by the command and again by the app: only
those shapes, each parameter once, bounded lengths, no control characters, and
an absolute local path without `..` (on Windows a drive path, never a UNC or
`\\?\` one). A link only ever shows something; it never runs a command. A
refused link exits 2, an unknown device or notification 5, as for commands.
The request is `{"method": "link", "caller", "op": {"url"}}`.

## The protocol

One JSON request line, one JSON response line, then the socket closes. A
request is `{"method", "caller", "op"}`, where `caller` carries the pane,
workspace, daemon socket, and window, and `op` is tagged with `"op"`. Browser
requests keep their original flat form (`browser.open`, `browser.reload`,
`browser.feedback`, `notes.send`). `status` reports the protocol version,
currently 6. Requests and responses are bounded to 256 KiB, a request that
waits says for how long (at most 600 seconds), and the app answers at most a
few requests at a time.

### Windows

On Windows the same lines travel over a named pipe,
`\\.\pipe\herdr-gpui-control-SID`, where `SID` is your Windows user's
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
cargo install --locked --git https://github.com/sheprd-labs/sheprd sheprd-acp --root ~/.herdr
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
