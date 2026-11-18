<!-- ABOUTME: Records snapshot indexing and deterministic sidebar lookup verification. -->
<!-- ABOUTME: Preserves ordering, captions, duplicate semantics, and cache ownership boundaries. -->

# Sidebar lookup costs

Date: 2026-11-18. This scope belongs to the original `gfde` finding.
Doctor Biz authorized implementation and validation through the root task.
Final popup capture and cleanup remain this owner's first priority.

## Inspected behavior

Plugin ordering scans all agents for each requested pane ID.
Each displayed row separately scans panes, workspaces, and tabs for its name and place.
Nested workspace layouts repeat ordering and filtering for each workspace.
Go To also repeats agent and pane lookups while building destination rows.

First-match semantics matter even for duplicate IDs.
Plugin empty orders hide all agents; unknown IDs disappear; repeated IDs repeat the first matching agent.
Stable priority ties preserve source order.
Tab visibility counts all matching workspace tabs and uses the first global tab-ID match.
Pane captions preserve the existing safe-text bound and fallback order.
Configured token rows retain their existing distinct title/label precedence.

## Required result

Prepare one bounded index per exact snapshot identity.
Store only lookup keys and row positions, with prepared local orders and per-workspace agent positions.
Borrow labels and rows from the caller's snapshot.
Reuse the same index in expanded, nested, device-tree, rail, token, and Go To callers.
Keep cross-host recency merging tied to the current LiveState values.

Root approved narrow LiveState field, constructor, exhaustive-destructure, and apply_window_update changes.
Preserve the existing exact-ID notice pause logic in apply_window_update.
Use Weak snapshot identity and an immutable Arc index to preserve transport Send/Clone.
Arc::make_mut must invalidate the old identity even when revision and boot remain unchanged.
The index must not retain a strong snapshot owner or copy agent/workspace/tab/pane records.

## Implementation and evidence

Test-only counters now instrument actual traversal predicates in the existing production helpers.
A valid 128-workspace/tab/pane/agent fixture checks row output and a linear visit budget.
A separate case checks duplicate, unknown, repeated, and empty plugin IDs.
The tests live in `sidebar/agents/tests/lookup_budget.rs`.
The baseline measured 49,408 real row visits for 640 source rows and exceeded the linear budget.
Log: `/tmp/spaceport-qnz9/gfde-sidebar-lookup-red.log`.

The index now stores first-match positions, workspace membership, and stable grouped/priority orders.
Nested layouts use prepared workspace positions instead of reordering and filtering every agent for each workspace.
Expanded, device-tree, rail, configured-token, and Go To callers reuse the same index.
Palette sources clone the populated weak cache beside their existing snapshot Arc.
They share the immutable index and keep their existing snapshot ownership.

Only SnapshotIndexCache::indexed accepts a source snapshot in production.
It checks exact weak identity before returning a borrowed indexed view.
The index, raw pairing constructor, lookup tables, and view's source field stay private.
Callers cannot pair positions from one source with another source's records.
Unchanged window projections preserve the UI cache without changing notice inheritance.
New allocations and Arc::make_mut rebuild the index even with unchanged boot and revision.

The complete sidebar filter passes 399 tests, including index lifetime and operation-budget cases.
Palette passes 124 tests; state passes 60 tests.
All three scopes report zero failures and ignored cases.
Logs: `/tmp/spaceport-qnz9/gfde-sidebar-consumers-green.log`, `gfde-palette-consumers-green.log`, and `gfde-state-consumers-green.log`.
The immutable executable is `/tmp/spaceport-9s44-gpui.xD8o8D/spaceport-tests`.
The final budget records 896 actual row visits for the same 640 source rows.
Both budget and duplicate-contract cases pass without failures or ignored cases.
Log: `/tmp/spaceport-qnz9/gfde-sidebar-budget-final.log`.
The exact-count executable is `/tmp/spaceport-9s44-gpui.VP8SIj/spaceport-tests`.

Tests verify source release, borrowed row identity, unchanged-projection reuse, and transport Send.
They also cover same-revision replacement, Arc::make_mut, first minimum tab, and first matching pane.
Existing plugin, safe-caption, token-rule, layout, accessibility, navigation, and recency tests remain green.
Root independently reviewed the final API, callers, cache ownership, and logs without a material finding.
The final owner fresh-eyes read also found no material issue.

Owned formatting and diff checks pass.
Source-size, sorted-region, and localization checks pass with 2174 messages in 114 files.
Log: `/tmp/spaceport-qnz9/gfde-sidebar-structure.log`.
The default build reports only the existing block 0.1.6 future-compatibility notice.

## Limits

Each LiveState presentation cache builds linear metadata storage and one stable priority sort per snapshot allocation.
The visit budget measures real source traversals, not wall-clock time or every hash/sort instruction.
Native release timing remains unqualified; no native speedup is claimed.
Aggregate all-feature lint remains a later shared gate.
This slice does not close the whole `gfde` finding.
Streaming, main-image region ownership, and release-profile evidence remain with root.
No commit, publication, or Kata write is authorized for this owner.
