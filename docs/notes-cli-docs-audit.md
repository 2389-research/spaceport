# CLI docs audit: `docs/cli.md` against the command surface

Read-only audit. Nothing in the repository was edited. This note feeds a
follow-up worker; every finding names both sides.

## What was audited

| Side | Path |
| --- | --- |
| Docs under audit | `docs/cli.md` (288 lines), `skills/spaceport/SKILL.md` (43 lines) |
| Command catalog / parsing | `crates/sheprd-gpui/src/cli.rs`, `crates/sheprd-gpui/src/cli/control.rs`, `crates/sheprd-gpui/src/cli/control/{app,chat,hosts,plain,settings,tabs,work}.rs` |
| Wire surface | `crates/sheprd-control-protocol/src/protocol.rs`, `protocol/{app,chat,events,hosts,settings,tabs,work}.rs`, `socket.rs`, `socket/{unix,pipe}.rs`, `error.rs` |
| Client behavior, exit codes, printing | `crates/sheprd-gpui/src/control.rs`, `control/client.rs`, `control/client/{app,chat,events,hosts,local,settings,tabs,work}.rs`, `control/client/local/{diagnostics,logs,processes}.rs`, `control/client/settings/{config,language}.rs` |
| Server behavior | `crates/sheprd-gpui/src/control/server.rs`, `control/server/{app,chat,events,hosts,links,settings,tabs,work}.rs` |
| App command catalog | `crates/sheprd-commands/src/catalog.rs` (`controls` in the app) |
| Recorded help text | `crates/sheprd-gpui/src/control/client/tests/family_help.txt` (snapshot-guarded by `family_help.rs:61`) |

Reproduce the lists (all read-only):

```sh
rg -n '^        name: ' crates/sheprd-gpui/src/cli/control.rs          # the 29 families
sed -n '3,31p' crates/sheprd-gpui/src/control/client/tests/family_help.txt  # what --help lists
rg -n 'PROTOCOL_VERSION: u32' crates/sheprd-control-protocol/src/protocol.rs
rg -n 'pub const (REFUSED|USAGE|NOT_RUNNING|NOTHING|NOT_FOUND|BUSY|TIMEOUT|UNSUPPORTED)' \
   crates/sheprd-gpui/src/control/client.rs                            # exit table
```

## Verdict

- Command surface: **29 families** exist (`cli/control.rs:113-281`,
  mirrored in `family_help.txt:3-31`). `docs/cli.md` names **27**; `skill` is
  absent entirely and `wait` appears only inside an example block.
- Exit-code table: values 0-8 all exist and are wired correctly
  (`control/client.rs:26-40`, `232-248`), but the row for **4** is both
  stale and incomplete, and one command documents the wrong code.
- Bounded-request claims (256 KiB, 600 s, "a few at a time") are true.
- No test ties `docs/cli.md` or `skills/spaceport/SKILL.md` to the source:
  `rg -n 'cli\.md|docs/cli' crates/ --include=*.rs` returns nothing. Drift in
  these two files is therefore unchecked by CI today.

## Findings

Drift types: `missing-in-docs`, `stale-in-docs`, `wrong-claim`,
`missing-in-source`. Priority: P1 (user-visible wrong behavior/exit status),
P2 (misleading or incomplete), P3 (cosmetic/completeness).

| ID | Drift | Docs side | Source side | What is wrong | Pri |
| --- | --- | --- | --- | --- | --- |
| F01 | missing-in-docs | `docs/cli.md` (whole file: the word `skill` never appears as a command) | `crates/sheprd-gpui/src/cli/control.rs:119` (`Family { name: "skill" }`), `cli/control/plain.rs:12-17`, `control/client.rs:341-347` | `spaceport skill` prints the managed agent skill and is listed by `spaceport --help` (`family_help.txt:4`) and implemented, but the CLI doc never mentions it. `integrations install-skill` is documented (`docs/cli.md:152`) while the command that prints the same text is not. | P1 |
| F02 | missing-in-docs | `docs/cli.md:173-177` (examples only, no row) | `cli/control.rs:251` (`Family { name: "wait" }`), `cli/control/work.rs:58-65`, `protocol/events.rs:88-96` (`Condition::{Window, TabClosed, Online}`) | `wait` has no table row or description; `wait online` (the selected host comes online) is named nowhere in the docs, though `WAIT_USAGE` (`cli/control/work.rs:58`) documents all three conditions. The docs also never state the default timeout (60 s, `cli/control/work.rs:62`). | P1 |
| F03 | stale-in-docs | `docs/cli.md:221` ("currently 36") | `crates/sheprd-control-protocol/src/protocol.rs:42` (`PROTOCOL_VERSION: u32 = 37`, with the 37 entry documented at `protocol.rs:41`) | Documented control-protocol version is one release behind; `status` prints 37 and the client refuses a mismatch (`control/client/app.rs:89-95`). | P1 |
| F04 | stale-in-docs + wrong-claim | `docs/cli.md:50` ("Nothing yet: no notes, no event, no pull request, condition not met") | `control/client/{hosts.rs:36,49, chat.rs:45, work.rs:49,62, events.rs:91,170, logs.rs:20, settings/config.rs:38}`; no PR surface at all (`protocol.rs:23`, "20: removed GitHub and PR control routes"; `protocol.rs:32`, "29: removed native PR, issue, and task activity panel control targets") | The only exit-4 paths are: no notes, no events, `wait` unmet, ports/usage empty, no chat panes, no log yet, `config unset` on an unset key. "No pull request" is left over from removed PR control routes, and ports/usage/chats/log/key are missing from the row. Same wording problem in the printed help: `control/client/help.rs:10` says "4 nothing yet (no event)". | P1 |
| F05 | wrong-claim | `docs/cli.md:110` ("Exits 8 when `sheprd-acp` is missing here, the host cannot be reached, or the agent is not installed on it"); same claim in `cli/control/chat.rs:4` (`CHAT_USAGE`) | `control/server/chat.rs:85-99` (`start_code`), `chat/start.rs:49-53` (`NotConnected`), `control/client.rs:244` (`Failed` → exit 1) | An unreachable/disconnected host yields `chat::Error::NotConnected`, mapped to `ErrorCode::Failed`, i.e. **exit 1**, not 8. Only `Disabled`, `AgentMissing`, `ProgramMissing`, `NotLocal`, `CoreTerminalRequired` are exit 8 (`server/chat.rs:88-92`). Both the doc and the `--help` text carry the wrong code. | P1 |
| F06 | missing-in-source | `docs/cli.md:166-168` ("notes kept for a pane; worktree notes changing") | `protocol/events.rs:39-49` (`EventKind`: window_opened/closed/focused, tab_opened/closed, `Note` = "A worktree note changed", host); `control/server/events.rs:1-5` (events are derived from windows, app tabs, worktree notes, host status) | There is no pane-note event kind and no pane-note source; the sentence describes an event that does not exist, and lists the worktree note twice. | P1 |
| F07 | wrong-claim | `docs/cli.md:35` ("`--json`: print one JSON document"); `skills/spaceport/SKILL.md:37-38` ("Add `--json` to any command") | `control/client/events.rs:97-119` (`events --follow` writes one JSON line per event, each with a `cursor` field) | `events --follow --json` emits JSON Lines, never a single document. Also `--json` is ignored by `help` and `skill` (`control/client.rs:337-347` return before printing JSON), and must follow the family word: `spaceport --json status` is a usage error (`cli/control.rs:290-296` tests `args.first()`), which the docs' "Every command accepts" hides. | P1 |
| F08 | wrong-claim (help text) | `cli/control/plain.rs:75-76` (`COMMANDS_USAGE`: "The commands `run` accepts, **with their shortcuts**") | `control/client/local.rs:13-24` (prints `name` + English label only; no shortcut data is in `CommandInfo`, `crates/sheprd-commands/src/lib.rs`) | `spaceport commands --help` promises shortcuts the command never prints. `docs/cli.md:83` says only "The names `run` accepts", which is accurate. | P2 |
| F09 | wrong-claim | `docs/cli.md:33-42` ("Every command accepts … `--window ID`: act in that window") | `control/client/local.rs:13-24`, `control/client/settings/config.rs:25-39`, `control/client/settings/language.rs:21-28`, `control/client/settings.rs:51-77` (`theme`), `agent_skill` path in `settings.rs:186-205` | Commands answered locally (`commands`, `processes`, `logs`, `diagnostics`, `config path/get/set/unset`, `theme list/set`, `language list/set`, `skill`, `integrations install-skill`) parse `--window` and then ignore it: there is no `Context` and no server-side validation, so a wrong ID is not refused. | P2 |
| F10 | wrong-claim | `docs/cli.md:218-220` ("A request is `{"method", "caller", "op"}`") | `protocol.rs:51-73` (`Request`: `Status(Caller)` at `:53`; every other family is `Call<T>` at `:89-92`, whose `caller` is a nested object, `:143-159`) | `status` is not `{method, caller, op}`; it is `{"method":"status", "pane_id":…, "workspace_id":…, "daemon_socket":…, "window":…}` (see the wire fixture `protocol.rs:259-270`). The stated shape holds only for the other eleven families. | P2 |
| F11 | stale-in-docs (skill) | `skills/spaceport/SKILL.md:38-39` ("Exit statuses are the ones in the table above") | `docs/cli.md:44-54` is the only exit table; the skill file contains no table (`skills/spaceport/SKILL.md:20-35` is a "To / Run" table with no exit codes) | Dangling cross-reference. The skill is installed standalone into agent directories (`agent_skill.rs:14`, `105-130`), so "the table above" points at nothing an agent can see. | P1 |
| F12 | stale-in-docs (skill) | `skills/spaceport/SKILL.md:26` ("Open a dev server **in a tab**") | `control/server/hosts/ports.rs:249-296` → `browser/view.rs:192-205` (`open_workspace_page` calls `cx.open_url`, i.e. the system handler); `docs/cli.md:133` states this correctly | The browser tab family is gone; `ports open` hands `http://localhost:PORT/` to the system handler. The skill still promises an in-app tab. | P2 |
| F13 | stale-in-docs (skill) | `skills/spaceport/SKILL.md:3` (description names "native chat, **files**, notes, settings, and window state") | `cli/control.rs:113-281` (no `files` family; 29 families listed above), `family_help.txt:3-31` | No `spaceport files` command exists. The description is all an agent sees before loading the skill (`AGENTS.md`, Command Line), so it advertises a capability with no command. | P2 |
| F14 | missing-in-docs | `docs/cli.md:143` ("`config check` \| Report invalid settings, exit 1 if any") | `control/client/settings/config.rs:81-123` (also sends `SettingsOp::CheckConfig` and reports configured fonts the running app lacks; prints "Fonts were not checked" when no app), `cli/control/settings.rs:79` (usage says so) | The docs omit that `check` asks the running app about missing fonts, that fonts are skipped without a window, and that a font problem alone also exits 1. | P2 |
| F15 | missing-in-docs | `docs/cli.md:145-146` (`theme [list]`, `theme set NAME`; `font [list]`, `font set NAME`, `font reset`) | `cli/control/settings.rs:102` (`light:NAME,dark:NAME`), `control/client/settings.rs:51-77` (theme list/set work without the app), `cli/control/settings.rs:103` + `control/client/settings.rs:79-87` (font needs the app) | Neither surface states which half runs without the app — the doc uses a "(no app needed)" marker everywhere else (`docs/cli.md:141,142,152,153,154`) — and `theme set light:NAME,dark:NAME` is undocumented. | P2 |
| F16 | missing-in-docs | `docs/cli.md:153` (`logs [--lines N] [--path]`) | `cli/control/plain.rs:82-90` (default 100, `1..=TAIL_RECORDS`), `crates/sheprd-diagnostics/src/lib.rs:52` (`TAIL_RECORDS = 5000`) | No bound or default given, though `AGENTS.md` requires bounded requests and the `--help` text states both. `logs` also exits 4 when there is no log yet (`control/client/local/logs.rs:18-21`) — not stated. | P2 |
| F17 | missing-in-docs | `docs/cli.md:105-106` (`notes set TEXT`) | `cli/control/work.rs:12-19` (16 KiB CLI limit), `cli/control/work.rs:35` (one line, at most 500 characters; `list`/`get` exit 4 when none) | Neither the 16 KiB CLI refusal nor the 500-character note rule appears in the doc; both are in `--help`. | P3 |
| F18 | missing-in-docs | `docs/cli.md:79` (`ui font-size +\|-\|reset\|POINTS`) | `cli/control/app.rs:90-96` (`POINTS` must be 8-48, error "POINTS (8-48)") | Accepted range undocumented. | P3 |
| F19 | missing-in-docs | `docs/cli.md:76` (`window size [WIDTHxHEIGHT]`) | `cli/control/app.rs:51-58` (width 200-16384, height 150-16384) | Accepted range undocumented. | P3 |
| F20 | missing-in-docs | `docs/cli.md:78-80` (`ui sidebar\|statusbar`, `ui panel …`) | `control/server/app.rs:303-326` (`ui panel` returns `Unsupported`, exit 8, when Core is off for `automations`, when `[sidebar] activity_bar = false`, or when that panel is turned off in settings) | The doc lists the panel names but not the refusals; a script gets exit 8 with a settings-dependent message. The general Core sentence (`docs/cli.md:62`) covers only the Core case. | P2 |
| F21 | missing-in-docs | `docs/cli.md:78,99,109` | `cli/control/app.rs:74` (`status-bar` alias), `cli/control.rs:305-313` (`show`/`hide` as switch aliases), `cli/control/chat.rs:11` (`chat list`), `cli/control/tabs.rs:24-30` (`devices [open]`) | Four accepted aliases are undocumented. Harmless, but a doc reader cannot tell whether `devices open` is valid. | P3 |
| F22 | wrong-claim (minor) | `docs/cli.md:142` ("Read or change `spaceport/config.local.toml`") | `control/client/settings/config.rs:64-79` (`Config::effective_value`: override else default, with `source: "local"\|"default"`), `cli/control/settings.rs:79` ("your override, else the default") | `config get` reads the effective value, not the local file, so a key set only by the managed defaults still answers. | P3 |
| F23 | missing-in-docs | `docs/cli.md` (no mention) | `crates/sheprd-gpui/src/app.rs:351-354` (`--build-info`), `:355-372` (`--help` header names `--socket` and lists the families), `cli.rs:148-186` | Two top-level flags appear in `--help` but nowhere in `docs/cli.md`, which otherwise introduces itself as the CLI doc. (`--socket` is only described in `docs/windows-local-access.md:94`.) | P3 |
| F24 | wrong-claim (minor) | `docs/cli.md:51` (exit 5 = "The window, workspace, tab, host, or file is not there") | `control/client/app.rs:152-169` (an unknown `run` name is refused locally with `ErrorCode::NotFound`) | `spaceport run nosuchcommand` exits 5 but is not a window/workspace/tab/host/file. | P3 |
| F25 | observation (source) | — | `crates/sheprd-commands/src/catalog.rs:230,232` (`"install_browser_skill"`, `"check-for-updates"`), `window/commands.rs:733-748` (`install_browser_skill` installs the agent skill) | The catalog keeps two legacy names: `install_browser_skill` (now the agent-skill installer) and the only hyphenated command name. `docs/cli.md:81` uses `run check-for-updates` correctly, so this is a source-side naming leftover, not doc drift — noted so a follow-up does not "fix" the doc to match a wrong name. | P3 |

### Findings that need a decision, not just an edit

- **F04/F24**: the exit-code table is the shared contract (`AGENTS.md`,
  Command Line). Fixing it means choosing whether to enumerate every exit-4
  source or keep it a category. `control/client/help.rs:9-11` and
  `docs/cli.md:50` must then say the same thing, and
  `control/client/tests/family_help.txt` (a byte-exact snapshot) must be
  regenerated in the same change.
- **F05**: fix the doc *and* `cli/control/chat.rs:4`, or the two surfaces keep
  disagreeing.

## Keep-honest: claims verified true

These were checked against code and hold. A follow-up worker should not
"simplify" them.

| Claim | Evidence |
| --- | --- |
| Socket at `$XDG_STATE_HOME/spaceport/control.sock`, default `~/.local/state/spaceport/`, user-only | `docs/cli.md:14-17`; `sheprd-client/src/local_path.rs:76-96`, `protocol/socket/unix.rs:15-17,23-42` (dir 0700, socket 0600) |
| Every command returns 0-8 from one table; codes map as listed | `docs/cli.md:44-54`; `control/client.rs:26-40`, `232-248` (only row 4 is wrong, F04) |
| `--json` errors are `{"error": {"code", "message"}}` on stdout | `docs/cli.md:35-36`; `control/client.rs:284-303` |
| Default window = the one showing `HERDR_WORKSPACE_ID`; none → exit 5, never another session's window; outside Herdr the frontmost | `docs/cli.md:37-42`; `control/server.rs:119-186` (`window_for`), `control.rs:80-90` (`main_windows` sorts the active window first) |
| Requests and responses bounded to 256 KiB; a wait states its seconds, capped at 600 | `docs/cli.md:222-223`; `protocol.rs:45-47`, `socket.rs:275-283` (`fits`) and `socket/tests.rs:145-160` (`an_answer_over_the_bound_is_refused`) |
| "The app answers at most a few requests at a time" | `docs/cli.md:223-224`; `socket.rs:33-36` (`QUEUE = 4`, `CONNECTIONS = 8`), `control/server.rs:20` (`MAX_WATCHES = 16`) |
| Windows pipe name, `XDG_STATE_HOME` hash suffix, owner-only DACL, first instance, `PIPE_REJECT_REMOTE_CLIENTS`, identification-level open, foreign pipe → exit 1 | `docs/cli.md:228-244`; `protocol/socket/pipe.rs:55-102` (`O:{sid}D:P(A;;GA;;;{sid})`, `accept_remote(false)`), `183-222` (IDENTIFICATION, owner check, `PipeForeign`), `error.rs:18-19`, `control/client.rs:246` (`_ => REFUSED`) |
| A pipe has no send/receive timeout; reads peek; the out buffer holds a whole answer | `docs/cli.md:246-249`; `protocol/socket/pipe.rs:13-15,83-85,120-181` |
| Links: two shapes only, each parameter once, bounded lengths, no control characters, never runs a command; refused link 2, unknown device/notification 5 | `docs/cli.md:210-214`; `crates/sheprd-gpui/src/link.rs:11-32`, `control/server/links.rs:13-33`, `cli.rs:120-130` (`LinkArguments`) |
| Link request shape `{"method":"link","caller","op":{"url"}}` | `docs/cli.md:214`; `protocol.rs:68-72`, `control/client.rs:158-164` |
| `status` reports version, protocol, windows, hosts and the Core catalog's state/epoch/count; no machine lookup | `docs/cli.md:70`; `protocol/app.rs` (`Status`, `CoreCatalog`), `control/client/app.rs:55-104` |
| `window list` marks a stale Core group and prints hub/document/group/machine | `docs/cli.md:72,85-91`; `control/client/app.rs:26-42` |
| `window new` joins the selected Core group and refuses a stale selection | `docs/cli.md:73,88-90`; `cli/control/app.rs:19` (usage), `control/server/app/tests/core_window_new.rs` |
| `tab list` shows Devices and Core terminals only, omits retired kinds | `docs/cli.md:97`; `control/server/tabs.rs:41-49` (`kind`), `79-93` (`list` filters Core tabs), `tab_help.rs:4-8` |
| Core off: Core-only commands exit 8, saved Core tabs hidden from `tab list`, `config reload` keeps the session's Core mode, restart needed | `docs/cli.md:58-64`; `core_feature.rs:7-33`, `app.rs:476` (install only at startup), `control/server/tabs.rs:85`, `control/server/app.rs:304-309` |
| `herdr_core_enabled` needs a restart; `tab_labels.agent_title`, `focus_follows_mouse`, `sound.volume`, `chat.max_width` are real settings | `docs/cli.md:59,150,157-162,257-260`; `config/preferences.rs:145,165,169`, `config/notifications.rs:59-90`, `config/chat.rs:38,66` (320-4000, default 800) |
| `config set`/`unset` keep comments, refuse a value that would not load or an unknown key, write nothing then; no app needed | `docs/cli.md:142`; `config/edit.rs:113-160`, `control/client/settings/config.rs:27-39` |
| `ports open` hands `localhost:PORT` to the system handler and forwards a remote port first | `docs/cli.md:133`; `control/server/hosts/ports.rs:249-296,300-329`, `browser/view.rs:192-205` (`cx.open_url`) |
| `processes`, `logs`, `diagnostics bundle`, `config path`, `commands`, `install-skill` need no running app | `docs/cli.md:135,141,152,153,154`; `control/client/local.rs:1-24`, `control/client/local/{logs,diagnostics,processes}.rs`, `control/client/settings.rs:186-205` |
| `diagnostics bundle` holds build info, settings check, overrides, recent logs and the newest crash reports, redacted, never overwrites | `docs/cli.md:154`; `control/client/local/diagnostics.rs:22-118` (`CRASH_REPORTS = 3`, `LOG_RECORDS = 500`, `BundleExists`) |
| `chat new` agent list, Herdr 2 pane split, `core:` pane id, exit 8 when ACP chats are off | `docs/cli.md:110,255`; `cli/control/chat.rs:4`, `control/server/chat.rs:16-53,88-92`, `chat/core_pane.rs:32-34` |
| `chat status --json` reports `input_mode` and `session_id`; `blocked` means input is needed | `docs/cli.md:285-288`; `protocol/chat.rs:29-77` (`InputMode`, `ChatInfo`), `control/server/chat.rs:102-160` |
| `chat show/hide/toggle` switches presentation without starting or stopping the agent | `docs/cli.md:287-288`; `control/server/chat.rs:54-80` (`set_chat_raw`) |
| Remote `sheprd-acp` searched in `~/.herdr/bin`, `~/.cargo/bin`, `~/.local/bin`, then remote `PATH` | `docs/cli.md:267-268`; `chat.rs:205-236` (`REMOTE_PROGRAM_DIRS`) — the code also tries the app-bundle `Contents/MacOS` dirs and both `sheprd-acp`/`herdr-acp`, which the doc omits (harmless) |
| `sheprd-acp agents` lists installed agents | `docs/cli.md:110,275`; `crates/sheprd-acp/src/cli.rs:46,112,163` |
| `run check-for-updates` opens Sparkle's check in an installed macOS preview and refuses unsupported builds | `docs/cli.md:81`; `crates/sheprd-commands/src/catalog.rs:230`, `control/server/app.rs:369-380`, `updates::Error::Unsupported` → `ErrorCode::Unsupported` (exit 8), busy → exit 6 |
| `run` takes any `[keybindings]` name and refuses an unknown one | `docs/cli.md:82`; `control/client/app.rs:152-169` (exit 5, see F24) |
| `events` cursor is `INSTANCE:SEQ`, and a cursor from an earlier run yields everything the new run kept | `docs/cli.md:169-171`; `protocol/events.rs:128-175`, `control/client/events.rs:36-67`, `control/server/events.rs:60-66` |
| Event kinds and `--kind` spelling (`tab_closed`, `-` or `_`) | `docs/cli.md:174`; `protocol/events.rs:51-88` (`FromStr` maps `-` to `_`) |
| `wait window` also waits for the app to start | `docs/cli.md:176`; `control/client/events.rs:126-146` (`START_POLL` retry on `Unavailable`) |
| The removed `update` command is rejected before app startup | `docs/cli.md:187`; `crates/sheprd-gpui/tests/cli/removed_updater.rs:6-24` (exit 2, "Unknown option"), `cli/tests/removed_updater.rs:6-17` |
| No worktree-script control requests remain | `docs/cli.md:181-182`; `protocol.rs:35` ("32: removed worktree script control requests") |
| `spaceport COMMAND --help` prints per-family usage ending in the shared text | `docs/cli.md:28`; `control/client/help.rs:14-32`, `family_help.rs:32-69` |

## Prioritized fix list for the follow-up worker

Do these in order; each is one commit-sized change, and the docs-only ones
must keep `just check-file-size` and `just check-sorted` green (both files are
short, so this is a wording exercise, not a split).

1. **P1 F03** `docs/cli.md:221`: 36 → 37.
2. **P1 F01/F02** `docs/cli.md`: add `skill` and `wait` to the command tables
   (`skill` belongs in "Settings and services" next to
   `integrations install-skill`; `wait window|online|tab-closed ID
   [--timeout SECONDS]` belongs in "Events"), naming the default 60 s and the
   600 s cap.
3. **P1 F04** rewrite the exit-4 row in `docs/cli.md:50` and the same line in
   `control/client/help.rs:10`, then regenerate
   `control/client/tests/family_help.txt` (snapshot, `family_help.rs:61-69`).
   Drop "no pull request"; cover no notes / no event / wait unmet / no ports /
   no usage / no chat panes / no log yet / key not set.
4. **P1 F05** fix "the host cannot be reached" in `docs/cli.md:110` **and**
   `cli/control/chat.rs:4` so both say exit 1 for an unreachable host and keep
   exit 8 for missing `sheprd-acp`/agent/ACP-off.
5. **P1 F06** `docs/cli.md:166-168`: drop "notes kept for a pane" and say the
   note event is a worktree note change.
6. **P1 F07** `docs/cli.md:35-36` and `skills/spaceport/SKILL.md:37-38`: say
   one JSON document per answer and one JSON object per line for
   `events --follow`, and that `--json` follows the command word.
7. **P1 F11** `skills/spaceport/SKILL.md:38`: replace "the table above" with
   the codes themselves or a pointer to `spaceport COMMAND --help`.
8. **P2 F08** fix `COMMANDS_USAGE` (`cli/control/plain.rs:75-76`) to stop
   promising shortcuts, or add shortcuts to `commands` output (the doc at
   `docs/cli.md:83` is already right; the help text is wrong).
9. **P2 F14/F15/F16/F20** fill in the missing behavior notes in the existing
   table rows (`config check` fonts, `theme`/`font` app requirement and
   `light:/dark:`, `logs` bounds and exit 4, `ui panel` refusals).
10. **P2 F09/F10** tighten the two protocol/option paragraphs.
11. **P2 F12/F13** fix the two skill prose claims ("in a tab", "files").
12. **P3** F17-F24: ranges, aliases, `config get` wording, `--build-info`/
    `--socket`, `run` unknown-name exit.
13. **Structural** (recommended, and the reason this drift keeps happening):
    add a test beside `family_help.rs` that asserts (a) every
    `cli::control::FAMILIES` name appears in `docs/cli.md`, (b) the documented
    protocol version equals `PROTOCOL_VERSION`, and (c) every
    `spaceport …` command line in `skills/spaceport/SKILL.md` parses through
    `cli::control::parse`. That converts F01/F02/F03/F21 from review findings
    into CI failures.

## Appendix A: command surface as documented (`docs/cli.md`)

| Line | Documented form |
| --- | --- |
| 27-28 | `spaceport --help`, `spaceport COMMAND --help` |
| 35-42 | `--json`, `--window ID` |
| 70-83 | `status`; `app focus\|hide\|quit`; `window [list]`; `window new`; `window focus ID`; `window fullscreen [on\|off\|toggle]`; `window size [WIDTHxHEIGHT]`; `window detach\|reconnect`; `ui sidebar\|statusbar [on\|off\|toggle]`; `ui font-size +\|-\|reset\|POINTS`; `ui panel workspaces\|overview\|automations\|usage`; `run check-for-updates`; `run COMMAND`; `commands` |
| 97-99 | `tab [list]`; `tab focus\|close ID`; `devices` |
| 105-110 | `notes [list]`; `notes get\|clear`, `notes set TEXT`; `worktree [status]`, `worktree mute\|unmute`; `chat [status]`; `chat show\|hide\|toggle [--pane ID]`; `chat new AGENT [--pane ID]` |
| 128-135 | `host [list]`, `host select ID`; `host mute\|unmute [HOST]`; `host tag ID [TAG...]`; `ports [list]`; `ports forward\|unforward PORT`; `ports open PORT`; `usage`; `processes [--pane ID]` |
| 141-154 | `config [path]`; `config get\|set\|unset KEY [VALUE]`; `config check`; `config reload`; `theme [list]`, `theme set NAME`; `font [list]`, `font set NAME`, `font reset`; `language [status]`, `language list`, `language set system\|TAG`; `keybinds [list]`, `keybinds check`; `caffeine [status]`, `caffeine on\|off\|toggle [--for 30m]`; `sound [list]`, `sound play [done\|request]`, `bell`; `integrations [list]`; `integrations install-skill`; `logs [--lines N] [--path]`; `diagnostics bundle PATH` |
| 166-177 | `events --follow --kind …`, `events --after INSTANCE:SEQ`; `wait tab-closed ID --timeout N`, `wait window` (examples only) |
| 198-214 | `spaceport URL` (`spaceport://device?id=ID`, `spaceport://notification?tag=TAG`) |

Missing from that surface, present in `FAMILIES`: `skill`, `wait` (as a
documented command), plus the aliases in F21.

## Appendix B: source of truth

Families, in `--help` order (`cli/control.rs` line, `family_help.txt` line):

| Family | `cli/control.rs` | ops / parser |
| --- | --- | --- |
| status | 113 | `Request::Status(Caller)`; `cli/control/plain.rs:8` |
| skill | 119 | `ControlCommand::Skill`; `plain.rs:15` |
| app | 125 | `AppOp::{Focus,Hide,Quit}` |
| window | 131 | `WindowOp::{List,New,Focus,Fullscreen,Size,Detach,Reconnect}` |
| ui | 137 | `UiOp::{Sidebar,StatusBar,FontSize,Panel}` |
| tab | 143 | `TabOp::{List,Focus,Close}` |
| devices | 149 | `TabOp::Devices` |
| chat | 155 | `ChatOp::{Status,New,View}` |
| notes | 161 | `WorkOp::{Notes,Note,SetNote}` |
| worktree | 167 | `WorkOp::{MuteStatus,Mute}` |
| host | 173 | `HostOp::{List,Select,Mute,Tag}` |
| ports | 179 | `HostOp::{Ports,Forward,Unforward,OpenPort}` |
| processes | 185 | `ControlCommand::Processes` (local) |
| usage | 191 | `HostOp::Usage` |
| config | 197 | `ConfigCommand::{Path,Get,Set,Unset,Reload,Check}` |
| theme | 203 | `ThemeCommand::{List,Set}` |
| font | 209 | `ThemeCommand::{List,Set(None)=reset}` |
| language | 215 | `LanguageCommand::{Status,List,Set}` |
| keybinds | 221 | `KeybindsCommand::{List,Check}` |
| caffeine | 227 | `CaffeineCommand::{Status,Set}` → `SettingsOp::Caffeine` |
| sound | 233 | `SoundCommand::{List,Play}` |
| bell | 239 | `SettingsOp::Bell` |
| events | 245 | `EventsOp::{Poll,Wait}` |
| wait | 251 | `ControlCommand::Wait{WaitFor}` → `EventsOp::Wait` |
| run | 257 | `CommandOp::Run` |
| commands | 263 | `ControlCommand::Commands` (local) |
| logs | 269 | `ControlCommand::Logs` (local) |
| diagnostics | 275 | `ControlCommand::Diagnostics` (local) |
| integrations | 281 | `IntegrationsCommand::{List,InstallSkill}` |

Wire families in `protocol.rs:51-73`: `status`, `app`, `window`, `ui`,
`command`, `tab`, `work`, `host`, `settings`, `events`, `link`, `chat` — note
that `link` is a wire family with **no CLI family** (it is reached by
`spaceport URL`), and `command`/`work`/`settings` are shared by several CLI
families. Error codes: `protocol.rs:163-179`; exit mapping:
`control/client.rs:232-248`.

## Appendix C: `skills/spaceport/SKILL.md` table against the source

Every command line in the skill's table (`SKILL.md:22-35`) parses today:
`notes set`, `worktree mute`, `host mute`, `ui panel`, `ports open`,
`processes`, `chat new`, `chat`, `config set`, `config get`, `wait tab-closed`,
`events --follow`, `caffeine on --for`, `run check-for-updates`, `status`,
`window`, `tab`, `tab focus`. No stale command names. The drift is in the
prose (F07, F11, F12) and the frontmatter description (F13), plus the fact
that nothing tests this table against `FAMILIES` (see the structural item).