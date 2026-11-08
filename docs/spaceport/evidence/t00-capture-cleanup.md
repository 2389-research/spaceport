# T00 private Core capture cleanup

The T00 `scripts/capture-core-contract.py` fixture used a private `spcore-*` state root and started `herdr-core serve`, but on macOS the pinned Core moves a normal `serve` start into launchd (`crates/herdr-core/src/platform/macos_launchd.rs`). The Python script reaped only its short-lived launcher and removed the temporary socket directory. Three `--hc-launchd` processes remained after earlier captures. Their exact PIDs, test binary path, `spaceport-t00` state roots and deleted `spcore-*` directories were checked with `ps` and `lsof`; no personal Core or terminal holder matched. The three verified fixture PIDs (6372, 18329, 30474) then exited after targeted SIGTERM. No process-name or group kill was used. The old private sockets were already gone, so Core's `server stop` could not reach those orphans.

The fixture now passes the pinned Core's immediate-launcher `HC_LAUNCHD_DIRECT` guard with its Python PID, checks that the direct child remains alive through `rpc.discover`, calls `server stop` against its private `HC_SOCKET`, and waits for that child. Failed or timed-out stop is reported after the child is reaped. This is a test-process cleanup rule; the app's normal installed-Core startup is unchanged.

- Before the fix, the three exact private processes had parent PID 1 and `spcore-*` state roots that the script had removed.
- After the fix, two fresh private captures each reported 32 methods from Core 0.1.18; a process scan found no new `spcore-*` or `--hc-launchd` process from the test binary.
- `git diff --check` and `just check-file-size` passed.

The earlier T00 schema observation remains valid. This correction prevents the capture tool from leaving a Core process behind or silently accepting a detached one; it does not change application behavior or satisfy desktop runtime acceptance.
