# wx41 — the chat extraction's last stage: residual deletion and smoke rows

Base: `origin/main` @ `c0dd95889` (*Move the chat's selectable text into its
crate, #216*). Branch `feat/wx41-chat-residual`. This is PR 4 of
`notes-wx36.md`'s four-PR series.

## Bottom line

- **Nothing is left to delete.** #212, #215 and #216 removed every file whose
  content moved, and the compiler already proves the rest: an unused
  `pub(crate) use` in this crate warns (wx40 removed `tool_text` for exactly
  that), and `cargo check -p spaceport-gpui -p spaceport-chat-core --all-targets
  --features integration-test` is clean at this base. The survey below names
  every remaining file and symbol with its callers.
- **No new smoke row.** `HERDR_TEST_RICH_CHAT` stays as the
  parse/prepared/selection row. The design's gap — no fixture ever builds a
  non-local `Target` — is real, but a native row cannot assert either side of it
  today, so it is recorded rather than forced (`notes-wx43.md`'s precedent).
- **Counts re-measured** and `docs/gpui-split-plan.md`'s `chat` row rewritten for
  the shipped stages 3b-3d.

## The residual deletion is already done

What each move deleted, from `git show --diff-filter=D --name-only`:

| Commit | Deleted from `crates/spaceport-gpui/src/chat` |
| --- | --- |
| #212, the rich parse and its policy | `rich/parse.rs`, `rich/remote.rs`, `rich/math_input.rs`, `rich/tests/code.rs`, `rich/tests/prose.rs` |
| #215, the prepared entry model and the widgets | `tests/prepared.rs`, `tests/prepared/cut.rs`, `tests/prepared/fences.rs`, `widgets/error.rs` |
| #216, the selectable text | `selection/document.rs`, `selection/tests.rs` |

`chat/prepared.rs` and `chat/selection.rs` were rewritten rather than deleted —
the first kept a shell, the second became a facade — and that is the whole of
what "residue" could still mean, so the survey is of what survived those three
commits, not of what they left behind.

## What remains under `chat/rich/`, and why none of it is moved code

| File | Lines | What it holds | Callers |
| --- | --- | --- | --- |
| `rich/math.rs` | 112 | `draw` (RaTeX layout and rasterization, its pixel and side bounds, the texture upload) and `within_limits`, which bounds recursion and control characters before the third-party parser | `chat/visual.rs:185` through `rich.rs`'s `draw_math`; `rich/tests.rs` and `rich/tests/display_math.rs` |
| `rich/tests.rs` | 174 | the app's parse *shell* (the crate's `parse` with this client's `destination`, its wants handed to a real `Visuals`), the rasterizer's limits and the link policy | itself |
| `rich/tests/display_math.rs` | 31 | display math's optical size and its scaling past the old 32 px cap, measured on the raster | itself |

`draw` is not a candidate for the crate by construction: it returns
`visual::Drawn`, which holds `gpui::RenderImage`, and it is the only place the
rasterizer's own bounds live. Everything that *was* the crate's — the block
model, the parse, `remote_content`, the math input protection — is already
`spaceport-chat-core/src/rich/`.

The two tests stayed for the same reason: they drive `Visuals` and the window's
texture path, which no crate test can build.

## What stays as a shell or a facade, and the callers that keep it

The design allowed the facade form where it is cheaper than renaming importers;
each of the three survivors is load-bearing, and the compiler proves it:

| Path | Lines | What it is | Why it stays |
| --- | --- | --- | --- |
| `chat/prepared.rs` | 113 | a shell, not a facade: it holds the crate's `Prepared` *beside* `Visuals` and the selectable `Document` | the two things only a window has. It answers the model's wants in the one order that works (`take_wants` before `retain`, `chat/prepared.rs:87-90`), forwards `super::rich::is_local_path` into `Document::set_remote_links`, and keeps `forget_text`/`clear` over both halves. 9 files read it |
| `chat/selection.rs` | 9 | a facade over `spaceport_chat_core::selection` | 7 files import `crate::chat::selection::{Caret, Cell, Document, Key, Region, Segment, Selection}`; renaming them would be a diff with no behaviour in it |
| `chat/rich.rs` | 65 | the link policy (`destination`, `is_local_path`, `remote_content` over the crate's function) and `highlight`, the grammar cache | 6 files read it. The crate cannot own either: `destination` needs `WebUrl` + `terminal::file_target` (which destinations *this* client may act on), and `highlight` colours through `spaceport-chat-core`'s syntect cache while the app's `Visuals` job owns the spans |

`chat.rs`'s re-exports are the same story and every name in them has a caller:
`ChatInput`, `Chats`, `LauncherDeck` and `Toward` (42/29/6/20 sites),
`check_env` (`chat/start.rs:275`), `env_text`/`parse_env`
(`settings_window/controls/chat_env.rs`), `Refusal`, `MAX_CHATS`,
`core_pane_key` (`control/server/chat.rs:34`), `native_start` (`smoke.rs:94`),
`StartError`, `Program`, `Receipt`, and the crate modules `env`, `existing`,
`highlight`, `link`, `list`, `session` and `summary` (the last reached as
`super::super::summary::Summary` from `chat/view/summary.rs:7`). Nothing there
names app code that no longer exists; those names either moved *to* the crate
and are re-exported at their old paths by design, or still exist in the app.

No duplication of the classification survives either: `terminal::file_target`
is the one reader of a local path, called from `rich.rs`'s `destination` and
`is_local_path`, and Design C (a separate `spaceport-link` crate) never landed —
`crates/spaceport-web-url` is where `WebUrl` went instead.

## Smoke rows

`HERDR_TEST_RICH_CHAT` (`smoke.rs:97`, owned by `chat/preview.rs`) stays as it
is: it is the row that runs a synthetic conversation through the real reducer,
the crate's parse, the `Visuals` workers and the painter, and it is the only
native coverage `spaceport-chat-core`'s model has.

Coverage of the three moved models, by layer:

| Surface | Crate tests | Headless GPUI | Native |
| --- | --- | --- | --- |
| the block model and parse, with the app's policies injected | `rich/tests.rs` + `rich/tests/{blocks,code,display_math,prose,wants}.rs`, 22 tests | `chat/rich/tests.rs`, 4 tests (the app shell) | `HERDR_TEST_RICH_CHAT` cases `math`, `code`, `diagram`, `conversation`, `all` |
| the prepared entry model and the widgets | `prepared/tests.rs`, 10 tests; `widgets/tests.rs`, 5 | `chat/prepared/tests.rs`; `chat/tests/{code,embeds,questions,user_messages,widgets,relocale}.rs` | cases `widgets`, `widgets-end`, `tools`, `steps`, `running`, `attachments` |
| the selectable text | `selection/tests.rs`, 17 tests | `chat/tests/selection*`, `document_text`, `view/selection/tests.rs` | case `selection` plus the XTest probe (`chat/preview/selection.rs`) |
| a remote conversation's links, drawn | — | `chat/view/rich/links/tests.rs`, 2 tests | **none** |
| a remote conversation's links, copied | `selection/tests.rs`'s remote cases | — | **none** |

### The gap: no fixture builds a non-local `Target`

`chat/preview.rs:508-514` is the only `Target` a native fixture constructs and it
hard-codes `route: Route::Local`. So two one-line reads of the same fact are
unproven end to end:

- `chat/chats.rs:298` — `ChatPane::new` hands `!target.is_local()` to
  `Prepared::set_remote_links`, which is what makes a copy of a remote
  conversation write a link's path out beside its label;
- `chat/view/rich/links.rs:33` — the painter computes the same flag again for
  `Links::content`, which is what makes the path visible instead of actionable.

**Decision: no native row is added, and the gap is recorded instead.** The
reasons, in the order they mattered:

1. **A native case cannot assert either side.** The fixture would build its pane
   through `Chats::preview`, which is `#[cfg(feature = "integration-test")]`, so
   the case lives only in that fixture; `Prepared::text()` returns a `Document`
   whose whole text has no accessor (`key`/`cells`/`segment` only), so the copy
   side is not readable from the fixture without new crate API; and the drawn
   side exists only in the painter's transformed `Cow<[Inline]>`, which the
   fixture cannot see at all. What the row would really produce is a screenshot
   of a boolean branch, for a human to read.
2. **The transformation itself is already covered**, headlessly and
   deterministically, on both sides: `remote_content` by
   `chat/view/rich/links/tests.rs` (2 tests, including the exact
   `Changes and details (/tmp/file.md)` text the design names) and the copy
   projection by the crate's own remote-link cases.
3. **The wiring left is two reads of `Target::is_local()`,** a derived equality
   over `route == Route::Local`. Nothing in this series changed it; #212/#215/
   #216 changed what the flag is *handed to*, and both hand-overs are exercised
   by the crate's tests.
4. **A headless test is the cheaper honest closure, and it is not available
   today**: reaching a remote pane without a daemon means `Chats::preview`
   (feature-gated) or `Chats::reconcile` (which starts a link and needs a
   `connect` that cannot succeed), and `ChatPane::sync_view` is private, so a
   test cannot prepare a pane's bodies from a transcript it fed by hand.

Left explicitly unproven, for whoever revisits this: the `!` in `chats.rs:298`
and the second computation in `links.rs:33`. Both become cheap to assert the
moment either lands — a whole-text accessor on `Document`, or a `Links`
constructor that does not need a window's chats — and the fixture case then
costs one `Options` arm and one `preview_branding` run.

The design's other gap, a guard test that `spaceport-chat-core/src` names no
`gpui`, is not in this brief: the fact holds today (0 occurrences of `gpui` in
the crate, and no `gpui` dependency), but no test enforces it. Recorded, not
added.

## Counts after the series

Measured the way `docs/gpui-split-plan.md` counts: every line of every `.rs`
file, split by whether it is a test file (`tests.rs`, `tests/**`, `*_tests.rs`).

| Side | Production | Test | Files | `#[test]` functions |
| --- | --- | --- | --- | --- |
| `crates/spaceport-gpui/src/chat` + `chat.rs` | 16,145 | 14,139 | 174 | 200 |
| `crates/spaceport-chat-core/src` | 5,225 | 3,202 | 48 | 126 (122 run on this macOS host; the rest are `cfg(target_os)` arms) |

The app's chat is 30,284 lines against the crate's 8,427; the row's earlier
20,706 / 16,719 was counted before the extraction series and is replaced. The
other three columns were re-measured the same way: 74 is the literal `gpui::`
paths in the app's chat production files (334 more are in its tests), 141 is the
distinct `fl!` message ids it writes, and 30 is the files in
`crates/spaceport-gpui/src` that name `spaceport_chat_core`. What
stays app-side, by group: `view/` (10,005 lines with its tests) and the painter
entry `view.rs` (654), the `Visuals` workers (`visual.rs` 479, `visual/` 47),
the window and tab side (`window.rs` 865, `chats.rs` 829), the process and I/O
paths (`start.rs` 434, `start/` 111, `files.rs` 401, `discovery.rs` 148,
`core_pane.rs` 328, `reader.rs` 253, `harnesses.rs` 198, `page.rs` 157,
`pasted.rs` 85), the fixtures (`preview.rs` 912, `preview/` 316) and the three
survivors above. The app's own chat tests, 11,049 lines of them, stay with the
window they drive.

## Verification

- `cargo fmt --all --check` clean
- `cargo check -p spaceport-gpui -p spaceport-chat-core --all-targets --features integration-test` clean (no dead or unused re-export)
- `cargo clippy -p spaceport-gpui -p spaceport-chat-core --all-targets --features integration-test` clean
- `cargo test -p spaceport-gpui --bin spaceport chat` — 445 passed
- `cargo test -p spaceport-chat-core` — 122 passed
- `git grep` per symbol before concluding a deletion: the counts above are the
  evidence that no file qualified
- Deferred to CI: the native fixtures (no desktop in this session), and the
  Windows-only arm of `file_target`

## What this does not do

No behaviour change, no file reshuffle, no new dependency, no new test and no
new fixture row. The closing argument of this series is that the app's chat is
now the window's — painters, workers, processes, words — and that everything
model-shaped below it is the crate's.
