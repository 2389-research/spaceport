# T04 page-only tab socket operations

Source base: integration `c33ffb9cd`. The named browser CLI was already gone,
but a caller could still send page-only `tab` operations directly to the
control socket. This cut removes navigate, back, forward, text, screenshot,
zoom, and filtered page listing from the tab wire and server. The text and
screenshot helpers, page capture module, and page-only replies are removed.
`tab` list/focus/close and Devices keep their existing targets. The control
protocol rises from 33 to 34. The pure `ZoomChange` type stays for the native
page UI until that UI is removed.

The new negative wire test failed 0/1 on `list.pages` before the cut. Its
first post-change run still failed: Serde ignored the extra `pages` field on
a unit `List` variant even with `deny_unknown_fields`. Making `List` an empty
struct variant makes that old request fail decoding. The corrected negative
test and full control-protocol suite passed 63/63. Retained headless tab
server tests passed 9/9 and client formatting tests 2/2. The app all-target,
all-feature check passed with pinned Rust 1.99.0 and `--locked --offline`;
`just format-check`, `check-file-size`, `check-sorted`, `check-i18n`, package
integrity and its 21 tests, and `git diff --check` passed. The only Cargo
notices were the inherited macOS compact-unwind linker warning and pinned
`block v0.1.6` future-compatibility warning.

This is a socket reachability cut, not A28 completion. Page tabs, the native
webviews, and `wry` remain in the product. The system URL opener remains
available; no live forwarding, native UI, Core daemon, Windows or Linux check
ran here. T04 remains blocked by G-LINK and A28 remains `not_run`.
