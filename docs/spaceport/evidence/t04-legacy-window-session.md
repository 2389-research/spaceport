# T04: remove named-session `window new`

At source baseline `2b62dc21`, the CLI parsed `window new --session NAME` into
`WindowCommand::New { session }`; the control request carried the optional
name, and `control/server/app.rs` used it to create
`ConnectTarget::Session { development: false }`. That path could select a
Herdr 1 session by name, separate from Core machine identity. The plain
`window new` path called `new_window_target`, which first uses an explicit
`--window`, then the caller daemon socket, then the frontmost/default target.

The CLI regression initially failed because `--session legacy` still parsed
successfully. The protocol regression initially failed because a direct
`window.new` request with a `session` field still deserialized. Removing the
field and changing the wire variant to an empty struct made the latter refuse
the extra field: Serde's tagged unit variant accepted it despite
`deny_unknown_fields`. Control protocol version 15 records the changed request
shape; old clients also sent `session: null` for plain `window new`, so those
requests require a matching version-15 client. The old named-session request
is rejected, not retargeted. The retained
CLI scenario checks `window new --window 42` still carries window ID 42. The
server's `new_window_target` function and the default connection bridge were
not changed.

`cargo +1.99.0 test --locked -p sheprd-control-protocol removed_legacy_window_session`
passed 1/1. `cargo +1.99.0 test --locked -p sheprd-gpui --bin sheprd
removed_legacy_window_session` passed 1/1. Both used `CARGO_INCREMENTAL=0` and
the coordinated warm target. The GUI link emitted its existing macOS
`__eh_frame` warning and Cargo's `block v0.1.6` future-compat notice; no new
source warning remained. Native desktop, Windows/Linux, and full workspace
checks were not run for this slice.

`just format-check` passed after formatting one test line. `just
check-file-size`, `just check-sorted`, `git diff --check`, and the package-root
`uv run --with jsonschema==4.26.0 python tools/spec.py check` passed. The spec
checker verifies package integrity, not application behavior.

RM-LEGACY stays in progress. The default Herdr 1 bridge and startup,
explicit socket route, in-app named-session UI, and legacy snapshot consumers
still need removal or Core-backed replacement. This change does not establish
Core machine parity for window selection.
