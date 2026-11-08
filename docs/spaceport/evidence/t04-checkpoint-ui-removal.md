# T04/RM-GIT: native checkpoint menu removal

At base `0893a857`, the workspace popover offered Checkpoints when a checkout
could run host Git. That row opened `Page::Checkpoints`; its render, keyboard,
selection and restore confirmation lived in `menu/checkpoints.rs` and its
timeline. This cut removes that action, page, dialog code, English UI messages
and the dialog-only tests. Its restore-result flash and messages have no
remaining UI initiator and are removed too. Ordinary workspace menu actions, notes and worktree
scripts keep their selected workspace identity. No file or daemon state was
read or changed by this cut.

The new headless menu test failed first because a local checkout still offered
Checkpoints. It passes after the cut and checks the Notes action and `w4`
target. Retained script-menu, mute-menu and page-title tests also pass.
Default and all-feature app checks pass. Format, file-size, sorted and i18n
checks pass (5,049 messages in 265 files). Package render/check and its 21
tool tests pass; that package check does not prove app conformance. The test
linker reports the existing macOS compact-unwind notice, and Cargo reports
`block v0.1.6` future compatibility. Native window, Windows/Linux, full
workspace and release checks did not run.

This is not full RM-GIT removal. `window/monitors.rs::update_checkpoints`
still watches agent turns, and the `agent_checkpoints` setting in `config.rs` plus
Settings > General still enables automatic Git snapshots. That path uses
`sheprd-vcs::checkpoint::Checkpoints` and its worker; the VCS module also
still contains list/restore view code with no UI caller. Those worker,
setting and VCS tails need a separate coherent cut. Other Git and
worktree routes remain. Inherited `website/site` checkpoint claims still
need a separate content audit; it is not a qualified Spaceport release.
The inherited root README still shows an upstream checkpoint popover image
and copy; T12 needs to qualify or replace that entrypoint rather than silently
editing the screenshot's description here.

At integration, T04 is `blocked` by G-LINK on main; this independent cut does
not close that gap. RM-GIT remains `in_progress`; A29/A30 remain `not_run`.
A33/A34 native visual and accessibility checks have not run.
