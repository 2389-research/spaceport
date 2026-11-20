<!-- ABOUTME: Tracks isolated release profiles and the remaining whole-window performance work. -->
<!-- ABOUTME: Separates native observations, source findings, qualification limits, and pending regressions. -->
# Working native session profile implementation plan

**Goal:** Resolve gg9v through release profiling and bounded fixes for measured redraw costs.

**Architecture:** Launch only test-owned processes inside the existing Sandbox.
Drive the private Herdr API and Spaceport control CLI.
Use a gated observer only for state the public CLI cannot report.
Sample the owned GUI process; retain the raw stacks, workload observations, CPU readings, and native captures.

**Tech stack:** Existing Rust, GPUI, serde, Sandbox, Herdr TUI/Core, and Windows WPR/xperf.
Mac `sample`/`ps` support remains separate from the held Mac native checks.

## Current checkpoint — November 20

- Kata remains 50 closed and three open; gg9v is the active performance issue.
- Doctor Biz keeps Mac foreground checks and actual speech pending.
- Native `538ff3ca6` is audited with valid settings, captures, identities, and zero-loss traces; animation activity was not recorded.
- Ordinary `e076fbac1` passed both motion regressions after actual RED; published `7d1772994` passed all four ordinary jobs again.
- Disposable desktop preparation passed real enable, repeated-refusal, restoration, and fresh-process checks at `dcadebfaf`.
- Native `dcadebfaf` passed with real animation activity, verified clocks, captures, identities, and zero-loss traces; dominant CPU attribution remains unresolved.
- Core `454d16f22` reached the intended paint failure but broke a projection control; localized setup `acc229aeb` awaits CI.
- Core caching has not been implemented; the proposed boundary and controls appear at this document's end.

## Authority and limits

Root coordinates implementation, CI, exact process ownership, and read-only reviews.
Doctor Biz authorized scoped fixes, validation, normal commits/pushes, draft PR updates, verified Kata closures, and Crew coordination.
No main merge or release is authorized.
Local compilation, VMs, and Mac native windows remain held; disposable Windows CI supplies build and native execution.
Use only the fixture's exact private TUI/Core processes and explicit runtime paths.
No provider credential or personal content enters the fixture.
The controlled Working status is synthetic and must be labeled synthetic in every report.
The existing dense rendering budget does not substitute for this working-session profile.
The original two-TUI harness brief evolved into the qualified mixed TUI/Core work recorded below.
Remaining work includes cache boundaries, chat, frame copying, and polling; a green native job alone cannot close gg9v.

## Files and boundaries

- `tests/working_profile.rs` owns the opt-in native test and its exact child processes.
- `tests/working_profile/` may hold focused measurement and fixture helpers.
- `src/working_profile.rs` owns the integration-test-only observation handshake.
- A shared observation schema may live under `src/working_profile/`.
- `Cargo.toml` registers this test with `required-features = ["integration-test"]`.
- `main.rs` declares the module on macOS and Windows with `integration-test`.
- `app.rs` starts observation only for a normal window with the explicit profile environment variable.
- `group_terminals.rs` may expose one gated, borrowed observation accessor; Root reserved this narrow footprint.
- `presentation.rs` may expose the approved gated borrowed frame/image pair, without advancing presentation.

Root released the startup wire gate after the native Settings build and the focus baseline build.
The observer never calls `Presentation::picture`, steers a connection, or requests a redraw.
Latest projected surfaces must not be labeled presented surfaces.
Root approved the exact borrowed presented-frame/image accessor under the supported-platform/integration-test gate.

## Test and observation contracts

`SPACEPORT_WORKING_PROFILE_DIR` is a new, integration-test-only harness contract.
Its value names the fixture's existing absolute private directory.
Normal launches without it create no profile observer.
The harness starts Spaceport with its explicit private client socket.
It supplies `HERDR_TEST_BINARY` explicitly and uses a private fixture parent.
Mac tests require a short parent such as `/tmp`; Windows CI creates its own parent under `RUNNER_TEMP`.
Mixed tests also require the explicitly selected `HERDR_CORE_TEST_BINARY`.
Every GUI, API, CLI, sample, and CPU command uses the Sandbox's cleared environment.

The observer answers bounded explicit requests outside the sampling interval.
It reports actual group identities, selected terminal tabs, surface revisions, dimensions, pane counts, and generated-marker presence.
It saves real GPU captures without replacing the renderer.
Observation must not poll or redraw the UI during a measured interval.
Disk reads, writes, socket waits, and image encoding stay off the UI thread.
The test report labels projected and presented observations separately.

The existing public window listing exposes a selected workspace but no terminal group or frame counts.
Therefore command queue acknowledgement alone cannot establish readiness.
The observer and captures must establish two distinct, populated, drawn terminal groups before sampling.

## Workload

1. Start the explicitly selected installed Herdr TUI in private HOME/XDG/config/socket paths.
2. Create a uniquely labeled workspace using the private newline JSON API.
3. Read every workspace, tab, and pane ID from real responses or observed snapshots.
4. Generate bounded, deterministic terminal output using semantic TextCommit and Enter.
5. Keep another terminal idle.
6. Use private control CLI `run split_editor` and `run new_tab` to populate both Spaceport groups.
7. Wait for observed groups, actual surfaces, and visible generated markers; retain a native capture.
8. Sample an idle phase before reporting synthetic Working.
9. Report synthetic Working through the real private `pane.report_agent` API and observe the daemon's status projection.
10. Sample Working with the same two populated groups and output workload recorded explicitly.
11. Report Idle with sequence two through the same verified contract and verify the resulting state.
12. Sample the final idle phase, then stop and reap only exact owned GUI/server children.

The known report request uses `pane_id`, `source`, `agent`, `state`, and monotonic `seq`.
The existing live test proves those fields for `pane.report_agent`.
Two real TUI tabs provide the required terminals; no pane-split or agent-release API is needed.
No speculative API fields or endpoints may be added.

## Measurements

Each phase records wall duration and cumulative process CPU time before and after sampling.
Process CPU percentage derives from the CPU-time delta divided by wall duration.
It is an all-thread process metric, not a UI-thread CPU percentage.
The raw `sample` artifact supplies actual UI-thread stacks for the owned GUI PID.
Reports must retain sample duration, interval, PID, workload, generated byte/line counts, and observer state.
Each phase invokes `/usr/bin/sample PID 5 1 -file PATH` against the exact owned GUI child.
Each CPU endpoint invokes `/bin/ps -p PID -o time=` in the cleared sandbox environment.
The wall interval encloses both CPU reads and the sample command.
The report includes measurement overhead and sampler perturbation; it does not estimate uninstrumented CPU usage.
No observer request runs during this interval.
The profile imposes no uncalibrated speed threshold.
The first valid run establishes measured behavior and potential bottlenecks.

## Checkpoints

- [x] Write measurement semantics tests and the bounded harness contract before implementing observation.
- [x] Have Root record valid runtime reds or document an impractical baseline precisely.
- [x] Implement exact child ownership, bounded waits, private commands, and artifact retention.
- [x] Implement only the approved observation accessors and startup wires.
- [x] Require actual group/surface readiness and native captures before sampling.
- [ ] Have Root run the release opt-in fixture with explicit Herdr and short temporary paths.
- [ ] Inspect raw CPU/stack evidence and label all unsupported claims.
- [ ] Obtain independent ownership and measurement review.
- [ ] Run scoped validators and the shared host/Windows gates.

These checked implementation items do not claim compilation or native execution success.
Root ran the measurement tests before their implementations.
Two valid-clock/interval tests failed with explicit unimplemented errors; the refusal test passed.
The baseline log is `/tmp/spaceport-9s44-profile-measurement-red.log`.
The baseline executable is `/tmp/spaceport-9s44-profile-measurement-red-tests`.
Root reports all three measurement tests green after implementation.
Native measurement remains pending.

## Launch and artifacts

Root must build and run this test on an active macOS desktop with the release GUI binary.
The exact test target is `working_profile`; its ignored native case is `working_terminal_groups_profile`.
Use `--features integration-test --release --test working_profile -- --ignored --nocapture`.
Set `HERDR_TEST_BINARY` to the explicit installed Herdr TUI executable.
Set `HERDR_TEST_TMPDIR=/tmp` explicitly.
The harness refuses debug execution before creating any child.
The three ordinary measurement tests can run without a desktop or daemon.

The harness writes private app settings through their actual `config.toml` and `view.toml` paths.
It sets immediate tab creation and disables managed skill installation only inside the sandbox.
It sets top-level `show_listening_ports = false` in private `config/spaceport/config.toml`.
The phase and final reports explicitly label the listening-port poller disabled.
This avoids ambient port/process scans and their polling noise during the synthetic group workload.
The owned-PID `ps` CPU reads remain part of the measurement instrumentation.
It obtains the sole window ID from the real `window list --json` response.
It targets that ID for size, split, and new-tab commands.
The observer verifies two distinct selected tabs and populated, connected, current presented frames.
At least one presented frame must contain the complete generated output marker.
Every phase saves native GPU captures before and after the sample, plus typed observations.
The final `report.json` includes both CPU clocks, wall duration, process CPU percentage, and raw sample paths.
The private directory survives both success and failure for review.

Each observation records GPUI active and visible states, native key status, and AppKit active/hidden states.
GPUI visibility means frames can be shown; it includes occlusion and display state, beyond the app's hidden flag.
The existing exact-window native helper records full AppKit activation facts without sending input or changing activation.
The harness requests the actual selected window through the supported `window focus ID` CLI before readiness checks.
Before each sample, it requires an active, visible, native-key window and an active, unhidden app.
If those conditions remain false, it saves a capture and refuses foreground qualification before sampling.
Each completed phase records active flags at both endpoints and saves its own JSON, even if final activation fails.
The observer does not monitor activation during sampling; endpoint checks do not prove uninterrupted foreground state.
Inactive or hidden measurements cannot close foreground performance work, including the gg9v qualification gate.

The first combined compile found a missing `AppContext` trait import for `background_spawn`.
That import is corrected; Root reports the next combined compile succeeded.

Independent review found a gap between the readiness wait and the final captured observation.
The activation wait could outlive group or synthetic status readiness.
Five tests exercised the actual pre-sample consumer with its existing foreground-only guard.
The valid Idle/Working case passed; four invalid readiness cases failed as intended.
The recorded runtime baseline is `/tmp/spaceport-9s44-profile-before-red.log`.
The cases remove one group, stale a presented frame, remove Working, or introduce Working during Idle.
The final pre-sample guard now rechecks foreground, populated current groups, and the exact target pane's expected Working membership.
It runs after capture and before either CPU clock or sample command.
It adds no polling inside the measured interval.
The latest Root run passes these readiness cases.
Independent review approved the exact readiness guard and private poller labels.

The private observer accepts newline JSON requests through `profile.sock` inside the owned directory.
It permits at most 512 requests and eight captures, with 64 KiB request/reply bounds and 15-second socket deadlines.
Its dedicated thread waits without a GPUI timer, UI polling, or a blocked GPUI worker.
Only an explicit request schedules a read of the real window.
Image encoding, file writes, and reply writes run off the UI thread.
The explicit Stop request ends the observer thread before normal teardown.
If a failure leaves it waiting, terminating the exact owned GUI child bounds its lifetime.

⚠️ A measured native working session remains pending because the GUI cannot establish foreground presentation in this environment.
This fixture measures settled generated output and a synthetic status transition.
It does not exercise a streaming producer, real provider execution, ACP, or Core.

## First release attempt

The first release attempt failed while waiting for the seed client's exact generated output marker.
The log is `/tmp/spaceport-9s44-working-native-profile-first.log`.
The retained private fixture is `/tmp/h46be-d5eda0-0`.
The daemon started and private shell history contains the complete generated command.
No GUI started, no CPU clock was sampled, and no working-session measurement exists from this attempt.
The earlier timeout omitted the last observed surface, so it could not identify the marker failure's cause.

The seed peer now retains bounded diagnostic facts from actual snapshot and surface events.
Timeout context and `seed-observation.json` record event counts, selected IDs, revisions, geometry, and final surface rows.
Row diagnostics retain at most 32 rows, 160 cells per row, and 320 characters per row.
The config diagnostic is capped at 256 characters.
These observations contain only the sandbox's generated terminal content.
The diagnostic checkpoint preserved startup ordering, command, target, timeout, and the exact non-echo marker predicate.

Root's diagnostic rerun reproduced the timeout in `/tmp/spaceport-9s44-working-native-profile-seed-diagnostic.log`.
The retained observation is `/tmp/h17945-1383c9e0-0/seed-observation.json`.
It records 18 surfaces and matching snapshot/surface boot IDs and projection revision two.
Both snapshot and surface target the created pane `w2:p1`.
The actual 80-cell output row contains `SP_PROFILE_READY`, 63 spaces, and the scrollbar cell `▐`.
Whole-row whitespace trimming preserves that scrollbar, so equality with the marker rejects valid generated output.

The original predicate was first shared between the seed consumer and group observer without changing its behavior.
Four protocol-validated frame tests cover the exact observed row, plain output, actual command echo, and row boundaries.
Root recorded the expected runtime red in `/tmp/spaceport-9s44-profile-marker-red.log`.
The decorated-row case failed; plain output, actual command echo refusal, and row-boundary controls passed.
The shared helper now accepts the exact contiguous marker substring within one actual row.
It preserves terminal decorations rather than stripping arbitrary characters.
The command echo must continue failing readiness, and a marker split across rows must remain invalid.
No Herdr source or display behavior changes belong to this correction.
Root's latest target run passed 15 tests, with only the explicit native profile case ignored.
The log is `/tmp/spaceport-9s44-profile-latest-harness-green.log`.
This includes all four marker controls, five readiness cases, and activation/clock/sandbox checks.

## Corrected native attempt

The corrected real release run passed seed generation and started the owned GUI.
Its log is `/tmp/spaceport-9s44-working-native-profile-corrected.log`.
The retained private fixture is `/tmp/hed91-4415b00-0`.
The GUI reported release mode, a connected selected terminal, and the created workspace.
GPUI visibility and activation remained false; native key and app activation also remained false.
AppKit reported a shown, non-miniaturized, eligible window on the active Space, with no bundle identifier.
Drawn group bounds remained zero and no presented frame became available.
The initial presentation wait timed out before either editor split or any CPU/sample interval.
Root reaped the exact owned children.
This result provides no native working-session performance measurement and cannot close the foreground qualification gate.

Canonical host lint found four harness issues after this checkpoint.
The harness removes an unused import and an unfulfilled dead-code expectation.
The response guard uses the suggested let-chain.
Only the included test sandbox module allows its existing `Command::new`; the shared sandbox remains unchanged.
These lint-only corrections preserve the verified workload and qualification guards.
Canonical host `just lint` now passes both all-features and no-default-features workspace/all-target checks.
Canonical Windows GNU cross-lint passes both configurations too; native Windows execution remains unqualified.
Logs are `/tmp/spaceport-9s44-latest-host-lint-green.log` and `/tmp/spaceport-9s44-latest-windows-lint.log`.

## Failure handling

Bound startup, API reads, observer requests, CLI commands, sampling, and shutdown.
Every failure reports the exact stage and owned PID.
Stop and reap owned children before handling fixture files.
Preserve the private fixture directory and all completed artifacts after failure.
Do not stop a daemon by name, kill a process group, or target a personal session.

## Current checkpoint

The bounded plan is written.
No profile, daemon, Cargo, or native input command has run from this agent.
Shared startup and registration files are wired within Root's approved scope.
Root approved the narrow borrowed presented-state and group accessors.
The harness uses only existing verified workspace, input, control, and agent-report contracts.
The latest test harness passes 15 ordinary checks; its real native presentation gate remains blocked as recorded above.
The final ordinary release test rerun passes fifteen tests with no failures and one native opt-in ignored.
The log is `/tmp/spaceport-9s44-profile-stop-tests.log`.
Canonical lint and quick checks pass; actual foreground presentation and CPU measurement remain pending.

## Windows adapter scope, November 20

Kata gg9v requires a release profile before redraw changes.
It adds no live-provider, model, or credential requirement.
The controlled Working status above remains a valid first measurement through the real private Herdr API.
Call it controlled Working status; do not claim execution of an AI agent.
The settled TUI workload cannot close Core, chat, continuous output, or every mirrored path by itself.

Doctor Biz keeps Mac and audible-speech checks pending.
Use a disposable windows-2025 runner for this platform adapter.
Keep the normal app, existing Sandbox, checksum-pinned daemon, authoritative group readiness, and exact owned-child cleanup.
Reuse the shared observation schema and marker checks instead of duplicating their policy.
Keep native Windows activation facts distinct from Mac-only AppKit facts.
Use existing interprocess support and private endpoint policies for the Windows observer.
Keep observation off the measured interval and blocking I/O off the UI thread.

First inspect installed sampling tools and built-in WPR profiles without starting a trace.
The branch workflow is windows-working-profile.yml.
Actual tool availability, permissions, symbols, and trace extraction remain unqualified until observed.
Retain exact source, release executable identity, raw CPU endpoints, wall interval, and actual frame observations.
Do not estimate measured CPU from frame count or a nominal frame cost.
Keep raw captures private until their scope and contents have been inspected.

- [x] Qualify installed tool/configuration support; trace quality remains pending below.
- [x] Define Windows transport, native observation, and measurement edges while preserving the Mac path.
- [x] Add tests first for real endpoint bounds, native readiness, and CPU-clock semantics.
- [x] Run the normal release app with idle, controlled Working, and final idle phases.
- [x] Add bounded real TUI output; this does not qualify continuous patch or frame-copy costs.
- [ ] Inspect native captures, actual samples, and CPU/frame data before changing redraw behavior.
- [ ] Verify the original Core, mirror, chat, copying, and polling clauses separately before closing gg9v.

### Windows execution plan and contracts

Use the planning and TDD skills within this existing approved plan.
Keep implementation inline in the assigned worktree; Root owns builds and CI dispatch.

1. Add Windows clock and observation contract tests in `tests/working_profile/windows_contracts.rs`.
   Root runs the test-only checkpoint before implementation.
   A separate unsupported `cpu_ticks` parser preserves the Mac clock grammar for runtime RED.
   The existing schema requires Mac-only Boolean facts.
2. Add `src/working_profile/transport.rs` for the observer's bounded platform endpoint.
   Keep Unix socket deadlines and private-directory checks.
   Windows uses existing private-endpoint SID/owner checks and a private first-instance named pipe.
   Requests and replies retain the 64 KiB bound, capture bound, and dedicated waiting thread.
3. Keep `src/working_profile/report.rs` authoritative for shared readiness and frame observations.
   Make Mac-only facts optional; Windows records its actual GPUI HWND separately.
   Add Windows native facts through a bounded PowerShell helper in `tests/working_profile/`.
   Read foreground HWND, visibility, minimized state, process ID, and actual HWND thread ID.
   Require the HWND owner to match the owned GUI PID before sampling.
4. Adapt `tests/working_profile/peer.rs` to the sandbox's actual Windows PowerShell producer and API pipe.
   Generate 120 bounded real output lines and the split ready marker.
   Preserve correlated `pane.report_agent` replies and monotonic status sequence numbers.
5. Split only platform measurement edges from `tests/working_profile/harness.rs`.
   Windows reads exact-PID `TotalProcessorTime.Ticks`, using 100ns units.
   Retain raw endpoints and measured wall duration around each controlled phase.
   Start an owned WPR CPU instance only after refusing unrelated active recording.
   Stop only after acknowledged start; retain raw ETL privately.
   Extract stacks with exact GUI PID and actual HWND thread filtering through installed xperf.
   Qualify actual extraction and symbol support from observed output; never invent stack counts.
6. Widen only integration-test platform gates in `main.rs`, `app.rs`, `group_terminals.rs`, and `presentation.rs`.
   Preserve normal application rendering and all Mac-specific native observation behavior.
   Do not enable GPUI's profiler feature: its optional hdrhistogram dependency is absent from the current lockfile.
7. Root runs Windows ordinary tests, then the explicit release native fixture.
   Inspect captures, phase JSON, and filtered stack text before reporting measurement success.
   Run static format/diff/size checks and obtain independent review before publication.

Ordinary RED command:
`cargo test --locked -p sheprd-gpui --features integration-test --test working_profile -- --nocapture`

These ordinary clock/schema contracts do not depend on optimization.
Only the explicit native profile accepts a release executable.

Native GREEN command after implementation:
`cargo test --locked --release -p sheprd-gpui --features integration-test --test working_profile working_terminal_groups_profile -- --ignored --exact --nocapture`

Set `HERDR_TEST_BINARY` to the checksum-pinned installed daemon's absolute executable.
Set `HERDR_TEST_TMPDIR` to an existing private runner temporary parent.
No local build, daemon, native window, VM, model, or credential run is authorized for this agent.

Primary measurement references:
[WPR options](https://learn.microsoft.com/en-us/windows-hardware/test/wpt/wpr-command-line-options),
[xperf actions](https://learn.microsoft.com/en-us/windows-hardware/test/wpt/xperf-actions),
[stack filters](https://learn.microsoft.com/en-us/windows-hardware/test/wpt/stack), and
[process CPU clock](https://learn.microsoft.com/en-us/dotnet/api/system.diagnostics.process.totalprocessortime).

### Windows contract RED and implementation checkpoint

Root ran ordinary Windows contracts at `45dec3577` in run `44766284975`, job `136391654250`.
Four controls passed and two new contracts failed for the intended reasons.
The tick parser returned its explicit unsupported error.
The observation decoder rejected null AppKit facts because the old fields required Booleans.
No GUI, daemon, WPR trace, model, or credential ran in that checkpoint.

The adapter now keeps the Mac clock grammar and shares only interval CPU arithmetic.
Windows uses the explicit `cpu_ticks` parser with 100ns units and signed TimeSpan limits.
Windows observations carry an actual GPUI HWND and null Mac-only facts.
The harness reads that HWND's actual owner/thread, foreground HWND, visibility, and minimized state.
Only the owned GUI HWND can qualify the Windows foreground gate.

The Windows observer uses a first-instance, local-only pipe with the existing user-only SID/DACL policy.
Its complete bounded reply fits the pipe's output buffer.
Idle reads peek with a deadline on the dedicated worker thread.
Readiness uses a private `profile.ready` file after a successful observer bind.
The harness connects with the existing owner-checked, deadline-bearing client stream.

New real-pipe tests cover rebind refusal, whole-reply buffering, and idle-read timeout.
No old Windows profile server exists for a runtime baseline: the original module was Mac-only.
These new server contracts still require actual Windows GREEN execution.
Ordinary helper tests read the actual test process's CPU clock and reject an invalid real HWND.
The invalid-HWND test requires the helper's specific ownership diagnostic.

Only formatting and `git diff --check` have run locally under Root's compilation hold.
The temporary sampling refusal remains until actual tool output establishes the supported trace edges.

### Complete-line boundary RED

Run `44770754711` tested source `d1c5ff7d178694c5a3c18c7021772b82dc43fb61` on both platforms.
Mac job `136407451705` passed 17 ordinary tests and failed the new complete-line boundary test.
Windows job `136407451151` passed 23 ordinary tests and failed that test plus the invalid-HWND helper control.
Both jobs ignored the explicit native profile case.
The encoder accepted a 64 KiB JSON payload before adding its newline.
The real consumer rejected that complete line because the newline exceeded its bound.
The encoder must reserve one byte for that newline.

Windows clock, schema, real-pipe deadlines, and complete-line round-trip controls passed.
The invalid-HWND helper failed after 15 seconds with empty stderr.
Its cause remains unknown; retain the deadline and add stage diagnostics before changing behavior.
No GUI, daemon, WPR recording, CPU profile, or stack extraction ran.

### Native helper and recorder RED

Run `44772447956` tested source `7375a66ebb7067b54eda3d5407fcd4f04f98e946`.
Mac job `136413342857` passed all 18 ordinary tests, including both complete-line boundary controls.
Windows job `136413343402` passed 26 tests and failed two; its native profile remained ignored.
The recorder's allowed-telemetry control failed with the explicit unimplemented preflight error.
Its recording and unclassified-state refusal controls passed.
Both real serialization boundary controls passed on Windows too.

Passive helper stages reached script entry and `add_type_begin`, then hit the unchanged 15-second deadline.
They never reached `add_type_complete` or the native window query.
The separate CPU helper passed using only direct .NET calls.
This localizes the failure to the first cmdlet boundary.
The established [module bootstrap](review-ci-qualification.md#explicit-system-module-bootstrap) addresses that prior sandbox failure class.
Reuse that bootstrap while preserving JSON-only stdout, private diagnostics, isolation, and the existing deadline.
The correction still requires actual Windows verification.

### Private retention prerequisite

The runner's tool probe found GPG at the installed Git path.
Raw machine traces must remain private after the disposable runner exits.
Use GPG's [recipient-file option](https://www.gnupg.org/documentation/manuals/gnupg/GPG-Key-related-Options.html) with the reviewed public-only export.
Its fingerprint is `F550E9D83215B9D9A5437A4176C5B051D8CB8646`; it expires on December 20, 2026.
The private key stays outside Git in operator state.
The retention job first encrypts only a generated marker archive.
Upload only its encrypted `.gpg` output.
Download and decrypt that probe privately before using this path for native traces.
The local generated-marker encryption/decryption round trip passed exact byte comparison.
Encryption used a separate empty keyring and only the public recipient file.
The Windows-to-Mac probe and native trace retention remain unqualified.

Run `44776566454`, job `136427828678`, tested the generated marker probe at `ab604e140`.
Installed Git GPG 2.4.9 treated native Windows paths as relative MSYS paths and refused encryption.
No artifact uploaded.
Convert all four GPG paths through its sibling `cygpath -u` before retrying in a fresh runner.
Keep native Windows paths for ZIP creation and the upload target.
The correction still needs a successful remote encryption and private decryption.

At source `addb3635ff9f1a46796104086654dbb21301dfe8`, retention job `136431018132` passed.
Run `44777201478` uploaded only encrypted artifact `14649278750`.
The downloaded artifact and ciphertext hashes match the runner's recorded hashes.
Private decryption recovered a ZIP containing only `marker.txt`.
Its bytes exactly match the authored marker and its SHA-256 is recorded in the [receipt](review-squad/windows-profile/retention-addb3635f.json).
This qualifies the generated-marker Windows-to-Mac transfer, not native trace contents or performance.

### Bootstrap GREEN and real-inventory refusal

Run `44774374418` tested source `9f39d8525519dc9ddffbbe4ddc0b3d74ee00bb90`.
Mac job `136420117937` passed 18 ordinary tests and scoped app/harness Clippy.
Windows job `136420117445` passed 30 ordinary tests, with two explicit resource tests ignored.
The native helper completed Utility import, compilation, and the actual invalid-HWND ownership query within 1.38 seconds.
Both serialization boundaries and the recorder unit controls passed.

The separate installed-tool consumer refused `DefenderApiLoggerLowPriv`.
The broad `contains("wpr")` check accidentally matches `LowPriv`.
Use a delimited WPR identity while retaining system-mode and profiling-provider refusals.
Windows Clippy also found test-module ordering in `harness/windows.rs`; fix the ordering without an allowance.
No recording ran; these failures keep Windows qualification pending.

### Sampler lifecycle test checkpoint

The candidate prepares owned-instance sampling and post-phase PID/thread-filtered extraction.
The workflow still runs no native profile.
Three ordinary ownership tests must first expose receipt failure and known pre-spawn failures.
The current draft retires Stop authority before receipt writing, output setup, or successful child creation.
Its unknown-outcome test must continue rejecting a second submission.
Independent review confirmed the log-setup gap; native recording stays gated on its correction.
Also persist completed CPU clocks and wall duration before Stop, so Stop failure cannot erase those observations.
The current draft still writes that aggregate after successful Stop; this remains a known gap.
These lifecycle changes require runtime RED, correction, and GREEN before any recording.

At `addb3635ff9f1a46796104086654dbb21301dfe8`, Windows job `136431017530` produced exactly the three ownership REDs.
The receipt, missing output parent, and missing executable controls failed for their expected authority assertions.
The unknown-outcome/no-replay control passed; 32 ordinary tests passed and two resource cases remained ignored.
The separate real recorder inventory passed against 105,752 bytes of actual logger output.
Scoped Windows app/harness Clippy passed.
Mac job `136431018190` passed 18 ordinary tests and scoped Clippy.
No native profile ran.

The marker probe now calls `scripts/profile/encrypt-artifacts.ps1` so real captures use the same encryption path.
The extraction and its temporary-directory cleanup still need remote verification.

At `8f9093f1edc27b24709e514b9a477ab98947f834`, retention job `136445967627` passed using the shared script.
Artifact `14652966084` matched its recorded archive and ciphertext hashes.
Private decryption again recovered only the exact authored marker.
The [shared-script receipt](review-squad/windows-profile/retention-8f9093f1e.json) qualifies that producer/consumer path and successful temporary cleanup.
It provides no native profiling measurement.

The ownership correction prepares Stop's command and both output files before Start.
It retains authority when process creation fails and retires authority only after a real child exists.
Receipt failures cannot skip waiting for that child.
Completed CPU/wall JSON now persists before Stop and retains an explicit pending outcome on failure.
Independent static review found no further material defect.
Runtime GREEN remains pending.
The new persistence test reads actual saved JSON after a real missing-executable error.
Its old full-path runtime baseline requires an active WPR recording and was not run while recording remained gated.
That baseline gap does not substitute for the pending native profile.

At `8f9093f1e`, Windows job `136445967801` stopped at three denied unnecessary-qualification diagnostics.
The Stop refactor imports `File`, so the tests must use that import instead of `fs::File`.
No Windows runtime controls ran at that source; their GREEN remains pending.
Mac job `136445967717` passed its ordinary tests and scoped Clippy.

At `69c3200e0`, Windows job `136456602608` compiled and passed 35 ordinary tests, with two resource cases ignored.
Only the receipt-failure control failed: its actual `cmd.exe` child exited with code one before the receipt-cause assertion.
The output-setup, known-spawn, no-replay, and measurement-retention controls passed.
The real recorder inventory and scoped Windows Clippy passed.
Capture the control child's actual stdout/stderr before changing its command invocation.
The workflow contains a throwaway standalone Windows probe for that boundary; it starts no recorder.

Probe job `136466512416` at `032797f0b` reproduced exit one for both argument forms.
Its stderr reports directory-creation errors while processing the executable suffix and command text.
The executable path was `C:\Windows\System32/cmd.exe`.
Compare it with a native-separator path while keeping the wrapper, environment, and arguments unchanged.
The receipt control now uses component-wise native path joining and retains bounded child stderr on failure.
Runtime verification of that correction remains pending.

At `d7faebfd7`, standalone probe job `136470858602` isolated the executable-path difference.
All four mixed-separator cases exited one with directory-creation diagnostics.
All four native-separator cases exited zero; both echo cases returned exactly `acknowledged\r\n` with empty stderr.
The wrapper, cleared environment, deliberate receipt failure, and argument variants stayed unchanged.
This proves the test-helper path correction.
Windows job `136470858021` passed all 36 ordinary tests, with two resource cases ignored.
The actual recorder inventory and scoped app/harness Clippy passed.
Mac job `136470858800`, retention, and tools also passed; run `44788858393` completed successfully.
The throwaway comparison step was removed after this proof; the permanent lifecycle controls remain.

### Concurrent redraw change

PR [145](https://github.com/2389-research/spaceport/pull/145) merged as `55c206e7c6be1a16f2e0643819f4dfc5df360f66` on November 20.
Its only changed file lowers the mark cap from 30 to 20 fps and adds benchmark comments.
The PR reports lower CPU use, but its referenced `docs/notes-gg9v-redraw.md` returns 404 on main.
Root requested the retained profile notes and receipts through Crew.
Those reported numbers remain unverified here.
The PR explicitly leaves Core/chat and mirrored-picture measurements unqualified.
Keep Kata gg9v open and choose the exact measured source before starting a native run.

### Explicit native execution channel

The native workflow runs only on `fix/review-squad-windows-profile-native`.
Ordinary code-branch pushes cannot launch its windows or recorder.
Root pushes an exact reviewed source there only after safety controls and retention qualify.
The first Windows workload measures this branch's original 30 fps cap.
It does not measure the merged 20 fps change or close the remaining Core/chat clauses.
The workflow retains the exact release executable, PDB, identities, captures, and partial traces through the shared sealer.
Only ciphertext uploads; Root must inspect native observations and filtered stacks before qualification.
Native workflow static review found no material defect.
Root verified source `d7faebfd7f8e1afaed07d71729ea464bb50abb8e` on the dedicated native branch.
Run [44790226021](https://github.com/2389-research/spaceport/actions/runs/44790226021) started its release build.
Native measurements and retained evidence remain pending.

### Core follow-up boundary

The existing Windows Core fixture provides private `HC_SESSION`/`HC_SOCKET`, explicit `serve`, and holder-aware shutdown.
Reuse `tests/accessibility_windows/core.rs`; prepare its stop logs before starting Core.
Enable Core in private `config.local.toml` before normal app launch.
Keep the seeded TUI group, then use existing `split_editor` and `new_core_terminal` commands.
Observe creation through real Core listing and the matching app tab location; command dispatch alone is insufficient.
Use actual returned shell arguments before generating bounded Core output through existing attachments.

Current profile readiness proves two TUI frames and cannot qualify a Core group.
Core's existing frame probe observes mailbox state before paint; `shown()` also includes hidden views.
Select the drawn `Shown::Page` and exact Core terminal location.
Require Core identity, live state, and projected marker content.
Label borrowed bounds as last layout and frame content as projected.
Retain identity observations around each native GPU capture.
Root must inspect a unique authored Core marker in those captures before qualifying visible content.
This avoids adding a paint-receipt API solely for profiling.
GPUI captures its last rendered scene; projected metadata can lead that scene after navigation or output.
Finish navigation and output before sampling; keep screenshot review pending until the marker is actually visible.
The GPU scene does not prove physical desktop presentation or uninterrupted foreground state.
Do not call `frame_to_paint` from the observer; it mutates the veil cache.
The Core test checkpoint adds optional projected metadata without changing existing TUI JSON fields.
Its selected-view accessor deliberately returns no observation until remote RED confirms the tests.
Expected failures: six on Mac and five on Windows; three serialization and fixture controls should pass.
Independent static review found no material defect.
No Core process or performance measurement has run.

At `0af2f433b`, Mac job `136489057580` produced the expected runtime RED.
All six selected-Core cases failed at the explicit missing-observation assertion.
The three serialization and fixture controls passed; no compiler or fixture failure substituted for those REDs.
The ordinary harness passed 18 tests with its native case ignored.
Scoped app/harness Clippy passed.
Windows job `136489057556` produced its five expected runtime REDs at the same assertion.
Its three controls, 36 ordinary tests, actual recorder inventory, and scoped Clippy passed.
Neither platform substituted a compiler failure for the expected baseline.
The Mac binary-test link emitted the existing `__eh_frame` size warning.
The same warning appears in base job `136364441214` at `ac69cfa00`.
The existing `block v0.1.6` future-compatibility warning also remains.

At `517770dc8`, Windows passed all eight Core observer tests and its existing contracts.
Mac passed eight observer tests and failed the connection-loss case at its selected-view precondition.
The app correctly hides lost Core groups immediately; the fixture wrongly expected the page to remain visible.
The corrected fixture asserts that exclusion, then reads the narrow accessor's disconnected metadata directly.
The visibility guard and production accessor remain unchanged; remote GREEN still needs confirmation.

### First Windows native TUI result

Run `44790226021`, job `136475625222`, passed the release fixture at exact source `d7faebfd7`.
Artifact `14657851882` contains only the encrypted evidence archive.
Its archive and ciphertext hashes match the runner receipts.
Private decryption, ZIP integrity, and executable/PDB identity checks passed.
The [structured receipt](review-squad/windows-profile/native-d7faebfd7.json) records the verified values and limits.

Root inspected captures 6, 11, 12, and 16.
Each phase's before/after images have matching hashes.
They show two populated terminal groups, authored output, and the controlled Working status in its phase.
The workload contains 120 generated lines and settled output; no AI provider ran.
All six endpoint checks identify the same foreground, visible HWND and its owned process/thread.
They do not prove uninterrupted foreground state or actual animation cadence.

| Phase | Process CPU delta | Wall interval | Process CPU rate |
| --- | ---: | ---: | ---: |
| Idle before | 0.203125 s | 5.592026 s | 3.6324% |
| Synthetic Working | 0.156250 s | 5.454093 s | 2.8648% |
| Idle after | 0.156250 s | 5.394914 s | 2.8962% |

These are all-thread process clocks, expressed relative to one CPU core.
One short sequence supports no improvement or statistical comparison.
An independent review confirmed phase identity, clocks, and acknowledged recording lifecycle.
No CPU, WPR, or xperf stderr diagnostic was present.

The stack exports resolve local app symbols but contain unclassified stack-event hits.
Their commands specify PID/thread filters and omit the event filter.
Microsoft's [stack reference](https://learn.microsoft.com/en-us/windows-hardware/test/wpt/stack) defines that missing filter.
The HTML reports omit event identity; their percentages cannot establish CPU shares.
Retain the raw traces privately and obtain sampled-profile event exports before attributing CPU hotspots.
Sampling interval and trace-range alignment also remain unqualified.
This result qualifies the fixture and process clocks, not Core/chat/mirrors or the merged 20 fps change.

### Next Core readiness checkpoint

Preserve the existing two-TUI-group workload as its own typed case.
Add a separate mixed case containing one generated TUI group and one single-pane Core group.
Match the exact Core ID, tab ID, terminal ID, active role, connection state, and expected marker.
Reject ended, failed, disconnected, hidden, empty, or stale views before starting CPU clocks.
Keep Core content projected; require separate capture inspection for visible-content qualification.
First prove this predicate's failures and positive boundary through the existing ordinary harness target.
Only then wire normal Core startup, app commands, real output, and holder-aware teardown.
No Core runtime starts before its lifecycle implementation and source review are ready.
The mixed predicate checkpoint deliberately returns false.
Nine new ordinary tests should yield one positive-readiness RED and eight passing controls on each platform.
The existing two-TUI predicate and native case remain unchanged.
Independent review found no material readiness defect.

At `78c6006a3`, run `44797481556` produced that exact ordinary runtime RED on both platforms.
Windows passed 44 tests and failed only the mixed positive, with two resource cases ignored.
Mac passed 26 and failed that same positive, with its native case ignored.
Both failures reached the explicit projected-Core/generated-TUI assertion; all eight new refusal controls passed.
Windows Core observer tests, actual recorder inventory, and scoped Clippy passed.
Mac repeated the known lost-group fixture failure; its scoped Clippy passed.
The committed fixture correction and implemented mixed predicate still need remote GREEN together.

At `bdcedbc26`, run `44800498235` passed all four jobs.
Windows passed all 45 ordinary harness tests, eight Core observer tests, actual inventory, and scoped Clippy.
Mac passed all 27 ordinary tests, nine Core observer tests, and scoped Clippy.
The Mac connection-loss fixture correction now passes.
The native resource cases remain ignored in this ordinary run.

The next native run also retains throwaway read-only xperf postprocessing reports.
Installed-tool help at job `136495315872` verifies the event-statistics, sampled-profile, and frequency commands.
Those whole-trace reports remain private and require interpretation before any CPU attribution.
Review found an absent postprocessing deadline; the step now has a five-minute limit before sealing.

### Shared Windows Core owner extraction

Reuse the existing accessibility fixture's runtime owner through `crates/test-support/core_windows.rs`.
Keep unique targets, explicit binary selection, isolated commands, prepared stop streams, and exact bounded waits unchanged.
The caller still owns GUI/client shutdown and failed-state retention.
Its existing command runner owns and reaps the submitted stop child.
Do not import the Core-client live sandbox; its environment and retention policies differ.

The accessibility GUI keeps its explicit frame observer flag locally.
Core serve/stop commands no longer receive that unrelated flag.
The profile must never enable poll-time frame text logging.
Windows CI compiles, runs existing ordinary accessibility controls, and lints that test target.
The existing ignored real Core cases still need explicit native acceptance after extraction.

The dedicated native branch now points to exact reviewed source `bdcedbc267358fd0d9361f245fe3db14f0f88556`.
Run `44802807684`, job `136519017846`, is building its release TUI classification run.
Its Windows contracts and retention passed before launch; Mac headless contracts passed afterward.
The shared runtime extraction is a later source and does not enter this native run.
This still measures the 30 fps TUI source, with no Core process or 20 fps qualification.

At `b5da3c7b3`, run `44803314753` passed all four ordinary jobs.
Windows passed 45 harness tests, eight observer tests, ten ordinary accessibility controls, and actual recorder inventory.
Its app, harness, and accessibility-target Clippy checks passed.
Mac passed 27 harness tests, nine observer tests, and scoped Clippy.
The 34 native accessibility cases remain ignored in this ordinary pass.

### Classification run failure and retained evidence

Native run `44802807684` completed all three TUI phase guards and process CPU intervals.
The first xperf export then failed with status `0x80070570`.
Its idle-before Stop log reports 32,370 dropped events; xperf repeats that count and refuses unreliable data.
The other two Stop logs contain no dropped-event warning, but their analysis never ran.
This proves event loss, not its disk or buffer cause.
The first `d7faebfd7` run's Stop logs contain no such warning.

Failure artifact `14663671044` matched its archive and ciphertext hashes.
Private decryption and ZIP integrity passed.
All completed phase JSON, acknowledged Stop records, captures, and raw traces survived the ordinary failure.
No second-run stack attribution or native qualification is claimed.

The diagnostic collector now reads completed per-phase manifests and requires their matching acknowledged Stop.
It runs after ordinary fixture failure and attempts each safe export independently within its five-minute step bound.
It retains exit status when available, private diagnostics, partial exports, and an explicit unqualified outcome ledger.
The original profile failure remains a failure; only ciphertext uploads.

The next read-only tools job inspects installed `CPU` and `CPU.light` file-mode definitions.
It must verify sampled-profile events, sampled-profile stacks, and actual buffers before selecting Light.
The recorder still uses the existing CPU profile; no loss-ignoring flag or buffer guess was added.

### Mixed native implementation checkpoint

The new ignored Windows case is `working_tui_and_core_profile`.
It starts only explicit private TUI/Core runtimes and enables Core before normal app launch.
It submits creation once, then matches Core list/get state with the app's exact tab location.
It verifies supported observed shell arguments and bounded raw output whose markers cannot come from command echo.
Extra attachments drop before sampling; projected metadata and capture review remain distinct.
The old TUI-only path provides no native baseline for this new orchestration.
Pure mixed readiness had runtime RED and GREEN; real mixed execution still requires ordinary CI and native acceptance.

Lifecycle review found no material defect in registration, shutdown ordering, exact-child reaping, or failure retention.
The shared phase path preserves existing TUI guards and partial measurements.
The next native workflow stages the pinned Core binary and selects this exact mixed case with a fifteen-minute bound.
This workflow change does not affect the already completed TUI classification run.

At `66485b492`, run `44808826883` passed tools, retention, observer cases, and ordinary accessibility controls.
The harness failed compilation on both platforms due to one denied redundant qualification.
Windows also rejected the mixed entry point's too-narrow intermediate re-export.
The correction uses the imported `Workload` name and gives that test-crate re-export the required visibility.
No native Core execution occurred.

Installed WPR details at job `136539796095` show both CPU file profiles use 1,024 KB buffers with count 20.
Verbose includes sampled-profile and context-switch stacks; Light exposes no System Stacks section.
Light cannot supply the required stack evidence and was not selected.
The next tools probe exports the installed verbose definition through its documented read-only command.

At `d387b3fe6`, run `44811394158` passed all four ordinary jobs.
Windows and Mac harness, observer, and scoped Clippy checks passed.
Windows ordinary accessibility controls, recorder inventory, tools, and encrypted retention also passed.
No native Core execution occurred.

### Sampled CPU profile validation

The installed verbose definition includes context-switch stacks and extra event providers.
The new `scripts/profile/spaceport-cpu.wprp` keeps sampled-profile stacks and process/module identity events.
Its single system collector keeps the installed 1,024 KB buffers and count 20.
File and memory definitions share that collector without inheriting other providers.
The ordinary tools job validates available installed schema and both native parser selectors without recording.
Independent review caught its missing job checkout; the job now checks out with read-only contents permission.
Actual installed acceptance remains pending; the recorder still uses the built-in CPU profile.

At `e5eb16eb3`, tools job `136554356112` passed installed XSD validation and both native parser selectors.
WPR 10.0.26100 decoded one 1,024 KB × 20 collector, four keywords, and only sampled-profile stacks.
The Windows checkout's exact profile hash is `b85415f22593dcc7214c121df856877c9ef8c7eebd4c3e1157f0d506020475bc`.
Its checkout newline bytes differ from local LF bytes; each native fixture must retain and hash its actual definition.
This proves configuration acceptance, not recording quality or absence of event loss.

The next ordinary Windows test retains the definition and checks its identity through the recorder's exact WPR executable.
Its initial retention helper deliberately refuses; this should produce runtime RED before implementation.
The recorder remains unchanged at this checkpoint.

Run `44813491558` failed Windows compilation before the intended retention refusal.
The pinned SHA output type lacks `LowerHex`; the test now uses the existing per-byte formatting pattern.
Mac contracts, Windows observer/accessibility controls, tools, and retention passed.
This compile failure supplies no runtime RED evidence.
Review also added an exact retained-path assertion rather than comparing only helper-produced selectors.

Corrected checkpoint `68cadef20fe3ab7cfcdce65fa3201bf926952015` runs under `44816254167`.
Windows job `136565263564` reached the intended runtime refusal: retention is not implemented.
The other 45 harness tests, eight observer tests, ten accessibility controls, inventory, and scoped Clippy passed.
This supplies runtime RED; the earlier compile failure does not.

The helper now retains exact embedded definition bytes and their SHA-256 receipt before Start.
It refuses existing definition or identity files without replacing them.
WPR receives one native selector for the retained copy and explicit file mode.
The instance, Stop authority, pre-opened Stop streams, and cleanup ordering remain unchanged.
The ordinary native-decoder and no-replacement controls still need remote GREEN.
No new native recording or event-loss qualification is claimed.
Independent review found no material defect in retained bytes, selectors, refusal, or recorder ownership.

At `04791a5df`, run `44817841476` passed all runtime checks.
Windows passed 47 harness tests, including actual retained-copy WPR decoding and both no-replacement controls.
Eight observer tests, ten accessibility controls, and actual recorder inventory also passed.
Mac, tools, and encrypted retention jobs passed.
Windows scoped Clippy rejected a redundant test `Digest` import already in scope from its parent.
The correction removes that redundant import; a warning-free remote gate remains pending.

At `2c86f1971ff81690dbfb99b80bc1c8673718961c`, run `44819608809` passed all four jobs.
Windows passed 47 harness tests, eight observer tests, ten accessibility controls, inventory, and scoped Clippy.
Mac passed 27 harness tests, nine observer tests, and scoped Clippy.
The known Mac linker and `block` future-compatibility warnings remain; no new warnings were introduced.

The native branch advanced normally from `bdcedbc26` to this exact validated source.
Native run `44821546335` started with the real mixed TUI/Core case and sampled-only profile selected.
Its build, actual Core execution, capture review, trace quality, and attribution remain pending.
Next inspect its completed job and retained encrypted artifacts; never retry uncertain runtime actions blindly.
Doctor Biz's Mac and actual-speech holds remain in force, with Kata still 50 closed and three open.

### Remaining original performance scope

The mixed native run can qualify one baseline; it cannot close every clause in gg9v.
Source observations below refer to selected candidate `2c86f1971`, not newer main or PR 147.

| Clause | Inspected source or evidence | Remaining proof |
| --- | --- | --- |
| Release profile | First TUI process clocks and captures qualify narrowly; stack event identity did not. | Inspect mixed native captures, loss statistics, actual samples, and executable identity. |
| Working redraw rate | `src/mark_layer.rs` still caps this candidate at 30 fps. Main's 20 fps change is outside it. | Measure actual cadence and compare exact sources before claiming savings. |
| Parked and mirrored terminals | This candidate retains `render_terminal_picture`'s painting canvas. PR 147 proposes cached picture views separately. | Profile a populated native mirror and qualify cache invalidation and appearance. |
| Core panes | `src/core_terminals/view.rs` paints terminal cells through its canvas callback. | Establish its native cost and cache boundaries from actual samples and paint evidence. |
| Chat panes | `render_chats` and `core_pane_body` call chat rendering from the window. | Profile bounded real chat rendering separately; the current fixture has no chat workload. |
| Frame copying | `sheprd-client/src/session.rs` uses `Arc::make_mut` for scroll and ordinary patches. | Measure copy cost under real bounded updates; settled output does not exercise it continuously. |
| Window polling | `window_effects` retains its 16 ms timer before `tick`. | Attribute idle and Working wakeup costs before changing scheduling policy. |

These source observations identify work to measure; they do not establish CPU shares or new regressions.

### First mixed native evidence and configuration defect

Native run `44821546335`, job `136583385750`, passed the real mixed case, all nine exports, and encrypted retention.
Artifact `14673097102` passed archive/ciphertext hashes, private decryption, ZIP integrity, and actual executable/PDB identity checks.
All three trace statistics report zero lost events and buffers, both total and per trace.
All six endpoint checks preserve owned foreground HWND 196992, PID 1836, and UI thread 8176.
Captures 6, 7, 11, and 16 visibly show TUI output and the unique Core marker.
Capture 12 matches 11; capture 17 matches 16 by exact hashes.
Core cleanup acknowledges private Stop and successful owned-serve exit; this is not an independent holder-liveness audit.

The capture audit found a transient configuration diagnostic in capture 6, absent from capture 7.
Private structured logs confirm `MigrationConflict` and a startup listening-port scan.
The fixture writes common settings into personal `config.toml`, then Core writes different settings into `config.local.toml`.
The real loader prefers the latter and correctly refuses conflicting migration.
Thus the hard-coded report field `listening_port_poller_enabled: false` supplies no verified runtime fact for this run.
The process clocks remain observations; first-idle configuration activity and short unequal intervals prohibit comparison claims.

The new ordinary configuration tests use real `Config::prepare_files` and `Config::load_fixture` consumers.
Their initial producer retains the conflicting file layout; TUI should pass and mixed should fail with `MigrationConflict`.
No daemon, window, or recording enters those tests.

At `3e303a633`, Mac job `136597882525` reached the expected configuration runtime RED.
The TUI control passed; mixed preparation failed with the real migration-conflict diagnostic.
The other 28 tests and all nine observer tests passed; scoped Clippy passed.
Windows job `136597882520` reached the same intended runtime failure, with the TUI control passing.
Its other 48 harness tests, eight observer tests, ten accessibility controls, inventory, and scoped Clippy passed.

Preparation now writes one combined local override and prepares managed defaults through the real settings crate.
It parses and validates the required ports, skill, and Core settings before any child starts.
Report metadata now names configured settings and explicitly excludes a live poller-state claim.
Remote GREEN and native configuration acceptance remain pending.
Independent review found no material defect in lock lifetime, startup ordering, configuration scope, or test relevance.

At `538ff3ca6d1bec6b9d92d0fe6d4aef755d8fe1b8`, run `44827740662` passed all four jobs.
Windows passed 49 harness tests, eight observer tests, ten accessibility controls, inventory, and scoped Clippy.
Mac passed 29 harness tests, nine observer tests, and scoped Clippy.
Both real configuration consumers now pass in both workload modes.

The native branch advanced normally to this exact source.
Corrected-config native run `44829422916` started from this exact source.
Inspect its retained settings, structured logs, captures, loss counters, and identities before accepting a new baseline.
The selected source still has the 30 fps cap and retains the remaining original performance clauses above.

### Motion policy and passive render observations

The app honors Windows `UISettings::AnimationsEnabled` through its cached reduced-motion policy.
Prior captures and reports did not record that decision or count mark-layer render calls.
Identical Working endpoint images can alias an animation; they do not establish cadence or prove static rendering.

The private observation schema now carries the cached policy and passive mark-layer render-call count.
The counter observes existing renders without scheduling frames; it cannot prove physical display cadence.
The ordinary counter test uses actual headless draws.
The readiness test requires a Working sample to refuse reduced motion while permitting an idle control.
The initial counter accessor returns zero and the eligibility guard is absent, so both tests should reach runtime RED.
The corrected-config native run retains its frozen source and continues independently.

### Corrected configuration native audit

Native run `44829422916`, job `136610302731`, completed successfully on exact source `538ff3ca6`.
Artifact `14676985748` passed download size/hash checks, private decryption, member validation, and ZIP integrity checks.
Actual executable, PDB, profile, trace, and image hashes match the retained evidence.
All three traces report zero lost events and buffers; all nine exports succeeded.
All six endpoints preserve owned HWND 131480, PID 7676, and UI thread 820.
Root viewed captures 7, 12, and 17; each matches its after image by exact hash.
They show populated TUI and Core content, the authored Core marker, and the expected synthetic status.

The combined local override contains the required ports, skill, Core, and language settings.
The structured log contains no migration conflict or startup port-scan entry.
This qualifies fixture configuration and startup evidence, not a live poller-state probe.
Two jump-list warnings and one aggregate paint warning remain; the aggregate is not phase-specific.
Private Core Stop acknowledges shutdown and the owned serve exits successfully.
Retained holder and terminal PIDs are null; no independent OS holder-liveness check ran.

Recorded CPU deltas are 0.015625, zero, and 0.031250 seconds over the three short intervals.
Zero measured delta does not establish zero work.
Motion policy and render counts remain absent, so no cadence, savings, or cross-run comparison is qualified.
The curated receipt is `review-squad/windows-profile/native-538ff3ca6.json` beside this plan.
An independent raw-evidence review found no material recording or cleanup defect.

Ordinary motion-observation checkpoint `bc2b4f2d1` is running in CI as `44832512025`.
Its deliberate counter and readiness runtime failures remain pending.
The native branch remains frozen at `538ff3ca6`; the zero-counter stub is not promoted there.

### Source boundaries for the remaining measurements

Read-only review of `bc2b4f2d1` distinguishes construction from cell painting.
Primary TUI layers already replay stable paint; parked and mirrored pictures still use an uncached canvas.
Core's separate entity also lacks a cached mount, and `set_pane` currently relies on its next render.
Adding caching alone could leave pane appearance stale, so invalidation must be tested first.
Chat uses prepared entries and virtualization, but its Working pulse shares the window construction path.
The 16 ms timer polls independently; an unchanged tick does not always request a redraw.
These source facts establish boundaries, not CPU shares.

PR 147 at `6f268561e` supplies a cached parked/mirror picture proposal and headless tests.
Its appearance key includes theme; graphics placements deliberately prevent replay.
Native mark-driven replay, streaming-copy cost, and polling attribution remain separate requirements.

The next ordinary tools job reads Windows `UISettings::AnimationsEnabled` without changing it.
The read follows Microsoft's [WinRT bridge](https://devblogs.microsoft.com/oldnewthing/20180110-00/?p=97755)
and [UISettings contract](https://learn.microsoft.com/en-us/uwp/api/windows.ui.viewmanagement.uisettings.animationsenabled?view=winrt-26100).
That OS observation cannot replace the native app's own cached-policy report.

### Motion observation runtime RED and implementation

At `bc2b4f2d1`, run `44832512025` reached both intended runtime failures on both platforms.
Windows job `136620700680` passed 50 harness tests; only the reduced-motion refusal assertion failed.
Mac job `136620700536` passed 30 harness tests with the same single harness failure.
Both idle reduced-motion controls passed.
Both actual headless render tests failed with `render counter did not advance: 0 -> 0`.
Observer, Windows accessibility/inventory, scoped Clippy, tools, and retention checks passed.
The known Mac linker and `block` future-compatibility warnings remain.

The accessor now returns the existing counter, enabled for supported integration builds as well as tests.
The counter remains passive and does not notify or schedule frames.
The shared readiness validator now refuses reduced-motion Working samples at both endpoints.
The after check still retains partial phase metrics before refusing a changed state.
Ordinary GREEN and actual native policy/render observations remain pending.

### Disposable Windows animation preparation

The read-only tools job `136626281423` executed the WinRT getter successfully and reported `animations_enabled: false`.
This is a fresh ordinary runner, not proof of the earlier native runner's policy.
An isolated animation branch now exercises desktop preparation without app builds, daemons, or tracing.
Its initial setter does nothing; the fresh-process eligibility assertion should reach runtime RED on a disabled desktop.
Restoration always reads and verifies the saved Boolean, including after an assertion failure.
The saved file uses exclusive creation; an existing record refuses rather than replacing restoration authority.

The intended setter is Microsoft's `SystemParametersInfoW`, `SPI_SETCLIENTAREAANIMATION` (`0x1043`).
Its [documented contract](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-systemparametersinfow)
passes the Boolean as `pvParam`, with zero flags to avoid profile persistence and broadcast.
The actual WinRT property must become true; an API success return alone cannot qualify the preparation.
Only disposable Actions desktops enter this helper; app accessibility behavior remains unchanged.

At `8dd67cf65`, isolated run `44834668588`, job `136627923986`, reached the intended runtime RED.
Preparation and fresh-process reads both returned false; the eligibility assertion failed for that reason.
The always-run restoration control passed and preserved the original false preference.
No app, daemon, build, trace, or setting mutation entered that RED run.

The setter now calls the documented API with a pointer-sized Boolean and zero persistence/broadcast flags.
The saved record includes the native client-area value separately from the WinRT property.
Cleanup restores the native value and verifies both observations, avoiding assumptions that the getters always agree.
A repeated-preparation control checks exclusive refusal and the unchanged original-record hash.
Fresh processes verify eligibility and restoration independently.
Actual setter acceptance and property mapping remain pending.

At `8d0e39146`, isolated run `44835104225`, job `136629375437`, passed every animation contract.
Both original values were false; preparation made both true, including the fresh-process WinRT read.
Repeated preparation refused and preserved the exact original-record hash.
Restoration returned both values to false; a fresh process independently confirmed the original WinRT value.
This proves the setter on that disposable runner, not the earlier native runner's settings.

The helper now refuses immediately if its WinRT postcondition is false.
The native workflow prepares before building, saves preparation evidence, and always attempts recorded restoration after profiling.
The app continues to read normal OS preferences and reject reduced-motion Working samples.
The counter measures existing render calls, not physical display presents or exact CPU-interval frame rates.
The native branch has not advanced yet; Windows ordinary GREEN remains pending.

At `e076fbac161782d182eba5d689b9af1775b50ff8`, run `44834178683` passed all four ordinary jobs.
Windows passed 51 harness tests, eight observer tests, the passive counter test, ten accessibility controls, inventory, and Clippy.
Mac passed 31 harness tests, nine observer tests, the passive counter test, and Clippy.
Both regression fixes now have actual runtime RED and GREEN on both platforms.
Known Mac linker and future-compatibility warnings remain; no new warning appeared.

Source `dcadebfaf993392a3774f16f2fd4bfaba47921c4` contains no crate, dependency, lockfile, or Rust-toolchain changes from that GREEN source.
Its extra changes cover the independently tested desktop helper, workflow wiring, and evidence.
The final helper-postcondition check is running separately as `44835645896`.
Native execution still requires its own app-policy, counter, capture, trace-quality, and cleanup audit.

The final helper check passed in run `44835645896`, job `136631244180`, on exact `dcadebfaf`.
Preparation changed both false values to true; restoration returned both to false, confirmed from fresh processes.
The native branch then advanced normally from `538ff3ca6` to this exact source.
Native run `44835762627` is in progress with the real mixed workload and the normal OS motion policy.
No new native execution result, render cadence, or savings is claimed yet.
Next audit actual app policy and render counts, visible content, raw clocks, loss counters, identities, and restoration evidence.
All original performance clauses and Doctor Biz's Mac/speech holds remain in force.

### Core cell-paint regression checkpoint

The next headless regression uses the existing selected-Core fixture and real emulator output.
It requires an initial Core paint, actual bounds, and four advancing Working-mark frames.
Its recording worker accepts resize/focus commands; the test also requires the original populated frame and no view failure.
After warm-up, unchanged Core cells must not re-enter the painter during those frames.
The existing scoped paint probe now counts Core paints in tests only; native baseline source remains frozen.
The current uncached canvas should fail the zero-paint assertion; no cache implementation has been added yet.

The intended boundary caches cells only, preserving outer geometry, focus reporting, input, and accessibility every frame.
Content, cursor, selection, metrics, font, full paint theme, and veil changes must invalidate cached cells.
Add repaint and input/geometry controls before accepting the implementation.
Native baseline and post-change visible-content/counter evidence remain separate requirements.

Pinned `gpui-pre` 0.3.11 checks bounds, mask, text style, opacity, dirtiness, and refresh state before replay.
A hit skips the child's render, prepaint, and paint callbacks.
Paint replay preserves input handlers, but per-frame accessibility nodes are cleared separately and not replayed.
The Core interaction canvas must therefore remain live, as primary TUI interaction already does.

The proposed private child holds only cell-paint data and uses GPUI's existing cached-view boundary.
Replace it when the frame, highlights, pane appearance, metrics, font, ground, or actual painter appearance changes.
Keep its bounds aligned with the existing canvas and preserve clipping, veil, status, and padding order.
Retain outer `fit`, input registration, focus reporting, event listeners, and terminal accessibility nodes.
The painter's actual theme, font size, and line height must enter the key; current Core `Look` omits some inputs.
A read-only appearance snapshot avoids creating another authoritative configuration store.
Test input dispatch, resize/IME geometry, content, selection/cursor, and foreground-only theme changes before accepting caching.
The new regression started as run `44836845723` on `a0de2f5d9`; no runtime RED is claimed yet.

The first Mac execution stopped at the initial-paint precondition, before the regression assertion.
The projection fixture inserted a view without synchronizing the visible tab's pane state.
Its normal window therefore drew a connecting placeholder; existing projection tests drew the entity separately.
The fixture now runs the actual `sync_core_views` consumer before inserting its controlled view.
This is a fixture correction, not a cache fix or an accepted regression RED.

### Verified moving-mark native baseline

Native `dcadebfaf993392a3774f16f2fd4bfaba47921c4` passed run `44835762627`, job `136631645252`.
Artifact `14678249686` passed archive/ciphertext hashes, private decryption, member validation, ZIP integrity, and actual executable/PDB checks.
All three trace statistics report zero lost events and buffers; all nine exports succeeded.
All six endpoints preserve owned HWND 196998, PID 7160, and UI thread 7560.
Each endpoint reports `reduce_motion: false`.
Mark-layer render calls remain 36→36 while idle, advance 90→251 while Working, and remain 271→271 afterward.
Those observations surround recorder setup/stop; 161 divided by the CPU wall interval is not a valid frame rate.

Root viewed captures 6, 11, 12, and 16.
Both distinct Working images show populated TUI/Core content and the authored Core marker; each idle pair matches by hash.
Configuration remains correct without migration-conflict or startup port-scan logs.
Private Core Stop and owned-serve exit pass; no independent holder-liveness audit ran.
Desktop receipts confirm original false/false, prepared true/true, and restored false/false.

Actual Working ticks advance 282500000→465156250: 18.265625 CPU seconds over 6.1949839 recorded wall seconds.
That is 294.845399% relative to one core across all app threads, not a UI-thread share or portable hardware result.
The UI-only stack export has 320 hits, including overlapping Core paint 25, window render 59, tick 23, and DirectX draw 48.
Most all-thread leaf weight remains `Unknown!Unknown`; those UI counts cannot attribute the process total.
The trace includes `d3d10warp.dll` activity, but no explicit selected-adapter identity or GPU execution timing.
See Microsoft's [WARP guide](https://learn.microsoft.com/en-us/windows/win32/direct3darticles/directx-warp) for that module's role.

Two jump-list warnings and one aggregate paint warning remain; the aggregate is not phase-specific.
The curated receipt is `review-squad/windows-profile/native-dcadebfaf.json` beside this plan.
An independent raw audit found no material recording or cleanup defect.
No savings, cross-run comparison, exact cadence, mirror/chat, or continuous-update qualification is claimed.
The next code gate remains the corrected Core cell-paint runtime RED, before cache implementation.

At `454d16f22`, Mac job `136640846589` reached the intended cache assertion: 760 cell paints instead of zero.
The earlier population, bounds, advancing-mark, stable-frame, and no-failure checks passed.
The global fixture correction also broke the existing projection-only test's pre-layout assertion.
GPUI can draw after the synchronized setup update, even before an explicit test draw.
The shared projection fixture must retain its original undrawn semantics.
Initialization now occurs only inside the rendering test, with the paint probe active first.
It restores the controlled view after the normal synchronizer prepares pane state.
No cache implementation has been added; clean controls and the isolated intended RED remain pending.

The next native analysis also exports stacks for the exact GUI PID across all its threads.
It uses the existing verified xperf stack flags without the UI-thread filter.
This read-only export uses already-stopped traces; it starts no new recording and remains bounded to 32 MiB.
It may expose callers beneath unresolved worker leaves; readability or command success alone cannot qualify attribution.
The existing ledger keeps qualification false until actual outputs are reviewed.
No new native run is launched solely for this export; it accompanies the next justified recording.
