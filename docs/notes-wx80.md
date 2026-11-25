# wx80 — documentation sweep, part 2

Crew task wx80. Five items across `docs/cli.md`, `skills/spaceport/SKILL.md`,
`docs/spaceport/START_HERE.md`, the `AGENTS.md` crate table, and a new
`docs/notes-next-preview.md`.

Fetched `origin` first and branched `feat/wx80-docs-2` from
`origin/main` = `a8fbdb566884cd514ac7899405fe10f2ee0aca08` (PR #246 merged,
described as `v20261125.1-9-ga8fbdb5668`). Every check below ran against that
tree, and the CLI evidence is the binary built from it.

## 1. `docs/cli.md` against the commands that exist

Cross-checked every documented row against the three places a command lives:
`crates/spaceport-gpui/src/cli/control.rs` (`FAMILIES`, the one list of family
words), each family's parser and usage text under
`crates/spaceport-gpui/src/cli/control/`, and the handlers under
`crates/spaceport-gpui/src/control/server/`. The 29 families the binary parses
are exactly the 29 the document has table rows for, and no row names a command
the binary refuses.

Built `cargo build --locked -p spaceport-gpui` and ran `--help` for the binary
and for each family; that output is the evidence at the end of this note.

### Drift found and fixed

| Where | Was | Now | Why |
| --- | --- | --- | --- |
| `notes set` row | `notes set TEXT` | `notes set TEXT...` | The parser joins every following word (`args.words()`), so a note may be sent unquoted |
| `theme set` row | `theme set NAME` | `theme set NAME...` | Same: `args.words()` |
| `font set` row | `font set NAME` | `font set NAME...` | Same: `args.words()` |
| `keybinds` row | "Shortcuts and reachable non-plugin custom Herdr commands" | "This window's shortcuts, and Herdr's custom commands other than a plugin's, each with whether its keys reach this window; `check` says which do not and exits 1 if any" | `keybinds list` prints every non-plugin custom command with its `Reach`, including ones that cannot run here, so "reachable" described neither the list nor `check`, which exits 1 when one does not reach |
| Events section | no `--wait` | `--wait SECONDS` documented | The parser and `EventsOp::Poll` both take it: the call parks until an event arrives or the wait runs out (at most 600 seconds), then answers with whatever it has |

### Checked and already correct

- Exit-code table (0–8) matches `crates/spaceport-gpui/src/control/client.rs`'s
  `exit` constants one for one, and the `--json` category names (`usage`,
  `not_running`, `timeout`, `unsupported`, `refused`) match the same file's
  naming.
- The app's own error codes named in the document (`workspace_not_found`,
  `no_window`, `not_found`, `invalid_request`, `failed`) are all variants of
  `protocol::ErrorCode`.
- Protocol version 38 matches `PROTOCOL_VERSION`; a test already pins the two
  together (`crates/spaceport-gpui/src/control/client/tests/documented_commands.rs`).
- Bounds: `window size` 200–16384 by 150–16384, `ui font-size` 8–48, caffeine
  `--for` 1 s to 24 h, `logs` 100 by default and 5000 at most, `wait` 600 at
  most, notes one line of 500 characters and 16 KiB before the app sees them,
  host tags 8 of 24 characters — each read in the parser or the settings type
  the document names.
- Paths and platform facts: `$XDG_STATE_HOME/spaceport/control.sock`, the pipe
  name `\\.\pipe\spaceport-control-SID` and its owner check, "the removed
  `update` command is rejected before app startup" (tested by
  `crates/spaceport-gpui/src/cli/tests/removed_updater.rs`),
  `run check-for-updates` in the catalog, and both `spaceport://` link
  shapes.

### Commands added since 2026-11-22 that the document omits

None. `git log --since=2026-11-22 -- crates/spaceport-commands
crates/spaceport-control-protocol crates/spaceport-gpui/src/control
crates/spaceport-gpui/src/cli` returns six commits (#233, #232, #205, #201,
#172, #179) and every one is a refactor or a fix to a command the document
already describes. `git log --oneline v20261125.1..origin/main` over the same
paths is empty. No family word, verb, or flag was added there in that window, so
this item only needed the drift fixes above.

## 2. `skills/spaceport/SKILL.md`

The skill's `name` is untouched: installed copies are found by it. Its command
table lists only commands that exist, and every `spaceport ...` line in it still
parses — the test `every_skill_command_line_parses` in
`crates/spaceport-gpui/src/control/client/tests/documented_commands.rs` proves
that, and it passed after the edit.

One row changed, to carry the fact `docs/cli.md` now states: "Block until
something happens in the app" also shows `spaceport events --wait 30` for the
next event, beside the `wait tab-closed` and `events --follow` forms it already
had. An agent that would otherwise poll in a loop has a bounded alternative that
`docs/cli.md` documents.

The frontmatter description did not need to change: item 1 added no new kind of
task, only corrected spellings of verbs the table already showed.

## 3. `docs/spaceport/START_HERE.md`

Read first, then changed one thing. The document has one place where it states
the fork's current decision posture — "Current runtime policy", which cites D58
— so D72 now sits beside it:

> Terminal links open read-only (D72). Spaceport reads the target itself — a URL
> to the browser, a file to the default app — and no Herdr `[[link_handlers]]`
> plugin runs from a click. The inherited plugin link handler stays removed as
> deliberate fork scope, not as an incidental simplification.

Where the wording comes from: `docs/spaceport/plan/decisions.json` D72, and
`docs/spaceport/evidence/t04-legacy-plugin-link-dependency.md`, which records
that the live plugin risk the audit described is gone rather than gated.

No other claim in the document was stale, so nothing else changed. Nothing was
restructured.

**Mobile:** the document does not describe mobile at all — no mention of the
phone, iOS, Android, or UniFFI — so the mobile shared-rules direction had no
claim to correct. `SPEC.md` line 13 does describe mobile ("Defer iOS, Android,
UniFFI and Ghostty evaluation"), but that file is outside this task's scope and
its deferral is about the desktop plan, not the separate mobile workstream. Left
alone deliberately; see the open questions below.

## 4. `AGENTS.md` crate table, newest rows

- **`spaceport-activation`** does not exist. `ls crates` has no such crate, and
  nothing in the tree mentions the name. The activation machinery lives in
  `spaceport-live`'s `activation` module (`Fence`, `TIMEOUT`,
  `SurfaceActivation`), and the `spaceport-live` row already describes the fence
  it implements. Nothing to check, and nothing invented.
- **`spaceport-keymap`** — one sentence was stale. The row said the crate exists
  "for `Keystroke`, `Modifiers`, `OperatingSystem` and `Keystroke::parse_with_os`
  alone", but the crate also names `gpui::KeybindingKeystroke` (in
  `typed_matches`, which `custom_command` calls in production, not just in
  tests) and `gpui::InvalidKeystrokeError` (the source of two of its `Error`
  variants). The row now says so, and keeps the rest of its wording, including
  the queries list, which matches the nine `Keymap` methods exactly.
- **`spaceport-live`** — accurate. `lib.rs`, `state.rs` (`surface_images`,
  `pending_snapshot`, `apply`) and `projection.rs` match the row's snapshot,
  surface, images, notices, mailboxes, ordered reduction, coherent-pair rule,
  one newer projection, activation fence and navigation target. Its
  dependencies confirm "no GPUI".
- **`spaceport-palette`** — accurate. `entry.rs` carries label, detail, badge
  and action; `action.rs` the identity; `Target` the destination context; `Key`
  the frecency key; `agent_filter.rs` the Go To filter and heading order;
  `nucleo-matcher` the query scoring. Its dependency list confirms "no GPUI",
  and it re-exports `spaceport-live`'s navigation target as the row says.

Rows that were accurate were left alone, per the task.

## 5. `docs/notes-next-preview.md`

New. `git-cliff --config cliff.toml v20261125.1..origin/main` returns four
`Changed` entries (#238, #239, #241, #243) and two `Documentation` entries. The
range holds nine commits; the `ci`, `chore` types are dropped by `cliff.toml`,
which is why two CI commits and one chore commit do not appear.

Read all six commit bodies before writing a word of summary, so the plain
language says what each commit actually did:

- #238 and #241 touch the phone as well as the desktop. #241 fixes two real
  phone defects (a `/private/tmp` path a macOS host prints, and a search sent
  against an unsettled frame). Those are the only user-visible fixes in the
  range, and neither is in the desktop preview build, so the draft says so
  instead of implying a desktop fix.
- #239 and #243 change no shortcut, no `fl!` string, and no `spaceport keybinds`
  answer, which is what the draft's "shortcuts work exactly as before" rests on.

The draft is marked a draft in its first lines, gives the range and the SHAs it
came from, and ends with what must be re-run before publishing.

## Open questions

Two things this sweep could not settle from the documents alone. Neither blocks
this change; both are the supervisor's call.

1. **`SPEC.md` still says mobile is deferred (line 13 at this revision).** It
   reads "Defer iOS,
   Android, UniFFI and Ghostty evaluation. Do not create speculative mobile
   crates or compatibility layers." That was the desktop plan's deferral. The
   repository now has a mobile workstream that reads the desktop crates by path,
   which `README.md` and the root `AGENTS.md` both describe. `START_HERE.md`
   never mentions mobile, so item 3 gave me no claim to fix there. Whether
   `SPEC.md` should be amended is a scope call I did not make.
2. **`spaceport-activation` in the task's crate list.** No such crate exists, so
   item 4's first row had nothing to check. If a later task extracts
   `spaceport-live`'s `activation` module into a crate of that name, the row
   should be written from the crate as it lands.

## What ran

```
cargo build --locked -p spaceport-gpui                                   passed
./target/debug/spaceport --help                                          passed
./target/debug/spaceport <family> --help, all 29 families                passed
cargo test --locked -p spaceport-gpui --bin spaceport control::client    passed
  (50 passed, 0 failed; includes the three documented_commands contracts)
git diff --check                                                         passed
cd docs/spaceport && uv run --with jsonschema==4.26.0 python tools/spec.py check
                                                                         passed
  (72 decisions, 381 local file links checked, schema validation passed)
```

Not run here: `just ci`, cross-platform lints, and native window checks.
Nothing in this change touches a window or a `cfg` gate, so those jobs are the
existing CI ones and this branch defers to them.

## Evidence: the CLI's own `--help`

Each block is the binary's output, not a transcription.

### `spaceport --help`

```
spaceport [--socket CLIENT_SOCKET]
Connects to Local and saved SSH hosts; never installs remote software.
Starts the local Herdr daemon if needed; never stops it.
Explicit --socket targets are attach-only and isolate the GUI to one existing daemon.
  --build-info        Print the executable's build identity without starting the GUI

Commands for the running app (each with --help):
  status            Whether Spaceport runs, its version, windows, and hosts
  skill             Print the agent skill describing these commands
  app               Bring the app forward, hide it, or quit it
  window            List, open, focus, resize, detach, or reconnect windows
  ui                Sidebar, status bar, and terminal font size
  tab               List, focus, or close the app's own tabs
  devices           Open the Devices overview
  chat              Native views of sheprd-acp chats: start one, list them, show or hide one
  notes             Notes kept on worktrees
  worktree          Mute or unmute the caller's worktree
  host              List the window's hosts, show one, or mute one
  ports             Listening ports, and forwarding them to this machine
  processes         The processes running under a pane
  usage             AI plan usage on the selected host
  config            Spaceport settings: path, get, set, reload, check
  theme             List or choose a color theme
  font              List or choose the terminal font
  language          The language Spaceport shows, or choose one
  keybinds          List shortcuts and check them for conflicts
  caffeine          Keep the machine awake
  sound             List or play notification sounds
  bell              Ring the terminal bell
  events            What happened in the app, as a stream
  wait              Wait for a window, a host to come online, or a tab to close
  run               Run any app command by name, as the command palette does
  commands          List the commands `run` accepts
  logs              Print or locate the app's log
  diagnostics       Write a redacted bundle for a bug report
  integrations      Agent integrations and the agent skill
```

### `spaceport status --help`

```
spaceport status
    Whether Spaceport runs, its version, protocol, windows, hosts, and the
    current Core machine catalog's freshness and count. No machine lookup runs.
    Exits 3 when it does not run.

Every command accepts --json, printing one JSON document (`events --follow`
writes one object per line instead), and --window ID, acting in that window
(IDs come from `window list`); a command about the whole app, such as `status`
or `caffeine`, only checks that the window is there. By default a command acts
in the window showing the caller's workspace (HERDR_WORKSPACE_ID), and exits 5
when no window shows it; outside Herdr, in the frontmost window.
Exit status: 0 done, 1 refused, 2 invalid arguments, 3 Spaceport not running,
4 nothing yet (no notes, no events, no chat panes, no ports, no usage, no log,
a key not set, or the wait not met), 5 not found, 6 busy (a dialog is open),
7 timed out, 8 unsupported on this platform or by this app version.
```

### `spaceport skill --help`

```
spaceport skill
    Print the agent skill describing these commands.

Every command accepts --json, printing one JSON document (`events --follow`
writes one object per line instead), and --window ID, acting in that window
(IDs come from `window list`); a command about the whole app, such as `status`
or `caffeine`, only checks that the window is there. By default a command acts
in the window showing the caller's workspace (HERDR_WORKSPACE_ID), and exits 5
when no window shows it; outside Herdr, in the frontmost window.
Exit status: 0 done, 1 refused, 2 invalid arguments, 3 Spaceport not running,
4 nothing yet (no notes, no events, no chat panes, no ports, no usage, no log,
a key not set, or the wait not met), 5 not found, 6 busy (a dialog is open),
7 timed out, 8 unsupported on this platform or by this app version.
```

### `spaceport app --help`

```
spaceport app focus|hide|quit
    Bring the app forward, hide it, or quit it. Quitting leaves Herdr
    and its terminals running.

Every command accepts --json, printing one JSON document (`events --follow`
writes one object per line instead), and --window ID, acting in that window
(IDs come from `window list`); a command about the whole app, such as `status`
or `caffeine`, only checks that the window is there. By default a command acts
in the window showing the caller's workspace (HERDR_WORKSPACE_ID), and exits 5
when no window shows it; outside Herdr, in the frontmost window.
Exit status: 0 done, 1 refused, 2 invalid arguments, 3 Spaceport not running,
4 nothing yet (no notes, no events, no chat panes, no ports, no usage, no log,
a key not set, or the wait not met), 5 not found, 6 busy (a dialog is open),
7 timed out, 8 unsupported on this platform or by this app version.
```

### `spaceport window --help`

```
spaceport window [list]
    The app's windows, with IDs for --window.
spaceport window new
    Open another window on the selected window's exact Core group or legacy
    launch target, the caller's host, or the frontmost/default target. A
    stale Core group is refused before the window opens.
spaceport window focus ID
spaceport window fullscreen [on|off|toggle]
spaceport window size [WIDTHxHEIGHT]
    Print the window's size, or resize it. Windows cannot be moved.
spaceport window detach|reconnect
    Detach from, or reconnect to, the window's selected host.

Every command accepts --json, printing one JSON document (`events --follow`
writes one object per line instead), and --window ID, acting in that window
(IDs come from `window list`); a command about the whole app, such as `status`
or `caffeine`, only checks that the window is there. By default a command acts
in the window showing the caller's workspace (HERDR_WORKSPACE_ID), and exits 5
when no window shows it; outside Herdr, in the frontmost window.
Exit status: 0 done, 1 refused, 2 invalid arguments, 3 Spaceport not running,
4 nothing yet (no notes, no events, no chat panes, no ports, no usage, no log,
a key not set, or the wait not met), 5 not found, 6 busy (a dialog is open),
7 timed out, 8 unsupported on this platform or by this app version.
```

### `spaceport ui --help`

```
spaceport ui sidebar [on|off|toggle]
spaceport ui statusbar [on|off|toggle]
spaceport ui font-size +|-|reset|POINTS
spaceport ui panel workspaces|overview|automations|usage
spaceport ui agent-idle [--pane ID]
    Read the agent idle time observed by this window.
    For this window and session; settings are changed with `config`.

Every command accepts --json, printing one JSON document (`events --follow`
writes one object per line instead), and --window ID, acting in that window
(IDs come from `window list`); a command about the whole app, such as `status`
or `caffeine`, only checks that the window is there. By default a command acts
in the window showing the caller's workspace (HERDR_WORKSPACE_ID), and exits 5
when no window shows it; outside Herdr, in the frontmost window.
Exit status: 0 done, 1 refused, 2 invalid arguments, 3 Spaceport not running,
4 nothing yet (no notes, no events, no chat panes, no ports, no usage, no log,
a key not set, or the wait not met), 5 not found, 6 busy (a dialog is open),
7 timed out, 8 unsupported on this platform or by this app version.
```

### `spaceport tab --help`

```
spaceport tab [list]
    The app's own tabs: pages, Devices, Core terminals.
    Terminal tabs belong to Herdr: use `herdr tab`.
spaceport tab focus|close ID
    Show a tab, switching to its workspace, or close it.

Every command accepts --json, printing one JSON document (`events --follow`
writes one object per line instead), and --window ID, acting in that window
(IDs come from `window list`); a command about the whole app, such as `status`
or `caffeine`, only checks that the window is there. By default a command acts
in the window showing the caller's workspace (HERDR_WORKSPACE_ID), and exits 5
when no window shows it; outside Herdr, in the frontmost window.
Exit status: 0 done, 1 refused, 2 invalid arguments, 3 Spaceport not running,
4 nothing yet (no notes, no events, no chat panes, no ports, no usage, no log,
a key not set, or the wait not met), 5 not found, 6 busy (a dialog is open),
7 timed out, 8 unsupported on this platform or by this app version.
```

### `spaceport devices --help`

```
spaceport devices [open]
    Open the Devices overview tab in the caller's workspace.

Every command accepts --json, printing one JSON document (`events --follow`
writes one object per line instead), and --window ID, acting in that window
(IDs come from `window list`); a command about the whole app, such as `status`
or `caffeine`, only checks that the window is there. By default a command acts
in the window showing the caller's workspace (HERDR_WORKSPACE_ID), and exits 5
when no window shows it; outside Herdr, in the frontmost window.
Exit status: 0 done, 1 refused, 2 invalid arguments, 3 Spaceport not running,
4 nothing yet (no notes, no events, no chat panes, no ports, no usage, no log,
a key not set, or the wait not met), 5 not found, 6 busy (a dialog is open),
7 timed out, 8 unsupported on this platform or by this app version.
```

### `spaceport chat --help`

```
spaceport chat [status]
    The window's sheprd-acp chat panes on this machine or an SSH host: pane, agent, status,
    and whether the native chat view or the terminal shows. Exits 4 when
    there are none.
spaceport chat show|hide|toggle [--pane ID]
    Show the native chat view over the pane, or its terminal, for this
    window. The pane is the caller's own, else the focused one. Exits 5 when
    it runs no chat the window can show.
spaceport chat new AGENT [--pane ID]
    Open a tab running a sheprd-acp chat with AGENT (claude, codex, copilot,
    gemini, opencode, or pi) in the workspace and folder of the pane, else the
    caller's own. On an SSH host the tab runs that host's sheprd-acp, from
    ~/.herdr/bin or PATH. With no Herdr pane to go by, a Herdr 2 tab in the
    active group splits its pane in use with the chat instead. The chat view
    attaches once the program starts.
    Exits 8 when ACP chats are off in Settings, sheprd-acp is missing here,
    or the agent is not installed there. Exits 1 when the host cannot be
    reached.

Every command accepts --json, printing one JSON document (`events --follow`
writes one object per line instead), and --window ID, acting in that window
(IDs come from `window list`); a command about the whole app, such as `status`
or `caffeine`, only checks that the window is there. By default a command acts
in the window showing the caller's workspace (HERDR_WORKSPACE_ID), and exits 5
when no window shows it; outside Herdr, in the frontmost window.
Exit status: 0 done, 1 refused, 2 invalid arguments, 3 Spaceport not running,
4 nothing yet (no notes, no events, no chat panes, no ports, no usage, no log,
a key not set, or the wait not met), 5 not found, 6 busy (a dialog is open),
7 timed out, 8 unsupported on this platform or by this app version.
```

### `spaceport notes --help`

```
spaceport notes [list]
    Every worktree note, most recently edited first.
spaceport notes get|clear
spaceport notes set TEXT...
    The note on the caller's checkout (its repository and branch), or the
    focused workspace's outside a Herdr pane. Notes are one line of at most
    500 characters. `list` and `get` exit 4 when there is none.

Every command accepts --json, printing one JSON document (`events --follow`
writes one object per line instead), and --window ID, acting in that window
(IDs come from `window list`); a command about the whole app, such as `status`
or `caffeine`, only checks that the window is there. By default a command acts
in the window showing the caller's workspace (HERDR_WORKSPACE_ID), and exits 5
when no window shows it; outside Herdr, in the frontmost window.
Exit status: 0 done, 1 refused, 2 invalid arguments, 3 Spaceport not running,
4 nothing yet (no notes, no events, no chat panes, no ports, no usage, no log,
a key not set, or the wait not met), 5 not found, 6 busy (a dialog is open),
7 timed out, 8 unsupported on this platform or by this app version.
```

### `spaceport worktree --help`

```
spaceport worktree [status]
    Whether the caller's worktree is muted.
spaceport worktree mute|unmute
    Silence its notifications, or let them through again. Mutes stay in
    this app; nothing reaches the daemon.

Every command accepts --json, printing one JSON document (`events --follow`
writes one object per line instead), and --window ID, acting in that window
(IDs come from `window list`); a command about the whole app, such as `status`
or `caffeine`, only checks that the window is there. By default a command acts
in the window showing the caller's workspace (HERDR_WORKSPACE_ID), and exits 5
when no window shows it; outside Herdr, in the frontmost window.
Exit status: 0 done, 1 refused, 2 invalid arguments, 3 Spaceport not running,
4 nothing yet (no notes, no events, no chat panes, no ports, no usage, no log,
a key not set, or the wait not met), 5 not found, 6 busy (a dialog is open),
7 timed out, 8 unsupported on this platform or by this app version.
```

### `spaceport host --help`

```
spaceport host [list]
    The window's hosts and their status. Herdr's `machine` command
    manages saved hosts.
spaceport host mute|unmute [HOST]
    Silence a host's notifications as a whole, or let them through again,
    by ID or label; absent, the caller's host. Works while it is offline.
spaceport host select ID
    Show that host in the window, as choosing it in the sidebar does.
spaceport host tag ID [TAG...]
    Replace the local tags that group that host in the sidebar; with no
    tags, clear them.

Every command accepts --json, printing one JSON document (`events --follow`
writes one object per line instead), and --window ID, acting in that window
(IDs come from `window list`); a command about the whole app, such as `status`
or `caffeine`, only checks that the window is there. By default a command acts
in the window showing the caller's workspace (HERDR_WORKSPACE_ID), and exits 5
when no window shows it; outside Herdr, in the frontmost window.
Exit status: 0 done, 1 refused, 2 invalid arguments, 3 Spaceport not running,
4 nothing yet (no notes, no events, no chat panes, no ports, no usage, no log,
a key not set, or the wait not met), 5 not found, 6 busy (a dialog is open),
7 timed out, 8 unsupported on this platform or by this app version.
```

### `spaceport ports --help`

```
spaceport ports [list]
    Ports the caller's workspace's processes listen on, and forwards.
    Listening ports are known only while show_listening_ports is on.
spaceport ports forward|unforward PORT
    Forward a port of the workspace's SSH host to this machine, or stop.
spaceport ports open PORT
    Open localhost:PORT with the system URL handler, forwarding it first if remote.

Every command accepts --json, printing one JSON document (`events --follow`
writes one object per line instead), and --window ID, acting in that window
(IDs come from `window list`); a command about the whole app, such as `status`
or `caffeine`, only checks that the window is there. By default a command acts
in the window showing the caller's workspace (HERDR_WORKSPACE_ID), and exits 5
when no window shows it; outside Herdr, in the frontmost window.
Exit status: 0 done, 1 refused, 2 invalid arguments, 3 Spaceport not running,
4 nothing yet (no notes, no events, no chat panes, no ports, no usage, no log,
a key not set, or the wait not met), 5 not found, 6 busy (a dialog is open),
7 timed out, 8 unsupported on this platform or by this app version.
```

### `spaceport processes --help`

```
spaceport processes [--pane ID]
    The processes under a pane (default HERDR_PANE_ID), with CPU and
    memory. Runs here; Spaceport need not run.

Every command accepts --json, printing one JSON document (`events --follow`
writes one object per line instead), and --window ID, acting in that window
(IDs come from `window list`); a command about the whole app, such as `status`
or `caffeine`, only checks that the window is there. By default a command acts
in the window showing the caller's workspace (HERDR_WORKSPACE_ID), and exits 5
when no window shows it; outside Herdr, in the frontmost window.
Exit status: 0 done, 1 refused, 2 invalid arguments, 3 Spaceport not running,
4 nothing yet (no notes, no events, no chat panes, no ports, no usage, no log,
a key not set, or the wait not met), 5 not found, 6 busy (a dialog is open),
7 timed out, 8 unsupported on this platform or by this app version.
```

### `spaceport usage --help`

```
spaceport usage
    Plan usage on the selected host when Core supplies a feed; currently unavailable.

Every command accepts --json, printing one JSON document (`events --follow`
writes one object per line instead), and --window ID, acting in that window
(IDs come from `window list`); a command about the whole app, such as `status`
or `caffeine`, only checks that the window is there. By default a command acts
in the window showing the caller's workspace (HERDR_WORKSPACE_ID), and exits 5
when no window shows it; outside Herdr, in the frontmost window.
Exit status: 0 done, 1 refused, 2 invalid arguments, 3 Spaceport not running,
4 nothing yet (no notes, no events, no chat panes, no ports, no usage, no log,
a key not set, or the wait not met), 5 not found, 6 busy (a dialog is open),
7 timed out, 8 unsupported on this platform or by this app version.
```

### `spaceport config --help`

```
spaceport config [path]
    The settings files.
spaceport config get KEY
    The setting's value, as TOML: your override, else the default.
spaceport config set KEY VALUE
spaceport config unset KEY
    Change a setting in spaceport/config.local.toml. VALUE is read as TOML
    (14, true, ["a"], { show = true }), else as text. A value or key
    Spaceport would not load is refused and the file left as it was.
    A running Spaceport applies the change at once. KEY is dotted, such
    as terminal.size. Unsetting a key that is not set exits 4.
spaceport config check
    Report settings that do not load and unknown keys, and, when Spaceport
    runs, fonts that are not installed; exits 1 when there are any.
spaceport config reload
    Reread Spaceport's settings files.
Only reload needs Spaceport running.

Every command accepts --json, printing one JSON document (`events --follow`
writes one object per line instead), and --window ID, acting in that window
(IDs come from `window list`); a command about the whole app, such as `status`
or `caffeine`, only checks that the window is there. By default a command acts
in the window showing the caller's workspace (HERDR_WORKSPACE_ID), and exits 5
when no window shows it; outside Herdr, in the frontmost window.
Exit status: 0 done, 1 refused, 2 invalid arguments, 3 Spaceport not running,
4 nothing yet (no notes, no events, no chat panes, no ports, no usage, no log,
a key not set, or the wait not met), 5 not found, 6 busy (a dialog is open),
7 timed out, 8 unsupported on this platform or by this app version.
```

### `spaceport theme --help`

```
spaceport theme [list]
    The themes there are; * marks the selected one.
spaceport theme set NAME...
    Choose and save a theme, or light:NAME,dark:NAME to follow the
    system appearance. Saved even when Spaceport is not running.

Every command accepts --json, printing one JSON document (`events --follow`
writes one object per line instead), and --window ID, acting in that window
(IDs come from `window list`); a command about the whole app, such as `status`
or `caffeine`, only checks that the window is there. By default a command acts
in the window showing the caller's workspace (HERDR_WORKSPACE_ID), and exits 5
when no window shows it; outside Herdr, in the frontmost window.
Exit status: 0 done, 1 refused, 2 invalid arguments, 3 Spaceport not running,
4 nothing yet (no notes, no events, no chat panes, no ports, no usage, no log,
a key not set, or the wait not met), 5 not found, 6 busy (a dialog is open),
7 timed out, 8 unsupported on this platform or by this app version.
```

### `spaceport font --help`

```
spaceport font [list]
    The installed font families.
spaceport font set NAME...
spaceport font reset
    The font of every face; reset goes back to the default.
These need Spaceport running, which knows the installed fonts.

Every command accepts --json, printing one JSON document (`events --follow`
writes one object per line instead), and --window ID, acting in that window
(IDs come from `window list`); a command about the whole app, such as `status`
or `caffeine`, only checks that the window is there. By default a command acts
in the window showing the caller's workspace (HERDR_WORKSPACE_ID), and exits 5
when no window shows it; outside Herdr, in the frontmost window.
Exit status: 0 done, 1 refused, 2 invalid arguments, 3 Spaceport not running,
4 nothing yet (no notes, no events, no chat panes, no ports, no usage, no log,
a key not set, or the wait not met), 5 not found, 6 busy (a dialog is open),
7 timed out, 8 unsupported on this platform or by this app version.
```

### `spaceport language --help`

```
spaceport language [status]
    The language set, and the locale Spaceport shows: the best match for
    the setting among the languages there are, else en-US.
spaceport language list
    The languages there are, each named in itself, and the en-XA and en-XB
    pseudo-locales.
spaceport language set system|TAG
    Save the language setting: system follows the OS's languages; TAG is
    a BCP 47 tag such as fr-CA. A running Spaceport switches at once.
Only status asks Spaceport; without it, status reports what it would show.

Every command accepts --json, printing one JSON document (`events --follow`
writes one object per line instead), and --window ID, acting in that window
(IDs come from `window list`); a command about the whole app, such as `status`
or `caffeine`, only checks that the window is there. By default a command acts
in the window showing the caller's workspace (HERDR_WORKSPACE_ID), and exits 5
when no window shows it; outside Herdr, in the frontmost window.
Exit status: 0 done, 1 refused, 2 invalid arguments, 3 Spaceport not running,
4 nothing yet (no notes, no events, no chat panes, no ports, no usage, no log,
a key not set, or the wait not met), 5 not found, 6 busy (a dialog is open),
7 timed out, 8 unsupported on this platform or by this app version.
```

### `spaceport keybinds --help`

```
spaceport keybinds [list]
    The window's shortcuts.
spaceport keybinds check
    Herdr's custom commands this client cannot reach; exits 1 if any.

Every command accepts --json, printing one JSON document (`events --follow`
writes one object per line instead), and --window ID, acting in that window
(IDs come from `window list`); a command about the whole app, such as `status`
or `caffeine`, only checks that the window is there. By default a command acts
in the window showing the caller's workspace (HERDR_WORKSPACE_ID), and exits 5
when no window shows it; outside Herdr, in the frontmost window.
Exit status: 0 done, 1 refused, 2 invalid arguments, 3 Spaceport not running,
4 nothing yet (no notes, no events, no chat panes, no ports, no usage, no log,
a key not set, or the wait not met), 5 not found, 6 busy (a dialog is open),
7 timed out, 8 unsupported on this platform or by this app version.
```

### `spaceport caffeine --help`

```
spaceport caffeine [status]
spaceport caffeine on|off|toggle [--for 90s|30m|2h]
    Keep the display and the machine awake. With --for (at most 24h),
    caffeine turns off then, unless it was changed in the meantime.

Every command accepts --json, printing one JSON document (`events --follow`
writes one object per line instead), and --window ID, acting in that window
(IDs come from `window list`); a command about the whole app, such as `status`
or `caffeine`, only checks that the window is there. By default a command acts
in the window showing the caller's workspace (HERDR_WORKSPACE_ID), and exits 5
when no window shows it; outside Herdr, in the frontmost window.
Exit status: 0 done, 1 refused, 2 invalid arguments, 3 Spaceport not running,
4 nothing yet (no notes, no events, no chat panes, no ports, no usage, no log,
a key not set, or the wait not met), 5 not found, 6 busy (a dialog is open),
7 timed out, 8 unsupported on this platform or by this app version.
```

### `spaceport sound --help`

```
spaceport sound [list]
    The notification sounds.
spaceport sound play [done|request]
    Play one now, from the file Herdr's sound settings name or the
    built-in one.

Every command accepts --json, printing one JSON document (`events --follow`
writes one object per line instead), and --window ID, acting in that window
(IDs come from `window list`); a command about the whole app, such as `status`
or `caffeine`, only checks that the window is there. By default a command acts
in the window showing the caller's workspace (HERDR_WORKSPACE_ID), and exits 5
when no window shows it; outside Herdr, in the frontmost window.
Exit status: 0 done, 1 refused, 2 invalid arguments, 3 Spaceport not running,
4 nothing yet (no notes, no events, no chat panes, no ports, no usage, no log,
a key not set, or the wait not met), 5 not found, 6 busy (a dialog is open),
7 timed out, 8 unsupported on this platform or by this app version.
```

### `spaceport bell --help`

```
spaceport bell
    Ring the system bell, as a terminal bell does.

Every command accepts --json, printing one JSON document (`events --follow`
writes one object per line instead), and --window ID, acting in that window
(IDs come from `window list`); a command about the whole app, such as `status`
or `caffeine`, only checks that the window is there. By default a command acts
in the window showing the caller's workspace (HERDR_WORKSPACE_ID), and exits 5
when no window shows it; outside Herdr, in the frontmost window.
Exit status: 0 done, 1 refused, 2 invalid arguments, 3 Spaceport not running,
4 nothing yet (no notes, no events, no chat panes, no ports, no usage, no log,
a key not set, or the wait not met), 5 not found, 6 busy (a dialog is open),
7 timed out, 8 unsupported on this platform or by this app version.
```

### `spaceport events --help`

```
spaceport events [--after CURSOR] [--wait SECONDS] [--kind KIND]... [--follow]
    What happened since CURSOR, the INSTANCE:SEQ the last call printed:
    windows and tabs opening and closing, notes, and host status. A cursor
    from an earlier app run gets everything this run kept. --follow keeps
    printing until interrupted. Kinds:
    window_opened, window_closed, window_focused, tab_opened, tab_closed,
    note, host.

Every command accepts --json, printing one JSON document (`events --follow`
writes one object per line instead), and --window ID, acting in that window
(IDs come from `window list`); a command about the whole app, such as `status`
or `caffeine`, only checks that the window is there. By default a command acts
in the window showing the caller's workspace (HERDR_WORKSPACE_ID), and exits 5
when no window shows it; outside Herdr, in the frontmost window.
Exit status: 0 done, 1 refused, 2 invalid arguments, 3 Spaceport not running,
4 nothing yet (no notes, no events, no chat panes, no ports, no usage, no log,
a key not set, or the wait not met), 5 not found, 6 busy (a dialog is open),
7 timed out, 8 unsupported on this platform or by this app version.
```

### `spaceport wait --help`

```
spaceport wait window|online|tab-closed ID [--timeout SECONDS]
    Wait (default 60, at most 600 seconds). Exits 4 when time runs out.

Every command accepts --json, printing one JSON document (`events --follow`
writes one object per line instead), and --window ID, acting in that window
(IDs come from `window list`); a command about the whole app, such as `status`
or `caffeine`, only checks that the window is there. By default a command acts
in the window showing the caller's workspace (HERDR_WORKSPACE_ID), and exits 5
when no window shows it; outside Herdr, in the frontmost window.
Exit status: 0 done, 1 refused, 2 invalid arguments, 3 Spaceport not running,
4 nothing yet (no notes, no events, no chat panes, no ports, no usage, no log,
a key not set, or the wait not met), 5 not found, 6 busy (a dialog is open),
7 timed out, 8 unsupported on this platform or by this app version.
```

### `spaceport run --help`

```
spaceport run COMMAND
    Run an app command by its [keybindings] name, as the command palette
    does, for example toggle_sidebar. See `commands`.

Every command accepts --json, printing one JSON document (`events --follow`
writes one object per line instead), and --window ID, acting in that window
(IDs come from `window list`); a command about the whole app, such as `status`
or `caffeine`, only checks that the window is there. By default a command acts
in the window showing the caller's workspace (HERDR_WORKSPACE_ID), and exits 5
when no window shows it; outside Herdr, in the frontmost window.
Exit status: 0 done, 1 refused, 2 invalid arguments, 3 Spaceport not running,
4 nothing yet (no notes, no events, no chat panes, no ports, no usage, no log,
a key not set, or the wait not met), 5 not found, 6 busy (a dialog is open),
7 timed out, 8 unsupported on this platform or by this app version.
```

### `spaceport commands --help`

```
spaceport commands
    The commands `run` accepts, with their labels.

Every command accepts --json, printing one JSON document (`events --follow`
writes one object per line instead), and --window ID, acting in that window
(IDs come from `window list`); a command about the whole app, such as `status`
or `caffeine`, only checks that the window is there. By default a command acts
in the window showing the caller's workspace (HERDR_WORKSPACE_ID), and exits 5
when no window shows it; outside Herdr, in the frontmost window.
Exit status: 0 done, 1 refused, 2 invalid arguments, 3 Spaceport not running,
4 nothing yet (no notes, no events, no chat panes, no ports, no usage, no log,
a key not set, or the wait not met), 5 not found, 6 busy (a dialog is open),
7 timed out, 8 unsupported on this platform or by this app version.
```

### `spaceport logs --help`

```
spaceport logs [--lines N] [--path]
    The app's latest log records (default 100, at most 5000), or the log file's path.
    Runs here; Spaceport need not run. Exits 4 when there is no log yet.

Every command accepts --json, printing one JSON document (`events --follow`
writes one object per line instead), and --window ID, acting in that window
(IDs come from `window list`); a command about the whole app, such as `status`
or `caffeine`, only checks that the window is there. By default a command acts
in the window showing the caller's workspace (HERDR_WORKSPACE_ID), and exits 5
when no window shows it; outside Herdr, in the frontmost window.
Exit status: 0 done, 1 refused, 2 invalid arguments, 3 Spaceport not running,
4 nothing yet (no notes, no events, no chat panes, no ports, no usage, no log,
a key not set, or the wait not met), 5 not found, 6 busy (a dialog is open),
7 timed out, 8 unsupported on this platform or by this app version.
```

### `spaceport diagnostics --help`

```
spaceport diagnostics bundle PATH
    Write build information, settings diagnostics, your settings
    overrides, and recent logs to a new file at PATH for a bug report,
    with home paths shortened and secret-looking values redacted. Read
    it before sharing. Never replaces an existing file. Runs here.

Every command accepts --json, printing one JSON document (`events --follow`
writes one object per line instead), and --window ID, acting in that window
(IDs come from `window list`); a command about the whole app, such as `status`
or `caffeine`, only checks that the window is there. By default a command acts
in the window showing the caller's workspace (HERDR_WORKSPACE_ID), and exits 5
when no window shows it; outside Herdr, in the frontmost window.
Exit status: 0 done, 1 refused, 2 invalid arguments, 3 Spaceport not running,
4 nothing yet (no notes, no events, no chat panes, no ports, no usage, no log,
a key not set, or the wait not met), 5 not found, 6 busy (a dialog is open),
7 timed out, 8 unsupported on this platform or by this app version.
```

### `spaceport integrations --help`

```
spaceport integrations [list]
    The agent integrations Herdr offers the window's host. Install them
    from Spaceport's integrations settings.
spaceport integrations install-skill
    Install or update the agent skill for Claude Code and other agents.
    Runs here; Spaceport need not run.

Every command accepts --json, printing one JSON document (`events --follow`
writes one object per line instead), and --window ID, acting in that window
(IDs come from `window list`); a command about the whole app, such as `status`
or `caffeine`, only checks that the window is there. By default a command acts
in the window showing the caller's workspace (HERDR_WORKSPACE_ID), and exits 5
when no window shows it; outside Herdr, in the frontmost window.
Exit status: 0 done, 1 refused, 2 invalid arguments, 3 Spaceport not running,
4 nothing yet (no notes, no events, no chat panes, no ports, no usage, no log,
a key not set, or the wait not met), 5 not found, 6 busy (a dialog is open),
7 timed out, 8 unsupported on this platform or by this app version.
```
