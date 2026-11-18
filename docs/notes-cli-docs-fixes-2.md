# CLI docs fixes, second slice: audit items 5-12

Follow-up to `docs/notes-cli-docs-audit.md` and `docs/notes-cli-docs-fixes.md`.
This note records, item by item, what the audit's prioritized fix list items
5-12 asked for, what the code said when it was checked, what changed, and how
it was verified.

Scope: items **5-12** (findings F06-F24 as they appear in those items). Items
1-4 and the structural drift test landed in the first slice
(`crates/sheprd-gpui/src/control/client/tests/documented_commands.rs`), and
were not redone. This branch was fast-forwarded onto that slice's tip
(`cbce4964e`) so the structural test is present and stays green.

Every finding was re-verified against this checkout before editing; none had
drifted out of date, so no item is recorded as stale.

## Edits, one audit item at a time

### Item 5 (F06): the events prose named an event that does not exist

- Verified: `crates/sheprd-control-protocol/src/protocol/events.rs`
  `EventKind` holds `window_opened`, `window_closed`, `window_focused`,
  `tab_opened`, `tab_closed`, `note` ("A worktree note changed"), `host`.
  There is no pane-note kind and no pane-note source, so the doc's "notes kept
  for a pane" described an event nothing emits, and named worktree notes
  twice.
- `docs/cli.md`, Events section: "the app's tabs opening and closing; a
  worktree note changing; and a host's connection status changing."
- Commit `eb70b5fe8`.

### Item 6 (F07): `--json` promised one document

- Verified: `crates/sheprd-gpui/src/control/client/events.rs` `--follow` writes
  one JSON object per line, each with its own `cursor` (the loop that inserts
  `cursor` into each event's object). `control/client.rs::run` returns from
  `ControlCommand::Help` and `ControlCommand::Skill` before `finish(json)` is
  reached, so both ignore `--json`. `cli/control.rs::wants_json` only looks at
  `args.first()`, and `cli.rs::LaunchOptions::parse` rejects a leading
  `--json` as `UnknownOption`, i.e. exit 2.
- `docs/cli.md`, Options: the `--json` bullet now says `events --follow` is the
  one stream and writes one JSON object per line with its `cursor`, and that
  `--help` and `skill` print text whatever the options say. A new sentence
  after the bullets: "Both options follow the command word: `spaceport --json
  status` is an unknown option, not `status` asked for JSON."
- `skills/spaceport/SKILL.md`: "Add `--json` after the command word for
  machine-readable output ... `spaceport events --follow` writes one JSON
  object per line instead of one document."
- Commit `eb70b5fe8`.

### Item 7 (F11): the skill's dangling exit-table reference

- Verified: `skills/spaceport/SKILL.md` had no exit-code table, and the file is
  installed standalone into agent directories (`crates/sheprd-gpui/src/
  agent_skill.rs` `include_str!`s it), so "the table above" pointed at nothing
  an agent can see.
- `skills/spaceport/SKILL.md`: the sentence is replaced with the codes
  themselves (0 done, 1 refused or failed, 2 invalid arguments, 3 not running,
  4 nothing yet, 5 not found, 6 busy, 7 timed out, 8 unavailable) and a pointer
  to `spaceport COMMAND --help`, which prints the same table from
  `control/client/help.rs::COMMON`.
- Commit `eb70b5fe8`.

### Item 8 (F08): `commands --help` promised shortcuts

- Verified: `crates/sheprd-gpui/src/control/client/local.rs::commands` prints
  each command's `name` and its English label; no shortcut data is in
  `CommandInfo`. Shortcuts belong to `spaceport keybinds`.
- `crates/sheprd-gpui/src/cli/control/plain.rs` `COMMANDS_USAGE`: "The commands
  `run` accepts, with their shortcuts." → "... with their labels."
- `crates/sheprd-gpui/src/control/client/tests/family_help.txt`: the
  `=== commands ===` block regenerated for that one line. This is the only
  edit in this slice that the byte-exact `--help` snapshot sees.
- Commit `a2ef7b2d7`.

### Item 9 (F14/F15/F16/F20): the behavior the tables left out

- `config check` (F14), verified in `control/client/settings/config.rs::check`:
  it also asks the running app (`SettingsOp::CheckConfig`) for configured fonts
  it lacks, appends them to the problems (so a font problem alone exits 1), and
  without an app prints "Fonts were not checked: that needs a running Spaceport
  window." Doc row now says all three.
- `theme`/`font` (F15), verified in `cli/control/settings.rs` (`THEME_USAGE`
  documents `light:NAME,dark:NAME`) and `control/client/settings.rs`
  (`theme` lists from the config files and saves the setting when no app
  answers; `font` asks the app for the installed families). Doc row now names
  the `light:`/`dark:` pair, and says `list` needs no app while `set` saves
  whether or not the app runs; the `font` row says it needs the running app.
- `logs` (F16), verified in `cli/control/plain.rs` (`--lines` defaults to 100,
  bounded `1..=TAIL_RECORDS` = 5000) and `control/client/local/logs.rs`
  (`.nothing()`, exit 4, before the first log). Doc row now states both.
- `ui panel` (F20), verified in `control/server/app.rs` `UiOp::Panel`: exit 8
  for `automations` while Core is off, when the activity bar is off
  (`[sidebar] activity_bar = false`), and when that section is turned off in
  settings. Doc row now lists the three refusals.
- Commit `f9e56880b`.

### Item 10 (F09/F10): the option and protocol paragraphs

- F09, verified by reading each handler: `commands`, `processes`, `logs`,
  `diagnostics bundle`, `config path|get|set|unset`, `theme list`,
  `language list|set`, `skill`, and `integrations install-skill` take no
  `Context` and never reach the app, so they accept `--window` and cannot
  refuse an ID. `theme set` is not in that set: the app resolves the window
  (`control/server/settings/saves.rs::theme` calls `window_for`), so the doc
  does not mark it "no app needed". `docs/cli.md`, Options: the `--window`
  bullet now says a command marked "no app needed" accepts the ID and ignores
  it. Commit `eb70b5fe8`.
- F10, verified in `crates/sheprd-control-protocol/src/protocol.rs`:
  `Request::Status(Caller)` flattens the caller fields beside `"method"`,
  while every other family is `Call<T> { caller, op }`. `docs/cli.md`, The
  protocol: "Every request but `status` is `{"method", "caller", "op"}` ...
  `status` carries those caller fields beside `"method": "status"` instead,
  with no `op`." Commit `6a447e140`, reflowed in `2e011f271`.

### Item 11 (F12/F13): two skill claims the app cannot keep

- F12, verified: `control/server/hosts/ports.rs::open` →
  `browser/view.rs:203` `cx.open_url` — the system handler, not an in-app tab.
  `skills/spaceport/SKILL.md` row: "Open a dev server in your system browser,
  forwarding a remote port to this machine first".
- F13, verified by the family list: there is no `files` family among
  `cli::control::FAMILIES`. The frontmatter `description`, which is all an
  agent sees before loading the skill, no longer lists "files"; it still tells
  the agent to share a URL or file path for the system handler.
- Commit `c378b7db8`.

### Item 12 (F17-F24): ranges, aliases, wording, and top-level flags

- F17 `notes`: `cli/control/work.rs` refuses text over 16 KiB (`MAX_TEXT`) and
  `NOTES_USAGE` states one line of at most 500 characters; `control/client/
  work.rs::notes` returns `.nothing()` for `list`/`get` with no note. Doc row
  now says all of it.
- F18 `ui font-size`: `cli/control/app.rs` accepts `+`/`larger`, `-`/`smaller`,
  `reset`, or 8-48 points. Doc row names the range and the two spelled-out
  forms.
- F19 `window size`: 200-16384 wide and 150-16384 high (`cli/control/app.rs
  ::size`). Doc row names both ranges.
- F20: covered by item 9.
- F21 aliases: `status-bar` (`cli/control/app.rs::ui`), `show`/`hide` for any
  `switch` (`cli/control.rs::switch`), `chat list` (`cli/control/chat.rs`),
  `devices [open]` (`cli/control/tabs.rs`). Each is now in its own row.
- F22 `config get`: `control/client/settings/config.rs::get` answers with
  `Config::effective_value` (override, else default, with `source`
  `local|default`). Doc row: "Read a key's effective value (your override, else
  the default)".
- F23 top-level flags: `crates/sheprd-gpui/src/app.rs` prints
  `--build-info` and `--socket` in the launch help and
  `cli.rs::LaunchOptions::parse` implements them. `docs/cli.md` now shows
  `spaceport --build-info` in its opening block and explains both flags.
- F24 unknown `run` name: `control/client/app.rs::run_command` refuses a name
  outside the catalog with `ErrorCode::NotFound`, i.e. exit 5, before asking
  the app. The `run COMMAND` row says so, and the exit-5 row now reads "The
  window, workspace, tab, host, file, or `run` command name is not there".
- Commit `788ec28ec`.

### The audit file

`docs/notes-cli-docs-audit.md` keeps every finding and now carries a
`(fixed in <commit>)` annotation for each prioritized item, plus a status line
pointing at this note and the first slice's. Nothing was deleted. Commit
`7af507db0`.

## Deliberately not done

- **`control/client/help.rs::COMMON` still says `--json` prints "one JSON
  document".** Item 6 named only `docs/cli.md` and the skill, and this slice
  owns those files, so the shared help text was left alone rather than widened
  into. It is now the one surface that disagrees with the new doc sentence
  about `events --follow`. A follow-up one-line change there also needs
  `family_help.txt` regenerated (every family's usage ends with `COMMON`).
- **The structural test was not extended.** The audit's remaining items imply no
  new guarded surface: it asked for families, the protocol version, and the
  skill's command lines, all of which landed in the first slice. Pinning the
  exit-code rows was not requested, so no assertion was added.
- **F25** (the catalog's legacy `install_browser_skill` and hyphenated
  `check-for-updates` names) needs no doc change, as the audit says; the doc
  already uses the real names.
- The exit-4 and exit-5 row wording changes are `docs/cli.md` only: `COMMON`
  is unchanged, so no snapshot regeneration was needed for them.

## Validation

Run in this worktree on 2026-11-18, in this order:

- `just check-commits` — **exit 0** for all commits of this slice (one subject
  was shortened after the first run rejected it at 84 characters).
- `just check-spec` — **exit 0**: 34 tool tests, spec check passed, 333 local
  file links checked.
- `git diff --check` — clean, on the working tree and on `origin/main..HEAD`.
- `cargo test --locked -p sheprd-gpui --bin spaceport -- control::client::tests::`
  — **not run locally, deferred to CI.** It needs the full `spaceport` bin
  build (GPUI included), which the first slice measured at ~9 Gi; this volume
  had 5.6 Gi free with a peer session mid-build, so the disk guard applies and
  the run was deferred rather than risk an ENOSPC on this and a peer's build.
  The `just test-gpui` and `just check-spec-tools` recipes named in the task do
  not exist in this worktree's justfile (this branch predates that PR).
- Local evidence for the three assertions in `documented_commands.rs`, checked
  by script against the files the tests `include_str!`:
  - 29 of 29 `cli::control::FAMILIES` names appear in `docs/cli.md` as
    `` `name ...` `` or `` `name` `` — none missing.
  - The doc's "protocol version, currently 37" equals
    `PROTOCOL_VERSION` (37).
  - All 23 `spaceport ...` spans in `skills/spaceport/SKILL.md` name a real
    family (the two that name the program, `spaceport --help` and
    `spaceport COMMAND --help`, are skipped by the test as before).
  This is the same evidence the tests assert for (a) and (b); for (c) it
  checks family membership rather than the full `cli::control::parse` call, so
  CI's run of the real test remains the authority.
- No native/UI check applies: nothing here changes rendering, input, or
  startup.
