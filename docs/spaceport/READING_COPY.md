# Spaceport — complete specification reading copy

**Generated. Edit the component documents and registries, then run `python tools/spec.py render`.**

This is a specification and handoff, not an application implementation or runtime test report.

---

<!-- Source: START_HERE.md -->

# Spaceport — fork specification and coding-agent handoff

**Revision 1.0 · 7 November 2026 · Desktop implementation specification.**

Build **Spaceport**, an independent, fully rebranded fork of Sheprd (formerly herdr-gpui). Keep the desktop experience the user likes. Remove the unwanted product surfaces and backend duplication. Consume a separately installed **Herdr Core**; do not replace it.

This package is a specification plus working **specification-navigation/validation tools**. It is not a fork, an application implementation, or evidence of successful runtime integration. No remote repository has been changed.

## Start here, not with the entire repository

1. Read [AGENTS.md](AGENTS.md), then [SPEC.md](SPEC.md) and [SYSTEM.md](SYSTEM.md).
2. Install the specification-tool dependency as described in [tools/README.md](tools/README.md). Run `python tools/spec.py check`, `python tools/spec.py ready`, then `python tools/spec.py context T00`.
3. Follow [the delivery plan](docs/04-delivery-plan.md). Complete the source/capability baseline before bulk deletion.

[BOOTSTRAP_PROMPT.md](BOOTSTRAP_PROMPT.md) is the prompt to give the implementing agent. `READING_COPY.md` is a generated combined reading copy, not another source of truth.

## The whole system in one paragraph

Herdr owns live terminals, machine connections and execution. Host-side agent programs own their conversations and agent-specific behavior. Spaceport owns views, terminal emulation, input translation, presentation metadata and desktop integration. A thin Core adapter produces one inspectable, explicitly versioned projection. The GUI and existing local CLI use the same command semantics. Every change is linked to a user decision, acceptance scenario and source revision. Useful discoveries become a test, a small contract or a corrected document—not another service or an ever-growing prompt.

## Placement in the fork

Keep this package intact under `docs/spaceport/` in the new fork, or at one explicit external specification root while bootstrapping. Run package tools from that root; their `--root` option also accepts it. Candidate code paths in the deletion/source inventories are relative to the **application repository**, not the package. Add a short pointer to the fork's root `AGENTS.md` while preserving applicable upstream engineering rules; do not overwrite that file blindly. The package is not a substitute for reading the actual source.

## Document authority

| Artifact | Owns |
|---|---|
| `plan/decisions.json` | Historical user decisions and their qualifications; generated view in `docs/DECISIONS.md` |
| `SPEC.md` | Product requirements, identified by `SP-xxx` |
| `SYSTEM.md`, `docs/01-*`, `docs/02-*`, `docs/06-*` | System boundaries and implementation contracts |
| `plan/tasks.json` | Work graph and task status; no parallel handwritten progress tracker |
| `plan/acceptance.json` | Planned scenarios and their actual execution status; initially all `not_run` |
| `plan/capabilities.json` | Source-observed backing mechanisms and unresolved compatibility gates |
| `plan/removals.json` | Feature disposition and candidate code/dependency roots |
| `plan/sources.json` | Exact upstream evidence revisions; not deployment pins |
| `evidence/validation.json` | Checks actually run on this specification package |

User decisions outrank implementation convenience. Source establishes what exists, not what the user wants. A proposal, available method, successful compile, and verified behavior are four different states.

## Current runtime policy

D58 permits Herdr 1 alongside Core and supersedes D55. Preserve the working desktop UX and exact runtime ownership. The Core-specific corrections below still apply to Core routes; they do not require removal of Herdr 1. See [decision reconciliation](docs/07-reconciliation.md).

## Three corrections carried into the plan

**Remote Core support exists.** Current Core documentation assigns SSH connections, reconnects, and terminal/event aggregation to the local Core. The old “Core is local-only” premise described Sheprd's integration, not Core's actual capability. [S01, S02]

**Core is not the old workspace daemon.** Sheprd's session-model document distinguishes Core terminals from the older daemon's workspaces/tabs/agents. Workspace views need an explicit presentation model, not invented `workspace.*` RPCs. [S04]

**Rich chat is not a terminal engine feature.** It uses a host-side adapter. Sheprd documents local native chat attachment for Core, with remote Core terminals retaining their terminal view. Preserve the renderer, prove the route, and record the remote-chat gap rather than building another server. [S05, S06]

Sources are linked in [SOURCES.md](docs/SOURCES.md). The engineering resolutions are in [decision reconciliation](docs/07-reconciliation.md).


---

<!-- Source: AGENTS.md -->

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


---

<!-- Source: SPEC.md -->

# Spaceport product specification

**Status:** implementation requirements, with explicit backend and release gates. Not a claim that the application exists.

`MUST` denotes an acceptance requirement. `SHOULD` permits a documented equivalent. `MAY` is optional, not backlog authorization. User decision IDs are in `plan/decisions.json`. Technical source IDs are in `docs/SOURCES.md`. Engineering refinements below are design choices, not claims about existing upstream behavior.

## Product contract

Spaceport is a native desktop workspace for interacting with agents and terminals running in Herdr 1 or Herdr Core. Preserve the approved desktop experience while removing excluded features. Decision D58 permits Herdr 1 and supersedes the earlier Core-only requirement.

### SP-001 — Product and platform boundary

Deliver macOS, Windows and Linux desktop clients under the **Spaceport** identity. Preserve GPUI and the existing terminal implementation. Defer iOS, Android, UniFFI and Ghostty evaluation. Do not create speculative mobile crates or compatibility layers. [D20, D39, D51–D53]

### SP-002 — Preserve the approved desktop experience

Retain workspaces-first navigation, chat and terminal as equal first-class views, existing conversation organization, sidebar/status display, windows/tabs/splits, pane interactions, layouts, themes, fonts, keyboard shortcuts, menus, settings, profile shell and command palette. Remove only excluded feature surfaces and make necessary backend substitutions. Preserve accessibility and internationalization in retained UI. Use baseline fixtures/screenshots for comparison, excluding the explicit rebrand and removed controls. [D01, D03, D08–D09, D16, D20–D21, D28–D29, D38–D46, D50]

### SP-003 — Runtime ownership

The selected Herdr runtime owns terminal processes and execution lifecycle. Core owns machine connections for Core routes; existing Herdr 1 connection infrastructure may serve Herdr 1 routes. Host-side agents/adapters own their conversation execution. Spaceport may display state and invoke supported operations, not schedule agents, coordinate teams, migrate execution, configure providers or infer runtime state from screen text. User-requested launches go through the selected runtime. Do not add another transport manager. [D11, D13, D23, D27, D32, D48, D56, D58]

### SP-004 — Runtime compatibility

Retain Herdr 1 and Core adapters where they preserve approved behavior. Pin and test each supported runtime/protocol range independently of the app's release version. Record runtime identity, installed version, advertised methods/features where available, and tested source/schema evidence. Core protocol generation 1 is distinct from the Herdr 1 product. Unsupported capabilities fail explicitly; never move or recreate a session in another runtime as a fallback. [D58; S02, S03, S04, S09]

### SP-005 — Installation and startup

Use separately installed Herdr 1 or `hc`/Herdr Core. Connect first; start the installed local runtime with its documented startup command only when necessary. Existing Herdr 1 plain startup is allowed. Explicit sockets remain attach-only. Do not bundle/download/install/update either runtime locally or remotely, run installation scripts, or select an unverified executable from a project directory. Guided onboarding explains missing/incompatible software and supports manual machine setup. [D24–D25, D37, D58]

### SP-006 — Host connections and Tailscale

For Core routes, use Core's machine catalog, identity, SSH/custom-connection handling, reconnect policy and aggregated events. Existing Herdr 1 host connections may remain for Herdr 1 sessions. Keep the host manager's runtime ownership explicit; do not create a new machine registry or reconnect manager. Tailscale is externally installed/configured; addresses and aliases are entered manually. No discovery, embedded VPN, OAuth, cloud-provider provisioning or public control listener. [D06, D15, D40, D48, D56, D58; S01]

### SP-007 — Workspace presentation, not a new runtime

Preserve workspace create/rename/open/archive/delete controls. Herdr 1 may supply its native workspace semantics; Core does not. For Core routes, store shared grouping metadata in a Spaceport-namespaced, versioned Core document using `doc.get`/`doc.set`. Keep per-device geometry/preferences local and never write another client's document namespace. The selected runtime owns terminal lifecycle. Archive/delete presentation grouping must not implicitly kill processes or delete project files. A separately explicit termination action uses the selected runtime and existing confirmation behavior. [D01, D04, D14, D36, D58; S02, S04]

### SP-008 — Full terminal behavior

Keep current emulation, cell model, rendering, input encoding, selection, copy/paste, scrollback, mouse behavior, splits and supported observer views. Retain the Herdr 1 cell path and Core's authoritative first paint and ordered byte stream; obey each protocol's bounds, backpressure and negotiation. Do not invent shell state from screenshots. A terminal can exist without an agent or native chat. [D03, D39, D43, D58; S02, S09, S11]

### SP-009 — Native chat without ownership creep

Keep rich rendering and the existing host-side adapter/attach contract where verified compatible. Starting a host-side adapter is a selected-runtime terminal launch, not a desktop-owned agent child. Showing/hiding a chat is presentation-only: no kill/resume/convert side effects. Do not expose model/provider configuration or automatically deploy helpers. Prove local and remote routes for each supported runtime separately. A remote-chat gap must not be hidden by a new Spaceport server or by labeling terminal-only support as full native-chat parity. [D08, D23, D27–D28, D32, D56, D58; S05, S06]

### SP-010 — Capability-correct interaction

Offer prompt composition, attachments, permission answers, queues and slash-command affordances only when their backing session supports them. Keep permission prompts distinct from ordinary messages and descriptive tool-call output. Unknown agent status is unknown, not idle. Input targets the exact runtime/terminal/conversation instance the user or caller selected. [D11, D23, D28, D32, D58]

### SP-011 — Activity, search and monitoring

Preserve the current activity timeline rather than adding a new navigation model or history service. Search only the data available in that retained timeline. Keep supported agent status, usage/rate-limit reporting and read-only automation visibility. Remove CPU/GPU/memory dashboards, provider polling/sign-in subsystems, automation execution/editing and repository indexing. Unavailable information is not zero, no history, or “no jobs.” [D07, D12, D17–D19, D49]

### SP-012 — Notifications and lifecycle

Retain desktop OS notifications, attention markers, badges and sounds, configurable using existing preferences. Last-window close quits Spaceport; no tray daemon or mobile push system remains. Herdr terminals survive disconnect and app quit. Do not promise notifications after the client exits. Persist view intent, not agent execution recipes; restoring a view never restarts an absent agent. [D10, D29, D33, D36, D57]

### SP-013 — Online-only semantics

No persistent offline transcript/activity/terminal cache and no offline mutation queue. After disconnection, disable backend mutations and clearly mark or clear previously observed content. In-memory stale content must never be presented as live. Durable preferences, layout references, safe diagnostic metadata and Core-owned documents do not constitute an offline execution/content feature. Reconnection starts from a fresh authoritative snapshot. [D18, D36]

### SP-014 — Files and services

Retain file/image attachments, drag/drop, existing general transfer behavior and external file opening through verified runtime routes. For Core, prefer `terminal.put` and file/machine facilities; do not assume text file APIs provide every binary-transfer operation. Existing Herdr 1 routes may satisfy these features when separately tested. Keep port discovery/forwarding as an explicit requirement and open URLs in the system browser. Do not expose servers publicly, alter bind addresses or silently configure Tailscale Serve to hide a missing route. Record parity blockers per runtime. [D05, D22, D44, D58; S01, S03, S09]

### SP-015 — Complete feature deletion

Delete IDE/editor integration, code navigation/indexing, specialized diff/review UI, Git/worktree/branch/checkpoint controls, GitHub account/PR/issue/security surfaces, embedded browser/annotations, cloud providers/provisioning, Teleport/migration, orchestration/dispatch engines, automation editing/execution, client mods/plugins, terminal-to-agent annotations and mobile remote-control/pairing/push components. Herdr 1 itself is retained under D58; this does not restore any excluded feature carried by that backend. Delete excluded settings, assets, CLI commands, workers, dependencies, tests and documentation after extracting shared retained primitives. No disabled excluded implementations shipped behind flags. [D02, D04–D07, D13, D27, D30–D31, D34, D54, D57–D58]

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

Rebrand visible application identity, executables, bundle/package IDs, local IPC/state paths, icons, URLs, update manifests and release channels. The About view explicitly credits both the Herdr and Sheprd teams. Preserve licenses/attribution and external protocol identifiers required for compatibility. Do not reuse upstream update keys/endpoints, access another app's credentials, claim a domain/account or mutate upstream user state. Preserve the profile shell without fake authentication. [D25–D26, D50–D53, D59]

### SP-023 — Independent release engineering

Keep automatic Spaceport updates, with explicit Spaceport-controlled signing and deployment configuration. D60 removes the inherited custom updater; replacement platform/package update routes remain a release requirement and must be verified before enabling them. No public release until identifiers, artifacts, update signatures and all supported platforms are tested. Never treat development placeholders as working release infrastructure. Independent maintenance does not prohibit reviewing security fixes. [D26, D51–D53, D60]

### SP-024 — Agent-ergonomic repository

Maintain a small linked specification, exact source/capability evidence, task-sized context packets, a single task graph and traceable acceptance scenarios. Each module has an owner, input/output contract, side-effect boundary and focused tests. A useful discovery becomes a regression fixture, corrected contract or concise rationale. Do not build an autonomous learning/orchestration platform to manage the fork.

### SP-025 — Preservation and migration safety

Operate only in a confirmed fork checkout and isolated test environment. Do not edit Herdr, install on remote hosts, change the user's machines/credentials, migrate application state or publish builds without the relevant authorization. Old product settings are not silently imported. When authorized migration is implemented, it is copy-based, explicit and limited to compatible presentation preferences; excluded settings and credentials stay behind.

### SP-026 — Honest completion

Record all retained-feature/backend gaps and distinguish specification checks from application tests. Do not declare full desktop parity while retained remote chat, forwarding, transfer or other required paths remain unproved. Complete independent work when a gate is blocked. A reduced release requires an explicit product-scope change, not a hidden capability flag or a green compile.

## Definition of done

The retained feature matrix is demonstrated on the supported desktop platforms; excluded behavior/dependencies are absent; Herdr 1 or Core owns each session's execution through its verified route; agent and human control paths agree; lifecycle and adverse scenarios pass; release identity and updates are verified; docs/task/capability evidence reflect the shipping commit. Every exception is visible and approved, not silently reclassified as optional.


---

<!-- Source: SYSTEM.md -->

# Spaceport as one system

Decision D58 permits Herdr 1 alongside Core. The Core topology below describes Core routes. Existing Herdr 1 cell/snapshot, workspace and host-connection paths may remain for Herdr 1 sessions. Every operation retains its exact runtime target; this does not authorize a new transport manager, session migration or runtime installation. Core-specific capability gaps remain scoped to Core until another supported route is separately proved.

## 1. Organizing idea: an observable client, not a second brain

Spaceport should give an operator—human or agent—the same reliable loop:

**Observe the right state → understand what is supported → target precisely → request an effect → observe what actually happened.**

“Agent-intuitive” means the system describes itself in domain terms. “Agent-ergonomic” means useful evidence and actions are available without screen scraping or enormous context dumps. “Agent-accretive” means each debugging or implementation pass leaves durable, tested understanding in the repository. It does not mean adding persistent runtime memory, an agent scheduler or autonomous self-modification to the product.

The central invariant is **one authority per fact, one owner per side effect, multiple views of the same semantics**. Two clients can render the same terminal without owning two processes. Two windows can display a conversation without maintaining independent permission state. A CLI can select a pane without synthesizing keystrokes against a guessed window.

## 2. Tower of abstractions

```text
Human desktop interaction          Coding/operating agent
        |                         Spaceport CLI for views
        |                         hc CLI for runtime-native work
        +------------+------------+
                     |
      Retained GPUI UI / private local control IPC
                     |
      One command catalog + explicit targets/outcomes
                     |
      Pure session/view projection + content references
             /                    \
  Terminal emulator / painter   Verified chat reducer/view
             \                    /
      Existing Core client adapter + capability mapping
                     |
          Installed local Herdr Core
          /          |             \
   Local terminals   Core documents  Core-owned machine connections
                                          |
                               SSH over externally managed Tailscale
                                          |
                              Remote Core / terminal / agent
```

The chat branch is **capability-gated**. Its existing host-side attach protocol is not magically provided by Core's terminal stream. Prove the route before enabling it; never smuggle a second remote server into the drawing. [S05, S06]

### Layer A — External authority

Each selected runtime owns its processes and lifecycle. Core owns machine registrations, terminal streams, program-reported status and supported file/document operations for Core sessions. Herdr 1 supplies its existing native workspace, snapshot and terminal contracts for Herdr 1 sessions. Agent programs own agent sessions, models, permissions and conversation state. Tailscale owns network membership/routing. Spaceport does not duplicate these policies. [D58; S01–S04, S09]

### Layer B — Narrow Core adapter

Reuse and extend `sheprd-core-client` to call the methods the selected Core actually advertises. Implement local socket/named-pipe and framed stream correctness once. Core itself carries remote machine traffic. The adapter exposes typed facts, stream subscriptions and effect results, not GPUI entities. No provider-specific cloud adapters or a generalized plugin transport.

### Layer C — Session and view projection

Reuse existing session infrastructure and retain `ClientShell*` snapshots for Herdr 1 views. Use Core projections for Core views. Produce immutable or coherently published view revisions without mixing runtime identities. A reducer applies backend facts and produces view changes; it does not launch processes.

Store presentation grouping in a namespaced Core document; maintain local window geometry/preferences separately. Do not create a workspace database or treat a local list as authoritative remote execution state. Terminal and document reads have different versions; the combined view must expose that it is a **composite observation**, not an atomic distributed snapshot.

### Layer D — Command semantics

A single typed catalog declares each retained client action's owner, arguments, target kind, required capability, safety class and observable postcondition. All surfaces call it. An action executes through either the local presentation owner or the verified backend adapter, never both implementations independently. The catalog is not a generic RPC proxy exposing every Core method.

### Layer E — Presentation and automation surfaces

Keep GPUI and existing interactions. The private CLI/IPC exposes concise state and client controls, not a new runtime. Agents use `hc` directly for runtime functions. Native notifications, external browser opening and platform dialogs live at this boundary.

Logical layers are responsibility boundaries, not a mandate for a new crate per box. Reuse current modules; extract only where a concrete dependency or test boundary justifies it. [S08–S10]

## 3. Ownership matrix

| Fact or operation | Authority | Spaceport role |
|---|---|---|
| Process/terminal existence and termination | Selected Herdr runtime | Inspect, render, send an explicit supported request |
| Remote SSH catalog and retry policy | Core for Core routes; existing Herdr 1 connector for Herdr 1 routes | Use the selected runtime's established infrastructure; no new manager |
| Agent conversation, queues, model and approval policy | Agent/verified host adapter | Render and forward supported user interaction |
| Workspace grouping metadata | Native Herdr 1 workspace or Spaceport schema stored by Core `doc.*` | Keep runtime identity explicit; grouping changes do not implicitly terminate sessions |
| Window position, theme, selected view | Spaceport local preferences | Persist/restore presentation intent |
| Event observation and notification deduplication | Reconstructable client projection | Bounded in-memory processing, no history warehouse |
| Terminal cell state | Herdr 1 cell protocol or pure emulator from Core stream | Preserve the selected path and rebuild when appropriate |
| Existing service forward | Backend-supported route | Show state and open externally; gate absent route |
| User/provider credentials | Existing OS/runtime/agent owners | Do not import or create a new credential subsystem |
| Spaceport release/update identity | Spaceport release configuration | Sign and update only Spaceport |

## 4. Identity is not a label

A runtime reference identifies its backend before naming a session. Core references include the selected **Core identity**, machine route/identity when available, backend terminal ID/address and client connection epoch. Herdr 1 references retain the exact connection target, daemon boot and workspace/tab/pane identity. Names are display hints. A persistent Core ID is not a boot ID. Core event sequences restart with a process, so every reconnection invalidates sequence continuity and obtains a fresh snapshot. [D58; S02, S09, S12]

Use a client-generated connection epoch as an observation fence, not as invented backend authority. Keep native identity fields intact. If a remote identity cannot be established, expose the narrower identity guarantee and reject unsafe ambiguous automation. A machine alias change must not silently redirect a queued or delayed action.

View IDs live in a separate namespace from terminal IDs. Conversation IDs and pending approval IDs come from their source. Do not derive persistent identity by hashing a title, filesystem path, PID alone or screen text.

## 5. State transitions and uncertainty

The Core connection moves through `disconnected → connecting → negotiating → synchronizing → ready`; incompatible schema/capabilities and authentication errors are distinct terminal/recoverable states. A disconnect invalidates mutation eligibility immediately. A reconnect is not proof that an old terminal instance still exists.

Events are applied only within the current epoch. Follow Core's snapshot/`missed` semantics; a gap triggers refresh rather than fabricated event replay. Refresh watched documents after subscribing, reconnecting and `missed`, because terminal snapshots do not contain document values. [S02]

A terminal's process state, reported agent status, connection reachability and UI focus are separate dimensions. `done` is not process exit; unreachable is not terminated; no reported status is not idle; foreground process data is not a reliable conversation ID. Do not collapse them into a misleading green/red Boolean.

## 6. Action path

```text
intent + explicit target
  → capability and identity check
  → required confirmation / operation-specific preconditions
  → one typed dispatch
  → submitted or refused
  → backend acknowledgement, if available
  → observed postcondition, or explicitly unknown outcome
```

A display action can often be confirmed locally. A process launch needs evidence from its selected runtime. A prompt send may only prove transport acceptance, not model execution. A timed-out mutation is not retried automatically unless the actual protocol guarantees idempotency for that operation. Do not implement an “exactly once” claim with a client UUID.

For workspace metadata, Core's document version is a real CAS boundary. For terminal actions, a locally checked revision is not an atomic remote precondition. Report the difference; do not advertise capabilities the server does not implement.

## 7. Multi-operation partial outcomes

Launching a terminal and placing its reference in a workspace are separate effects. If launch succeeds and the document write conflicts, the terminal remains visible as ungrouped. Reconcile membership; do not kill the new process as an automatic rollback. Conversely, a failed launch must not leave a fabricated running pane.

Deleting a workspace presentation record never means deleting files or terminating all member agents. Explicit termination is separately targeted and confirmed. Renaming a machine and reconnecting may change addresses; reconcile using authoritative identity rather than stale aliases.

## 8. Resource economy

Keep summaries cheap and content explicit. List metadata without loading transcripts. Fetch tool output and large terminal text only when requested. Share subscriptions across windows; render only visible rows and damaged terminal regions. A hidden view should not continuously parse everything merely to populate a badge.

Credits are returned according to actual consumption, not blindly on arrival into another unbounded queue. Control traffic must remain responsive when a terminal floods output. Reuse Core's fresh-paint behavior rather than dropping arbitrary VT bytes. Preserve Unicode/input semantics; speed does not justify corrupting terminal state. [S02, S11]

## 9. The repository is the agent's durable working memory

The useful linked chain is:

**User decision → requirement → source/capability evidence → task → code change → acceptance scenario → result.**

Each edge has a stable ID and a real file reference. `tools/spec.py` validates the graph and produces a bounded task packet. A regression becomes a fixture/test and a short contract note. There is no vector database, knowledge-graph server, prompt-injection-prone “memory ingestion” service or additional task orchestrator.

Scope changes require explicit user authority. Engineering fixes may improve legibility, tests and module boundaries without reopening settled product choices. A failing capability gate is information to preserve, not an incentive to build more product.


---

<!-- Source: docs/01-herdr-contract.md -->

# Herdr integration contract

Under D58, Herdr 1 is also allowed. This document's method map and machine-hub rules describe Core routes. Preserve the existing Herdr 1 adapter for Herdr 1 sessions and verify its capabilities separately. Do not route a Core session through Herdr 1 to hide an unsupported operation, or claim Core parity from a Herdr 1 test.

Implements SP-003–SP-010, SP-013–SP-014 and SP-018–SP-019. Sources: [S01–S06, S10–S12](docs/SOURCES.md).

## 1. Establish capability, not wishful compatibility

The implementing agent must save an isolated runtime's version, protocol generation, `hello.methods`, `hello.features` and `rpc.discover` schema alongside the app/source revision. The source pins in this pack are investigation baselines; neither is a proven release support promise. Inspect the installed runtime instead of assuming `main`, a release tag and the user's installed binary behave identically.

The adapter maps a typed operation to an actual advertised method. Unsupported, authorization failure, missing agent helper and disconnected are separate outcomes. An advertised method is evidence of shape, not proof of complete semantics on every relay/path. Per-attach confirmations matter for features such as observer resizing. A capability descriptor includes evidence level (`source_observed`, `runtime_advertised`, `scenario_verified`), scope, source revision and platform coverage.

### Source-backed operation map

| Requirement | Actual Core mechanism observed | Important limitation |
|---|---|---|
| Negotiate | `hello`, `rpc.discover` | Core product generation and wire generation are different concepts |
| Observe terminals | `terminal.list`, `terminal.get`, `events.subscribe` | Reconnect requires a new snapshot; no durable replay assumption |
| Start a program/template | `terminal.run`, `templates.list` | A timed-out run may have started; no automatic second launch |
| Render/interact | `terminal.attach`; framed DATA/RESIZE/FOCUS/FACTS/CREDIT/CLOSE | Attach is not an ACP conversation channel |
| Programmatic interaction | `terminal.send`, `terminal.keys`, `terminal.interrupt`, `terminal.wait` | Preserve backend readiness/quiet/blocked semantics; do not emulate them locally |
| Terminate/remove | `terminal.kill`, `terminal.rm` | Killing a live process is not closing a view |
| Manage remote machine | `machine.add/list/connect/rename/rm` | Core owns the connection; machine removal does not mean killing remote programs |
| Attach an image/file | `terminal.put` | Writes bytes, but does not itself type a path or submit input |
| Files/short commands | `file.read/write/edit`, `machine.exec` | Text file APIs and collected exec output are not a generic persistent bidirectional transport |
| Presentation documents | `doc.get`, `doc.set` | Opaque JSON, versioned CAS; Core does not implement workspace semantics |
| Rich agent chat | Host-side adapter contract | No native `chat.*` Core API established in inspected method table |
| Service port forwarding | No dedicated method in inspected table | Explicit parity gate, not permission to create a parallel SSH manager |

The table is sourced from the inspected Core method registry, not proposed endpoint names. Preserve upstream generated types where practical; hand-maintained copies need schema-conformance tests. [S03]

## 2. Connect locally; let Core connect remotely

The normal desktop topology is one installed local Core acting as the machine hub. Spaceport opens its local Unix socket or Windows named pipe (or the supported local connection mechanism). Core manages remote hosts, including addresses reachable through the user's existing Tailscale network. Do not run one Spaceport SSH loop per machine.

Adding a machine must use the supported non-installing path. A missing/incompatible remote Core is reported, not repaired by copying a binary, running `--install`, calling `machine update` or executing a fetched installer. Password/MFA/host-key interaction remains in the established SSH/Core workflow; no hidden approval or key import. Known-host mismatches are hard refusals.

`machine.list` may need a bounded refresh distinct from terminal events; verify the actual schema/event surface. Polling a supported read is not ownership of reconnect policy. Never implement reconnect by starting agents again.

## 3. Event and attach correctness

Subscribe before reading documents. Apply the initial terminal snapshot and subsequent events within one connection epoch. Core's `missed` notification replaces the applicable terminal snapshot; then refetch watched documents. Do not infer missing historical events or retain old sequence numbers after reconnect. Preserve per-source order; a merged multi-host timeline is not a universal causal clock. [S02]

For each terminal attachment, complete negotiation before processing its first paint. Keep stream bytes ordered even across arbitrary chunk boundaries. Follow credit accounting and bound queues. A slow terminal must not starve metadata or command responses. On detach close the channel, not the terminal. Only resend input when the caller explicitly issues a new operation and understands the previous outcome.

Preserve observer behavior. An observer must not send keys or seize terminal size ownership. If a negotiated resize notification arrives, process it before subsequent data. Do not emit new automatic terminal query replies unless the retained engine needs them; when emitted, use the backend's confirmed tagging/fallback rules, not byte-content heuristics. [S02, S11]

## 4. Workspace metadata contract — proposed design

This is the main ownership correction to the early Q&A. Core does not supply the old daemon's workspace model. Spaceport therefore defines **view grouping**, not a new execution model. [S04]

Use a namespaced document named `spaceport` with an explicit schema version. Its proposed contents are stable workspace IDs, labels, optional archived flag, explicit launch-location defaults, and references to Core terminal identities. Shared tab/pane grouping may live there when required for cross-window consistency; pixel geometry, font sizes and per-device selections remain local preferences. Do not store process status, transcripts, agent credentials, runnable restore recipes, queues or duplicate machine catalogs.

All shared edits use `doc.get` and version-checked `doc.set`. Validate known fields, retain compatible unknown fields when safe, and refuse unsupported newer document schemas rather than rewriting them. Keep documents within the observed Core size limit; pagination/sharding is not a reason to add an unneeded storage framework. Never overwrite `herdr-app` or other clients' documents. An explicit import, if later authorized, is read-and-copy with provenance, not shared ownership of another client's schema.

Workspace create/rename/archive/delete affects grouping metadata only. A terminal may have zero, one or multiple views. Restore resolves each stored terminal reference against current authoritative state. Missing terminals render as unavailable/closed references according to the retained UX, never as instructions to restart a program. Launch context defaults are used only for an explicit new launch.

A cross-host workspace is an ordered group of qualified references. Duplicate human-readable names remain distinguishable by stable IDs and host labels. Remote paths are interpreted on the named machine; never canonicalize them against the local filesystem.

### Concurrency

Two writers read document version 8. Writer A's rename commits version 9. Writer B's stale tab insertion conflicts. B rereads, revalidates its intent and may apply a nonconflicting edit to 9; it must not overwrite A's rename or treat its stale document as authoritative. Limit automatic conflict retries. If intent cannot be preserved, return a conflict and current state.

Starting a terminal and assigning it to a workspace is non-atomic. If assignment fails, show the new terminal as ungrouped and return a partial result. Never compensate by killing a successfully launched agent.

## 5. Rich chat: preserve the renderer, verify the channel

Sheprd's host-side adapter owns the structured agent conversation and is kept alive as a terminal program; the GUI attaches to it. The Core chat path labels the terminal with an endpoint identifier. The inspected docs explicitly restrict native Core chat attachment to local terminals. [S05, S06]

Retain the pure chat model/reducer and native renderer. Retain only host-side adapter behavior necessary for supported chat, with the selected Herdr runtime owning its process. Do not let the desktop resume/restart agents automatically, convert terminal agents by killing them, manage model/provider credentials or deploy adapters to remote hosts. Such old paths are incompatible with the thin-client boundary.

Local IPC to an already running host adapter is an existing compatibility dependency, not proof that Core routes native chat remotely. Inventory it explicitly. The target is a verified Herdr-owned route; a separate Spaceport SSH relay, HTTP/WebSocket service or mobile server is forbidden as an unapproved workaround. Do not pass binary chat traffic through an emulated terminal and assume a PTY is a transparent stream.

Gate **G-CHAT** requires an exact discovery, identity, authentication, transport and permission-response contract, with reconnect and concurrent-view tests. Terminal fallback keeps a session usable but does not satisfy a native-chat parity claim.

## 6. Services, files, automation and usage

For files, prefer supported Core APIs and preserve the distinction between transferring bytes and submitting their path as input. General binary upload/download, overwrite behavior, symlink handling, temporary-file lifetime and cancellation must be checked separately; the existence of `file.read` does not prove SFTP-equivalent parity.

For services, keep port discovery/opening UI and the explicit forwarding requirement. Opening an already reachable Tailscale URL is not equivalent to forwarding a remote loopback-only service. Do not widen a service's bind address or create a separate host connection manager. If Core lacks the needed forwarding primitive, record G-PORT and a minimal external contract request; do not implement a new Core feature in this repository.

For automation and usage, the inspected Sheprd subsystems contain their own schedulers/provider probes. Removing those does not create an equivalent Core API. Retain passive displays only when an authoritative, supported feed exists. No host script scanning, provider credential reads or local schedule engine should reappear under “visibility.” Gate G-OBS records the actual supported subset.

## 7. Gap closure evidence

A gate closes with a pinned method/schema or documented adapter contract **and** a reproducible scenario showing the desired effect on the relevant platform/topology. A README statement, source symbol or mock alone cannot close runtime parity. If a missing capability belongs in Herdr, write an external dependency note, not an incidental upstream patch. Continue unaffected work and report the blocked retained behavior plainly.


---

<!-- Source: docs/02-agent-interface.md -->

# Agent-facing client contract

D58 permits both Herdr 1 and Core. Core-specific examples below retain their scope. Existing Herdr 1 control commands keep exact daemon/boot/workspace/pane identity; do not translate them into a different runtime or infer equivalent capability without tests.

Implements SP-017–SP-021 and SP-024. This is an **additive refinement of the retained local CLI**, not a new product daemon or a replacement for `hc`. Existing relevant commands keep their semantics after removed features disappear. Source baseline: [S07, S09](docs/SOURCES.md).

## 1. Two tools with distinct jobs

An operating agent uses **`hc` for runtime truth and effects**: terminals, program input, machine management and supported files. It uses **`spaceport` for the client**: windows, view selection, layout, presentation settings, diagnostics and inspecting what the user can currently see. Do not force a backend operation through the GUI merely to appear “agent-native.” Do not expose redundant independently implemented runtime commands.

The local control socket/named pipe exists only with the desktop process. Keep it private to the OS user. No TCP listener, network-accessible agent API, MCP server, plugin runtime or background tray process is part of this design. A remote agent without access to the local client control endpoint is not granted a new route by default.

## 2. Discover → inspect → act → observe

The following CLI spellings are **proposed**, not existing commands or runnable implementation in this pack. Reuse equivalent upstream verbs rather than adding aliases unnecessarily.

```sh
spaceport status --json
spaceport commands --json
spaceport inspect --window WINDOW_ID --fields identity,selection,capabilities --json
spaceport inspect --window WINDOW_ID --workspace WORKSPACE_ID --fields agents,attention --limit 20 --json
spaceport window focus WINDOW_ID --json
```

The operator should be able to answer: Which window/workspace am I targeting? Which Core and machine owns this terminal? Is the displayed state fresh? What action is actually available? What would it affect? What happened after submission? Which exact evidence supports that conclusion?

Do not require dumping full transcripts or taking screenshots to answer those questions. Screenshots remain useful for visual regression testing, not the primary control channel.

## 3. Command catalog

Maintain one typed catalog that drives the native action dispatcher, command palette/shortcut lookup, CLI help/schema and contract tests. Each entry names:

- stable action ID and local/backend owner;
- input and output type, target kind and selection rules;
- required capabilities and applicable platform/path;
- read-only, presentation-changing, input-affecting, destructive or external-open safety class;
- confirmation behavior, concurrency preconditions and retry semantics;
- what acknowledgement and observation can actually prove.

The catalog contains only retained client capabilities, not every Core RPC. Backend-specific constraints stay in the adapter; presentation never carries a second implementation of them. Generate descriptions from this catalog where useful. Do not maintain four manually synchronized command lists.

## 4. Typed observations

A structured observation contains a schema version, source identity, connection epoch, freshness, coverage, relevant version watermarks and explicitly selected content. Example fields in `contracts/control.schema.json` are a **seed design contract**; the implementation must adapt them to actual available identity fields.

`coverage` distinguishes complete-for-this-query, partial and unavailable. `has_more` and a continuation marker describe paging. `content_available=false` is not an empty transcript. `status=null` means no reported status, not idle. `redacted` and `truncated` are explicit. Include backend sequence, document version and chat revision separately when available; do not invent a globally atomic `revision`.

Default output is metadata and evidence references. Raw terminal output, full tool results, images and transcripts require an explicit target and bounded read. Same query against the same projection should serialize deterministically except for documented observation timestamps. Never expose credentials or private environment variables in generic inspection.

## 5. Targets and races

Mutation from automation must not default to a frontmost window when several could match. A display name is accepted only after resolving to one exact target and returning that target in the result; ambiguity is a structured refusal. Preserve human keyboard focus behavior for normal interaction, but snapshot its target at command dispatch so a later host switch cannot redirect input.

A proposed command request contains a request ID, action, qualified target, typed arguments and optional expected observation/version. These fields provide correlation and local stale-view detection. They do **not** create backend transactional fencing. Only backend-supported compare-and-set or instance checks can provide those guarantees. For an operation that needs stronger concurrency than available, refuse or require explicit interaction; do not claim safety from a local preflight alone.

Permission responses target the exact conversation instance and pending request ID. The user-approved content/destination must not change while a dialog is open. An agent cannot grant itself the user's confirmation by sending `approved: true`. Confirmation tokens, if needed by the retained implementation, must bind the actual target/action/content and expire; they are not general reusable authority.

## 6. Outcome semantics

| State | Meaning | Retry behavior |
|---|---|---|
| `rejected` | Dispatch did not happen | Correct the reason and issue a new request |
| `submitted` | Client queued/sent the request | Not proof of backend execution |
| `acknowledged` | Backing endpoint accepted/replied | Only the endpoint's documented guarantee |
| `observed` | Declared postcondition was observed | Report evidence and its scope |
| `unknown` | Request may have had an effect, but outcome cannot be established | Do not replay a non-idempotent operation |

Not all commands pass through all states. A local view focus can immediately be observed. Transport acceptance of terminal input cannot prove the agent acted. An error before send differs from disconnect after send. “Wait timed out” is not “the operation failed.”

Receipts remain bounded, short-lived in-process correlation data; no durable offline operation journal or execution queue is introduced. Logs may retain redacted diagnostic IDs, not a replayable job system. Safe reads and document CAS conflicts can be retried according to their actual semantics; input, launch, approval and arbitrary file writes must not be blindly replayed.

Preserve upstream CLI exit meanings where retained. New structured error kinds may distinguish `unsupported`, `disconnected`, `stale_target`, `ambiguous_target`, `confirmation_required`, `outcome_unknown` and `resource_limit`; validate names against the implementation catalog. Return recovery hints as typed suggestions or clearly quoted text, never automatically executable shell fragments from untrusted data.

## 7. Cheap observation and event consumption

Use subscriptions/waits for supported live state instead of tight polling. Share backend subscriptions between GUI windows and CLI readers. Repeated state reads should not trigger provider network probes or transcript reparsing. Set explicit default output/retention bounds; return page metadata rather than silently dropping rows.

Recommended initial **engineering budgets**, to be measured and adjusted with evidence: 50 metadata rows/page; 64 KiB default structured response; 256 KiB maximum explicitly requested text page; one metadata subscription per Core connection; no full transcript or terminal scrollback in default inspection. These are proposed client budgets, not observed performance or Core protocol limits. Preserve stricter upstream validation/attachment limits where applicable.

## 8. Agent-accretive behavior belongs in development evidence

When a control ambiguity is found, improve the command contract and add the smallest failing fixture. When a stale-view race is found, add a deterministic reducer/dispatch test. When a dependency is removed, add an absence check. When an integration gap is found, record its exact source and required external primitive.

Do not build a runtime memory store, automated skill installer, recursive self-improvement loop or prompt summarizer to achieve this. The useful accumulated knowledge is executable tests and concise documentation at the owning module.


---

<!-- Source: docs/03-fork-and-deletion.md -->

# Fork, deletion and rebrand plan

Implements SP-001–SP-004 and SP-015–SP-016, SP-022–SP-026. Candidate roots are recorded in `plan/removals.json`; inspect the real transitive dependency graph before editing. The inventory is based on [S08, S09](docs/SOURCES.md), not a complete source audit.

## 1. Preserve before removing

Fork at an explicit Sheprd commit. Record the origin without promising routine merges. Build and exercise an isolated baseline. Capture retained views, command behavior, emulator fixtures, permissions, terminal lifetime and relevant platform packaging. Record any pre-existing failures instead of laundering them into successful tests.

Separate behavioral deletion from mechanical renaming. Retain working Herdr 1 and Core paths under D58; remove excluded feature reachability independently. A directory named `sheprd-client` mixes retained Herdr 1 code with useful helpers. A directory named `sheprd-protocol` also supplies Core rendering/input types. Preserve both runtime contracts and Core protocol generation 1. [D58; S02, S09, S11]

## 2. Deletion units

A deletion unit is a complete unwanted behavior: UI entry points, command catalog entries, settings/schema fields, background workers, source modules, assets/translations, dependencies, docs, fixtures and package/service integration. Do not count code as removed while it remains reachable from a hidden CLI, migration path or worker.

| Family | Candidate code roots | Treatment |
|---|---|---|
| Git/diffs/IDE/indexing | `sheprd-vcs`, relevant GPUI review/code/editor modules | Delete specialized behavior, retain plain chat/terminal text |
| GitHub and issue dispatch | `sheprd-github`, `sheprd-orchestrator` | Delete accounts/PR/issues/security/Beads dispatch; extract generic cache/file primitives only if independently necessary |
| Cloud and Teleport | `sheprd-remote` and GPUI provider UI | Remove cloud features, provider credentials/setup and migration; audit shared helpers before deleting whole crate |
| Automations/mods | `sheprd-automate`, `sheprd-automation`, `sheprd-luau`, `sheprd-mods` | Delete schedulers/execution/editing/runtime; retain only proven passive display types needed by an external feed |
| Embedded pages/annotations | GPUI browser module and webview-specific dependencies | Delete browser, IDE webviews and annotation handlers; retain ordinary system URL/file opening |
| Mobile service/push/pairing | `sheprd-remote-control`, `sheprd-apns`, `sheprd-push-relay`, `sheprd-nearby`, `ios/`, platform nearby helpers | Delete from the desktop fork/build graph; extract shared pure session/chat types before removal |
| Herdr 1 | Existing client/snapshot/bridge paths | Retain under D58; remove excluded feature routes independently |
| Provider usage/config | provider probes/sign-in inside `sheprd-usage`, UI model/provider settings | Delete client-owned collection/config; retain reported values from supported runtime/agent sources |
| Terminal feedback | terminal annotation/send-selection-to-agent paths | Delete; normal selection/copy/paste remains |

Negative requirements apply to **product code**, not to the engineering repository's ordinary Git history, GitHub Actions CI or tests mentioning removed names. Likewise, absence checks must not reject a user transcript containing `git diff`, a Rust dependency with an incidental name, or legally required attribution.

## 3. Retained code candidates

Reuse GPUI, terminal emulator, Core adapter, pure session and chat reducers, control-protocol/command catalog, diagnostics, i18n, updater and Windows launcher. Rename their package identities deliberately after behavior is stable. Avoid a top-to-bottom architecture rewrite.

`sheprd-acp` is a special case: retain the minimal verified host-side agent adapter contract necessary for native chat, not its desktop-owned resume/deployment or model/provider-management workflows. Its process belongs to Herdr. A compile-time dependency is not permission to keep excluded behavior.

Credentials are another mixed case: delete provider/account uses; retain only generic OS/private-endpoint primitives that a kept feature demonstrably needs. Do not broadly migrate credential stores. Upstream “report-only” automation features are not automatically compatible with a Core-owned data source.

## 4. Reachability and dependency proof

For each deletion record: inventory call sites and entry points; identify shared survivors; make the smallest extraction; remove behavior; update command/config/assets/docs; regenerate lockfile with the chosen toolchain; run retained-feature tests plus negative reachability tests. Verify the release dependency graph on all targets and feature combinations actually shipped.

Searches are supporting evidence, not sole proof. A string remaining in attribution can be correct; a worker without UI can still violate scope. Check build features, packaged binaries, launch agents/services, updater helpers and platform-only source. Use fake provider credentials/servers to prove removed components are never contacted, without real network side effects.

Delete old tests only when their tested product behavior is intentionally removed. Keep renderer, identity, permission and lifecycle regression fixtures even when their old file location disappears. Add representative tests proving excluded commands/configuration are rejected and obsolete background services are not installed.

## 5. Rebrand domains, not raw strings

**Product identities:** Spaceport display names, executables, app/package IDs, bundle resources, application state, local control endpoint, installer metadata, desktop entries, URL handlers, help, website references and update endpoints.

**Compatibility identities:** Herdr RPC method names/fields, `hc` invocation, protocol constants, agent ecosystem variables, existing host-side attach contract identifiers. Change these only with explicit versioned adapter compatibility evidence. Do not replace every occurrence of `herdr` or `acp`.

**Attribution:** Keep applicable license and copyright notices and document the fork origin. Do not claim trademark/domain availability or upstream endorsement.

An engineering placeholder application ID is allowed only in isolated development builds. Shipping owner-controlled IDs, signing accounts/keys, update channel and icons are release inputs. Unknown values are hard release gates, never silently inherited upstream identifiers. A signed Spaceport update must never install Sheprd or Herdr.

## 6. User state migration

Default to independent Spaceport state. Do not reuse the upstream state directory, mobile App Group, service names, keychain access group or updater identity. If preference migration is implemented, make it an explicit copy from an existing app, validate compatible fields, and exclude GitHub/cloud tokens, runtime recipes and removed settings. Leave the original untouched. No migration is required just to complete the code-removal task.

## 7. Completion evidence

Provide before/after dependency and packaged-binary inventories, retained scenario results, representative screenshots, removed command/settings checks, rebrand identity report and an unresolved-gate list. File count or binary-size reduction alone is not acceptance. A fork that compiles but quietly loses native chat, forwarding or terminal persistence is not finished.


---

<!-- Source: docs/04-delivery-plan.md -->

# Delivery plan

The canonical work graph is `plan/tasks.json`. Status starts at `not_started`; this document is an explanation, not a second tracker. Run `python tools/spec.py ready` and `python tools/spec.py context T00`.

## Phase 0 — Prove the boundary before cutting

**T00** pins source and toolchain, inventories features and dependencies, builds a read-only reference, exercises isolated Core behavior and records actual gaps. The first output is evidence and a deletion map, not a repository-wide rename. Confirm the existing session model and rich-chat path, because runtime capabilities differ beyond the terminal codec. D58 permits Herdr 1; Core-specific work is not a requirement to remove the working Herdr 1 path.

After T00, **T01** isolates the Spaceport development identity and **T02** establishes the Core capability adapter. These can proceed independently after agreeing on shared types. Production signing remains later.

## Phase 1 — One thin vertical path

**T03** creates the Core-backed presentation projection and versioned workspace metadata, without a new runtime. **T05** attaches one real local terminal through it, restores the view without restarting the process, and proves last-window quit preserves that process. Then test the same flow with a Core-managed remote terminal.

This proves the Core route independently. D58 permits the existing Herdr 1 backend to remain; do not delete it merely to force this migration.

## Phase 2 — Preserve the agent experience

**T06** proves chat adapter identity, observation, input and permissions with local and remote coverage distinguished. **T07** adapts host, file and service controls to Core, recording G-PORT/G-FILE rather than adding a parallel connection manager. **T08** adapts activity/notifications and supported usage/automation visibility. **T09** unifies retained UI/CLI command semantics and exposes bounded inspectable state.

If a backend gate blocks a task, complete its independent inspection/renderer tests and mark the exact unproved path. Other ready tasks continue. Do not call unsupported features “completed with graceful degradation” when the user's requirement is to retain them.

## Phase 3 — Delete completely

**T04** removes excluded IDE/Git/GitHub/browser/cloud/orchestration/mod features after the view seam is available. It may run alongside the retained-feature tasks with clear file ownership. **T10** removes phone server/push/pairing and remaining excluded mixed-crate remnants only after their retained dependencies are extracted and exercised.

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


---

<!-- Source: docs/05-verification.md -->

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

Removed UI, CLI commands, settings, worker services, provider calls and packaged binaries must be absent. No browser/webview, Git/GitHub workflow, cloud provisioning, Teleport, plugin engine, local scheduler or mobile server is reachable. Herdr 1 itself remains allowed under D58. Plain terminal/chat text mentioning excluded concepts still renders. Product code-removal checks must not disable CI or erase attribution.

## Cross-platform acceptance

Native macOS, Windows and Linux tests cover Herdr 1 and Core startup/discovery, socket/pipe ownership, IME/paste, clipboard images, external browser/file opening, notification permissions, last-window quit, detached process survival, renderer input, settings persistence, installer and updater. On Linux include the window-system configurations the release actually claims; on Windows include console versus windowed launch behavior. A Linux cross-compile is not a Windows UI test.

Build only the architecture matrix deliberately selected for release and report it exactly. Do not claim support just because upstream publishes an architecture. Capture baseline package/renderer requirements before deleting platform-specific dependencies.

## Agent usability evaluation

Give an operating agent a bounded task and only the shipped CLI help/schema: identify which agent is waiting, find its exact host/terminal, inspect current state, navigate to it, and report the supported route for responding. Measure wrong-target attempts, unsupported actions attempted, bytes/tokens of context fetched, command count and successful observed outcomes. Compare to the retained upstream CLI baseline using the same fixture. These are evaluation measurements to collect, not performance claims.

Require semantic success, no unauthorized/destructive side effect, no fabricated outcome and no unnecessary transcript fetch. Repeat with duplicate labels, a stale window, a disconnect during submit, unreported status and a malicious tool-output instruction. Record whether uncertainty was handled correctly, not merely whether the agent kept trying.

## Resource tests

Use a reproducible corpus of idle terminals, active streams, visible panes and large transcripts. Record machine/toolchain/Core versions, CPU, resident memory, subscription count and response/paint latency. Provisional acceptance is no unbounded growth, bounded queues/output, no work proportional to hidden transcript size for a metadata query, and no material regression against the same reference fixture without explanation.

Do not claim a universal latency or memory number before measuring. Proposed budgets in the agent-interface document are limits to implement/test, not results. Include malformed/oversized frames, slow readers, repeatedly opened/closed windows and reconnect storms.

## Release evidence

A final report separates passed, failed, not-run, blocked and owner-approved exceptions. The absence of CI credentials, native runners, release signing or backend support remains visible. Specification validation is reported separately from application conformance. No result is promoted to “verified” merely because an agent wrote a file saying it passed.


---

<!-- Source: docs/06-security-and-resources.md -->

# Security, failure containment and resource ownership

Implements SP-005–SP-006, SP-012–SP-014, SP-018–SP-023 and SP-025.

## Boundaries

Tailscale supplies network connectivity; it is not application consent, a substitute for SSH host verification, or permission to trust content. Core controls runtime access; the OS protects the private local client endpoint. A capability advertises what can be done, not who is authorized to do it.

The retained local control socket/pipe must be restricted to the owning OS user and validate its peer/endpoint using the retained platform policy. This is a same-user control surface, not a sandbox against arbitrary code already running as that user. Do not claim stronger isolation. Never expose it through a phone server, generic TCP bind, browser bridge or tailnet listener.

## Untrusted data stays data

Terminal escape sequences, OSC metadata, chat Markdown, tool output, filenames, repository files, remote status, document labels and links are untrusted. They cannot set new executable paths, request provider credentials, grant confirmations, mutate command catalog definitions or alter update endpoints. Render text safely, bound lengths/images and preserve existing URL/clipboard restrictions. Tool-output instructions are not user requests.

Opening a file or URL is an explicit outward action. Do not fetch external images or follow local file paths simply because remote text references them. Preserve safer upstream behavior when adapting rich content. Generic protocol fields must not bypass typed argument validation or safe process invocation.

## Process lifecycle

Spaceport owns only its UI, workers and deliberately short-lived support processes. Herdr 1, Core and their terminal/agent programs are not children to be swept up by application shutdown. On quit, cancel subscriptions and client-owned transfers, close IPC and exit; do not invoke either runtime's stop/update or agent termination. Tests must prove actual platform process survival, not just that a kill function was not called.

A user-confirmed terminal kill is distinct and precisely targeted. Chat view conversion/resume code that manipulates an agent merely to achieve a layout is removed. A Core keep/recovery policy is backend behavior; Spaceport must not recreate it.

## Credentials and executable selection

Do not read cloud/GitHub/model tokens to populate a usage panel. Do not copy credentials between upstream and fork identities. Existing SSH agents/key stores remain the user's setup. Resolve the installed Core binary from a trusted configured/system location, present its identity/version, and do not pick `./hc` from an untrusted workspace. Never pass remote content through a local shell to construct commands.

Machine onboarding is non-installing and non-upgrading. Do not auto-accept changed host keys or downgrade authentication to make a test pass. Fixtures use separate known-host files and explicit test identities.

## Transfers and forwarding

Distinguish read, replace, append, attach and execute. Preserve overwrite confirmations and target-machine/path identity; a remote path is not a local path. Bound data, validate content type where rendering depends on it, and do not turn a received path into automatic execution. Verify symlink semantics against Core before advertising safe replacement; a client preflight alone cannot eliminate a backend race.

A service forward must have an explicit endpoint/lifetime/owner and a verified route. Do not expose a remote loopback service by rebinding it publicly. Quitting a client must not stop the service itself. If forwarding is Core-owned, accurately describe its actual lifetime; do not assume it ends with the client. No secret second remote connection manager.

## Logging and inspection

Default diagnostics contain operation IDs, qualified nonsecret identities, error classes, versions and bounded timing/state transitions—not prompts, full file contents, tokens, environment dumps or conversation text. Optional diagnostic export is explicit, inspectable and redacted. Never automatically upload it. Panic/crash artifacts require the same attention to secrets.

No offline content cache or durable action queue. Preferences and view references may persist. Logs rotate and bound storage. A bounded in-process receipt buffer is not a replay mechanism; after process loss, unknown effects are reconciled from backend truth rather than invented history.

## Resource accounting

Every queue, parser buffer, transcript page, image, document, request, subscription and retry loop has an owner, bound, cancellation path and overload outcome. Keep control and bulk-data capacity separate. Apply backpressure before allocation where possible. Coalesce repeated display invalidations, not arbitrary terminal bytes or permission requests.

A discarded view cancels its expensive subscriptions unless another view uses them. Multi-window metadata uses shared subscriptions. Permission and completion events are not silently dropped to save resources; on a stream gap, refresh state and report coverage limits. Retain stricter upstream parser/attachment bounds rather than replacing them with larger global defaults.

## Updates and supply chain

Use independent Spaceport update identities and verified signed manifests/artifacts. Reject tampered manifests, wrong app identity and untrusted keys. Updates must not replace Core or upstream app installations. Require explicit owner-controlled production configuration; do not ship development credentials/placeholders. Record retained licenses/dependencies and remove unused service/push bundles from installers.


---

<!-- Source: docs/07-reconciliation.md -->

# Decision reconciliation and architectural corrections

## D58 — Herdr 1 is allowed (2026-11-09)

Doctor Biz explicitly allowed Herdr 1 after clarifying that runtime generation matters less than preserving working UX. This supersedes D55's removal requirement and scopes the earlier Core-only startup/connection rules to Core routes. Existing Herdr 1 plain startup, explicit socket attachment, native workspaces and established host connectors may remain. There is no requirement to retire them merely because they use Herdr 1. Do not restore excluded features or introduce another transport manager. Runtime installation/update, cross-runtime migration and untested parity remain unapproved. Existing Core capability evidence and gaps retain their original scope; permission to use Herdr 1 is not a passing scenario.

This document resolves contradictory assistant paraphrases and false implementation assumptions without reopening the user's product preferences. The chronological ledger remains intact in `plan/decisions.json`.

## R1 — Preserve by default, not redesign by questionnaire

The repeated “keep existing” choices establish the baseline. Existing desktop visual hierarchy, interaction patterns and customization survive unless expressly removed. The new agent ergonomics primarily improve inspection, control correctness, module boundaries and implementation evidence—not a new navigation design.

D49 (“No change”) limits the earlier broad activity-history language. Keep the existing timeline and search its available data. Do not add a global event database, cross-host history service or a new activity-center architecture.

## R2 — Herdr Core is remote-capable

The earlier local-only claim conflated Sheprd's Core integration with Core itself. The inspected Core documents say the local Core owns remote connections, terminal/event aggregation and retry. For Core routes, Spaceport consumes that path through the retained host-manager UI. D58 separately permits the existing Herdr 1 connectors for Herdr 1 sessions. [D58; S01–S03]

D48/D56 qualify D40 for Core routes. A Core machine is Core state; a favorite/color/selection is client presentation. Do not duplicate Core's authoritative catalog in the client or confuse it with a Herdr 1 target.

## R3 — Workspaces need presentation semantics

The earlier claim that Core owns the old daemon's full workspace/agent model was too broad. Sheprd's own session-model document says Core owns terminals and describes unfinished client session-model work. Core separately exposes opaque versioned documents. [S02, S04]

The engineering resolution is a Spaceport-owned **view schema** persisted through Core's document API. It is not a scheduler, execution database or invented `workspace.create` endpoint. Create/rename/archive/delete grouping stays supported; process termination is a different explicit operation. This preserves the user's “client, not replacement runtime” boundary while making the workspace UI implementable.

Writing Core-backed metadata does not make Core interpret that metadata. Documentation and inspection must say which layer owns its semantics. Do not claim interoperability with another client's private workspace schema or silently write `herdr-app`.

## R4 — Native chat is a host-side application contract

Rich chat is not supplied by Alacritty or by Core's raw terminal stream. Sheprd has a host-side adapter, and its Core-specific native attach path is documented as local. [S05, S06, S11]

Preserve the pure renderer/protocol and necessary existing host-side adapter, with Core owning its execution. Remove desktop-owned restart/resume/conversion/deployment policies. Local support does not prove remote parity. A missing Core-mediated remote chat route is an explicit external dependency, not license to restore the deleted remote-control server.

## R5 — The client is thin, not stateless

Client-owned themes, window layouts, input composition, terminal emulation, local CLI IPC and notifications are legitimate. “No independent workspace state” means no competing execution authority or offline operational database, not the impossibility of a view model. Persistent selection/preferences are compatible with online-only content access.

Last-window close quits Spaceport, so desktop notifications stop with it. No background notification daemon is added to satisfy earlier generic “full notifications” wording. Mobile push is out of scope.

## R6 — Agent controls versus agent configuration

D27 removes client-owned provider/model configuration, and D32 means use supported runtime/agent interaction. Keep explicit launch/template selection through Core and dynamically supported prompts/permissions. Do not add provider account screens, credential probes, model selectors or execution policy stores. An agent's terminal may naturally accept its own commands; Spaceport does not censor or independently implement them.

Read-only automation/usage visibility cannot depend on keeping the removed Sheprd scheduler/provider collector. Show only supported source data; record missing feeds as a capability gap. Do not manufacture zero usage or an empty job list.

## R7 — Delete behavior, not every shared type

Herdr 2's current wire protocol is generation 1. `sheprd-protocol` types are also used by the Core renderer. Under D58, retain the Herdr 1 backend and shared rendering/input types; remove only independently excluded behavior. [D58; S02, S09, S11]

Deleting a mobile server does not delete the desktop CLI's private local control endpoint. Deleting Git UI does not prohibit using Git to build the fork or rendering a plain textual patch in chat.

## R8 — Retained features can reveal external gaps

Core's inspected method table has no dedicated forwarding or rich-chat API. General file APIs do not by themselves prove binary transfer parity. [S03]

Keep those requirements and prove their routes. Missing support blocks the affected completion claim. Do not silently drop the feature, build an alternate backend, call terminal fallback full parity or turn “unsupported” into a permanent hidden feature flag. Continue independent fork work, with minimal external contract requests documented for the backend owner.

## R9 — Rebranding is separate from redesign and compatibility

Full Spaceport identity is required; wholesale layout redesign is not. New names/icons/app IDs coexist with retained interaction design. Preserve required attribution and external protocol constants. Unknown production signing/ownership values are release gates, not defaults borrowed from upstream.


---

<!-- Source: docs/08-agent-journeys.md -->

# End-to-end agent journeys

These journeys exercise the complete system rather than isolated modules. Their corresponding automated/manual scenarios live in `plan/acceptance.json`.

## J1 — Find the agent that needs attention

An operator asks, “Which agent in this workspace needs me?” The controlling agent reads client status/capabilities, then a bounded workspace summary. It receives stable Core/terminal/conversation references, reported status, pending attention where authoritative, freshness and coverage. It opens the exact view using the local client command.

It does not fetch every transcript, scan process names, parse an idle-looking screenshot or guess that the most recent terminal is the right one. If status was never reported, it says so. If the host disconnected, it does not use an old green indicator as current evidence.

**Success:** correct target and uncertainty, bounded metadata read, no runtime mutation. **Durable improvement:** any ambiguity found becomes a targeting fixture and catalog clarification.

## J2 — Respond without double-submitting

A user chooses a pending permission answer. The client binds the response to the precise conversation and request, invokes its existing backing contract, then observes the resulting state. Another view may answer first; the second receives the source protocol's stale/already-answered outcome.

If the connection drops after send, Spaceport does not resend on reconnect. It refreshes pending requests and reports an unknown result until reconciled. A generic tool-output card never becomes a permission prompt merely because it says “approve.”

**Success:** no fabricated consent, wrong-session answer or duplicate effect. **Durable improvement:** disconnect/dual-view fixtures and a regression test.

## J3 — Work across a tailnet without owning the network

The user supplies a Tailscale hostname/SSH alias. Spaceport asks Core to add/connect the machine using a non-installing path. Core exposes terminals/events from it. Spaceport draws host and workspace views from that source. When connectivity drops, Core retries while the client displays stale/disconnected state.

Spaceport does not create another SSH daemon, request tailnet admin tokens, install `hc`, update a remote agent or run a second retry scheduler. On Windows, interactive-login limitations are surfaced rather than worked around by unsafe credential capture.

**Success:** a remote program survives disconnect and client quit, and reconnect does not duplicate it. **Durable improvement:** exact supported topology and version recorded in capability evidence.

## J4 — Restore work without restarting it

A user closes the last window. The client exits, leaving Herdr running. On next launch, it restores geometry and references, obtains fresh Core/document state and reattaches to surviving terminals. Missing terminals are not recreated from remembered shell commands.

If a workspace was renamed by another client while this one was closed, the fresh Core document wins. There is no offline transcript browser, hidden prompt queue or separate agent recovery loop.

**Success:** views restored, processes not duplicated, current shared metadata respected. **Durable improvement:** lifecycle regression with actual process IDs and authoritative terminal identity.

## J5 — Modify the fork efficiently

A coding agent receives T04. The task capsule contains the relevant user decisions, deletion units, requirements, source pins and acceptance scenarios. It inspects dependency edges, extracts a shared pure type if needed, removes one coherent feature, runs focused tests and records actual evidence.

It does not read every unrelated source file, ask the user again whether to keep menus, remove a needed renderer type because its name says protocol, or change the terminal engine. Its handoff lists the changed contract and next executable step.

**Success:** unwanted behavior is unreachable, retained behavior is demonstrated, context and changes stay bounded. **Durable improvement:** updated removal record plus negative reachability and retained-feature tests.

## J6 — Discover a backend gap without expanding scope

The agent tries to preserve remote native chat or loopback forwarding and cannot find a supported Core route. It records the inspected method table, existing adapter behavior and the smallest missing contract. It keeps the required UI/renderer work isolated, marks the integration gate open and continues unrelated tasks.

It does not resurrect the phone server, add a custom tunnel manager, expose loopback servers publicly or call a terminal-only workaround full parity. It does not repeatedly ask preference questions to avoid the technical work.

**Success:** the blocker is precise and independently actionable; the product boundary stays intact. **Durable improvement:** a capability contract/test fixture that can be exercised when the backend gains support.


---

<!-- Source: docs/DECISIONS.md -->

# User decision ledger

**Generated from `plan/decisions.json`; edit the JSON, not this view.**

| ID | Decision | Status |
|---|---|---|
| D01 | **Workspaces first** — Keep projects/workspaces as the primary navigation. | active |
| D02 | **No IDE or diff viewer** — Delete IDE integrations and specialized diff/review UI. | active |
| D03 | **Full terminals** — Keep general-purpose shells, tabs, splits and interactive agent terminals. | active |
| D04 | **No Git integration** — Remove Git/worktree/branch/checkpoint management and Git-specific navigation. | active |
| D05 | **Browser versus forwarding** — Remove embedded browser and annotations; keep listening-port discovery and forwarding; open URLs externally. | active |
| D06 | **No cloud provisioning** — Remove provider integrations and provisioning; consume existing Herdr machines. | active |
| D07 | **Automation visibility only** — Keep supported read-only automation status/results; remove editing, scheduling engines and mods. | active |
| D08 | **Equal chat and terminal** — Both remain first-class views; do not imply every terminal has a native chat capability. | active |
| D09 | **Unified and host views** — Unified workspace navigation with optional grouping by host; retain upstream presentation where possible. | active |
| D10 | **Full notifications** — Retain configurable agent attention, completion, error and supported job-result notifications. | active |
| D11 | **Herdr integrations** — Surface supported Herdr integration state/actions; do not own an integration subsystem. | active |
| D12 | **Agent-focused monitoring** — Keep reported agent/usage/rate-limit information; remove detailed host resource dashboards. | active |
| D13 | **No Teleport** — Remove client migration and execution relocation; reconnect to existing sessions. | active |
| D14 | **Workspace controls** — Keep create/rename/open/archive/delete controls; operate on Herdr-backed presentation metadata, not a new runtime. See reconciliation. | clarified |
| D15 | **External Tailscale** — Manual addresses/SSH aliases; Tailscale is independently installed and managed; no discovery or embedded login. | active |
| D16 | **Full customization** — Keep existing themes, layouts, fonts, shortcuts and appearance controls. | active |
| D17 | **Activity history** — Keep an activity timeline; later decision D49 rules out adding a new timeline architecture or history store. | qualified |
| D18 | **Online only** — No persistent offline content cache or offline action queue. View preferences may persist. | active |
| D19 | **Timeline-scoped search** — Search only information actually available in the retained activity timeline; no new file or global terminal index. | active |
| D20 | **Desktop first and full windows** — macOS/Windows/Linux first; retain windows, tabs, splits and pane management. Mobile is a separate exercise. | active |
| D21 | **Full command palette** — Keep the palette and relevant actions; remove commands for deleted features. | active |
| D22 | **File interactions** — Keep drag/drop, clipboard images, attachments, file transfers and external opening; no IDE navigation. | active |
| D23 | **Session creation** — Expose supported session/template launch controls through Herdr. Do not create a second agent registry or runtime. | active |
| D24 | **Local startup** — Connect to or start the separately installed local Herdr Core when needed. | active |
| D25 | **No bundled Herdr** — Do not bundle, download, install or update Herdr on the user's behalf. | active |
| D26 | **Spaceport auto-updates** — Retain app updates using Spaceport-owned identities and release infrastructure; no Herdr updates. | active |
| D27 | **No provider/model configuration** — Remove client-owned model/provider credentials and configuration. Agent programs keep their own settings. | active |
| D28 | **Rich chat** — Keep rich text, code, images, tool events and interactive prompts through a verified existing chat contract. | active |
| D29 | **Attention indicators** — Keep existing sidebar indicators, unread badges, OS badges and notification sounds. | active |
| D30 | **No terminal annotation bridge** — Remove terminal-selection annotations and send-to-agent tools; keep ordinary copy/paste. | active |
| D31 | **No GitHub integration** — Remove client-side GitHub sign-in, account integration, repository/PR/issue/security UI. | active |
| D32 | **Mirror supported input** — Expose only capabilities the backing Herdr/host-side agent contract actually supports. | active |
| D33 | **Quit on last window** — No background tray service; quitting Spaceport must not terminate Herdr or its agents. | active |
| D34 | **No plugins** — Remove all client-side plugin/mod extension systems. | active |
| D35 | **Client CLI** — Keep the relevant full client CLI; agent-runtime operations belong to hc, not duplicate Spaceport implementations. | active |
| D36 | **Restore views** — Restore windows/layout/selection and reattach; do not restart missing agents to recreate a view. | active |
| D37 | **Guided onboarding** — Detect/start installed local Core and offer manual remote setup through Core; never bootstrap missing Herdr. | active |
| D38 | **Existing status display** — Preserve upstream sidebar/status presentation; only report status supported by authoritative data. | active |
| D39 | **Defer terminal swap** — Keep current Alacritty-based emulation/GPUI rendering; no Ghostty migration in this fork. | active |
| D40 | **Host manager UI** — Keep the host-manager experience; D48/D56 place machine state and remote connection logic in Herdr. | qualified |
| D41 | **Existing conversation organization** — Preserve current conversation navigation and session organization. | active |
| D42 | **Existing sidebar** — Preserve hierarchy, interactions and layout options minus deleted feature entries. | active |
| D43 | **Existing terminal UX** — Preserve selection, scrollback, keyboard, mouse, themes and shortcuts. | active |
| D44 | **Existing transfers** — Retain file transfer behavior where supported by the Herdr-backed route; parity must be proved. | active |
| D45 | **Existing settings UI** — Keep design and organization, deleting settings for removed features. | active |
| D46 | **Existing menus** — Keep native/context menu structure and shortcuts, deleting removed actions. | active |
| D47 | **Diagnostics** — Keep troubleshooting logs and diagnostic tooling, without deleted-feature or secret leakage. | active |
| D48 | **Herdr owns connections** — Core owns remote machine catalog, transport, reconnect and aggregated state; Spaceport consumes it. | active |
| D49 | **No timeline redesign** — No change to activity timeline behavior beyond necessary backend migration and deleted-feature cleanup. | active |
| D50 | **Keep profile shell** — Keep profile/avatar UI where meaningful; remove GitHub actions and do not add a client identity system. | active |
| D51 | **Complete rebrand** — Spaceport product identity, executable, app IDs, release identity and assets; preserve required attribution and wire compatibility. | active |
| D52 | **Name** — Spaceport. | active |
| D53 | **Independent fork** — Maintain independently rather than routinely merging upstream; keep provenance and security diligence. | active |
| D54 | **Real deletion** — Delete unwanted code/dependencies/UI/tests/settings/docs; do not merely hide it behind flags. | active |
| D55 | **Herdr 2 only** — Remove legacy Herdr 1 compatibility; this does NOT mean removing Core protocol generation 1. | superseded |
| D56 | **Herdr-mediated connectivity** — Use Core for remote runtime connectivity; no new Spaceport transport/federation service. | active |
| D57 | **Remove mobile server** — Delete phone remote-control server and its desktop integration; defer all mobile design/implementation. | active |
| D58 | **Allow Herdr 1** — Herdr 1 is an allowed runtime alongside Herdr Core. Preserve working native UX instead of requiring a Core-only startup or deleting the Herdr 1 backend. D55 is superseded. D24, D37, D48 and D56 apply to Core routes; existing Herdr 1 startup and connectors may serve Herdr 1 routes. Retain exact runtime targets and session ownership. This does not restore excluded features, authorize runtime installation/update or imply untested parity. | active |
| D59 | **Credit both upstream teams** — The Spaceport About view must explicitly reference the Herdr team and the Sheprd team. Label upstream links as credits, retain required copyright/license notices and avoid implying endorsement. | active |
| D60 | **Remove the inherited custom updater** — Delete the inherited custom Rust updater and its update/install/restart machinery. Spaceport updates remain a required release capability; this is a replacement transition, not removal of SP-023. Support macOS, Windows, Linux and package-manager installation routes. Sparkle and other platform mechanisms were discussed but no replacement library, Homebrew update-ownership policy, signing identity, release endpoint or publication was approved by this deletion instruction. Preserve release-build refusal while independent identity and update routes remain unverified. | active |
| D61 | **macOS signed preview workflow** — Implement a macOS-only build-and-sign/notarize artifact workflow using 2389 Research signing credentials and bundle ID ai.2389.spaceport. Windows signing is deferred for this work, not removed from the eventual desktop scope. D64 extends this route to tag-triggered GitHub prereleases. App installation, credential upload, automatic updater and general production release enablement remain separate from this decision. Retain the release refusal and distinguish ordinary development builds from signed previews. | active |
| D62 | **Rocket emoji icon** — Use a rocket as Spaceport's app icon. D63 selects the later sculpted nebula artwork after the initial emoji exploration. Keep Spaceport as the product name; derive platform artwork from one shared source. | qualified |
| D63 | **Ship the nebula rocket artwork** — Use the approved sculpted ivory/red rocket with a violet/cyan nebula background as Spaceport's app artwork. The selected master is design/spaceport-icon/rocket-nebula.png. Generate platform and branding exports from this image. | active |
| D64 | **Tag-triggered macOS preview releases** — Extend the macOS preview workflow to build, sign, notarize and publish a DMG plus checksum to a GitHub prerelease when harperreed pushes vYYYYMMDD.COUNTER to harperreed/spaceport. Require the exact tagged commit to belong to main and pass full main CI. Keep signing credentials in a protected signing job; grant release write access only to publication. Preserve manual artifact-only runs, immutable version reservations and the broader production release refusal. Doctor Biz's follow-up instruction to push and make it work authorizes main integration, hosted environment setup, credential upload and the first verified tagged preview; keep private signing inputs outside Git. | active |
| D65 | **Sparkle updates for macOS previews** — Use pinned Sparkle 2.10.1 for installed macOS previews, with native Check for Updates UI, consent for automatic checks and automatic installation off by default. Keep development/worktree builds excluded. Embed and sign the framework inside the existing ai.2389.spaceport bundle for Team HD9NM9NSMK. Use a separate Spaceport Ed25519 key outside Git/Cargo and in the protected macos-preview environment; require signed feeds and pre-extraction archive verification. Publish the signed preview feed as the fixed macos-preview-feed GitHub release asset only after the immutable dated release passes verification. Enforce exact Spaceport app/Team identity in packaging, signing and publication. Sparkle permits trusted publisher key and app identity rotation; do not claim its runtime enforces an unconditional old app ID match. Preserve normal GUI quit/detach and never change Herdr/Core lifecycle. Qualify with isolated real signed update/relaunch fixtures and negative trust/rollback cases. Carry implementation through full source CI and a fresh signed release. Other platform/package routes and overall T12 gates remain open. | active |


---

<!-- Source: docs/IMPLEMENTATION_TASKS.md -->

# Implementation task graph

**Generated from `plan/tasks.json`; this is not a second progress tracker.**

## T00 — Pin, inventory and reproduce the baseline

Status: **done** · Prerequisites: none

Pin source/toolchain; generate actual feature/dependency/capability evidence and isolated baseline. No broad code deletion.

Requirements: SP-001, SP-004, SP-024, SP-025, SP-026

Acceptance: A01, A45, A47

Gates to investigate: G-CHAT, G-PORT, G-FILE, G-OBS, G-PLATFORM

Gates this task must close: none; record findings without claiming full parity

## T01 — Isolate Spaceport development identity

Status: **in_progress** · Prerequisites: T00

Separate names/state/control endpoint from upstream; preserve compatibility and attribution. Production values remain gated.

Requirements: SP-001, SP-022, SP-025

Acceptance: A42, A47

Gates to investigate: G-RELEASE

Gates this task must close: none; record findings without claiming full parity

## T02 — Adapt the verified Core capability boundary

Status: **blocked** · Prerequisites: T00

Typed Core calls/streams/capabilities with explicit runtime targets and no new remote manager; existing Herdr 1 routes remain allowed under D58.

Requirements: SP-003, SP-004, SP-005, SP-006, SP-019, SP-020

Acceptance: A02, A03, A04, A05, A53

Gates to investigate: G-MACHINE

Gates this task must close: G-MACHINE

## T03 — Build one authoritative view projection

Status: **in_progress** · Prerequisites: T02

Keep each view authoritative for its selected runtime; retain Herdr 1 snapshots and implement namespaced Core presentation documents and local view preferences. Core-only default startup is no longer required under D58.

Requirements: SP-002, SP-007, SP-013, SP-018, SP-024

Acceptance: A06, A07, A08, A09, A24, A53

Gates to investigate: none additional

Gates this task must close: none; record findings without claiming full parity

## T04 — Remove excluded product feature families

Status: **blocked** · Prerequisites: T01, T03

Delete IDE/Git/GitHub/browser/cloud/Teleport/orchestration/mod/scheduler surfaces and dependencies in coherent units.

Requirements: SP-002, SP-015, SP-016, SP-022

Acceptance: A28, A29, A30, A33, A34

Gates to investigate: G-LINK

Gates this task must close: G-LINK

## T05 — Prove terminal and lifecycle parity

Status: **in_progress** · Prerequisites: T03

Prove supported local/remote Herdr 1 and Core terminal attach, input, window lifecycle and process survival through the retained renderer; record runtime-specific coverage.

Requirements: SP-002, SP-008, SP-012, SP-013, SP-018, SP-020

Acceptance: A09, A10, A11, A12, A13, A22, A23, A24, A40

Gates to investigate: none additional

Gates this task must close: none; record findings without claiming full parity

## T06 — Preserve native chat under the ownership contract

Status: **blocked** · Prerequisites: T03, T05

Keep renderer/host adapter where valid; remove desktop-owned conversion/resume/config; prove route or record blocked parity.

Requirements: SP-003, SP-009, SP-010, SP-019, SP-026

Acceptance: A14, A15, A16, A17, A54, A55

Gates to investigate: G-CHAT

Gates this task must close: G-CHAT

## T07 — Adapt hosts, transfers and service access

Status: **blocked** · Prerequisites: T02, T03

Preserve host/file/service UX through separately verified Herdr 1 or Core routes; no public rebinding or new transport workaround. Core-specific gaps remain until verified or explicitly rescoped with evidence.

Requirements: SP-006, SP-014, SP-019, SP-026

Acceptance: A04, A05, A25, A26, A27, A28

Gates to investigate: G-PORT, G-FILE, G-MACHINE

Gates this task must close: G-PORT, G-FILE, G-MACHINE

## T08 — Preserve activity and attention without collectors

Status: **blocked** · Prerequisites: T03, T06

Existing timeline and notifications consume authoritative data; usage/job gaps explicit; no background service.

Requirements: SP-002, SP-010, SP-011, SP-012, SP-013, SP-021

Acceptance: A18, A19, A20, A21, A24, A41

Gates to investigate: G-OBS

Gates this task must close: G-OBS

## T09 — Unify client commands and structured inspection

Status: **in_progress** · Prerequisites: T03, T05

One catalog for existing GUI/CLI; exact targets, bounded observations, honest outcomes and preserved confirmations.

Requirements: SP-017, SP-018, SP-019, SP-020, SP-021, SP-024

Acceptance: A16, A17, A35, A36, A37, A38, A39, A41, A45, A46, A52, A55

Gates to investigate: none additional

Gates this task must close: none; record findings without claiming full parity

## T10 — Delete mobile and excluded remnants and finalize dependencies

Status: **in_progress** · Prerequisites: T04, T05, T06, T07, T08, T09

Remove all excluded reachable behavior and packaged services after necessary pure primitives are extracted.

Requirements: SP-004, SP-015, SP-016, SP-017, SP-022, SP-026

Acceptance: A29, A30, A31, A32, A42, A54

Gates to investigate: none additional

Gates this task must close: none; record findings without claiming full parity

## T11 — Run native desktop and end-to-end qualification

Status: **not_started** · Prerequisites: T10

Native platform, adversarial, resource and agent-usability evidence. Unproved requirements remain blocked.

Requirements: SP-001, SP-002, SP-008, SP-012, SP-018, SP-019, SP-020, SP-024, SP-026

Acceptance: A33, A34, A38, A39, A40, A46, A48, A49, A50, A51, A52

Gates to investigate: G-CHAT, G-PORT, G-FILE, G-OBS, G-PLATFORM

Gates this task must close: G-CHAT, G-PORT, G-FILE, G-OBS, G-PLATFORM

## T12 — Complete release identity and verified updates

Status: **blocked** · Prerequisites: T01, T10, T11

Owner-controlled signed packages and verified platform/package update routes after D60 removes the inherited custom updater; no publishing without authorization; all product gates reconciled.

Requirements: SP-022, SP-023, SP-025, SP-026

Acceptance: A42, A43, A44, A48, A49, A50, A51

Gates to investigate: G-RELEASE, G-PLATFORM

Gates this task must close: G-RELEASE, G-PLATFORM


---

<!-- Source: docs/SOURCES.md -->

# Source evidence and scope

**Review date:** 7 November 2026. **Sheprd:** `806b49870ac7761b98cbcea4bd115aa2f2b00d75`. **Herdr Core:** `cd12ddd88bc11490228343291dc1096c9ed3bf77`.

These revisions were obtained from the repositories during this task. They are investigation baselines, not verified release pins. Source was read through the connected GitHub tools; a container-side attempt to retrieve full source bytes failed because network name resolution was unavailable. No repository checkout, application build, live daemon test or native-platform test was performed.

A Library search did not locate a Spaceport-specific design document to patch. This package consolidates the user decisions in the conversation into new authoritative documents; it does not modify unrelated Library documents or a remote repository.

## Primary sources

### S01 — `herdrdev/herdr-core/docs/machines.md`
[Pinned source](https://github.com/herdrdev/herdr-core/blob/cd12ddd88bc11490228343291dc1096c9ed3bf77/docs/machines.md) · Core-owned remote machines, aggregation, reconnect and file operations.

### S02 — `herdrdev/herdr-core/schema/README.md`
[Pinned source](https://github.com/herdrdev/herdr-core/blob/cd12ddd88bc11490228343291dc1096c9ed3bf77/schema/README.md) · Wire protocol, attach streams, events, sequence resets, opaque documents and reported status.

### S03 — `herdrdev/herdr-core/crates/herdr-core/src/api.rs`
[Pinned source](https://github.com/herdrdev/herdr-core/blob/cd12ddd88bc11490228343291dc1096c9ed3bf77/crates/herdr-core/src/api.rs) · Complete method table and hello capability declaration; inspected lines 1–210.

### S04 — `sheprd-labs/sheprd/docs/session-model.md`
[Pinned source](https://github.com/sheprd-labs/sheprd/blob/806b49870ac7761b98cbcea4bd115aa2f2b00d75/docs/session-model.md) · Actual split between Herdr 1 workspaces and Core terminal/layout state; migration plan is not implemented parity.

### S05 — `sheprd-labs/sheprd/docs/rich-chat.md`
[Pinned source](https://github.com/sheprd-labs/sheprd/blob/806b49870ac7761b98cbcea4bd115aa2f2b00d75/docs/rich-chat.md) · Herdr 2 chat endpoint labels, local-only native attach, conversion and resume side effects; inspected lines 1–190.

### S06 — `sheprd-labs/sheprd/crates/sheprd-acp/README.md`
[Pinned source](https://github.com/sheprd-labs/sheprd/blob/806b49870ac7761b98cbcea4bd115aa2f2b00d75/crates/sheprd-acp/README.md) · Host-side conversation adapter, attach protocol, agent ownership and restore behavior; retrieved output was truncated after the behavior section.

### S07 — `sheprd-labs/sheprd/docs/cli.md`
[Pinned source](https://github.com/sheprd-labs/sheprd/blob/806b49870ac7761b98cbcea4bd115aa2f2b00d75/docs/cli.md) · Existing private local control interface, JSON output, targeting and command families; inspected lines 1–210.

### S08 — `sheprd-labs/sheprd/Cargo.toml`
[Pinned source](https://github.com/sheprd-labs/sheprd/blob/806b49870ac7761b98cbcea4bd115aa2f2b00d75/Cargo.toml) · Current crate inventory and dependencies; inspected lines 1–150.

### S09 — `sheprd-labs/sheprd/AGENTS.md`
[Pinned source](https://github.com/sheprd-labs/sheprd/blob/806b49870ac7761b98cbcea4bd115aa2f2b00d75/AGENTS.md) · Crate responsibilities and engineering conventions; requested lines 1–220, output was truncated in Rust conventions.

### S10 — `sheprd-labs/sheprd/crates/sheprd-core-client/src/method.rs`
[Pinned source](https://github.com/sheprd-labs/sheprd/blob/806b49870ac7761b98cbcea4bd115aa2f2b00d75/crates/sheprd-core-client/src/method.rs) · Existing Core adapter exposes only a subset of the current Core API.

### S11 — `sheprd-labs/sheprd/crates/sheprd-terminal/README.md`
[Pinned source](https://github.com/sheprd-labs/sheprd/blob/806b49870ac7761b98cbcea4bd115aa2f2b00d75/crates/sheprd-terminal/README.md) · Pure Alacritty-based emulator, damage, input and bounded rendering contracts.

### S12 — `herdrdev/herdr-core/schema/hc.openrpc.json`
[Pinned source](https://github.com/herdrdev/herdr-core/blob/cd12ddd88bc11490228343291dc1096c9ed3bf77/schema/hc.openrpc.json) · Inspected lines 1–160: persistent core identity, attach flags and versioned document result types; not a full schema audit.

## Interpretation rules

Facts about existing software are limited to the inspected source scope. API shape is not proof of tested runtime behavior. Absence of a dedicated method in the inspected table is not proof that no supported route could ever exist; affected capabilities remain investigation gates.

User scope choices come from this conversation, not from upstream documentation. All new architecture, interface budgets, task sequencing, workspace view schema and acceptance criteria are proposed engineering requirements for this fork. No trademark, domain, signing-account or production-infrastructure ownership has been verified.


---

<!-- Source: docs/VALIDATION.md -->

# Validation report — Spaceport specification 1.0

**Scope: specification artifacts and their local tools only. No Spaceport application was forked, built or exercised in this task. No remote repository was changed.**

## Executed checks

| Check | Result |
|---|---|
| User decision ledger | 57 unique decision IDs |
| Normative requirement coverage | 26 requirements, each assigned to tasks and acceptance scenarios |
| Task dependency graph | 13 tasks; references valid; no cycles |
| Application scenario registry | 55 planned scenarios; all `not_run` |
| Capability/removal/gap references | 10 capability entries, 19 disposition entries, 6 open gates |
| Source references | 12 source entries with exact commit pins and read-scope notes |
| Authored JSON parsing | Passed; duplicate keys and non-finite numbers refused |
| Control JSON Schema and synthetic examples | Passed; 2 examples validated |
| Local Markdown file links and generated decision/task views | Passed; anchors and external URLs not validated by the tool |
| Specification-tool unit tests | 21 passed; no failures, errors or skips |
| Task-packet bounds | All 13 fit the 65,536-byte ceiling; largest 49,037 UTF-8 bytes |
| Input-change detection | SHA-256 input manifest included; local environments/build trees excluded |

Runtime used for these checks: Python 3.13.5, jsonschema 4.26.0. These versions describe specification validation, not the application toolchain.

Machine-readable reports: `evidence/validation.json` and `evidence/tool-tests.json`. Full tooling test output: `evidence/spec-tool-tests.txt`. Input hashes provide change detection, not proof of approval or actual runtime behavior. The validator checks evidence-reference presence, not authenticity or sufficiency.

## Not executed

**Application acceptance scenarios executed: 0.** Source review is not product conformance. No native macOS/Windows/Linux build or UI test, real Herdr startup/attach/chat/forwarding test, tailnet test, permission race test, product performance measurement, package/signing/update test or production deployment occurred.

The upstream audit is limited to the exact source sections recorded in `plan/sources.json`; it is not a whole-repository dependency audit or a guarantee that candidate deletion paths have no surviving consumers. T00 must establish that baseline in the implementation environment.

G-CHAT, G-PORT, G-FILE, G-OBS, G-RELEASE and G-PLATFORM remain open. Specification checks do not close them. Every task starts `not_started`; no task is represented as completed implementation.

## Reproduce

From the package root, after installing `tools/requirements.txt`:

```sh
python tools/spec.py render
python tools/spec.py check --write-report
python tools/test_spec.py
python tools/spec.py ready
python tools/spec.py context T00
```

The generated combined reading copy is refreshed by `render`. The registries and component documents remain authoritative.


---

<!-- Source: BOOTSTRAP_PROMPT.md -->

# Prompt for the Spaceport fork agent

You are implementing **Spaceport**, an independent, fully rebranded desktop fork of Sheprd/herdr-gpui. The specification package beside this prompt is authoritative. Keep it intact at `docs/spaceport/` in the fork and integrate a pointer into the repository's root agent instructions without erasing applicable upstream rules. Run its tools from the package root; candidate code paths refer to the application repository. Read `AGENTS.md`, `SPEC.md` and `SYSTEM.md`, then run the package checks and obtain the T00 context packet.

The user likes the existing desktop UX. Preserve it by default. Remove IDE/diff/Git/GitHub/embedded-browser/cloud-provisioning/Teleport/orchestration/mod/scheduler features, terminal annotations and the mobile remote-control/push service completely. Keep terminals, rich chats, workspaces, existing UI customization, file interactions, required service forwarding, notifications, diagnostics and the local desktop CLI, subject to the exact capability/ownership contracts.

Use separately installed Herdr 1 or **Herdr Core** runtimes under D58. Preserve the existing Herdr 1 startup and connection paths where needed for native UX. Core owns Core machine connections; keep every target tied to its runtime. Tailscale is external. Do not bundle/install/update either runtime or edit its repository. Do not implement another transport manager, scheduler, session recovery engine, provider configuration system or mobile server. Do not change the terminal engine or start mobile/UniFFI work.

The spec corrects earlier assumptions: Core already owns remote machine connections; Core does not supply the old daemon's native workspace semantics; rich chat depends on a host-side adapter and remote native parity must be proved. Use Core documents for explicit presentation metadata as specified. Do not invent backend RPCs or claim that an untested adapter is supported.

Begin with T00: verify checkout safety and source revisions, build an isolated reference, inventory dependencies, inspect actual Core methods/features and record capability gaps. Then follow `plan/tasks.json`, using task-sized context packets. Do not start with a bulk rename or deletion of the legacy client crate. Reuse shared types and existing command/control infrastructure where they fit.

For each bounded change: reproduce, implement, test, inspect the diff, update task/acceptance evidence, and preserve one useful regression or contract clarification. Prefer source/schema/structured observations over screenshots or guessed state. Do not blindly retry uncertain side effects. Keep permissions and exact target identity intact.

Do not ask the user to repeat settled choices. If a required backend route is missing, record the smallest precise external dependency, mark the affected task blocked, and continue independent work. Do not silently ship a smaller product or cross the client/runtime boundary to hide a gap.

Deliver code commits, retained/excluded feature evidence, capability matrix, native-platform results, release identity audit, current documentation and a concise list of any unresolved gates. A passing specification validator or compilation is not evidence that the application works. Do not publish or mutate real user environments without explicit authorization.
