# Core byte-route audit for G-CHAT — 2026-11-09

This source/schema audit ran no runtime commands itself. Root separately captured
the installed Core 0.1.19 schema in an isolated fixture; that CLI query started a
private Core process, later identified by its exact private socket/state and
executable and stopped/reaped. The schema is retained below. The process tests
and their narrower claims are recorded separately; no personal daemon was used.

Evidence roots:
- `C` = `/private/tmp/spaceport-core-reference`, pinned Core `cd12ddd88bc11490228343291dc1096c9ed3bf77` (0.1.18).
- `S` = `/Users/harper/Public/src/2389/spaceport-t00`, application source at task start `65f6ac438` (another agent may edit it concurrently).
- Installed Core 0.1.19 schema: [raw OpenRPC](t06-core-0.1.19-openrpc.json), one physical line. Exact JSON pointers below identify entries without inventing line numbers. Its implementation was not available in this audit; 0.1.18 internals must not be attributed to 0.1.19.

## Finding

A chat-named RPC is unnecessary in principle. The installed adapter already has a transparent bidirectional stdio connector to an existing chat. A Core-owned generic, authenticated, non-PTY duplex process stream could carry that attach wire while the terminal-owned adapter retains agent lifecycle and permission authority. However, neither the installed 0.1.19 advertised contract nor pinned 0.1.18 implementation supplies that generic duplex exec stream. Existing `machine.exec` is a bounded batch call. Existing framed stream bytes terminate in terminal viewers or event subscriptions, not exec stdin/stdout. G-CHAT remains open; no remote or native parity is established by this audit.

## Existing capabilities, with evidence

1. **Non-PTY execution, including Core-owned remote execution, exists.** Installed schema `/methods/19` (`machine.exec`) accepts `argv` or `line`, `on`, `cwd`, `env`, one base64 `input`, and `timeout`. It returns `ExecResult`, not a channel. `input` is at most 8 MB; absent means stdin closed; timeout defaults to 60 seconds, at most 600. Result stdout is first 8 MB, stderr first 1 MB, with truncation and timeout flags (`/components/schemas/ExecResult`). Pinned `C/crates/herdr-core/src/api/types.rs:585-640`, `exec.rs:17-25,132-223,276-284` confirm that exact architecture: piped stdio, feed one buffer and close, capture concurrently, return only after exit/termination. `cli.rs:503-506` reads all piped input before sending the RPC. There is no interactive output callback or follow-up stdin handle in this contract.

2. **Core already owns the remote connection and checks its saved identity.** Pinned `C/crates/herdr-core/src/remote.rs:760-796` routes `machine.exec` through `on`; `named` at 745-754 accepts saved machine name OR permanent ID. This is useful for a future generic stream: an ID-bearing machine selector is already possible for routed calls. `machines.rs:59-76,229-249` constructs SSH with `-T` in the ordinary path, or the user's configured `via` command; it passes stdio to `hc connect`, not an arbitrary program endpoint. `remote.rs:260-264` compares remote hello Core ID with the saved ID and rejects a different Core. SSH provides the ordinary transport authentication; a custom `via` command's authentication is its configured mechanism, not an extra guarantee supplied by this audit. The reverse-peer callback allowlist is narrower (`remote.rs:524,760-783`); `machine.exec` cannot target arbitrary inbound peers. Do not replace this with desktop SSH or another reconnect owner.

3. **Core's stdio bridge speaks the Core protocol.** Pinned `C/crates/herdr-core/src/cli.rs:101-106,829-848` defines `hc connect` as stdin/stdout to the Core socket. It transparently carries Core JSON/framed protocol, but does not accept an arbitrary executable or adapter socket to bridge. It calls `connect_or_start`; no CLI was invoked here. This is not a generic byte-route escape hatch.

4. **Framing alone does not create a process endpoint.** Pinned `C/crates/herdr-core/src/wire.rs:1-31` has channel-0 JSON, DATA/CREDIT/CLOSE and terminal controls; 256 KiB initial credit, 4 MiB pending cap. `server.rs:305-373` dispatches nonzero DATA to `viewer_input`, and `viewers.rs:254-265` sends it to a terminal link through terminal input gates. Only `terminal.attach` receives the remote attach special handling (`server.rs:420-431`); its result is first paint then terminal bytes (`api/types.rs:412-415`, installed schema `/methods/13`, `/components/schemas/AttachResult`). `remote/viewers.rs:82-137` maps and relays that terminal channel both ways. Terminal reattach repaints and drops input during line gaps (`remote/viewers.rs:20-69,135-136`). `wire.rs:237-265` may discard a slow terminal backlog and ask the caller for a repaint; event overflow preserves line framing and replaces missed facts. Those are unsuitable assumptions for lossless arbitrary adapter frames. A new consumer cannot simply reuse a terminal channel without a verified distinct byte-stream contract.

5. **Cancellation is present but batch-scoped.** Pinned `C/crates/herdr-core/src/server.rs:305-309,325-349,378-386` tracks exec tasks by request ID and aborts on cancel/disconnect; `exec.rs:286-338` owns process-tree cleanup. `remote.rs:100-155` forwards pending-call cancellation with `$/cancelRequest`, and 849-852 budgets exec timeout plus cleanup. No streaming stdin half-close or durable attach lifetime follows from that. The installed 0.1.19 schema explicitly says exec descendants end at exit, timeout or caller loss; actual 0.1.19 cancellation implementation remains uninspected.

6. **Events carry facts, not arbitrary child output.** Installed `/methods/25` (`events.subscribe`) lists snapshot/fact changes, heartbeat and missed+snapshot behavior; `C/crates/herdr-core/src/api.rs:131-133` agrees. Remote terminal facts preserve labels (`remote.rs:411,470,515`), so existing `acp` label discovery is useful raw material. No observed event subscription feeds incremental bytes into an exec. Building a document/event mailbox tunnel would add a new transport protocol and remote helper behavior, not merely adapt the existing connector.

## Existing adapter/client seam

- `S/crates/sheprd-acp/src/cli.rs:38-39,106,163-171`, `main.rs:51`, `relay.rs:23-53`: `sheprd-acp connect ID` connects to the existing private endpoint, copies stdin to the socket and socket to flushed stdout, and closes on EOF. It does not start or own the agent. That connector is the simplest existing host-side seam; no new Spaceport remote server is necessary if Core later provides a suitable duplex route.
- Endpoint discovery already uses the `acp` label and `HERDR_ACP_ENDPOINT`; endpoint IDs are validated 32 lowercase hex (`S/crates/sheprd-chat-protocol/src/endpoint.rs:14-55`). Address resolution is host-local (`endpoint.rs:66-90`). Unix listener modes are directory 0700/socket 0600 (`sheprd-client/src/private_endpoint/unix.rs:17-27`); Windows source implements same-user pipe restrictions/owner checking (`private_endpoint/windows/security.rs:1-8`). These are source findings, not new platform tests. Endpoint ID and attach hello are not independent cryptographic credentials.
- Adapter hello/version and replay cursor are end-to-end (`S/crates/sheprd-acp/src/attach.rs:115-145`); permission authority remains in `session/commands.rs:158-167`, rejecting unknown permissions/options and resolving the accepted one. A future byte relay should preserve that wire, not reinterpret approval as terminal input.
- The app currently rejects qualified Core IDs in `S/crates/sheprd-gpui/src/chat/core_pane.rs:35-55`. Its `spawn` at 72-96 labels a runtime-owned terminal. `chat/chats.rs:72-81` offers local socket or legacy direct SSH relay only. The Core client method table (`sheprd-core-client/src/method.rs:30-49`) has no `machine.exec`; `connection.rs:240-266,343-355,514-574` already handles subscription/terminal channel registration but exposes no general adapter transport. Adding batch `machine.exec` to that table alone cannot fix this.

## Smallest remaining dependency and client work

External need: a Herdr-owned, explicitly supported way to open a non-PTY bidirectional byte route to an already-installed adapter connector/private endpoint on a saved Core machine, or evidence of such a route in a newer Core contract. A generic streaming process facility could suffice; do not require or invent a chat-specific RPC name. Its contract must define identity selection, authenticated access/refusal, binary fidelity, bounded backpressure without byte loss/repaint, disconnect/cancel/EOF/exit behavior, and attach recovery. Preserve the runtime terminal's adapter and agent when a viewer detaches. No automatic adapter deployment, desktop agent child, second transport manager or Spaceport server follows.

Client work after that contract exists: carry saved machine ID + qualified terminal + validated endpoint together; resolve fresh endpoint facts; call the verified Core route; pass bytes to existing attach framing; separate observer loss from agent lifecycle; surface unsupported capability explicitly; fence stale connections and permission replies. Preserve the current local route independently. Local/remote two-view process tests must exercise permissions, reconnect, detach, stale identities and backpressure; native supported-platform runs must prove UI and OS behavior before claiming parity. Existing component fixtures in `t06-local-adapter-route.md` do not prove this missing remote route.

This narrows `S/docs/spaceport/evidence/external-core-dependencies.md` and `t06-local-adapter-route.md`: discovery labels, remote execution, remote identity checks, stdio adapter relay and attach permissions already exist in separate pieces. The missing external primitive is their supported duplex connection, with lifetime/bounds guarantees. The absence of a chat-named RPC alone is not evidence of that gap.

## Machine action guard recheck

Installed schema `/methods/28` (`machine.connect`) and `/methods/30` (`machine.rm`) accept only `machine`; `/methods/29` (`machine.rename`) accepts `machine,new_name`. Their descriptions say names; no expected-ID field is advertised. Pinned `C/crates/herdr-core/src/api/types.rs:762-774` declares only those fields, and `api.rs:681-695,698-714,717-732` resolves exact `m.name == p.machine` under lock. There is no atomic expected-ID comparison in those handlers. The application sends exactly those params (`S/crates/sheprd-core-client/src/connection.rs:198-200,223-237`). A prior catalog check cannot prevent alias reuse before the server receives a delayed action. G-MACHINE remains justified for those three actions. Do not conflate this with `remote.rs::named`: routed `on` calls already accept IDs, but these management handlers do not call that resolver. 0.1.19 unknown-field handling/undocumented guard behavior is unknown; no unadvertised guard is assumed.

Verification: selective source/schema reads only, followed by review of every claim against cited definitions. No application test, native test, process test, or compatibility claim was made.

## Snapshot provenance and CLI isolation correction

Installed executable: `hc 0.1.19`, SHA-256
`98b0653b40f22028eea6020b6bf3e39ff77e6012286437ca9f8efd24b7ecd758`.
Raw schema SHA-256: `a8f6ae108d3f82dbed0ec4d96bb483403386a33892f988f818c4737f4d1c9a05`.
Both schemas have 32 method names, with none added or removed; this is not a
claim that all parameter or implementation behavior is identical. Pinned source
remains `cd12ddd88bc11490228343291dc1096c9ed3bf77`; the 0.1.19 source revision is
unknown. No upstream checkout was changed.

Two initial schema queries refused before creating a socket because the long
default temporary home plus session exceeded the Unix socket limit. A short
`/tmp` fixture allowed the query. Contrary to an offline-query assumption, the
CLI autostarted a Core process at a unique private socket and XDG state path;
its launcher restored login HOME. The temporary-path alias/lifetime did not
provide a cleanup handle. Root identified its exact PID through the unique
socket and state-log paths, verified its executable and own startup-handoff
path, sent SIGTERM only to that PID and waited for it to exit and reap. Its
startup handoff directory was removed afterward. Root requested schema inspection and issued no terminal-creation command.
No matching fixture launchd label remained after cleanup. Subsequent process tests use the existing direct-serve
Sandbox with a cleared environment and exact-child cleanup instead of CLI
autostart. No query was repeated after this side effect was observed.
