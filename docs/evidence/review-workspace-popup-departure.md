<!-- ABOUTME: Records workspace popup departure, scroll geometry, and input ownership checks. -->
<!-- ABOUTME: Separates headless verification from pending native keyboard and accessibility checks. -->

# Workspace popup departure

Verified on 2026-11-18 in `.worktrees/review-squad-fixes` for the popup motion report (`ydr4`).
Doctor Biz authorized the implementation and validation.
This work does not authorize commits, publishing, or issue-tracker changes by this owner.

## Behavior and ownership

Workspace popovers and every `WorkspaceAction` dialog preserve their content while closing.
The dialogs include Rename, NewTab, NewWorkspace, Note, and OpenWorktree.
Their active target clears immediately through the existing menu reset.
The departing projection retains displayed labels, selected states, errors, and numeric scroll offsets.
It retains no workspace target, request correlation, connection fence, or permission confirmation.

Dialog capture moves the existing bounded `DialogInput` before the legacy `Kept` swap.
It does not introduce another draft model.
Active and departing fields share their frame, text shaping, and line drawing.
Departing fields register no entity update, focus callback, native input handler, or text-input role.
Active selection, composition, caret, and IME geometry updates remain in the active renderer.

The worktree projection captures search paint and the visible row slice.
Rows retain displayed labels, paths, status text, and measured placement.
They retain no source workspace ID, request target, search entity, subscription, or scroll handle.
Menu reset drops the live picker and its search owner.
Existing late-response, boot, epoch, connection, and source fences remain unchanged.

Active workspace shells and worktree status areas track separate scroll handles.
Capture stores only their numeric offsets.
Departing shells and status areas clip passive content at those offsets.
OpenWorktree explicitly ignores a previous workspace shell offset.
The shared motion owner integrated the capture, width, shell tracking, and frozen dispatch.

Frozen controls omit hover, click, focus, and accessibility actions.
Detached-template tests click mute, rename, close, submit, cancel, input, worktree rows, and Escape.
They have no parent input shield.
Replacement targets, drafts, selection, focus, IME bounds, and picker selection stay unchanged.
Workspace-close requests, captured batches, confirmations, and endpoint lifecycle code were not changed.

## Regression evidence

The initial runtime baseline lost the popover header and every dialog body after reset.
The scrolled worktree rows also disappeared.
Both tests failed for that behavior in `/tmp/spaceport-qnz9/ydr4-workspace-red.log`.

After content capture, narrow scroll tests exposed two remaining failures.
The workspace popover and Rename dialog each jumped 80 pixels when closing.
The independent worktree error area jumped 40 pixels.
See `/tmp/spaceport-qnz9/ydr4-workspace-scroll-red.log`.
The fixes preserve exact shell and error-area bounds without relaxing either equality assertion.

An existing listing test expected the former raw unsupported-method diagnostic.
Its expectation now uses the established localized `error-client-unsupported-method` message.
This changed no production error behavior.

## Verified commands and results

The default test executable built successfully with Rust 1.99.0 and incremental compilation disabled.
Cargo used the shared upstream-fixes target directory and the shared build lock.

```sh
/usr/bin/lockf -k /tmp/spaceport-qnz9/cargo.lock env CARGO_INCREMENTAL=0 RUSTUP_TOOLCHAIN=1.99.0 CARGO_TARGET_DIR=/Users/harper/Public/src/2389/spaceport/.worktrees/upstream-fixes/target cargo test -p sheprd-gpui --locked --bin spaceport --no-run
```

Exit: 0.
Log: `/tmp/spaceport-qnz9/ydr4-workspace-offset-green-build3.log`.
The existing debug-link unwind warning and `block 0.1.6` future-compatibility notice remain.
The executable was copied under the build lock to `/tmp/spaceport-qnz9/ydr4-workspace-offset-final-tests`.

The focused runner uses the repository's isolated sandbox and synthetic fixtures.
It ran outside the Cargo lock against that immutable executable.

```sh
/tmp/spaceport-qnz9/run-accessibility-consumers /tmp/spaceport-qnz9/ydr4-workspace-offset-final-tests /tmp/spaceport-qnz9/ydr4-workspace-final menu::appear::tests::workspace menu::workspace_tests menu::worktree_open_tests dialog_input::tests input::tests
```

Exit: 0.
Log: `/tmp/spaceport-qnz9/ydr4-workspace-final.log`.

| Filter | Passed | Failed | Ignored |
| --- | ---: | ---: | ---: |
| Workspace departure | 4 | 0 | 0 |
| Workspace contracts and detached controls | 34 | 0 | 0 |
| Worktree contracts | 10 | 0 | 0 |
| Dialog input | 9 | 0 | 0 |
| Input family | 74 | 0 | 0 |

Filter counts overlap and should not be added as unique tests.
The worktree contracts include correlated responses, stale replies, cancellation, composition, and reopening.
The departure tests include narrow layouts, long labels, scrolled errors, and search-owner release.

Root independently approved the final source and exact-position results.
The review found no material ownership, routing, scroll-offset, or passive-control issue.
A final fresh-eyes pass found no further material issue in this owner's source and tests.
Targeted `rustfmt --check` and `git diff --check` passed.
`just check-file-size check-sorted check-i18n` passed.
The i18n check validated 2,174 messages across 114 files.
Log: `/tmp/spaceport-qnz9/ydr4-workspace-quick-checks.log`.

## Remaining qualification

Native keyboard, screen-reader, glyph, and release-performance checks remain pending by Doctor Biz's instruction.
These headless checks do not qualify native end-to-end behavior.
The earlier all-feature host and Windows lints predate this popup family.
The root owner must record the final consolidated lint boundary separately.
