# CLI docs fixes: `docs/cli.md` against the command surface

Follow-up to `docs/notes-cli-docs-audit.md`. This note records every edit
against the audit's prioritized fix list, what it was verified against, and
what was deliberately left out of this slice.

Scope of this slice: audit fix-list items **1-4** (F03, F01/F02, F04, F05) and
the structural test at the end of that list. Items 5-12 remain open.

## Audited findings, re-verified before editing

Every finding below was checked against this checkout first; all four held
exactly as the audit stated.

| Finding | Audit claim | Verified at |
| --- | --- | --- |
| F03 | Docs say protocol 36, code says 37 | `crates/sheprd-control-protocol/src/protocol.rs:42` (`PROTOCOL_VERSION: u32 = 37`); doc claim was `docs/cli.md:221` |
| F04 | Exit-4 row names a removed PR source and omits the real ones | only `Output::nothing()` sets 4 (`control/client.rs:75-77`), reached from `control/client/hosts.rs:36,49`, `settings/config.rs:38`, `events.rs:92,170`, `local/logs.rs:20`, `work.rs:49,62`, `chat.rs:45`; no PR surface exists (protocol versions 20 and 29 removed it) |
| F05 | `chat new` on an unreachable host exits 1, not 8 | `control/server/chat.rs:86-100`: `StartError::NotConnected` → `ErrorCode::Failed`; `control/client.rs:245` maps `Failed` → exit 1. Only `Disabled`, `AgentMissing`, `ProgramMissing`, `NotLocal`, `CoreTerminalRequired` are `Unsupported` (exit 8) |
| F01/F02 | `skill` and `wait` are families with no documentation | `crates/sheprd-gpui/src/cli/control.rs:119` (`skill`), `:251` (`wait`); `wait` default 60 s and cap 600 s at `cli/control/work.rs:59-63`; `Condition::{Window,TabClosed,Online}` at `protocol/events.rs:90-98` |

## Edits, one audit item at a time

### Item 1 (F03): protocol version

- `docs/cli.md:226` — "the protocol version, currently 36" → "37". Verified
  against `PROTOCOL_VERSION` (37), which `status` prints and which the client
  refuses to mismatch (`control/client/app.rs:89-95`).

### Item 2 (F01/F02): the two missing families

- `docs/cli.md:153` — new row in **Settings and services**, beside
  `integrations install-skill`, the command that installs what it prints:
  `` | `skill` | Print the agent skill `integrations install-skill` installs (no app needed) | ``.
  `skill` returns before any socket work (`control/client.rs:341-347`), hence
  the "(no app needed)" marker the doc already uses for such families.
- `docs/cli.md:174-176` — the **Events** section had prose and an example block
  but no table; it now has a one-row table for `wait`:
  `` `wait window\|online\|tab-closed ID [--timeout SECONDS]` `` blocking until
  a window exists, the selected host comes online, or app tab `ID` closes,
  "60 seconds unless `--timeout` says otherwise, and at most 600", exiting 4
  when time runs out. This names `wait online`, which appeared nowhere in the
  docs, and states the default and the cap.
- `crates/sheprd-gpui/src/cli/control.rs:252` — the `wait` summary in
  `spaceport --help` said "Wait for a window or a tab to close", which omitted
  `online` and would have contradicted the new doc row. One line:
  "Wait for a window, a host to come online, or a tab to close". This is the
  only help text changed for a reason other than F04/F05; it is needed for the
  documented surface and the printed surface to agree.

### Item 3 (F04): the exit-4 row

- `docs/cli.md:50` — `Nothing yet: no notes, no event, no pull request,
  condition not met` → `Nothing yet: no notes, no events, no chat panes, no
  ports, no usage, no log, a key not set, or the wait not met`. This drops the
  removed PR source and covers each remaining exit-4 path found by reading
  every `Output::nothing()` call site (listed above).
- `crates/sheprd-gpui/src/control/client/help.rs:9-11` — the same wording in the
  shared `--help` text: `4 nothing yet (no event)` → `4 nothing yet (no notes,
  no events, no chat panes, no ports, no usage, no log, a key not set, or the
  wait not met)`. The two surfaces now say the same thing word for word.
- `crates/sheprd-gpui/src/control/client/tests/family_help.txt` — regenerated
  in the same commit, as the byte-exact snapshot requires
  (`tests/family_help.rs:61-69`).

### Item 4 (F05): `chat new` on an unreachable host

- `docs/cli.md:110` — the `chat new` row now reads "Exits 8 when `sheprd-acp` is
  missing here or the agent is not installed on it (`sheprd-acp agents` lists
  those that are), and 1 when the host cannot be reached". The ACP-off case is
  already documented under [Chats on SSH hosts](#chats-on-ssh-hosts).
- `crates/sheprd-gpui/src/cli/control/chat.rs:4` — `CHAT_USAGE` said the same
  wrong thing; it now ends "Exits 8 when ACP chats are off in Settings,
  sheprd-acp is missing here, or the agent is not installed there. Exits 1 when
  the host cannot be reached." Both surfaces were fixed together, as the audit
  asked, and the snapshot was regenerated for this text too.

## The structural test that keeps this from recurring

`crates/sheprd-gpui/src/control/client/tests/documented_commands.rs` (new,
registered in `tests.rs`), three deterministic unit tests beside
`family_help.rs`:

1. `the_cli_doc_names_every_family` — every `cli::control::FAMILIES` name is
   documented as a command in `docs/cli.md` (in backticks, either
   `` `name ...` `` or `` `name` ``). Catches F01/F02 and any future family.
2. `the_cli_doc_states_the_protocol_version_the_app_speaks` — parses the doc's
   `protocol version, currently N` claim and asserts it equals
   `PROTOCOL_VERSION`. Catches F03.
3. `every_skill_command_line_parses` — every `` `spaceport ...` `` span in
   `skills/spaceport/SKILL.md` parses through `cli::control::parse`. Two forms
   name the program rather than a family and are skipped explicitly:
   `spaceport --help` and `spaceport COMMAND --help`. An all-uppercase
   placeholder (`ID`, `KEY`, `VALUE`) is parsed as `1`, the sample value that
   parses wherever a real one would.

Both files are read with `include_str!`, so a change to either is a compile-time
input and no test touches the filesystem at run time.

## Deliberately not in this slice

Items 5-12 of the audit fix list are still open in `docs/cli.md` and
`skills/spaceport/SKILL.md`: the events prose (F06, "notes kept for a pane"),
`--json` for `events --follow` and option ordering (F07), the skill's dangling
exit-table reference (F11), `COMMANDS_USAGE` promising shortcuts (F08), the
`config check` / `theme` / `font` / `logs` / `ui panel` behavior notes
(F14-F16, F20), the request-shape and local-`--window` paragraphs (F09/F10),
the two skill prose claims (F12/F13), and the P3 ranges and aliases (F17-F24).

F25 (source-side legacy catalog names) needs no doc change, as the audit says.

Note for whoever takes F06: the new `wait` table in the Events section sits
above prose that still lists "notes kept for a pane"; fix both together.

## Validation

- `cargo test --locked -p sheprd-gpui --bin spaceport -- control::client::tests`
  — **14 passed, 0 failed** (run 2026-11-18). That run covered
  `family_help::help_text_matches_the_recorded_text` (the byte-exact snapshot),
  `family_help::{every_listed_family_has_usage_starting_with_its_own_command,
  family_names_are_unique}`, and the three new tests in
  `documented_commands`.
- After that run, one line was removed from the new test file: `use super::*;`,
  which the compiler reported as an unused import (the file uses only explicit
  `crate::` imports). That removal cannot change what the tests assert, so the
  suite was not re-run for it; it is not re-executed evidence. CI re-runs the
  whole job anyway.
- `cargo fmt --all -- --check` — clean, after the removal above.
- `just check-file-size` — all files within 1000 lines.
- `just check-commits` — exit 0 for the two commits of this slice.
- `git diff --check origin/main..HEAD` — no whitespace errors.
- No native/UI check applies: nothing here changes rendering, input, or
  startup. `docs/cli.md`, the help text, and the snapshots are the whole
  user-visible surface.

Disk note: the one gpui test build took this volume from 11 Gi to 1.8 Gi free
(other worktrees were building at the same time). The worktree's `target/` was
removed afterwards to give the space back, so no build output is kept here.
