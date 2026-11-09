# T03 selected Core group connection overlay

Source base: `b51283b6d`. This cut changes the retained window's reconnect
overlay reader. A selected Core presentation group now uses its existing Core
workspace status and join error UI; a legacy endpoint outage cannot cover it
or offer a retry against Herdr 1. The old overlay still appears when the
window leaves the Core group and shows a legacy view.

## Reproduction and verification

The headless regression selected a fresh `hub-a` group while the legacy local
endpoint held an outage. Before the code change,
`core_group_does_not_show_the_legacy_reconnect_overlay` failed at the expected
`!view.reconnecting()` assertion. After the guard in `window/reconnecting.rs`,
the focused test passed (1/1). It also checks that the legacy overlay remains
visible before selection and after leaving the Core group. Both runs used:

```sh
CARGO_TARGET_DIR=/Users/harper/Public/src/2389/spaceport-t00/target \
  CARGO_INCREMENTAL=0 cargo +1.99.0 test --locked --offline \
  -p sheprd-gpui --bin spaceport \
  core_group_does_not_show_the_legacy_reconnect_overlay
```

The existing macOS compact-unwind linker warning and `block v0.1.6` future
compatibility notice appeared; this change added no warning. The RED run
first found a test-only private-method access error; that assertion was
removed before the expected failing assertion was observed.

The Core status and join control already render from the observed document
and `Hub` in `core_terminals/workspace_ui/render.rs`. The new guard does not
claim Core is connected, create a fallback endpoint or change terminal
lifecycle. No real Core, Herdr 1, native drag, or personal daemon was used.

## Remaining constructor gate

`SheprdWindow::new` still creates `endpoints[0]` and calls legacy `reconnect`;
UI New Window and CLI `window new --window` still copy its legacy target.
This slice removes one false reader dependency but does not make a Core-only
window constructible. The next cut needs an exact expected Core hub identity
through app launch and an attach-only `Start::Join` path for additional
windows, then a Core-only initial `Start::IfMissing` path using separately
installed Core. T03 remains in progress. A09 and A24 remain not run.
