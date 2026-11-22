# tng8 records

One directory for the records the [`tng8` run sheet](../tng8-run-sheet.md) asks
each session to leave behind: `<ID>-herdr1.md` and `<ID>-core.md` per leg, raw
command output under `logs/`, and other captures in the subdirectories the sheet
names.

Every record here is a **partial attestation**. None of them moves a leg to
`passed`: each states the command that ran, the counts it produced, which
partial claims it re-tested, and — explicitly — what it does not prove.
`plan/acceptance.json` cites each file on the leg whose runtime produced it and
leaves that leg `not_run` until the remaining halves in the sheet's Fixture
column have run.

## Session H (headless), 2026-11-22

Run at `708ca09f3a2d7a61c3b56e7d9bd3537ed9125c52` on macOS 26.7.1, arm64, with
no daemon selected. All eight commands passed:

| Row | Command | Result | Log |
| --- | --- | --- | --- |
| H1 | `just test` | default: 5471 passed, 0 failed, 31 ignored (73 suites); `--all-features`: 5522 passed, 0 failed, 47 ignored (75 suites) | `logs/H1-just-test.log` |
| H2 | `just test-protocol` | 640 passed, 0 failed, 20 ignored (16 suites) | `logs/H2-test-protocol.log` |
| H3 | `cargo test --locked -p sheprd-terminal` | 80 passed, 0 failed | `logs/H3-osc52-commands.log` |
| H4 | `... --bin spaceport core_terminals::workspace_document::edit::tests -- --nocapture` | 7 passed, 0 failed, 3 ignored (the live Core cases) | `logs/H4-H7-gpui-filters.log` |
| H5 | `... --bin spaceport control::targets::tests` | 1 passed, 0 failed | `logs/H4-H7-gpui-filters.log` |
| H6 | `... --bin spaceport endpoint::lifecycle_tests` | 172 passed, 0 failed | `logs/H4-H7-gpui-filters.log` |
| H7 | `... --bin spaceport daemon::tests` | 34 passed, 0 failed | `logs/H4-H7-gpui-filters.log` |
| H8 | `... --test cli`; `... --test removed_updater`; `just test-build` | 21 passed; 2 passed; release build and 21 passed | `logs/H8-*.log` |

Records: `A12-core.md`, `A28-herdr1.md`, `A35-herdr1.md`, `A37-core.md`,
`A38-herdr1.md`. `logs/A28-audit-deps.log` is the `just audit-deps`
(`cargo deny check`) run A28 also asks for; it passed
(`advisories ok, bans ok, licenses ok, sources ok`).

## Session P1 (private Herdr 1), 2026-11-22

Same revision and machine, with `HERDR_TEST_BINARY=/Users/clint/.local/bin/herdr`
(Herdr `0.9.5`, sha256 `fb77c2232a05a8035421382c894392f4174e4e38a18e48af3dd0290157c3a7e2`).

- `P1-a` `just test-live <herdr>`: 4 passed, 0 failed — `logs/P1a-live-client.log`.
  `logs/P1a-live-client-first-attempt.log` is kept because its one failure is a
  host prerequisite, not a product failure: without
  `HERDR_TEST_TMPDIR=/tmp` the sandbox's named-session socket nested past
  `sun_path`, and `herdr` exited with "local socket name length exceeds capacity
  of sun_path of sockaddr_un".
- `P1-b` was not run separately: the two tests it names
  (`named_session_create_and_delete_cli_contract`,
  `daemon_completion_status_live`) are part of the `P1-a` suite that passed, and
  `P1-b` exists to isolate a failure.
- `P1-c` `--features integration-test --test sparkle_lifecycle
  failed_fixture_cleanup_stops_shell_without_pid_signals`: 1 passed, 0 failed —
  `logs/P1c-sparkle-lifecycle-cleanup.log`, whose marker reads
  `LIFECYCLE cleanup PASS: shell_pid=80103; no sentinel, no PID signals; exact
  owned daemon child stopped and shell exited`. No row's Fixture column names
  `P1-c`, so it has no record of its own.

Records: `A03-herdr1.md`, `A06-herdr1.md`, `A18-herdr1.md`, `A23-herdr1.md`,
`A35-herdr1.md`.

## Running these sessions locally

Three host steps were needed, and a future session on a fresh machine will need
them again:

1. **Sparkle.** `sparkle-updater`'s build script refuses to compile without
   `SPARKLE_FRAMEWORK_PATH`. Use CI's own pinned step, which downloads Sparkle
   2.10.1 and validates its digest:
   `bash scripts/release/prepare-sparkle-sdk.sh /tmp/spaceport-sparkle-sdk`,
   then export `SPARKLE_FRAMEWORK_PATH=/tmp/spaceport-sparkle-sdk`.
2. **Runtime loading inside `just`.** CI exports `DYLD_FRAMEWORK_PATH` and runs
   `cargo test` itself; `just` strips `DYLD_*` from recipe environments, so a
   `just test` run links Sparkle but cannot load it, and the `spaceport` binary
   the CLI tests spawn aborts in dyld. Either run the recipe's `cargo` commands
   directly with the variable exported, or place the framework where the crate's
   `@executable_path/../Frameworks` rpath looks: symlink it into
   `target/Frameworks`, `target/debug/Frameworks` and
   `target/release/Frameworks`.
3. **Live fixtures.** `HERDR_TEST_TMPDIR=/tmp`, as CI sets, so the sandbox's
   nested socket paths stay under `sun_path`.

Disk is the other constraint: the macOS debug workspace build exceeds 4.8 GiB,
which exhausted a host with 5.2 GiB free (`No space left on device` while
compiling `sheprd-gpui`'s test binary). The session above ran with
`CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0` — the same tests and features,
no DWARF and no incremental directory — which is a deviation from CI's profile
and every record says so.
