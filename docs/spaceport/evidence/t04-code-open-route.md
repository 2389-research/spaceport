# T04/RM-IDE: read-only code-tab control route

At parent `1461ce7a`, `crates/sheprd-gpui/src/cli/control/tabs.rs::code` accepted `sheprd code open FILE[:LINE]`, `crates/sheprd-gpui/src/control/client/tabs.rs::code` resolved a local file, and `crates/sheprd-control-protocol/src/protocol/tabs.rs::TabOp::Code` carried it through the private control socket. `crates/sheprd-gpui/src/control/server/tabs/arrive.rs::code` checked the absolute path and exact caller destination, then called `crates/sheprd-gpui/src/code_view.rs::open_code_view` to create or reuse a `Location::Code` tab. This cut removes that CLI verb, wire operation, server handler, help, skill promise and CLI documentation. The isolated branch raised the protocol from 14 to 15; integration with the independent named-window wire change assigns this removal version 16. The old command now fails before socket lookup, and the old JSON operation fails to decode.

`TabKind::Code`, `Location::Code`, saved code tabs and `crates/sheprd-gpui/src/code_view.rs` remain reachable through UI commands, `code_search.rs`, `security/actions.rs` and restored Store tabs. They need a separate coherent graph cut. The current `code edit` route remains reachable: `crates/sheprd-gpui/src/control/server/tabs/arrive.rs::edit` passes an `EditorTarget` to `crates/sheprd-gpui/src/editor.rs::open_in_editor`, which splits a terminal pane, types a shell command and can reuse a private Neovim socket. This is excluded editor integration, not the retained system file opener. `window/file_links.rs`, `review/view/keys.rs` and `code_search.rs` also call `open_in_editor`; removing it requires a later shared-caller cut. `window/file_links.rs` keeps the explicit system-opening route. No disabled handler or compatibility shim was added.

The parser and wire absence tests were red against `1461ce7a`. The wire's retained `Edit` round trip and server's invalid-path and missing-workspace tests keep the shared path and target checks. T04/RM-IDE stay in progress and A29 remains not run; this slice does not claim all code navigation or editor behavior is gone. No Core, personal daemon, real editor, user file or network service was touched.

Verification: `cargo +1.99.0 test -p sheprd-control-protocol --locked` passed 58 tests. Focused GPUI parser, skill, client tab, server target, terminal file-link and editor media tests passed; the real-binary CLI test rejected `code open` before socket lookup. `just format-check`, `just check-file-size`, `just check-sorted`, and the Spaceport package render/check/test commands passed. Native editor and cross-platform CI checks remain open. The build reported the existing macOS compact-unwind linker warning and Cargo's `block v0.1.6` future-compatibility notice.

After merge with the named-window change, the combined protocol suite passed
59 tests and the real-binary CLI suite passed all 15 tests, including the
`code open` refusal. The first combined CLI run failed four old Herdr 1 launch
flag expectations; those tests were corrected and rerun green in the same
bounded follow-up. This is parser and control-route evidence, not native UI
proof that all code views have been removed.
