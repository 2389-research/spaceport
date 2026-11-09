# T01 diagnostics identity and crash test

This bounded cut starts from integration revision `e44f4de5c`. It keeps the
diagnostics writer, bounded queue, rotation, crash hook, and CLI log reader.
It changes the log and export filename to `spaceport.jsonl`, the rotated name
to `spaceport.1.jsonl`, and the crash report and panic/crash record labels to
Spaceport. The diagnostics README section now names the Spaceport state path.
The app continues to use `sheprd_client::local_path::state_dir()` as its one
state root. No existing state directory or installed file was read, moved,
deleted, or rewritten by this change.

The child crash test had a fork regression: it looked for reports in
`<XDG_STATE_HOME>/herdr/gpui/logs`, while the production state root had already
moved to `<XDG_STATE_HOME>/spaceport`. With test expectations changed first,
`cargo +1.99.0 test -p sheprd-diagnostics crash::tests --locked --offline
--quiet` failed the old crash report label and the old log filename; the
parent could now find the report at the correct state root. After the code
change, the full diagnostics suite passed 23 tests, with its deliberately
crashing child entry ignored. The desktop log reader filter passed 3 tests,
and the diagnostics CLI crash-report filter passed 2 tests. The GPUI test
link emitted the pre-existing macOS compact-unwind warning.

This is source and isolated child-process evidence on macOS. Native app
launch, Windows minidump behavior, Linux log paths, package identity, and
side-by-side install were not exercised. A42 remains `not_run`, T01 remains
`in_progress`, and G-RELEASE remains open. The broad upstream crate/package
names and other stale sections of the old crate README need their own route
review; this cut does not claim full product rebranding.
