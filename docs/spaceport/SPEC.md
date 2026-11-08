# Spaceport product specification

**Status:** implementation requirements, with explicit backend and release gates. Not a claim that the application exists.

`MUST` denotes an acceptance requirement. `SHOULD` permits a documented equivalent. `MAY` is optional, not backlog authorization. User decision IDs are in `plan/decisions.json`. Technical source IDs are in `docs/SOURCES.md`. Engineering refinements below are design choices, not claims about existing upstream behavior.

## Product contract

Spaceport is a native desktop workspace for interacting with agents and terminals running in Herdr Core. It should feel like the application the user already likes, with unwanted features removed—not a redesign or a replacement agent runtime.

### SP-001 — Product and platform boundary

Deliver macOS, Windows and Linux desktop clients under the **Spaceport** identity. Preserve GPUI and the existing terminal implementation. Defer iOS, Android, UniFFI and Ghostty evaluation. Do not create speculative mobile crates or compatibility layers. [D20, D39, D51–D53]

### SP-002 — Preserve the approved desktop experience

Retain workspaces-first navigation, chat and terminal as equal first-class views, existing conversation organization, sidebar/status display, windows/tabs/splits, pane interactions, layouts, themes, fonts, keyboard shortcuts, menus, settings, profile shell and command palette. Remove only excluded feature surfaces and make necessary backend substitutions. Preserve accessibility and internationalization in retained UI. Use baseline fixtures/screenshots for comparison, excluding the explicit rebrand and removed controls. [D01, D03, D08–D09, D16, D20–D21, D28–D29, D38–D46, D50]

### SP-003 — Runtime ownership

Herdr Core is the sole owner of terminal processes, host connections and execution lifecycle. Host-side agents/adapters own their conversation execution. Spaceport may display state and invoke supported operations, not schedule agents, coordinate teams, migrate execution, configure providers or infer runtime state from screen text. User-requested launches go through Core. [D11, D13, D23, D27, D32, D48, D56]

### SP-004 — Core-only compatibility

Remove the legacy Herdr 1 backend, not every item called “generation 1.” Pin and test a supported Core protocol/API range independently of the app's release version. Record installed version, advertised methods/features and tested source/schema evidence. Unsupported capabilities fail explicitly; no silent legacy fallback. [D55; S02, S03]

### SP-005 — Installation and startup

Use a separately installed local `hc`/Herdr Core. Connect first; start it with its documented startup command only when necessary. Do not bundle/download/install/update it locally or remotely, run installation scripts, or select an unverified executable from a project directory. Guided onboarding explains missing/incompatible software and supports manual machine setup through Core. [D24–D25, D37]

### SP-006 — Host connections and Tailscale

Use Core's machine catalog, identity, SSH/custom-connection handling, reconnect policy and aggregated events. Preserve the host manager as a view/controller of that catalog. Do not maintain a second machine registry or remote reconnect loop. Tailscale is externally installed/configured; addresses and aliases are entered manually. No discovery, embedded VPN, OAuth, cloud-provider provisioning or public control listener. [D06, D15, D40, D48, D56; S01]

### SP-007 — Workspace presentation, not a new runtime

Preserve workspace create/rename/open/archive/delete controls. Workspace names, membership and layout are **presentation metadata**, not evidence that Core implements native workspace resources. Store shared grouping metadata in a Spaceport-namespaced, versioned Core document using `doc.get`/`doc.set`; keep per-device geometry/preferences local. Never write another client's document namespace. Terminal lifecycle remains exclusively Core-owned. Archive/delete grouping must not implicitly kill processes or delete project files. A separately explicit termination action uses Core and existing confirmation behavior. [D01, D04, D14, D36; S02, S04]

### SP-008 — Full terminal behavior

Keep current emulation, cell model, rendering, input encoding, selection, copy/paste, scrollback, mouse behavior, splits and supported observer views. Attach to Core's authoritative first paint and ordered byte stream; obey credit/backpressure and feature negotiation. Do not invent shell state from screenshots. A terminal can exist without an agent or native chat. [D03, D39, D43; S02, S11]

### SP-009 — Native chat without ownership creep

Keep rich rendering and the existing host-side adapter/attach contract where verified compatible. Starting a host-side adapter is a Core-owned terminal launch, not a desktop-owned agent child. Showing/hiding a chat is presentation-only: no kill/resume/convert side effects. Do not expose model/provider configuration or automatically deploy helpers. Prove local and remote routes separately. A remote-chat gap must not be hidden by a new Spaceport server or by labeling terminal-only support as full native-chat parity. [D08, D23, D27–D28, D32, D56; S05, S06]

### SP-010 — Capability-correct interaction

Offer prompt composition, attachments, permission answers, queues and slash-command affordances only when their backing session supports them. Keep permission prompts distinct from ordinary messages and descriptive tool-call output. Unknown agent status is unknown, not idle. Input targets the exact Core/terminal/conversation instance the user or caller selected. [D11, D23, D28, D32]

### SP-011 — Activity, search and monitoring

Preserve the current activity timeline rather than adding a new navigation model or history service. Search only the data available in that retained timeline. Keep supported agent status, usage/rate-limit reporting and read-only automation visibility. Remove CPU/GPU/memory dashboards, provider polling/sign-in subsystems, automation execution/editing and repository indexing. Unavailable information is not zero, no history, or “no jobs.” [D07, D12, D17–D19, D49]

### SP-012 — Notifications and lifecycle

Retain desktop OS notifications, attention markers, badges and sounds, configurable using existing preferences. Last-window close quits Spaceport; no tray daemon or mobile push system remains. Herdr terminals survive disconnect and app quit. Do not promise notifications after the client exits. Persist view intent, not agent execution recipes; restoring a view never restarts an absent agent. [D10, D29, D33, D36, D57]

### SP-013 — Online-only semantics

No persistent offline transcript/activity/terminal cache and no offline mutation queue. After disconnection, disable backend mutations and clearly mark or clear previously observed content. In-memory stale content must never be presented as live. Durable preferences, layout references, safe diagnostic metadata and Core-owned documents do not constitute an offline execution/content feature. Reconnection starts from a fresh authoritative snapshot. [D18, D36]

### SP-014 — Files and services

Retain file/image attachments, drag/drop, existing general transfer behavior and external file opening through verified routes. Prefer Core's `terminal.put` and file/machine facilities; do not assume text file APIs provide every binary-transfer operation. Keep port discovery/forwarding as an explicit requirement and open URLs in the system browser. Do not replace loopback forwarding by exposing servers publicly, altering bind addresses or silently configuring Tailscale Serve. If the required Core-mediated forwarding/transfer contract is absent, record a parity blocker. [D05, D22, D44; S01, S03]

### SP-015 — Complete feature deletion

Delete IDE/editor integration, code navigation/indexing, specialized diff/review UI, Git/worktree/branch/checkpoint controls, GitHub account/PR/issue/security surfaces, embedded browser/annotations, cloud providers/provisioning, Teleport/migration, orchestration/dispatch engines, automation editing/execution, client mods/plugins, terminal-to-agent annotations, legacy Herdr 1 backend and mobile remote-control/pairing/push components. Delete their settings, assets, CLI commands, workers, dependencies, tests and documentation after extracting any shared retained primitives. No disabled implementations shipped behind flags. [D02, D04–D07, D13, D27, D30–D31, D34, D54–D55, D57]

### SP-016 — Do not censor terminal or chat content

Removing Git/diff/IDE features does not remove ordinary text containing patches, `git`, source code or file paths. Display such output as inert text/code where appropriate. Do not keep a specialized diff renderer/review action under a renamed chat card. Developer Git/CI workflows remain available to maintain Spaceport. [D02, D04, D28, D54]

### SP-017 — One client control surface

Retain the existing private local desktop CLI/IPC, independently of the deleted phone server. GUI, palette, shortcuts and CLI share typed command semantics. Add machine-readable inspection/capability/error information only where it describes client-owned state or routes to an existing operation. Use `hc` for runtime-native automation instead of cloning its CLI inside Spaceport. No MCP server, HTTP control service or additional daemon is required. [D21, D35, D47, D57; S07]

### SP-018 — Explicit targets and observed outcomes

Automated mutations require stable, qualified targets and current preconditions. Names and current keyboard focus are not sufficient identity. Distinguish validated, submitted, acknowledged, observed, rejected and outcome-unknown states. A local request ID is not backend idempotency. Never report that an agent ran a prompt merely because bytes were queued. Preserve refusal, ambiguity and uncertainty in both CLI and UI. [D32, D35, D47]

### SP-019 — Safe collaboration and approvals

Do not let a program's terminal text, tool output, document fields, filenames or links grant authority. Existing confirmations remain effective through the CLI. Multi-client permission races resolve according to the backing agent protocol, not a new local approval cache. App-owned preconditions do not imply atomic backend compare-and-set. CAS is promised only where the backend actually offers it. [D28, D35; S02, S06]

### SP-020 — Bounded and efficient operation

Use bounded event/request queues, on-demand content, paged/filterable machine output and streaming subscriptions. Prefer one shared metadata subscription per Core connection, not per window. Avoid hidden terminal emulators unless genuinely required; reattach for fresh paint rather than lossy byte dropping. Keep UI painting and blocking I/O separated. Default operation must not require model calls, embeddings, summarization services or screenshot/OCR loops. [D03, D18, D47; S02, S09, S11]

### SP-021 — Evidence-bearing diagnostics

Retain bounded diagnostics and troubleshooting. Correlate client actions with exact targets, connection epoch and available backend observations; redact secrets and avoid default transcript/prompt logging. Explain unsupported, permission-denied, stale, disconnected and outcome-unknown states separately. A live JSON view must reveal its coverage, source and freshness. [D47]

### SP-022 — Complete but safe branding

Rebrand visible application identity, executables, bundle/package IDs, local IPC/state paths, icons, URLs, update manifests and release channels. Preserve licenses/attribution and external protocol identifiers required for compatibility. Do not reuse upstream update keys/endpoints, access another app's credentials, claim a domain/account or mutate upstream user state. Preserve the profile shell without fake authentication. [D25–D26, D50–D53]

### SP-023 — Independent release engineering

Keep automatic Spaceport updates, with explicit Spaceport-controlled signing and deployment configuration. No public release until identifiers, artifacts, update signatures and all supported platforms are tested. Never treat development placeholders as working release infrastructure. Independent maintenance does not prohibit reviewing security fixes. [D26, D51–D53]

### SP-024 — Agent-ergonomic repository

Maintain a small linked specification, exact source/capability evidence, task-sized context packets, a single task graph and traceable acceptance scenarios. Each module has an owner, input/output contract, side-effect boundary and focused tests. A useful discovery becomes a regression fixture, corrected contract or concise rationale. Do not build an autonomous learning/orchestration platform to manage the fork.

### SP-025 — Preservation and migration safety

Operate only in a confirmed fork checkout and isolated test environment. Do not edit Herdr, install on remote hosts, change the user's machines/credentials, migrate application state or publish builds without the relevant authorization. Old product settings are not silently imported. When authorized migration is implemented, it is copy-based, explicit and limited to compatible presentation preferences; excluded settings and credentials stay behind.

### SP-026 — Honest completion

Record all retained-feature/backend gaps and distinguish specification checks from application tests. Do not declare full desktop parity while retained remote chat, forwarding, transfer or other required paths remain unproved. Complete independent work when a gate is blocked. A reduced release requires an explicit product-scope change, not a hidden capability flag or a green compile.

## Definition of done

The retained feature matrix is demonstrated on the supported desktop platforms; excluded behavior/dependencies are absent; Core owns all runtime and remote connectivity; agent and human control paths agree; lifecycle and adverse scenarios pass; release identity and updates are verified; docs/task/capability evidence reflect the shipping commit. Every exception is visible and approved, not silently reclassified as optional.
