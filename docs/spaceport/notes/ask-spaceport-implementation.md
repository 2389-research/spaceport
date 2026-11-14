# Ask Spaceport implementation sequence

## Goal and authority

Build the [approved chat experience](overview-meta-chat.md) as a separate activity section.
Keep Overview unchanged and use installed agents' existing model configuration and sign-in.
Qualify Pi first; keep Codex and Claude Code as intended later choices.
The [runtime evidence](ask-spaceport-runtime-evidence.md) defines the current qualification gaps.

This sequence records implementation boundaries and test gates.
Resolve the qualification gates before treating the sequence as an executable release plan.
Do not commit, push, merge, install, or publish without the corresponding delivery instruction.

## Implementation progress

The task worktree now contains the restricted Pi helper, observation broker, controller, and native activity section.
Pi 1.5.2 is the only implemented agent profile.
The helper writes its extension into private state and verifies its executable tool inventory before accepting questions.
The controller preserves one conversation across windows and keeps each window's draft separate.

Local tests cover observation fencing, helper receipts, literal questions, durable Stop, and confirmed interruption.
The real native fixture passed both themes at 480 and 960 pixels.
It also verified keyboard isolation from the hidden terminal.
Independent review found shutdown-budget, multiline-input, and text-contrast defects.
Those defects are fixed and independently reviewed.
The native and runtime notes record the regression evidence.
Existing `spaceport status --json` exposes bounded controller facts without starting inference.

Automatic continuation applies to confirmed graceful interruption.
After abrupt helper death, an uncertain request remains unknown and never replays automatically.
Real Pi dispatch, observation tools, answer completion, Stop, and interruption have separate captured evidence.
The final fixed-build attempt completed both tools but timed out before its answer finished.
Exact native source navigation and Back remain unqualified.
Full workspace and platform checks remain deferred to CI.
The checkboxes below retain those broader acceptance gates instead of treating partial evidence as completion.

### Final local checkpoint

Source and independent reviews are complete for this implementation pass.
Strict Clippy, default compilation, formatting, file size, sorting, and localization checks passed.
Focused checks passed: 399 helper units, one integration, and 13 actual-helper lifecycle scenarios.
The current GPUI filters passed 33 Ask tests and 37 observation tests, with two existing tests ignored.
Chat and control protocol suites passed 59 and 69 tests respectively, plus four real socket tests.
The [native evidence](ask-spaceport-native-evidence.md) records the failed final live acceptance run without marking it passed.
The task remains open because live source navigation and full CI remain pending.
Doctor Biz authorized committing and pushing the task branch after the settings pass.
Merging, installation, tagging, and release remain outside that delivery instruction.

### Optional feature settings

Doctor Biz requested an independent on/off switch and agent choice.
Settings > General > Experimental now saves `chat.ask_enabled`, which defaults false.
Enablement changes apply after restarting Spaceport; the current session keeps its startup policy across windows.
Disabled launches create no Ask worker, shared controller, restore operation, or polling task.
Dormant view entities keep rendering ownership simple without performing background work.
The separate `chat.ask_agent` picker offers Pi or the first-use choice.
Agent edits apply to the next New Chat and do not change the current conversation's agent.
Disabling the feature retains its saved conversation.
The CLI exposes both preferences through `config set` and refuses `ui panel ask` while disabled.

The configuration regression first failed because `ask_enabled` was unknown.
A window regression then exposed an unconditional controller and polling task while disabled.
That regression failed before the controller factory and dormant-task fixes.
Independent lifecycle review confirmed the correction.
The final Ask filter passed 104 tests, with one existing opt-in test ignored.
Native Settings checks passed in both themes and at 680 and 960 pixels.
They exercised actual private preference writes and reloads without making agent requests.
Native Ask presentation also passed its existing typing, focus, width, and theme fixture after explicit opt-in.
Strict Clippy and canonical formatting, size, sorting, and localization checks passed.
Evidence lives under ignored `target/ask-spaceport/enable-*.log` and `settings-native/`.
The source-click callback regression and default GPUI compilation also passed after the fixture opt-in update.
These settings checks do not close the earlier provider-backed source-navigation acceptance gate.

## Ownership

Use one app-owned conversation controller, independent of any page or window.
Views subscribe to its prepared transcript and state.
The controller owns the requested meta-chat turn, helper connection, persistence, and continuation intent.
It never owns observed terminals or coding agents.
Use the existing helper process and transport patterns with an explicit restricted role.
Do not change ordinary chat launch defaults to implement this role.

The app's observation broker owns reads from prepared connected-session state.
The broker gathers every open window's relevant endpoints, rather than only the active window.
Deduplicate exact runtime/session identities and preserve connection epochs and boot identities.
Do not merge content from different boots or infer unseen sessions from a device label.
Return coverage when a device, window, or session is unavailable.
Respect the immutable Core gate before inspecting any Core route.

## Observation contract

Add shared typed requests in `sheprd-control-protocol` without GPUI dependencies.
Use the existing private control socket and its framing and response limits.
Do not add a network listener or generic command proxy.

The proposed tool contract has exactly three operations:

| Operation | Inputs | Result |
| --- | --- | --- |
| List observed sessions | Bounded page request and optional status filter | Session metadata, opaque targets, observation times, coverage, next page |
| Read conversation excerpt | Opaque target, bounded range, requested limit | Inert text, source identity, freshness, truncation, unsupported/unavailable reason |
| Read terminal excerpt | Opaque target, bounded range, requested limit | Inert text, source identity, freshness, truncation, unsupported/unavailable reason |

The broker clamps limits against shared constants and the existing frame bound.
It rejects stale, ambiguous, or unsupported targets with typed errors.
A detail read cannot accept an arbitrary path, URL, command, or socket address.
A tool result cannot grant new authority through its text.
Register no mutation, terminal-input, approval, file, shell, web, plugin, or subagent tool.
Represent source navigation as structured client targets, separate from rendered labels.
Human source clicks use existing exact-target navigation and return behavior.

## Task 1: Qualify the restricted Pi role

Primary existing owners:

- `crates/sheprd-acp/src/pi.rs` and `pi/bridge.rs`: RPC translation and cancellation.
- `crates/sheprd-acp/src/harness.rs`: child I/O and lifecycle.
- `crates/sheprd-acp/src/session/agent.rs`: conversation startup and load behavior.

Add the launch profile separately from ordinary terminal/native-chat profiles.
Use the inspected Pi candidate and an owned extension with the three observation operations.
Explicitly remove unowned extensions, MCP, skills, templates, and project instruction discovery.
Keep provider routing and authentication owned by Pi.
Refuse a profile that needs different authentication or relies on an unqualified provider extension.

- [ ] Write failing inventory and dispatch tests using isolated configuration and synthetic sentinels.
- [ ] Prove prohibited tool names cannot execute, including after reload, model change, and session restore.
- [ ] Prove session input cannot invoke hidden shell or command-dispatch routes.
- [ ] Prove Stop clears queued work before abort and reaps only exact owned children when required.
- [ ] Prove the shipped extension loads from a trusted app-owned location.
- [ ] Add the restricted role only after these tests identify a working executable boundary.

Transport fixtures verify framing and reducers but do not qualify a real agent.
Keep real-agent/provider qualification separate and explicitly report unrun checks.
Initial offline inventory and denied-activation probes passed for 0.87.1 and 1.5.2.
This is partial evidence, not completion of Task 1.
Treat user questions as literal text so native slash commands cannot consume them.
Prove that input contract separately from the tool registry.

## Task 2: Add the observation broker and shared types

Implemented owners:

- `crates/sheprd-control-protocol/src/protocol/observation.rs`: bounded request/result types and errors.
- `crates/sheprd-gpui/src/control/server/observation/`: app-owned projections, reads, and exact source navigation.
- `crates/sheprd-gpui/src/control/server/observation.rs`: narrow broker dispatch and registry ownership.

Reuse `control/server/app.rs` host/window discovery and Overview's prepared observations.
Do not reuse Overview's unknown-to-idle presentation mapping as a source fact.
Perform blocking reads through existing workers and apply results through context updates.

- [ ] Start with failing tests for cross-window scope, duplicate observations, and Core disabled.
- [ ] Test stale epochs, changed boots, disappeared panes, and ambiguous identity.
- [ ] Test empty, malformed, oversized, truncated, Unicode, and disconnected responses.
- [x] Test that source navigation cannot select a similarly named replacement session.
- [x] Add the broker and real producer/consumer round trips.

## Task 3: Add the durable conversation controller

Implemented owners:

- `crates/sheprd-gpui/src/ask.rs`: module wiring and app-global installation.
- `crates/sheprd-gpui/src/ask.rs`: single-owner state transitions and turn dispatch.
- `crates/sheprd-gpui/src/ask/worker.rs`: background process ownership and bounded checkpoint reads.
- `crates/sheprd-acp/src/ask/checkpoint.rs`: helper-owned durable request and transcript writes.
- `crates/sheprd-gpui/src/ask/error.rs`: typed failures at this boundary.

Persist original agent/profile, conversation identity, request identity, turn receipt, received text, Stop, and outcome.
Use atomic replacement and existing private-state conventions without logging transcript contents.
Persist continuation intent before dispatch and record receipts before reporting acceptance.
Keep an app-local request ID distinct from backend idempotency.

Proposed states: unconfigured, ready, starting, answering, restoring, resuming, stopped, failed, and outcome unknown.
Keep the last answer and draft separate from operation state.
Share one controller across windows so only one owner can resume a request.
Do not automatically retry failed or uncertain submissions.

On ordinary navigation, keep the requested turn running.
On quit, checkpoint the requested turn and coordinate only the app-owned helper's lifecycle.
The precise quit/resume mechanism must pass Task 1's real-agent recovery tests.
On return, reconcile active, completed, confirmed interrupted, and unknown outcomes before dispatch.
Continue only the saved unfinished request with its original qualified profile.
Stop and completed outcomes clear continuation intent.

- [ ] Write failing reducer tests for every state transition and stale completion.
- [ ] Test restart around dispatch, acceptance, text, tool reads, Stop, and final persistence.
- [ ] Test repeated opens, concurrent windows, late old-turn events, and Settings changes.
- [ ] Test disk errors and corrupt state without losing the last valid history or sending another prompt.
- [ ] Test close/reopen without touching any observed coding session.

## Task 4: Add the native activity section and Settings

Primary existing owners:

- `crates/sheprd-gpui/src/sidebar/activity.rs`: Ask Spaceport activity item and selection.
- `crates/sheprd-control-protocol/src/protocol/app.rs`: typed activity-panel value.
- `crates/sheprd-gpui/src/chat/view.rs`: reusable native transcript and composer behavior.
- `crates/sheprd-gpui/src/config.rs` and its preference owner: saved default agent.
- `crates/sheprd-gpui/src/settings_window/pages/general.rs`: Settings entry point.
- `crates/sheprd-gpui/locales`: localized labels, hints, state, and error copy.

Add focused `ask/view.rs`, `ask/composer.rs`, and `ask/tests/` files as needed.
Extract only the actual shared rendering seam; do not construct a fake Herdr pane for app-owned chat.
Keep source links and tool details inside the existing bounded reading column.
Changing the configured default agent affects the next New Chat only.
An unavailable or unqualified agent shows a reason instead of silently selecting another.

- [ ] Write headless interaction tests before adding navigation and rendering.
- [ ] Cover first use, idle, answering, Stop, New Chat confirmation, resuming, and unavailable agent.
- [ ] Verify keyboard isolation, selection, copy, IME, and draft preservation.
- [ ] Capture real native light/dark, wide/narrow, long labels, and source navigation.
- [ ] Keep browser prototype checks separate from native QA claims.

## Task 5: Expose client-owned state and close release gates

Extend the existing CLI/control catalog in the same implementation.
Reuse `ui panel` to open Ask Spaceport with exact-window targeting.
Add bounded Ask Spaceport status inspection without adding unattended prompt submission.
Document concrete command names and help once the typed catalog is implemented.
Update `docs/cli.md` and the managed Spaceport skill where appropriate.
Record the approved meta-chat scope in the authoritative decision and acceptance graph.
Do not reopen ordinary agent-pane conversion or scheduling scope.

- [x] Add pure parsing, help, JSON, target-refusal, and protocol round-trip tests.
- [ ] Run relevant short local checks and send longer workspace/platform checks to CI.
- [ ] Run real-agent end-to-end scenarios with explicit fixture/provider authorization.
- [x] Use one independent review for tool enforcement, persistence, and lifecycle contracts.
- [ ] Report native and provider qualification gaps before enabling the corresponding agent.

## Integration boundaries

The symbol-audit worktree owns the private window-type rename.
The ACP worktree owns the helper executable rename.
Reconcile those branches before editing overlapping production files.
No integration, commit, push, or release occurred during this design pass.
