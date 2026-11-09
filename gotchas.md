# Gotchas

- 2026-11-09: Doctor Biz wants future task worktrees inside the main checkout's `.worktrees/<task>` directory. The ignore entry prevents nested checkouts from appearing as source changes. Keep active worktrees and delivered test artifacts intact when cleaning old sibling checkouts.

- 2026-11-09: Runtime generation alone is not a deletion criterion. D58 in `docs/spaceport/plan/decisions.json` permits Herdr 1 alongside Core and supersedes D55. Preserve exact runtime ownership and working UX; remove excluded feature routes independently. Core-specific gaps do not establish Herdr 1 parity.

- 2026-11-09: `crates/sheprd-gpui/src/browser` owns shared native tab IDs, split groups, persisted view references, and Herdr terminal navigation after the embedded page engine is removed. Keep that container for Core/Herdr 1 terminal and Devices views; old web/local/blank payloads only decode so the store can discard them while reserving their IDs.
