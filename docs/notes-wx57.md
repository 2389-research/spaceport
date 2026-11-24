# wx57: `spaceport-endpoint`, the dependency-free policy (stage 1)

Branch `feat/wx57-endpoint-core`, cut from `0a2456281` (main after the endpoint
design landed; the design is `docs/notes-endpoint-extraction.md`, its §8 is
this stage's task list). No behaviour change: no message, no paint, no line of
the link machine's arithmetic differs.

## 1. The survey (§8 item 1), measured on `0a2456281`

`crates/spaceport-endpoint` did not exist and no crate under `crates/` was
named `spaceport-endpoint`.

| §0's count | on the base |
| --- | --- |
| endpoint production lines (`src/endpoint*`, not tests) | 3,126 (the design's 3,068, grown by #223/#225 and friends) |
| endpoint test lines (the same command's `tests/*`, `*tests.rs`, `fixtures.rs`) | 13,103 |
| whole endpoint tree | 16,229 (the plan's 16.2k) |
| files with `impl SpaceportWindow` | 7 (`endpoint.rs`, `polling.rs`, `catalog.rs`, `order.rs`, `follow.rs`, `fence.rs`, `wake.rs`) |
| `fl!("` sites over the same files | 15 |
| files importing `gpui` in production | 3 (`polling.rs`, `catalog.rs`, `follow.rs`, `Context`/`App`/`WindowId` only; nothing builds an element) |

The four things §8 asked to confirm, all confirmed:

- `History` names only `spaceport_client::protocol::ClientShellSnapshot`,
  `std::collections::VecDeque` and the `LIMIT` constant.
- The four `order` tests and the nine `history` tests drive public
  functions; the two `#[gpui::test]` tests in `order/tests.rs` and the one in
  `wake/tests.rs` drive a window and stay app-side (§3).
- `wake/tests.rs` builds a `WakeClock` directly, from explicit `Instant` and
  `SystemTime` values, and never reads the wall clock.
- Nothing outside `history.rs`, `wake.rs` and `order.rs` reads `History`'s
  `travel`: the app's callers are `observe`, `can`, `peek` and `begin`
  (`endpoint.rs:215-217`, `polling.rs:186`, `titlebar/navigation.rs`, and the
  `group_terminals` test), which is why the field list is unchanged.

## 2. What moved

| From `crates/spaceport-gpui/src/endpoint` | To `crates/spaceport-endpoint/src` |
| --- | --- |
| `history.rs:1-123` `History`, `Step`, `LIMIT`, their doc comments | `history.rs` (121 lines) |
| `history/tests.rs` (190 lines, 9 tests) | `tests/history.rs` (188) |
| `endpoint.rs:42-58` `LOCAL`, `SAVED_PREFIX`, `WSL_PREFIX`, `saved_profile_id` | `ids.rs` (21) |
| `order.rs:21-39` `rank`, `moved` (the module doc's first paragraph) | `order.rs` (33) |
| `order/tests.rs`'s `unarranged_devices_rank_after_arranged_ones` and `a_move_lands_beside_its_anchor` | `tests/order.rs` (31) |
| `follow.rs:23-44` `focus_requested` | `follow.rs` (27) |
| `endpoint/tests/follow_focus.rs`'s six snapshot tests | `tests/follow.rs` (65, driving `focus_requested` itself) |
| `wake.rs:38-143` `Trigger`, `NetworkChanges`, `SLEEP_GAP`, `NETWORK_SETTLE`, `WakeClock`, `watch_network`, `network_changes` | `wake.rs` (143) |
| `wake/network.rs` (100, the macOS path monitor) | `wake/network.rs` (100, unchanged) |
| `wake/tests.rs`'s five clock tests | `tests/wake.rs` (142) |

472 production and 446 test lines in the crate, over `lib.rs`'s wiring and five
modules. The app's endpoint tree went 3,126 → 2,720 production and 13,103 →
12,687 test lines (−406 / −416); `impl SpaceportWindow` blocks stayed at 7 and
`fl!("` sites at 15, because no window code moved.

`tests.rs` (20) is wiring and one fixture: the protocol's own
`endpoint-snapshot-v1.json`, `include_str!`'d by relative path, which is how
`spaceport-sound` reads the same file.

## 3. What the design left to this stage, and what it came to

- **R5: the macOS monitor moved.** `wake/network.rs` is in the crate, so its
  five `unsafe extern "C"` declarations, `block2` and `dispatch2` are now the
  crate's (macOS-only dependency block), and `dispatch2` left
  `spaceport-gpui`'s macOS dependencies because nothing else there used it.
  `block2` stayed: `app_badge.rs` uses it. The `unsafe` is scoped exactly as it
  was — two `#[allow(unsafe_code)]` items against the workspace's `deny`. This
  is the aggressive option, and it is proven on the platform it names: the
  workspace was checked on macOS (Darwin 25.6.0, arm64) in §5. A Linux run
  compiles the `cfg(not(target_os = "macos"))` arm, which is `NetworkChanges::
  default()`.
- **`recover_connections` stays in `wake.rs`.** The app's `wake.rs` is now 54
  lines: that one `impl SpaceportWindow` method, its doc comment, the module
  paragraph about recovering, and three names re-exported for its neighbours
  (`pub(super) use spaceport_endpoint::{NetworkChanges, Trigger,
  network_changes};`), so `endpoint.rs`'s `wake::Trigger` and `polling.rs`'s
  `super::wake::network_changes()` resolve exactly as before and neither file
  changed. The clock levers — sleep detection and the settling gap — moved.
- **No `error.rs`.** Nothing in the crate can fail, and the policy crates that
  are in the same position (`spaceport-reorder`, `spaceport-motion`,
  `spaceport-ago`, `spaceport-truncate`, `spaceport-words`) have none either;
  `lib.rs`'s doc says so, so the next stage has the reason in front of it.
  `NoHost`/`SurfaceSwitchingUnsupported` stay in `crates/spaceport-gpui/src/
  endpoint/error.rs`, as §8 said.
- **`rank` and `moved` keep their `crate::endpoint::order::` path.**
  `catalog.rs:250` calls `super::order::rank`, so `order.rs` re-exports both
  (`pub(super) use spaceport_endpoint::{moved, rank};`) instead of the façade
  in `endpoint.rs` naming them. The façade holds what is reached from outside
  the `order` module; every other path the app used is unchanged, and no app
  source file outside `src/endpoint/` was touched.
- **The façade is one `pub(crate) use`.** `crate::endpoint::{History, Step,
  LOCAL, SAVED_PREFIX, WSL_PREFIX, WakeClock, focus_requested,
  saved_profile_id, watch_network}` all resolve as they did. `SAVED_PREFIX` is
  re-exported because `catalog.rs` imports it from `super`.
- **Three `#[gpui::test]`s stayed, and six follow tests became snapshot
  tests.** The `order` and `wake` window tests need a `SpaceportWindow`, so
  they stayed where they were (they are 2 of the 4 in `order/tests.rs` and 1 of
  the 6 in `wake/tests.rs`). `follow_focus.rs`'s six `noted(...)` tests drove
  `focus_requested` through an `Endpoint` and an inbox; the crate's version
  calls `focus_requested(old, new)` on two snapshots, which is the same six
  cases without the app type. `a_replaced_connection_forgets_a_focus_it_saw`
  is about the endpoint's own state and stayed. The window-level follow tests
  (seven) are untouched.
- **`History`'s fields are `pub(crate)`**, as §8 anticipated: its bounded test
  reads `panes` and `cursor` directly. Nothing in the app can see them.
- **No `keep-sorted` markers in `AGENTS.md`.** `scripts/check-sorted.py` scans
  `*.rs`, `*.toml`, `*.py` and `*.sh` only, so a marker in a Markdown table
  would be unchecked noise; the row went in at its alphabetical place instead.
  The crate's `mod` lists do carry markers. The app's new re-export list does
  not: rustfmt reorders the items in a braced `use` (uppercase first) and
  `check-sorted.py` sorts case-insensitively, so a marker there can only ever
  fail one of the two. `just check-sorted` passes.

## 4. What stays

`LiveState` (`state.rs`), `ConnectionBridge` (`connection.rs` + `reader.rs`),
`Endpoint`, `Link`, `Phase`, `Release`, `Redraw`, the `Polling` tick, the
saved-host `Catalog` and `reconcile_devices`, `fence`, `config_reload`,
`origin`, `error.rs` and every view: 2,720 production lines. The
socket-driving lifecycle suite and the rest of `tests/` stay with them: 12,687
test lines.

## 5. Verification

Run on the branch, with `CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0`:

| command | result |
| --- | --- |
| `cargo metadata --format-version 1` | `Cargo.lock`: one package added, `dispatch2` moved from the app to the crate |
| `cargo check -p spaceport-endpoint` | passed |
| `cargo check -p spaceport-gpui` | passed |
| `cargo check -p spaceport-gpui --all-targets` | passed (all app test targets compile) |
| `cargo clippy -p spaceport-endpoint --all-targets` | passed, no warnings |
| `SPARKLE_FRAMEWORK_PATH=$PWD/target/sparkle-sdk cargo clippy -p spaceport-gpui --all-targets --features integration-test` | passed, no warnings (framework acquired with `python3 scripts/release/sparkle_framework.py acquire target/sparkle-sdk`) |
| `cargo test -p spaceport-endpoint` | 22 passed: 9 history, 6 follow, 2 order, 5 wake |
| app tests, `endpoint::order::tests::` | 2 passed |
| app tests, `endpoint::wake::tests::` | 1 passed |
| app tests, `endpoint::tests::follow_focus::` | 10 passed |
| app tests, `endpoint::tests::` | 44 passed |
| app tests, `endpoint::lifecycle_tests::{wake_recovery,reconnect_backoff,catalog_link}` | 7 + 4 + 1 passed |
| app tests, `titlebar::navigation` and `group_terminals::tests::titlebar_navigation` (the `History`/`Step` callers) | 4 + 3 passed |
| `cargo fmt --all -- --check` | clean |
| `just check-file-size` | clean; the longest new file is `tests/history.rs` at 188 |
| `just check-sorted` | clean |
| `just check-i18n` | 2,114 messages in 114 files valid and used |

Deferred to CI: the full workspace `just ci`, the Windows and Linux legs of the
`cfg` split (`wake/network.rs`'s `unsafe` and the `not(macos)` arm), and the
native/live suites, which need a desktop and an installed daemon.
