# A03 installed runtime startup — live attach half (partial, Herdr 1 leg)

Sessions `H` (commands `H7`, `H6`) and `P1` (command `P1-a`). This is a partial
attestation: A03's pass rule also needs the Linux shutdown fixture (`L·L2`) and
the macOS quit/restart step (`M·M6`), neither of which has run. The Herdr 1 leg
stays `not_run`.

## What ran

| Field | Value |
| --- | --- |
| Revision | `708ca09f3a2d7a61c3b56e7d9bd3537ed9125c52` (`feat/wx3-session-h`, tree clean) |
| Runtime | `/Users/clint/.local/bin/herdr`, Herdr `0.9.5`, 22,536,176 bytes, sha256 `fb77c2232a05a8035421382c894392f4174e4e38a18e48af3dd0290157c3a7e2`, selected explicitly through `HERDR_TEST_BINARY` |
| Commands | `cargo test --locked -p sheprd-gpui --bin spaceport daemon::tests`; `... endpoint::lifecycle_tests`; `HERDR_TEST_TMPDIR=/tmp just test-live /Users/clint/.local/bin/herdr` |
| Machine | macOS 26.7.1, arm64 |
| Logs | [`logs/H4-H7-gpui-filters.log`](logs/H4-H7-gpui-filters.log), [`logs/P1a-live-client.log`](logs/P1a-live-client.log) |

## Result

The launch-policy half passed: `daemon::tests` **34 passed, 0 failed**, including
`daemon::tests::attach_only_and_cancelled_do_not_launch ... ok`,
`daemon::tests::explicit_and_remote_targets_never_start_local_daemon ... ok` and
`daemon::tests::existing_daemon_does_not_launch ... ok`. The lifecycle policy
half passed too: `endpoint::lifecycle_tests` **172 passed, 0 failed**.

The live half passed: `just test-live` ran **4 passed, 0 failed** against the
explicitly selected Herdr 1 binary, each fixture in its own sandbox
(`/tmp/hd627-*`) with its own daemon pid. It shows a real attach, input and
output, and a detach that leaves the daemon up:

```
semantic TextCommit + Enter produced shell output: HERDR_LIVE_54823_OK
resize verified: 100x30 surface
detach verified: daemon alive; reconnected to same boot 54824-1795324301697820000 with 2 workspaces / 3 tabs
```

## What this record does not prove

A03's pass rule also requires that "the shutdown fixture's fake `herdr` on `PATH`
leaves no `started` file and the window holds 'Herdr stopped' for the full retry
ceiling before the start button reconnects" (`L·L2`) and the quit/restart
survival step (`M·M6`). Neither ran here: no window was opened and no fake
`herdr` was staged. The leg stays `not_run`.
