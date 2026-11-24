# Endpoint stage 3: `spaceport-live`, the live-state crate

The prerequisite stage of `docs/notes-endpoint-extraction.md` §6, shared with the
chat program's stage 4 and the sidebar's `&LiveState` reads. Base revision:
`3c3e0b5b68` (origin/main, after #228, #229, #230, #231). Worktree branch
`feat/wx61-live-state`.

It changes no behaviour. What crosses the crate boundary is the *value and its
rules*; every word, every effect, and the window's two copies of the selected
endpoint's state stay where they were.

## What moved

| was | now | lines |
| --- | --- | --- |
| `src/state.rs` (`LiveState`, `ConnectionStatus`, `apply`, `only_surface_changed`, `Answer`, `RenameResult`, `SurfaceActivation`, `AnnouncementDismissal`, the sanitizers) | `spaceport-live/src/state.rs` | 825 → 795 |
| `src/state/activation.rs` (`Fence`, `Failure`, `Transition`, `TIMEOUT`) | `spaceport-live/src/activation.rs` | 317 |
| `src/state/projection.rs` (the coherent displayed pair, `sound_live`, `latest_snapshot`) | `spaceport-live/src/projection.rs` | 171 → 168 |
| `src/state/agent_recency.rs` | `spaceport-live/src/agent_recency.rs` | 57 → 56 |
| `src/notice.rs` (`Notices`, `Cause`, `HeldNotice`) | `spaceport-live/src/notice.rs` | 165 |
| `src/notice/tests.rs` | `spaceport-live/src/notice/tests.rs` | 77 |
| the model half of `src/notifications.rs` (`Notice`, `Toasts`, `Stage`, the bounds) | `spaceport-live/src/notifications.rs` | 618 → 378 + policy |
| `crates/spaceport-palette/src/navigation.rs` (interim home after #229) | `spaceport-live/src/navigation.rs` | 40 |

`spaceport-live` is 2,051 lines: 1,974 production and the 77 moved notice tests.
It has **zero `gpui` and zero app imports** — `cargo tree -p spaceport-live`
contains no `gpui`, and its only `crate::` paths are its own modules. Its
dependencies are `spaceport-client`, `spaceport-sound`, `spaceport-osc52`,
`serde_json` and `tracing`.

## The words: `Labels`

A crate below the app has no messages (`docs/i18n.md`), so the seven `fl!` sites
that would have crossed became one struct, as `spaceport-chat-core`'s `Labels`
did:

```rust
pub struct Labels {
    pub activation_invalid: String,
    pub describe: fn(&ClientShellSnapshot, &str) -> Option<String>,
    pub host_key_unverified: String,
    pub localize: fn(&spaceport_client::Error) -> String,
    pub notification_untitled: String,
}
```

The app reads them in the language it shows once per applied event
(`crate::state::labels()`), and once per notification pass. `apply` keeps its
shape apart from the second argument: `apply(&mut self, event, &Labels)`.

Two surveys the design asked for, and their answers:

- **Every `LiveState` construction is `Default` plus overrides.** All
  construction sites outside `state.rs` (`endpoint/tests/live_of.rs`,
  `sidebar/layout_tests.rs`, `presentation/tests.rs`, `group_terminals/tests.rs`,
  `smoke/support/tests.rs`, `connection/reader.rs`) build `LiveState::default()`
  or `LiveState { .., ..LiveState::default() }`, so the crate's `Default` covers
  them unchanged.
- **`Answer`/`DialogResponse` name no app error.** Both sources were already a
  `spaceport_client::Error` inside the app's root error — `CommandRejected`'s
  `reason`, and `Response`'s `Endpoint` refusal — so the slot now holds
  `Arc<spaceport_client::Error>` exactly, with no new error type and no new
  i18n mapping. `chat::Error::Request` and `overview::Error::ReopenRequest`
  changed to that type; `chat::Error::Start` keeps the app's own error for a
  core-tab start that fails in this window.

## Where the remaining words and effects live

- **The status bar.** `ConnectionStatus` keeps `is_connected` and `Display` (the
  log's English, `// i18n: data`); the six status words and the "status: error"
  line are `crate::state::status_label` and `crate::state::status_text` in the
  app.
- **`Notice.agent` is an identity, not a mark.** `spaceport-icons` depends on
  gpui, so the notice keeps the lowercased identity string (≤ 32 bytes) and
  `window/toasts/badge.rs` resolves it with `AgentIcon::from_identity`, filtering
  `Generic` there — the same behaviour as the old construction-time filter.
- **`context::describe` stays in the app**, reached through `Labels.describe`:
  it names the agent's place as the sidebar does, which is the app's words.
- **`tick`, `take_system`, `SystemPost`** (the evidence, delay, mute, focus and
  OS-delivery policy over `&mut [Endpoint]`) stay in `crate::notifications`; the
  crate holds the queue, the stage machine and the boot-fenced targets they act
  on.

## The subtle constraint, preserved

`SpaceportWindow.live` is still the window's own copy of the **selected**
endpoint's live state, refreshed by `reset_selected`, while every other host
reads its endpoint's copy through `live_of`. Stage 3 moves the value, not the
storage or the rule: no field was merged, `live_of`/`reset_selected`/polling are
untouched, and `endpoint/tests/live_of.rs` (whose fixture deliberately makes the
two copies differ) passes unchanged.

## `local_peer_warning` moved to `spaceport-client`

`LiveState` held `Option<crate::daemon::LocalPeerWarning>`, which a crate cannot
name. Under the supervisor's ruling the type and its `UntrustedEndpoint` moved
to `spaceport-client` (`crates/spaceport-client/src/local_peer.rs`), beside
`SshFailure`, `VersionMismatch` and `SendError` — the family argument: each is
produced by the dialer, stored on `LiveState`, and shown once. The words stay in
`crate::daemon::warning` (`notice`, `body`) as free functions over the type's
accessors. Stage 4's reader should not re-litigate this home.

## Visibility and gates

- The app drives this struct, so its fields and methods are `pub` in the crate.
  `pub(crate)` remains only for what stays inside it (`coherent`,
  `AgentRecency::observe`, `AnnouncementDismissal::covers`,
  `Notice::resolve_target`, `Notices::transition`).
- `spaceport-live` gains a `test-support` feature (the repo's convention, as
  `spaceport-sound` and `spaceport-client` do) for `Fence::for_test`,
  `Notices::is_some`, `Notice::set_expires` (`+ unix`), `Transition::Reset`. The
  app enables it from its own `integration-test` feature and from its
  dev-dependencies, so both its unit tests and its integration-test build see
  exactly those items.
- Every re-export in the app's façades is used outside tests, except the three
  the unit tests alone need (`AgentRecency`, `SurfaceActivation`, the two
  sanitizer bounds), which are gated `#[cfg(test)]` to match their consumers.

## Where the app's paths stay

`crate::state`, `crate::notice` and `crate::navigation` are re-exports from
`spaceport-live` plus the app's words, so the rest of the window keeps naming
them as before; `crate::notifications` keeps its scheduler and re-exports the
model. `spaceport-palette` depends on `spaceport-live` for the navigation target
a Go To row carries and re-exports it.

## What did not move

The 1,791 lines of `src/state/tests/*` stay in the app, re-pointed through the
façade: they drive app fixtures and assert the app's words, and the design
allows either move or re-pointing. `Reconcile`/`Release`/`polling.rs`, the
socket half, and the lifecycle suite are untouched, as the design requires.

## Verification

Run on this branch with `CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0`:

- `cargo check` + `cargo clippy --all-targets` for `spaceport-live`
  (`--features test-support`), `spaceport-client`, `spaceport-palette`, and
  `spaceport-gpui --all-targets --features integration-test`
  (`SPARKLE_FRAMEWORK_PATH=target/sparkle-sdk`).
- `cargo test -p spaceport-live --features test-support` (5 notice tests).
- `cargo test -p spaceport-client` (401 + 2 + 1).
- `cargo test -p spaceport-gpui --bin spaceport` (3,407 passed, 10 ignored).
- `cargo fmt --all -- --check`, `just check-file-size`, `just check-sorted`,
  `just check-i18n` (2,114 messages).

Native UI checks were not run: no paint or layout changed, but the status bar,
toasts and the badge draw through the values above.
