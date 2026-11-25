# wx77: `spaceport-keymap`, the keymap's stage 1

Branch `feat/wx77-keymap-1`, cut from `d8c49af364` (main after the design commit
`docs(spaceport): Record the keymap extraction design`). This is stage 1 of
`docs/notes-keymap-extraction.md`: the crate exists, and the daemon half moves
into it. No behaviour change — no `fl!` string, no keybinding, no diagnostic and
no paint line differs.

## 1. What moved

| From `crates/spaceport-gpui/src` | To `crates/spaceport-keymap/src` | lines |
| --- | --- | --- |
| `keymap/daemon.rs` (`ACTIONS`, `Target`/`Indexed`, `Trigger`, `DaemonKeys`, `entries`, `prefixes`, `triggers`, the Herdr→GPUI translator, `set_modifier`) | `daemon.rs` | 543 |
| `keymap/daemon/error.rs` (`Missing`, `NoKeys`, `Parse`, `TooLarge`) | `daemon/error.rs` | 15 |
| `keymap/host.rs` (`HOST_OS`, `parse_keystroke`, `secondary`, `secondary_key`) | `host.rs` | 40 |
| — | `lib.rs` (the crate doc and re-exports), `error.rs` (the crate root) | 40 |
| `keymap/daemon/tests.rs` (13 tests, less the layering half of one) | `daemon/tests.rs` | 388 |
| `keymap/daemon/tests/detach.rs` (2 tests, less the layering half of one) | `daemon/tests/detach.rs` | 25 |
| `keymap/daemon/tests/literal_plus.rs` (5 tests) | `daemon/tests/literal_plus.rs` | 82 |

600 production and 495 test lines leave the app; 40 of the production lines are
new (the crate doc and the root `Error`), and the rest is the move plus the four
`pub` widenings §5 records. The 20 daemon tests are unchanged apart from §4's
split and the retargeted assertions.

App-side: `keymap.rs` keeps its contents and gains the façade, `error.rs`'s
`ServerKeybindings` takes the crate's root error, `window/server_keys.rs` maps
one failure at its call site, and `i18n/errors/keymap.rs` words the crate root.
`AGENTS.md` gains the crate's row.

The module split is kept: `daemon.rs` (543) + its tests are the largest files, and
stage 2's `keymap.rs` (507) + `custom.rs` (281) land beside them, so no file
crosses the 1,000-line limit on this move or the next.

## 2. Survey decision 1: the crate root, and `ServerKeybindings` stays

The crate's root error gains the variant the design names:

```rust
// crates/spaceport-keymap/src/error.rs
#[error(transparent)]
Daemon(#[from] crate::daemon::Error),
```

The app's root `ServerKeybindings` variant **stays**, retargeted from
`crate::keymap::daemon::Error` to `spaceport_keymap::Error`. The design offered
both (§1.4, §4 stage 1) and asked for the evidence; the evidence says keep:

- **The one real consumer breaks when it retires.** `ServerKeymap::build`
  (`window/server_keys.rs:36-37`) chains the daemon's profile read with this
  GUI's own `Keymap::with_overrides` through `.and_then`, so both failures share
  one `Result`. `with_overrides` is still the app's until stage 2, and it answers
  the app's error. Retiring `ServerKeybindings` therefore forces either a second
  definition of the same failure — a `Daemon(#[from] spaceport_keymap::Error)`
  arm on the app's 13-variant `keymap::Error`, which stage 2 deletes again — or a
  `match` where the chain is. Keeping the root variant is one `map_err`
  (§3 below), one changed line in `error.rs`, and one changed line in
  `root.rs:126`, which keeps reading `Self::ServerKeybindings(error) =>
  error.localize()`.
- **It is the app's own convention.** `Sound(#[from] spaceport_sound::Error)`,
  `Menu(#[from] spaceport_menu::Error)` and `Settings(#[from]
  spaceport_settings::Error)` are the same shape: one crate error, wrapped once,
  worded through its own `Localize`. Nothing is mirrored here — the four daemon
  variants are still defined once, in `spaceport_keymap::daemon::Error`.
- **The name still fits.** In stage 1 the crate root wraps exactly one failure,
  the profile a device published, so `ServerKeybindings` names what it carries.
  Stage 2, which moves `keymap::Error` into the crate and points the app's
  `Keymap` variant at `spaceport_keymap::Error`, is where it becomes redundant;
  retiring it there is a one-line change with no new decision.

Words: `i18n/errors/keymap.rs` gains `impl Localize for spaceport_keymap::Error`
with its one `Daemon` arm delegating to `impl Localize for daemon::Error`, which
is unchanged (17 message IDs, no new `.ftl` entry, no `fl!` site moved). The one
test assertion that names the wrapping — `window/server_keys/tests.rs:132-139` —
now names `Error::ServerKeybindings(spaceport_keymap::Error::Daemon(..))`. The
other assertions that name the type are the four in the moved daemon test, which
are about the crate root now (§4).

## 3. Survey decision 2: `from_profile` answers the crate's `Result`, mapped at the call site

`DaemonKeys::from_profile` now returns `spaceport_keymap::Result<Self>`. The one
call site keeps the app's `Result` and maps once:

```rust
// crates/spaceport-gpui/src/window/server_keys.rs
let keymap = DaemonKeys::from_profile(profile)
    .map_err(crate::Error::from)
    .and_then(|keys| Keymap::with_overrides(overrides, pane_keys, &keys));
```

The crate's `Result` does not flow further because the chain mixes two owners: the
profile read is the crate's and the `[keybindings]`/`[pane_keys]` resolution is
still the app's. One conversion at the boundary is the whole seam; letting the
crate's error flow would mean converting `with_overrides`'s answer into it
instead, which is the same conversion pointing the other way and would put a
stage-2 problem in stage 1's diff. Stage 2, where both live in the crate, deletes
the `map_err`.

## 4. Survey decision 3: the four profile assertions stay in the crate

`daemon/tests.rs`'s `an_unusable_server_profile_is_an_error` reads what it
asserts: which daemon variant each unusable profile answers with (`Missing`,
`Parse`, `NoKeys`, `TooLarge`), and that the parse failure keeps its source
chain. That is the reader's business, not the app's diagnostic — so it stayed,
retargeted from `crate::Error::ServerKeybindings(..)` to
`crate::Error::Daemon(..)`, and it now proves the crate root wraps the daemon
error. The app's own diagnostic (`server_keybindings_error`'s English text, and
the variant behind it) is already asserted in
`window/server_keys/tests.rs::an_unusable_profile_falls_back_to_local_and_says_why`,
which stays where it is.

Two tests that did reach into the app moved out, and are why the test count grew
by two:

| test | what split |
| --- | --- |
| `daemon/tests.rs::user_bindings_precede_defaults` | the binding order is the crate's (`bindings[first].0`); the chord an app `Keymap` gives the key is not |
| `daemon/tests/detach.rs::detach_reads_the_remote_profile_and_does_not_override_configured_chords` | that a profile reads like the local table is the crate's; that a configured chord answers before a default one is not |

Each crate half stayed (renamed `detach_reads_the_remote_profile_like_the_local_table`
where the old name promised both halves), and the `Keymap::with_overrides` halves
became `crates/spaceport-gpui/src/keymap/tests/daemon_layering.rs`, a topic file
as AGENTS.md asks. Stage 2 takes them back into the crate with the resolution.

## 5. The `pub` widenings, and why each is minimal

Crossing a crate boundary has no `pub(super)`. Four things widened, and nothing
else did:

| item | why | stage 2 |
| --- | --- | --- |
| `DaemonKeys::{prefixes, bindings, navigate_up, navigate_down}` | the app's `Keymap::layer` reads all four, and `keymap/tests.rs` builds one by struct literal | narrow back to `pub(crate)`-in-crate when `layer` moves in |
| `DaemonKeys::{from_profile, from_table}`, `Default` | `config.rs`, `window/server_keys.rs`, `window/prefix.rs` and the shortcut editor call them | unchanged |
| `Trigger`, `triggers` | `keymap/custom.rs:104,272` reads the daemon's custom-command spellings through `triggers`, and `Trigger` is what a binding is | unchanged |
| `host::{HOST_OS, parse_keystroke, secondary}` | 30-odd app and test sites print, parse and test the platform modifier | moves with `keymap.rs` |

`Target`, `Indexed`, `ACTIONS`, `entries`, `prefixes`, `keystroke`,
`modifier_combo`, `set_modifier`, `DEFAULT_PREFIX`, `MAX_ENTRIES` and
`MAX_PROFILE_BYTES` are still private to `daemon.rs`.

## 6. Risk 6: `secondary_key` is a `test-support` export

`secondary_key` was `#[cfg(test)]` in the app's own module, where other modules'
tests could read it. Across a crate boundary that gate is wrong — a dependency's
`cfg(test)` is false inside its consumer's test build — so it is now

```rust
#[cfg(any(test, feature = "test-support"))]
pub fn secondary_key() -> Modifiers
```

exported the same way from `lib.rs`, with `test-support = []` in the crate's
`[features]` (the `spaceport-sidebar` pattern) and
`spaceport-keymap = { features = ["test-support"], workspace = true }` added to
`crates/spaceport-gpui`'s `[dev-dependencies]`. The app's façade keeps the old
gate on its re-export — `#[cfg(test)] pub(crate) use spaceport_keymap::secondary_key;`
— so the ~30 link-test sites are untouched. Both passes were built to prove it:
`cargo check -p spaceport-gpui` (no dev-dependencies, no `secondary_key` at all)
and `cargo check -p spaceport-gpui --all-targets` (the app's tests, feature on),
plus the same pair with `--features integration-test`.

## 7. The boundary check, worded so it means something

`rg -n 'App|Window|Context|Entity|Element' crates/spaceport-keymap/src` cannot be
empty for a reason that has nothing to do with GPUI's runtime: `Window` matches
inside `OperatingSystem::Windows`. Run with word boundaries —

```sh
rg -n '\b(App|Window|Context|Entity|Element)\b' crates/spaceport-keymap/src   # no hits
```

— it is empty. The one hit the unanchored form had was `host.rs`'s doc sentence
about reading the OS "without an `App`", now "which lets an input path below the
app read it too": the same claim, and the survey is a check again.

What the crate does name is `Keystroke`, its `Modifiers`, `OperatingSystem`,
`InvalidKeystrokeError` and `Keystroke::parse_with_os` — three data values and one
error, exactly what the design's §0.2 measured — and no `App`, `Context`,
`Entity`, `Element`, window or paint.

## 8. Validation

Run on this machine (macOS, `CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0`):

| command | result |
| --- | --- |
| `cargo test -p spaceport-keymap` | 20 passed, 0 failed |
| `cargo test -p spaceport-gpui --bin spaceport` | 3382 passed, 0 failed, 10 ignored |
| `cargo check -p spaceport-gpui` | clean |
| `cargo check -p spaceport-gpui --all-targets` | clean |
| `cargo check -p spaceport-gpui --all-targets --features integration-test` | clean |
| `cargo clippy -p spaceport-gpui --all-targets --features integration-test` | clean |
| `cargo clippy -p spaceport-keymap --all-features --all-targets` | clean |
| `cargo check -p spaceport-keymap --no-default-features` / `--features test-support` | clean, both |
| `cargo fmt --all -- --check` | clean |
| `just check-file-size` | all files within 1,000 lines |
| `just check-sorted` | clean |
| `just check-i18n` | 2,114 messages in 114 files, valid and used |
| `rg -n '\b(App\|Window\|Context\|Entity\|Element)\b' crates/spaceport-keymap/src` | no hits |

Deferred to CI: `just test-gui <herdr>` (the device menu's three failures are
asserted headlessly here, but the native menu and prefix mode need a desktop),
`just lint-linux` / `just lint-windows` (`HOST_OS`'s three platform branches),
and the full workspace gates. `just check-spec` was not run: nothing under
`docs/spaceport/` changed.

## 9. What stage 2 still has to decide

Named here so stage 2 starts from the same place this one did: `PaneKey`'s
visibility (`terminal.rs:14,502` and `config/tests/keybindings.rs:240-243` read
it, so `pub`), whether `key_code` (`terminal.rs:581-628`) keeps exactly two
callers, and the retirement of `ServerKeybindings` §2 leaves for it. The daemon
half as it stands names `spaceport_commands::Command`, `toml`, `gpui` and
`thiserror`, so the crate's dependency list grows by `spaceport-client` and
`spaceport-settings` when `keymap.rs` and `custom.rs` arrive.
