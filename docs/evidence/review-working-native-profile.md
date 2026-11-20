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
