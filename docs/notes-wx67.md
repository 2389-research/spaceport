# wx67: the device catalog's `herdr` CLI calls move to `spaceport-remote`

Branch `feat/wx67-setup-remote`, cut from `0a2456281e` (main before #233/#234),
rebased onto `9b1556ac31`. This is the exception `docs/notes-menus-extraction.md`
stage 5 names: `menu/devices/setup.rs` (S7) is the one page-body seam that can
move without the text-field, endpoint, keymap and i18n stories the other stages
wait on. It answers that design's open question 4 — the home is
`spaceport-remote`, not a new `spaceport-herdr-cli`. No behaviour change: the
same `herdr` arguments, the same 15 s deadline, the same bounds, the same log
categories and the same English text the log and the window show.

## 1. What moved

| From `crates/spaceport-gpui/src` | To `crates/spaceport-remote/src` |
| --- | --- |
| `menu/devices/setup.rs` (271) — `Claim`, `claim_saved`, `remove`, `rename`, `device_label`, `run_cli`, `run_bounded`, `last_line`, `cli_environment`, `LABEL_LIMIT`, `CATALOG_ROOTS` | `setup.rs` (264) |
| `menu/devices/error.rs` (25) — `Busy`, `InvalidLabel`, `NoProfile`, `Remove`, `Rename`, `SetupTimeout` | `setup/error.rs` (31), plus the two variants the move made necessary (below) |
| `menu/devices/setup/tests.rs` (77) — the claim, label and CLI tests | `setup/tests.rs` (77), unchanged apart from the paths |

The crate's root `Error` gains `Setup(#[from] setup::Error)`, as it already
wraps `host_script::Error`, and the crate gains `tracing` (the move keeps every
`tracing::info!`/`warn!`/`debug!` site and its category).

## 2. The survey: what of `setup.rs` was the app's

The design's "no GPUI at all" holds — no `gpui` type, no `fl!` word, no window
or entity, no `SpaceportWindow` method. Four things were nonetheless the app's,
and each is now the caller's:

- **The `herdr` executable.** `remove` and `rename` resolved it through
  `crate::daemon::executable()`, which searches `PATH`, the user's `~/.local/bin`
  and `~/.cargo/bin` and Homebrew's prefixes. That is this app's discovery of an
  installed runtime, not the catalog's business, so both functions now take the
  executable as `&OsStr` and `host_menu` passes `crate::daemon::executable()?`
  from the background task it already runs on. The private halves that took an
  executable before (`remove_with`, `rename_with`) were the shape the move
  needed, so they became the public functions and the two wrappers went away.
- **The login environment.** `crate::login_env::apply` was already
  `spaceport_client::login_env::apply` re-exported by the app's `main.rs`.
- **The error type.** It has the words `error-devices.ftl` translates and the
  app's `Localize` mapping, so moving the enum alone would have left the app
  mapping a foreign type either way. The enum moved to `setup/error.rs` and the
  app kept every word: `i18n/errors/devices.rs` now implements `Localize for
  spaceport_remote::setup::Error` and still paints from the same key per
  variant, and `docs/i18n.md`'s rule (the crate holds English `Display` for the
  log, the app holds the messages) is what `spaceport_client::Error` already
  does. The `thiserror` text is byte-identical: `host_menu`'s
  `error.to_string()` and its status-bar notice show the same sentence as
  before.
- **`crate::test_executable::write`** in the tests, which is
  `crates/test-support/executable.rs`, included by `#[path]` in the crate's
  `lib.rs` under `#[cfg(all(test, unix))]` as `spaceport-client` and
  `spaceport-usage` already include it. Nothing else about the tests changed.

`setup::Error` gained two variants, because the module now owns I/O the app's
root error used to absorb: `Client(#[from] spaceport_client::Error)` for
`resolve_destination` and `wait::exit`, and `Io(#[from] std::io::Error)` for
spawning the CLI and its drain threads. Both are `#[error(transparent)]`, so no
message changed; the app's mapping delegates the first to
`spaceport_client::Error`'s own words and shows the second verbatim, exactly as
the app root's `Client` and `Io` do.

## 3. Wiring, and what stayed app-side

- The app root `Error` drops `Devices`; both setup errors now travel through its
  existing `Remote(#[from] spaceport_remote::Error)` variant, and
  `error/remote.rs` gained `impl From<spaceport_remote::setup::Error> for Error`
  beside the host-script one it already had. `i18n/errors/remote.rs` delegates
  `Error::Setup` to it; `i18n/errors/root.rs` loses its `Devices` arm.
- **No facade.** `menu/devices.rs` loses `mod setup;` and `mod error;` and
  nothing re-exports either: `host_menu.rs` imports `spaceport_remote::setup`
  itself, which its `setup::` call sites already spelled, and the app's
  `Devices` error re-export has no production consumer left once the error type
  moved. The two test sites that built `crate::menu::devices::Error::Busy` now
  build `spaceport_remote::setup::Error::Busy` through the same `From`.
- `host_menu/tests.rs`, `host_menu/tests/mute.rs` and
  `i18n/errors/tests/devices.rs` changed only their imports. The i18n test
  splits the delegated variants out: `Client` goes through `assert_localized`
  with the rest, `Io` through `assert_verbatim`, the check the app root uses for
  the system's own text.

App lines `menu/devices/setup.rs` + `error.rs` + the setup tests: 373 removed;
crate: 372 added across the three files. `impl SpaceportWindow` blocks and
`fl!` sites are unchanged (the words stayed).

## 4. Validation run

- `cargo check -p spaceport-remote --all-targets` — clean.
- `cargo clippy -p spaceport-remote --all-targets --all-features -- -D warnings`
  — clean.
- `cargo test -p spaceport-remote` — 14 passed, including the four moved
  `setup::tests`.
- `cargo clippy -p spaceport-gpui --all-targets --features integration-test --
  -D warnings` — clean (`SPARKLE_FRAMEWORK_PATH` acquired through
  `scripts/release/sparkle_framework.py`).
- `cargo test -p spaceport-gpui --bin spaceport -- devices menu` — 344 passed.
- `cargo test -p spaceport-gpui --bin spaceport -- i18n::errors` — 58 passed,
  the new mapping's own check included.
- `cargo fmt --all`, `just check-file-size`, `just check-sorted` — clean.

Native QA not run: this moves no pixel and changes no words, and no window was
opened.
