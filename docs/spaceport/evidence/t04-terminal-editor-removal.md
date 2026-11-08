# T04/RM-IDE: terminal editor pane removal

At base `ee75a851`, `editor.rs` used a local pane split and typed a shell command from clicked terminal or rich-chat file paths. It could start a private Neovim listener and reuse that pane. `review/view/keys.rs` also invoked it on `e`. `open_files_in` and `editor_command` selected that client path. This was client-owned editor integration, separate from Core terminal tabs and the retained checked system file opener.

This cut removes the client pane launch, Neovim job/socket, config, error and locale surfaces, and the review `e` route. Explicit local file clicks now use `window/file_links.rs::opened` through the system handler. That check still resolves symlinks, rejects network paths, opens only safe documents or plain folders, opens a parent folder for executables or unknown types, and fences asynchronous answers by selection epoch. Rich-chat code copy/rendering and px0 tabs are unchanged. Stale editor config keys become visible unknown-key diagnostics; no personal config is read or rewritten. Old saved tab IDs and their parse-only retired source handling are unchanged.

`Command::EditScrollback` stays: `sheprd-commands/src/lib.rs` sends the advertised daemon method `PaneEditScrollback` and `state.rs` only tracks its availability. It does not start this client's editor pane or use its removed config. The separate px0 embedded workbench and local CLI search index remain reachable; T04/RM-IDE and A29/A30 stay open.

The config absence test was added while the two editor settings were still parsed as known fields; a source check confirmed that initial failing condition. Cargo was paused then because free disk was below the 8 GiB floor, so an executable red run was not made. The review-key absence test rejects plain `e`; the old `review_key` arm would have consumed it. The retained file-link test now checks an ordinary click and an Alt click use the system opener without a pane connection.

On macOS, `cargo +1.99.0 check -p sheprd-gpui --bin sheprd --locked` passed after removing an orphaned error attribute and locale match. The GPUI test binary compiled; its existing compact-unwind linker warning and Cargo's `block v0.1.6` future-compatibility notice remain. Focused GPUI filters passed: retired editor config 1, checked file links 7, rich-chat links 5, plugin link fallback 3, terminal link cases 41, retired answers 2, review keys 3, retired source tab 1. `sheprd-commands` passed 25/25. `just format-check`, `just check-file-size`, `just check-sorted`, and `just check-i18n` passed. The Spaceport package render/check and 21 spec tests passed. No native window, Windows/Linux build, actual system handler, personal config, daemon or service was exercised.

## Merged verification

At `58a94491`, the editor cut followed the separate checkout-search CLI cut.
The combined app passed default and all-feature production checks with no new
code warnings. From the merged GUI test binary, checked local file links
passed 7 tests, rich-chat file links 5, review keys 3, retired editor config
1, and Core catalog 5. These checks preserve explicit file-click and Core
picker behavior while the client editor pane is gone. The merged i18n check
passed with 5,166 valid messages in 269 files; format, file-size, sorted-list
and package checks passed. Native handler and remote path behavior still need
native and platform tests; A29 remains `not_run`.
