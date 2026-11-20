# Windows identity and portable endpoint coverage

Date: 2026-11-17. Scope: `1wwk` under `9s44`.
Doctor Biz authorized fixes and validation without commits, publication, or installation on this host.

## Identity and state

The root agent corrected Windows state root selection.
An explicit `XDG_STATE_HOME` still wins.
Otherwise Windows uses `LOCALAPPDATA`, then `USERPROFILE/AppData/Local`, before a shell's `HOME` fallback.
Desktop and Git Bash launches therefore select the same default state root.
The compiled build namespace still separates development, worktrees, and installed previews.
No state migration occurs.

Independent review inspected the actual selector, platform gates, callers, and tests.
No material state-selection findings remained.
Root evidence includes two failing baseline policy tests, 11 passing path tests, and Windows client Clippy.

The installer now owns GUID `{E3008A35-FD63-4B85-A416-3FC47E25A820}`.
It uses the Spaceport name, startup label, and `%LOCALAPPDATA%\Programs\Spaceport` folder.
The compiler and native smoke read AppUserModelID from the app's `development-app-id` file.
They validate that value before passing it as an ISCC define.
The installer removes its own notification and startup registrations on uninstall.
It no longer migrates or deletes upstream shortcuts.
Existing URL handlers and the inherited upstream `site/install.ps1` remain separate.

The native smoke checks shortcut target, model identity, display name, startup preservation, and notification-key removal.
It refuses an account with existing Spaceport installation state.
Archive and setup filenames keep their existing `Sheprd-VERSION-TARGET` contract.
Production-release refusal and signing gates remain unchanged.

## Portable test fixtures

The endpoint fixture now serves real Unix sockets or native Windows byte pipes.
Windows uses the daemon's actual path-to-pipe mapping and the production client connector.
The server owns an unsplit handle converted safely into a standard file.
Nonblocking writes and exclusive-reader peeks bound fixture I/O to three seconds.
Closing the exact accepted handle signals a disconnect without an unbounded pipe-drop flush.
These peers simulate daemon frames; they are headless integration tests, not live Herdr end-to-end tests.

Connection generations, shutdown, reconnect, focus fencing, notices, and mouse lifecycles share this fixture.
Wake tests also use the native listener and verify that Windows creates no socket file.
Unix clipboard and AppKit physical-key fixtures retain their platform gates.

All 38 individually Unix-gated link cases now compile on Windows.
They use native file paths and platform-correct local and localhost file URLs.
The three file freshness checks also run on Windows.
The broad non-Unix dead-code allowance is removed.

## Verification

The compiler identity regression failed before the installer changes.
Its arguments omitted the app's model identity and its smoke queried upstream startup state.

- Windows packaging Python scope: 24 passed.
- Rebrand identity Python scope: three passed.
- Inherited PowerShell installer scope: two passed; two native Windows checks skipped explicitly.
- macOS headless link scope: 58 passed.
- Final remote-drop scope: seven passed with real transport and publication barriers.
- Final macOS headless endpoint scope: 169 passed.
- Final Windows GPUI all-target/all-feature Clippy: passed.
- Final macOS GPUI all-target/all-feature Clippy: passed with the validated Sparkle SDK.
- `cargo fmt --all -- --check`, `git diff --check`, file-size, and sorted checks: passed.

The Python checks use the repository's existing standard-library harness.
No uv skill exists in this session, and this harness has no `pyproject.toml`.
The existing mock ISCC and registry fixtures check argument and smoke contracts only.
They do not prove that Inno Setup compiled or installed a working package.
The CI `windows-package` job remains the actual install/reinstall/uninstall end-to-end oracle.
Final fresh-eyes review found the wizard's PATH task still named the upstream executable.
It now names `spaceport`; the installer message validator passes with that correction.

## Remaining validation

At the original checkpoint, native Windows pipe, link, and installer execution remained pending Windows CI.
No personal Windows installation, registry, SSH credentials, or daemon was used locally.
Native keyboard and performance checks remain pending at Doctor Biz's request.
Full workspace CI remains deferred.

The first expanded macOS endpoint run passed 161 tests and exposed eight fixture-contract failures.
Six keyboard fixtures depended on a removed interceptor; their owner supplied the actual focused render scope.
The original Unix fixture comparison reproduced both remote-drop failures: five passed and two failed.
Its log remains at `/tmp/spaceport-qnz9/1wwk-remote-drop-original-fixture.log`.

The reader now always publishes terminal state on exit.
The gap tests therefore hold the real inbox lock while the stopped handle is already visible.
The replacement test no longer clears an outage on a dead handle or clones cleared state.
It closes the real peer and reconnects the same private address.
It withholds the welcome until the UI observes the new generation.
Then it sends a welcome, snapshot, and valid surface frame over the actual transport.
The final seven remote-drop tests pass in `/tmp/spaceport-qnz9/1wwk-remote-drop-final.log`.
No production Link, reader, retry, or daemon startup policy changed during this repair.

The root agent independently reviewed final installer ownership, pipe bounds, and repaired lifecycle tests.
No material findings remained, conditional on the final queued scopes.

## Native Windows closure

Source `46738c99c92851245887114dc174d95221cf95b1` passes all four native Windows workspace legs.
Both all-feature GPUI suites report 4,322 passed, zero failed, and six private fixtures ignored directly.
The native runs include state-root selection, 135 portable endpoint cases, and all 41 link cases.
Canonical document paths and remote-prefix refusal also pass on both architectures.

Verified [x64 all-feature logs](https://github.com/2389-research/spaceport/actions/runs/44452942264/job/135321241319)
and [ARM all-feature logs](https://github.com/2389-research/spaceport/actions/runs/44452942264/job/135321241301).
The [installer job](https://github.com/2389-research/spaceport/actions/runs/44452942264/job/135321241179) also passes.
It executes the actual CLI and install/reinstall/uninstall smoke checks.
Those checks preserve shortcut identity, startup state, and owned uninstall cleanup.

Kata `1wwk` closes with event `9781` (`01M7RZFXYM5BMTJN1HPAND0198`).
A fresh issue read confirms its closed status.
This closure does not qualify screen-reader speech, foreground Mac input, or interactive release performance.
