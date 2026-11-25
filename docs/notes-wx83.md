# i18n stage 1: the lookup moves into `spaceport-i18n`

The owner's ruling (`docs/notes-owner-decisions-2026-11-25.md`, ruling 2): the
translation lookup lives in `spaceport-i18n`, crates hold the labels' keys, and
the i18n crate holds the words. This is that work's stage 1: land the home and
change **zero call sites**. Base revision `bb148e4889` (origin/main). Worktree
branch `feat/wx83-i18n-1`.

It changes no behaviour. Every `crate::i18n::fl!` and every other `i18n::…`
call in the app keeps working through a facade.

## What moved

| was | now |
| --- | --- |
| `crates/spaceport-gpui/locales/en-US/*.ftl` (114 files) | `crates/spaceport-i18n/locales/en-US/` |
| `crates/spaceport-gpui/locales.rs` (the build script's module) | `crates/spaceport-i18n/locales.rs` |
| — | `crates/spaceport-i18n/build.rs` (`mod locales;`, `locales::generate`) |
| `crates/spaceport-gpui/tests/locales.rs` | `crates/spaceport-i18n/tests/locales.rs` |
| `src/i18n/check.rs` (`check`, the compile-time id/arg check) | `spaceport-i18n/src/check.rs` |
| `src/i18n/current.rs` (the catalog shown, per test thread) | `spaceport-i18n/src/current.rs` |
| `src/i18n.rs`'s `text`, `message`, `english`, `format`, `takes`, `available`, `endonyms`/`endonym` | `spaceport-i18n/src/lookup.rs` |
| `src/i18n.rs`'s `macro_rules! fl` | `spaceport-i18n/src/lib.rs` |
| `src/i18n/tests/macros.rs` (both tests) | `spaceport-i18n/src/tests/macros.rs` |
| `src/i18n/tests/messages.rs`' third test (embedded files, sorted table, `available`) | `spaceport-i18n/src/tests/messages.rs` |
| — | `spaceport-i18n/src/tests/current.rs` (the per-thread catalog) |

What stayed in `crates/spaceport-gpui/src/i18n.rs`: `CurrentLocale` (the GPUI
global), `install`/`follow_config`/`toggle_pseudo`/`switch`/`show`,
`shown_pseudo`, the catalog command's two names (`command`,
`command_english`), and re-exports of the moved lookup. `src/i18n/current.rs`
keeps one function, `replace`, which is the only place the shown locale's
direction is published to `spaceport-widgets`.

The app's build script loses `mod locales;`, its `rerun-if-changed=locales.rs`
line, `BuildError::Locales`, and `BuildError::MissingOutDir` (nothing reads
`OUT_DIR` there now). `tests/build_identity.rs` no longer writes a `locales.rs`
stub into its fixture crate.

## The macro's `$crate`

`$crate` is now `spaceport_i18n`, so the two things the expansion names outside
the crate are reachable there:

- `spaceport_i18n::__private::check` — the const `check`, `#[doc(hidden)]`,
  re-exported from `pub mod __private`.
- `spaceport_i18n::__private::{FILES, MESSAGES}` — the generated table, now
  written inside that module with `crate::File` (a crate cannot name itself).
- `spaceport_i18n::text` and `spaceport_i18n::FluentValue` — both documented
  public items.

`crate::i18n::check` is gone from the app, except that the facade re-exports
`spaceport_i18n::__private::check` for the one call site that legitimately names
a message id by *constant*: `menu/workspace/popover.rs`'s `Label::new`, a
`const fn`. That keeps the zero-call-site rule; it is the same check `fl!` runs.

## `test-support`, not `cfg(test)`

`cfg(test)` does not cross crates: inside a consumer's test build the i18n
crate is compiled with `cfg(test) == false`. So the per-thread catalog is behind
`cfg(any(test, feature = "test-support"))` (the global `RwLock` is
`cfg(not(any(test, …)))`), following `spaceport-activation` and
`spaceport-widgets`, and `crates/spaceport-gpui/Cargo.toml`'s dev-dependencies
enable `spaceport-i18n/test-support`. `src/tests/current.rs` pins it: a thread
that switches to `en-XA` does not change the caller's text.

`takes` is test-only in the same way: `cfg(any(test, feature = "test-support"))`
in the crate, re-exported by the app's facade under `#[cfg(test)]`.

## The public surface

Documented: `fl!`, `text`, `message`, `english`, `takes`, `shown_locale`,
`set_locale`, `available`, `endonym`, plus the types the crate already had
(`Catalog`, `File`, `Locale`, `Language`, `Direction`, `Pseudo`, `CjkScript`,
`Error`, `Result`, `FluentValue`, `FluentArgs`, `negotiate`, `system_languages`,
`pseudo`).

Deviations from the plan's "nothing else" list, and why:

- **`available` and `endonym`** must be public: the app's language picker and
  `CurrentLocale::resolve` read them, and the zero-call-site rule keeps
  `i18n::available()`/`i18n::endonym()` working through the facade.
- **`shown_pseudo`** stays an app function (one line over `shown_locale()`) since
  it knows nothing the crate does.
- **`endonyms`** (plural) stayed private; only `endonym` is public.

`spaceport-i18n` gained one dependency, `tracing`, for the warning `load` already
logged when a catalog cannot be built — `spaceport-client`, `spaceport-live`,
`spaceport-settings` and others depend on it already.

## Zero call sites

`rg -c '\bfl!'` per file under `crates/spaceport-gpui/src` is unchanged except
for the four files that moved or lost the macro definition:

| file | before | after |
| --- | --- | --- |
| `src/i18n.rs` | 4 | 2 (both comments; the macro and its `use` left) |
| `src/i18n/check.rs` | 5 | moved out |
| `src/i18n/tests/macros.rs` | 5 | moved out |
| `src/i18n/tests/messages.rs` | 2 | 1 (the third test moved out) |

Total 1949 → 1936 lines; every other file matches exactly.

## Checks

- `scripts/check-i18n.py`: the plist/installer filter is now
  `("crates", "spaceport-i18n")`, and its docstring names the i18n crate's
  `locales.rs`. `crates/spaceport-i18n/src/lib.rs` carries `// i18n: migrated`,
  so the crate's surface may hold no English literal.
- `scripts/release/tests/*` pointed at `crates/spaceport-gpui/locales` in twelve
  places (the desktop copy guard, the absence tests, one `test_ci_changes`
  fixture path). All now name `crates/spaceport-i18n/locales`.
- `docs/i18n.md`: "Where things are", the message path in "Adding a message",
  the build that validates a language file, "Adding a language" step 1, and the
  ownership paragraph (the words and the lookup are `spaceport-i18n`'s; the app
  embeds nothing). `AGENTS.md`'s crate table is left for the supervisor.
- `available()` still returns only `en-US`: contractual, not behavioural.

## Not done here

Stage 2 (delete the `Labels` seams one crate per commit), stage 3 (move each
`impl Localize` beside its `Error`, ≈537 sites), and stage 4 (menu stage 6) are
untouched.
