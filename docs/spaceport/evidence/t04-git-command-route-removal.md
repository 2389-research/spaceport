# Direct Git menu command removal

Base: integration `389017d19`. The T04 source audit found two ways to open
the native Git menu: the titlebar button and `Command::GitMenu` through the
shared command catalog, palette/key bindings, and GPUI command dispatcher.
The titlebar, menu actions, Git status worker, sidebar badges, GitHub polling,
worktree picker and close preflight remain live. This cut removes only the
direct command route; it is not negative reachability for all Git behavior.

The new catalog regression first failed because `git_menu` was present. After
removing the enum variant, catalog entry, GPUI dispatch and settings command
classification, it passed. The test also verifies that stale `"GitMenu"`
command configuration is rejected and Workspace and NewCoreTerminal remain
available. The obsolete headless tests for this command's titlebar anchor
were removed; Git button tests remain. The now-unused English command label
was removed; terminal/chat text containing `git` is unaffected.

Verification on the isolated branch: `cargo +1.99.0 test -p sheprd-commands
--locked --offline` passed 28/28; `cargo +1.99.0 check -p sheprd-gpui
--all-targets --locked --offline` passed on macOS. Focused retained GPUI
`titlebar::git_button_tests` passed 6/6, confirming the titlebar path remains
live pending its own cut. The app test link emitted the known macOS
compact-unwind warning and Cargo reported the `block` 0.1.6 future-compat
note. `just format-check`, `just
check-file-size`, `just check-sorted` and `git diff --check` passed. The
package validator and 21 tool tests passed using the existing package
virtual environment. System `python3` lacks `jsonschema` and could not
perform schema validation. `just check-i18n` was run on both the isolated
branch and clean integration base `389017d19`; both fail at exactly
`menu/github.rs:760` "GitHub" literal and unused `browser-no-pages`. This
cut added no i18n finding. No native window or Linux/Windows test
ran. The package validator checks the evidence graph, not app behavior.

T04 remains `blocked` by G-LINK; A29 stays `not_run`. Next Git cut must remove
the titlebar/menu/worker and sidebar/status paths coherently, and preserve
workspace close safety while removing its Git preflight. No backend method was
invented and no Core or user environment was changed.
