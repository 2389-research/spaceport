# 2rqs — Bound SSH chat uploads

## Result and scope

Worktree: `/Users/harper/Public/src/2389/spaceport/.worktrees/review-squad-fixes`.
No commits, pushes, PRs, Kata changes, credentials, or personal daemons.

Owned files:
- `crates/sheprd-gpui/src/chat/files.rs`
- `crates/sheprd-gpui/src/chat/tests/uploads.rs`
- `crates/sheprd-gpui/src/chat/tests/uploads/bounds.rs`

The task preserves other agents' changes, including `chat/core_pane.rs`.

## Cause and fix

The existing xtjm lifecycle already owns cancellation and exact conversation fences.
The remaining upload path admitted unlimited simultaneous batches.
It uploaded every selected path before applying the ten-attachment cap.
Completion reported success even when insertion discarded a suffix.
That success prevented the existing remote cleanup path from running.

Admission now permits one active SSH upload per pane.
It truncates selected paths to the composer's remaining attachment capacity before copying.
A full composer starts no upload worker.
A concurrent request refuses with the existing copying diagnostic.
After completion, another request can start.

The existing operation now carries `Kind::Prepare` or `Kind::Upload`.
The copy gate counts only `Upload`, preserving concurrent image preparation.
Dialogs and image saves still use the same file-work owner and cancellation flags.

Completion rechecks capacity and rejects the whole batch if it no longer fits.
That false result sends every unchanged returned path through existing original-host cleanup.
No partially accepted batch can strand a discarded remote suffix.
Normal admission already limits a batch, so this handles a changed-capacity completion safely.
No saved, wire, CLI, or command schema changed.

## Ownership and callers

`FileWork::transition` remains the sole lifecycle writer.
`begin_kind` constructs an immutable operation and dispatches the existing Begin event.
`copying` only projects current operations; it never mutates lifecycle state.
The existing transition still owns cancellation stores, removal, and idle normalization.

`ChatPane::uploading()` still counts preparation and copying for prompt blocking.
`pasted.rs` finishes preparation before passing the saved picture to attachment admission.
The separate operation kind prevents another pending picture from blocking that handoff.

`ChatPane::poll` synchronizes operations after received conversation updates.
`Chats::reconcile` drops file work when the exact target disappears.
Target replacement or epoch changes cancel old work and reject old completions.
Sequence advancement preserves work in the same conversation.
Rejected successful batches call `remove_uploaded_files` using the captured original SSH host.
Owner/window loss retains the same existing cleanup path.

## TDD evidence

Initial RED: `/tmp/spaceport-qnz9/2rqs-red.log`.
Three regressions failed for their expected causes; six existing upload tests passed.
Concurrent copies had two pending files instead of one.
Oversized selection had eleven pending files with only one available slot.
Oversized completion falsely returned acceptance.

Initial GREEN: `/tmp/spaceport-qnz9/2rqs-green.log`, nine passed.

Review found that image preparation shares the pending-file count.
Picture regression RED: `/tmp/spaceport-qnz9/2rqs-picture-red.log`.
It observed one pending file instead of preparation plus upload, which requires two.
The operation kind fixes that caller regression.
The final suite also covers a full composer and retry after completion.

## Final verification

Every Cargo command used Rust 1.99.0, `--locked`, and the shared lock wrapper:

`/usr/bin/lockf -k /tmp/spaceport-qnz9/cargo.lock env CARGO_INCREMENTAL=0 RUSTUP_TOOLCHAIN=1.99.0 CARGO_TARGET_DIR=/Users/harper/Public/src/2389/spaceport/.worktrees/upstream-fixes/target`

- `cargo test --locked -p sheprd-gpui --bin spaceport chat:: -- --test-threads=1`: 521 passed, zero failed.
  Log: `/tmp/spaceport-qnz9/2rqs-chat-green.log`.
  This includes eleven upload tests, conversation cancellation, image preparation, composer, attachment UI, and CLI chat routes.
- `cargo test --locked -p sheprd-client --lib upload:: -- --test-threads=1`: 23 passed, zero failed, one inherited ignored test.
  Log: `/tmp/spaceport-qnz9/2rqs-client-green.log`.
  Tests run real upload/cleanup shell scripts against isolated temporary files.
- `cargo test --locked -p sheprd-client --lib upload::posix::tests::short_and_growing_sources_are_rejected_and_prior_success_is_removed -- --ignored --test-threads=1`: one passed.
  Log: `/tmp/spaceport-qnz9/2rqs-client-ignored.log`.
  This separately ran the suite's inherited ignored source-growth case without retries.
- `just check-file-size`: all source files satisfy the 1,000-line limit.
- Scoped `rustfmt --edition 2024 --config skip_children=true --check`: passed for all three owned files.
- Scoped `git diff --check`: passed.

`cargo fmt --all -- --check` initially found formatting across concurrent changes.
See `/tmp/spaceport-qnz9/2rqs-fmt.log`; owned files were formatted and verified separately.
The global check is not claimed as passing.
The shared lockfile briefly changed during dependency-owner work; the refused `--locked` attempt was rerun after reconciliation.
Existing linker unwind and `block` future-compatibility warnings appear in the baseline and final logs.

## Review and limits

Fresh-eyes review found and fixed the image-preparation gate issue.
Root independently reviewed the diff, lifecycle callers, and original-host cleanup.
Root reported no material findings.

No real SSH server or native desktop was used for this subtask.
Headless GPUI proves admission, lifecycle, and completion behavior.
Client tests prove actual shell transfer and cleanup behavior.
Actual SSH authentication and native error presentation remain manual remote/native QA.
Full workspace, all-feature, and cross-platform gates remain CI/coordinator work.
No known in-scope code defect remains.
