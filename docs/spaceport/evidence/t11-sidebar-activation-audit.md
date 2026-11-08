# T11/G-PLATFORM: macOS sidebar decoy activation audit

Source and recorded-log review at `b499b112`; no native run or app change in
this audit. The fixture launcher in `crates/sheprd-gpui/tests/live_gui.rs`
starts `sheprd --sidebar-test` with a private sandbox and no daemon for
`native_sidebar`. The recorded follow-on run used that route after a paint
tolerance change; it is not a qualification run of the current app.

The [follow-on log](t03-native-sidebar-tolerance.log) reports 72 Classic
frames, Superset, Orca and Minimal layouts, and native dialog checks before
`sidebar_hosts` fails. It reports the same decoy deadline failure twice:
`key=false, resized=true, app_active=false, app_hidden=false`. Thus the main
window's requested viewport settled, the decoy did not become key, and
AppKit said the test app was inactive and visible at the deadline. The log
does not identify the frontmost process or why activation failed. The second
copy comes from the test's printed `gui.log` tail, not a second test attempt.

`crates/sheprd-gpui/src/smoke_hosts.rs::sidebar_hosts` opens a decoy window,
prepares synthetic endpoints, calls `cx.activate(true)` and
`window.activate_window()`, then waits up to three seconds for both the
decoy's AppKit `isKeyWindow` and the main viewport resize. Its
`sidebar/native_tests.rs::Target::is_key` reads the exact native window's
`isKeyWindow`. It does not call a personal daemon. The activation request,
wait and native target code, plus the `live_gui.rs` launcher, have no diff
from inherited commit `3b9800ff` to this audit's base. That is evidence
against a direct fork edit to this test path, not proof that the surrounding
fork cannot affect app activation.

The [pinned upstream run](t00-upstream-native-sidebar.log) and the
[initial fork run](t00-native-sidebar.log) both stopped at the earlier Orca
glyph-bound assertion, before decoy activation. There is therefore no
upstream decoy outcome for comparison. Current evidence cannot classify the
failure as an inherited harness/desktop focus condition or a fork regression.
Changing the timeout or forcing `makeKeyWindow` would hide that unknown.

Next native check: run the same paint-tolerant sidebar fixture at pinned
upstream and current fork revisions on one otherwise idle macOS desktop,
with separate private fixture sandboxes. At activation request and deadline,
record the test process ID, frontmost app process ID, NSApplication activation
policy and active/hidden state, and each fixture window's key/visible state.
Then compare both outcomes before changing activation code. Keep the exact
window routing and resize assertions. No screenshot, release package,
personal app, daemon or Core state was used here. G-PLATFORM and T11's native
acceptance remain open.
