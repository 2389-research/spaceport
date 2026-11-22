# A35 unified command semantics — headless half (partial, Herdr 1 leg)

Session `H`, command `H1`. This is a partial attestation: A35's pass rule needs
a four-path table (GUI, palette, shortcut, CLI) that only a desktop session can
produce. The Herdr 1 leg stays `not_run`.

## What ran

| Field | Value |
| --- | --- |
| Revision | `708ca09f3a2d7a61c3b56e7d9bd3537ed9125c52` (`feat/wx3-session-h`, tree clean; only `target/` was built) |
| Commands | `just test` (the workspace suite, which contains `-p sheprd-gpui --bin spaceport cli::tests`); `cargo test --locked -p sheprd-commands` |
| Machine | macOS 26.7.1, arm64 |
| Runtime | none: catalog and CLI parsing only, no daemon is contacted |
| Logs | [`logs/H1-just-test.log`](logs/H1-just-test.log), [`logs/H3-osc52-commands.log`](logs/H3-osc52-commands.log) (third run in that file) |

## Result

`just test` ran both CI legs green: **5471 passed, 0 failed, 31 ignored** with
default features (73 suite summaries) and **5522 passed, 0 failed, 47 ignored**
with `--all-features` (75 suite summaries). 129 of those tests are
`cli::tests::*` in the `spaceport` binary, covering argument parsing, the
separator form, help and version precedence, and per-subcommand arity.

`cargo test --locked -p sheprd-commands` passed **40 passed, 0 failed**: the
catalog's names, default shortcuts per platform, and the daemon request each
command becomes against a snapshot.

## What this record does not prove

The pass rule is "the same command reaches the same typed request from all four
paths, and a refusal (bad target, ambiguous name) produces the same exit code
and message from CLI and dialog". This session attests only the catalog and the
CLI path; the palette, the shortcut and the dialog were never opened, and no
live CLI (`P1·P1-a`) was run against a daemon. The leg stays `not_run` until the
`M·M8`/`W·W1` cross-check records the four-path table.
