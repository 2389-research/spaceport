# CLI/GUI command consistency and exact targeting (task y1rm)

Scope: task **y1rm** ("Finish UI and CLI command consistency and exact
targeting"), spec refs T09, A35-A39/A55. This note is the deliverable: a
parity matrix of every `spaceport` command against the app behaviour it
drives, the fixes the audit earned, and the questions that need an owner.

Read-only audit plus five small fixes. No command was added, no protocol
ordering changed, no vendored bincode touched.

## What was audited

| Side | Path |
| --- | --- |
| Commands, parsing | `crates/sheprd-gpui/src/cli/control.rs` (`FAMILIES`, `parse`, `switch`, `--json`/`--window`), `cli/control/{app,chat,hosts,plain,settings,tabs,work}.rs` |
| Wire types | `crates/sheprd-control-protocol/src/protocol.rs`, `protocol/{app,chat,events,hosts,settings,tabs,work}.rs` |
| Handlers | `crates/sheprd-gpui/src/control/server.rs` (`window_for`, `serve`, `in_window`, `in_free_window`), `control/server/{app,chat,events,hosts,settings,tabs,work}.rs`, `control/server/{hosts,settings,tabs,work}/*.rs`, `control/targets.rs` |
| Client output, exit codes | `crates/sheprd-gpui/src/control/client.rs` and `control/client/*.rs` |
| GUI capability | `crates/sheprd-gpui/src/window/*`, `sidebar/*`, `menu/devices/host_menu/*`, `menu/devices/host_menu/tags.rs`, `menu/devices/host_menu/forwards.rs`, `pane_menu/processes.rs`, `caffeine.rs`, `window/status_bar.rs`, `settings_window/*`, `browser/view.rs` |
| Contracts under test | `control/client/tests/{wire.rs,wire.txt,family_help.rs,family_help.txt,documented_commands.rs}`, `control/server/**/tests*` |
| Docs | `docs/cli.md`, `skills/spaceport/SKILL.md` (embedded by `agent_skill.rs`, so the repo file *is* the installed skill) |

How the five checks were applied:

- **(a) docs ↔ handler**: does `control/server` do what `docs/cli.md` says?
- **(b) GUI parity**: does a window, menu, or palette entry offer the same
  capability, or does the doc say the command is CLI-only?
- **(c) `--json`**: is the document bounded, and is its shape documented?
- **(d) targeting**: does the command refuse another session's window or a
  stale one, and is there a test?
- **(e) destructive/outward**: does it act where the app would confirm?

## Matrix

29 families, in `spaceport --help` order. Each cell names the evidence.

| Command | (a) docs↔handler | (b) GUI parity | (c) `--json`/bounds | (d) targeting | (e) dialog | Verdict |
| --- | --- | --- | --- | --- | --- | --- |
| `status` | MATCH: `Status` carries version, protocol, pid, windows, Core catalog state/epoch/count; client prints all | app-wide report of the app itself; no window control to compare | bounded as every response; shape = `app::Status` | app-wide, `--window` existence-checked in `serve` | n/a | MATCH |
| `skill` | MATCH: prints `agent_skill::text`, the same text `integrations install-skill` writes | the app installs/refreshes the same file | text only, documented as such | no app | n/a | MATCH |
| `app focus\|hide\|quit` | MATCH: focus activates, hide hides, quit answers first then quits after 200 ms | palette Quit, window chrome | `{"message": …}` | `focus` → `window_for` refuses; `hide`/`quit` app-wide (now documented) | quit is not destructive: Herdr and terminals keep running | MATCH |
| `window [list]` | MATCH: IDs, size, selected legacy workspace or Core group, staleness mark, hosts | window chrome/menus | `Vec<WindowInfo>` | app-wide listing, existence-checked | n/a | MATCH |
| `window new` | MATCH: exact Core group or legacy target; stale group refused (`app/tests/core_window_new.rs`) | New Window command | `{"message"}` | resolves `--window`, else caller daemon, else frontmost (`new_window_target`) | opens a window; nothing destructive | MATCH |
| `window focus ID` | MATCH | clicking, palette | `{"message"}` | the ID *is* the target (`window_for`) | n/a | MATCH |
| `window fullscreen` | MATCH: `on/off/toggle`, `show/hide` aliases | F11 / palette | `{"message"}` | `in_window` | n/a | MATCH |
| `window size` | MATCH: 200-16384 × 150-16384, refuse/print, no move | **none**: the GUI only drags edges | `{"width","height"}` | `in_window` | n/a | UNDOCUMENTED → fixed (CLI-only now stated) |
| `window detach\|reconnect` | MATCH | palette Detach/Reconnect | `{"message"}` | `in_free_window` (exit 6 while a dialog is open) | reversible | MATCH |
| `ui sidebar\|statusbar` | MATCH | toggles, palette | `{"message"}` | `in_window` | n/a | MATCH |
| `ui font-size` | MATCH: 8-48 points, `+/-/reset`, `larger/smaller` | palette increase/decrease/reset | `{"points"}` | `in_window` | n/a | MATCH |
| `ui panel` | MATCH: three exit-8 refusals (Core off for `automations`, activity bar off, section off); `workspaces` expands the sidebar (`sidebar/activity.rs`) | activity bar | `{"message"}` | `in_window` | n/a | MATCH |
| `tab [list]` | MATCH: Devices + Core terminals; blank/retired tabs omitted (`kind()` returns `None`) | app tabs | `Vec<TabInfo>` (`tests/tab_help.rs`) | app-wide listing, existence-checked | n/a | MATCH |
| `tab focus\|close ID` | MATCH: switches to the tab's workspace, then shows it | clicking a tab, close button | `{"message"}` | refuses a `--window` that does not show the tab (`server/tabs.rs`, `tabs/tests/windows.rs`) | close drops the tab, not the Core terminal | MATCH |
| `devices [open]` | MATCH | Devices command | `{"id","workspace"}` or `{"message"}` | `arrive::devices` → `WorkspaceNotFound` when no window shows the workspace | n/a | MATCH |
| `chat [status]/list` | MATCH: agent, status, attached, shown, permissions, queued, model, context, `input_mode`, `session_id`; exit 4 when none | native chat views | `Vec<ChatInfo>`; fields documented in `docs/rich-chat.md` | `in_window` | n/a | MATCH |
| `chat show\|hide\|toggle` | MATCH: switches presentation only | chat view controls | `{"pane_id","shown"}` | `in_window`; a pane of another session is not found | n/a | MATCH |
| `chat new AGENT` | MATCH: built-in agents, exit 8 for ACP-off/missing program/missing agent, exit 1 for an unreachable host | workspace menu, palette | `{"agent","started"}` | `in_window` | starts a program; the menu does the same | MATCH |
| `notes [list]` | MATCH: most recent first | sidebar notes | `{"notes","omitted"}`; bounded by `LIST_BUDGET` (256 KiB − 4 KiB) | app-wide listing, existence-checked | n/a | UNDOCUMENTED (minor): `omitted` is not in the prose |
| `notes get\|set\|clear` | MATCH: one line, 500 chars, 16 KiB refused, exit 4 when none (`worktree_notes::clean`, `MAX_CHARS`) | note dialog | `{"note"}` | `window_for` + workspace lookup (`work/tests/identity.rs`) | set/clear are reversible | MATCH |
| `worktree [status]/mute/unmute` | MATCH: workspace mute plus host mute | sidebar menus | `MuteInfo` | `in_window` + `find` (`work/tests/muting.rs`) | n/a | MATCH |
| `host [list]` | MATCH: tags, mute, selected, status | sidebar, host menu | `Vec<HostInfo>` | `in_window` | n/a | MATCH |
| `host select ID` | MATCH; refuses a disabled host with its own words | sidebar click | `{"message"}` | `in_free_window`; same-named workspace of another daemon refused (`hosts/tests/strict.rs`) | n/a | MATCH |
| `host mute\|unmute` | MATCH: works while the host is offline | host menu | `{"message"}` | `in_window` | n/a | MATCH |
| `host tag ID [TAG…]` | MATCH: replaces tags, no tags clears | host menu "Move to"/"Also tagged" (`menu/devices/host_menu/tags.rs`) | `{"message"}` | `in_window` | n/a | UNDOCUMENTED → bounds fixed; **answer does not echo the tags kept** (see below) |
| `ports [list]` | MATCH: listening + forwards + `note` why empty | status bar ports | `Ports` | `in_window` + `Place` (`hosts/tests/ports.rs`, `strict.rs`) | n/a | MATCH |
| `ports forward\|unforward` | MATCH: saved SSH host only; local/WSL refused with the reason | host menu forwards (`forwards.rs` starts and stops) | `{"message"}` | `in_window` + `Place` | reversible | MATCH |
| `ports open PORT` | MATCH: system handler, forwarding a remote port first (protocol comment said "browser tab" — stale) | clicking a port | `{"message"}` | `window_for` + `Place` | hands a URL to the system handler, as clicking does | UNDOCUMENTED (comment only) → fixed |
| `processes [--pane]` | MATCH: no app needed | pane menu processes view | bounded rows | n/a | n/a | MATCH |
| `usage` | MATCH: selected host, `note` while there is no feed, exit 4 when empty | activity panel/status bar | `{"host","usage","note"}` | `in_window` | n/a | MATCH |
| `config path\|get\|set\|unset\|check` | MATCH: effective value with its source, refuses a value that would not load, comments kept | settings pages edit the same files | `config get` prints `{value, source}` | local: nothing to target | writes only `config.local.toml`; the settings pages write the same file | MATCH |
| `config reload` | MATCH | palette Reload Config | `{"message"}` | `in_free_window` | n/a | MATCH |
| `theme [list]/set` | MATCH: `light:…,dark:…`, saved without the app | Themes page | `{"current","themes"}` | `set` → `window_for`; `list` local | n/a | MATCH |
| `font [list]/set/reset` | MATCH: installed families, saved for every face | Settings fonts | `Vec<String>` | **app-wide:** no window resolution, so `--window` is only existence-checked | writes the settings file | UNDOCUMENTED → fixed |
| `language [status]/list/set` | MATCH: only `status` asks the app | Settings > General | `LanguageInfo` | **app-wide** (the locale is the app's) | n/a | UNDOCUMENTED → fixed |
| `keybinds [list]/check` | MATCH: `check` exits 1 on an unreachable binding | Keybinds window | `{"bindings","custom"}` | `in_window` | n/a | MATCH |
| `caffeine [status]/on/off/toggle` | MATCH, but `status` can answer `until_agents_finish`, which no doc named | **the cup alone sets that mode** | `{"state","seconds"}` | **app-wide** | n/a | UNDOCUMENTED → fixed (state, `--for` forms, app-wide) |
| `sound [list]/play` | MATCH | Settings sound preview | `{"name","plays"}` / `{"message"}` | `play` → `in_window`; `list` local | n/a | MATCH |
| `bell` | MATCH: rings, plus attention when the window is in the background | **none**: only a program's bell rings it | `{"message"}` | `in_window` | rings the bell, as a terminal bell does | UNDOCUMENTED → fixed |
| `events` (`--after`, `--kind`, `--follow`) | MATCH: 7 kinds, `INSTANCE:SEQ`, a cursor from an earlier run replays what was kept | **none**: there is no event log to show | bounded: `KEEP` 512, `BATCH` 256; `--follow` writes JSON lines with `cursor` (documented) | app-wide; existence-checked | n/a | MATCH |
| `wait window\|online\|tab-closed` | MATCH: 60 s default, 600 s cap, exit 4 | **none**: for scripts | `{"met"}` / `{"message"}` | **GAP: `wait online` swallowed `window_for`'s refusal** and answered "not met" (exit 4) after watching another session's window | n/a | **GAP → fixed + regression test** |
| `run COMMAND` | MATCH: unknown name exit 5 before asking the app; Core-gated commands exit 8; exit 6 while a dialog is open | the palette runs the same functions | `{"message"}` | `in_free_window` | no catalog command deletes, kills, migrates, or provisions, and the palette asks for no confirmation either; `check-for-updates` opens Sparkle's own UI | MATCH |
| `commands` | MATCH: names + English labels | the palette is the GUI list | `[{name,label}]` | local | n/a | MATCH |
| `logs [--lines N] [--path]` | MATCH: 100 default, ≤5000, exit 4 before the first log, no app needed | log window tails it | bounded rows | local | n/a | MATCH |
| `diagnostics bundle PATH` | MATCH: build info, settings check, overrides, logs, 3 newest crash reports, redacted, never overwrites | **none** | writes a new file | local | writes a new file only | UNDOCUMENTED → fixed |
| `integrations [list]` | MATCH: selected host's offers, with status | Settings integrations | `Vec<IntegrationInfo>` | `window_for` (`settings/services.rs`) | n/a | MATCH |
| `integrations install-skill` | MATCH: writes only files carrying the managed-skill marker | the app offers it once and keeps it current | `{"paths"}` | local | writes under `$HOME` with the marker | MATCH |

Cross-cutting rows:

| Surface | Finding | Verdict |
| --- | --- | --- |
| `spaceport --help`/`COMMAND --help` shared text | said `--json` prints "one JSON document" although `events --follow` streams, and implied every command refuses another session | UNDOCUMENTED/stale → fixed |
| `--json` shapes | replies are the protocol types in `crates/sheprd-control-protocol/src/protocol`; nothing said so | UNDOCUMENTED → fixed with one sentence |
| `--json`/`--window` on every command | both are parsed before any family parser (`cli/control.rs::parse`), so no command lacks either; `--help`/`skill` print text by design | MATCH |
| Request/response bounds | 256 KiB, 600 s wait cap, `MAX_WATCHES` 16, socket `QUEUE`/`CONNECTIONS` | MATCH (unchanged) |
| Per-command wire pinning | `control/client/tests/wire.txt` records 177 command lines' requests, target window, and `--json` flag for the nine single-operation families | MATCH |
| Structural doc tests | `documented_commands.rs` pins family names, protocol 37, and the skill's command lines; `family_help.rs` pins the whole `--help` text | MATCH |
| Managed skill (`skills/spaceport/SKILL.md`) | every backticked `spaceport …` line parses; its table matches real commands (notes, worktree, host mute, `ui panel automations`, `ports open`, `processes`, `chat new`, `config get/set`, `wait tab-closed`, `events --follow`, `caffeine --for`, `run check-for-updates`, `status`, `window`, `tab`) | MATCH |

Counts: 22 MATCH, 6 UNDOCUMENTED, 1 GAP, before the fixes below.

## Fixes made

1. **GAP (real bug): `wait online` acted on another session's window.**
   `control/server/events.rs` evaluated `Condition::Online` by asking
   `window_for`, and treated its refusal as "not yet": a caller whose workspace
   no window shows waited out the whole timeout (default 60 s) and exited 4,
   against the contract that such a caller exits 5 rather than be served by
   another session's window. `serve` now resolves the window once, before the
   request parks, and answers with the refusal itself. New
   `control/server/events/tests/online_target.rs` (`#[gpui::test]`, layout
   fixture) asserts `WorkspaceNotFound` for an unshown workspace and success
   for the caller's own workspace, or none. `docs/cli.md`'s `wait` row says so.
2. **`--help` shared text** (`control/client/help.rs::COMMON`): one JSON
   document "(`events --follow` writes one object per line instead)", and a
   command about the whole app "only checks that the window is there".
   `family_help.txt` regenerated.
3. **`docs/cli.md` — `--window` bullet**: names the app-wide commands
   (`status`, `window list`, `tab list`, `events`, `notes list`, `caffeine`,
   `font`, `language`, `app hide`, `app quit`) that change nothing window by
   window but still refuse an ID no window has. This closes the
   accept-and-ignore gap without changing behaviour.
4. **`docs/cli.md` — bounds and values that were undocumented**: `host tag`
   (8 tags of 24 characters, as the sidebar keeps them), `caffeine` (`off`,
   `pending`, `on`, `until_agents_finish`; `--for` `90s`/`30m`/`2h`, 1 s to
   24 h), `--json` (the reply is the protocol's type; app answers are bounded
   to 256 KiB), and a paragraph in "What stays in the app" naming the commands
   with no window, menu, or palette equivalent (`window size WIDTHxHEIGHT`,
   `bell`, `config path`, `config get`, `status`, `events`, `wait`).
5. **Protocol comments that had drifted**: `Reply::Caffeine`'s `state` now
   lists `until_agents_finish` (the cup's agents mode, which no request sets);
   `HostOp::OpenPort` says the system URL handler rather than "a browser tab",
   which the browser-tab removal left behind.
6. **New `scripts/help-snapshot.py`**: rebuilds `family_help.txt` from
   `FAMILIES`, the `cli/control/*.rs` usage constants, and `COMMON`, so the
   byte-exact snapshot can be recorded or checked without a full `spaceport`
   build (`--check`/`--write`). Before editing anything it reproduced the
   committed snapshot byte for byte, and after fix 2 the only line it changed
   was the intended one.

Not done, deliberately: no new command (the contract says a command lands with
its feature), no change to the vendored bincode or protocol version, and no
behaviour change for the app-wide commands (their GUI counterparts are
app-wide too).

## Needs an owner decision

1. **`--json` shapes are pinned by types, not by prose.** Every document is a
   protocol reply type, bounded by the socket, and `wire.txt` pins what each
   command *sends*; nothing pins what each command *prints* in one place. A
   reviewer wanting an exhaustive "this command prints exactly this" reference
   needs a decision on whether to generate it from the types or keep the
   current type-is-the-contract rule.
2. **`host tag` answers "Tagged X" without saying what was kept.** The
   sidebar's rules (8 tags, 24 characters, whitespace collapsed) do the
   bounding, so a script that passes nine tags or a 40-character tag is told it
   succeeded while part was dropped. Echoing the kept tags is a one-line
   message change, but it changes output a script may parse, so it is the
   owner's call.
3. **`caffeine until_agents_finish` is GUI-only.** The status bar cup sets it;
   no command can. Documented as a readable state. Adding `--until-agents` (or
   a verb) is a feature, not a fix, and the CLI contract says features bring
   their commands.
4. **`font` and `language` never refuse another session.** They change app-wide
   settings, so they only existence-check `--window`; that is now documented
   rather than changed. If the intent is that *every* command must exit 5 for
   an unshown session, these two would need `window_for` and a behaviour change
   in the same commit as their tests.
5. **`notes --json` carries `omitted`** (older notes that did not fit one
   answer) with no prose. Whether to document reply fields one by one is the
   same decision as item 1.

## Verification

Run in this worktree on 2026-11-20. The two cargo gates were blocked at first:
this volume held 1.3 GiB free (the disk guard stops below 3 GiB) and a sibling
lane's `target/` was 9.4 G. The supervisor removed the finished lanes'
worktrees, leaving 12 GiB free, and both then ran here.

Gates the task named:

- `cargo test -p sheprd-control-protocol --locked` — **ok, 71 passed, 0
  failed** (plus 0 doc-tests).
- `cargo test -p sheprd-gpui --bin spaceport --locked -- control` — **ok, 229
  passed, 0 failed** (3493 filtered out by the substring filter). The new test
  ran as
  `control::server::events::tests::online_target::a_wait_for_a_host_of_another_session_is_refused_at_once`.
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest
  scripts/release/tests/test_control_product_copy.py` — **ok, 1 test**.
- `cargo fmt --all -- --check` — **exit 0**.
- `just check-sorted` — **exit 0** (10 self-tests, all regions sorted).
- `just check-commits` — **exit 0** for all six commits of this lane.

Also run:

- `cargo test -p sheprd-gpui --bin spaceport --locked -- control::client::tests`
  — **ok, 14 passed**, including
  `family_help::help_text_matches_the_recorded_text` (so the regenerated
  `family_help.txt` is byte-exact against the binary's own text) and the three
  `documented_commands` assertions (29 families, protocol 37, the skill's
  command lines).
- `just check-file-size` — **exit 0**.
- `python3 scripts/help-snapshot.py --check` — **exit 0**.
- `python3 -m py_compile scripts/help-snapshot.py` — **exit 0**.
- Doc contract checked by script as well, mirroring `documented_commands.rs`:
  29 of 29 families named in `docs/cli.md`, and the doc's protocol version (37)
  equals `PROTOCOL_VERSION`.
- No native/UI check applies: no rendering, input, or startup path changed.

Two first-run compile errors in the new test (the fixture helper returns a
window context beside the view; `rustfmt` then split two lines) were fixed in
`style(control): Format the new wait-online test` before the gates above.

## Residual risk

- `wait online` now exits 5 where it used to wait out the timeout. A script
  that ran `spaceport wait online` before any window existed now gets exit 5
  instead of a silent 60 s wait; `wait window` is the condition for "an app
  with a window", and the doc says so.
- `family_help.txt` is regenerated by `scripts/help-snapshot.py` rather than by
  running the binary; the script reproduced the old snapshot byte for byte
  before the change and the byte-exact test now passes against the binary, so
  the two agree today. The script parses the Rust sources, so a future change
  to how those constants are written needs it updated with them.
