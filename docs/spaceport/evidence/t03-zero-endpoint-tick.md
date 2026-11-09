# T03: full window tick without a legacy endpoint

Source base: integration `23df7f8c0`. This is a constructor prerequisite,
not a Core-only window launch. The headless fixture selects an exact
`CoreGroupKey` from a fresh namespaced document, then removes its legacy
endpoint and snapshot before calling the complete window `tick`. A private
Core join callback returns `NotInstalled`; the test cannot contact or start a
personal Core or Herdr 1.

The test first failed at `endpoint/polling.rs:211`: `poll_endpoints` indexed
the selected endpoint even when none existed. After a scoped empty-list
return, it failed at `window/links.rs:376`, where the legacy daemon link
poller did the same. The next run reached
`menu/version_mismatch.rs:74`, where the Herdr 1 handshake notice indexed
endpoint zero. The final run passed 1/1 after those legacy-only readers and
the independent session scan, legacy resize, and legacy focus reporting
handled an absent endpoint. Core polling still runs later in the same tick;
the prior exact-hub binding keeps its request join-only. The existing
plain-local and explicit-socket Core join tests passed 2/2, and 58 legacy
link tests passed by direct execution of the freshly built test binary.

The probe is deliberately narrower than a constructor or native UI test.
`SheprdWindow::new` still takes `ConnectTarget`, creates `endpoint::Catalog`
and an endpoint, derives chrome preferences from its legacy socket, and
calls `reconnect` (`window/construct.rs:21-34,87,184-190,302-309`). The
normal constructor also starts a shared-settings read from the inherited
view settings path (`settings_panel.rs:22-46`, `herdr_settings.rs:400-410`).
`app::open_window_placed` passes only `ConnectTarget` (`app.rs:107-151`);
UI New Window copies `endpoints[0]` (`window/commands.rs:383-388`), as does
CLI `window new --window` (`control/server/app.rs:172-249`). Core tab
tear-off still requires `browser_key`, which excludes Core groups, and its
`TornTab` carries a legacy target/session (`window/tab_tear.rs:18-68`).

The next complete cut must carry the exact hub/document/group through
window launch and attach additional windows with `Start::Join`, never a
legacy `ConnectTarget` or a fabricated endpoint. It must keep Core chrome
preferences under Spaceport's own local state, prove render and input with
an actual Core view, and recheck stale handoffs before hiding or returning
tabs. The zero-endpoint tick test does not prove CLI targeting, constructor
side effects, native focus/drag, chat, file links or terminal input. These
are client-side dependencies; no missing Core RPC was inferred. T03 remains
`in_progress`; A09 and A24 remain `not_run`.

Focused Rust check: `cargo +1.99.0 test --locked --offline -p sheprd-gpui
--bin spaceport selected_core_group_ticks_without_legacy_endpoints`, 1/1
passed with `CARGO_INCREMENTAL=0` and the shared warm target. The prior
macOS compact-unwind linker warning and pinned `block v0.1.6`
future-compatibility notice appeared. No full workspace suite, native
window, other platform or live Core ran for this cut.
