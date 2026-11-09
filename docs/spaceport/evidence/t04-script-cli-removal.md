# T04 worktree script CLI and control removal

Source base: integration `78dc8177c`. The inherited `spaceport scripts`
family listed `.herdr/worktree.toml` through a local `git rev-parse` child and
could ask the app to run setup or run scripts through `work.script`. SP-015
excludes automation editing and execution and Git/worktree controls.

The CLI family, local Git lookup, script listing, wire variant, server handler,
and script-only error variants are removed. Control protocol 32 rejects both
former `work.script` kinds. Notes and notification mutes keep their typed
control requests and exact caller targeting. The script-only server helper
for selecting a connected legacy workspace is also gone. The app's inherited
worktree script menu and its implementation remain live; this cut does not
complete T04 or A29/A30.

## Reproduction and checks

The new wire test first failed because `work.script` with `setup` still
deserialized. After the removal, all 61 control protocol tests passed,
including malformed request rejection and notes/mute round trips. The CLI
parser test now requires `scripts`, `scripts list`, `scripts setup`, and
`scripts run` to return `UnknownOption` before starting GPUI; its focused
app parser filter passed 10/10. The first app build reported an unused script
config re-export, which was removed. An all-target, all-feature workspace
`cargo check --locked --offline` passed with no new source warning. The real
CLI integration suite passed 20/20, including a new test that the removed
commands exit before any app or socket startup. The
broader app control filter passed 108/108 after the separate T01 control-copy
fix described in `t01-control-protocol-copy.md`.

`just format-check`, `check-file-size`, `check-sorted`, and `check-i18n`
passed; i18n checked 3,338 used messages in 208 files. `git diff --check`
passed. The inherited macOS compact-unwind linker warning
and `block v0.1.6` future-compatibility notice remain. Native macOS, Windows,
and Linux tests did not run. No installed Core, user daemon, personal
checkout, or user state was changed.
