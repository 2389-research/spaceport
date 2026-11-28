# B10 binary transfer matrix

B10 (`05m4`, "Binary upload/download behavior complete + verified",
`docs/notes-backlog-2026-11-26.md`) is the backlog row for gap **G-FILE** and
acceptance **A25/A26** under requirement SP-014. SP-014 and
[the contract note](../docs/01-herdr-contract.md) ask for six things to be
*checked separately*: binary upload, download, overwrite behavior, symlink
handling, temporary-file lifetime and cancellation. This note records which
route carries each of them at `6a678a90f5`, what proves it, and what is still
missing. It is a source-and-test audit plus the deterministic runs listed at the
end; no native drag/drop, real SSH host, Windows/Linux client or Core runtime
was exercised for it.

## Upload, and the safety cells around it

The retained upload is Herdr 1's: `spaceport_client::upload_files` stages a
regular file on a POSIX SSH host as a private temporary file, and
`remove_uploaded_files` removes it again. `window/transfers.rs` runs it off the
UI thread and pastes the returned path into the pane it captured, and
`chat/files.rs` uses the same pair for a chat attachment. Both hold the
distinction SP-014 keeps: bytes are copied first, and submitting the path is a
separate effect that only happens after the copy acknowledges.

Each cell is decided in `crates/spaceport-client/src/staging.rs` (the one
`/bin/sh` script that mints `${TMPDIR:-/tmp}/herdr-upload.XXXXXXXXXXXX` under
`umask 077`, creates the file exclusively, counts the bytes and removes what the
client did not keep) and in `src/upload.rs` (the client half, which polls
cancellation and an idle deadline on every read and write and owns rollback).

- **Binary upload** — `multi_file_binary_roundtrip_preserves_quoted_names_and_private_permissions`
  streams three chunks of every byte value, an empty file and a symlink to a
  regular file through the real scripts, and reads back identical bytes with
  mode `0600` in a `0700` directory, including a name carrying `'`, `$`,
  backticks and a space.
- **Overwrite** — `remote_exclusive_create_never_overwrites_an_existing_file`
  and `remote_trap_owns_created_file_before_acknowledgement` prove the host's
  `set -C` create never replaces a file, and
  `full_upload_collision_rolls_back_prior_success_without_deleting_collision`
  proves a later failure rolls back the batch without touching the colliding
  file or streaming the failing payload. `incomplete_created_ack_never_grants_file_cleanup_ownership`
  keeps cleanup from claiming a file whose acknowledgement was lost.
- **Symlink** — a source symlink to a regular file is followed, a symlink to a
  FIFO is refused by `public_limits_names_and_descriptor_validation_precede_spawn`,
  and `cleanup_refuses_replaced_staging_directory_and_does_not_follow_file_symlinks`
  proves cleanup never follows a symlink, in the staging directory or in the
  file's place, and never removes unrelated contents.
- **Temporary-file lifetime** — the staged file outlives the transfer and is
  removed only by `remove_uploaded_files`
  (`successful_upload_paths_can_be_cleaned_after_late_cancellation_and_retried`,
  `cleanup_uses_bounded_command_with_256_quoted_path_pairs`), while a short
  receive is removed by the host's own exit trap
  (`remote_short_receive_trap_removes_file_and_directory`). The client-side
  cleanup after a cancelled or superseded paste is covered by
  `window/transfers/tests/completion.rs` and `cancellation.rs`.
- **Cancellation** — `cancellation_and_idle_deadline_are_polled_on_every_loop`,
  `cancellation_interrupts_a_backpressured_socket` and
  `sparse_larger_than_u32_reports_u64_total_without_loading_it` prove a cancel
  stops data within one chunk, including on a socket whose peer stopped
  reading, and that the abandoned staging directory is removed.
- **The bytes that arrive are the bytes measured** —
  `copy_refuses_a_source_that_shrank_or_grew_after_it_was_measured` refuses a
  source rewritten shorter or longer than the length `prepare` measured. It
  covers, on CI, what the end-to-end
  `short_and_growing_sources_are_rejected_and_prior_success_is_removed` covers
  only when run by hand: that test stays `#[ignore]`d as flaky on macOS CI
  runners.

Two other binary paths exist and are bounded, but neither is a general file
transfer. One clipboard image or one image file reaches a remote pane through
the daemon's image bridge (`clipboard` in `spaceport-client`,
`window/images.rs`), with reservations, per-frame bounds and cancellation, and
`chat` attachments are turned into ACP content blocks —
`a_picture_goes_as_an_image_when_the_agent_takes_images`,
`text_is_embedded_and_binary_or_large_files_are_linked` and
`only_absolute_paths_to_files_are_readable` in `spaceport-acp/src/files.rs`.

## What is not there

**No retained client route downloads a file.** No window, menu, palette, control
command or CLI request in this repository copies bytes from a host to this
machine; `docs/cli.md` has no such command. The requirement to *check* download
therefore ends in a finding rather than a test: there is nothing in the retained
product to exercise, and Core has no binary read to build one on.

**Core has only a text read.** `Core::file_read` is typed, bounded and refused
before dispatch when absent (`spaceport-core-client/src/tests/file_read.rs`), and
Core's handler returns UTF-8 lines only and rejects non-text bytes; that is why
those tests say they stand in for neither binary download nor file replacement.
`file.write`, `file.edit` and `machine.exec` were left untyped in the client
because they overwrite without a precondition, follow symlinks, or truncate a
one-shot command's output — using `machine.exec` as a transport would be exactly
the text-API masquerade SP-026 forbids.

**Core's upload has no desktop handoff.** `Core::terminal_put` stages a binary
basename of at most 8 MiB and returns the staged path; scripted-peer and private
fixture tests prove the exact machine-qualified target, the base64 payload, the
size and name refusals, and distinct non-overwriting staging on a real pinned
Core. The app never calls it: a bare Core pane draws `CoreTerminalView` with no
file-drop handler, and pasting the returned path into the target program is a
product decision about the handoff that this note does not make.

## Checks run for this note

At `6a678a90f5` plus this note's one added test, on macOS 26.7.1:

```
cargo test --locked -p spaceport-client --lib upload        # 33 passed, 1 ignored
cargo test --locked -p spaceport-client --lib clipboard     # 28 passed
cargo test --locked -p spaceport-client --lib copy_refuses_a_source_that_shrank
                                                            # 1 passed
cargo test --locked -p spaceport-client --lib               # 420 passed, 2 ignored
cargo test --locked -p spaceport-acp --bin sheprd-acp files::   # 3 passed
```

The upload tests drive the real `/bin/sh` scripts through the injected command
seam rather than a real `ssh` child, so they prove the script, the byte
accounting, the cleanup and the client's state machine, not SSH trust policy or
a remote account. `spaceport-gpui`'s window-level transfer tests were not run
for this note (they need a full GUI test build), and no native drag/drop,
Windows, Linux or Core runtime run was performed. A25/A26 stay `not_run` on both
legs: a passed leg needs a scenario on the running product, and the download
half of A26 has no route to run.

## Recommendation

B10 as worded cannot close in this repository: its upload half is implemented
and proved, and its download half needs a Core-owned binary read route that does
not exist (already recorded as an external dependency in
[t07-file-route.md](t07-file-route.md)). Split the row: the upload matrix is
this note, and what remains is (a) an external contract request for a bounded,
cancellable binary read on an exact machine and path with a stated symlink
policy, and (b) the Core pane's post-`terminal.put` handoff, which needs a
product decision before code. G-FILE stays open for both.
