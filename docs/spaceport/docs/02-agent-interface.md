# Agent-facing client contract

Implements SP-017–SP-021 and SP-024. This is an **additive refinement of the retained local CLI**, not a new product daemon or a replacement for `hc`. Existing relevant commands keep their semantics after removed features disappear. Source baseline: [S07, S09](SOURCES.md).

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
