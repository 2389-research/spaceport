# A23 explicit termination — live half (partial, Herdr 1 leg)

Session `P1`, command `P1-a`. This is a partial attestation: A23's Herdr 1 half
needs the native terminate action with duplicate labels, and its Core half needs
`P2·P2-a`. The Herdr 1 leg stays `not_run`.

## What ran

| Field | Value |
| --- | --- |
| Revision | `708ca09f3a2d7a61c3b56e7d9bd3537ed9125c52` (`feat/wx3-session-h`, tree clean) |
| Runtime | `/Users/clint/.local/bin/herdr`, Herdr `0.9.5`, 22,536,176 bytes, sha256 `fb77c2232a05a8035421382c894392f4174e4e38a18e48af3dd0290157c3a7e2`, selected explicitly through `HERDR_TEST_BINARY` |
| Command | `HERDR_TEST_TMPDIR=/tmp just test-live /Users/clint/.local/bin/herdr` |
| Machine | macOS 26.7.1, arm64 |
| Log | [`logs/P1a-live-client.log`](logs/P1a-live-client.log) |

## Result

The suite passed **4 passed, 0 failed** against an isolated daemon with several
workspaces and tabs, which is the shape A23's qualified-target question needs:

```
detach verified: daemon alive; reconnected to same boot 54824-1795324301697820000 with 2 workspaces / 3 tabs
test stable_endpoint_live ... ok
```

## Partial claims this record re-attests

The leg already cites `evidence/t05-explicit-termination.md`, which is a **Core**
record (duplicate labels, `terminal.kill` by exact address) that the leg schema
kept on the leg it was recorded on. This session did not retest that Core path
and does not re-attribute it: no Core binary was used. What it adds is the
Herdr 1 side's live precondition — one daemon holding several tabs and
workspaces, addressable per tab — which is what a duplicate-label terminate
action would act on.

## What this record does not prove

No terminal was terminated here, no duplicate display labels existed, and no
before/after PID comparison was taken. The native terminate action A23 requires
did not run, so the leg stays `not_run`.
