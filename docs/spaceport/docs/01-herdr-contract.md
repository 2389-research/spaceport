# Herdr integration contract

Implements SP-003–SP-010, SP-013–SP-014 and SP-018–SP-019. Sources: [S01–S06, S10–S12](SOURCES.md).

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

Retain the pure chat model/reducer and native renderer. Retain only host-side adapter behavior necessary for supported chat, with Core owning its process. Do not let the desktop resume/restart agents automatically, convert terminal agents by killing them, manage model/provider credentials or deploy adapters to remote hosts. Such old paths are incompatible with the thin-client boundary.

Local IPC to an already running host adapter is an existing compatibility dependency, not proof that Core routes native chat remotely. Inventory it explicitly. The target is a verified Herdr-owned route; a separate Spaceport SSH relay, HTTP/WebSocket service or mobile server is forbidden as an unapproved workaround. Do not pass binary chat traffic through an emulated terminal and assume a PTY is a transparent stream.

Gate **G-CHAT** requires an exact discovery, identity, authentication, transport and permission-response contract, with reconnect and concurrent-view tests. Terminal fallback keeps a session usable but does not satisfy a native-chat parity claim.

## 6. Services, files, automation and usage

For files, prefer supported Core APIs and preserve the distinction between transferring bytes and submitting their path as input. General binary upload/download, overwrite behavior, symlink handling, temporary-file lifetime and cancellation must be checked separately; the existence of `file.read` does not prove SFTP-equivalent parity.

For services, keep port discovery/opening UI and the explicit forwarding requirement. Opening an already reachable Tailscale URL is not equivalent to forwarding a remote loopback-only service. Do not widen a service's bind address or create a separate host connection manager. If Core lacks the needed forwarding primitive, record G-PORT and a minimal external contract request; do not implement a new Core feature in this repository.

For automation and usage, the inspected Sheprd subsystems contain their own schedulers/provider probes. Removing those does not create an equivalent Core API. Retain passive displays only when an authoritative, supported feed exists. No host script scanning, provider credential reads or local schedule engine should reappear under “visibility.” Gate G-OBS records the actual supported subset.

## 7. Gap closure evidence

A gate closes with a pinned method/schema or documented adapter contract **and** a reproducible scenario showing the desired effect on the relevant platform/topology. A README statement, source symbol or mock alone cannot close runtime parity. If a missing capability belongs in Herdr, write an external dependency note, not an incidental upstream patch. Continue unaffected work and report the blocked retained behavior plainly.
