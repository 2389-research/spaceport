# wx74: the phone's missing cell and terminal fields

Branch `feat/wx74-phone-fields`, cut from `e99d575b6c`. `origin/main` was fetched
for this work: it is **`a8fbdb566884cd514ac7899405fe10f2ee0aca08`** (#246's docs
refresh and #243's keymap move). Neither commit touches `mobile/`, and the
branch base and `origin/main` differ in no file this change reads, so the work
stays on the base it was cut from and needs no rebase.

The fence PR #240 edits the projection's fence and navigation state. Nothing
here touches it: the changes are the two UniFFI records, one new record, the
`terminal()` mapping below them, and one new test file.

## 1. The gap

The wire carries three things the phone's records dropped:

| Wire field | Where | Phone record before |
| --- | --- | --- |
| `CellData.hyperlink: Option<u32>` | `crates/spaceport-protocol/src/wire.rs:260-268` | absent from `MobileCell` |
| `FrameData.hyperlinks: Vec<String>` | `wire.rs:284` | absent from `MobileTerminal`, so an index would have had nothing to point at |
| `PaneSurfaceScrollMetrics { offset_from_bottom, max_offset_from_bottom, viewport_rows }` (`PaneSurfacePane.scroll: Option<…>`) | `wire.rs:451-456`, `:437-450` | absent from `MobileTerminal` |

The mapping at `mobile/rust/src/projection.rs` rebuilt both records field by
field, so a wire field the phone did not name was dropped with no compile error
on either side. That is why the fix is worth making once per attribute rather
than waiting for a phone feature that wants one.

## 2. What the desktop does with them

**Hyperlink index and the frame's table.** `crates/spaceport-gpui/src/terminal/links.rs`
reads the pair, never one alone:

- `link_cell` (`:299-319`) finds the cell under the pointer and the cell that
  *draws* it, because a wide glyph's continuation cell need not carry the
  index; the lookup is `cells[selected].hyperlink.or(cells[source].hyperlink)`.
- `frame_link` (`:321`, index read at `:337-338`) resolves that index through `frame.hyperlinks`. A
  missing target yields no link; a present one is then classified by
  `RowTarget`: a validated web address, a path (explicit links name one whether
  or not the file exists), another host's `file://`, a malformed target, or
  another scheme such as `mailto:`. A destination this client would never open
  is *still* reported as a link, so a click can say what is wrong.
- `pane_hyperlink_at` (`:264`) is the same question asked for Herdr's
  plugin link handlers, which read destinations this client would not open.
- The table is per frame, numbered once. A pane's cell window indexes the
  frame's whole table; the numbers do not restart per pane, and a popup frame
  carries its own.

**Scroll metrics.** Two consumers:

- `crates/spaceport-gpui/src/terminal.rs:574-608` `Scrollbar`: the thumb size is
  `height * viewport_rows / (max_offset_from_bottom + viewport_rows)` and its
  top is `(height - thumb) * (1 - offset_from_bottom / max_offset_from_bottom)`,
  clamped to a 24 px minimum. It places the daemon's own `scrollbar_rect` (a
  cell rect) to the pixel, and the same value is what a drag turns back into an
  offset.
- A fence, not a picture: `crates/spaceport-gpui/src/links.rs:111,146,169`
  records `offset_from_bottom` beside a link cell, so a pane that has scrolled
  since the pointer reached the link refuses the stale click.
  `crates/spaceport-gpui/src/copy_mode.rs:295,677,758` uses the same offset to
  place the viewport in history and `max_offset_from_bottom` as the bound of a
  scroll request.

The phone has neither consumer today: no scrollbar anywhere in
`mobile/android` or `mobile/apple`, and it scrolls by sending
`scroll(target, lines)` (`mobile/android/.../TerminalSurface.kt:185-192`,
`mobile/apple/Sources/MobileConnection.swift:415-417`) without ever drawing
where it is.

## 3. Decisions

### The hyperlink is carried as the wire's index, not a resolved URL

`MobileCell.hyperlink: Option<u32>` plus `MobileTerminal.hyperlinks: Vec<String>`.

Resolving the index into an owned URL per cell at projection time was the
alternative, and it is worse on four counts:

1. **One spelling per concept.** The protocol, `spaceport-terminal`,
   `links.rs` and every fixture already name a link by an index into the
   frame's table. A resolved per-cell string would be a second spelling of the
   same fact, kept in sync by hand.
2. **Resolution is a policy, and the policy is the app's.** Turning
   `Some(0)` into a string is not the decision that matters; deciding whether
   the string is a web page, a path with a `:line` suffix, another host's file,
   a malformed target or a `mailto:` is. The desktop deliberately keeps a
   destination it would never open. Resolving in the projection would either
   throw that away or commit the projection to a reading of URLs that belongs
   beside the phone's UI, not beside its wire decoding.
3. **A continuation cell does not carry the index.** Wide glyphs make the
   pointer's cell and the drawing cell different. A per-cell URL would be
   `None` on the continuation and so lose the link; matching the desktop means
   walking back to the glyph's own cell, which needs the index, not a copy.
4. **Cost.** A pane full of one long URL would clone it into hundreds of
   cells of a record that crosses the FFI boundary every frame. An index is
   four bytes and the table ships once per terminal.

A popup's terminal carries `popup.frame.hyperlinks` — its own table, not the
surface. A popup is where the phone most often shows a link (a confirmation
prompt), so it is carried, not skipped.

### The scroll metrics are carried; `scrollbar_rect` is not

`MobileTerminal.scroll: Option<MobileScroll>` with the three numbers the wire
reports. The phone paints no scrollbar and reads no scroll position today, so
this is carried-but-unused, and that is only worth doing because it is cheap: a
new field on the record churns the generated Swift and Kotlin once, and the
next `PaneSurfacePane` change (splits, graphics, image ids) will churn them
again anyway. `#[uniffi(default = None)]` means no existing construction site
had to change for it.

`scrollbar_rect` is left out on purpose. It is a rect in *surface cells*, while
`MobileTerminal` describes a window sliced out of one pane; the desktop can use
it because it paints in the whole surface. A phone thumb needs its own track
and the three metrics above; the daemon's cell rect would be a coordinate the
phone would have to translate with no consumer asking for it.

A popup keeps `scroll: None`: `ClientShellPopupSurface` has no scroll state, so
there is nothing to carry.

## 4. What changed

| File | Change |
| --- | --- |
| `mobile/rust/src/projection.rs:33-45` | `MobileCell.hyperlink: Option<u32>`, `#[uniffi(default = None)]` |
| `mobile/rust/src/projection.rs:47-55` | new `MobileScroll` record: `offset_from_bottom`, `max_offset_from_bottom`, `viewport_rows`, all `u64` |
| `mobile/rust/src/projection.rs:56-70` | `MobileTerminal.hyperlinks: Vec<String>` (`default = []`) and `MobileTerminal.scroll: Option<MobileScroll>` (`default = None`) |
| `mobile/rust/src/projection.rs:604-659` | `terminal()` keeps the focused pane's `scroll`, carries the chosen frame's `hyperlinks`, and copies each cell's `hyperlink` |
| `mobile/rust/src/lib.rs:12-15` | exports `MobileScroll` |
| `mobile/rust/src/tests/cell_fields.rs` | three tests: an index kept with its table, a scroll position kept, a popup's own table with no scroll |
| `mobile/rust/src/tests/mod.rs` | declares `mod cell_fields;` |

The cell's index is copied unchanged, including out of the pane's window: the
numbers stay the frame's, so `MobileCell.hyperlink` can be looked up in
`MobileTerminal.hyperlinks` exactly as `links.rs` does on the desktop.

## 5. Bindings

`mobile/build-apple` and `mobile/android/build-native` install the
`uniffi-bindgen` their `Cargo.lock` pins (`0.32.2`) and generate on build, so
nothing generated is tracked here. To check the generated shape without an
Xcode or NDK cross-build, the same generator version was installed and run
against the host library:

```sh
cd mobile/rust && cargo build --lib
uniffi-bindgen generate --language swift  --no-format --metadata-no-deps --out-dir /tmp/wx74/swift  target/debug/libspaceport_mobile.a
uniffi-bindgen generate --language kotlin --no-format --metadata-no-deps --out-dir /tmp/wx74/kotlin target/debug/libspaceport_mobile.a
```

Every new field has a `#[uniffi(default = …)]`, so both generators emit it as a
defaulted parameter and the constructors stay callable as they were:

```swift
public init(symbol: String, fg: UInt32, bg: UInt32, modifier: UInt16, skip: Bool,
            hyperlink: UInt32? = nil)
public init(width: UInt16, height: UInt16, cells: [MobileCell], cursorX: UInt16?, cursorY: UInt16?,
            hyperlinks: [String] = [], scroll: MobileScroll? = nil)
```

```kotlin
data class MobileCell (… , var `hyperlink`: kotlin.UInt? = null)
data class MobileTerminal (… , var `hyperlinks`: List<kotlin.String> = listOf(), var `scroll`: MobileScroll? = null)
```

That keeps the only two places that construct these records compiling
unchanged — `mobile/apple/Tests/TerminalPreferencesTests.swift:45,47` and
`mobile/android/app/src/test/java/ai/spaceport/mobile/PanePreviewTest.kt:11` —
so no Kotlin or Swift file is touched by this change. The consuming side is
untouched and does not read the new fields yet: `TerminalDisplay.swift` paints
colour, weight, underline and the cursor, and `TerminalSurface.kt` does the
same plus gestures. A tap on a link or a scroll thumb is a later UI change that
these fields now make possible without another record change; the direct
Ghostty attachment (`GhosttyPaneView.swift`) draws its own links and still
swallows every open request, which is unchanged.

## 6. Validation

- `cd mobile/rust && cargo check --all-targets` — clean.
- `cd mobile/rust && cargo test` — 59 passed. The baseline in this tree is 56
  library tests (measured by stashing this change and listing them); the three
  new ones are `tests::cell_fields::*`. The task brief's "60" does not match
  this tree. Two ignored tests in `tests/live.rs` still require an explicitly
  configured SSH fixture.
- `cargo check -p spaceport-protocol -p spaceport-client -p spaceport-gpui --all-targets`
  — clean; those crates are unchanged.
- `cargo fmt --check` at the repository root and in `mobile/rust` — clean.
- Not run here: the Gradle and Xcode app builds. No Android SDK is installed
  on this machine, and the iOS Rust targets are not added. `xcodegen` and
  `xcodebuild` are present, but a full app build would compile every Swift file
  to prove two call sites that the generated signatures already settle, so the
  app compilations are left to CI. What was checked here is the generator's
  output for the pinned version, quoted above.

## 7. Risks

- **#240 touches `mobile/rust/src/projection.rs`.** This change edits the
  records above `MobileView` and the `terminal()` mapping only. If the fence PR
  adds a field to `MobileView`, the two hunks are adjacent but not overlapping.
- **Scroll metrics are carried and unused.** Nothing breaks, and a phone scroll
  thumb can now be drawn from `MobileScroll` alone. If the phone never wants
  one, the field costs one nullable struct per frame and nothing else.
- **`hyperlinks` is the whole frame's table.** A phone that draws one pane's
  window receives destinations that pane does not reference. The protocol
  already bounds a frame's size and validates every index against the table
  (`crates/spaceport-protocol/src/frame.rs:12,52`), so this is bounded data,
  not an invitation to resolve links the phone never shows.
