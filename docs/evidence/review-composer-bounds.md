# Composer limits and prepared text

Date: 2026-11-17. Partial scope: `d378` under `9s44`.
The clipboard acquisition work remains open.

## Editing bounds

The multiline chat field enforces the shared protocol's 256 KiB prompt limit before changing its draft.
It counts UTF-8 bytes after CRLF normalization and accounts for the replaced selection.
Rejected edits preserve text, selection, and marked composition.
They show the existing translated prompt-length error beside the field and in its accessibility description.
Restored drafts use the same guard.
History recall replaces text transactionally instead of clearing first.

Three regressions failed before the guard.
Native replacement enlarged the draft, IME replacement changed marked text, and an extra byte exceeded the exact limit.
The guarded input scope passed all 11 tests.
Logs: `/tmp/spaceport-9s44-composer-limit-red.log` and `composer-limit-green.log`.

## Prepared layout

The unchanged-repaint regression observed eight full layout preparations instead of zero.
A test-only counter observes the actual production layout function without a timing budget.
The cache uses text invalidation plus the full font, size, width, foreground, and marked range.
Render sizing and paint share the prepared lines.
Caret motion and scrolling reuse those lines.
Edits, composition, appearance, and wrapping changes still invalidate them.

The font-size regression also exposed a fractional viewport boundary.
Bottom alignment could push the first caret slightly above the viewport.
Caret reveal now never scrolls past that row's own top.

All 13 input tests and seven chat-composer integration tests passed.
The unchanged-redraw test now observes zero extra full preparations.
Existing editing tables restore drafts through the real setter rather than bypassing cache invalidation.
Independent review approved the bounds, cache keys, invalidation, ownership, and transactional replacements.
Final combined clipboard validation and Clippy remain pending.
The clipboard owner is implementing off-thread chat and Core acquisition next.
Do not treat these bounds as completion of `d378`.
Native IME, keyboard, and performance qualification remains pending.
