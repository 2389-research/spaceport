# T04: GitHub account state removal

Base: integration `8b48b7d48`. This cut removes the application's remaining
GitHub account ownership after the workspace PR reader cut. It does not close
T04 or A29: the final Git/VCS and native-platform gates remain open.

## Excluded routes

- The titlebar opens `Page::Profile` and a neutral `icons/user.svg`. The old
  `Page::GitHub`, avatar identity, attention and quota indicators, account
  switching, OAuth device flow, `gh` CLI probe, pasted token, approval,
  sign-out and scheduled account polling are no longer in the GPUI module
  graph. The 16 ms window tick no longer calls an account worker.
- `Config` and its Settings parser have no `[github]` field. An old account
  table gets a typed `RemovedGitHub` error, including when it contains a
  token. The editor no longer writes account entries. No old config file,
  stored credential or machine account was read, erased or migrated.
- Removing a saved device no longer offers or runs GitHub credential cleanup.
  It still claims and removes the exact saved profile, reports failures and
  shows progress until catalog reconciliation drops the device. The removal
  marker now prunes at that reconciliation point, rather than from account
  polling.
- Account-only profile image fetching, menu strings, fixtures and quota
  smoke paths are gone. Repository owner avatars and other GitHub-labeled
  historical/local data remain for the independent Git cut.

## Retained UX and dependency boundary

The titlebar retains its 40 px slot, 28 px pointer target, 20 px glyph,
left/right click access, keyboard focus and accessible popup role. The popup
retains the 400 px maximum width, shadow and viewport clamp. It shows a
visible, neutral Profile heading and local Settings, Keyboard Shortcuts,
Themes and About actions. Each row uses the existing `activate_menu` route;
no signed-in identity or authentication state is claimed. Usage remains in
its own status bar/panel and diagnostics remain in their existing local UI.

The `sheprd-github` crate is still in the dependency graph. The VCS Git
reader and legacy workspace close guard still use its pull-request/Git types
on this branch. GPUI also still has repository owner parsing and typed error
localization for those paths. This cut does not claim that the GitHub crate,
all Git actions or PR library code have been removed. The separate legacy
Close/Delete cut and subsequent Git/VCS cut must close those references.

## Verification and gates

The new headless regressions first failed on the old `Page::GitHub` click
route and accepted `[github]` config. They assert the profile slot and focus,
all four local rows, their destinations, no auth controls/worker call, and
typed config refusal. Existing native chrome and sidebar layout fixtures were
adapted to the neutral profile. Host removal tests keep the failure/progress
contract and assert no credential cleanup control.

With `CARGO_INCREMENTAL=0`, the pinned Rust 1.99 toolchain and the shared
target, `cargo check -p sheprd-gpui --bin spaceport --tests --locked --offline`
passed after a scoped clean of stale cross-worktree package artifacts.
The same `cargo check` with `--all-targets --all-features` passed, including
the gated native fixture source; this is compilation, not a native UI run.
Focused `cargo test -p sheprd-gpui --bin spaceport <filter> --locked --offline`
passed for the profile click, four local row destinations, typed config
rejection, absence of account polling/action source, host-removal control,
failure/progress and mute cleanup, unknown-key handling, theme-save retention,
and en-XA page title: 10 focused tests. The broader sidebar text/layout test
passed 1/1; titlebar headless tests passed 6/6; headless native-chrome geometry
passed 1/1. The earlier account click and config tests were observed red before
implementation. The test linker reported its inherited macOS compact-unwind
warning; Cargo reported the inherited `block` 0.1.6 future-compat note.

`just format-check`, `just check-file-size`, `just check-sorted` and
`git diff --check` passed. `just check-i18n` found only the clean-base
`browser-no-pages` orphan; removing the account module also removed the newly
orphaned `github-menu.ftl`. These checks cover source and headless behavior,
not the installed app or native desktop interaction. The integrated build
and platform checks remain with the integration branch.

No native macOS UI check ran: the machine presented `loginwindow`. Doctor Biz
can unlock it for a later active-desktop run. No real user environment was
mutated, and no release or Core repository was touched.
