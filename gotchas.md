# Gotchas

Operational lessons this fork learned the hard way: release signing, macOS
sandbox escapes, CI cache scoping, daemon lifecycle traps, test-harness
isolation. Each entry is a heading with the date it was recorded and a short
title, then four fields:

- **RULE** — the durable instruction: what to do, or not to do, next time.
- **SYMPTOM** — what you observe when the trap springs. `Not recorded.` means
  the original lesson stated no observable symptom.
- **CAUSE** — why it happens, so the rule can be re-derived rather than obeyed
  blindly.
- **REFERENCES** — evidence documents, code paths and decision records, kept
  verbatim from the original lesson.

Entries stay in the order they were recorded, which is not strictly
chronological. A lesson that covered several independent traps was split into
one entry per trap under its recorded date; no entry, clause or reference was
dropped, and no date was changed.

## 2026-11-18 — The security-report route is approved and enabled

- **RULE**: Report a suspected vulnerability through Spaceport's own private
  advisory form, which `SECURITY.md` and the issue chooser name. The 2026-11-10
  entry's "still needs an owner-approved Spaceport route" clause is resolved.
- **SYMPTOM**: The 2026-11-10 entry still reads as if no approved route exists,
  so work that defers shipping on one repeats a resolved blocker.
- **CAUSE**: The owner approved enabling GitHub private vulnerability reporting
  for `2389-research/spaceport`; the setting was enabled and freshly read back
  as `enabled: true`, and the issue chooser and policy use that form.
- **REFERENCES**: `docs/spaceport/notes/branding-audit-followups.md`,
  `docs/audits/AUDIT_REPORT_2026-11-12.md`, `SECURITY.md`,
  `.github/ISSUE_TEMPLATE/config.yml`

## 2026-11-16 — Terminal `skip` is not a continuation marker

- **RULE**: Use `terminal::selection::graphemes` and its shared continuation
  mask, and preserve real text and spaces after one-column emoji sequences.
- **SYMPTOM**: Not recorded.
- **CAUSE**: `skip` is not a reliable continuation marker: Herdr 1 can send
  unmarked empty or blank cells after wide glyphs.
- **REFERENCES**:
  [upstream backport evidence](docs/spaceport/evidence/upstream-correctness-backports.md)

## 2026-11-13 — Core is opt-in and restart-only (D69)

- **RULE**: Read `core_feature::enabled` or the window Hub's frozen bit for
  runtime availability, preserve hidden Core tabs in Store, and make
  Missing-Herdr status respect the effective mode.
- **SYMPTOM**: The saved `Config::herdr_core_enabled` may describe the next
  launch, and hiding one of tab labels, IDs or `browser_tab_entries` does not
  hide the rendered controls.
- **CAUSE**: D69 makes Core opt-in restart-only, and tab labels, IDs and
  `browser_tab_entries` have separate producers.
- **REFERENCES**: `docs/spaceport/notes/core-opt-in.md`

## 2026-11-12 — `dispatch_action` defers, so one update is not enough

- **RULE**: Cross an update boundary before asserting the result of an action.
  Rendering fixtures can call the shared synchronous handler, with separate
  tests for actual action dispatch.
- **SYMPTOM**: Inspecting the destination in the same native update observes
  old state. Focus and drawing alone do not flush the deferred action.
- **CAUSE**: GPUI `Window::dispatch_action` queues `cx.defer`.
- **REFERENCES**: `docs/spaceport/evidence/t01-profile-and-chat-docs.md`

## 2026-11-12 — Native routing runs while macOS is locked

- **RULE**: Use `just test-sidebar-routing` for native routing: it runs while
  macOS is locked, uses exact fixture AppKit views, and keeps key-window
  qualification separate in `just test-sidebar`. Startup may request
  activation, but routing does not require it.
- **SYMPTOM**: Not recorded.
- **CAUSE**: Not recorded.
- **REFERENCES**: `docs/spaceport/evidence/t11-native-routing-without-unlock.md`

## 2026-11-12 — Native routing fixtures: isolated executable and layout

- **RULE**: Keep the focused 600-pixel setup independent of Settings' larger
  desktop fixtures, and use `integration-test` for its loose executable.
- **SYMPTOM**: Not recorded.
- **CAUSE**: The cleared sandbox cannot load an all-features Sparkle framework
  through inherited loader overrides.
- **REFERENCES**: `docs/spaceport/evidence/t11-native-routing-without-unlock.md`

## 2026-11-12 — Native routing fixtures: coordinate probes and filter breadth

- **RULE**: Use the endpoint's painted control for header click coordinates,
  and keep the original sidebar filter broad enough to run its two negative
  fixtures.
- **SYMPTOM**: Pinned headers repeat labels, so text-keyed probes can retain
  the clipped scrolling copy.
- **CAUSE**: Not recorded.
- **REFERENCES**: `docs/spaceport/evidence/t11-native-routing-without-unlock.md`

## 2026-11-12 — GitHub cache scopes isolate release tags

- **RULE**: Seed reusable release caches from trusted main.
- **SYMPTOM**: New version tags cannot read earlier tags' caches.
- **CAUSE**: GitHub cache scopes isolate release tags.
- **REFERENCES**: Not recorded.

## 2026-11-12 — `rust-cache` ignores `key` when `shared-key` is set

- **RULE**: Put compiler and SDK identifiers in `shared-key` itself.
- **SYMPTOM**: The `key` you set has no effect.
- **CAUSE**: The pinned `rust-cache` action ignores `key` when `shared-key` is
  set.
- **REFERENCES**: Not recorded.

## 2026-11-12 — Keep host preview and explicit-target build profiles separate

- **RULE**: Keep host preview and explicit-target build profiles separate.
- **SYMPTOM**: Not recorded.
- **CAUSE**: Not recorded.
- **REFERENCES**: Not recorded.

## 2026-11-12 — Stage exact binaries before saving a cache

- **RULE**: Stage exact binaries in a fixed directory before saving, and expect
  the pinned tool installer to select `.install-action/bin` instead of
  `.cargo/bin`.
- **SYMPTOM**: A successful cache step can still warn and save nothing.
- **CAUSE**: The pinned tool installer may select `.install-action/bin` instead
  of `.cargo/bin`.
- **REFERENCES**: Not recorded.

## 2026-11-12 — Cache Cargo release profiles, not all of `target`

- **RULE**: Cache Cargo release profiles instead of all of `target`; its
  dependency cleanup still runs with `cache-targets: false`.
- **SYMPTOM**: Non-Cargo files disappear while empty packaging directories are
  left behind.
- **CAUSE**: `rust-cache` removes non-Cargo files but leaves empty packaging
  directories.
- **REFERENCES**: Not recorded.

## 2026-11-11 — A legacy helper lookup name does not prove a mismatch

- **RULE**: Follow the full lookup chain and compare packaging before
  diagnosing a missing helper. Preserve bundled-before-PATH ordering and both
  existing name generations. File-lookup fixtures prove selection, not
  executable behavior or a working agent session.
- **SYMPTOM**: A missing helper reported under the legacy lookup name looks
  like a packaging mismatch.
- **CAUSE**: `chat::start` requests `herdr-acp`, but `bundled::locate_in`
  already prefers the shipped `sheprd-acp` name (including `.exe`) before the
  existing legacy fallback.
- **REFERENCES**: Not recorded.

## 2026-11-11 — The performance adapter must use the startup `WINDOW_TITLE`

- **RULE**: Select the process's fixture window with the same `WINDOW_TITLE`
  startup uses.
- **SYMPTOM**: A stale product literal prevents all native input measurement
  after first paint.
- **CAUSE**: The macOS performance adapter selects its fixture window by title.
- **REFERENCES**: Not recorded.

## 2026-11-11 — Qualify performance through the release-only sandbox harness

- **RULE**: Use the release-only `performance_gui` sandbox harness for
  qualification: it clears personal state paths, forwards only the performance
  budget, and reaps its exact child before removing the sandbox. Just and
  Windows CI select that harness through Cargo so external target directories
  work.
- **SYMPTOM**: Not recorded.
- **CAUSE**: Not recorded.
- **REFERENCES**: Not recorded.

## 2026-11-11 — Cached and uncached performance selection is explicit

- **RULE**: Keep cached/reference selection explicit: the recipe uses presence
  of `HERDR_PERF_UNCACHED` (even `0`), and the cached CI test must not inherit
  it.
- **SYMPTOM**: Not recorded.
- **CAUSE**: The recipe tests presence of `HERDR_PERF_UNCACHED`, so any set
  value selects the uncached path.
- **REFERENCES**: Not recorded.

## 2026-11-11 — Linux must refuse native benchmarks, not pass zero tests

- **RULE**: Linux must refuse the native benchmark explicitly rather than
  report a zero-test pass.
- **SYMPTOM**: A green run that measured nothing.
- **CAUSE**: Linux lacks native benchmark event delivery.
- **REFERENCES**: Not recorded.

## 2026-11-11 — `APP_NAME` owns the product name

- **RULE**: Read Spaceport's product name from `spaceport_client::APP_NAME`; GPUI's
  window-title constant reads it, and the dependency-free `app_name.rs` module
  supplies the real value to standalone identity and packaging probes.
- **SYMPTOM**: Typed client diagnostics and the desktop diverge, or compiling
  constants alone or parsing a literal window title misses the dependency.
- **CAUSE**: `spaceport_client::APP_NAME` owns Spaceport's product name.
- **REFERENCES**: Not recorded.

## 2026-11-11 — Pass the product name to Fluent as data

- **RULE**: Pass the name to Fluent as data to preserve it in pseudo-locales,
  including setup-relay and version-refusal errors.
- **SYMPTOM**: The product name is not preserved in pseudo-locales.
- **CAUSE**: Not recorded.
- **REFERENCES**: Not recorded.

## 2026-11-11 — Version guidance only asks for a strictly newer generation

- **RULE**: Version guidance asks for a client update only for a strictly newer
  generation; older or equal generations keep the Herdr command.
- **SYMPTOM**: Not recorded.
- **CAUSE**: Not recorded.
- **REFERENCES**: Not recorded.

## 2026-11-11 — Helper stderr and relay frames are separate display paths

- **RULE**: Changing the dialog catalog does not rename setup helper stderr or
  relay frames; treat them as separate display paths.
- **SYMPTOM**: A renamed dialog string leaves the helper's stderr or relay
  frame under the old name.
- **CAUSE**: Setup helper stderr and relay frames are separate display paths.
- **REFERENCES**: Not recorded.

## 2026-11-11 — Localize the Add Device local-network refusal at the boundary

- **RULE**: Add Device's local-network refusal must use `Localize` at the
  display boundary.
- **SYMPTOM**: The refusal stays English even when the dialog's locale changes.
- **CAUSE**: Plain `Display` is English.
- **REFERENCES**: Not recorded.

## 2026-11-11 — Keep-awake identity is OS metadata, not only UI copy

- **RULE**: Derive both the keep-awake reason and the app name from Spaceport's
  product-name constant.
- **SYMPTOM**: State tests with stand-in guards pass without proving that
  metadata.
- **CAUSE**: Keep-awake identity is OS metadata as well as UI copy: pinned
  keepawake uses the reason on macOS/Linux and the app name on Linux.
- **REFERENCES**: Not recorded.

## 2026-11-11 — What the opt-in keep-awake test does and does not prove

- **RULE**: Treat the opt-in macOS power-assertion test as covering only its own
  PID; RAII releases its real guards even when a check fails.
- **SYMPTOM**: Not recorded.
- **CAUSE**: The test checks only its own PID.
- **REFERENCES**: Not recorded.

## 2026-11-11 — Canonical DNS can block every Linux job

- **RULE**: Use temporary Ubuntu Deb822 sources with Canonical primary and
  independent MIT fallback while preserving suites, components and the Ubuntu
  archive keyring.
- **SYMPTOM**: Every Linux job is blocked, or one independent mirror can also
  become unreachable.
- **CAUSE**: GitHub's Ubuntu mirror list falls back between Canonical domains,
  so a Canonical DNS outage can block every Linux job.
- **REFERENCES**: Not recorded.

## 2026-11-11 — Require `--error-on=any` and bound metadata attempts

- **RULE**: Require `--error-on=any` before installation, bound metadata
  attempts to two, and install once only after success.
- **SYMPTOM**: Plain `apt-get update` can succeed after failed index downloads.
- **CAUSE**: Not recorded.
- **REFERENCES**: Not recorded.

## 2026-11-11 — Never weaken APT to restore a build

- **RULE**: Keep developer sources untouched, reject unknown formats, and never
  disable APT signature verification to restore a build.
- **SYMPTOM**: Not recorded.
- **CAUSE**: Not recorded.
- **REFERENCES**: Not recorded.

## 2026-11-10 — Recover a queued preview run's reservation

- **RULE**: Recover their reservation from the valid push tag and full source
  SHA; never ignore the record or free its version.
- **SYMPTOM**: Queued GitHub preview runs may retain a commit-subject title
  before `run-name` initializes.
- **CAUSE**: `run-name` initializes after the record is created.
- **REFERENCES**: Not recorded.

## 2026-11-10 — Preview changelogs start at a published ancestor

- **RULE**: Preview changelogs start at a published ancestor, not the last Git
  tag.
- **SYMPTOM**: A preview changelog range based on the last Git tag is wrong.
- **CAUSE**: A failed tag reserves its version without publishing its changes.
- **REFERENCES**: Not recorded.

## 2026-11-10 — First release source and git-cliff range rendering

- **RULE**: Before the first release, use the actual fork source in
  `t00-dependencies.json`, not an earlier candidate source from the plan.
  Render the explicit range with the shared pinned git-cliff.
- **SYMPTOM**: A same-commit rebuild has empty history; filtered build-only
  history still renders a version heading under `render_always`.
- **CAUSE**: Not recorded.
- **REFERENCES**: Not recorded.

## 2026-11-10 — "Ship" approval includes the rebuild and verification

- **RULE**: Doctor Biz's approval to “ship” branding includes rebuilding a
  usable preview and carrying verification/delivery through. His explicit
  “push and make it work” also authorizes the necessary GitHub push and signing
  setup; an uncommitted workflow is not that result.
- **SYMPTOM**: An uncommitted workflow is presented as the delivered result.
- **CAUSE**: Approval scope covers delivery, not only local changes.
- **REFERENCES**: Not recorded.

## 2026-11-10 — Blocked access: retain the failure and keep working

- **RULE**: If signing or desktop access blocks the final checks, retain the
  concrete build and exact failure, and continue independent work while asking
  for the missing access.
- **SYMPTOM**: Not recorded.
- **CAUSE**: Not recorded.
- **REFERENCES**: Not recorded.

## 2026-11-10 — An inherited `RUSTUP_TOOLCHAIN` overrides the pin

- **RULE**: Select `RUSTUP_TOOLCHAIN=1.99.0` explicitly for the preview builder
  when the shell activates another version; do not change dependency
  requirements to fit the shell.
- **SYMPTOM**: The preview builder builds with a toolchain other than the
  repository's pinned one.
- **CAUSE**: An inherited `RUSTUP_TOOLCHAIN` can override this repository's
  pinned `rust-toolchain.toml`.
- **REFERENCES**: Not recorded.

## 2026-11-09 — The approved icon source and its provenance

- **RULE**: Treat `design/spaceport-icon/rocket-nebula.png` as the approved
  source; SVG wrappers and raster/platform assets derive from its exact bytes.
  Preserve Twemoji reference provenance in NOTICE and LICENSE-twemoji.
- **SYMPTOM**: Not recorded.
- **CAUSE**: D62/D63 select the sculpted nebula rocket after Doctor Biz found
  the first flat emoji too plain.
- **REFERENCES**: Not recorded.

## 2026-11-09 — Icon regeneration, asset keys and the development rim

- **RULE**: Keep existing internal asset filenames/catalog keys until their
  packaging and embedded references change together, and regenerate both
  `.icns` and `.car`. Development builds use a berry rim; the underlying
  approved artwork stays unchanged.
- **SYMPTOM**: Not recorded.
- **CAUSE**: macOS 26 may select the catalog instead of the flattened icon.
- **REFERENCES**: Not recorded.

## 2026-11-09 — `hc schema` can start Core

- **RULE**: Use the direct-serve test Sandbox with explicit short private
  socket/XDG paths and exact-child cleanup for runtime probes, and do not treat
  a successful schema query as proof that no process started.
- **SYMPTOM**: A "read-only" Core CLI inspection leaves a running process.
- **CAUSE**: Core CLI inspection can have lifecycle side effects: `hc schema`
  uses the API and can autostart Core, whose launcher restores login HOME even
  when the query had a private environment.
- **REFERENCES**: Not recorded.

## 2026-11-09 — What a remote-chat transport test must show

- **RULE**: For remote chat, do not test for a chat-named RPC; an authenticated
  lossless duplex byte route could suffice, while current `machine.exec` is
  batch-only.
- **SYMPTOM**: Not recorded.
- **CAUSE**: Missing a chat-named RPC is not the test.
- **REFERENCES**: Not recorded.

## 2026-11-09 — Where the Teleport helpers landed

- **RULE**: Retained helpers now live in
  `spaceport-remote::{host_script,agent,snapshot}`. Exact target/session refusal,
  argument permission choices and output/cancellation limits must survive
  extraction.
- **SYMPTOM**: Teleport module names concealed live chat, pane and process
  helpers.
- **CAUSE**: The remaining migration engine is removed.
- **REFERENCES**: Not recorded.

## 2026-11-09 — Search inferred calls; keep observation and launch paths

- **RULE**: Search inferred method calls as well as qualified references: the
  process viewer still needs `ProcessInfo::foreground_pids`. Preserve
  observation/attachment and explicit runtime launches. T06 still requires
  verified remote transport and native parity.
- **SYMPTOM**: Not recorded.
- **CAUSE**: Desktop chat conversion and reboot recovery were subsequently
  removed.
- **REFERENCES**: Not recorded.

## 2026-11-10 — Signed preview identity, trigger and scope (D61/D64)

- **RULE**: Keep macOS signed previews as `ai.2389.spaceport` under 2389
  Research, Inc (Team `HD9NM9NSMK`), with tag-triggered GitHub prereleases.
  Ordinary development builds retain `spaceport.development`; Windows signing is
  deferred for this task. This does not enable the inherited public release
  pipeline; D65 separately selects Sparkle preview updates.
- **SYMPTOM**: Not recorded.
- **CAUSE**: D61/D64 approve the signed preview route for the fork.
- **REFERENCES**: Not recorded.

## 2026-11-10 — Hosted repository move and permissions (D66)

- **RULE**: D66 moves the hosted repository and active feed to
  `2389-research/spaceport` and keeps trusted actor `harperreed`.
  Review jvcf restores ordinary Windows check, native and selected package jobs.
  Only the performance step stays gated by `WINDOWS_ENABLED` pending h8g5.
  Require scheduled Windows jobs in aggregate and preview source checks.
  A green run does not establish Windows performance acceptance.
  Keep macOS/Linux release checks required. Organization members inherit
  repository permissions after transfer, so do not retain the old sole-writer
  assumption. Do not recreate `harperreed/spaceport`.
- **SYMPTOM**: Existing installed previews cannot reach the transferred feed.
- **CAUSE**: Existing installed previews need GitHub's old-path redirects to
  reach the transferred feed.
- **REFERENCES**: Not recorded.

## 2026-11-10 — CI prerequisites for a tagged preview

- **RULE**: The tagged commit must belong to main and pass full CI there. Local
  installed-keychain access does not supply CI's P12/password, and the
  protected signing environment needs separate main-branch and `v*` tag
  deployment rules.
- **SYMPTOM**: Not recorded.
- **CAUSE**: Not recorded.
- **REFERENCES**: Not recorded.

## 2026-11-10 — `RELEASE_BUILD` belongs to app constants

- **RULE**: The legacy `RELEASE_BUILD` flag now belongs to app constants after
  deleting the unused credential store; its expression and false preview policy
  stay unchanged.
- **SYMPTOM**: Not recorded.
- **CAUSE**: The unused credential store was deleted.
- **REFERENCES**: Not recorded.

## 2026-11-10 — Keep private signing files out of Git and Cargo

- **RULE**: Keep private signing files outside Git and never pass signing
  secrets into Cargo.
- **SYMPTOM**: Not recorded.
- **CAUSE**: Not recorded.
- **REFERENCES**: Not recorded.

## 2026-11-10 — Notarization submission is a side effect

- **RULE**: Persist the notarization submission's UUID and poll it; never retry
  an uncertain submission blindly. When CI progress is opaque, query
  authenticated notarytool history/info read-only before inferring an Apple
  queue delay. An Accepted app submission does not establish that a DMG was
  submitted or published.
- **SYMPTOM**: A stuck release attributed to an Apple queue delay, or a
  published release assumed from an accepted app submission.
- **CAUSE**: The combined step also covers signing, stapling and disk-image
  creation.
- **REFERENCES**: Not recorded.

## 2026-11-10 — Signing scripts must run under macOS Bash 3.2

- **RULE**: Verify actual output and completion status of signing scripts;
  empty arrays with nounset can exit early under macOS Bash 3.2.
- **SYMPTOM**: A signing script exits early with no visible failure.
- **CAUSE**: Empty arrays with nounset can exit early in Bash 3.2.
- **REFERENCES**: Not recorded.

## 2026-11-10 — Match complete codesign identity fields

- **RULE**: Match complete codesign identity fields.
- **SYMPTOM**: Substring matches accept ID suffixes.
- **CAUSE**: Substring matching does not compare complete identity fields.
- **REFERENCES**: Not recorded.

## 2026-11-10 — A valid `find-identity` result is not signing access

- **RULE**: Prefer an isolated temporary import of the supplied matching P12
  over changing the login keychain, and do not read a valid
  `security find-identity` result as proof of signing access.
- **SYMPTOM**: A locked login keychain rejects signing.
- **CAUSE**: A locked login keychain can still reject signing.
- **REFERENCES**: Not recorded.

## 2026-11-09 — D60 deletes the updater, not Spaceport updates

- **RULE**: Preserve the release-build refusal; do not accidentally enable
  upstream release publishing by deleting its guard.
- **SYMPTOM**: Not recorded.
- **CAUSE**: D60 authorizes deleting the inherited custom updater, not dropping
  Spaceport updates; D65 separately selects Sparkle for macOS previews, and
  other platform and package-manager routes, identity and complete update tests
  remain T12 gates.
- **REFERENCES**: Not recorded.

## 2026-11-10 — Sparkle must require a signed feed

- **RULE**: Require `SURequireSignedFeed` and exact numeric
  `SUSignedFeedFailureExpirationInterval=0`, and verify archives before
  extraction.
- **SYMPTOM**: Not recorded.
- **CAUSE**: Sparkle otherwise allows recovery after 20 days of feed-signature
  failure.
- **REFERENCES**: Not recorded.

## 2026-11-10 — Sparkle extraction callbacks do not prove unarchiving

- **RULE**: Neither callback proves unarchiving; do not treat either extraction
  callback as proof of it.
- **SYMPTOM**: `will-extract-update` and `did-extract-update` both arrive
  before installer validation.
- **CAUSE**: Sparkle 2.10.1 emits both callbacks before installer validation:
  `SPUInstallerDriver` immediately completes launcher setup, and
  `SPUCoreBasedUpdateDriver` forwards that completion as did-extract.
- **REFERENCES**: Not recorded.

## 2026-11-10 — What archive refusal must show

- **RULE**: Archive refusal needs the exact pre-unarchive 4005 → [4005,3002]
  cause and EdDSA mismatch; validly signed identity/version failures may
  unarchive to inspect metadata. All negative cases must forbid genuine
  installation/relaunch events or later-stage install choices.
- **SYMPTOM**: Not recorded.
- **CAUSE**: Not recorded.
- **REFERENCES**: Not recorded.

## 2026-11-10 — Sparkle trusts its signed publisher

- **RULE**: Do not add an invented unconditional installer check: exact
  Spaceport ID/Team belong to the release gate.
- **SYMPTOM**: Not recorded.
- **CAUSE**: Sparkle trusts its signed publisher to rotate keys and app
  identity.
- **REFERENCES**: Not recorded.

## 2026-11-10 — Pin and bundle the Sparkle SDK LICENSE separately

- **RULE**: The SDK root LICENSE is outside Sparkle.framework: pin and bundle
  it separately.
- **SYMPTOM**: Not recorded.
- **CAUSE**: The SDK root LICENSE is outside Sparkle.framework.
- **REFERENCES**: Not recorded.

## 2026-11-10 — Feature builds need an explicit bundle-relative rpath

- **RULE**: Feature builds need an explicit bundle-relative rpath, and compiled
  worktree exclusion is independent of preview app identity.
- **SYMPTOM**: Not recorded.
- **CAUSE**: Not recorded.
- **REFERENCES**: Not recorded.

## 2026-11-11 — A private HOME does not isolate the OS temp root

- **RULE**: Pin the exact OS temp root and its owner before launch.
- **SYMPTOM**: `TMPDIR` is set but the fixture still writes into the account's
  protected temp root.
- **CAUSE**: Foundation `NSTemporaryDirectory` uses the OS protected per-user
  temp root despite `TMPDIR`.
- **REFERENCES**: Not recorded.

## 2026-11-11 — `cfprefsd` writes outside `CFFIXED_USER_HOME`

- **RULE**: Prove preference-domain absence before launch, and clean only owned
  test preference values after process exit.
- **SYMPTOM**: Preferences appear under the account's Preferences directory
  despite a private `HOME` and `CFFIXED_USER_HOME`.
- **CAUSE**: `cfprefsd` may write the unique fixture bundle-ID domain under the
  account's Preferences directory despite `CFFIXED_USER_HOME`.
- **REFERENCES**: Not recorded.

## 2026-11-11 — LaunchServices overwrites `HOME`

- **RULE**: The fixture validates its signed private root, then restores its
  explicit environment with a bounded same-PID exec before running every
  isolation guard.
- **SYMPTOM**: `HOME` is not the private fixture root once the app is running.
- **CAUSE**: LaunchServices overwrites `HOME` despite signed `LSEnvironment`.
- **REFERENCES**: Not recorded.

## 2026-11-11 — Sparkle installer helpers use the account cache domain

- **RULE**: Claim the absent exact random cache-domain directories 0700 in both
  account and fixture homes before launch, and retain owner/device/inode proof.
- **SYMPTOM**: Cache directories appear in the account's `Library/Caches`
  during an isolated fixture run.
- **CAUSE**: Sparkle installer helpers independently use `NSFileManager`
  `URLForDirectory(NSCachesDirectory, NSUserDomainMask)`, which can choose the
  account `Library/Caches` despite the host's private HOME/search-path guards.
- **REFERENCES**: Not recorded.

## 2026-11-11 — Remove only owned claims in an isolated fixture

- **RULE**: Remove only those same owned claims after hosts/helpers exit, an
  executable-path census is clear, and scoped exact-tree `lsof` finds no open
  files; fail and retain on uncertainty. Never retroactively delete old
  unclaimed caches, signal recorded helper PIDs, or scan/reset personal domains
  or `cfprefsd`.
- **SYMPTOM**: Not recorded.
- **CAUSE**: An unclaimed cache directory, a still-running helper or an open
  file makes the deletion unsafe.
- **REFERENCES**: Not recorded.

## 2026-11-10 — `rust-cache` deletes a mid-job SDK under `target/`

- **RULE**: Keep the CI Sparkle SDK under `RUNNER_TEMP`, with linker, loader
  and official-tool tests using that same explicit path. An explicitly
  configured missing SDK must fail its signing test instead of silently
  skipping it.
- **SYMPTOM**: A signing test fails or silently skips after a partial cache hit,
  because the SDK acquired earlier in the job is gone.
- **CAUSE**: `rust-cache` recursively removes non-Cargo files beneath `target/`
  on a partial cache hit, including an SDK acquired earlier in the job.
- **REFERENCES**: Not recorded.

## 2026-11-09 — Usage provider names and icons are presentation only

- **RULE**: Keep provider visibility filtering in presentation after removing
  collectors. Do not restore polling or sign-in readers to populate a missing
  feed; G-OBS requires a separately verified authoritative source.
- **SYMPTOM**: Not recorded.
- **CAUSE**: Usage provider names/icons are passive presentation metadata, not
  evidence of a supported report feed.
- **REFERENCES**: Not recorded.

## 2026-11-09 — Unknown CLI families reject before control JSON handling

- **RULE**: Valid control commands keep their JSON contract; do not parse an
  unknown-family rejection as their response schema.
- **SYMPTOM**: Even with `--json`, a removed family such as `system` returns
  exit 2 and text usage on stderr.
- **CAUSE**: Unknown top-level CLI families reject through the launch parser
  before control-command JSON handling.
- **REFERENCES**: Not recorded.

## 2026-11-10 — About credits and attribution

- **RULE**: About credits distinguish Spaceport by 2389 Research, Inc., its
  modifications copyright, and original Sheprd work by Fabien Penso. Preserve
  both Herdr and Sheprd team credits (D59), the original NOTICE text and
  Apache-2.0. Spaceport's repository is primary; upstream links stay under
  credits.
- **SYMPTOM**: Not recorded.
- **CAUSE**: Not recorded.
- **REFERENCES**: Not recorded.

## 2026-11-10 — The shared repository constant drives banner and issue links

- **RULE**: The worktree PR banner and Report issue route use the same
  repository constant; check their expected URLs when that constant changes.
- **SYMPTOM**: Banner or Report issue links point at the wrong repository.
- **CAUSE**: Both surfaces read one repository constant.
- **REFERENCES**: Not recorded.

## 2026-11-10 — Community surfaces name Spaceport; security route is open

- **RULE**: Community issue forms, duplicate-search links and the Documentation
  contact name Spaceport; preserve real Herdr daemon links. The inherited
  private security-report contact still needs an owner-approved Spaceport route
  before general production shipping.
- **SYMPTOM**: Not recorded.
- **CAUSE**: Not recorded.
- **REFERENCES**: Not recorded.

## 2026-11-10 — Fluent placeables carry invisible FSI/PDI marks

- **RULE**: Keep Fluent placeables' invisible FSI/PDI marks in rendered/source
  text, while native glyph inspection skips their ink bounds and compares every
  visible character. The full sidebar bidi check separately verifies their zero
  width and lack of missing-glyph boxes.
- **SYMPTOM**: Not recorded.
- **CAUSE**: Fluent placeables include invisible FSI/PDI marks.
- **REFERENCES**: Not recorded.

## 2026-11-09 — Task worktrees live inside the main checkout

- **RULE**: Doctor Biz wants future task worktrees inside the main checkout's
  `.worktrees/<task>` directory; keep active worktrees and delivered test
  artifacts intact when cleaning old sibling checkouts.
- **SYMPTOM**: Not recorded.
- **CAUSE**: The ignore entry prevents nested checkouts from appearing as source
  changes.
- **REFERENCES**: Not recorded.

## 2026-11-09 — Runtime generation alone is not a deletion criterion

- **RULE**: Preserve exact runtime ownership and working UX; remove excluded
  feature routes independently. Core-specific gaps do not establish Herdr 1
  parity.
- **SYMPTOM**: Not recorded.
- **CAUSE**: D58 in `docs/spaceport/plan/decisions.json` permits Herdr 1
  alongside Core and supersedes D55.
- **REFERENCES**: `docs/spaceport/plan/decisions.json`

## 2026-11-09 — The browser module keeps tab identity and navigation

- **RULE**: Keep that container, `crates/spaceport-gpui/src/browser`, for Core and
  Herdr 1 terminal and Devices views; old web/local/blank payloads only decode
  so the store can discard them while reserving their IDs.
- **SYMPTOM**: Not recorded.
- **CAUSE**: After the embedded page engine is removed, that module owns shared
  native tab IDs, split groups, persisted view references, and Herdr terminal
  navigation.
- **REFERENCES**: `crates/spaceport-gpui/src/browser`

## 2026-11-09 — One pseudo-locale leak surface per test

- **RULE**: Test each pseudo-locale leak surface in its own test.
- **SYMPTOM**: Consecutive popovers with the same labels can produce no new
  messages.
- **CAUSE**: Pseudo-locale leak fixtures record newly shaped text.
- **REFERENCES**: Not recorded.

## 2026-11-09 — `under_en_xa` twice toggles the locale off; use `uncut`

- **RULE**: Calling `under_en_xa` twice in one test toggles the thread-local
  pseudo-locale off on the second call, even with a fresh app context, so call
  it once per test. Use the existing `uncut` helper for truncation measurements,
  and keep whole untranslated labels subject to the leak assertion.
- **SYMPTOM**: A pseudo-locale test stops observing leaks after the second
  setup call.
- **CAUSE**: `under_en_xa` toggles a thread-local pseudo-locale.
- **REFERENCES**: Not recorded.

## 2026-11-11 — Runtime announcements belong to Herdr

- **RULE**: Label these Herdr rather than Spaceport: daemon snapshot release
  notes, available versions, install commands and product announcements.
  Spaceport's signed application updates use their separate qualified route.
  Pass the runtime name and versions as Fluent data, and keep shared
  chat-protocol refusal text product-neutral without changing serialized enum
  values.
- **SYMPTOM**: Not recorded.
- **CAUSE**: Daemon snapshot release notes, available versions, install commands
  and product announcements belong to Herdr; pinned Herdr update and
  announcement sources confirm that ownership.
- **REFERENCES**: Not recorded.

## 2026-11-17 — Clean changed members of a shared Cargo target

- **RULE**: Clean changed workspace members under the shared build lock before
  final checks; retain external dependency caches. Use `CARGO_INCREMENTAL=0`
  when sharing this task's space-limited cache. Serialization alone prevents
  concurrent writes, not stale source reuse.
- **SYMPTOM**: Root's tool-liveness build linked an older transcript crate until
  a member clean forced recompilation.
- **CAUSE**: A shared Cargo target can reuse a workspace crate from another
  worktree despite serialized commands.
- **REFERENCES**: [tool-liveness verification](docs/evidence/tool-liveness.md)
