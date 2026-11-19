# Historical error-copy proposal from PR 123

This note records PR 123's original proposal, before integrating the `cx2r` review fixes.
Its tables preserve historical wording, including superseded recovery advice.
Use [the final integration evidence](evidence/review-jargon-main-integration.md) for current behavior.
The active [error-language evidence](evidence/review-error-language.md) also covers `cx2r`.

Current guidance preserves saved connection data.
Timed-out requests may have happened; check the result before retrying.
Timed-out input may have arrived; choose Terminal → Reconnect.
Shared message-size errors describe the limit without blaming a message sender.

## Where the user actually reads these strings

- **The connection notice.** `sheprd_client::connect` turns the worker's final
  `Error` into `ClientEvent::Disconnected { reason, .. }` with
  `error.to_string()` (`crates/sheprd-client/src/connect.rs`), and
  `LiveState::apply` shows that string in a notice verbatim
  (`crates/sheprd-gpui/src/state.rs`). So the `Display` of every
  `sheprd_client::Error` and `sheprd_protocol::Error` is *literally* the text
  on screen — a toast reading "surface before snapshot", "patch before
  baseline", or "client is disconnected" before this change.
- **Localized messages.** `i18n::Localize` mirrors each of those `Display`
  strings into an `en-US` Fluent message, and
  `errors::tests::assert_localized` requires the two to be equal. Every
  `Display` change here is therefore a paired change in
  `crates/sheprd-gpui/locales/en-US/error-*.ftl`.
- **The command line.** `spaceport COMMAND` prints `error.to_string()` for
  every error it reaches (`control/client.rs::finish`), and the app's answers
  over the control socket are `Display`/`localize()` text too.
- **Control protocol.** `sheprd-control-protocol`'s `Error` `Display` strings
  are what `spaceport` prints when the socket itself fails.

Nothing about a variant, a field, a wire format, or an error category changed:
only `#[error("…")]` text, the matching Fluent values, and the comments that
describe them.

## One name per thing

| Thing | The app and `docs/cli.md` call it | Jargon found in error text | Settled as |
| --- | --- | --- | --- |
| Herdr running on a machine | "daemon" (`daemon-warning.ftl`, `settings*.ftl`, `docs/cli.md`) | "Herdr server", "h… server", "server" | **the daemon / the Herdr daemon** |
| A machine the app connects to | "host" (`spaceport host list`, `docs/cli.md` tables) | "endpoint", "remote endpoint" | **host** (the word "endpoint" is gone from user text) |
| A saved entry in the sidebar | "device" (`devices.ftl`: Add Device, All Devices) | — | unchanged; "device" is the user's word for the saved entry, "host" for the connection |
| A terminal's screen area | "pane" (`pane-menu.ftl`, `docs/cli.md`) | "surface" | **pane** / "a pane screen" |
| One message on the connection | — | "frame", "framing" | **a message** |
| One run of the daemon | "session" (`sessions-menu.ftl`, `error-palette-no-session`) | "boot", "snapshot boot ID", "snapshot revision" | **the daemon's session** |
| The SSH connection to a host | — | "SSH bridge", "the bridge" | **the SSH connection** |
| This app | "Spaceport", "this app" | "client" (`client I/O failed`, `… requires a Linux or macOS client`, "SSH … client") | **this app / Spaceport** |
| How a screen is encoded | — | "codec", "endpoint generation", "endpoint protocol" | **protocol**, "protocol versions" |

Message IDs keep their old spelling (`error-client-surface-before-snapshot`,
…). Per `docs/i18n.md` an ID is a stable identifier, not English text; the
meaning is unchanged, so the ID stays and only the value moves.

`herdr update` (a command to run), `checkout`, `host`, `workspace`, `pane`,
`tab`, and config keys stay exactly as they were: those are the user's words
or names the user types.

## Before → after

### `sheprd_client::Error` (`crates/sheprd-client/src/error.rs`) and `error-client.ftl`

| Variant / message ID | Before | After |
| --- | --- | --- |
| `Io` / `error-client-io` | client I/O failed: {0} | Could not talk to the daemon: {0} |
| `Json` / `error-client-json` | invalid endpoint JSON: {0} | The daemon's answer was not valid JSON: {0} |
| `CatalogSchema` / `error-client-catalog-schema` | invalid endpoint catalog schema | The saved host list cannot be read; update Spaceport, or remove it to start over. |
| `SelectionSchema` / `error-client-selection-schema` | invalid endpoint selection schema | The saved host selection cannot be read; remove it so Spaceport can write a new one. |
| `Full` / `error-client-full` | client command queue is full | Too many commands are waiting for the daemon; try again. |
| `Disconnected` / `error-client-disconnected` | client is disconnected | The app is not connected to the daemon. |
| `InputWriteTimeout` / `error-client-input-write-timeout` | terminal input write timed out; reconnect required | The daemon did not take the terminal input in time; reconnect. |
| `MissingBootId` / `error-client-missing-boot-id` | invalid client command: snapshot boot ID required | That command needs the daemon's session first; reconnect and try again. |
| `CommandBoot` / `error-client-command-boot` | command does not match a ready snapshot boot | The daemon's session changed before the command could run; reconnect and try again. |
| `UnsupportedMethod` / `error-client-unsupported-method` | method not advertised by endpoint | This Herdr daemon does not offer that request; run `herdr update` and reconnect. |
| `UnsupportedSurfaceInterest` / `error-client-unsupported-surface-interest` | surface interest capabilities not advertised by endpoint | This Herdr daemon cannot send pane screens the way this app asks; run `herdr update` and reconnect. |
| `EmptySurface` / `error-client-empty-surface` | surface dimensions must be nonzero | A pane screen was asked for with no rows or columns; nothing was sent. |
| `GeometryLimit` / `error-client-geometry-limit` | surface geometry exceeds endpoint limits | The pane's screen is larger than the daemon allows; make the pane or window smaller. |
| `InvalidSession` / `error-client-invalid-session` | invalid session name | That is not a valid session name. |
| `NoLocalSocket` / `error-client-no-local-socket` | remote endpoints have no local socket path | A host reached over SSH has no local socket to connect to. |
| `Cancelled` / `error-client-cancelled` | client stopped | The connection was stopped. |
| `EventReceiverDropped` / `error-client-event-receiver-dropped` | event receiver dropped | The connection's events stopped. |
| `SocketClosed` / `error-client-socket-closed` | socket closed | The daemon closed the connection. |
| `PartialFrameTimeout` / `error-client-partial-frame-timeout` | partial frame timed out | The daemon started a message and never finished it; reconnect. |
| `FramePrefix` / `error-client-frame-prefix` | invalid frame prefix | The daemon sent a message this app cannot read (bad message prefix); reconnect. |
| `FrameLength` / `error-client-frame-length` | invalid frame length | The daemon sent a message this app cannot read (bad message length); reconnect. |
| `HealthTimeout` / `error-client-health-timeout` | endpoint health check timed out | The host did not answer its health check in time; check the network and reconnect. |
| `HandshakeTimeout` / `error-client-handshake-timeout` | handshake/snapshot timed out | The host did not finish connecting in time; check the network and reconnect. |
| `RequestTimeout` / `error-client-request-timeout` | endpoint request timed out; not replayed | The daemon did not answer in time; the request was not sent again. |
| `ExpectedWelcome` / `error-client-expected-welcome` | expected stable endpoint welcome | The host did not greet this app as expected; run `herdr update` and reconnect. |
| `LegacyDaemon` / `error-client-legacy-daemon` | this Herdr server predates the endpoint protocol; run `herdr update` and reconnect | This host's Herdr daemon is older than this app needs; run `herdr update` and reconnect. |
| `ClosedBeforeWelcome` / `error-client-closed-before-welcome` | Herdr server closed the connection without answering; it is likely older than {version}, which this app requires. Run `herdr update` and reconnect | The host's Herdr daemon closed the connection without answering; it is likely older than {version}, which this app requires. Run `herdr update` and reconnect |
| `EndpointGeneration` / `error-client-endpoint-generation` | Herdr server {server_version} speaks endpoint generation {generation}, this app speaks {ours}; {advice} | This app and the host's Herdr daemon (version {server_version}) speak different protocol versions ({ours} here, {generation} there); {advice} |
| `WelcomeKind` / `error-client-welcome-kind` | expected endpoint.welcome.v1 | The host's Herdr daemon sent a greeting this app cannot read; run `herdr update` and reconnect. |
| `WelcomeRejected` / `error-client-welcome-rejected` | {code}: {message} | The host's Herdr daemon refused the connection ({code}): {message} |
| `IncompatibleCodecs` / `error-client-incompatible-codecs` | incompatible endpoint generation/codecs | This app and the host's Herdr daemon cannot agree on a protocol; run `herdr update` and reconnect. |
| `MissingSurfaceInterest` / `error-client-missing-surface-interest` | Herdr server {server_version} lacks safe surface interest support; run `herdr update` and reconnect | The Herdr daemon on this host (version {server_version}) cannot send pane screens the way this app asks; run `herdr update` and reconnect |
| `MissingHealthCheck` / `error-client-missing-health-check` | Herdr server {server_version} on this SSH host lacks the health_check capability; run `herdr update` there and reconnect | The Herdr daemon on this SSH host (version {server_version}) has no health check this app can use; run `herdr update` there and reconnect |
| `SnapshotIdentity` / `error-client-snapshot-identity` | endpoint boot changed or snapshot revision regressed; reconnect required | The daemon's session changed, or its screen data went backwards; reconnect. |
| `SurfaceBeforeSnapshot` / `error-client-surface-before-snapshot` | surface before snapshot | A pane screen arrived before the daemon's session data; reconnect. |
| `SurfaceIdentity` / `error-client-surface-identity` | invalid surface identity/revision | A pane screen arrived for the wrong pane or an out-of-date revision; reconnect. |
| `PatchBeforeBaseline` / `error-client-patch-before-baseline` | patch before baseline | A pane screen update arrived before the full screen it belongs to; reconnect. |
| `EncodedSurfaceBeforeBaseline` / `error-client-encoded-surface-before-baseline` | encoded surface before baseline | An encoded pane screen arrived before the full screen it belongs to; reconnect. |
| `SurfaceEncodingNotNegotiated` / `error-client-surface-encoding-not-negotiated` | surface encoding was not advertised by endpoint | This Herdr daemon did not offer the screen encoding this app asked for; run `herdr update` and reconnect. |
| `ResponseBoot` / `error-client-response-boot` | response boot mismatch | The daemon's session changed while the request was in flight; try again. |
| `ResponseLimit` / `error-client-response-limit` | response limit exceeded | The daemon's answer was larger than this app accepts. |
| `UnsolicitedResponse` / `error-client-unsolicited-response` | unsolicited response | The daemon answered a request this app never sent; reconnect. |
| `ResponseId` / `error-client-response-id` | response ID mismatch | The daemon's answer did not match the request; reconnect. |
| `BridgeIncompatible` / `error-client-bridge-incompatible{,-version}` | Herdr{ (version X)} on this host cannot serve this app; {advice} | The Herdr daemon{ (version X)} on this host cannot serve this app; {advice} |
| `RemoteCommand` / `error-client-remote-command` | remote command failed ({status}) | The command on the host failed ({status}). |
| `RemoteOutput` / `error-client-remote-output` | remote command returned unexpected output | The command on the host printed something this app did not expect. |
| `ScriptUnsupported` / `error-client-script-unsupported` | host scripts require a Linux or macOS client | Host scripts need this app running on Linux or macOS. |
| `UploadUnsupported` / `error-client-upload-unsupported` | SSH file transfer requires a Linux or macOS client | SSH file transfer needs this app running on Linux or macOS. |
| `UploadSizeOverflow` / `error-client-upload-size-overflow` | file transfer total size exceeds u64 | The files are too large to transfer at once. |
| `UploadCleanupPath` / `error-client-upload-cleanup-path` | cleanup requires unchanged absolute paths returned by upload_files | Cleaning up the files left on the host needs the paths the upload returned. |
| `CatalogNotFile` / `error-client-catalog-not-file` | endpoint catalog is not a regular file | The saved host list is not a regular file; remove it so Spaceport can write a new one. |
| `CatalogLimit` / `error-client-catalog-limit` | endpoint catalog exceeds storage limit | The saved host list is larger than Spaceport stores; remove a host. |
| `CatalogVersionOrCount` / `error-client-catalog-version-or-count` | unsupported catalog version or too many profiles | The saved host list is in a format this app does not know, or it lists too many hosts; update Spaceport or remove a host. |
| `ProfileId` / `error-client-profile-id` | invalid or duplicate endpoint profile id | A saved host has an ID this app cannot use, or two hosts share one; remove one. |
| `ProfileLabel` / `error-client-profile-label` | invalid endpoint label | A saved host's label cannot be used. |
| `SelectionNotFile` / `error-client-selection-not-file` | endpoint selection is not a regular file | The saved host selection is not a regular file; remove it so Spaceport can write a new one. |
| `SelectionLimit` / `error-client-selection-limit` | endpoint selection exceeds storage limit | The saved host selection is larger than Spaceport stores; remove it to start over. |
| `SelectionVersion` / `error-client-selection-version` | unsupported endpoint selection version | The saved host selection is in a format this app does not know; update Spaceport. |
| `SelectionUnavailable` / `error-client-selection-unavailable` | selected endpoint is absent or disabled | The selected host is not there, or it is turned off; pick another host. |
| `StoragePath` / `error-client-storage-path` | invalid storage path | Spaceport's saved data has a path it cannot use. |
| `StorageDestinationNotFile` / `error-client-storage-destination-not-file` | storage destination is not a regular file | Spaceport's saved data is not a regular file. |
| `Endpoint` (empty message) / `error-client-endpoint` | Invalid daemon error | The daemon refused the request without saying why. |
| `ResponseSchema` / `error-client-response-schema` | invalid endpoint response | The daemon's answer is not one this app can read. |
| `ResponseMissingResult` / `error-client-response-missing-result` | endpoint response carried neither a result nor an error | The daemon's answer carried neither a result nor a reason. |
| `ResponseType` / `error-client-response-type` | endpoint answered with a result of another method | The daemon answered a different request than the one sent. |
| `ForwardSpawn` / `error-client-forward-spawn` | could not start SSH for the port forward | Could not start SSH for the port forward. |
| `ForwardLocalPort` / `error-client-forward-local-port` | no local port is free to forward to | No local port is free to forward to. |
| `ForwardControl` / `error-client-forward-control` | could not create the port forward's private control directory | Could not prepare the port forward's private folder. |
| `ForwardTimeout` / `error-client-forward-timeout` | SSH did not connect in time; check host trust and authentication | SSH did not connect in time; check host trust and authentication. |
| `ForwardRefused` / `error-client-forward-refused` | SSH could not listen on a local port ({status}) | SSH could not listen on a local port ({status}). |
| `ForwardExit` / `error-client-forward-exit` | SSH port forward ended ({status}); check host trust, authentication, and the network | The SSH port forward ended ({status}); check host trust, authentication, and the network. |

### `sheprd_protocol::Error` (`crates/sheprd-protocol/src/error.rs`) and `error-protocol.ftl`

These reach the notice through `sheprd_client::Error::Protocol`, which is
`#[from]`/transparent. Every one said what failed in wire terms only; each now
says what arrived in a person's words, keeping the fact that only a technician
would use (that the cells did not match the size, that the row was outside the
screen, and so on).

| Variant / message ID | Before | After |
| --- | --- | --- |
| `Io` | protocol I/O failed: {0} | Could not talk to the daemon: {0} |
| `Encode` | frame encoding failed: {0} | A message could not be written for the daemon: {0} |
| `Decode` | frame decoding failed: {0} | A message from the daemon could not be read: {0} |
| `FrameLimit` | frame exceeds limit | A message from the daemon is larger than this app accepts. |
| `EmptyFrame` | empty frame | The daemon sent an empty message. |
| `TrailingBytes` | trailing frame bytes | The daemon sent extra bytes after a message. |
| `ClipboardImageSize` | clipboard image must contain 1 byte through 16 MiB | A clipboard image must hold 1 byte through 16 MiB. |
| `ClipboardImageExtension` | unsupported clipboard image extension | A clipboard image is in a format this app cannot read. |
| `ClipboardImageTarget` | clipboard image target must contain 1 through 1024 bytes | A clipboard image's target must hold 1 through 1024 bytes. |
| `CellCount` | cell count does not match frame dimensions | A pane screen arrived with the wrong number of cells for its size. |
| `HyperlinkIndex` | invalid hyperlink index | A pane screen named a link it does not carry. |
| `CursorBounds` | cursor outside frame | A pane screen put the cursor outside the screen. |
| `PatchIdentity` | surface patch identity mismatch | A pane screen update named a pane or revision it does not match. |
| `PatchRowBounds` | surface patch row outside frame | A pane screen update named a row the screen does not have. |
| `PatchGeometry` | surface patch changed pane geometry | A pane screen update tried to change the pane's size. |
| `PatchCursorBounds` | patch cursor outside frame | A pane screen update put the cursor outside the screen. |
| `EncodingLimit` | surface encoding exceeds the frame limit | An encoded pane screen is larger than a message may be. |
| `EncodingPayload` | invalid surface encoding payload: {0} | An encoded pane screen could not be decoded: {0} |
| `EncodingJson` | invalid surface encoding JSON: {0} | An encoded pane screen's JSON could not be read: {0} |
| `ScrollCount` | surface scroll has an invalid scroll count | A pane screen scroll asked for an impossible number of lines. |
| `ScrollTruncated` | surface scroll is truncated | A pane screen scroll ended early. |
| `ScrollPayload` | surface scroll does not carry a pane patch | A pane screen scroll came without the screen lines it moves. |
| `ScrollBounds` | surface scroll region is outside the frame or overlaps another | A pane screen scroll named rows the screen does not have. |
| `SurfaceBaseline` | surface update does not match its baseline | A pane screen update does not match the screen it belongs to. |
| `DeltaGridLimit` | surface delta grid exceeds its limits | A pane screen update is larger than the size limits this app accepts. |
| `DeltaMetadataCells` | surface delta metadata carries cells | A pane screen update carried cells where its metadata belongs. |
| `DeltaSpan` | surface delta span is outside its grid, unsorted, or over budget | A pane screen update named cells outside the screen, out of order, or too many. |
| `DeltaPopup` | surface delta popup update does not match its popup | A pane popup update does not match the popup it belongs to. |

### Daemon startup (`crates/sheprd-gpui/src/daemon/error.rs`) and `error-daemon.ftl`

| Variant / message ID | Before | After |
| --- | --- | --- |
| `Cancelled` | daemon startup cancelled | Daemon startup cancelled. |
| `Exited` | herdr server exited before accepting connections: {status}. Check the Herdr server log. | The Herdr daemon exited before accepting connections: {status}. Check the daemon's log. |
| `Spawn` | Could not start herdr server: {error}. Use Terminal > Reconnect to retry. | Could not start the Herdr daemon: {error}. Use Terminal > Reconnect to retry. |
| `Timeout` | Timed out waiting for herdr server at {path}. Check the Herdr server log and use Terminal > Reconnect. | Timed out waiting for the Herdr daemon at {path}. Check the daemon's log and use Terminal > Reconnect. |

The "Herdr server" the user was told to look for a log of is the daemon the
rest of the app calls a daemon; the menu path `Terminal > Reconnect` was
checked against `menus.rs` and is correct.

### Installing Herdr (`crates/sheprd-gpui/src/daemon.rs`)

`MissingInstallation`'s `Display` is what the client's `Io` wraps, and the
window shows it in the notice and decides on its own "Herdr is missing" card
from the type:

| Before | After |
| --- | --- |
| Could not start herdr server: {0} | Herdr is not installed: {0} |

The notice now reads "Could not talk to the daemon: Herdr is not installed: No
such file or directory (os error 2)".

### The app's own root error (`crates/sheprd-gpui/src/error.rs`) and `error.ftl`

| Variant / message ID | Before | After |
| --- | --- | --- |
| `NoSnapshot` / `error-no-snapshot` | No current daemon snapshot. | The daemon's current session data is not loaded; reconnect. |
| `Request` / `error-request` | {method}: {error} | Could not send {method} to the daemon: {error} |

`Request` is raised where a request is queued, so "Could not send" is the fact;
the daemon's method name (`workspace.create`) stays as the detail.

### The SSH connection (`crates/sheprd-client/src/ssh/error.rs`) and `error-client-ssh.ftl`

| Message ID | Before | After |
| --- | --- | --- |
| `error-client-ssh-closed` | SSH bridge closed; check host trust, authentication, and remote Herdr installation | The SSH connection closed; check host trust, authentication, and remote Herdr installation. |
| `error-client-ssh-refused` | SSH bridge closed: {error} | The SSH connection closed: {error} |

### Direct terminal attach (`crates/sheprd-client/src/terminal_attach/error.rs`) and `error-client-terminal-attach.ftl`

| Message ID | Before | After |
| --- | --- | --- |
| `error-client-terminal-attach-protocol` | terminal attach framing failed | The daemon sent a message this app cannot read while attaching the terminal. |
| `error-client-terminal-attach-rejected` | Herdr refused the terminal hello (protocol {version}) | Herdr refused the terminal attach (protocol {version}) |
| `error-client-terminal-attach-unexpected` | Herdr answered the terminal hello with an unexpected message | Herdr answered the terminal attach with a message this app did not expect. |

### Control protocol (`crates/sheprd-control-protocol/src/error.rs`) and `error-control.ftl`

| Message ID | Before | After |
| --- | --- | --- |
| `MessageSize` / `error-control-message-size` | Control request or response exceeds {limit} bytes | The request or the answer is larger than {limit} bytes. |

The rest of `error-control.ftl` already speaks to the person at the terminal
("Spaceport is not running, or its control socket … is unreachable", "Another
Spaceport already serves …") and was left alone.

## Regression tests

- `crates/sheprd-gpui/src/i18n/errors/tests.rs` grew
  `WIRE_WORDS` + `assert_no_wire_words`: no message in the client, protocol,
  daemon, or root family may say `boot`, `bridge`, `codec`, `endpoint`,
  `frame`, `herdr server`, or `surface` to the person reading it. It runs over
  the same variant lists the localization test already walks, so a new variant
  that falls back to wire words fails the build's tests.
  The two protocol variants that carry bincode's own diagnostic
  (`Decode`, `Encode`) sit out the vocabulary check: that text is the codec's,
  kept verbatim as `docs/i18n.md` requires, and bincode prints our error type
  as `Debug` inside it.
- `assert_localized` (already there) keeps each English message equal to its
  `Display`, so the FTL and the Rust text cannot drift apart.
- Every test that asserted an old string was updated to the new one:
  `sheprd-client`'s `tests/errors.rs`, `tests/commands.rs`, `tests/framing.rs`,
  `tests/handshake.rs`, `tests/health.rs`, `tests/partial_input.rs`,
  `tests/session_state.rs`, `tests/surfaces.rs`, `tests/version_mismatch.rs`,
  `tests/error_kinds.rs`, `catalog/tests.rs`, `response/tests.rs`,
  `ssh/tests/incompatible.rs`, `ssh/failure/tests.rs`; `sheprd-gpui`'s
  `error/tests.rs`, `find/tests.rs`, `pane_menu/tests.rs`,
  `tab_menu/tests.rs`, `menu/worktree_open_tests/listing.rs`,
  `port_forward/tests.rs`, `state/tests/connection_status.rs`,
  `window/accessibility/tests/insertion.rs`,
  `window/tests/{core_unavailable,herdr_missing}.rs`,
  `endpoint/lifecycle_tests/herdr_missing.rs`, `daemon/tests.rs`.
- `tests/commands.rs` now names the error it actually gets: a snapshot identity
  failure, not a bare "boot changed" substring.

## Deliberately left

- **Words that are the user's own.** `herdr update` (a command they type),
  `checkout`, `branch`, `workspace`, `pane`, `tab`, `host`, `device`, `SSH`,
  `PATH`, `HOME`, `XDG_*`, and every config key (`keybindings.`, `pane_keys.`,
  `chat.env`, `sidebar.activity_bar`) stay as written: they are things the user
  reads in a config file or types. `Terminal > Reconnect` stays because that is
  the menu item's name.
- **The host-script family** (`error-client-script-*`, 11 messages). It says
  "host script" consistently, names no wire vocabulary, and reads as advice
  already; only capitalization differs from the sentences above it. Changing
  them is style, not jargon, and is worth its own change.
- **The SSH upload family** (`error-client-upload-*`). Same reasoning: "SSH
  file transfer" is the user's operation, not an internal name. The two members
  that named the app a "client" and spoke of `u64` were changed.
- **`version-mismatch.ftl` and `status-bar.ftl`** still say "The Herdr server …
  / Starting Herdr server…". They are not error strings — one is the
  reconnection card, one a status line — and they are the same object as the
  daemon, so they are the obvious next step for whoever settles this word in
  the whole window. Leaving them is a scope call, not an oversight.
- **`error-chat-protocol.ftl`** keeps "chat frame", "EntryId", and "chat
  attach protocol". Those name `sheprd-chat-protocol`'s own wire, the IDs are
  kept as the protocol writes them by design, and the obvious fix for a person
  reading one is already in the message that wraps it.
- **Failure advice was not invented.** Where the app cannot know what the
  person should do (a pane screen arriving before its baseline), the message
  says what happened and that reconnecting is the way out, which is what the
  code does next; no message tells the user to do something the app cannot
  offer.
