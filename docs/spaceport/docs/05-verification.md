# Verification and agent-ergonomic acceptance

The canonical scenarios are `plan/acceptance.json`. All application scenarios initially have `status: not_run`. The package validator checks links, references and shapes—not Spaceport's behavior.

## Evidence levels

A source read establishes an implementation fact at a revision. A mock proves a local contract under its assumptions. A unit test checks an isolated function. An integration test proves a real boundary for its tested version/platform. An end-to-end scenario proves an observable journey. Screenshots can verify visual preservation, not process ownership or permission safety.

Each executed scenario records source/app/Core revisions, schema digest, OS/architecture, test command, fixture setup, observed result and artifact references. Tests use isolated state directories and allowlisted fixture hosts. Secrets and real transcripts never enter committed evidence. Redaction must be checked, not assumed.

## Core safety scenarios

A terminal survives closing one view, all windows and the application; an explicit termination ends only the intended program. Restoring a missing view never reruns the last command. A Core disconnect after `terminal.run` returns an unknown outcome until observed, never a second automatic launch. Remote machine removal leaves remote processes alive. An alias/target change while a confirmation is open invalidates the operation.

Two clients rename/workspace-edit concurrently: document CAS prevents lost updates. A successful terminal launch followed by failed membership write leaves a visible ungrouped terminal, not an automatically killed one. A Core restart resets event sequence interpretation; persistent Core identity is not mistaken for a boot epoch.

A missed event refreshes state. Unknown status stays unknown. A terminal reporting `done` remains a live process unless Core says otherwise. A disconnected host's last green status is marked stale. Time or order differences across hosts never become fabricated global causality.

## Terminal and chat parity

Replay representative terminal fixtures at every byte partition, including partial escape sequences, Unicode width/combining behavior, resizing, alternate-screen transitions, bracketed paste, mouse reporting and keyboard modes supported by the upstream baseline. Compare render/input contracts, not just screenshot similarity. A flooding terminal must not block control responses. Hidden panes are bounded and reattach correctly.

For rich chat: verify endpoint ownership and terminal/conversation identity; prove prompts and tool/permission events reach the same underlying session; reject a stale permission ID; test two viewers answering once; show/hide never kills/resumes the agent. A read-only log mirror must never look like an approval-capable live session. Keep remote and local results separate.

## Scope-negative tests

Removed UI, CLI commands, settings, worker services, provider calls and packaged binaries must be absent. No browser/webview, Git/GitHub workflow, cloud provisioning, Teleport, plugin engine, local scheduler, legacy Herdr 1 backend or mobile server is reachable. Plain terminal/chat text mentioning those concepts still renders. Product code-removal checks must not disable CI or erase attribution.

## Cross-platform acceptance

Native macOS, Windows and Linux tests cover Core startup/discovery, socket/pipe ownership, IME/paste, clipboard images, external browser/file opening, notification permissions, last-window quit, detached process survival, renderer input, settings persistence, installer and updater. On Linux include the window-system configurations the release actually claims; on Windows include console versus windowed launch behavior. A Linux cross-compile is not a Windows UI test.

Build only the architecture matrix deliberately selected for release and report it exactly. Do not claim support just because upstream publishes an architecture. Capture baseline package/renderer requirements before deleting platform-specific dependencies.

## Agent usability evaluation

Give an operating agent a bounded task and only the shipped CLI help/schema: identify which agent is waiting, find its exact host/terminal, inspect current state, navigate to it, and report the supported route for responding. Measure wrong-target attempts, unsupported actions attempted, bytes/tokens of context fetched, command count and successful observed outcomes. Compare to the retained upstream CLI baseline using the same fixture. These are evaluation measurements to collect, not performance claims.

Require semantic success, no unauthorized/destructive side effect, no fabricated outcome and no unnecessary transcript fetch. Repeat with duplicate labels, a stale window, a disconnect during submit, unreported status and a malicious tool-output instruction. Record whether uncertainty was handled correctly, not merely whether the agent kept trying.

## Resource tests

Use a reproducible corpus of idle terminals, active streams, visible panes and large transcripts. Record machine/toolchain/Core versions, CPU, resident memory, subscription count and response/paint latency. Provisional acceptance is no unbounded growth, bounded queues/output, no work proportional to hidden transcript size for a metadata query, and no material regression against the same reference fixture without explanation.

Do not claim a universal latency or memory number before measuring. Proposed budgets in the agent-interface document are limits to implement/test, not results. Include malformed/oversized frames, slow readers, repeatedly opened/closed windows and reconnect storms.

## Release evidence

A final report separates passed, failed, not-run, blocked and owner-approved exceptions. The absence of CI credentials, native runners, release signing or backend support remains visible. Specification validation is reported separately from application conformance. No result is promoted to “verified” merely because an agent wrote a file saying it passed.
