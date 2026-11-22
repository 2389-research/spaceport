# A35 unified command semantics — headless half (partial, Herdr 1 leg)

Session `H`, command `H1`. This is a partial attestation: A35's pass rule needs
a four-path table (GUI, palette, shortcut, CLI) that only a desktop session can
produce. The Herdr 1 leg stays `not_run`.

## What ran

| Field | Value |
| --- | --- |
| Revision | `708ca09f3a2d7a61c3b56e7d9bd3537ed9125c52` (`feat/wx3-session-h`, tree clean; only `target/` was built) |
| Commands | `just test` (the workspace suite, which contains `-p sheprd-gpui --bin spaceport cli::tests`); `cargo test --locked -p sheprd-commands`; `HERDR_TEST_TMPDIR=/tmp just test-live /Users/clint/.local/bin/herdr` |
| Machine | macOS 26.7.1, arm64 |
| Runtime | none for the catalog half; `/Users/clint/.local/bin/herdr` (Herdr `0.9.5`, sha256 `fb77c2232a05a8035421382c894392f4174e4e38a18e48af3dd0290157c3a7e2`, selected through `HERDR_TEST_BINARY`) for the live CLI half |
| Logs | [`logs/H1-just-test.log`](logs/H1-just-test.log), [`logs/H3-osc52-commands.log`](logs/H3-osc52-commands.log) (third run in that file), [`logs/P1a-live-client.log`](logs/P1a-live-client.log) |

## Result

`just test` ran both CI legs green: **5471 passed, 0 failed, 31 ignored** with
default features (73 suite summaries) and **5522 passed, 0 failed, 47 ignored**
with `--all-features` (75 suite summaries). 64 of those distinct tests are
`cli::tests::*` in the `spaceport` binary (65 with `--all-features`), covering
argument parsing, the separator form, help and version precedence, and
per-subcommand arity.

`cargo test --locked -p sheprd-commands` passed **40 passed, 0 failed**: the
catalog's names, default shortcuts per platform, and the daemon request each
command becomes against a snapshot.

The live CLI path also ran, against an explicitly selected Herdr 1: `just
test-live` passed **4 passed, 0 failed**, and its
`named_session_create_and_delete_cli_contract` drives the same CLI surface
against a real daemon in a private sandbox.

## What this record does not prove

The pass rule is "the same command reaches the same typed request from all four
paths, and a refusal (bad target, ambiguous name) produces the same exit code
and message from CLI and dialog". This session attests the catalog and the CLI
path, including one live CLI route; the palette, the shortcut and the dialog
were never opened, and no refusal was compared between the CLI and a dialog.
The leg stays `not_run` until the `M·M8`/`W·W1` cross-check records the
four-path table.
