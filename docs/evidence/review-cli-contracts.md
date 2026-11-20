# CLI safety and transport contracts

Date: 2026-11-17.
Scope: Kata `4qzb`, `f7r9`, and `p32e`, under `9s44`.
Doctor Biz authorized implementation and validation.
This task does not authorize commits, pushes, PR changes, or Kata writes.

## Changes

Transport failures keep their existing typed sources.
CLI classification uses `io::ErrorKind` and `serde_json::Error` categories.
EOF and broken connections exit 3.
Read/write timeouts exit 7.
Unexpected response kinds, response shapes, and explicit protocol mismatches exit 8.
The CLI never retries a submitted mutation after a lost answer.
A missing answer leaves a mutation's outcome unknown.

Tab lists and actions resolve one window and one caller session.
Caller workspace and daemon qualifiers constrain the result.
Ambiguous windows require `--window`.
Hidden tabs and unshown workspaces cannot be focused or closed.
The selected endpoint distinguishes identical workspace names on different sessions.
Core-disabled actions retain their existing unsupported result.

Closure waits validate an existing available tab before parking.
Online waits validate their session before parking.
Each online wait retains its original endpoint ID and typed connection target.
A replacement session cannot satisfy the wait.
An unknown target exits 5 instead of succeeding or waiting until timeout.

The server permits eight long waits and eight additional connections.
Request reads also consume the additional connection allowance.
The UI queue remains bounded at four requests.
A ninth wait exits 6 without consuming all quick-command capacity.
Cancellation releases its waiting allowance.

The private transport retains native OS path bytes.
The existing caller qualifier uses a UTF-8 wire string.
An unrepresentable caller daemon path now refuses before connecting.
This prevents silently dropping or rewriting its session qualification.
The wire format remains unchanged.

## Regression evidence

`socket::tests::capacity::eight_parked_waits_leave_capacity_for_quick_commands`
failed before the capacity fix.
Eight parked waits prevented the next quick request from reaching the UI.
The corresponding green run passed.

The initial control regression run failed 12 intended cases.
It reproduced exit-code errors, unscoped tabs, unknown closure waits, and unshown online waits.
It also exposed an existing parser golden change, handled by the integrating owner.
The later endpoint regression reproduced same-named workspace leakage within one window.
The online replacement regression reproduced a wait following a replacement session.

Five executable regressions used real private Unix sockets and scripted peers.
All five failed before transport changes.
They observed EOF exit 1, timeout exit 1, and incompatible replies exiting 1 or 0.
Each now checks the promised exit status and JSON error code.
The peers keep listening until command exit to detect automatic retries.

The non-UTF-8 caller regression observed exit 3 before the refusal fix.
The raw API path also needed validation before discovery normalized its filename.
The address-construction regression preserves native OS bytes on Unix.
A Linux-only regression exercises real sockets with a non-UTF-8 filename.
macOS bind rejected that filename with EILSEQ during local verification.
The Linux-only roundtrip remains unverified locally.

## Validation

All Cargo commands use Rust 1.99.0 and `--locked`.
They serialize through `/tmp/spaceport-qnz9/cargo.lock`.
They use the shared target directory in `.worktrees/upstream-fixes/target`.

- `cargo test --locked -p sheprd-control-protocol socket:: -- --nocapture`: 20 passed on macOS.
- `cargo test --locked -p sheprd-gpui --bin spaceport control:: -- --nocapture`: 125 passed, including endpoint replacement refusal.
- `cargo test --locked -p sheprd-gpui --test cli transport:: -- --nocapture`: six passed, including raw non-UTF-8 refusal.
- `cargo clippy --locked -p sheprd-control-protocol -p sheprd-gpui --tests -- -D warnings`: passed.
- `just check-file-size check-sorted`: passed, including ten sorted-check utility tests.
- Scoped `git diff --check`: passed.

The implementation owner reviewed the diff and relevant callers.
That review found the endpoint replacement gap and added its regression before fixing it.
The integrating owner independently reviewed the diff, callers, and regression tests.
No material findings remained in transport classification, wait capacity, or session targeting.

Full workspace CI, all-feature builds, and native Windows execution are deferred.
Headless fixtures and scripted peers do not establish native desktop behavior.
The integrating owner owns private native smoke coverage.

Cargo reports the existing future-incompatibility notice for `block v0.1.6`.
No unrelated dependency changes belong to this task.

Local logs use the `cli-` prefix in `/tmp/spaceport-qnz9/`.
The durable regression tests live beside the changed control modules and in `tests/cli/transport.rs`.
