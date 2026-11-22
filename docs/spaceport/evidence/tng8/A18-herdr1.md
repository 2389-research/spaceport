# A18 unknown versus idle — live half (partial, Herdr 1 leg)

Session `P1`, command `P1-a`. This is a partial attestation: A18's artifact asks
for a `spaceport status --json` capture per program, and this session ran the
live client fixture instead of capturing the CLI's JSON. The Herdr 1 leg stays
`not_run`.

## What ran

| Field | Value |
| --- | --- |
| Revision | `708ca09f3a2d7a61c3b56e7d9bd3537ed9125c52` (`feat/wx3-session-h`, tree clean) |
| Runtime | `/Users/clint/.local/bin/herdr`, Herdr `0.9.5`, 22,536,176 bytes, sha256 `fb77c2232a05a8035421382c894392f4174e4e38a18e48af3dd0290157c3a7e2`, selected explicitly through `HERDR_TEST_BINARY` |
| Command | `HERDR_TEST_TMPDIR=/tmp just test-live /Users/clint/.local/bin/herdr` |
| Machine | macOS 26.7.1, arm64 |
| Log | [`logs/P1a-live-client.log`](logs/P1a-live-client.log) |

## Result

The suite passed **4 passed, 0 failed**, 2 non-ignored tests filtered out by
`--ignored`. `daemon_completion_status_live` is the fixture A18 names, and it
passed against a live daemon whose reported states came back distinct and in
sequence:

```
live status: wire=working status=Working seq=1
live status: wire=blocked status=Blocked seq=2
live status: wire=idle status=Idle seq=3
test daemon_completion_status_live ... ok
```

## What this record does not prove

A18's pass rule is that three differently-behaving programs — never reports,
reports done but stays alive, becomes unreachable — each map to a different
reported state, and that no unreported program renders as idle. The log above
shows one program moving through three reported states, which is not the same
claim, and the `spaceport status --json` capture per program that A18's artifact
column requires was not taken. The leg stays `not_run` until a session records
those three captures.
