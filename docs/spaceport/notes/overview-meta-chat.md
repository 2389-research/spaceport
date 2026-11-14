# Overview meta chat

## Status

Doctor Biz approved the first-cut experience on November 13, 2026.
Doctor Biz requested Impeccable to refine the interaction design.
The native implementation and local review pass are recorded; live acceptance remains incomplete.
The final fixed-build run completed both observation tools but timed out before the answer finished.
Exact native source navigation and Back remain unqualified; full workspace and platform CI remain pending.
See the [native evidence](ask-spaceport-native-evidence.md) for the current gate and captured results.
After trying the remote prototype, Doctor Biz said “i kind of like that.”
Keep this as the preferred visual direction while qualifying the runtime.
Doctor Biz repeatedly requested continued work after progress recaps.
Continue through native implementation and validation without treating Q&A replies as stopping points.
Commit, push, merge, installation, and release still require their delivery instructions.
Doctor Biz has now requested committing and pushing this task branch.
That approval does not include merging, installation, tagging, or release.
Worktree: `.worktrees/overview-meta-chat`, branch `feat/overview-meta-chat`.
Authoritative approved scope: D70 in `plan/decisions.json`.
The [implementation sequence](ask-spaceport-implementation.md) separates the qualification and native UI work.
Base: main `64d51b05befd80a7930e09f5ecf8fe555cf45c7f`.
The helper rename remains separate in `.worktrees/sheprd-acp-t6vb`.
Session-view UX and existing-session conversion remain iceboxed.

### Live test approval

Doctor Biz approved isolated live Pi tests by replying “continue” to the explicit sign-in request.
Pi may use its existing configured provider and sign-in for synthetic qualification content.
This approval does not permit personal session data, copied credentials, changed providers, or publication.
Keep each test app, helper, runtime, and transcript in private isolated state.

## Settled experience

Doctor Biz requested an on/off setting and an agent choice after the local implementation pass.
Ask Spaceport defaults off under Settings > General > Experimental.
Save `[chat] ask_enabled = true` to opt in on the next application launch.
Apply enablement changes after restart, and show a restart notice while the saved choice differs.
A disabled launch must not restore a request or start an Ask worker.
Keep the saved conversation when disabling Ask Spaceport.
Keep `[chat] ask_agent` separate from enablement.
Pi is the only qualified choice; an unset choice prompts on first use.
Agent changes apply to the next New Chat, preserving the current conversation's original agent.

Ask Spaceport is a chat window where a person types and submits messages.
The agent responds only when asked.
Opening the view or changing workspaces must not trigger a new request or an automatic briefing.
Returning after app shutdown may continue an unfinished request under the explicit resume decision below.
The agent process runs on the same machine as Spaceport.
The conversation can answer questions such as “what needs my attention?”.
Its scope spans all connected Spaceport workspaces and devices, independently of the selected workspace.
Its first tools are read-only and limited to session data Spaceport already exposes.
That includes status, conversations and terminal output.
It has no general project-file or system-inspection tools in the initial scope.
Answers include deep links to the relevant sessions.
Following a source link changes the view without controlling the underlying agent.

The approved retrieval policy starts with status and fetches relevant details as needed.
Use bounded excerpts, source links and truthful stale/unreachable states.
A local agent process can still send fetched content through its configured model provider.

## Conversation persistence

Keep one conversation across view reopening and app restarts, with an explicit New Chat action.
Doctor Biz approved this behavior in the first-cut experience summary.
Preserving history alone does not authorize new background prompts or replaying actions.
Doctor Biz separately authorized automatically continuing an unfinished answer when returning after app shutdown.
This authorization applies only to the saved Ask Spaceport request and its original agent.
It does not authorize restarting observed coding sessions.

## Agent choice

Doctor Biz selected an installed-agent picker on first use.
Remember that choice and make it configurable in Settings.
Doctor Biz selected applying a changed agent choice to the next New Chat only.
The current conversation keeps its original agent.
Do not assume an installed agent can enforce the required restricted tool set before qualification.
Do not read personal provider credentials or submit API-backed agent turns during design.
Doctor Biz prefers Pi, Codex, and Claude Code as the intended agent choices.
Doctor Biz accepts Pi as the first qualification target.
Keep model endpoint and API-key setup out of Spaceport; the installed agent owns that configuration and sign-in.
Do not silently substitute another authentication route to qualify an adapter.
The [runtime evidence](ask-spaceport-runtime-evidence.md) records feasibility and why Mux is outside the first version.

## View placement

Doctor Biz clarified that the chat should behave like the existing Overview.
Select it from the left activity bar and show it across the main content area.
Do not replace the narrow workspace list with a chat panel.
The current Overview already uses this activity-section pattern.
Doctor Biz rejected replacing the existing Overview and suggested a separate activity entry.
Keep the current Overview available without a Summary/Chat mode switch.
Doctor Biz approved a separate “Ask Spaceport” item with a speech-bubble icon.
Overview provides a live scan; Ask Spaceport provides requested answers with session evidence.
Do not add a Summary/Chat switch to Overview.

## Impeccable interaction brief

This is an Operate surface inside the existing native visual system.
Keep the existing theme, typography, controls, spacing, and motion conventions.
Use the retained chat reading column and composer where their ownership permits reuse.
The historical `remote-chat-empty.png` illustrates layout, not current product identity or qualified behavior.

### Proposed hierarchy

Keep one compact header with the title, current agent, and New Chat action.
Show “All connected workspaces and devices” as scope text.
Place the transcript above a bottom composer within the existing chat reading width.
Keep tool details collapsed behind short activity descriptions.
Do not add a dashboard, metric cards, file rail, or persistent tool inspector.

On first use, show the agent picker inline rather than opening a modal.
Selecting an agent configures the conversation; it does not submit a prompt.
After selection, focus the composer and show the example “What needs my attention?”.
An example fills the draft; only Submit requests an answer.
Explain that fetched session text uses the selected agent's configured provider.

### Answer and source behavior

Lead with the answer and the reason attention is needed.
Place each source beside the claim it supports.
Use readable session, workspace, and device labels rather than raw identifiers.
Show observation time and unavailable coverage when relevant.
Keep source identity separate from its display label.
If a source no longer resolves, explain that without selecting a similar session.
Following a source must preserve the chat and offer the existing navigation return path.

### States and recovery

Before Submit, show no inference activity.
During a turn, show concrete progress such as “Checking session status” when supported by real tool events.
Keep draft text, reading position, and existing answers when a turn fails.
Do not retry failed requests automatically.
Resume unfinished requests after app shutdown only under the explicit resume decision below.
Keep the composer usable for drafting during a turn.
Allow one active request; keep Send unavailable until it finishes or the person chooses Stop.
Do not introduce a hidden prompt queue in the first cut.
Distinguish no connected sessions, incomplete coverage, unavailable agent, and request failure.
Do not describe unreachable sessions as idle or terminated.
Treat existing answers as dated observations rather than live state.

Switching away keeps the requested answer running and saves its result.
Returning restores the conversation without submitting another prompt.
Changing the Settings agent applies only to the next New Chat.
Keep the current agent identifiable until that conversation ends.

### Native layout and input

Use the existing bounded chat column on wide windows.
On narrow windows, wrap prose and source labels without adding horizontal page scrolling.
Keep the composer and primary controls reachable when the header compresses.
Preserve keyboard focus isolation so chat input cannot reach a hidden terminal.
Reuse native chat selection, copy, keyboard, and IME behavior.
Use the existing motion module for section and disclosure transitions.
Do not animate incoming text or steal scroll position while someone reads earlier messages.
Verify light, dark, narrow, long-label, keyboard, and IME cases during implementation.

## Lifecycle decisions remaining

Doctor Biz selected finishing and saving the requested answer when the chat view closes.
Doctor Biz selected automatically resuming an unfinished answer when returning after whole-app shutdown.
Runtime ownership, enforced tool configuration and the resume mechanism remain design work.
If New Chat is chosen during a turn, offer “Stop and start new” and “Keep this chat” inline.
Keep the current conversation and answer running until the person chooses the first action.
Starting a new conversation must not erase the previous saved conversation as an incidental effect.
Do not stop or restart observed sessions as part of any meta-chat lifecycle action.

The proposed startup behavior creates the helper when the person submits the first message or resumes an unfinished request.
Opening an idle chat, choosing an agent, and restoring completed history do not request an answer.
Keep the conversation owner independent of the page that renders it.
Allow one requested turn at a time in the first cut.
Keep Stop available during a turn and preserve text already received.
Stop applies only to the requested overview-chat turn, never to observed coding sessions.
The exact adapter must prove cancellation and child cleanup before this behavior ships.

The approved quit experience saves received text and the unfinished request's identity.
On return, restore the conversation and automatically continue the unfinished answer.
Show a short “Resuming answer…” state while actual resumption runs.
If the answer finished while the app was away, show the finished answer without requesting another.
If the original turn remains active, reattach rather than submit a duplicate.
If continuation needs another inference request, preserve the original conversation and refresh session evidence as needed.
Never present conversation loading alone as proof that an unfinished turn resumed.
If the adapter cannot establish the turn's outcome, expose that uncertainty rather than blindly resend.
Prove these behaviors for each supported agent before promising automatic resumption in the picker.
An explicit Stop or completed answer clears resume intent.

### Resume acceptance scenarios

- Reopening an idle or completed conversation makes no model request.
- Reopening after an unfinished request continues that request with its original agent.
- Repeated opens and multiple windows do not create concurrent continuations.
- An answer completed while away appears without another request.
- An explicit Stop remains stopped after app restart.
- An unavailable agent preserves history and shows a truthful recovery state.
- An uncertain submission outcome never causes a blind duplicate request.

## Context inspected

- `crates/sheprd-gpui/src/overview.rs` gathers prepared agent facts across a window's connected endpoints.
- `crates/sheprd-gpui/src/overview/model.rs` separates presentation mark identity from navigation targets.
- `docs/cli.md` documents the existing private control interface and its exact-window targeting rules.
- The retained chat bridge owns a conversation; native views attach as subscribers.
- `crates/sheprd-acp/src/session/agent.rs` capability-gates conversation loading; this alone does not prove turn recovery.

The existing Overview maps unknown agent status to idle.
The meta chat must not repeat that interpretation as a known fact.
Decide the in-scope correction when implementing observation tools; no code changed during Q&A.
Deep links must identify the exact live session, rather than rely on the Overview's persistent mark key alone.
Core observation must respect its disabled-by-default app-session gate.

## Design work remaining

- [x] Restore the prior Q&A decisions and inspect relevant Overview/chat context.
- [x] Settle conversation persistence, activity-section placement and local agent selection.
- [x] Preserve the existing Overview and record the separate-entry direction.
- [x] Define the proposed ownership, on-demand startup, detach, reset and resume behavior.
- [ ] Define read-only tool contracts, enforcement, bounded reads and source targets.
- [ ] Confirm the interaction brief and resolve remaining lifecycle choices.
- [x] Record approved scope as D70 and prepare the implementation sequence with explicit qualification gates.

## Native implementation progress

- Observation owner: shared bounded protocol, global prepared-state broker, exact source resolution, and CLI inspection.
- Helper owner: restricted Pi role, owned extension, durable receipts/checkpoints, literal questions, and exact child cleanup.
- Native owner: app-global controller, attach subscription, transcript/input, activity navigation, preferences, and native tests.
- Parent owner: contracts, test/build coordination, integration, independent review, and qualification evidence.

Reuse the existing helper attach protocol with pane-free sessions.
Keep ordinary chat replay behavior separate; Ask must not blindly resend pending prompts.
The helper owns its durable checkpoint; native views own only presentation state.
The broker endpoint getter must represent a successful bind by this app, not a computed path to another instance.
Pin Cargo to `+1.99.0` and serialize its shared target access.

Keep existing runtime ownership and no-surprise lifecycle rules.
The agent must not start, stop or reconfigure observed sessions.
Do not restore desktop conversion/recovery or build agent scheduling as part of this feature.
Do not commit, push, merge or release without an explicit delivery request.

## Design evidence and limits

Impeccable context found an established implementation with no PRODUCT.md or DESIGN.md.
The existing system remains the visual authority; this work does not redesign it.
PRODUCT.md captures confirmed product facts and links to the authoritative scope.
Its platform description uses native desktop because Impeccable's platform enum omits that target.
The preceding Q&A and explicit experience approval supply the product interview.
An interactive prototype lives in `target/ask-spaceport/ask-spaceport.html` with synthetic session evidence.
It demonstrates the proposed flow; it does not connect to agents or establish adapter qualification.
Browser checks cover selection, submission, view switching, Settings timing, Stop, and source navigation.
Follow-up and reload checks preserve earlier turns and an explicit sample observation time.
The throwaway prototype restores up to six prior turns and keeps chat archives only in memory.
Those limits do not define production persistence or history behavior.
Independent review scored both conversation-retention and freshness fixes resolved for the prototype.
Static JavaScript/DOM checks and specification integrity passed.
The design detector flagged small captions and titlebar padding, which were adjusted before the final captures.
Its window-wrapper padding warning does not describe text placement; padded child chrome owns that text.
Native implementation, model-backed qualification, and native screenshots remain unrun.
Offline Pi inventory and denied-activation checks passed for 0.87.1 and 1.5.2.
Those checks used isolated state and inert fixtures, without a model prompt.
Full real-agent qualification remains open; see the runtime evidence for exact limits.
The layout follows inspected `sidebar/activity.rs`, `overview/view.rs`, and `chat/view.rs`.
