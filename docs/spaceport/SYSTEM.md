# Spaceport as one system

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

Core owns machine registrations, process identity/lifecycle, terminal streams, program-reported status and supported file/document operations. Agent programs own agent sessions, models, permissions and conversation state. Tailscale owns network membership/routing. Spaceport does not duplicate these policies. [S01–S03]

### Layer B — Narrow Core adapter

Reuse and extend `sheprd-core-client` to call the methods the selected Core actually advertises. Implement local socket/named-pipe and framed stream correctness once. Core itself carries remote machine traffic. The adapter exposes typed facts, stream subscriptions and effect results, not GPUI entities. No provider-specific cloud adapters or a generalized plugin transport.

### Layer C — Session and view projection

Reuse `sheprd-session` and extract remaining view readers away from legacy `ClientShell*` snapshots. Produce immutable or coherently published view revisions. A reducer applies backend facts and produces view changes; it does not launch processes.

Store presentation grouping in a namespaced Core document; maintain local window geometry/preferences separately. Do not create a workspace database or treat a local list as authoritative remote execution state. Terminal and document reads have different versions; the combined view must expose that it is a **composite observation**, not an atomic distributed snapshot.

### Layer D — Command semantics

A single typed catalog declares each retained client action's owner, arguments, target kind, required capability, safety class and observable postcondition. All surfaces call it. An action executes through either the local presentation owner or the verified backend adapter, never both implementations independently. The catalog is not a generic RPC proxy exposing every Core method.

### Layer E — Presentation and automation surfaces

Keep GPUI and existing interactions. The private CLI/IPC exposes concise state and client controls, not a new runtime. Agents use `hc` directly for runtime functions. Native notifications, external browser opening and platform dialogs live at this boundary.

Logical layers are responsibility boundaries, not a mandate for a new crate per box. Reuse current modules; extract only where a concrete dependency or test boundary justifies it. [S08–S10]

## 3. Ownership matrix

| Fact or operation | Authority | Spaceport role |
|---|---|---|
| Process/terminal existence and termination | Core | Inspect, render, send an explicit supported request |
| Remote SSH catalog and retry policy | Core | Display/edit via `machine.*`; reconnect only its local Core link |
| Agent conversation, queues, model and approval policy | Agent/verified host adapter | Render and forward supported user interaction |
| Workspace grouping metadata | Spaceport view schema, stored by Core `doc.*` | Versioned presentation operations, no execution ownership |
| Window position, theme, selected view | Spaceport local preferences | Persist/restore presentation intent |
| Event observation and notification deduplication | Reconstructable client projection | Bounded in-memory processing, no history warehouse |
| Terminal cell state | Pure emulator from Core stream | Render and discard/rebuild when appropriate |
| Existing service forward | Backend-supported route | Show state and open externally; gate absent route |
| User/provider credentials | Existing OS/runtime/agent owners | Do not import or create a new credential subsystem |
| Spaceport release/update identity | Spaceport release configuration | Sign and update only Spaceport |

## 4. Identity is not a label

A runtime reference includes the selected **Core identity**, machine route/identity when available, backend terminal ID/address, and the client connection epoch for freshness. Names are display hints. A persistent Core ID is not a boot ID. Core event sequences restart with a process, so every reconnection invalidates sequence continuity and obtains a fresh snapshot. [S02, S12]

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

A display action can often be confirmed locally. A process launch needs Core evidence. A prompt send may only prove transport acceptance, not model execution. A timed-out mutation is not retried automatically unless the actual protocol guarantees idempotency for that operation. Do not implement an “exactly once” claim with a client UUID.

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
