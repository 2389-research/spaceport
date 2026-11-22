# A06 workspace document CRUD — Herdr 1 side (partial, Herdr 1 leg)

Session `P1`, command `P1-a`. This is a partial attestation: A06's Herdr 1 half
also needs the native workspace controls in `M·M6`, and its Core half needs
`P2·P2-c`. The Herdr 1 leg stays `not_run`.

## What ran

| Field | Value |
| --- | --- |
| Revision | `708ca09f3a2d7a61c3b56e7d9bd3537ed9125c52` (`feat/wx3-session-h`, tree clean) |
| Runtime | `/Users/clint/.local/bin/herdr`, Herdr `0.9.5`, 22,536,176 bytes, sha256 `fb77c2232a05a8035421382c894392f4174e4e38a18e48af3dd0290157c3a7e2`, selected explicitly through `HERDR_TEST_BINARY` |
| Command | `HERDR_TEST_TMPDIR=/tmp just test-live /Users/clint/.local/bin/herdr` |
| Machine | macOS 26.7.1, arm64 |
| Log | [`logs/P1a-live-client.log`](logs/P1a-live-client.log) |

## Result

The suite passed **4 passed, 0 failed**. The fixture that speaks to A06's Herdr 1
side is `named_session_create_and_delete_cli_contract`, which creates a named
session through the CLI, talks to its stable endpoint, and deletes it through the
same CLI, all inside one sandbox (`/tmp/hd627-*`) with a private daemon pid:

```
isolated daemon pid=54825 dir=/tmp/hd627-276f3700-2
focus verified: workspace=w1 tab=w1:t2
test named_session_create_and_delete_cli_contract ... ok
```

## What this record does not prove

That is session lifecycle through the CLI, not the presentation-workspace
document A06 is about: no workspace was created, renamed, archived or deleted,
and no before/after `spaceport processes` PID comparison was taken, so the
"daemon keeps the pane process" clause is unattested. The native workspace
controls are part of `M·M6`, which has not run. The leg stays `not_run`.
