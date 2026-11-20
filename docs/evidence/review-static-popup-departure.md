<!-- ABOUTME: Records simple popup departure content and inactive control checks. -->
<!-- ABOUTME: Separates headless lifecycle evidence from pending native qualification. -->

# Static popup departure

Date: 2026-11-17. Scope: five simple dialog families under `ydr4` and `9s44`.
Doctor Biz authorized the reported fixes and their validation.
No commit, push, publication, or Kata write was made by this owner.

## Behavior

About, Agent Skill, Version Mismatch, Open Link, and Update share their live and departing templates.
Their departing state contains only the data needed to paint.
It retains no input entity, focus handle, subscription, task, or action closure.
The shell owner captures this data before resetting live menu state.

Version Mismatch keeps its captured runtime notice after reset.
Open Link keeps the address the question showed after reset.
Confirming Open Link clones the bounded address before dismissal so departure can capture it.
The existing consent reset and explicit OS-open behavior remain intact.

Update keeps bounded labels, the safe suggested command, and cached parsed release-note lines.
It stores the scroll offset as a plain point.
The live body owns its scroll handle through MenuState.
The departing body clips the shared content at the captured offset without scroll handlers.
Markdown links keep their styling but register no outgoing link actions.

About and Agent Skill need no dynamic menu data for their body.
The Agent Skill template moved from `window/commands.rs` to `agent_skill/offer.rs`.
All active button and link callbacks remain in the shared templates.
Inactive templates omit those callbacks.
This prevents stale controls from acting on replacement consent.

## Regression evidence

Three actual headless window regressions failed before implementation.
Version Mismatch and Open Link lost their bodies when dismissal reset their live state.
Update lost its scrolled body after a replacement snapshot arrived.
Log: `/tmp/spaceport-qnz9/ydr4-static-dialogs-runtime-red.log`.
All three failed for the expected missing departing body.

The three regressions then passed after capture and template wiring.
Three additional tests mount frozen content without the shell's input shield.
They check About links, replacement Open Link consent, and replacement Skill/Version dismissal.
All six family cases passed.
The broad `static_` filter also ran one unrelated existing chat-widget test.
Log: `/tmp/spaceport-qnz9/ydr4-static-family-green.log`: seven passed, zero failed.

Root independently reviewed capture ownership, callbacks, bounded data, and scroll handling.
That review found no material defects in these five families.
Other popup families and generic retained DialogInput remain with the shell owner.
This record does not claim those families are complete.

## Final consumer checks

An additional actual Markdown test checks that live links open while static links stay inactive.
Its first build stopped on a concurrent CloseConfirmation test signature change.
Log: `/tmp/spaceport-qnz9/ydr4-static-links-green.log`.
After that consumer fix, the link test passed.
Log: `/tmp/spaceport-qnz9/ydr4-static-links-final.log`.

The exact consumer script is `/tmp/spaceport-qnz9/check-static-dialog-consumers.sh`.
Run it under the shared Cargo lock against the freshly built GPUI test artifact.
It covers About, runtime notices, release notes, managed skills, static templates, motion fixtures,
pseudo-locales, and actual terminal hyperlink confirmation.
All 56 tests passed with no failures or ignored cases:

| Scope | Passed |
| --- | ---: |
| About | 8 |
| Version Mismatch | 4 |
| Runtime announcements and notes | 7 |
| Safe Markdown | 11 |
| Managed skill | 16 |
| Frozen controls without a parent shield | 3 |
| Actual departing dialog bodies | 3 |
| Pseudo-locales | 2 |
| Terminal hyperlink confirmation | 2 |

Summary: `/tmp/spaceport-qnz9/ydr4-static-consumers-final.log`.
Full logs: `/tmp/spaceport-qnz9/ydr4-static-consumer-{1..9}.log`.
The source received a final direct fresh-eyes review with no material findings.
The size, sorted-region, localization, and full diff checks passed afterward.
Localization reported 2162 messages in 114 files.
Logs: `/tmp/spaceport-qnz9/ydr4-static-structure-final.log` and `ydr4-static-diff-final.log`.
The aggregate format check found only concurrent chat and status-bar test module ordering.
Log: `/tmp/spaceport-qnz9/ydr4-static-format-final.log`.
Both owners received their exact paths.
The direct Rust 1.99.0 format check passed for all 14 static-family source and test files.
Log: `/tmp/spaceport-qnz9/ydr4-static-owned-format.log`.
The final aggregate Clippy gate remains with the integration owner after source stabilizes.

All Cargo commands use `/usr/bin/lockf -k /tmp/spaceport-qnz9/cargo.lock`.
They use Rust 1.99.0, locked dependencies, and `CARGO_INCREMENTAL=0`.
The shared target is `.worktrees/upstream-fixes/target`.
The initial green build reports the existing debug-linker unwind and `block 0.1.6` notices.

Native animation, keyboard delivery, screen-reader behavior, and performance qualification remain pending.
