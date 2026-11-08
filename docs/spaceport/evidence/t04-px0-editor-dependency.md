# T04/RM-IDE: px0 embedded editor reachability

Read-only source audit at `a33bfcb7`. No app code, Core checkout, saved tab
file, editor process, workbench address or user configuration was accessed or
changed. T04 remains blocked by G-LINK, RM-IDE stays in progress, and A29
remains not run.

## Live routes

- `crates/sheprd-gpui/src/app.rs` installs `px0::Launcher` at normal startup.
  The launcher watches `[px0]` config and starts its worker lazily when an
  editor tab needs a workbench (`px0.rs`, `px0/{settings,worker,server}.rs`).
  `browser/view.rs::poll_browser` calls `ensure_px0_page` on window ticks;
  `browser/px0.rs` can bind a saved tab to a loopback page and request the
  launcher. Removing only the command or button would leave this route live.
- `Command::OpenPx0` in `sheprd-commands/src/lib.rs` reaches the window
  command, Terminal menu and group menu. The private `sheprd px0 [open]`
  command reaches `TabOp::Px0` through `cli/control.rs`, `control/client/tabs.rs`,
  `control/server/tabs.rs` and `control/server/tabs/arrive.rs`. The server uses the shared
  `open_there` caller/workspace check. The protocol is at version 25;
  removing this wire variant needs a versioned negative decode test while
  preserving the other exact-target tab operations.
- `settings_window/{section,catalog,controls,controls/px0,persistence}.rs`
  gives px0 its own Settings section. `config.rs` parses `[px0]`; the example
  config, `docs/cli.md`, the managed skill and the GPUI README advertise
  it. `locales/en-US/{browser-px0,settings-px0,error-px0,commands,menus}.ftl`
  and `settings.ftl` contain its labels. The `icons/code.svg` path is used by
  the px0 tab and Settings section, but asset ownership needs a final
  post-cut reference check before deletion. There is no px0 Cargo crate or
  package companion binary: its launcher searches for an installed program.

## Shared state boundary

`browser/editor.rs::Editor::Px0` is the sole live editor variant.
`browser/store.rs::Place` serializes it as `"px0"` in `browser-tabs.json`;
`Store::with_saved` currently keeps it. The same Store and `TabId` sequence
hold page, Devices and Core terminal tabs and their groups. The prior VS Code,
review, source and Orchestrator migrations parse retired entries, discard
them in memory and reserve their IDs before filtering; px0 needs the same
proved rule so a saved group pick cannot acquire a new tab with an old ID.
`control/server/tabs.rs::kind` reports every editor place as
`TabKind::Editor`, and list/focus/close share this Store. Deleting `Editor` or
`Place::Editor` while saved px0 entries remain would reject the whole tab
file and risk losing retained Core/page tabs. The pending T03 Core tab-store
owner must be settled before changing that common parser and control mapping.

Generic `WebUrl`, webview page storage, layout/group movement and `Store`
also serve other existing page and Core/chat tab paths. The deliberate local
file route in `window/file_links.rs::opened` and rich-chat file links use the
system handler, not px0; they must survive. No px0-only worker, settings or
UI entry can be removed now without leaving a broken live path or silently
dropping saved state. A source-only command cut could remove one route, but
it would not remove the embedded editor and would overlap the shared T03
tab protocol seam.

Next coherent cut: first test loading old `"px0"` tabs beside Core/page tabs,
discard px0 in memory, retain exact IDs, and reserve discarded IDs. Then
remove all px0 UI, command, Settings/config, tick/launcher, worker, locale,
docs and exclusive assets together. Reject stale CLI/config/wire inputs and
retain Core tab list/focus/close, system file opening and rich-chat code copy.
This audit ran no product or native tests; package integrity checks do not
prove the proposed migration or A29.
