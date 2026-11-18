# Extracting the sound module into `sheprd-sound`

Design for the `sound` row of [`docs/gpui-split-plan.md`](gpui-split-plan.md), which
is the one extraction there that would drop a dependency from `sheprd-gpui`. It
is a design only: no production code changed.

**Verdict: worth doing.** The inversion costs about 120 new lines in the crate and
one argument at two call sites; what moves is 857 production lines and 1,170 test
lines, and 17 packages leave the app's build graph on macOS aarch64 — `rodio`,
`cpal`, CoreAudio and the four `objc2` audio bindings, the four `symphonia`
crates, `dasp_sample`, `mach2`, `num-rational`/`num-bigint`/`num-integer`, plus
`encoding_rs`/`simdutf8` through `symphonia-metadata` (measured by re-walking
`Cargo.lock`'s edges from `sheprd-gpui`'s direct dependencies with `rodio`
removed: 356 reachable packages become 339). `cargo tree -p rodio --target
aarch64-apple-darwin` resolves 49 packages for it alone. The win is a dependency
and a native audio stack leaving the app crate, not a line reduction — the tests
are the bulk of what moves, and they gain the ability to run as `cargo test -p
sheprd-sound` without building GPUI.

Counted today:

| | production | tests |
| --- | --- | --- |
| `src/sound.rs` | 455 | — |
| `src/sound/config.rs` | 196 | — |
| `src/sound/playback.rs` | 159 | — |
| `src/sound/error.rs` | 47 | — |
| `src/sound/tests.rs` | — | 481 |
| `src/sound/config/tests.rs` | — | 239 |
| `src/sound/playback/tests.rs` + `tests/gain.rs` | — | 185 + 26 |
| `src/sound/tests/{mutes,play,repeats,staged_metadata,volume}.rs` | — | 72 + 15 + 69 + 59 + 24 |
| **total** | **857** | **1,170** |

(848 in the split plan predates the current tree; 857 is what `wc -l` says now.)

## 1. Who uses it, and what they use

The split plan counts 16 files importing `crate::sound`. That is exact but reads
low: 4 of the 16 are inside the module itself, 4 more app files call methods on a
`Service` value without ever naming the path, and one imports `sound::Error`
through a `crate::{...}` list (`i18n/errors/sound.rs:5`). **17 app files use the
module; 13 of them name it.** Symbols by reference count, app side only:

| Symbol | References | Where |
| --- | --- | --- |
| `Service` (the type) | 10 | `app.rs:189,193,197`, `window.rs:292`, `window/sound.rs:4,12`, `endpoint/lifecycle_tests/sounds.rs:6,61`, `settings_window/controls/tests/sound_volume.rs:22`, `window/tests/sound_volume.rs:10` |
| `Service::preview` | 4 | `settings_window/pages/notifications.rs:132`, `window/render.rs:774`, two tests |
| `Service::set_volume` | 4 | `menu/settings.rs:240`, `settings_window/persistence.rs:323`, `window/sound.rs:13`, `window/tests/sound_volume.rs:20` |
| `Service::poll` | 1 | `endpoint/polling.rs:125` |
| `Service::play` | 1 | `control/server/settings.rs:75` |
| `Service::reload` | 1 | `menu/settings.rs:26` |
| `Service::new` | 1 | `app.rs:197` |
| `Service::default` | 1 | `endpoint/lifecycle_tests/sounds.rs:46` |
| `Service::recording` (`cfg(test)`) | 2 | `endpoint/lifecycle_tests/sounds.rs:6,61` |
| `Service::recording_volume` (`cfg(test)`) | 2 | `settings_window/controls/tests/sound_volume.rs:22`, `window/tests/sound_volume.rs:10` |
| `Policy` | 2 | `endpoint.rs:188` (field), `endpoint.rs:254` (`Default`) |
| `MAX_PENDING` | 2 | `state.rs:660`, `connection/tests.rs:286` |
| `muted()` | 1 | `control/server/settings.rs:69` |
| `Error` | 18 | `error.rs:218`, `i18n/errors/sound.rs:5` + 8 match arms, `i18n/errors/tests/sound.rs:2` + 8 constructs |

Nothing else is touched: `Settings`, `SoundConfig`, `PlaybackRequest`, `Recent`,
`Job` and the four helpers in `playback.rs` have no app-side reader. `Service`,
`Policy`, `MAX_PENDING`, `muted` and `Error` are the entire public surface, and
of those only `Policy` and `Error` are behind a blocker.

Which blocker each symbol sits behind:

| Symbol | Blocker |
| --- | --- |
| `Policy`, `Service::poll` | 1: `&mut LiveState` (`sound.rs:53-59`, `408-448`) |
| `Service::new`, `Service::start` | 2: `config::view_settings_path` and `config::SoundConfig` (`sound.rs:237-265`) |
| `Settings` (internal) | 2 and 3: `sound/config.rs:1` imports both `crate::config::view_settings_path` and `crate::lenient` |
| `Error` | 4: `error.rs:218`, `i18n/errors/sound.rs:7` |
| `MAX_PENDING`, `muted`, `Service` methods, `Service::recording*` | none — they move as they are |

## 2. The blockers, inverted

### 2.1 `Policy::poll(&mut LiveState)`

`Policy::poll` (`sound.rs:53`) reads five things from `LiveState`, and
`Service::poll` (`sound.rs:408`) reads four of the same plus `reload_sound`:

| Read | `LiveState` (`state.rs`) | Consumer |
| --- | --- | --- |
| cancel generation, identity compared | `sound_cancel: Arc<AtomicBool>` (`:79`) | both (`sound.rs:66-74`, `:421`) |
| connection cancel flag | `sound_connection_cancel: Arc<AtomicBool>` (`:80`) | both (`sound.rs:75`, `:422`) |
| connected? | `status.is_connected()` (`state.rs:29`) | `Policy` (`sound.rs:76`) |
| queued notifications, drained | `sound_events: VecDeque<(Instant, SemanticNotification)>` (`:63`) | both (`sound.rs:84,86`, `:417,419`) |
| newest metadata | `latest_snapshot()` (`state/projection.rs:77`) | both (`sound.rs:122`, `:423`) |
| reload requested, cleared | `reload_sound: bool` (`:67`) | `Service` (`sound.rs:415`) |

Three candidate inversions, in order of preference:

1. **A borrowed view struct owned by `sheprd-sound`** (recommended). Named
   `Live`, it carries exactly those six reads and nothing else:

   ```rust
   /// What one endpoint's live state offers the policy for a single poll, and
   /// the queues it drains.
   pub struct Live<'a> {
       /// Replaced on every connection generation; a new `Arc` retires the work
       /// queued against the old one.
       pub cancel: &'a Arc<AtomicBool>,
       pub connection_cancel: &'a AtomicBool,
       /// Whether the daemon is connected, as `ConnectionStatus::is_connected`.
       pub connected: bool,
       /// A settings reload the worker has not seen; `Service::poll` clears it.
       pub reload: &'a mut bool,
       /// Bounded by the connection at [`MAX_PENDING`]; drained here.
       pub events: &'a mut VecDeque<(Instant, SemanticNotification)>,
       /// The newest metadata, which may be ahead of the displayed pair.
       pub snapshot: Option<&'a ClientShellSnapshot>,
   }
   ```

   `Policy::poll` becomes `fn poll(&mut self, live: Live<'_>, delay: Option<u64>,
   focused: bool, now: Instant, emit: impl FnMut(SemanticNotification))`
   (6 parameters), and `Service::poll` becomes
   `pub fn poll(&self, policy: &mut Policy, live: Live<'_>, focused: bool,
   now: Instant, silenced: impl Fn(&SemanticNotification) -> bool)` (6).

   The app builds it at the one call site, `endpoint/polling.rs:125`:

   ```rust
   // `latest_snapshot` is a method on `&self`, so resolve it before the
   // `&mut` borrows below; the clone is the one `Service::poll` already made.
   let snapshot = endpoint.live.latest_snapshot().cloned();
   self.sound.poll(
       &mut endpoint.sounds,
       sheprd_sound::Live {
           cancel: &endpoint.live.sound_cancel,
           connection_cancel: &endpoint.live.sound_connection_cancel,
           connected: endpoint.live.status.is_connected(),
           reload: &mut endpoint.live.reload_sound,
           events: &mut endpoint.live.sound_events,
           snapshot: snapshot.as_deref(),
       },
       index == self.selected_endpoint && self.active,
       Instant::now(),
       |event| silenced.notice(...),
   );
   ```

   Two details that cost a compile if missed: the `Arc` clone inside the emit
   closure must be `Arc::clone(live.cancel)` (`live.cancel.clone()` on a
   `&Arc<AtomicBool>` field resolves to `&Arc`, not `Arc`); and `ClientShellSnapshot`
   is `sheprd_protocol::ClientShellSnapshot` (`crates/sheprd-protocol/src/wire.rs:312`),
   already on `LiveState` as `Arc<ClientShellSnapshot>`, so no app type crosses.

   What stays: `LiveState` keeps all five fields, `state.rs:660` keeps filling
   `sound_events` under `MAX_PENDING`, `ConnectionStatus` stays in the app, and
   the queue stays the connection's to bound and the policy's to drain.

2. **Plain arguments.** Ten parameters (`cancel`, `connection_cancel`,
   `connected`, `reload`, `events`, `snapshot`, `delay`, `focused`, `now`,
   `emit`) trip `clippy::too_many_arguments` (default threshold 7; the workspace
   only denies `expect_used`/`unwrap_used`, so this would be a warning to fix,
   not an error). Rejected.

3. **Move the queue and flags into the crate and embed them in `LiveState`.**
   `state.rs` would hold `sheprd_sound::Queue` and the app's `LiveState` methods,
   `Default`, and `comparable()` (`state.rs:355-362`) would reach through it.
   More churn in the app's hottest state struct for no gain. Rejected.

No new trait: `Live` is one owned-by-the-crate struct with borrowed fields, and
the two behaviours it needs from the app (whether it is connected, what the
newest metadata is) arrive as a `bool` and an `Option<&_>`.

### 2.2 `config::view_settings_path` and `config::SoundConfig`

Two different settings are tangled here, and they separate cleanly.

**Herdr's `config.toml`** (`[ui.sound]`, `[ui.toast] delay_seconds`) is read by
`sound/config.rs`: `Settings::load` (`:149`) resolves its own path with
`config::view_settings_path(sheprd_client::local_var)` (`sound/config.rs:1`).
The path policy is the app's — `crates/sheprd-gpui/src/config/daemon.rs:16`,
`spaceport/view.toml`, already tested in
`crates/sheprd-gpui/src/config/tests/discovery.rs`. So:

- `Settings::load(path: &Path)` takes the file it reads. `load_path`, `parse`,
  `ui_delay` and `path_for` (`sound/config.rs:153-190`) are unchanged.
- `Settings` and `SoundConfig` become `pub(crate)` in the new crate: the app
  never names either once `Service::new` takes a path (below).
- The worker's reload closure goes away in favour of the path:

  ```rust
  /// `path` is the settings file the worker reads on its first tick and on
  /// every reload; `volume` plays every sound until a caller sets one.
  pub fn new(path: PathBuf, volume: u8) -> Self
  ```

  with `start(load, play, volume)` (`sound.rs:252`) keeping its injectable
  loader for tests. The app passes `crate::config::view_settings_path(
  sheprd_client::local_var)` at `app.rs:197`. A closure
  (`new(load: impl FnMut() -> Result<Settings, Error>, volume)`) is the
  alternative if a reload source other than one file ever appears; nothing
  needs it today, so the plain `PathBuf` wins.

**Spaceport's own volume** (`[sound] volume`, `crates/sheprd-gpui/src/config/notifications.rs:63-79`)
is the app's setting and stays in the app, with one piece moving down:

- `SoundConfig::MAX_VOLUME` (`config/notifications.rs:71`, used at
  `config/notifications.rs:84`, `config/preferences.rs:236`,
  `settings_window/inputs.rs:53`) is the playback contract — `gain`
  (`sound/playback.rs:73-76`) clamps to it and `set_volume`'s doc
  (`sound.rs:231`) says above 100 plays at full level. Define it once, in
  `sheprd-sound`: `pub const MAX_VOLUME: u8 = 100;` and
  `impl SoundConfig { pub const MAX_VOLUME: u8 = sheprd_sound::MAX_VOLUME; }`.
  The three app call sites keep working unchanged.
- The default volume (50) stays the app's:
  `Service::start` currently seeds it from
  `crate::config::SoundConfig::default().volume` (`sound.rs:265`); with
  `new(path, volume)` the app passes `SoundConfig::default().volume` instead.
  `use_sound` (`window/sound.rs:12`) still sets it per window.

No trait, no callback: one path parameter and one volume parameter replace both
couplings.

### 2.3 `lenient`

`sound/config.rs` reads Herdr's file leniently through
`crate::lenient::{or_default, or_true, entries, value}`
(`crates/sheprd-gpui/src/lenient.rs`, 107 lines, `serde` and `toml` only). The
peer worker's `feat/wave7-extract` branch exists in `.worktrees/wave7-extract`
but carries no commits yet (`git log f824c0534..HEAD` is empty), so this design
is against the module as it stands.

`sheprd-sound` depends on `sheprd-lenient` for those four functions, which must
be `pub` there with the signatures they have today (`lenient.rs:9-48`). If
`sheprd-lenient` has not landed when this starts, it lands first: it is four
`pub fn`s and a test file, and the alternative — a private copy of the same
deserializers in `sheprd-sound` — is exactly the duplication AGENTS.md forbids.
Nothing else in the sound module touches `lenient`.

### 2.4 The root `Error::Sound` variant

Section 3, below, because the shape is the whole answer.

## 3. The error shape

`crates/sheprd-gpui/src/sound/error.rs` is already the crate-shaped piece: eight
variants, three `#[from]`s into rodio, and three `impl From<rodio::…> for
crate::Error` (`error.rs:30-45`) that exist only so playback can use `?`. What it
lacks is the three app variants `sound/config.rs` borrows: `crate::Error::Io`
(`sound/config.rs:159`), `crate::Error::Toml` (`:173`) and `crate::Error::Path`
via `.at_path()` (`error/path.rs:7`, `sound/config.rs:157,164,170`).

`crates/sheprd-sound/src/error.rs`:

```rust
//! Why a notification sound did not play or its settings did not load.

use std::{
    io,
    path::{Path, PathBuf},
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Audio playback cancelled")]
    Cancelled,
    #[error("Sound configuration exceeds 1 MiB")]
    ConfigSize,
    #[error("Could not decode MP3 sound: {0}")]
    Decode(#[from] rodio::decoder::DecoderError),
    #[error("Could not open audio output: {0}")]
    Device(#[from] rodio::DeviceSinkError),
    #[error("Could not read sound file {}: {source}", path.display())]
    File {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("Sound must be a regular file of at most 16 MiB")]
    FileSize,
    #[error("{0}")]
    Io(#[from] io::Error),
    #[error("Could not load sound settings from {}: {source}", path.display())]
    Path {
        path: PathBuf,
        #[source]
        source: Box<Error>,
    },
    #[error("Audio output failed: {0}")]
    Stream(#[from] rodio::cpal::StreamError),
    #[error("Audio playback timed out")]
    Timeout,
    #[error("{0}")]
    Toml(#[from] toml::de::Error),
}

impl Error {
    /// This failure, as one concerning `path`.
    pub(crate) fn at_path(self, path: &Path) -> Self {
        Self::Path {
            path: path.to_owned(),
            source: Box::new(self),
        }
    }
}
```

`Path`, `Io` and `Toml` mirror `crate::Error`'s own variants by spelling and by
`Display` (`i18n/errors/root.rs:78`, `:154`, `:110-112`) so log lines read the
same. `at_path` becomes private to the crate — no app caller needs it.

The app's root enum keeps the variant it already has, with one word changed
(`crates/sheprd-gpui/src/error.rs:217-218`), and its position in the
`keep-sorted` region does not move:

```rust
#[error(transparent)]
Sound(#[from] sheprd_sound::Error),
```

That is the workspace rule satisfied in one line: `sheprd-sound` owns a
`thiserror` `Error` with its own `#[from]`s (the rodio ones replace the three
app `impl From`s, which are deleted), and the app adds the single
`#[error(transparent)] Feature(#[from] feature::Error)` variant that wraps it.
Nothing in the app matches on `sheprd_sound::Error` except the localizer.

The app keeps the messages (docs/i18n.md — a crate below the app has none), so
`crates/sheprd-gpui/src/i18n/errors/sound.rs` imports `sheprd_sound::Error`
instead of `crate::sound::Error` and gains three arms:

```rust
Self::Io(error) => error.to_string(),
Self::Path { path, source } => fl!(
    "error-path",
    path = path.display().to_string(),
    error = source.localize()
),
Self::Toml(error) => error.to_string(),
```

`error-path` is the root message `i18n/errors/root.rs:110-112` already uses for
the app's own `Error::Path`, so **no new `.ftl` message** is needed and
`error-sound.ftl` is untouched. `crates/sheprd-gpui/src/i18n/errors/tests/sound.rs`
already constructs every variant; it gains the three new ones and keeps naming
rodio types (`rodio::decoder::DecoderError::UnrecognizedFormat`,
`rodio::cpal::StreamError::DeviceNotAvailable`), which is why `sheprd-gpui`
needs a **dev-dependency** on rodio after this lands (see step 2).

## 4. The crate, and the commit plan

```
crates/sheprd-sound/
  Cargo.toml
  SOUND-NOTICE.md          # moved from crates/sheprd-gpui/
  src/lib.rs               # module doc, re-exports
  src/error.rs             # 47 lines, + Io/Path/Toml
  src/config.rs            # Settings from sound/config.rs, load(path)
  src/policy.rs            # Live, Policy, Pending, MAX_PENDING
  src/service.rs           # Service, Job, PlaybackRequest, Recent, REPEAT_WINDOW
  src/playback.rs          # rodio: play, gain, decode, check, wait, MAX_VOLUME
  src/tests.rs + src/tests/{mutes,play,repeats,volume,config,policy,...}.rs
```

`Cargo.toml` copies the workspace fields from `crates/sheprd-i18n`'s:

```toml
[dependencies]
rodio.workspace      = true
serde.workspace      = true
sheprd-lenient.workspace = true
sheprd-protocol.workspace = true
thiserror.workspace  = true
toml.workspace       = true
tracing.workspace    = true      # the worker logs a failed load and playback

[target.'cfg(unix)'.dependencies]
rustix.workspace     = true      # O_NONBLOCK on a configured sound file

[dev-dependencies]
tempfile.workspace   = true

[features]
# `Service::recording`/`recording_volume`, and the test suppression in `new`,
# for the app's headless tests.
test-support = []
```

`sheprd-protocol` rather than `sheprd-client`: `sheprd_client::protocol` is a
re-export (`crates/sheprd-client/src/lib.rs:60`), and the crate needs no other
part of the client — the settings path now arrives as a parameter.
`test-support` follows the `sheprd-diagnostics` precedent
(`crates/sheprd-diagnostics/Cargo.toml:25-26`, used at `src/lib.rs:104`).
Step 1's `Cargo.toml` is this without `rodio`, `rustix`, `tracing` and
`tempfile`, which arrive with `playback.rs` and its tests in step 2.

### Step 0 — prerequisite, not on this branch

`sheprd-lenient` lands (peer, `feat/wave7-extract`). If it is not merged, land it
first as its own commit; `sheprd-sound` cannot take `sound/config.rs` before it.

### Step 1 — `refactor: Move the sound policy and settings into sheprd-sound`

No rodio yet. Adds the crate with `lib.rs`, `error.rs`, `config.rs`,
`policy.rs` and `MAX_PENDING`; moves `Settings` out of `sound/config.rs` and
`Policy` out of `sound.rs`, which keeps `Service`, `PlaybackRequest`, `Recent`,
`Job` and playback and whose `poll` now takes `sheprd_sound::Live<'_>` instead
of `&mut LiveState`; rewrites the one call site at `endpoint/polling.rs:125` to
build `Live` (once — step 2 does not touch it again); points `state.rs:660`,
`connection/tests.rs:286` and `endpoint.rs:188` at the crate; moves
`sound/config/tests.rs` (except `view_config_path_stays_with_spaceport`, whose
one uncovered assertion — the `env::temp_dir()` fallback for
`view_settings_path(|_| None)` — joins
`crates/sheprd-gpui/src/config/tests/discovery.rs`, which already covers the
rest) and the seven policy tests of `sound/tests.rs` (`:69,106,156,184,197,230,245`)
plus the `event()` fixture and a `Live`-based replacement for `live()`.

Compiles and tests on its own: `cargo test -p sheprd-sound -p sheprd-gpui`.

### Step 2 — `refactor: Move sound playback into sheprd-sound and drop rodio from sheprd-gpui`

Adds `playback.rs`, `service.rs`, the `MAX_VOLUME` move, `Service::new(path,
volume)` with the `cfg!(any(test, feature = "test-support"))` suppression,
`recording`/`recording_volume` behind `test-support`, and deletes
`crates/sheprd-gpui/src/sound*` and `mod sound;` (`main.rs:98`). Cargo:
workspace `members` and `[workspace.dependencies]` gain `sheprd-sound`;
`sheprd-gpui` gains `sheprd-sound.workspace = true`, a dev-dependency
`sheprd-sound = { features = ["test-support"], workspace = true }`, a
dev-dependency `rodio.workspace = true` (only for `i18n/errors/tests/sound.rs`),
and loses `rodio.workspace = true` from `[dependencies]`. `app.rs:197` becomes
`Service::new(view_settings_path(sheprd_client::local_var),
SoundConfig::default().volume)`; `playback.rs`'s `include_bytes!` becomes
`../../assets/sounds/{done,request}.mp3`; `SOUND-NOTICE.md` moves to the new
crate, and its five path references follow (`NOTICE:32`,
`assets/sounds/README.md:4`, `scripts/release/package-macos.sh:90`,
`scripts/release/package-windows.py:24`, three in
`scripts/release/tests/test_release.py`).

### Step 3 — `docs: Record the sound extraction in the split plan`

`docs/gpui-split-plan.md` (the "What moved" table, the sound row of "Next
extractions", and the dependency sentence in "What this does not do") and the
crate table in `AGENTS.md`.

### Test move plan (1,170 lines)

| Test file | Lines | Destination |
| --- | --- | --- |
| `tests.rs` — bridge bounding (`:39`) | 30 | **stays**: `ConnectionBridge` and its inbox |
| `tests.rs` — detach and boot cancellation (`:264`) | 44 | **stays**: exercises the inverted seam from the app side |
| `tests.rs` — 7 policy tests (`:69,106,156,184,197,230,245`) | ~180 | `sheprd-sound/src/tests/policy.rs`, with a `Live` fixture instead of `LiveState` |
| `tests.rs` — 5 worker/service tests (`:309,335,384,408,448`) | ~170 | `sheprd-sound/src/tests/service.rs` |
| `tests/{mutes,play,repeats,volume}.rs` | 180 | `sheprd-sound/src/tests/`, unchanged apart from `super::*` |
| `tests/staged_metadata.rs` | 59 | **stays**: `ClientEvent::Surface` staging is the app's projection |
| `config/tests.rs` (6 tests) | 239 | 5 to `sheprd-sound/src/tests/config.rs`; `view_config_path_stays_with_spaceport` is deleted, one assertion joining `config/tests/discovery.rs` |
| `playback/tests.rs` + `tests/gain.rs` | 211 | `sheprd-sound/src/tests/{playback,gain}.rs`, unchanged |

About 167 lines stay in the app (the two connection tests, the projection test,
and the path assertion); about 1,003 move. The moved policy tests need one new
fixture: today's `live(status)` (`tests.rs:9-22`) builds a `LiveState` through
`ClientEvent::Snapshot`; it becomes a `ClientShellSnapshot` from
`crates/sheprd-protocol/tests/fixtures/endpoint-snapshot-v1.json` plus a
`Live { .. }` with a local `VecDeque`, which is what the tests already assert on
through `state.sound_events`.

### Verification per step

`cargo check -p sheprd-sound -p sheprd-gpui`, `cargo test -p sheprd-sound`,
`cargo test -p sheprd-gpui` (headless), `cargo fmt --all -- --check`,
`just check-file-size`, `just check-sorted`, `just check-i18n`, and
`cargo metadata` for `Cargo.lock`, in that order. Every one already exists as a
`just` recipe or a cargo invocation in the split plan's step 7; this work adds
no recurring operation, so no new recipe is due. Deferred to CI: the Windows and
Linux legs, which are the only places the `cfg(unix)` lockfile-free open
(`playback.rs:97-99`) and the `windows`/`alsa` halves of cpal are compiled.

## 5. Risks

**Test-only device and settings suppression no longer covers the app.**
`sound.rs:236-243` refuses to start a worker under `cfg!(test)`; in a separate
crate `cfg!(test)` is false while `sheprd-gpui`'s tests run. Today nothing in the
default test build reaches `Service::new` — `shared_sound` (`app.rs:193`) is only
called from `open_window`/`open_additional_window`, and the tests that drive
those are `smoke` (`main.rs:128`, `#[cfg(feature = "integration-test")]`). A
future headless test that opens a window would spawn a real worker: a thread, one
read of the user's `view.toml`, and — only if a notification reached it — a
`cpal` device open. The `test-support` feature restores the guarantee
(`cfg!(any(test, feature = "test-support"))`) and `muted()`
(`playback.rs:22-24`) still answers to `HERDR_DISABLE_SOUND`/`NEXTEST`. Verify
with `cargo test -p sheprd-gpui --features integration-test` locally only if the
disk allows; otherwise state it as CI's to prove.

**Device probing stays on the worker, per job.** `playback::play`
(`playback.rs:29-71`) selects the default device inside each job and drops it
before returning, so an unplug or a sleep is picked up by the next sound and a
failed job is never replayed. Extraction changes none of this; the risk is only
that the app no longer links the audio stack, so a panic or abort inside cpal is
now entirely inside `sheprd-sound`'s worker thread — which is where it already
is.

**The worker's shutdown and reload handshake.** `Drop for Service`
(`sound.rs:451-455`) sets `stop`, and the worker polls `recv_timeout(50ms)`
(`sound.rs:279`). `Service::recording*` tests therefore rely on the channel
disconnecting within 3-second bounded waits (`tests.rs:384-405`); those tests
move with the worker and keep their bounds. Nothing here should be "improved"
into a join on the UI thread — the app drops the `Rc<Service>` while a window
loses its last reference.

**Reading settings at runtime.** Only the worker reads the file
(`Settings::load`, `sound/config.rs:149-176`), on its first tick and when
`reload` is set; the UI thread only reads the `Arc<Settings>` the worker
publishes (`sound.rs:267`, `:421-425`) for the delay. After the path becomes
a `PathBuf` parameter this is unchanged — but the app now decides *when* to
compute it, so `view_settings_path` must be resolved once in `shared_sound`
(`app.rs:193`), not per poll, or a test that overrides `XDG_CONFIG_HOME` mid-run
would silently change what the worker reads. Keep it at construction.

**Packaging references.** `SOUND-NOTICE.md` is named by both release packagers
and their tests (step 2). Missing one leaves a stale path in a packaging script
that only release CI or `scripts/release/tests/test_release.py` would catch.

**Disk.** Two full `sheprd-gpui` rebuilds (steps 1 and 2) on a volume that has
been close to full; check `df` before each, as this wave's brief says.

## 6. What this does not do

No behaviour change: same policy timings (`REPEAT_WINDOW`, the 50 ms grace, the
one-second expiry), same queue bounds (`MAX_PENDING` on `sound_events` and on
the worker's channel), same worker thread name, same rodio settings. No new
dependency for the app, one fewer. `sheprd-gpui` keeps `mermaid-rs-renderer`,
`ratex-*`, `image` and `syntect` until `chat` and `browser` move.
