# GitHub account entry cut

Base: integration `16987909c`. This T04 change removes the application menu's
`GitHub sign-in` row, its direct dispatch and locale label. A left click on the
titlebar profile slot now opens the existing popup without starting OAuth. The
right click already opened that popup without starting OAuth. The 40-pixel
account slot, 28-pixel hit target, popup focus and existing settings, shortcuts,
themes and status placement remain. This is a bounded entry cut, not a claim
that GitHub accounts have been removed.

The new application-menu test failed on the old sign-in row, then passed with
the retained general rows present. The new titlebar test used an invalid OAuth
client ID so the old automatic sign-in failed before any network request; it
failed on the changed auth state, then passed after the click stopped calling
`start_github`. Integration review added a source-route guard before the click,
so an ambient CI OAuth client ID cannot turn this test into a network request.
The titlebar suite passed 5/5 and the menu font-size suite
passed 3/3. `cargo +1.99.0 check --workspace --all-targets --locked --offline`
passed on macOS. `just format-check`, `just check-file-size`,
`just check-sorted` and `git diff --check` passed. The package validator
passed schema and local-link checks, and all 21 package tool tests passed.
These package checks do not prove app behavior. The fresh worktree had no
`.venv`, so system Python first failed on missing `jsonschema`; both checks
passed using the existing package virtual environment from the integration
worktree. The
linked GPUI test binary emitted the inherited macOS compact-unwind warning and
Cargo's inherited `block` 0.1.6 future-compatibility note. `just check-i18n`
still reports only the two clean-base findings: the `menu/github.rs:760`
GitHub literal and unused `browser-no-pages` message. Native desktop, Linux
and Windows interaction were not run.

The remaining account routes are live and must be removed in later T04 cuts:
`window.rs` calls `poll_github` every tick; `menu/github.rs` polls the main,
host and additional accounts and probes the GitHub CLI; `Page::GitHub` still
renders OAuth/token/add-account controls from the titlebar popup; `config/github.rs`
still reads and edits `[github]`; `menu/devices/host_menu.rs` offers host
credential cleanup; and the PR cache and workspace PR card still read
`pr_profile`. `sheprd-github` remains a build dependency. The generic
`sheprd-credentials` owner, usage UI and diagnostics stay for retained behavior.
No Core or personal credentials were changed. A29 and A33 remain `not_run`;
T04 remains blocked on G-LINK and unfinished GitHub negative reachability.
