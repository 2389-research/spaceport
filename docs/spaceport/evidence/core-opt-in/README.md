# Herdr Core opt-in evidence

Decision D69 makes Core experimental and disabled until explicitly enabled.
The approved behavior is recorded in [the task note](../../notes/core-opt-in.md).

## Native Settings

The macOS `native_local_controls` fixture passed on November 13, 2026.
It uses private HOME, XDG paths and a private listening Core socket.
The app saved `false`, `true` and `false` through the real preference writer.
The startup configuration reader recovered each saved value.
The running app stayed disabled and showed the restart hint only for pending enablement.
The fixture also exercised missing-Herdr fallback with the old Core preference enabled.
The listener received no Core connection during either light or dark execution.

These images show real native windows at 960 and 680 logical pixels wide.
The narrow views keep the label, helper and restart hint readable.

| View | Before | Disabled | Pending enablement |
| --- | --- | --- | --- |
| Dark, 960 | [Before](before/core-dark-960-off.png) | [Disabled](after/core-dark-960-off.png) | [Pending](after/core-dark-960-pending.png) |
| Dark, 680 | [Before](before/core-dark-680-off.png) | [Disabled](after/core-dark-680-off.png) | [Pending](after/core-dark-680-pending.png) |
| Light, 960 | [Before](before/core-light-960-off.png) | [Disabled](after/core-light-960-off.png) | [Pending](after/core-light-960-pending.png) |
| Light, 680 | [Before](before/core-light-680-off.png) | [Disabled](after/core-light-680-off.png) | [Pending](after/core-light-680-pending.png) |

The before images use the General renderer from `a0d22fb5` with the current isolated fixture.
They compare the previous page layout, rather than claiming an entire baseline binary run.

The command used the existing shared Cargo cache and this worktree's capture directory:

```sh
HERDR_TEST_LOCAL_CONTROLS_CAPTURE="$CAPTURE_DIR" \
RUSTUP_TOOLCHAIN=1.99.0 CARGO_INCREMENTAL=0 HERDR_BUILD_PR_NUMBER='' \
cargo test --locked -p sheprd-gpui --features integration-test \
  --test live_gui native_local_controls -- --ignored --exact
```

`CAPTURE_DIR` must be an existing absolute directory.
`CARGO_TARGET_DIR` selected the existing display-copy cache during these runs.
Untracked logs remain under `target/core-opt-in/native-final.log` and `native-before.log` in the task worktree.

The native host-routing fixture also passed with Core disabled.
It covered real host events, resized glyphs and independent scrolling.
It excluded key-window qualification and used no personal daemon.
The task note records its exact command and result.

## Behavioral coverage

Regression tests cover defaults, explicit opt-in, invalid values and targeted preference round trips.
App-session tests cover pending enablement, pending disablement and consistency across windows.
Rendered UI tests cover hidden Core controls, search results, saved tabs and fallback status.
Control handlers refuse Core-only commands and direct Core tab operations while preserving references.
Existing Core behavior tests explicitly opt in and retain their original assertions.

Review traced connection and start ownership through the Hub guard.
It found two display bypasses: missing-Herdr status and the saved-tab renderer.
Both have regressions that failed on their respective render paths before the fixes.

## Limits

The native fixture proves disabled startup and real persistence without a personal daemon.
Enabled lifecycle behavior uses the existing isolated Core tests.
This task does not install a runtime or claim a live installed-Core end-to-end run.
Headless pseudo-locale tests do not prove native IME or OS input behavior.
Linux, Windows and full workspace validation remain CI work.
Windows CI remains paused under D66.
No release or distribution build was created.
