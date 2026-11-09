# Instructions to the Spaceport implementation agent

## Mission

Fork the pinned Sheprd baseline into Spaceport. Preserve the approved desktop experience. Delete excluded behavior completely. Herdr 1 and Herdr Core are allowed independently installed runtimes under decision D58. Do not edit Herdr itself or reinterpret this task as building an agent platform.

Read `SPEC.md` and `SYSTEM.md` once. For each task, use `python tools/spec.py context TASK_ID` to obtain its specific requirements, sources, gaps and tests. Re-read changed authoritative inputs; do not trust a stale task capsule.

## Non-negotiable rules

- User work and unrelated changes are untouchable. Inspect repository status before writing. Use a separate branch/worktree for this engineering task; removing the product's worktree UI does not prohibit developer Git workflows.
- Never use a live personal daemon, actual agent transcript, real tailnet credentials or upstream production updater as a test fixture. Use isolated state directories and explicit fixture endpoints.
- The selected Herdr runtime owns its processes and lifecycle. Core owns connections for Core routes; the existing Herdr 1 connector may serve Herdr 1 routes. Spaceport may start an installed local runtime but may not install/update it, migrate sessions between runtimes, or add another transport manager.
- Keep GPUI, the current terminal engine and the existing desktop interaction design. No mobile/UniFFI/Ghostty work, framework migration, plugin engine or new service.
- **Do not delete by name.** Herdr 2 currently calls its wire protocol generation 1. Shared `sheprd-protocol` types may still support the renderer. Extract needed pure types before deleting legacy behavior.
- Use source-observed methods, their generated schemas and actual runtime capabilities. Do not invent endpoint names, workspace APIs, replay support, delivery receipts or idempotency guarantees.
- A timeout after submission may mean the action happened. Never blindly retry terminal creation, input, approval, file writes or destructive operations.
- A view is not a process. Closing windows, quitting, detaching and restoring views must not stop or restart agents.
- Preserve permission requests and human decisions. No “approve everything,” hidden force flags, automatic host-key acceptance, or silent cloud/agent installation.
- Never turn a backend gap into a fake local implementation. Record evidence, mark the affected task blocked, and continue independent tasks. Do not repeatedly ask settled product questions.
- Do not delete a retained feature's tests to make a build green. Replace removed-feature tests with absence/leakage checks where appropriate.
- Keep legal notices and protocol constants that compatibility requires. New product branding is not permission to rewrite external identities.

## Working loop

**Orient → inspect → reproduce → change → verify → distill.**

For one bounded task: identify the exact input revisions and file ownership; reproduce the relevant baseline; implement the smallest coherent vertical change; run targeted tests before broad suites; inspect the resulting diff; update the task record with real command results and evidence paths. Capture a newly discovered invariant in a regression test and a short explanation at its owner. Do not accumulate redundant architecture prose.

A task is done only when its required acceptance scenarios have evidence or it is explicitly a document/static task. “Compiled,” “tests not available,” and “looks right” are not interchangeable with passed integration or platform tests. Never mark an unavailable test as passed.

## Architecture discipline

Keep the dependency direction UI/CLI → command/session model → Core adapter and pure protocol/rendering types. Reuse upstream crates before inventing new ones. Logical layers do not require one crate each. No generic transport/plugin framework for a single implementation. Prefer enums and typed errors, bounded channels and pure reducers. Thread/socket ownership must not depend on which window happens to render a record.

Use one command catalog for UI dispatch, CLI schema/help and capability tests. Use one acceptance registry. Generated documents are regenerated, not hand-edited. Prefer stable semantic tests over hard-coded command counts.

## Collaboration and handoff

Parallelize only after shared contracts are settled. Give each worker explicit files, prerequisites and test ownership. Shared lockfile, app identity, command catalog and root dependency changes have one integrating writer. A second reviewer checks lifecycle, security, deletion completeness and retained-feature parity; a second agent's assertion is not test evidence.

Use `notes/HANDOFF_TEMPLATE.md` after an interruption. State the next executable step, not a narrative of everything considered. Record facts, rationale and evidence, not private reasoning transcripts. Uncommitted work, uncertain outcomes and failed tests must be visible.

## First task

Start at **T00**. Confirm the source revisions, generate a dependency/feature inventory, capture the runtime method schema in an isolated environment, and establish the current working desktop baseline. Do not begin by mass-renaming the repository or deleting the legacy client crate.
