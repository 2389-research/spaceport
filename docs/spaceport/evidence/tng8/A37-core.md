# A37 timeout is not failure — headless half (partial, Core leg)

Session `H`, commands `H1` and `H4`. This is a partial attestation: A37's fixture
ends with `P2·P2-d`, a live Core run, which this session could not execute. The
Core leg stays `not_run`.

## What ran

| Field | Value |
| --- | --- |
| Revision | `708ca09f3a2d7a61c3b56e7d9bd3537ed9125c52` (`feat/wx3-session-h`, tree clean; only `target/` was built) |
| Commands | `just test` (contains `chat::tests::delivery` and `chat::tests::session`); `cargo test --locked -p sheprd-gpui --bin spaceport core_terminals::workspace_document::edit::tests -- --nocapture` |
| Machine | macOS 26.7.1, arm64 |
| Runtime | none: the scripted peer is in-process, and no herdr-core binary is selected |
| Logs | [`logs/H1-just-test.log`](logs/H1-just-test.log), [`logs/H4-H7-gpui-filters.log`](logs/H4-H7-gpui-filters.log) |

## Result

`just test` ran both CI legs green: **5471 passed, 0 failed, 31 ignored** with
default features and **5522 passed, 0 failed, 47 ignored** with `--all-features`.
The delivery and session cases A37 names all passed:

```
test chat::tests::delivery::a_failed_write_is_unknown_and_is_never_replayed ... ok
test chat::tests::delivery::a_written_command_retries_with_the_same_id_only_in_its_epoch ... ok
test chat::tests::delivery::dispatch_waits_for_the_epoch_and_never_queues_a_command_twice ... ok
test chat::tests::session::commands_wait_for_their_acknowledgement ... ok
test chat::tests::session::an_outage_alone_never_drops_a_prompt ... ok
```

`H4` passed **7 passed, 0 failed, 3 ignored**. The fenced `WriteUnknown`
assertions A37 asks for are three of the seven:

```
test core_terminals::workspace_document::edit::tests::timeout_is_not_a_conflict_to_retry ... ok
test core_terminals::workspace_document::edit::tests::malformed_set_success_is_unknown_but_unsent_errors_are_definite ... ok
test core_terminals::workspace_document::edit::tests::rebased_write_timeout_keeps_its_attempted_version ... ok
```

The three ignored tests are exactly the live Core cases, each reporting
`ignored, needs HERDR_CORE_TEST_BINARY`:

```
test core_terminals::workspace_document::edit::tests::live::create_rename_archive_delete_only_the_grouping_document ... ignored, needs HERDR_CORE_TEST_BINARY
test core_terminals::workspace_document::edit::tests::live::disjoint_conflict_rebases_and_same_target_conflict_refuses ... ignored, needs HERDR_CORE_TEST_BINARY
test core_terminals::workspace_document::edit::tests::live::membership_edits_only_the_private_core_document ... ignored, needs HERDR_CORE_TEST_BINARY
```

## What this record does not prove

A37's fixture names `P2·P2-d` after the headless half, and that live Core run
did not happen here: no herdr-core binary is available in this lane, so nothing
in this record shows the reconciliation against a real Core that `P2·P2-d`
supplies. The run sheet's session table lists A37 without "(part)" while its own
Fixture column requires the live half; that inconsistency is reported rather
than resolved by claiming a pass. The leg stays `not_run`.
