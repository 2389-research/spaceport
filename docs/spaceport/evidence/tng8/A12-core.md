# A12 attach flow control — headless half (partial, Core leg)

Session `H`, command `H2`. This is a partial attestation: A12's pass rule needs
both the headless protocol suite and a live Core case, and this session ran only
the headless half. The Core leg stays `not_run`.

## What ran

| Field | Value |
| --- | --- |
| Revision | `708ca09f3a2d7a61c3b56e7d9bd3537ed9125c52` (`feat/wx3-session-h`, tree clean; only `target/` was built) |
| Command | `just test-protocol` (`cargo test --locked -p sheprd-protocol -p sheprd-chat-protocol -p sheprd-client -p sheprd-core-client -p sheprd-control-protocol`) |
| Machine | macOS 26.7.1, arm64 |
| Runtime | none: no Herdr 1 or herdr-core binary is selected by this fixture |
| Log | [`logs/H2-test-protocol.log`](logs/H2-test-protocol.log) |

## Result

Every suite passed: **640 passed, 0 failed, 20 ignored** across 16 suite
summaries. The ignored tests are the opt-in live fixtures that need
`HERDR_TEST_BINARY` or `HERDR_CORE_TEST_BINARY`.

The two tests A12 names both ran and passed:

```
test tests::connection::stream_bytes_that_follow_the_answer_are_kept_and_credited ... ok
test tests::resize::confirmed_observer_keeps_resize_before_later_data_and_credits_only_bytes ... ok
```

## A12's pass rule, clause by clause

- "Bounds and credit arithmetic hold in both suites" — the first suite holds;
  the second (`P2·P2-a`) did not run in this session.
- "The live Core case closes the late stream without ending the terminal" — not
  run: no herdr-core binary is available in this lane, so
  `real_core_late_streams_close_without_ending_the_terminal` was not executed.
- "No test reports an unbounded queue" — no failing or unbounded-queue report
  appears in the log; 69 of the log's test names contain credit, queue, bound or
  limit, across the protocol, client and core-client suites.

## What this record does not prove

`Violation::PastCredit` has no dedicated test: the refusal is a code path at
`crates/sheprd-core-client/src/stream.rs:112` (that path is pre-#169 naming; the file is `crates/spaceport-core-client/src/stream.rs` after the rename) and no test in this repository
names it, so this session cannot attest it beyond the suite passing. Nothing
here proves the live credit flow against a real herdr-core, so the Core leg
remains `not_run` until `P2·P2-a` runs. The Herdr 1 leg stays `not_run` for the
reason the run sheet gives: Herdr 1 has no credit flow control and no Herdr 1
flood fixture exists.
