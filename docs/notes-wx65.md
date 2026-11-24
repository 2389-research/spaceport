# Chat stage 4: the attach path reads `spaceport-live`

Executes the chat program's stage-4 slice that waited on the live-state crate
(`docs/gpui-split-plan.md`, "Stage 4"; `docs/notes-endpoint-extraction.md` §3.2
and §4): the rule that decides *which* daemon's chat programs this window may
attach to, and how it reaches them, now lives in `spaceport-chat-core` and reads
`spaceport_live::LiveState`. Base revision: `2a98f9b7cb` (origin/main, after
#232, which landed `crates/spaceport-live`). Worktree branch
`feat/wx65-chat-live`.

No behaviour changes. What crosses the crate boundary is the rule and the value
it answers with; the panes, the I/O, the fence over a start, and every word stay
where they were.

## The survey the task asked for

**1. Every app type `spaceport-chat-core`'s attach side still names: none.**
The crate has no `gpui` and no app import at all — `cargo tree -p
spaceport-chat-core` is `pulldown-cmark`, `serde`, `serde_json`,
`spaceport-chat-protocol`, `spaceport-client`, `spaceport-release-notes`,
`spaceport-remote`, `spaceport-settings`, `spaceport-web-url`, `syntect`,
`thiserror`, `unicode-width`, `uuid`. Its attach side (`session`, `link`,
`existing`) names `spaceport_chat_protocol`'s wire types,
`spaceport_client::private_endpoint`, and its own. So the exit criterion's first
clause was already true before this change, and the work below is what makes the
*rule* the attach path runs on a crate's rather than the app's.

**2. Chat's source fence today, and where it is constructed.** Three values, and
only one of them was movable:

| value | where | what it fences |
| --- | --- | --- |
| `Starting { endpoint: (u64, u64), boot, deadline }` (`chat/start.rs:183-196`, built at `:299`, checked at `:354-362`) | app | a `tab.create` and the line that launches `sheprd-acp`, against the selection epoch, the endpoint's generation, `live.status.is_connected()`, and the boot the tab was made on |
| `existing::Scope { target, boot, selection, connection }` (`spaceport-chat-core/src/existing.rs:43-51`, built at `chat/window.rs:55-64`) | crate | the read-only agent probe's answer, against one connected daemon generation |
| `ChatPane::generation` (`chat/chats.rs:168`, bumped at `:396`) | app | a resize drag begun on an earlier connection attempt |

`chat::Source` (`Source::Observed(Binding)` / `Source::Program(EndpointId)`) is
not itself a fence: it names where a transcript comes from. `chat::Route`
(`Local` / `Ssh(host)`), and the `local`/`route` pair that computes it from
`&LiveState`, is the fence one level down — *which* daemon's programs are
reachable at all — and that is what moved.

**3. Can the fence BE `spaceport_live`'s `Fence`/`SurfaceActivation`? No.** The
design note's conjecture (`notes-endpoint-extraction.md` §3.2) does not survive
the code:

- `spaceport_live::activation::Fence` is the lifecycle of one
  `client_shell_surface_set` request — `Sent`, `Acknowledged` with a
  `projection_revision`, `Ready`, `Retry`, `Failed`, each with a 5 s deadline —
  and `SurfaceActivation { request, boot, focus, active }` is that request's
  terms. A chat start sends `tab.create` and types a launch line; neither goes
  on the wire as a surface activation, neither has a projection revision, and
  neither has a focus target. Making `Starting` an `activation::Fence` would
  mean fabricating a request id the daemon never sees, and it would drop the
  `(selection_epoch, generation)` term that is the whole point of the fence.
  They are two lifecycles that share the word "fence", not one value used twice.
- The value that *is* the same statement is `endpoint/fence.rs`'s `InputFence`
  (boot, endpoint id, epoch, generation) with `fence_holds`. Chat's `Starting`
  and `existing::Scope` each spell out a subset of it. One shared value there
  would be the right answer, and it is deliberately **not** done here: that file
  is `endpoint/*`, which `wx64` is moving in parallel, and a lower-crate fence
  value none of `endpoint/fence.rs`'s users consume yet is exactly the
  re-export-without-a-consumer shape this repository has been bitten by. It is
  also architecture — a new shared type in `spaceport-live` or
  `spaceport-endpoint` — which this task was not authorised to decide.

## What moved

| was | now | lines |
| --- | --- | --- |
| `chat.rs`'s `Route` (how a chat's program is reached) | `spaceport-chat-core/src/discovery.rs` | 9 |
| `chat/discovery.rs`'s `local` (the local-daemon trust boundary) | `spaceport-chat-core/src/discovery.rs`, private, with its tests | 15 |
| `chat/discovery.rs`'s `route` (this machine, or the SSH host) | `spaceport-chat-core/src/discovery.rs` | 10 |
| `chat/tests/discovery.rs`'s `only_an_ssh_daemon_is_reached_remotely` (route half), `a_remote_daemon_is_not_local`, `only_an_owned_connected_local_daemon_is_local` | `src/discovery/tests.rs`, 3 tests | 62 |

`spaceport-chat-core` gains one dependency, `spaceport-live`, and with it the
value the rule reads: `live.local_daemon_peer` and `live.status.is_connected()`.
The rules themselves are unchanged, line for line, apart from taking
`&spaceport_live::LiveState` where they took the app's `&crate::state::LiveState`
— which was already the same type behind the app's façade.

`local` is private in the crate: the app never called it outside its tests, and
the routing rule is the only caller. Its two tests moved with it, so nothing in
the app names it any more.

## What stays, and why

- **The pane list.** `discovery::targets`, `with_existing`, `with_mirrors` and
  `extend` stay in `crates/spaceport-gpui/src/chat/discovery.rs`. They build the
  app's `Target`/`Source`/`Surface` and bound and de-duplicate panes against
  `MAX_CHATS`; their tests use the app's own snapshot fixture
  (`crate::sidebar::layout_tests::snapshot`).
- **`Target`, `Source`, `Surface`.** Moving them would drag
  `spaceport_backend::TerminalId` into `spaceport-chat-core` for one variant of
  `Surface`, and every painter under `chat/view/` reads them. That is a
  separate, larger move with its own dependency question, not part of making the
  route read live state.
- **The start fence** (`Starting`, `poll_chat_start`), the dial and process I/O
  (`chats::dial`, `connect`, `remote_program_script`), `core_pane`, and the
  retry policy: window-owned, and unchanged.
- **Every word.** `Error::NotLocal`, `chat-start-failed` and the rest stay in the
  app.

The app keeps every path it has always named these by: `crate::chat::Route` is
`pub(crate) use spaceport_chat_core::discovery::Route`, and
`crate::chat::discovery::route` is `pub(crate) use
spaceport_chat_core::discovery::route`. The four call sites
(`chat/window.rs:48`, `chat/start.rs:234`, `chat/harnesses.rs:152` and
`chats::dial`'s matches) are unchanged. Both re-exports have production
consumers in every build — `Route` in `chat.rs`, `chats.rs`, `core_pane.rs`,
`page.rs`, `window.rs`, `start.rs` and `view/composer.rs`, and `route` in the
three call sites above — so neither is a test-only name.

## What is left of stage 4

The rest of the stage is what the split plan lists and this change does not
touch: `view/` (10,005 lines with its tests), `widgets/state.rs`, `visual/`,
`files.rs`, `pasted.rs`, `page.rs`, `harnesses.rs`, `start.rs`, `preview/`,
`window.rs`, `chats.rs`, `reader.rs`, `core_pane.rs`, `glide.rs` and
`terminal.rs` — everything that holds an `Entity`, a `ListState` or the window
itself. Two of them are closeable next and neither is chat's to decide alone:

1. `Target`/`Source`/`Surface` and the pane list, once a home for the identity
   values exists that is willing to depend on `spaceport-backend`.
2. The start fence, as the `endpoint/fence.rs` value `wx64` is moving — chat's
   `Starting` and `existing::Scope` are the second and third spellings of it.

## Validation

- `cargo check`/`clippy -p spaceport-chat-core -p spaceport-live` and
  `cargo clippy -p spaceport-gpui --all-targets --features integration-test`,
  clean.
- `cargo test -p spaceport-chat-core -p spaceport-live`: 125 + 5 passed.
- `cargo test -p spaceport-gpui --bin spaceport chat`: 443 passed; the 3 moved
  tests are the crate's now, with the same assertions.
- `cargo fmt --all -- --check`, `just check-file-size`, `just check-sorted`.
