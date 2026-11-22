# Tunnel ownership after parent exit

Review child `zdrf` belongs to the review squad task `9s44`.
The original checkpoint authorized fixes and validation without publication or Kata writes.
Doctor Biz later authorized the public draft checkpoint and closing verified issues; see [integration evidence](review-main-integration.md).
The original `kata-zdrf` worktree remains unchanged.

## Cause and change

Click tunnels and saved-host forwards spawned plain SSH children.
Their stop and drop paths could not run after a parent crash.
An authenticated loopback listener could therefore remain open.
Saved-host forwards also left their private control directories behind.

`ssh::owned_process` now owns these private children.
Unix children share the existing mux pipe watchdog.
Closing the ownership pipe ends only its direct child.
The watchdog removes only its private control socket and directory.
It also removes OpenSSH's temporary `c.<random>` control sockets before removing that directory.
Normal stop and drop close this pipe without waiting on the UI thread.
Worker threads reap the exact owned watchdog children.

Executable lookup uses the command's PATH and working directory.
The existing locked `which` package resolves names before the watchdog starts.
Missing bare and absolute executables retain an I/O `NotFound` error.
Shell execution failures after lookup still use the shell's exit status.
The helper preserves argument boundaries and explicit environment changes.
Callers build private foreground SSH commands without clearing the inherited environment.
Both forwarding commands override configured `ForkAfterAuthentication` when the client knows it.
`ssh::foreground` checks once with `ssh -G`; OpenSSH before 8.7 lacks the option.
Such a client's config cannot set it, so omitting it there keeps the child in the foreground.
This prevents an SSH child from detaching beyond its watchdog.
Existing unverified-host forwarding restrictions remain active.

Windows click tunnels use an anonymous, noninherited kill-on-close job.
The safe `win32job` wrapper supplies that policy.
`process-wrap` suspends the child before the custom assignment hook runs.
Its standard job wrapper assigns its own job before resuming that child.
Assignment failures kill and reap the exact suspended child.
The nested job is necessary because the standard wrapper lacks kill-on-close support.
The client adds no asynchronous runtime or native unsafe code.
Windows saved-host `PortForward` remains explicitly unsupported.

## Source verification

- Inspected `process-wrap 10.0.1` registry sources: `generic_wrap.rs`, `std/job_object.rs`, and `windows.rs`.
- Confirmed ordered pre-spawn, spawn, post-spawn, and child-wrapping hooks.
- Confirmed the standard job wrapper creates suspended children and cleans up assignment or resume failures.
- Inspected `win32job 2.1.0`: `job.rs` and `limits.rs`.
- Confirmed anonymous job creation uses noninheritable handles and closes them on drop.
- Confirmed [kill-on-close limits](https://docs.rs/win32job/2.1.0/win32job/struct.ExtendedLimitInfo.html) terminate associated processes.

## Verification

All Cargo checks use Rust 1.99.0, `--locked`, and the shared Cargo lock.
The target directory is the existing `upstream-fixes/target` directory.

- Baseline forward crash regression failed with the expected surviving listener process.
- A startup crash regression exposed leftover temporary control sockets before cleanup was extended.
- An isolated click-tunnel module regression also failed with the former plain-child spawn behavior.
- The fixed forward tests passed: 17 tests, plus two subprocess fixture entry points.
- Click-tunnel and existing listening-port tests passed: 51 tests, plus two subprocess fixture entry points.
- The new missing-bare-executable regression failed before executable resolution was added.
- Windows GNU `cargo check -p sheprd-client --target x86_64-pc-windows-gnu --locked` passed.
- The client SSH tests passed: 69 tests, with one existing live-SSH test unrun.
- Client Clippy passed with all targets, all features, and `-D warnings`.
- Windows GNU client Clippy passed with all targets, all features, and `-D warnings`.
- The scoped Rust formatting check, `git diff --check`, file-size gate, and sorted-list gate passed.

The process tests bind real loopback listeners and use real files and child processes.
They cover forced parent exit, normal stop, drop, and cancellation during readiness or spawn.
They verify control directory removal and that unrelated children or listeners remain available.
Fixtures never use personal SSH hosts, keys, agents, or daemons.
Ignored fixture entry points run explicitly inside their owning regression tests.
They are not unexecuted acceptance tests.

## Limits and pending checks

Native Windows and Linux qualification was pending at the original local checkpoint.
The Windows GNU check alone proved compilation, not Windows job behavior.
APFS refused the Linux-only invalid UTF-8 filename fixture with `EILSEQ`.
No live SSH end-to-end connection was attempted.
Local process fixtures prove ownership and cleanup, not remote authentication or forwarding.
The full workspace gates and platform jobs remain CI work.
The GPUI link emitted the existing oversized unwind-section warning.
Cargo also reported the existing `block 0.1.6` future-incompatibility notice.

## Native platform qualification

CI source `620a31af7992f5020cf33ef5042132d6647f13fc` passes all three click-tunnel process regressions on Windows x64 and ARM.
They cover forced owner exit, drop/cancellation, and cancellation during readiness with real listeners.
Parent tests explicitly launch the two private fixture entries ignored directly by the harness.
See [x64 results](https://github.com/2389-research/spaceport/actions/runs/44441388353/job/135283127710)
and [ARM results](https://github.com/2389-research/spaceport/actions/runs/44441388353/job/135283127650).
Those jobs fail unrelated GUI fixtures; they do not qualify the whole Windows workspace.

The [Linux x64 default job](https://github.com/2389-research/spaceport/actions/runs/44441388353/job/135283127800) passes forward crash cleanup and all three click-tunnel cases.
It also passes the real non-UTF-8 executable-path fixture.
Kata `zdrf` closed after these results and a fresh issue-status readback.
Its closure event is `9760` (`01M7RSVHMGEAA4Z794F7DM7BB1`).
Windows saved-host forwards remain explicitly unsupported.
