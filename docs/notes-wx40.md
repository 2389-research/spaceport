# wx40 — chat PR 3: the selection module moves into `spaceport-chat-core`

Branch `feat/wx40-chat-selection`, cut from `origin/main` (`65c532dc4`). This is
PR 3 of `notes-wx36.md`'s four-PR series: the selectable text of a transcript
(`Document`, `Segment`, the carets and the range between them) is now the
crate's, and the app keeps only the gestures it makes with it.

## What moved (moved, not copied)

| Was | Is | Lines |
| --- | --- | --- |
| `crates/spaceport-gpui/src/chat/selection.rs` | `crates/spaceport-chat-core/src/selection.rs` | 378 → 395 |
| `crates/spaceport-gpui/src/chat/selection/document.rs` | `crates/spaceport-chat-core/src/selection/document.rs` | 540 → 545 |
| `crates/spaceport-gpui/src/chat/selection/tests.rs` | `crates/spaceport-chat-core/src/selection/tests.rs` | 470 → 538 |

`Region`, `Cell`, `Key`, `Caret` and `Selection` are unchanged in shape and
behaviour; they are `pub` now instead of `pub(crate)`. `Document` and `Segment`
keep every method the app calls (`key`, `cells`, `segment`, `set_expanded`,
`set_remote_links`, `update`, `clear`; `text`, `shared`, `code`, `atomic`,
`source_offset`), with `order`/`ends`/`caret`/`copy` left module-internal.

`Document`/`Segment` read bodies from the crate's own
`spaceport_chat_core::prepared::Prepared` (moved in #215) rather than through the
app's shell: the app's `Prepared::update` still hands over
`self.core.bodies()`, exactly as it did before.

The app keeps `crates/spaceport-gpui/src/chat/selection.rs`, now a 9-line
facade re-exporting the crate's names at the paths every importer already used
(`crate::chat::selection::{Caret, Cell, Document, Key, Region, Segment,
Selection}`), following the `chat/rich.rs`, `chat/visual.rs` and
`chat/summary.rs` precedent.

## What stayed app-side

- **The words.** `chat/text.rs`'s `turn` is unchanged and is now the app's
  implementation of `Labels::turn` (`chat/labels.rs:24`); `chat/text/tests.rs`
  stays app-side and still tests it directly.
- **The link classification.** `chat/rich.rs`'s `destination` and
  `is_local_path` stay in the app (`is_local_path` is now `pub(super)`): the
  crate cannot know which destinations this client may act on.
- **The window.** `chat/view/selection.rs`, `chat/view/text.rs`,
  `chat/view/rich/code.rs`, `chat/view/entries.rs`, `chat/view/visuals.rs`,
  `chat/view/summary.rs`, `chat/chats.rs`, `chat/reader.rs` and every test under
  `chat/tests/` compile unchanged.

## Labels injection (`Labels::turn`)

One field, in the shape #209 established:

```rust
/// How a turn ended, how long it took, and the tokens it used when the
/// agent reports them.
pub turn: fn(&TurnSummary) -> String,
```

`Document::project` writes a `Turn` entry with `(labels.turn)(turn)` where it
called `crate::chat::text::turn`, so the window keeps writing the language it
shows (`spaceport_chat_protocol::display` was deliberately not substituted: it
writes English for herdr-acp's own view). The three `Labels` construction sites
(`chat/labels.rs`, `chat/tests/labels.rs`, `chat-core/src/prepared/tests.rs`)
name the field.

## The clipboard bound

`crates/spaceport-gpui/src/terminal/selection.rs`'s `MAX_SELECTION_BYTES`
(4 MiB) is now `spaceport_client::text::MAX_COPY_BYTES`, defined once beside
`safe_text`, and re-exported as `MAX_SELECTION_BYTES` so `terminal.rs:12`, the
terminal selection (`:782`, `:789`), the window copy (`window/selection.rs:461`)
and the moved selection all read one value. Moved value, no behaviour change.

## Design deviations

1. **`is_local_path` is injected into `set_remote_links`.** `notes-wx36.md` §4.3
   said everything else in `selection/` is pure once `prepared` moved, but
   `document.rs` is `remote_content`'s only call site and the crate's signature
   is `remote_content(content, is_local_path: fn(&str) -> bool)`. The crate's
   `Document::set_remote_links(&mut self, remote: bool, is_local_path: fn(&str)
   -> bool)` now stores the classifier (`remote: Option<fn(&str) -> bool>`, so
   the flag and the test cannot disagree and `Default` still derives) and
   `Builder::inline` passes it through, matching
   `prepared::Prepared::update`'s `destination` parameter. The app shell keeps
   its own `Prepared::set_remote_links(remote: bool)` and forwards
   `super::rich::is_local_path`, so no app call site changed.
2. **`tool_text` leaves the app's `chat/prepared.rs` facade.** With `document.rs`
   gone, nothing in the app's production code calls it; only
   `chat/tests/labels.rs` and `chat/view/entries/tests.rs` did, and the re-export
   warned as unused. Both tests import `spaceport_chat_core::prepared::tool_text`
   directly. Every other facade name kept its path. The alternative — an
   `#[allow(unused_imports)]` on the facade — would have widened an allowance.
3. **The moved test file needed a fixture, not just new paths.** The app's tests
   read `Prepared::text()` (the shell's own document). In the crate's tests a
   local `Shown` fixture keeps a `Prepared` and a `Document` together and
   re-runs both passes in `update()`, and the tests name their own `Labels`
   (`turn` included), destination classification and `is_local_path` — which is
   why the file grew by 68 lines. Test set, assertions and behaviour are the
   same 17 tests as before.

## Verification

- `cargo fmt --all` (clean; `cargo fmt --all --check` exits 0)
- `cargo check -p spaceport-chat-core -p spaceport-gpui --all-targets --features integration-test`
- `cargo clippy -p spaceport-chat-core -p spaceport-gpui --all-targets --features integration-test`
- `cargo test -p spaceport-chat-core` — 122 passed
- `cargo test -p spaceport-gpui --bin spaceport chat` — 445 passed
- `cargo test -p spaceport-client` — passed
- `just check-file-size` and `just check-sorted` — clean
- Deferred to CI: the Windows-only drive-path arm of `file_target` behind
  `is_local_path`, and the native UI pass (`chat/preview/selection.rs`'s probe
  run needs a desktop).
