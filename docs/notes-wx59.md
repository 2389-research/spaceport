# wx59: the saved-host catalog (endpoint stage 2)

Branch `feat/wx59-endpoint-catalog`, cut from `93a197f78` (main after #226; the
design is `docs/notes-endpoint-extraction.md`, its §5 seam 7 and §6 stage 2 are
this stage's task list). No behaviour change: the same files are read and
written with the same bounds, the same loads retry every two seconds, the same
distinct failure warns once, and the window reconciles its endpoints from the
same device list in the same order.

## 1. The survey (§6 stage 2), measured on `93a197f78`

- **`StorageFailure::find` cannot be reached from the crate.** It is
  `pub(crate)` in `crates/spaceport-gpui/src/storage_warning.rs`, and the
  catalog is what needs it: `Catalog::last_failure` is the warn-once key, and
  the app's own `catalog/tests/storage_warning.rs` fixes what the key is (a
  failure on another path with the same display text is a different failure).
  So the value type moved whole — `find`, `PartialEq` and `warn` — and the app
  keeps its entry point: `warn_storage_failure(context, error)` and the
  `pub(crate) use spaceport_endpoint::StorageFailure;` in its place, so the five
  other call sites (`chat/files.rs:279`, `sessions.rs:354`,
  `window/transfers.rs:261`, `menu/devices/wsl.rs:426,508`) are unchanged.
- **The client's catalog loaders and stores are public.** `load_saved_hosts`
  (`crates/spaceport-client/src/catalog.rs:38`), `load_saved_host_selection`
  (`:52`) and `store_saved_host_selection` (`:121`), and
  `load_wsl_hosts`/`store_wsl_selection`
  (`crates/spaceport-client/src/wsl/catalog.rs:94,139`), are `pub fn`s the app
  already called by crate path. Nothing new is exported for this stage, and
  `WslHosts::selected_host`, `SavedHost` and `WslHost` are public with public
  fields.
- **`Catalog::new(&ConnectTarget)` reads only its argument.** It matches the
  target — `Socket` → no catalog at all, `Session` → that session's
  `development`, anything else → the release one — and calls
  `with_development`. No window field, no clock, no I/O; `inactive()` is
  `with_development(None)`.

## 2. What moved

| From `crates/spaceport-gpui/src` | To `crates/spaceport-endpoint/src` |
| --- | --- |
| `endpoint/catalog.rs`'s `Catalog`, `CatalogUpdate`, `load`, `store`, `Device`, `devices`, `entry_changed` (197 of its 382 lines) | `catalog.rs` (269, the doc comments widened for `pub`) |
| `endpoint/catalog/tests.rs`'s three window-free tests and the startup subprocess test | `catalog/tests.rs` (135) |
| `endpoint/catalog/tests/wsl.rs` (47) | `catalog/tests/wsl.rs` (47) |
| `endpoint/catalog/tests/storage_warning.rs` (76) | `catalog/tests/storage_warning.rs` (76) |
| `storage_warning.rs`'s `StorageFailure`, `find`, `warn` (36 of its 54 lines) | `storage.rs` (49) |
| `storage_warning/tests.rs`'s three value tests | `storage/tests.rs` (56, one more for the crate's own wrapper) |

818 production and 760 test lines in the crate now (stage 1 left 472 / 446), over
`lib.rs`'s wiring and eight modules. The app's endpoint tree went 2,711 → 2,463
production and 12,687 → 12,443 test lines (−248 / −244); `impl SpaceportWindow`
blocks stayed at 7 and `fl!("` sites at 15, because no window code moved. The two
`gpui`-importing files that remain are unchanged in kind: `polling.rs` and
`catalog.rs` still take a `Context<SpaceportWindow>`, and nothing here builds an
element.

## 3. What the design left to this stage, and what it came to

- **`storage_warning` moved its value, not its module** (§1). The app's
  `storage_warning.rs` is 21 lines: the module paragraph, the re-export, and
  `warn_storage_failure`. The two tests that need the app's own root `Error` to
  wrap a client failure stayed with it; the four that only compare failures went
  with the type.
- **The crate has an `error.rs`, its first.** `Error::Client(#[from]
  spaceport_client::Error)` and `Error::Worker(#[from] std::io::Error)` for the
  thread a load or a write could not start. `Client` is `#[error("{0}")]`, not
  `#[error(transparent)]` — a transparent wrapper's `source()` is the inner
  error's *source*, so `StorageFailure::find` would miss the client error
  entirely and the retry would log nothing. `Result<T, E = Error>` is re-exported
  at the root, as every crate here does.
- **`poll_write` answers with the crate's error, and the app words it.** The
  four call sites keep `warn_storage_failure("Save host selection", &error)`
  (`&(dyn StdError + 'static)` is unchanged) and `fl!("endpoint-save-selection-
  failed", error = error.localize())`; `crates/spaceport-gpui/src/i18n/errors/
  endpoint.rs` gains the mapping — `Client` through the client's own message,
  `Worker` verbatim, exactly as the app shows its own `Io` — and a
  `every_catalog_variant_has_a_message` test beside the endpoint one. The app's
  root `Error` gained no variant: nothing asks the window's error type for a
  catalog failure.
- **`entry_changed` takes two entries, not an `Endpoint`.** The crate's version
  is `entry_changed(saved, entry)` and the app passes `endpoint.saved_target()`,
  which is already documented as "the target this device was saved with … one
  never reconciled against the catalog has only its live target to go on". The
  rule the doc states — compare saved entries, never the session the connection
  is on — is the reason the function exists rather than an inline `!=`.
- **`devices(hosts, wsl)` returns the list; the window sorts and reconciles it.**
  `reconcile_devices` keeps the sort on `self.device_order`, the join/leave log
  lines, the endpoint rebuild, `prune_device_removals`, `device_filter` and
  `land_arrival`. It did not get shorter — 88 lines before and after, because
  every line of it but the two call sites reads the window's own endpoint list,
  order or filters — but what it no longer does is decide what a saved host
  *is*: the mapping to `Device`s and the saved-entry comparison are the crate's,
  and the sort stays beside the order it reads.
- **One `#[gpui::test]` stayed app-side.** `old_container_catalog_stays_foreign_
  to_startup_devices` drives `reconcile_devices` on a fixture window, so it is
  reconcile behaviour, not catalog behaviour; `endpoint/catalog/tests.rs` is now
  that one test (34 lines). The startup test that starts the test binary again
  moved with `load` and `devices`, and its `module_path!()`-named child run
  follows it (`catalog::tests::startup_…`).
- **`Catalog` joined the façade, `SAVED_PREFIX` left it.** `endpoint.rs`'s
  `pub(crate) use spaceport_endpoint::{…}` list now names `Catalog`, so
  `crate::endpoint::Catalog` resolves as it did for `window/construct.rs` and the
  sidebar's layout fixtures. `SAVED_PREFIX` was imported by `catalog.rs` alone
  and went with it; `WSL_PREFIX`, `LOCAL` and `saved_profile_id` stay because
  other modules read them. No file outside `src/endpoint/` changed — not
  `window.rs`, not `construct.rs`, not the sidebar's fixtures.
- **`tracing` is an unconditional dependency now.** The storage warning logs
  where the app's did (`category = "storage"`, context, operation, path,
  message), and the macOS block keeps only `block2`/`dispatch2`. The crate's
  `tempfile` dev-dependency is for the two catalog tests that keep a state file
  in a directory of their own.
- **The tests that drove the app's error wrap are the only ones left behind.**
  `storage_warning/tests.rs` keeps `finds_the_storage_step_inside_a_wrapping_gui_
  error` and `ignores_errors_without_a_storage_step`, both over `crate::Error`;
  the crate's copy of the same check uses its own wrapper.

## 4. What stays

`LiveState` (`state.rs`), `ConnectionBridge` (`connection.rs` + `reader.rs`),
`Endpoint`, `Link`, `Phase`, `Release`, `Redraw`, the polling tick,
`reconcile_devices`, `restore_selection`, `reconcile_catalog`, `fence`,
`config_reload`, `origin`, `error.rs` and every view: 2,463 production lines.
The socket-driving lifecycle suite and the rest of `tests/` stay with them:
12,443 test lines. The two `catalog_link` suites (1 lifecycle test, 3 unit
tests) pass untouched.

## 5. Verification

Run on the branch, with `CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0`:

| command | result |
| --- | --- |
| `cargo metadata --format-version 1` | `Cargo.lock`: `tempfile` and `thiserror` joined `spaceport-endpoint`'s own dependency list; no new external crate |
| `cargo check -p spaceport-endpoint` | passed |
| `cargo check -p spaceport-gpui --all-targets` | passed (every app target, tests included) |
| `cargo clippy -p spaceport-endpoint --all-targets` | passed, no warnings |
| `SPARKLE_FRAMEWORK_PATH=$PWD/target/sparkle-sdk cargo clippy -p spaceport-gpui --all-targets --features integration-test` | passed, no warnings (framework acquired with `python3 scripts/release/sparkle_framework.py acquire target/sparkle-sdk`) |
| `cargo test -p spaceport-endpoint` | 37 passed: 22 from stage 1, 4 `catalog::tests`, 4 `catalog::tests::storage_warning`, 3 `catalog::tests::wsl`, 4 `storage::tests` |
| `cargo test -p spaceport-gpui --bin spaceport -- endpoint` | 248 passed, 1 ignored |
| app tests, `endpoint::tests::catalog_link` and `endpoint::lifecycle_tests::catalog_link` | 3 + 1 passed, files untouched |
| app tests, `endpoint::catalog::tests` and `storage_warning::tests` | 1 + 2 passed |
| app tests, `i18n::errors::tests::endpoint` | 2 passed |
| `cargo test -p spaceport-gpui --bin spaceport` (whole suite) | 3,427 passed, 0 failed, 10 ignored |
| `cargo fmt --all -- --check` | clean |
| `just check-file-size` | clean; the longest new file is `catalog.rs` at 269 |
| `just check-sorted` | clean |
| `just check-i18n` | 2,114 messages in 114 files valid and used |

Deferred to CI: the full workspace `just ci`, the Windows leg (`cfg!(windows)`
decides whether the WSL list is read at all — a macOS run only compiles it), and
the native/live suites, which need a desktop and an installed daemon.
