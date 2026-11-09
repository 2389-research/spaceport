# Gotchas

- 2026-11-09: About credits must name both the Herdr and Sheprd teams (D59). Label their links as upstream references, retain copyright/license attribution and keep Spaceport's independent identity clear.

- 2026-11-09: Doctor Biz wants future task worktrees inside the main checkout's `.worktrees/<task>` directory. The ignore entry prevents nested checkouts from appearing as source changes. Keep active worktrees and delivered test artifacts intact when cleaning old sibling checkouts.

- 2026-11-09: Runtime generation alone is not a deletion criterion. D58 in `docs/spaceport/plan/decisions.json` permits Herdr 1 alongside Core and supersedes D55. Preserve exact runtime ownership and working UX; remove excluded feature routes independently. Core-specific gaps do not establish Herdr 1 parity.

- 2026-11-09: `crates/sheprd-gpui/src/browser` owns shared native tab IDs, split groups, persisted view references, and Herdr terminal navigation after the embedded page engine is removed. Keep that container for Core/Herdr 1 terminal and Devices views; old web/local/blank payloads only decode so the store can discard them while reserving their IDs.

- 2026-11-09: Pseudo-locale leak fixtures record newly shaped text, so consecutive popovers with the same labels can produce no new messages. Test each such surface in its own test. Calling `under_en_xa` twice in one test toggles the thread-local pseudo-locale off on the second call, even with a fresh app context. Use the existing `uncut` helper for truncation measurements; keep whole untranslated labels subject to the leak assertion.
