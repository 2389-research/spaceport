<!-- ABOUTME: Records the listening-port scan cost fix and worker lifetime contract. -->
<!-- ABOUTME: Keeps process-count evidence separate from pending native timing. -->

# Listening-port scan cost plan and evidence

Date: 2026-11-18. Scope: the listening-port part of `gfde`.
Root owns the other painting and polling costs.
Doctor Biz authorized this work under the existing all-findings goal.
No commit, push, or issue closure is authorized for this owner.

**Goal:** Reduce repeated process discovery and share identical host scans across application windows.

**Architecture:** One application registry keeps weak host entries.
Each window leases a UI-owned scan source with one existing bounded worker receiver.
Each window keeps an immutable `Arc` reading, so another consumer cannot consume its update.
The final source lease drops the worker and signals Stop without joining on the UI thread.

**Constraints:** Preserve exact Host and Daemon identity filtering.
Keep the 16-host admission bound per window.
Keep process, output, listener, port, timeout, and retry bounds.
Keep local, SSH, and WSL shell ownership and per-window tunnel actions.
Keep the last good ports after a scan error.
Do not add a runtime, dependency, transport abstraction, or external setting.
Native timing remains pending at Doctor Biz's request.

## Plan

- [x] Read current worker, parser, shell, Stop, daemon identity, and caller code.
- [x] Inspect the original finding and existing performance evidence.
- [x] Verify actual macOS ps flags against an owned process with a cleared environment.
- [x] Prove the shell command starts 128 ps processes for 128 private listener owners.
- [x] Run the compiled batch and two-consumer regressions before changing their production paths.
- [x] Batch the bounded fallback PID list into one ps invocation.
- [x] Reduce Linux environment parsing without losing NUL-separated values or socket ownership.
- [x] Add the weak application registry and immutable per-consumer readings.
- [x] Exercise two consumers, host removal, final lease release, errors, reconnect, and separate hosts.
- [x] Verify host bounds, daemon identity, and window-owned tunnel consumers.
- [x] Run owned formatting and quick structure/localization checks.
- [x] Obtain independent lifecycle review from root.
- [x] Record exact final results and remaining platform/native limits.

## Files and ownership

`listening_ports/scan.rs` owns the shell command and text parser.
Any shell helper extraction stays below that module.
`listening_ports.rs` owns per-window readings and the existing worker loop.
A focused child module may own the weak scan registry and UI source leases.
Topic tests live under `listening_ports/tests/`.
Root assigned the narrow `window/monitors.rs` and `window.rs` poll-context hunks to this owner.
Existing chip exports, tunnel code, and other owners' changes must remain intact.

## Baseline evidence

The actual current shell command ran in an env-cleared private fixture.
Its private ss and ps executable producers emitted 128 listeners and 128 owners.
The command invoked ps 128 times; the regression expects one bounded invocation.
The assertion failed for that count.
Evidence: `/tmp/spaceport-qnz9/gfde-scan-pipeline-runtime-red.json`.
The private shell output remains under the root recorded in that JSON.

The compiled topic regressions failed against the unchanged scanner.
Artifact: `/tmp/spaceport-9s44-gpui.ZcCcNN/spaceport-tests`.
Filter: `listening_ports::tests::scan_cost`.
Log: `/tmp/spaceport-qnz9/gfde-scan-cost-runtime-red.log`.
Both parent cases failed; their two private child entry points remained explicitly ignored.
The batch case parsed all 128 owners, then observed 128 ps calls instead of one.
Both consumer models received data, then observed two host scans instead of one.
The parent cases invoke their exact ignored child entry points themselves.
The fixture child owns its private HOME and executable path.
It never uses personal process environments as data.

## Capability checks

The installed macOS ps manual documents `-E`, repeated `-w`, `-o`, and comma-separated `-p` PIDs.
An owned private Rust child exposes its workspace and a socket path containing spaces through those flags.
Log: `/tmp/spaceport-ps-owned-20261118-6145-r2j4eu/result`.
The same probe did not expose those variables from an owned system sleep binary.
No cause for that difference is claimed here.

The exact production flags also ran against an env-cleared owned Rust child:
`ps eww -o pid= -o command= -p <owned PID>`.
The output held exactly that PID, its workspace, and its socket path containing spaces.
The current command then used real lsof restricted to that same owned PID.
It emitted the complete workspace and socket metadata.
Evidence: `/tmp/spaceport-scan-owned.UpBwIo/ps.out` and `scan.out`.
This throwaway probe killed and reaped only its exact child.

GNU awk documents NUL record separators for Linux process environments.
The installed macOS awk did not split a private NUL-separated probe into two records.
An implementation must test the capability or retain a correct fallback.
Source: [GNU awk record splitting](https://www.gnu.org/software/gawk/manual/gawk.html).
The Linux ps manual separately documents its environment-output option.
Source: [Linux ps](https://man7.org/linux/man-pages/man1/ps.1.html).

## Current status

The command now collects at most 1,024 listener records and 128 distinct owner PIDs.
It passes the bounded fallback PID list to one ps process.
The actual private command produces 128 owners with one ps invocation.
Log: `/tmp/spaceport-qnz9/gfde-scan-pipeline-runtime-green.json`.
The compiled checks below also pass against the integrated source.

On Linux, an awk capability probe selects NUL-separated environment parsing.
A capable awk reads all readable selected process environments in one invocation.
If awk lacks that feature, the command retains per-PID tr and awk processes.
That bounded compatibility fallback remains a process cost; it is not claimed eliminated.
macOS uses the single bounded ps invocation.
Linux-specific execution remains for Linux CI; macOS cannot qualify that platform branch.

The app registry holds weak exact-Host entries.
Each leased source owns one existing worker and its sole bounded result receiver.
Windows retain immutable readings; they do not compete for receiver events.
Dropping the final source lease signals the existing Stop without joining the worker.
The worker still owns shell cleanup and the existing scan/retry schedule.
The 16-host bound remains per window; the registry adds no application-wide admission cap.
Daemon filtering, watched-host selection, origins, and per-window tunnel ownership remain unchanged.

New private fixtures cover consumer departure, later updates, final Stop release, and separate hosts.
They also cover the 16-host bound and real-shell failure/reopen with last-good data.
An env-cleared owned child checks the exact ps flags and a socket path containing spaces.
Existing PATH and real-socket tests now isolate their producers and queried child.
Native timing remains pending at Doctor Biz's request.

## Qualified local checkpoint

Immutable artifact: `/tmp/spaceport-9s44-gpui.xD8o8D/spaceport-tests`.

```sh
/tmp/spaceport-9s44-gpui.xD8o8D/spaceport-tests listening_ports:: --test-threads=1
/tmp/spaceport-9s44-gpui.xD8o8D/spaceport-tests control::server::hosts::tests::ports --test-threads=1
```

The broad scanner scope passes 56 tests with no failure.
Its eight ignored entry points are private subprocess fixtures invoked by their nonignored parent tests.
Six scanner entry points and two tunnel entry points use that pattern.
Log: `/tmp/spaceport-qnz9/gfde-listening-ports-green.log`.
All five control-port consumer tests also pass, with no failure or ignore.
Log: `/tmp/spaceport-qnz9/gfde-port-consumers-green.log`.

The scanner scope proves actual process batching and exact-host sharing.
It also proves later updates after consumer departure, final worker release, and shell reopen after failure.
Two private windows each admit 16 distinct hosts, with no global cap.
Existing exact daemon, link, stale tunnel, cancellation, and process cleanup consumers pass.
The owned real listener case passes with the exact ps flags and a socket path containing spaces.
The repeated private-shell PATH cases pass without directory accumulation.

Root independently reviewed the final registry, immutable readings, worker loop, shell command, and green evidence.
That lifecycle and data-integrity review found no material issue.
Owned rustfmt checks, shell syntax, and git diff checks pass.
Source size, sorted regions, and localization checks pass; localization reports 2,174 messages across 114 files.

Fresh aggregate host and cross-platform lint remain part of the shared final qualification.
Linux execution must qualify its proc/NUL branch in Linux CI.
Windows native WSL execution remains unrun locally; unsupported Windows-local scanning remains explicit.
Release-profile timing remains pending.
This checkpoint does not close the full gfde finding.
