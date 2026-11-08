# T04/RM-GIT: automatic checkpoint worker removal

At base `ac9e06e3`, every `SheprdWindow` tick called
`window/monitors.rs::update_checkpoints`. It read staged agent metadata and
queued Git snapshots through `sheprd-vcs::checkpoint::Checkpoints`, which
started local, SSH or WSL shell work. The `agent_checkpoints` config key and
Settings > General toggle enabled that path by default. The earlier CLI and
native-menu cuts left no user path to list or restore these snapshots.

This cut removes the tick route, window field, setting and preference writer,
UI row/catalog, example config, checkpoint worker module and its exclusive
VCS dependency/feature edges. It also removes checkpoint-specific error
messages and tests. The generic error-chain text used by retained worktree
scripts moved to `control.rs`; a focused test checks its cause survives.
`sheprd-vcs::git`, Notes, ordinary workspace menus, Core terminal identity,
and worktree script controls remain. The separate `fan_out::journal::Checkpoint`
is a progress marker, not a Git snapshot worker, so it remains.

The new source reachability test failed before the cut on the window tick
call. It passes after the cut and guards the tick, config, Settings row and
VCS module. `cargo test -p sheprd-vcs --locked` passed 23 unit tests and one
doc test. Focused GPUI tests passed for error-chain reporting, preference
round-trip, Settings row actions, retained checkpoint-free workspace menu and
exact Core terminal/workspace membership (one each). Default and all-feature
app checks passed. Format, file-size, sorted and i18n checks passed; i18n
validated 5,034 messages in 265 files. Cargo still reports `block v0.1.6`
future compatibility and the test linker reports the existing macOS
compact-unwind notice. Native GUI, Windows/Linux, full workspace and release
checks were not run. The source guard proves the capture route is absent from
this build graph; it does not observe a native window or process table.

No personal checkout, daemon, service or config was read or changed. Old
`agent_checkpoints` config text is reported as an unknown key and cannot start
the removed worker; this cut does not rewrite it. A focused config test proves
that status while a retained port setting still loads. Existing hidden Git
refs in a person's repositories are left intact because deleting them would
mutate user data. The root README now states the fork's development status;
the inherited website still contains upstream checkpoint marketing. T12 needs
to qualify or replace that entrypoint.

RM-GIT remains `in_progress` because other Git/worktree routes are live.
T04 remains `blocked` by G-LINK; this cut does not close it. A29 stays
`not_run`, and native A33/A34 visual/accessibility checks remain open.

## Integrated branch verification

After merging as `f11c641c`, the default and all-feature production app
checks passed with `--locked`; only Cargo's existing `block v0.1.6` notice
remained. The source reachability test passed 1/1. `sheprd-vcs` passed 23
unit tests and one doc test. The merged GUI test binary passed the stale
checkpoint-config test, the checkpoint-free workspace menu test, and the
retained control error-chain test, one each. Its linker repeated the known
macOS compact-unwind warning. `just format-check`, file-size and sorted
checks passed; `just check-i18n` validated 5,000 messages in 263 files.
No native desktop or process-table check ran on this merged branch.
