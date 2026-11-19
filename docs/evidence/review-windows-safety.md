# Windows pipe trust and agent cleanup

Review children `q7h6` and `rder`, under `9s44`. Source review: 2026-11-17.

The original November 17 scope covered fixes and tests without publication.
Doctor Biz authorized the public draft checkpoint on November 18, as recorded in the integration evidence.

## Pipe connection

The Herdr connector opened its predictable pipe without checking its owner or limiting server impersonation.
Both initial attachment and startup polling now use the existing `Stream::connect_owned` primitive.
That primitive checks the connected handle's owner SID against the client's user SID before returning.
It opens the handle with identification-only security quality of service.

The address remains Herdr's exact legacy mapping: `\\.\pipe\` plus the lossy socket path.
The pinned interprocess source documents and implements this namespaced mapping.
Core and chat retain their explicit full-pipe addresses.
Permission errors still refuse attachment without starting another daemon.

Windows tests use real private pipes and cover these contracts:

- The current user's pipe exchanges bytes through Herdr's path mapping, including spaces and Unicode.
- Generic read/write and Herdr's OWNER RIGHTS descriptors remain usable.
- Denied write access remains a permission error, with automatic startup enabled or disabled.
- An identification-only server cannot open the client's private fixture file while impersonating it.
  The safe interprocess guard always restores the server thread's identity.

The negative access fixture does not prove rejection of another account's owner SID.
No independent account or privileged owner reassignment was available for that case.
The owner comparison reuses the existing Core/chat primitive and receives source review.

## Agent ownership

Windows previously stopped only the direct child, often the npm shim's `cmd.exe` process.
Windows ACP startup now uses the pinned `process-wrap` 10.0.1 std Job Object wrapper.
Default features remain disabled. Only std, creation flags and Job Objects are enabled.
No runtime or new unsafe production code is added.

The wrapper starts the child suspended, assigns its private job, then resumes the child.
Descendants therefore join the job before executing agent code.
Assignment and resume failures terminate and reap the suspended child before returning an error.
The explicit creation-flags wrapper preserves `CREATE_NO_WINDOW` from `sheprd_client::new_command`.

The Windows child owner retains the job and extracted stdin, stdout and stderr handles.
Stopping the agent terminates the job, even after the direct shim has exited successfully.
Dropping the last child owner also terminates and reaps the direct child.
The existing stderr drain and Unix TERM/KILL group cleanup remain in place.

Agent waiting still polls without holding the process mutex across sleeps.
It preserves the direct agent's cached exit status and allows a concurrent stop.
The child owner avoids the dependency's blocking completion-port wait after polling consumes notifications.
Tree termination uses `start_kill`; direct-child reaping uses repeated `try_wait` calls.

Windows regressions launch an isolated `.cmd` shim, a Rust agent fixture, and its tool child.
They hold exact descendant process handles and use bounded waits and cleanup guards.
They cover concurrent wait/stop, an exited shim's descendants, last-owner cleanup, repeated stop, and exit status.
Separate coverage exercises stdin, stdout, stderr, and the explicit child environment.
The only new unsafe calls are test-only handle acquisition, bounded waiting, and exact-handle cleanup.

Source/API references inspected:

- [process-wrap 10.0.1 std Job Object source](https://docs.rs/crate/process-wrap/10.0.1/source/src/std/job_object.rs)
- [process-wrap 10.0.1 Windows job and suspension source](https://docs.rs/crate/process-wrap/10.0.1/source/src/windows.rs)
- [interprocess 2.4.7 Windows namespaced mapping](https://docs.rs/crate/interprocess/2.4.7/source/src/os/windows/local_socket/name_type.rs)

## Verification

- `cargo test -p sheprd-acp process::tests --locked` passed: two Unix process regressions.
- `cargo check -p sheprd-acp --tests --target x86_64-pc-windows-gnu --locked` passed.
- `cargo check -p sheprd-gpui --bin spaceport --tests --target x86_64-pc-windows-gnu --locked` passed.
  It reported one unrelated dead-code warning in the concurrent endpoint fixture changes.
  The endpoint owner received the exact method names for correction.
- `cargo clippy -p sheprd-acp --tests --target x86_64-pc-windows-gnu --locked -- -D warnings` passed.
- Scoped rustfmt checks and `git diff --check` passed.
- `bash scripts/check-file-size.sh` passed.
- Root independently reviewed the diff, callers, dependency job handling, and exact-handle tests.
  That review found no material issues.
- Fresh-eyes review bounded the stdio test read and avoided waiting after a destructor's termination error.

Cargo ran under the shared task lock, Rust 1.99.0, and the existing task target directory.
Logs live under `/tmp/spaceport-windows-safety/`.
The first combined Windows clippy run stopped at another task's nested `if` in the control socket.
That finding was sent to its implementation owner.

At the original local checkpoint, Windows runtime tests had not run on this Mac.
Cross-compilation does not prove native behavior.
The Windows regressions were written before the fixes, but native failing-baseline proof remains unavailable.
The restored Windows CI jobs must run the pipe, process, and retained workspace tests before shipping.
Publication remained unauthorized at that original checkpoint.

## Native Windows agent qualification

CI tested source `208fa3929748d89cc815a79eba1340745b6a7a15` on native Windows x64 and ARM.
Both ACP unit suites report 419 passed and one private fixture entry ignored directly.
Parent tests explicitly launch that entry.
All four process regressions pass on both architectures:

- Stopping a Windows shim ends its agent and tool, including concurrent wait/stop.
- Stopping an exited shim still ends its tool and preserves the direct exit status.
- Dropping the last owner ends its process tree.
- Stdio, explicit environment, repeated stop, and cached status remain intact.

Verified [x64 job logs](https://github.com/2389-research/spaceport/actions/runs/44431160334/job/135248813591)
and [ARM job logs](https://github.com/2389-research/spaceport/actions/runs/44431160334/job/135248813776).
Those jobs later failed unrelated SSH parser fixtures; this does not qualify the whole job.
Kata `rder` closed after a fresh native-result read and issue-status readback.
The closure event is `9757` (`01M7RNCXC99A8HFWN28N4Q1HMF`).
Pipe-specific GUI tests and the foreign-owner limitation remain separate qualification work for `q7h6`.

## Native pipe qualification

Source `aaff481f9` passes all three daemon pipe regressions on native x64 and ARM, with both feature configurations.
They qualify same-owner byte exchange, denied-write refusal without startup, and identification-only impersonation limits.
They do not exercise a readable pipe owned by a different SID.

A new private fixture creates a pipe with current-user read/write access and the token's default owner.
The unchecked connector exchanges five fixture bytes and the test reads the actual opened pipe's owner.
The checked connector must then refuse a different owner, with automatic startup enabled and disabled.
If the default owner equals the current user, the fixture instead verifies acceptance and reports mismatch qualification unavailable.
It grants no new account access, changes no token privileges, and prints no SIDs.
The native job runs this exact test with uncaptured branch markers on both Windows architectures.
Only actual mismatched-owner refusal markers qualify the negative security boundary.
The issue remains open until those native results arrive.

The scheduling regression fails before the workflow step exists, then passes with its actual shell invocation.
Both scheduling cases, 11 CI policy cases, six release-transition cases, and three security cases pass locally.
Actionlint and offline Zizmor pass with their existing configured exclusions.
Windows GNU Clippy compiles the test; it cannot execute it on this Mac.
Independent review approves the private handles, replacement listener, bounded probe I/O, and explicit qualification limits.
