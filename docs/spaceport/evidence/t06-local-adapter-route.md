# T06 local chat route inventory

Current qualification: [the desktop lifecycle cut](t06-chat-lifecycle-policy-removal.md)
removes the conversion/recovery paths described below as remaining in older
snapshots. [The newer detach/permission process fixtures](t06-adapter-detach-process.md)
use explicit Core 0.1.19 and the updated adapter. Native, remote and real-agent
parity remain open. Earlier source/version results below retain their historical
scope; [the duplex audit](t06-core-duplex-route-audit.md) narrows the external
contract without requiring a chat-specific RPC.

Pinned Sheprd `806b49870ac7761b98cbcea4bd115aa2f2b00d75` supplies the current local Core route. `chat/core_pane.rs` starts the host-side `sheprd-acp` program as a Core terminal with a random endpoint in `HERDR_ACP_ENDPOINT` and an `acp` terminal label. Discovery reads that label from Core terminal facts, rejects invalid endpoint IDs and machine-qualified terminal addresses, and offers a native overlay only for local driven panes. `chat/chats.rs` then dials the adapter's local socket or Windows pipe. The attach protocol and renderer are separate from Core terminal bytes. Core owns the adapter process as a terminal program; the app still has legacy conversion and restore paths to remove.

The current adapter address is under the inherited `herdr/acp` state directory on Unix, or `herdr-acp-<id>` pipe on Windows (`sheprd-chat-protocol/src/endpoint.rs`). This is an identity-isolation issue for the Spaceport adapter cut; changing it while legacy Herdr 1 routes still consume the same protocol would need a scoped migration. The local route does not supply a remote Core attach channel: the Core target filter excludes `machine/terminal`, and the only remote chat dial in `chat/chats.rs` runs a separate SSH command for the legacy backend. The inspected Core method table has no structured chat method. G-CHAT therefore still needs a Core-owned remote adapter route with discovery, target identity, authentication and permission semantics.

- `CARGO_TARGET_DIR=/tmp/spaceport-t00-target cargo +1.99.0 test --locked -p sheprd-gpui --bin sheprd chat::core_pane::tests`: 7 passed, including local label and remote exclusion checks.
- `CARGO_TARGET_DIR=/tmp/spaceport-t00-target cargo +1.99.0 test --locked -p sheprd-chat-protocol --lib`: 66 passed, including endpoint shape, framing and reducer checks.

These are source and pure contract checks. No live `sheprd-acp` process, real ACP agent, GUI chat overlay, permission race, disconnect, remote Core transport, or native Windows/Linux route was tested. A14–A17, A54 and A55 remain `not_run`; T06 remains blocked on G-CHAT. The existing adapter must not be described as tested support on the strength of these checks.

## Host-side fixture proof at `388fc8a8`

The local contract has three source-backed parts. `chat/core_pane.rs` passes the same random `EndpointId` through `HERDR_ACP_ENDPOINT` and the Core terminal's `acp` label, then discovers only valid labels on local terminal IDs; its seven pure tests above cover that decision. `sheprd-acp/src/attach.rs` listens at that ID through `sheprd-client::private_endpoint`, requires a hello before commands, and forwards a command and its connection ID to the session. `sheprd-acp/src/session/commands.rs` accepts a permission answer only for a pending request and one of its offered options; the first accepted answer resolves the request, so a later answer is rejected. The latter two parts have isolated fixtures that run no agent or Core daemon:

- `CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/tmp/spaceport-t06-chat-target cargo +1.99.0 test --locked -p sheprd-acp --bin sheprd-acp attach::tests::`: 11 passed. The private socket fixture checks hello, command/ack routing, detach, version refusal, invalid ordering, endpoint IDs, stale socket cleanup and revision handling.
- `CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/tmp/spaceport-t06-chat-target cargo +1.99.0 test --locked -p sheprd-acp --bin sheprd-acp session::tests::permissions::`: 5 passed. The session fixture checks one answer from the GUI reaches the agent response and both GUI/in-pane views, a later answer is rejected, unknown options and unrelated agent requests are refused, and a permission names its tool call.
- `CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/tmp/spaceport-t06-chat-target cargo +1.99.0 test --locked -p sheprd-client --lib private_endpoint::tests::`: 8 passed on macOS. This includes a real temporary Unix socket and asserts directory mode `0700`, socket mode `0600`, and a second listener's refusal. It does not prove the Windows named-pipe owner check.

These component tests establish local parts separately, not a Core-launched adapter with an actual ACP agent or a native GUI run. The adapter address still uses the inherited `herdr/acp` namespace. The smallest remaining G-CHAT external dependency is a Core-owned way to discover and attach to a remote adapter by qualified terminal identity, with an authenticated byte route and the same permission request/response semantics. `chat/core_pane.rs` filters out machine-qualified terminal IDs; `chat/chats.rs` has only the legacy `Route::Ssh` child for remote attach, while `sheprd-core-client/src/method.rs` has no chat stream method. These component tests alone cannot prove A14–A17/A54, especially remote A15 or a live two-view race. T06 remains `blocked`; G-CHAT remains `open`.

## Private Core host-adapter process contract

At source base `33956560`, `sheprd-core-client/src/tests/live/acp.rs` now runs an opt-in Unix process scenario. The existing private Core `Sandbox` clears inherited environment, sets a private `HC_SOCKET`, HOME and XDG roots, sets `HC_LAUNCHD_DIRECT` to the test PID, checks private `server stop`, and reaps its exact `serve` child before removing the root. The test explicitly names the Core and `sheprd-acp` binaries. Core `terminal.run` starts the real `sheprd-acp --headless -- python3 -I <test peer>` with one endpoint ID in both the `acp` label and `HERDR_ACP_ENDPOINT`; the test checks Core's terminal label and connects to the adapter socket inside the private XDG state root. The Python file is a fixed test ACP peer, not an application mock mode or a provider.

Two attached clients observe one pending permission. The first sends `allow` and gets an accepted acknowledgement. A later `deny` from the second gets `UnknownPermission`. A new attach receives a snapshot and subsequent changes with no pending permission and the peer's `chosen allow` response. Changing the peer response to `chosen deny` made the transcript assertion fail; restoring the response made it pass. This proves the local Core-launched process and adapter attach/permission wire seam on this macOS fixture. It does not exercise the GPUI view, a real agent, process survival after GUI quit, a remote route, or native Windows/Linux behavior. A14–A18 and A54–A55 remain `not_run`; G-CHAT and T06 remain open/blocked.

- Pinned private Core binary: `/tmp/spaceport-t00-target/debug/herdr-core`, SHA-256 `30eee3cb46bad36ae483c8c459c0ad3acb394089fcf690a9390e1e470196f58b`.
- `CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/tmp/spaceport-t06-local-adapter-target cargo +1.99.0 build --locked -p sheprd-acp --bin sheprd-acp`: passed; resulting adapter SHA-256 `059b730a21633cd3d73d86833249126b33351b320eb015a4703d16f56b300195`.
- `HERDR_CORE_TEST_BINARY=/tmp/spaceport-t00-target/debug/herdr-core SHEPRD_ACP_TEST_BINARY=/tmp/spaceport-t06-local-adapter-target/debug/sheprd-acp CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/tmp/spaceport-t06-local-adapter-target cargo +1.99.0 test --locked -p sheprd-core-client --features adapter-live-test --lib live::acp::a_core_owned_adapter_shares_one_permission_result -- --ignored --nocapture`: one passed on macOS. The opt-in feature keeps the ordinary `just test-core` fixture independent of the adapter binary.
- `CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/tmp/spaceport-t06-local-adapter-target cargo +1.99.0 test --locked -p sheprd-core-client --lib`: 63 passed, 11 live tests ignored.
- `just format-check`, `just check-file-size`, `just check-sorted`, `uv run --with jsonschema==4.26.0 python tools/spec.py check`, `uv run --with jsonschema==4.26.0 python tools/test_spec.py`, and `git diff --check`: passed. The first bare `python3 tools/spec.py check` could not load JSON Schema because that interpreter lacked the pinned package; the isolated `uv` run passed.
- `CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/tmp/spaceport-t06-local-adapter-target cargo +1.99.0 clippy --locked -p sheprd-core-client --all-targets --all-features`: passed with one existing `clippy::type_complexity` warning in unchanged `connection.rs:90`; adding `-- -D warnings` failed on that same warning. No warning originated in this test slice.

At merged revision `302aee5b`, the same private Core scenario passed with the
main worktree's `sheprd-core-client` test binary (one passed, 74 filtered).
It used the separately built adapter binary named above and the pinned Core
binary; both ran only in the test's private sandbox. This rerun confirms the
local subset survived integration. It adds no native GUI, real-agent, or remote
chat claim. T06 and G-CHAT stay blocked/open; A14–A18 and A54–A55 stay
`not_run`.

## Pinned Core method clarification at `7440fad3`

Read-only inspection of `/private/tmp/spaceport-core-reference` confirmed
Core revision `cd12ddd88bc11490228343291dc1096c9ed3bf77`. Its generated
`schema/hc.openrpc.json` has 32 methods and no chat or ACP method.
`crates/herdr-core/src/api.rs` defines `terminal.attach` as a framed raw
terminal stream. `remote.rs::route` includes that method for a named remote
terminal, and `remote/viewers.rs::attach` relays its channel both ways and
reattaches after a line loss. This is evidence for remote *terminal* viewing,
not for structured adapter messages or permission answers.

The adapter address comes from the endpoint label but resolves only to the
client's local `herdr/acp` socket or Windows pipe
(`sheprd-chat-protocol/src/endpoint.rs`). `chat/core_pane.rs::targets`
rejects terminal IDs containing `/`; `chat/chats.rs::connect` uses that local
address for Core targets. Its SSH process branch serves the legacy route and
does not turn the Core raw stream into an authenticated adapter attach.
`sheprd-acp/src/session/commands.rs::answer` rejects an unknown permission
or option, as the prior private fixture exercised; no remote path was tested.

The missing external contract is a Herdr-owned way for a remote Core
terminal's adapter endpoint to be discovered with stable machine/terminal
identity and reached over authenticated structured transport, including
permission, reconnect and attach lifetime semantics. The method shape is
unspecified; no Spaceport RPC or relay is proposed. This source audit ran no
process, agent, GUI or network test and changes no application code. T06 and
G-CHAT remain blocked/open; A14–A17 and A54–A55 remain `not_run`.
