# T03: Core-only additional window constructor

Source base: integration `72d2eeec9`. This cut adds a typed Core launch to
the existing window constructor and routes the retained UI **New Window**
command through it when a Core group is selected. The launch carries the full
`CoreGroupKey`: persistent hub ID, machine ID, document name and group UUID.
The current watched Spaceport group document must be fresh and match that
key before GPUI creates the window. The constructor checks it again after
opening. A wrong hub, missing group or stale document refuses the open.
The current local observation permits only `machine_id: None`; remote native
group selection remains a separate parity gate.

The Core branch creates no legacy `Endpoint` or `ConnectTarget`, leaves the
legacy catalog inactive, and skips `reconnect` and the Herdr 1 shared-settings
reader. It keeps client-owned chrome preferences in a Core-ID-keyed file under
Spaceport state. The additional window binds the exact hub ID and requests
`Start::Join` through the existing Hub, which cannot start a replacement Core.
The legacy constructor remains temporarily for unconverted readers and
launches. No Core repository or personal runtime state changed.

The headless constructor regression uses a private join callback that asserts
`Start::Join` and returns `NotInstalled`. It verifies the additional window
has the same selected group, no legacy endpoints, and a completed join
attempt. A second regression supplies a different hub ID and verifies refusal
before another window opens. A preference-path test keeps Core and legacy
chrome files distinct. The wrong-hub test first exposed a preflight bug:
GPUI painted a zero-endpoint window before a post-open refusal, and the
legacy reconnect overlay indexed endpoint zero. The final route validates
the fresh group before `open_window` and shares that lookup with the window
selection check.

This cut does **not** prove native rendering, terminal input, CLI `window new`,
Core tab tear-off or remote native group parity. CLI still reads
`endpoints[0]`, and tear-off still uses a legacy launch target. Those are
client-side follow-ups; no missing Core RPC is inferred. T03 remains
`in_progress`; A09, A24 and A33 remain `not_run`.

RED: the first focused test failed to compile because
`open_additional_core_window_then` did not exist. After the first constructor
implementation, the wrong-hub regression failed at
`window/reconnecting.rs:24` because GPUI painted a zero-endpoint window
before post-open refusal. Both failures led to the preflight check.

Final focused checks, with `CARGO_INCREMENTAL=0` and the shared warm target:
`cargo +1.99.0 test -p sheprd-gpui --locked --features integration-test
--bin spaceport app::tests::core_window` passed 3/3 (two new tests and the
existing Core window-list test). The same command filtered to
`core_chrome_path_is_stable_and_distinct_from_socket_chrome` passed 1/1.
The existing legacy New Window, zero-endpoint tick and Core rejoin tests
passed 1/1 each. `just format-check`, `just check-file-size`,
`just check-sorted`, `git diff --check`, and package-root `tools/spec.py
check` passed. The spec checker verifies package integrity, not application
conformance. The shared Cargo target first reused a control-protocol
artifact from another worktree; a scoped `cargo clean -p
sheprd-control-protocol` rebuilt the pinned source before final tests.
`cargo +1.99.0 check -p sheprd-gpui --locked --bin spaceport` passed with
default features and no new app warning after gating a fixture-only import.
The prior macOS compact-unwind linker warning and `block v0.1.6`
future-compatibility notice remained. No live Core, native window, full
workspace suite, Linux or Windows test ran for this cut.
