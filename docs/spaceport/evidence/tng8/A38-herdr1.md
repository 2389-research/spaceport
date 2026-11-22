# A38 content is not authority — headless half (partial, Herdr 1 leg)

Session `H`, commands `H1` and the `-p sheprd-osc52` run A38 names. This is a
partial attestation: A38's pass rule also needs the live malicious-content step
in `M·M8`. The Herdr 1 leg stays `not_run`.

## What ran

| Field | Value |
| --- | --- |
| Revision | `708ca09f3a2d7a61c3b56e7d9bd3537ed9125c52` (`feat/wx3-session-h`, tree clean; only `target/` was built) |
| Commands | `cargo test --locked -p sheprd-osc52`; `just test`, which contains `cargo test --locked -p sheprd-gpui --bin spaceport control::` and `window::clipboard` |
| Machine | macOS 26.7.1, arm64 |
| Runtime | none: clipboard decoding and control-surface tests only, no daemon is contacted |
| Logs | [`logs/H3-osc52-commands.log`](logs/H3-osc52-commands.log) (first run in that file), [`logs/H1-just-test.log`](logs/H1-just-test.log) |

## Result

`-p sheprd-osc52`: **3 passed, 0 failed** — the two A38 names plus the payload
decoder:

```
test tests::rejects_non_utf8_and_oversized_payloads ... ok
test tests::rejects_malformed_and_noncanonical_payloads ... ok
test tests::decodes_padded_unpadded_and_multibyte_payloads ... ok
```

`just test` ran both CI legs green: **5471 passed, 0 failed, 31 ignored** with
default features and **5522 passed, 0 failed, 47 ignored** with `--all-features`.
Within that run, 111 distinct `control::*` tests and 30 `window::clipboard::*`
tests passed, including the macOS reader's bounds and helper handling:

```
test window::clipboard::macos::tests::incomplete_unknown_and_oversized_responses_are_rejected ... ok
test window::clipboard::macos::tests::helper_arguments_cannot_fall_through_to_normal_app_startup ... ok
test window::clipboard::macos::tests::cleanup::oversized_reader_is_killed_and_reaped ... ok
```

## What this record does not prove

A38's pass rule is that "no escape, path, URL or approval-looking tool text
triggers execution, a credential fetch, an approval, unexpected file/network
access, or a command-catalog change". Only the decoding half is attested here:
no malicious content was pasted into a running window, so the end-to-end claim
(the `M·M8` transcript showing nothing executed) is missing, and no approval or
tool-output path was exercised against a real agent. The leg stays `not_run`.
