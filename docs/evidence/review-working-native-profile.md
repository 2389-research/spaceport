<!-- ABOUTME: Defines an isolated release profile of a real Herdr TUI working session. -->
<!-- ABOUTME: Separates generated workload observations, sampling results, and unqualified providers or Core. -->
# Working native session profile implementation plan

**Goal:** Profile the release GUI with two populated terminal groups and controlled output and agent status.

**Architecture:** Launch only test-owned processes inside the existing Sandbox.
Drive the private Herdr API and Spaceport control CLI.
Use a gated observer only for state the public CLI cannot report.
Sample the owned GUI process; retain the raw stacks, workload observations, CPU readings, and native captures.

**Tech stack:** Existing Rust, GPUI, serde, Sandbox, Herdr TUI, macOS `sample`, and macOS `ps`.

## Authority and limits

Root owns every Cargo build, daemon launch, native interaction, and profile execution.
This agent owns the harness and its focused observation code.
No redraw optimization or scheduling change belongs in this step.
No new dependency, runtime, crate, unsafe code, or public command belongs in this step.
No provider credential, personal content, or Core process enters the fixture.
The controlled Working status is synthetic and must be labeled synthetic in every report.
The existing dense rendering budget does not substitute for this working-session profile.
No commit or publication is authorized.

## Files and boundaries

- `tests/working_profile.rs` owns the opt-in native test and its exact child processes.
- `tests/working_profile/` may hold focused measurement and fixture helpers.
- `src/working_profile.rs` owns the integration-test-only observation handshake.
- A shared observation schema may live under `src/working_profile/`.
- `Cargo.toml` registers this test with `required-features = ["integration-test"]` after Root releases its build hold.
- `main.rs` declares the module only on macOS with `integration-test`.
- `app.rs` starts observation only for a normal window with the explicit profile environment variable.
- `group_terminals.rs` may expose one gated, borrowed observation accessor; Root reserved this narrow footprint.
- `presentation.rs` may expose the approved gated borrowed frame/image pair, without advancing presentation.

Root released the startup wire gate after the native Settings build and the focus baseline build.
The observer never calls `Presentation::picture`, steers a connection, or requests a redraw.
Latest projected surfaces must not be labeled presented surfaces.
Root approved an exact borrowed presented-frame/image accessor under the same macOS/integration-test gate.

## Test and observation contracts

`SPACEPORT_WORKING_PROFILE_DIR` is a new, integration-test-only harness contract.
Its value names the fixture's existing absolute private directory.
Normal launches without it create no profile observer.
The harness starts Spaceport with its explicit private client socket.
It supplies `HERDR_TEST_BINARY` explicitly and requires `HERDR_TEST_TMPDIR=/tmp`.
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

- [ ] Qualify the runner's actual sampling tools and profile support.
- [ ] Define Windows transport, native observation, and measurement edges while preserving the Mac path.
- [ ] Add tests first for real endpoint bounds, native readiness, and CPU-clock semantics.
- [ ] Run the normal release app with idle, controlled Working, and final idle phases.
- [ ] Add bounded real output before claiming patch or frame-copy coverage.
- [ ] Inspect native captures, actual samples, and CPU/frame data before changing redraw behavior.
- [ ] Verify the original Core, mirror, chat, copying, and polling clauses separately before closing gg9v.
