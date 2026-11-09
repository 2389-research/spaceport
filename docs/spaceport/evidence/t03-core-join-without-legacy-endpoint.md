# T03: selected Core group joins without a legacy endpoint

Source base: integration `78dc8177c`. The Core Hub already binds a selected
`CoreGroupKey` to its persistent hub ID and forces `Start::Join`. The window's
`poll_core_terminals` still scheduled that join only when its legacy endpoint
list contained a local Herdr 1 target. That prerequisite was unrelated to the
Core group and prevented its own reconnection path from running without the
legacy target.

The focused headless test uses a fresh document for hub `hub-a`, selects its
exact Core group, then clears the fixture's legacy endpoints before polling.
It also clears the fixture's legacy snapshot, which a Core-only window would
not have.
Its connector records the requested start mode and returns `NotInstalled`, so
it cannot dial or start a personal runtime. Before the change, the test failed
with zero calls instead of the expected `Start::Join`. The production change
allows a selected Core group to request the existing Hub join, while the
preexisting plain-local metadata route remains. The Hub's exact-ID binding
still refuses a foreign handshake and converts start requests to join-only.

The zero-endpoint probe then exposed a render crash in
`surface_activation_ready`: it indexed the selected legacy endpoint to decide
whether input was ready. That query now returns false when there is no
endpoint. With the fixture's old legacy snapshot still set, the tab strip
also indexed that missing endpoint while reading mute state; clearing that
stale fixture data made the probe match Core-only conditions. This does not
prove every retained UI route can render or accept input without an endpoint.

This is a Core reconnection prerequisite, **not** a Core-only window launch.
The fixture constructs a legacy endpoint before clearing it; it does not
exercise `SheprdWindow::new`. That constructor still creates an
`endpoint::Catalog` and a legacy endpoint, reads `endpoints[0]` for preferences,
and calls `reconnect` (`window/construct.rs:87,184-190,302-310`). A local
legacy target can start Herdr 1 through `daemon::connect`. `tick` still calls
legacy endpoint polling and many retained readers index the selected endpoint
(`window.rs:493-513`, `window/lifecycle.rs:64`, `window/input.rs:234`,
`window/status_bar.rs:259-260`). New Window and CLI `window new` still route
through `ConnectTarget` (`app.rs:107-151,171-204`,
`window/commands.rs:385`, `control/server/app.rs:172-249`).

The next coherent constructor cut must launch a Core-only window without
creating or reconnecting a legacy endpoint, attach additional windows with
the expected persistent hub ID and `Start::Join`, and keep render/input/chrome,
tab return, CLI targeting and window listing valid without endpoint index
zero. A headless constructor regression must observe no legacy endpoint,
bridge or Herdr 1 start. Native drag/focus and a private installed-Core fixture
remain required. No Core RPC, transport manager or placeholder endpoint was
introduced here. T03 stays `in_progress`; A09 and A24 stay `not_run`.

`cargo +1.99.0 test --locked --offline -p sheprd-gpui --bin spaceport
selected_core_group_rejoins_without_a_legacy_local_endpoint` passed 1/1
with the private connector and a zero-endpoint headless render. The existing
exact-hub test passed 1/1, and the two existing plain-local/explicit-socket
join tests passed 2/2 by direct execution of the just-built test binary. The
first green attempt was blocked by stale shared Cargo metadata from a second
worktree; after scoped `cargo clean -p sheprd-control-protocol -p sheprd-gpui`,
the source compiled. `just format-check`, `just check-file-size`,
`just check-sorted`, and `git diff --check` passed. Package integrity/schema
validation passed, as did its 21 tool tests. Cargo still reported the prior
macOS compact-unwind linker warning and `block v0.1.6` future-compatibility
notice. No live Core, native window, other platform or full workspace suite
was exercised by this cut. Those checks and A09 remain open.
