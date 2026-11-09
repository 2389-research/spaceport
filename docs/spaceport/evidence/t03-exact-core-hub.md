# T03 exact Core hub binding prerequisite

Source base: integration `2696bf252`. This is a bounded prerequisite for an
additional Core-only window, not a new window route. `SheprdWindow::new` still
creates a legacy endpoint and calls its reconnect path; New Window and CLI
`window new --window` still copy that legacy target. The full constructor cut
remains open.

The existing per-window Core `Hub` now stores an expected persistent Core ID
from a fresh, validated `CoreGroupKey` when the user selects that group. The
binding itself does no I/O. It refuses binding during an already-submitted
`IfMissing` attempt, cancels a queued start behind a join, and makes later
requests join-only while the group is selected. It checks the ID returned by
Core's handshake before exposing a joined backend. Leaving or invalidating
the group clears that binding. A rejected selection shows a bounded in-app
warning, without logging the raw ID. Core owns the connection and process;
this cut adds no transport, start route or local runtime registry.

## Tests and observed limits

The Hub test first failed at missing API compile errors. Minimal API stubs then
produced behavioral RED: a later start reached `IfMissing`, a foreign ID was
accepted, a queued start caused another attempt, and an in-flight start was
accepted. With the guard implemented, the focused Hub filter passed 5/5;
after adding a clear-and-rebind identity check it passed 6/6. The headless
Core-group selection test was run with its binding temporarily absent and
failed at `[IfMissing]` rather than `[Join]`; restored binding passed 1/1.
That test also checks no connection occurs on selection and that an explicit
start is again allowed after leaving the group. These tests use an isolated
join callback and do not touch a personal Core.

All focused Rust runs used Rust 1.99.0, `--locked --offline`,
`CARGO_INCREMENTAL=0`, and the shared warm target. One attempted RED compile
reused stale `sheprd-commands` metadata from another worktree and failed on
`GitMenu` before reaching this test. A scoped `cargo clean -p sheprd-commands`
removed only eight build files; the next run produced the expected assertion.
The existing macOS compact-unwind linker warning and `block` future-compat
note remained. `just format-check`, `just check-file-size`,
`just check-sorted`, and `git diff --check` passed after formatting. `just
check-i18n` remains red at unchanged `menu/github.rs:760` (`GitHub` literal)
and unused `browser-no-pages`; this cut added no i18n finding. The package
validator and its 21 tool tests passed using the existing isolated package
virtual environment. An initial attempt with system Python could not validate
schemas because `jsonschema` is not installed there; that was an environment
setup failure, not a specification result. Package checks do not prove desktop
behavior.

The clear-and-rebind unit check verifies the latest target in the same helper
that `poll` calls when a result arrives. It does not supply a real mismatched
backend result to `poll`. No real Core, native window, or cross-platform run
was exercised. A09 remains `not_run` because restore and process survival were
not tested. T03 stays `in_progress`: constructor, exact-hub additional window,
stale tear-off rollback, and A24 remain open.
