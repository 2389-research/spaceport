# T04: native mockup and feedback route removal

Base: integration `ece5aae07`, protocol version 30. This cut removes the
development `--mockup` window and its `notes.send`/`browser.feedback` control
route. T04 stays blocked and A28 stays not run because embedded browser tabs,
native webviews, and their other control paths remain.

## Ownership and removed path

Before the cut, `--mockup` selected a separate GPUI window. Its Send action
called `control::client::send_notes`, which submitted `notes.send` to the
running app. `control/server/browser.rs` delivered notes into a Herdr pane or
kept them in `browser::Feedback`; `browser feedback`, top-level `feedback`, and
`wait feedback` collected them. The page annotation producer had already been
removed. Source search found no retained producer of this queue. The associated
event kind, socket write-failure return path, pending-delivery tick, build
feature, `just` recipe, CI test, development skill, and unused Fluent catalog
are removed. Protocol version is 31. Old wire requests fail strict decoding.

Worktree notes under `spaceport notes`, native notification feedback, retained
terminal and chat tabs, browser tab state, and explicit system URL/file
opening have different owners and remain. The retained event stream still
waits and cancels through the control socket. The native fixture screenshot
facility now uses `integration-test` directly, so removal of the mockup feature
does not remove that test tool.

## Verification

- Red: the new protocol test accepted `browser.feedback` on the base and failed
  as expected. The new CLI rejection test was then made green by the removal.
- `cargo +1.99.0 test --locked --offline -p sheprd-control-protocol`: 61 passed,
  including waiting caller, bounded response, cancellation, framing and the
  removed-wire rejection.
- `cargo +1.99.0 check --locked --offline -p sheprd-gpui --all-targets
  --all-features`: passed on macOS. Focused all-feature tests passed: removed
  CLI syntax (unit 1, executable 1), retained event log (4), page-post
  rejection (1), exact-window targeting and wait clamp (3).
- `just check-file-size`, `just check-sorted`, `just check-i18n`,
  `just format-check`, and `git diff --check` passed after the orphaned
  catalog and formatting were removed. From the package root,
  `tools/spec.py check --write-report` passed integrity/schema checks and
  `tools/test_spec.py` passed 21/21. These checks do not prove application
  conformance.

Cargo repeated the inherited `block 0.1.6` future-compatibility notice; the
macOS linker repeated its compact-unwind notice. No source warning remained
after the all-feature check. No native desktop check ran while `loginwindow`
was frontmost, and no Windows or Linux build ran in this worktree. No real
daemon, user state, Core repository, or release channel was touched.
