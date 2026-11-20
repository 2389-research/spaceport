<!-- ABOUTME: Records Core workspace popup capture and passive control verification. -->
<!-- ABOUTME: Preserves exact edit ownership while separating native and aggregate qualification gaps. -->

# Core workspace popup departure

Verified on 2026-11-18 in `.worktrees/review-squad-fixes` for the popup motion report (`ydr4`).
Root assigned Actions, Create, Rename, and Delete after the legacy workspace family completed.
This owner performed no commit, publication, issue-tracker write, or personal runtime access.

## Behavior and ownership

The four Core workspace popup pages now capture their display before menu reset.
Capture retains displayed labels, UUID text, notes, status, errors, selected state, and numeric scroll position.
Named forms move the existing bounded `DialogInput` before the legacy `Kept` swap.
The projection retains no edit target, document observation, backend, receiver, worker, focus handle, or subscription.
The UUID remains display text and cannot dispatch an edit.

Active and captured popups share the action and form templates.
Captured controls omit click, hover, focus, native input registration, and accessibility actions.
Captured fields use the existing passive DialogInput renderer.
The existing shared workspace scroll handle supplies a numeric offset before reset.
Frozen shells clip content at that offset.
Core reads the offset through `workspace_shell_offset()`.
The menu field retains its original `pub(super)` visibility.

Prepared action and status logic preserves the original checks and their order.
These include target version, observation epoch, document membership, pending edit, uncertain write, and backend availability.
The edit controllers retain their pending receivers and dialog generations.
Capture does not cancel, replay, or wait for accepted edits.
Late completion still cannot dismiss a newer dialog.

## Regression evidence

The initial runtime baseline used `/tmp/spaceport-9s44-gpui.dLJ3UC/spaceport-tests`.
Both new appearance tests failed for the expected behavior.
Actions and Delete lost visible content after the observation changed.
Rename moved 8.5 pixels when the renderer recomputed its status.
Actions and Create each jumped 80 pixels after closing a scrolled panel.
Log: `/tmp/spaceport-qnz9/ydr4-core-workspace-red.log`.

The captured projection fixes those failures without relaxing the equality assertions.
The detached-control test uses the existing private framed Core peer.
It captures genuinely armed forms and then opens a replacement target.
It clicks action rows, input, Submit, and Cancel without a parent input shield.
Replacement target, generation, pending state, draft, selection, IME bounds, and focus stay unchanged.
This armed-peer test runs on Unix.

An all-platform channel test checks accepted-result ownership through capture and dismissal.
The accepted receiver stays live until its completion arrives.
The completion leaves a newer dialog open.
This test does not perform or acknowledge a remote document write.
Existing epoch, version, uncertain-write, backend-identity, and generation consumers also pass.

## Verified checks

The shared default build succeeded and produced `/tmp/spaceport-9s44-gpui.WhxrQX/spaceport-tests`.
The shared build uses Rust 1.99.0, `CARGO_INCREMENTAL=0`, `--locked`, and the upstream-fixes target directory.
Cargo builds use `/usr/bin/lockf -k /tmp/spaceport-qnz9/cargo.lock`.
The build owner records the full build command in the shared build evidence.

The focused runner uses repository sandboxes and synthetic fixtures.
It ran outside the Cargo lock against the immutable executable.

```sh
/tmp/spaceport-qnz9/run-accessibility-consumers /tmp/spaceport-9s44-gpui.WhxrQX/spaceport-tests /tmp/spaceport-qnz9/ydr4-core-workspace-green menu::appear::tests::core_workspace core_terminals::workspace_ui::tests core_terminals::workspace_document::tests dialog_input::tests
```

Exit: 0.
Log: `/tmp/spaceport-qnz9/ydr4-core-workspace-green.log`.

| Filter | Passed | Failed | Ignored |
| --- | ---: | ---: | ---: |
| Core workspace departure | 2 | 0 | 0 |
| Core workspace UI | 30 | 0 | 3 |
| Core workspace document | 9 | 0 | 0 |
| Dialog input | 9 | 0 | 0 |

Both new private tests appear by name in the UI result.
The three existing live tests require `HERDR_CORE_TEST_BINARY` and did not run.

The final default build produced `/tmp/spaceport-9s44-gpui.v0RcOu/spaceport-tests`.
It includes the read-only accessor and restored private field scope.
It also includes explicit Ready and armed preconditions in the detached-control fixture.

```sh
/tmp/spaceport-qnz9/run-accessibility-consumers /tmp/spaceport-9s44-gpui.v0RcOu/spaceport-tests /tmp/spaceport-qnz9/ydr4-core-workspace-qualified menu::appear::tests::core_workspace core_terminals::workspace_ui::tests::departure
```

Exit: 0.
Both appearance tests and both private departure tests passed without failures or ignored cases.
The original Actions state was Ready, and all three original form submits were armed before capture.
Log: `/tmp/spaceport-qnz9/ydr4-core-workspace-qualified.log`.

Targeted `rustfmt --check` and `git diff --check` passed on the current source.
`just check-file-size check-sorted check-i18n` passed.
The i18n check validated 2,174 messages across 114 files.
Log: `/tmp/spaceport-qnz9/ydr4-core-workspace-quick-checks.log`.
A fresh-eyes pass found no further material issue in the owned source and tests.
Root independently approved the final source and qualified runtime results.
The review found no material ownership, routing, or control-isolation issue.

## Remaining qualification

Native keyboard, screen-reader, glyph, platform, and release-performance checks remain pending.
Doctor Biz explicitly kept native checks pending.
Headless tests and private protocol peers do not prove native end-to-end behavior.
Final consolidated all-feature host and Windows lint qualification remains with root.
