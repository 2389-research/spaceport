# T04/RM-MODS: plugin link activation dependency

Read-only source and pinned-schema audit at `0893a857`. No product code, daemon, Core repository, personal files or network route changed. T04 is blocked on G-LINK; independent T04 deletions can continue. A30 and A33 remain `not_run`.

## Current route

`window/links.rs` sends `LinkRequest::Activate` for a link-modifier click whenever Herdr 1 advertises `pane.link.activate`, including ordinary web and `file://` links. `links.rs` maps that request to `Method::PaneLinkActivate`. The reply may say `handled: true` after a plugin `[[link_handlers]]` handler runs, report `plugin_link_failed`, or return the URL for a declined click. Pre-existing `window/links/tests/file_handlers.rs` encodes all three behaviors for a file hyperlink, including a remote-host file that Spaceport itself cannot open. This is a live excluded plugin route, not a display-only wire field.

The same activation reply supplies the complete URL when web text wraps across rows. `pane.link.resolve` returns only row/column regions (`links.rs::ResolvedLink::from_response`). The retained test `window/links/tests/cases.rs` expects a wrapped HTTPS address to open. The row-local parser in `terminal/links.rs::frame_link` deliberately refuses multi-row web text: the painted surface has no soft-wrap flag, so it cannot know whether adjacent rows form one URL. Removing activation now would break that retained click; leaving it for wrapped URLs would still run plugin handlers. Inferring the URL from cells would guess terminal line semantics.

Pinned Core 0.1.18 `evidence/t00-core-runtime.json` advertises 32 methods and no link/URL inspection method. The Core client has no such route. Ordinary single-row web URLs, checked local files and explicit external-scheme confirmations already use local paths in `window/links.rs`, and rich-chat links open through `chat/view/rich/links.rs`; they do not require plugin activation.

## Required contract and next test

A side-effect-free, read-only route must yield a bounded, typed link target and span for an exact terminal identity, content revision and viewport/cell, with stale and no-link results; or an equivalent verified terminal surface contract must expose enough wrap and target data for the retained parser. This describes the missing contract, not an invented Core RPC. Keep current link behavior until that route is proved. Then test that no click sends `pane.link.activate`, a wrapped URL still opens once, local files retain their checks, remote file paths never open locally, untrusted schemes still require confirmation, and rich-chat links remain usable. Run native macOS/Windows/Linux interaction checks before claiming A33.

No red/green code test or product build ran for this blocked audit. Existing test source was inspected, not counted as a passing test in this worktree. `uv run --with jsonschema==4.26.0 python tools/spec.py check` passed schema and local-link validation; `uv run --with jsonschema==4.26.0 python tools/test_spec.py` passed 21/21. `git diff --check` passed. These checks cover record structure only.
