# T04: remove the native embedded-browser engine

Base: integration `30996ebbf`, under D58. This cut removes the remaining native
page engine after the page-entry, restore, popup, annotation, and control-route
cuts. It does not complete T04 or establish platform parity.

## Reachability and deletion

`browser::Pages` still compiled and was reached from `show_browser_tab_in`, the
window tick, rendering, font-size commands, macOS background cutouts and key
routing, and Windows overlay/tooltip handling. Although production web-page
entry points had been removed, these hooks still linked the engine. The
`browser` container also owns native tab IDs, split groups, layouts, Core
terminals and Herdr 1 navigation; deleting it would remove retained behavior.

Removed the engine's native/page/building/key/parking/stacking modules, private
local preview server, explainer kit, browser-file Markdown renderer, favicon
scripts, underlay, page toolbar/address field, page zoom, presentation hooks,
page-only platform probes, engine smoke fixtures, assets, and engine tests.
Native backgrounds now paint normally, and divider grips use the same centered
geometry for retained content on each platform. The shared tab/group container
and terminal/chat/file/service click handlers remain. Rich chat's own Markdown,
math and diagram owners remain separate from the deleted browser-file renderer.

Removed direct `lb-wry`, `objc2-web-kit`, `ammonia` and `percent-encoding`
dependencies. Retained macOS/Windows native handle users import the existing
workspace `raw-window-handle` 0.6 directly; it is a normal macOS dependency.
The root Wry/native-page-only errors are gone. The lockfile changes from 1,021
to 914 package versions, with 107 removals and no added or upgraded versions.
`lb-wry`, `objc2-web-kit`, WebView2 and WebKitGTK are absent from it.

Control protocol 35 removes `browser_tabs`, the browser-specific status suffix,
retired page tab kinds and the unused page-zoom type. Pending Core tabs report
`core_terminal`; retired blank/web/local tabs are neither listed nor targetable.
Only validated saved-state payloads remain to discard old pages while reserving
their IDs. Existing negative restore tests cover that boundary. No engine stub,
replacement runtime, new transport, personal daemon, or Herdr source edit is
part of this cut.

## Regressions found while retaining native tabs

Layout tests now use native Devices content where they formerly used blank
browser tabs. Their first run passed 96 of 98 tests: two new Devices fixtures
had omitted the normal overview-state initialization. Applying the existing
initializer fixed both; the browser/shared-layout filter then passed 98/98.

The broader retained run exposed a real scroll-reveal failure. A newly opened
tab spent the four-frame retry budget while growing, then could accept a
visible partial-width previous layout as its final geometry. Its close button
could stay clipped. The old title-growth test also used a blank browser tab,
whose label stayed `New Tab` even after its title changed. The replacement
uses a native Core terminal reference and asserts its title changes.

The deterministic `growth_does_not_spend_the_final_layout_reveal_budget` test
failed before the fix at its first non-growing reveal. The fix preserves the
bounded retry budget for the final layout and requires its full measured
width before declaring it visible. The native title test had three failures
in five diagnostic runs before the fix; all five diagnostic runs passed after
it. No assertion tolerance or retry wrapper was added. Temporary geometry
prints were removed.

Fresh-eyes review found one additional retained-path issue before commit: a
Core-group tab with no terminal reference yet would fall into an empty body
when the browser placeholder disappeared. The new headless pending-Core test
failed with no native connection-state view. The render match now routes that
exact Core reference through the existing native terminal renderer, which
already shows its connecting/unavailable state. It adds no replacement page
engine or runtime action.

## Verification

Pinned Rust 1.99.0 on macOS. Final Rust checks used the branch-private APFS-cloned
`target` after shared-target cross-worktree artifacts were identified. Only the
clone's app/control package artifacts were cleaned. Initial cache misses for
locked Cocoa/Core Graphics packages were fetched without changing versions;
the final commands below ran `--locked --offline`.

- `cargo +1.99.0 check --locked --offline -p sheprd-gpui --all-targets --all-features`:
  passed, no source warnings.
- `cargo +1.99.0 test --locked --offline -p sheprd-gpui --bin spaceport -- browser:: core_terminals:: window::links:: window::file_links:: chat::tests::embeds:: chat::rich:: chat::tests::visual:: control::server::tabs:: control::client::tabs:: window::tab_strip:: endpoint::lifecycle_tests::group_page_move::`:
  395 passed, eight existing opt-in tests ignored. Covers retained Core
  panes/splits/chat, Herdr 1 group navigation, native tabs and tear-off, terminal
  links, external files/services, explicit chat HTML opening, rich Markdown and
  math, and the CLI tab schema/targeting.
- `cargo +1.99.0 test --locked --offline -p sheprd-control-protocol`:
  63 passed, including rejection of retired page kinds and native tab replies.
- `python3 -m unittest scripts/release/tests/test_webview_source_absence.py`:
  five passed. Engine/dependency checks failed before the cut; the added schema
  exclusion failed before removing the retired outward page kinds.
- `just format-check`, `just check-file-size`, `just check-sorted`,
  `just check-i18n`, and `git diff --check`: passed.
- `python3 -m unittest discover -s scripts/release/tests`: 246 tests ran,
  with eight existing environment-dependent skips and no failures.
- Specification package `tools/spec.py render`, `tools/spec.py check`, and
  `python -m unittest discover -s tools -p test_spec.py`: passed, 21 tool tests.
  These are package-integrity checks, not application acceptance.

Clippy is not green. The all-target/all-feature `-D warnings` invocation stops
at the pre-existing `match_like_matches_macro` in
`sheprd-client/src/discovery.rs:120`. A `--no-deps` invocation for the changed
packages stops at three existing constant assertions in
`protocol/app/tests/removed_activity_panels.rs`,
`protocol/app/tests/removed_project_open.rs`, and `protocol/hosts/tests.rs`.
These unrelated diagnostics were not suppressed or changed. Cargo also reports
the existing `block 0.1.6` future-compatibility notice; test linking emitted the
existing macOS compact-unwind `__eh_frame` warning.

## Limits

No native desktop, real runtime, personal state, Windows/Linux build, release
bundle, or screenshot was exercised here. The eight ignored tests were not run.
The production build/packaged dependency inventory and native visual/input QA
remain integration/platform work. The shared runtime adapters and G-LINK are
unchanged. T04 stays blocked, A28 stays `not_run`, and native/platform acceptance
is unclaimed.
