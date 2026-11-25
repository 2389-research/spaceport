# i18n stage 3: each crate words its own errors

The third stage of the owner's P1 ruling
(`docs/notes-owner-decisions-2026-11-25.md`): the translation lookup moved into
`spaceport-i18n` (#250), the `Labels` seams are gone (#253), and each crate that
returns an `Error` now also words it. Base revision: `f782f94231` (origin/main).
Branch `feat/wx85-i18n-3`.

It changes no behaviour but for one English message a pre-existing divergence
made wrong (§4). Every id keeps the `.ftl` file it had; what changes is who
reads it.

## 1. Why the staging changed: the orphan rule

The plan was one commit per crate. That is impossible here, and the reason is
worth recording.

`Localize` was the app's own trait (`crates/spaceport-gpui/src/i18n/errors.rs`),
so the app could implement it for any type — including
`spaceport_client::Error`, whose `Error` it does not own. Rust's orphan rule
allows that only while the trait is local. Move the trait into
`spaceport-i18n` — which the plan also asked for, and which is what lets a crate
implement it at all — and the app's cross-crate impls stop compiling:

```text
error[E0117]: only traits defined in the current crate can be implemented for
              types defined outside of the crate
```

Two ways out: move the trait and every cross-crate impl in one commit, or keep
the app's trait alive beside the new one for the length of the branch, with every
wording table existing twice. The owner chose the first (`contact_supervisor`,
this task): one honest move beats two copies of every table, even mid-pull-request.

So the shape is: **one atomic commit** carrying the trait and every crate's
mapping, then separate commits for the docs. A crate's mapping and the trait it
implements are one unit, because the compiler says so.

## 2. Where the trait and its checks live now

`crates/spaceport-i18n/src/localize.rs`:

- `Localize`, with the blanket impls for `&T`, `Box<T>`, and `Arc<T>` that let a
  wrapped error be passed where a plain one is expected. It is lookup machinery,
  not app UI, so it belongs below the app with `fl!` and `message`.
- `assertions`, behind `test-support` and `cfg(test)`: `assert_localized`,
  `assert_no_wire_words`, `assert_verbatim`, and `ISOLATION`. These were
  `i18n/errors/tests.rs`'s; they moved because the mappings did — every crate's
  tests need them, and a crate cannot reach into the app to get them.

The app re-exports the trait at the path its own code already named it by,
`crate::i18n::Localize`, and `i18n/errors/tests.rs` re-exports the three
assertions for the mappings that stay app-side. Nothing in the app had to change
a `use` line.

`with_causes` stays app-side: it walks `crate::Error`'s own cause chain and
names the app's error type directly.

## 3. What moved, and what stayed

Each crate gained a `localize` module — `src/localize.rs`, or `src/localize/`
with one file per error type where a crate returns several — and one
`spaceport-i18n` dependency. Whoever reads the crate's error reads the sentence
beside it:

| Crate | Worded there now |
| --- | --- |
| `spaceport-protocol` | `Error` |
| `spaceport-client` | `Error`, `local_path::PathError`, `clipboard::Error`, `sessions::Error`, `ssh::Error`, `SshFailure`, `terminal_attach::Error`, `wsl::Error` |
| `spaceport-chat-protocol` | `Error`, `Rejection` |
| `spaceport-chat-core` | `Error`, `env::Error`, `widgets::Error`, `session::Refusal` |
| `spaceport-browser` | `Error` |
| `spaceport-web-url` | `Error` |
| `spaceport-settings` | `Error`, `ThemeParseError`, `ssh_hosts::Error`, `tab_layout::Error`, `BackdropProblem` |
| `spaceport-control-protocol` | `Error` |
| `spaceport-remote` | `Error`, `host_script::Error`, `setup::Error` |
| `spaceport-endpoint` | `Error` |
| `spaceport-keymap` | `Error`, `daemon::Error` |
| `spaceport-menu` | `Error` |
| `spaceport-palette` | `Error` |
| `spaceport-usage` | `Error`, `shell::Error` |
| `spaceport-sound` | `Error` |
| `spaceport-state-file` | `Error` |

The app keeps the mappings for errors it defines itself — `backdrop`,
`caffeine`, `chat` (its own `Error`, `StartError`, `pasted`, `page`, `visual`),
`clipboard`, `config`, `daemon`, `endpoint`, `file_drop`, `launch_at_login`,
`listening_ports`, `notification_mutes`, `overview`, `pane_move`,
`port_forward`, `processes`, `root`, `wsl_drop` — in `i18n/errors/`, unchanged
but for the trait's path.

Two of the app's re-exports existed only for mappings that moved, and are now
test-only where a test still names the type: `chat::Refusal` and
`chat::widgets::Error`. `config::ThemeParseError` lost its app-side reader
entirely, and its test-only re-export went with it.

### Deviations from the survey's crate list

- `spaceport-protocol` gained a `spaceport-i18n` dependency although it is a
  wire-format crate. Its `Error` is what a window shows when a frame will not
  decode, so under the ruling it words itself. The dependency is the same
  gpui-free lookup the rest of the workspace uses; the crate still depends on
  nothing with a runtime.
- `spaceport-web-url` and `spaceport-palette` were not on the list either, but
  their `Error`s were mapped from `browser.rs` and `palette.rs` in the app, so
  they moved with the rest. `spaceport-web-url` gets the sentence for an
  address a page named; `spaceport-palette` gets all 21 of its failures.

## 4. One wording fix, found by the move

`error-state-file.ftl`'s `error-state-file-missing-root` did not match the
`Display` of the variant it words:

| | Text |
| --- | --- |
| `Display` (logs) | `state file /tmp has no folder` |
| message (window) | `The state file /tmp names no folder to save it in` |

`docs/i18n.md` requires the English message to read exactly as `Display` does,
and `assert_localized` enforces it. No test covered `MissingRoot` — the app's
`root` topic only asserted `Size` — so the drift survived. The message now reads
`state file { $path } has no folder`, matching the log line and this file's
lowercase convention (as `error-state-file-size` already did).

This is the one user-visible text change in the stage. Nothing else about any
message changed: all 496 ids the app's `i18n/errors/` named are still named, and
the workspace has exactly the 1,936 `fl!` call sites it had before.

## 5. Tests

Each crate's mapping is checked by a topic file under its `localize` module
(`src/localize/tests.rs`, or `src/localize/tests/<topic>.rs`), following the
repository's test-topic convention. They call
`spaceport_i18n::assertions::{assert_localized, assert_verbatim,
assert_no_wire_words}` and switch locale with `spaceport_i18n::set_locale`,
which under `test-support` moves only that test thread's catalog — the crates
add `spaceport-i18n` with that feature to their dev-dependencies.

The app's 39 old error-test cases now run in 16 crates; the app's own 20 remain
in `i18n/errors/tests/`. Two pairs of topics merged where a crate words two
error types that one topic used to cover (`spaceport-usage`: root and host
shell; `spaceport-chat-core`: attach, refusals, cards, environment).

## 6. What the checks see

`just check-i18n` is unchanged at **2,114 messages in 114 files, valid and
used**. The tally does not move because `scripts/check-i18n.py` already counted
string literals across `crates/*.rs`, and a `fl!` call reads the same in
`spaceport-client` as it did in `spaceport-gpui`.

## 7. Verification

- `cargo check -p <crate> -p spaceport-gpui --all-targets` after each crate
- `cargo test` for all 17 touched crates: 1,435 passed, 0 failed
- `cargo test -p spaceport-gpui --bin spaceport`: 3,307 passed, 10 ignored
  (down 39 from 3,346, which is the 39 error-test cases now running in the crates)
- `just check-i18n` — 2,114 messages in 114 files, valid and used
- `cargo fmt --check`, `just check-file-size`
- Not run here: the native window checks and the `integration-test` probes that
  need a desktop and a pinned `herdr` (`just test-gui` / `just test-sidebar`);
  CI's platform jobs cover them. The headless `en-XA` leak tests ran in the same
  `--bin spaceport` pass.
