# T05: join existing Core from a plain local window

At source baseline `f328e684`, a plain launch still made a Herdr 1
`ConnectTarget::Local` endpoint (`cli.rs`, `window/construct.rs`). The Core hub
connected only for restored Core tabs, an explicit Core terminal, or the
missing-Herdr fallback. The app-global Core observation starts only after a
hub connects, so a normal window with Herdr 1 present did not read Core's
authoritative terminal and workspace metadata even if Core was running.

The headless local-window regression first failed with Hub Idle where it
expected a join. The explicit-socket regression already passed with Hub Idle.
The local endpoint now requests `Start::Join` on its existing Core hub during
the window tick. `hub.rs::join` returns a failed connection before binary
lookup or `server start` for that mode. The hub starts at most one connection
attempt at a time and waits two seconds after a failed join; the app-global
observation remains the only metadata subscription. The call runs on the
hub's worker, not during UI I/O. An explicit socket endpoint does not request
the new join. The headless test uses a private typed connector, so no personal
Core socket or service is opened.

`cargo +1.99.0 test --locked -p sheprd-gpui --bin sheprd
window::tests::core_join` passed 2/2 with an isolated Cargo target. The GUI
link emitted the existing macOS `__eh_frame` warning and Cargo reported the
existing `block v0.1.6` future-compat notice. The matching production
`cargo +1.99.0 check --locked -p sheprd-gpui --bin sheprd` passed, as did
`just format-check`, `just check-file-size`, `just check-sorted`, `git diff
--check` and the package-root `tools/spec.py check`. No native desktop,
Windows/Linux, full workspace or live Core scenario ran in this slice.

This is metadata visibility for an already-running Core, not default Core-only
window parity. The Herdr 1 bridge, snapshot-backed workspace/terminal input,
offscreen selection, and notification routes still run. The Core-first UI
replacement needs those retained reads and actions to move to Core-backed
state before the plain-launch bridge can be retired. T04, T05 and RM-LEGACY
remain in progress; A32 is not yet verified.
