# Remaining Teleport engine removal

Base `749a117a5`; code commit `cf3bb13c5`; bounded source change under T04, with T06/T07 survivor contracts.
Doctor Biz approved continuing removals after the signed-preview slice. The
remaining crate was inspected through actual callers before deletion; its name
was not treated as proof that all of it was disposable.

## Surviving ownership

The existing `sheprd-remote` crate now exposes neutral host-script, agent and
snapshot modules. Chat and pane movement retain the original bounded capture,
streaming, cancellation, exact local/SSH/session targeting, shell quoting and
refusal of development installations or unsupported explicit sockets/WSL routes.
The shared client worker still owns the SSH query lane. No new transport or
runtime manager was added. Existing agent-session validation and preserved
argument/permission choices remain for current chat and explicit reopen callers.

Only pane, reported agent-session and process read shapes survive from the old
snapshot model. Unknown status still maps to unknown/Other. The app compiler
found an inferred `ProcessInfo::foreground_pids()` call that the first qualified
reference search missed; the original pure method and idle-shell/foreground-ID
regression were restored before the passing app compile. Source absence is not
proof that consumers still work.

No generic file/attachment transfer caller imports this crate. These routes were
not removed. Core and Herdr 1 remain separately installed runtime choices under
D58; this extraction does not establish Core native-chat or remote parity.

## Removed behavior

Deleted the migration tree: Git/worktree preparation, dispatch/fan-out jobs,
layout reconstruction, provisioning/preparation, credential/session copying,
archive rewriting, resume launch recipes, fallback handoff prompts, warnings and
step/progress machinery. The GPUI Teleport wildcard shim is gone; retained
consumers import the actual neutral modules. Only obsolete migration tests were
removed; retained host, agent, process, chat, pane, CLI refusal and arbitrary
content tests remain. No production migration API or compatibility shim survives.

The extraction removes tar/filetime/xattr from the resolved lock graph and drops
unused remote edges without upgrading versions. `serde_json` is test-only and
unused `tempfile` is gone. The [combined dependency comparison](t04-tail-dependency-comparison.json)
records both cleanup commits: 883 to 876 packages, seven removed, no additions,
version or checksum changes. All-feature metadata resolves for arm64/x86_64
macOS, GNU/Linux and Windows MSVC. This is graph resolution, not compilation or
runtime evidence on those platforms.

## Verification

Before extraction, remote tests passed 56 with two ignored migration live tests.
The new negative source tests failed first on three engine/export contracts
(14 expected failing subtests). After extraction, five source tests pass, remote
unit tests pass 13 in each feature mode, and strict remote Clippy passes. Targeted
app checks passed: pane moves 14; existing chat 12; conversion 24; restore 23;
overview closed/reopen 14; localized host errors one; removed Teleport CLI one.
The application compiles, and file-size/sorted/i18n/format checks pass.

Independent review compared retained implementations, typed failures, bounds,
cancellation, exact targets, arguments and snapshot fields against preimages.
It found only unused dependency edges, addressed before final integration.
The combined source is integration `f76255c8258dac1593a5a71eeb0a1bcdef658f6c`,
containing this cut and credential cleanup `c83835788`. Both task reviews and the
final whole-phase review approved the source. The final fresh-eyes pass found no
further issue. Root inspected the merged manifests, lockfile and app error owner;
no manual conflict resolution or unrelated source change occurred.

## Integrated verification

Commands ran on macOS arm64 in the isolated integration worktree with
`CARGO_INCREMENTAL=0`, Rust 1.99.0, `--locked --offline` for Cargo. Unit/UI layout
tests are headless; none establishes a native desktop journey.

| Command | Result |
| --- | --- |
| `cargo +1.99.0 test --locked --offline -p sheprd-remote --all-features` | 13 passed; zero ignored; doc-tests zero |
| `cargo +1.99.0 test --locked --offline -p sheprd-gpui --bin spaceport` | 3,864 passed; nine existing ignores |
| Same app command with `--all-features` | 3,893 passed; nine existing ignores |
| App `--test removed_updater --test build_identity --test preview_identity --test cli` | 34 passed: 13 identity/refusal and 21 real CLI executable cases |
| `python3 -m unittest discover -s scripts/release/tests` | 279 cases; eight skipped; zero failures |
| `python3 scripts/release/test-release-security.py` | 24 cases; one skipped; zero failures |
| `just check-file-size check-sorted check-i18n check-commits` | Passed; i18n 2,193 valid/used messages in 112 files |
| `cargo +1.99.0 fmt --all --check` and `git diff --check` | Passed |

Strict app Clippy (`--all-targets --all-features --no-deps -- -D warnings`) still
fails on exactly 253 inherited diagnostics. Code, message, primary file and
multiplicity match the pre-cut baseline, with none added or removed in either
individual cut or combined source. Strict remote Clippy passes. No suppression
was added. The inherited `block v0.1.6` future-compatibility and test-linker
compact-unwind notices remain; the local-network refusal unit takes over 60
seconds in each full app mode. No failed check was hidden or retried unchanged.

The existing signed `20261109.1` preview is preserved and was not rebuilt with
these commits. No app was installed, signed or notarized and no GUI was launched in
this cleanup. Native, live SSH and cross-platform compilation/CI remain unrun.
PACT objects were not changed; its earlier invalid-reference incident still
pauses ledger writes. Kata and Chronicle retain the task/evidence pointers.

## Remaining gate

`chat/convert::run` and legacy restore still carry desktop-owned lifecycle policy.
This extraction preserves those existing callers without presenting them as
SP-009 compliant. T06 remains blocked: the runtime-owned replacement and native
local/remote journeys have not been verified. Four retained chat host fixtures
compile only on Linux and did not run on this Mac. Native GUI, real SSH, Windows,
Linux and release qualification remain open. No personal daemon, credentials,
user environment, upstream repository or signed artifact changed.
