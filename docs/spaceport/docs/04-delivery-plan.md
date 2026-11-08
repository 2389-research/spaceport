# Delivery plan

The canonical work graph is `plan/tasks.json`. Status starts at `not_started`; this document is an explanation, not a second tracker. Run `python tools/spec.py ready` and `python tools/spec.py context T00`.

## Phase 0 — Prove the boundary before cutting

**T00** pins source and toolchain, inventories features and dependencies, builds a read-only reference, exercises isolated Core behavior and records actual gaps. The first output is evidence and a deletion map, not a repository-wide rename. Confirm the existing session model and rich-chat path, because Core-only support changes more than the terminal codec.

After T00, **T01** isolates the Spaceport development identity and **T02** establishes the Core capability adapter. These can proceed independently after agreeing on shared types. Production signing remains later.

## Phase 1 — One thin vertical path

**T03** creates the Core-backed presentation projection and versioned workspace metadata, without a new runtime. **T05** attaches one real local terminal through it, restores the view without restarting the process, and proves last-window quit preserves that process. Then test the same flow with a Core-managed remote terminal.

This is the proof that makes legacy removal safe. Do not delete the old backend first and attempt to reconstruct every missing behavior from memory.

## Phase 2 — Preserve the agent experience

**T06** proves chat adapter identity, observation, input and permissions with local and remote coverage distinguished. **T07** adapts host, file and service controls to Core, recording G-PORT/G-FILE rather than adding a parallel connection manager. **T08** adapts activity/notifications and supported usage/automation visibility. **T09** unifies retained UI/CLI command semantics and exposes bounded inspectable state.

If a backend gate blocks a task, complete its independent inspection/renderer tests and mark the exact unproved path. Other ready tasks continue. Do not call unsupported features “completed with graceful degradation” when the user's requirement is to retain them.

## Phase 3 — Delete completely

**T04** removes excluded IDE/Git/GitHub/browser/cloud/orchestration/mod features after the view seam is available. It may run alongside the retained-feature tasks with clear file ownership. **T10** removes legacy Herdr 1, phone server/push/pairing and remaining mixed-crate remnants only after their retained dependencies are extracted and exercised.

Small coherent deletion commits are compatible with the user's demand for complete deletion. There must be no permanently hidden compatibility implementation in the final tree. Intermediate checkpoints are development safety, not a product feature-flag strategy.

## Phase 4 — Qualify and ship

**T11** runs desktop/platform/end-to-end and agent-usability scenarios on a native OS matrix, plus resource and adversarial tests. **T12** completes release identity, signing, package-manager/installer behavior and update verification. Publishing is not authorized by this spec alone.

## Gate semantics

Task `gaps` are context and investigation targets. `must_close_gaps` are completion gates. T00 may finish after accurately documenting an unsupported path; it must not wait for every release capability to exist. T06–T08 and T11–T12 cannot claim their full result until their required gates are resolved with evidence. A blocked branch does not invalidate already verified independent work.

## Merge discipline

One integrator owns root manifests/lockfile, command catalog, shared session contracts and release identities. Parallel workers own bounded feature trees. Each change carries its requirement IDs, tests and impact on deletion/capability records. Avoid cross-cutting cleanup in a feature removal. Do not add a general task orchestration service: the repository work graph and ordinary branches are enough.

## Stop conditions

Stop the affected action, not the entire effort, when it would modify a user's runtime/state, weaken host authentication, install missing software, violate ownership, or invent unsupported semantics. A blocked product capability produces a minimal external requirement with evidence and tests needed to close it. The implementing agent must not modify Herdr as an incidental convenience.

A releasable build requires all mandatory acceptance gates or explicit owner-approved scope exceptions. A green unit suite is not a waiver for untested macOS/Windows/Linux behavior, remote native chat or forwarding.
