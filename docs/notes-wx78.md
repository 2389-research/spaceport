# wx78: `spaceport-keymap`, the keymap's stage 2 — the resolution moves

Branch `feat/wx78-keymap-2`, cut from `fded31b78a` (main after #239, stage 1).
This is stage 2 of `docs/notes-keymap-extraction.md`, with one approved change
to how the stages divide (§2). No behaviour change: no `fl!` string, no
keybinding, no diagnostic and no paint line differs, and the `spaceport keys
--json` reply is byte for byte what it was.

## 1. What moved

| From `crates/spaceport-gpui/src` | To `crates/spaceport-keymap/src` | lines |
| --- | --- | --- |
| `keymap.rs` (`Keymap`, `Shortcut`, `PaneKey`, `with_overrides`, `layer`, `default_pane_keys`, `resolve_pane_keys`, `parse`, `typed_matches`, `has_modifier`, `identity`, the queries) | `keymap.rs` | 490 |
| `keymap/custom.rs` (`Reach`, `CustomBinding`, `available_commands`, the `impl Keymap` reach: `custom_command`, `custom_bindings`, `holds_directly`, `reach`, `bound_directly`, `direct_keystrokes`, `custom_label`, `unsupported_label`, `prefix_word`) | `custom.rs` | 281 |
| `keymap/error.rs` (13 variants) | `error.rs`, beside stage 1's `Daemon` | 56 |
| `terminal.rs`'s `reaches_pane` and the `Keystroke`→`ClientKeyCode` table (509-513, 581-628) | `pane.rs` | 54 |
| `actions.rs`'s `EDIT_KEYS`, `SYSTEM_KEYS`, `reserved_keystrokes`, `free_system_keys` | `reserved.rs` | 51 |
| `keymap/tests.rs` (15 tests and the fixtures) | `keymap/tests.rs` | 542 |
| `keymap/tests/{custom_defaults,custom_reference,daemon_layering,literal_plus,pane_keys,plugin_actions}.rs` (15 tests) | the same files | 380 |
| `platform_defaults.rs`, less the reserved-key test | `keymap/tests/platform_defaults.rs` (3 tests) | 104 |
| `shift_enter.rs`, less the serialization test | `keymap/tests/shift_enter.rs` (1 test) | 26 |
| `terminal/tests.rs::special_keys_and_text_are_separate` | `pane/tests.rs`, plus `reaches_pane`'s own assertions | 25 |
| — | `lib.rs` (re-exports and the crate doc) | 42 |

About 930 production and 1,050 test lines leave the app, and 126 lines of keymap
stay: the facade (19), the fixtures the two remaining tests read (44), and those
two tests (63).

The app keeps, unchanged, every consumer path: `crate::keymap::{Keymap,
DaemonKeys, Binding, PaneKeys, Reach, available_commands, HOST_OS,
parse_keystroke, secondary, Error, daemon}` is a re-export list, so
`palette.rs`, `menu/settings.rs`, `menu/shortcut_editor.rs`,
`control/server/settings/keys.rs`, `window/prefix.rs`, `window/server_keys.rs`,
`window/input.rs`, `chat/window.rs`, `window/status_bar.rs`, `config.rs` and the
~30 test files name the same things they named before. `actions.rs` and
`terminal.rs` are the two that now name the crate directly, because what they
call moved out from under them.

## 2. The one change to the plan: `custom.rs` and the reserved keys came too

Stage 2 as designed moves `keymap.rs` and leaves `keymap/custom.rs` for stage 3.
That is not possible, and the reason is the language rather than a preference:
`custom.rs:65-243` is `impl Keymap { … }`, and Rust forbids an inherent impl for
a type defined in another crate (E0116). The type moves, so the block moves with
it. `custom.rs:203` also reads the app's reserved keys
(`crate::actions::reserved_keystrokes()`), so the crate needs those too.

Three ways out were weighed before writing any code:

1. **Fold `custom.rs` and `reserved.rs` into stage 2** (chosen, with the
   supervisor's approval). Crate call sites do not change; the four
   `[[keys.command]]` test files the brief already listed for stage 2 can move;
   stage 3 shrinks to re-pointing the reserved-key assertion and answering the
   two `cfg`/`test-support` questions the design names.
2. Rewrite `custom.rs`'s block as free functions over `&Keymap`, then convert
   them back to methods in stage 3. Rejected: ~8 production and ~15 test call
   sites rewritten twice for no net change.
3. Keep `Keymap` in the app. Rejected: that is not stage 2.

`docs/notes-keymap-extraction.md` §4's stage table is now one stage short of the
work; the supervisor amends it in a separate docs commit, and this branch does
not touch the design note.

## 3. Survey decision 1: `key_code` is exported, not copied

`key_code` (`terminal.rs:581-628`) had exactly two production callers,
`reaches_pane` and `keystroke_input`, plus `terminal/tests.rs`. Moving
`reaches_pane` into the crate left the app still needing `key_code` for
`keystroke_input`, so both are `spaceport_keymap::pane`'s and the app imports
them (`terminal.rs`'s `use spaceport_keymap::{PaneKey, key_code}`). No third
caller appeared, and the `Keystroke`→`ClientKeyCode` table exists once.

`reaches_pane` is no longer named anywhere in `terminal.rs` (only the crate's own
`resolve_pane_keys` and test use it), so the app's `reaches_pane` wrapper is gone
and `terminal/tests.rs`'s assertions on it moved into `pane/tests.rs`.

## 4. Survey decision 2: `Keymap` keeps `Clone + Debug + PartialEq + Eq`

`palette.rs:510` and `window/server_keys.rs:105` compare two `Keymap`s and
`server_keys.rs:105` clones one, so the derives stay exactly as they were. The
same holds for `PaneKey` and `Shortcut`.

## 5. Survey decision 3: `PaneKey` stays `pub`

`terminal.rs:14,502` (a `[pane_keys]` target, and `PaneKey::{Key, Text}` in
`pane_key_input`) and `config/tests/keybindings.rs` read it, so it is exported
from `keymap.rs` and re-exported at the crate root. Every other name that
crossed widened only as far as its caller needed:

| item | visibility | why |
| --- | --- | --- |
| `Keymap::{with_overrides, pane_key, prefix_label, is_prefix, chord, triggers, navigates_workspace}` | `pub` | `config.rs`, `window/input.rs`, `chat/window.rs`, `palette/keys.rs`, `window/prefix.rs`, `window/status_bar.rs`, `window/zen.rs` |
| `Keymap::{custom_command, custom_bindings, holds_directly}`, `Reach`, `CustomBinding`, `available_commands` | `pub` | `window/prefix.rs`, `palette.rs`, `menu/settings.rs`, `control/server/settings/keys.rs` |
| `identity`, `typed_matches` | `pub`, re-exported under `test-support` | `custom.rs` and the app's one remaining catalog-default test |
| `Keymap::{shortcuts, prefixes}` fields, `Shortcut` and its fields, `MAX_KEYSTROKES`, `has_modifier` | `pub(crate)` | `custom.rs` reads the resolution it was written against |
| `DEFAULT_PANE_KEYS`, `PORTABLE_PANE_KEYS`, `MAX_PANE_KEYS`, `MAX_PANE_TEXT`, `TEXT_PREFIX`, `layer`, `default_pane_keys`, `resolve_pane_keys`, `parse` | private | the crate's own resolution and its tests |

## 6. `ServerKeybindings` retires; one failure keeps one definition

Stage 1 kept the app's `ServerKeybindings` variant because `with_overrides` was
still the app's and the chain at `window/server_keys.rs` mixed two owners. Both
answers are the crate's now, so the chain is one `Result` and the map happens
once, after it:

```rust
let keymap = DaemonKeys::from_profile(profile)
    .and_then(|keys| Keymap::with_overrides(overrides, pane_keys, &keys))
    .map_err(crate::Error::from);
```

`crate::Error::Keymap(#[from] spaceport_keymap::Error)` is the surviving variant,
and the crate root carries the 13 resolution variants beside `Daemon` — the
`spaceport-palette`/`spaceport-settings` shape, one root `Error` per crate:

```rust
// crates/spaceport-keymap/src/error.rs
Daemon(#[from] crate::daemon::Error),
DuplicateKeystroke { … },
…
UnsendablePaneKey { from: String, to: String },
```

`i18n/errors/keymap.rs` therefore has one `impl Localize for
spaceport_keymap::Error` with a `Daemon` arm delegating to `daemon::Error`,
where stage 1 had two impls. No `.ftl` entry is added or removed (2,114 messages
before and after) and `i18n/errors/root.rs` loses its `ServerKeybindings` arm.
One assertion is new:
`a_wrapped_profile_failure_reads_as_its_own`, because the wrapping arm is now
the only path a host profile's failure takes to a person.

## 7. The `test-support` exports, gated exactly

Three names leave the crate only for its consumers' tests, and each gate is the
narrowest that compiles:

| crate export | gate | reader |
| --- | --- | --- |
| `secondary_key` | `cfg(any(test, feature = "test-support"))` | the ~30 link tests, through `crate::keymap::secondary_key` (`#[cfg(test)]` there) |
| `identity`, `typed_matches` | `cfg(any(test, feature = "test-support"))` | `keymap/tests/platform_defaults.rs`'s reserved-key test |
| `free_system_keys` | `cfg(any(target_os = "macos", feature = "test-support"))` | `actions.rs`'s `bind_keys` on macOS, `actions/tests.rs` everywhere |

`spaceport-gpui`'s `[dev-dependencies]` already enabled `test-support` from
stage 1. The crate also gains a `spaceport-commands` dev-dependency with
`features = ["test-support"]`, for the platform's spelling of a catalog default
(`platform_default`) that the moved tests compare against — the same shape
`spaceport-gpui` uses for the same reason.

The facade's own gates follow its readers: `parse_keystroke` is
`cfg(any(test, feature = "integration-test"))`, because nothing in a plain
build reads it (`keymap.rs` itself was its last production reader) but the
Linux live-GUI smoke module does.

## 8. The two app-side tests that stayed, and where the fixtures went

Per the brief, two tests stay app-side because they check the app, not the
resolution:

| test | why it cannot move |
| --- | --- |
| `keymap/tests/platform_defaults.rs::platform_defaults_are_unique_and_leave_edit_keys_available` | it asserts the catalog's defaults against `crate::actions::reserved_keystrokes()`, and stage 3 re-points it |
| `keymap/tests/shift_enter.rs::shift_enter_sends_alt_enter_on_every_platform` | it asserts the serialized key event through `crate::terminal::pane_key_input`, which stays the app's |

They keep the fixtures they read (`keystroke`, `overrides`, `no_keys`, `with`,
`sent`, `key`) in `keymap/tests.rs`, now 44 lines instead of 586. The other 32
tests moved with the code they test; `platform_defaults.rs`'s three
platform/routing tests and `shift_enter.rs`'s removal-and-replacement test went
to the crate, since neither reads an app type.

## 9. The boundary check, worded so it means something

```sh
rg -n '\b(App|Window|Context|Entity|Element)\b' crates/spaceport-keymap/src   # no hits
```

The unanchored form still matches `OperatingSystem::Windows` in `host.rs`, as
stage 1 recorded. The crate names `Keystroke`, `Modifiers`, `OperatingSystem`,
`InvalidKeystrokeError`, `KeybindingKeystroke` and `Keystroke::parse_with_os` in
eight modules now (`pane.rs` and `reserved.rs` each name `Keystroke` alone) —
the same three data values and one error — and no `App`, `Context`, `Entity`,
`Element`, window or paint.

## 10. Validation

Run on this machine (macOS, `CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0`,
`SPARKLE_FRAMEWORK_PATH=$PWD/target/sparkle-sdk`):

| command | result |
| --- | --- |
| `cargo test -p spaceport-keymap` | 54 passed, 0 failed |
| `cargo test -p spaceport-gpui --bin spaceport` | 3349 passed, 0 failed, 10 ignored |
| `cargo check -p spaceport-keymap` / `--features test-support` | clean, both |
| `cargo clippy -p spaceport-keymap --all-features --all-targets` | clean |
| `cargo clippy -p spaceport-gpui --all-targets --all-features` | clean |
| `cargo fmt --all -- --check` | clean |
| `just check-file-size` | all files within 1,000 lines (largest: `keymap/tests.rs`, 587) |
| `just check-sorted` | clean |
| `just check-i18n` | 2,114 messages in 114 files, valid and used (unchanged) |
| `rg -n '\b(App\|Window\|Context\|Entity\|Element)\b' crates/spaceport-keymap/src` | no hits |

Deferred to CI: `just test-gui <herdr>` (prefix mode, the Keybinds page and the
shortcut editor need a desktop), `just lint-linux` / `just lint-windows` (the
`cfg(target_os)` branches in `host.rs`, `keymap.rs`'s macOS-only default pane
keys and `reserved.rs`'s macOS system keys), and the full workspace gates.

The `spaceport keys --json` answer was checked by reading the path and by its
tests rather than by a live socket: `control/server/settings/keys.rs` is
untouched, `Reach`, `Keymap::shortcuts` and `custom_bindings` keep their
signatures, and `control::server::settings` passes.

## 11. What stage 3 still has to decide

- Re-point `platform_defaults.rs`'s reserved-key test at
  `spaceport_keymap::reserved::reserved_keystrokes` and move it into the crate,
  which drops the last app-side keymap test but the serialization one.
- The two `cfg` questions the design asks, now with answers in place: the
  reserved list reads `cfg!(target_os = "macos")` inside `reserved_keystrokes`
  and `free_system_keys` is a `target_os`-or-`test-support` export, so the
  `windows` CI leg is the check that neither leaks.
- `docs/notes-keymap-extraction.md` §4's stage table, which the supervisor
  amends separately.
- Stage 4 (flattening `crate::keymap::X` in ~70 files) is untouched and still
  the right last step.
