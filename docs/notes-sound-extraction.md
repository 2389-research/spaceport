# Sound extraction — implementation notes

Implements [`notes-sound-extraction-design.md`](notes-sound-extraction-design.md)
on branch `feat/wave8-sound`, one commit per step, in the design's order. Six
commits are on the branch; the last is a small follow-up for what the clippy
gate caught.

```
df0130de9 fix(sound): Keep the moved sound tests clippy-clean
694be8fb6 docs: Record the sound extraction in the split plan
92b256abf refactor: Move sound playback into sheprd-sound and drop rodio from sheprd-gpui
dc3669bc4 refactor: Move the sound policy and settings into sheprd-sound
dc31261aa fix(lenient): Drop the tests' unused import of the crate root        ← step 0b
16b088d88 refactor: Move lenient settings decoding into the sheprd-lenient crate ← step 0a
```

## Step 0 — `sheprd-lenient` was not on main

The design's step 0 says the crate "lands first" if it is missing, and it was
missing: `feat/wave7-extract` held it, unmerged. Rather than write a second
copy, I cherry-picked its two commits (`e0a49b0ce` and its fix `d1c88c37b`) and
resolved the conflicts against main's newer docs state (main has since moved
`words` and `osc52` out, and the cherry-pick's base still listed them). The
result is the crate as that branch wrote it, `sheprd-lenient`'s four `pub fn`s
and its test, plus the `Cargo.toml`/`AGENTS.md`/`docs/gpui-split-plan.md` edits.

- **If `feat/wave7-extract` merges first**, my step-0 commits are the same
  content and a rebase should drop them; if it lands after, the supervisor
  should expect the duplicate and drop one side.
- `docs/gpui-split-plan.md` therefore also records the `lenient` move (its row
  leaves "Next extractions", the "What moved" table gains it). That is beyond
  the design's step 3 (which names only `sound`), but the file would otherwise
  describe a module that has moved.

## Step 1 — policy and settings (`dc3669bc4`)

`crates/sheprd-sound` gains `error.rs`, `config.rs`, `policy.rs` and the shared
test fixtures; the app keeps `Service`, `Recent`, `Job`, the playback backend
and its own `sound::Error` for one more commit.

`Policy::poll(live: Live<'_>, delay, focused, now, emit)` and
`Service::poll(&self, policy, live, focused, now, silenced)` are as designed.
`Settings::load(path)` takes the file; `Service::new()` resolves the path once,
at construction, as section 5 requires.

Deviations:

- **`Live::connection_cancel` is `&'a Arc<AtomicBool>`, not `&'a AtomicBool`.**
  The design's struct says `&AtomicBool`, but the worker's `Job` owns an `Arc`
  for both cancel flags, so `Service::poll` must clone the `Arc`; you cannot
  clone an `Arc` from an `&AtomicBool`. `Policy` only calls `.load()` on it, so
  nothing else changes. (The design itself flags the same class of problem for
  `cancel`.)
- **`LiveState::sound_live(snapshot)`** is one method on the app's live state
  that builds the `Live`, used by `endpoint/polling.rs` and by the two staying
  tests. The design inlines the struct literal at the call site; a method keeps
  one definition for the three sites and keeps `&mut` field borrows local.
- **Test files stayed children of the module they drive**:
  `config/tests.rs`, `policy/tests.rs`, `service/tests{,/mutes,/play,/reload,
  /repeats,/volume}.rs`, `playback/tests{,/gain}.rs`, with `src/tests.rs`
  holding the fixtures (`Endpoint` for a `Live`, `event`). The design's
  `src/tests/{policy,playback,…}` layout cannot see `Policy::pending`,
  `Service::sender`, `Job` or `playback::wait`; reaching them would mean
  widening about eight internals to `pub(crate)` purely for tests, which
  AGENTS.md's narrow-API rule rules out. The bodies are otherwise unchanged, as
  the design's "unchanged apart from `super::*`" asks.
- **`Settings`, `SoundConfig`, `Policy::forget`, `ui_delay`, `path_for` and
  `allows` are `pub` for this commit only**, because the app's worker still
  names them, and become `pub(crate)` in step 2 as designed.
- `Policy::waiting()` is `#[cfg(any(test, feature = "test-support"))]`: the mute
  and detach tests assert a queued notification is dropped while it waits, and
  `pending` is private to `crate::policy`.
- `MAX_FILE_BYTES` is a named constant in `config.rs` (the app had
  `1_048_576`/`1_048_577` literals inline).

## Step 2 — playback, worker, and `rodio` leaving the app's direct dependencies (`92b256abf`)

`playback.rs` and `service.rs` move; `crates/sheprd-gpui/src/sound*` and
`mod sound;` are deleted. `Service::new(path: PathBuf, volume: u8)` takes the
settings file and the starting volume, and `app::shared_sound` passes
`config::view_settings_path(sheprd_client::local_var)` and
`config::SoundConfig::default().volume`.

A correction recorded after merge: `rodio` left `sheprd-gpui`'s *direct*
dependencies, not its build graph — the crate depends on `sheprd-sound`,
which depends on `rodio`, so the audio stack still builds and links with the
app. The extraction's win is ownership and `cargo test -p sheprd-sound`
without GPUI, not link size (`docs/gpui-split-plan.md` says this correctly
now).

- **Error shape** as section 3: `sheprd_sound::Error` owns the seven playback
  variants (the three `impl From<rodio::…> for crate::Error` blocks are gone)
  plus `Io`, `Path` and `Toml`; the app's root keeps
  `#[error(transparent)] Sound(#[from] sheprd_sound::Error)` in its sorted
  place. `Localize for sheprd_sound::Error` gains the three settings arms
  (`error-path` reused, `error-sound.ftl` untouched), and the i18n test grows
  `Io`/`Path`/`Toml` — the last two through the existing `assert_verbatim`
  helper, since their text is the system's or the parser's.
- `SoundConfig::MAX_VOLUME = sheprd_sound::MAX_VOLUME`; `gain` clamps to it.
- `Service::new` suppresses device work under
  `cfg!(any(test, feature = "test-support"))`, and `sheprd-gpui`'s
  dev-dependency enables `test-support` (section 5's risk).
- **`include_bytes!` is `../../../assets/sounds/{done,request}.mp3`**, not the
  design's `../../assets/…`: from `crates/sheprd-sound/src/playback.rs` three
  levels reach the repository root (four from `src/playback/tests.rs`).
- **The three seam tests that stay in the app** are
  `connection/tests/sounds.rs` (bridge bounding and reload-once; boot-change and
  detach generation retirement) and `state/tests/sounds.rs` (staged agent status
  validating a delayed sound once). The `view_settings_path` temp-directory
  fallback joins `config/tests/discovery.rs`.
- `SOUND-NOTICE.md` moved, with its internal paths rewritten, and every
  reference followed: `NOTICE`, `assets/sounds/README.md`,
  `crates/sheprd-gpui/README.md` (a relative link), both packagers, and the
  three `scripts/release/tests/test_release.py` lists.
- `#![cfg_attr(test, allow(clippy::disallowed_methods))]` in the crate's
  `lib.rs` (commit `df0130de9`): the FIFO test starts `mkfifo`, as the app's own
  tests do, and `main.rs`/`sheprd-core-client` carry the same allowance.
- Also in `df0130de9`: the two app-side fixtures stopped setting
  `status = Connected` before `apply(ClientEvent::Snapshot(..))`. Checked
  against production: `commit_snapshot` ends with
  `self.status = ConnectionStatus::Connected;`
  (`crates/sheprd-gpui/src/state/projection.rs`), so a committed snapshot *is* a
  connected endpoint and the explicit assignment was redundant — not a behaviour
  change. The five app-side sound tests were re-run after the edit.

### Coverage that moved or thinned

- The moved `sounds_start_at_the_default_volume` asserted the app's default
  (50) reached the worker through `Service::start`. In the crate there is no
  default volume — the app passes it — so `recording()`/`recording_volume()` are
  documented as starting at `MAX_VOLUME` and the test became
  `a_fixture_worker_starts_at_full_level_until_a_caller_sets_one`. The app-side
  guarantee is now the expression
  `Service::new(path, SoundConfig::default().volume)` in `shared_sound`, and
  `SoundConfig::default().volume == 50` is still asserted in
  `config/tests/sound_volume.rs`. No test opens a window to observe the two
  together; that is the one coverage change of this work.
- `muting_while_a_sound_waits_out_its_delay_drops_it` keeps its white-box
  assertion through `Policy::waiting()`.

## Step 3 — docs (`694be8fb6`)

`docs/gpui-split-plan.md`: `sheprd-sound` row in "What moved" plus a section on
the three inverted couplings, `sound` gone from "Next extractions", and the
"What this does not do" dependency list no longer names `rodio`.
`AGENTS.md`'s crate table gains the `sheprd-sound` row.

## Gates actually run

| Gate | Result |
| --- | --- |
| `cargo test -p sheprd-lenient` | 1 passed |
| `cargo test -p sheprd-sound` | 29 passed (after step 2; 11 after step 1) |
| `cargo check -p sheprd-gpui --all-targets` | clean, no warnings |
| `cargo test -p sheprd-gpui --bin spaceport i18n::errors` | 53 passed |
| `cargo test -p sheprd-gpui --bin spaceport sounds` | 5 passed (re-run after `df0130de9`) |
| `cargo test -p sheprd-gpui --bin spaceport sound_volume` | 7 passed |
| `cargo test -p sheprd-gpui --bin spaceport config::tests::discovery` | 8 passed |
| `cargo test -p sheprd-gpui --bin spaceport control::server` | 64 passed |
| `cargo clippy -p sheprd-sound -p sheprd-lenient --all-targets --all-features -- -D warnings` | clean |
| `cargo clippy -p sheprd-gpui --all-targets -- -D warnings` | clean |
| `cargo fmt --all -- --check` | clean |
| `just check-file-size` (`scripts/check-file-size.sh`) | clean; largest new file 314 lines |
| `just check-sorted` | clean |
| `just check-i18n` | clean, 2125 messages in 113 files |
| `just check-commits` (`origin/main..HEAD`) | clean |
| `python3 -m pytest scripts/release/tests/test_release.py -q` | 7 passed (the notice's new path) |
| `cargo metadata` | `Cargo.lock` refreshed and committed with each step |

Not run, and left to CI:

- `cargo test -p sheprd-gpui` in full (about 4,080 tests): the focused filters
  above cover every touched module; a full run needs another few GiB.
- `cargo test -p sheprd-gpui --features integration-test` (the design's
  `smoke`/`open_window` path where `Service::new` is reached in a test build).
- The Windows and Linux legs, which are the only places `cfg(unix)`'s
  `O_NONBLOCK` open and cpal's `alsa`/`windows` halves compile.
- `just ci`, `just lint` (all features, both default and no-default), release
  packaging on either OS.

## Disk

The volume sat between 0.3 and 5 GiB free all session (another worker was
building in its own target). Builds reused
`.worktrees/clock-conversions/target` (7 GB of already-compiled GPUI/rodio
deps) with `CARGO_INCREMENTAL=0` rather than starting a third full tree; the
app's artifacts land there under hashes keyed to this worktree's paths.
At 1.4 GiB free I stopped and asked; **the supervisor authorized deleting
`clock-conversions/target/debug/incremental` (3.6 GB of pure cache) across the
worktree boundary at that point**, and that deletion is the only cross-worktree
change made. This worktree's own `target/` is deleted.
