# wx66 — endpoint stage 5: the `Host` view the sidebar asks for

Stage 5 of `docs/notes-endpoint-extraction.md` (§"Stage 5 — the `Host` view the
sidebar asks for"), built on `9b1556ac31` (`refactor(endpoint): Move the
connection machine into its crate`, #233), which supplies
`spaceport_endpoint::Host`, `Phase` and the link machine, and on
`2a98f9b7cb` (#232), which supplies `spaceport-live`'s `ConnectionStatus`.

One change: `spaceport_endpoint::HostView` is the host a sidebar row is handed
once everything a socket owns is left behind, prepared in one app-side function,
and the four row and rail renderers take it instead of `&crate::endpoint::Endpoint`.

## What the survey found, before any code

The design gives `HostView` six fields. `git grep` over the four named renderers
enumerated exactly what they read from the endpoint, and five of the six fit as
designed:

| fact | where it came from | how the view carries it |
| --- | --- | --- |
| the id rows key on | `endpoint.id` | `id` |
| the label rows word | `endpoint.label` | `label` |
| whether it wants a connection | `endpoint.enabled()` | `phase` (see below) |
| whether it is up | `endpoint.live().status.is_connected()` | `status` |
| its own fold | `endpoint.collapsed` (`host_folded`) | `collapsed` |
| its repositories' folds | `endpoint.collapsed_repos` | `collapsed_repos` |

**The sixth-and-a-half finding: the connection target.** Two reads had no place
in the designed view, both in `render/rows/workspaces.rs`:

```rust
let daemon = crate::listening_ports::Daemon::of(&endpoint.connection.target);
let silenced = crate::notification_mutes::Mutes::silenced_in(cx, endpoint, snapshot);
```

Both build a *daemon identity key* — `notification_mutes::Daemon` and
`listening_ports::Daemon` each take the endpoint id and the session or socket
the connection target names. The task's instruction was to propose a field list
through the supervisor rather than widen the view quietly, so this was put as a
choice: a seventh `target: &'a ConnectTarget` field, or six fields with the two
keys computed app-side. **The supervisor chose six fields**, because a view
carrying a `spaceport-client` connection type would hand the dependency to the
sidebar program's render lane (§2.5) that this stage exists to make possible.
The plumbing shortcut lost to the dependency boundary; that is the recorded
reason, not an accident of the survey.

`Phase` already answers whether a host wants a connection (`Phase::Disabled` is
exactly `Link::Disabled`), so `Host::enabled()` now delegates to a new
`Phase::enabled()`. One spelling of the rule, and a row that holds only a phase
can ask it.

## What moved

| was | now | lines |
| --- | --- | --- |
| (nothing — the seam did not exist) | `spaceport-endpoint/src/host/view.rs`: `HostView`, with doc | 29 |
| `Host::enabled`'s private `matches!` | `host/phase.rs`: `Phase::enabled`, `Host::enabled` delegating | 9 |
| the endpoint facts each renderer read | `Endpoint::view` (`src/endpoint.rs`) — the one app-side preparation | 14 |
| `rail/render.rs`'s three renderers, `render/rows.rs`, `render/rows/workspaces.rs`, `render/agent_rows.rs` | the same renderers taking `&HostView<'_>` | ~75 rewritten |

Production diff: **~130 lines touched across the seam, 29 of them a new crate
module**. The design's "~80" counted the view and the renderer signature changes;
the survey's daemon keys and the `Phase::enabled` delegation are the additional
40.

## What the renderers take now

- `sidebar/rail/render.rs`: `rail_host_box`, `rail_workspace_row` and
  `rail_agent_row` take `host: &spaceport_endpoint::HostView<'_>`. The rail's
  `silenced` map is still built in `rail_measure`, which holds the endpoint and
  calls the app's own `Daemon::of` — a measure step, not a row painter.
- `sidebar/render/rows.rs`: prepares `let view = endpoint.view()` per listed host
  and hands it to the row builders. The pinned and expanded **host header**
  (`sidebar/host_row.rs`) keeps `&Endpoint`: it toggles `collapsed`,
  `collapsed_repos`, opens menus and reads the origin glyph, so it acts on the
  endpoint rather than painting from it. That is a different job, and it stays
  the app's.
- `sidebar/render/rows/workspaces.rs`: `HostRows` carries `view: HostView<'a>`
  plus the two daemon keys (`mutes: notification_mutes::Daemon`,
  `ports: Option<listening_ports::Daemon>`), computed in `rows.rs` from the
  endpoint's target. The workspace rows therefore name no connection at all.
- `sidebar/render/agent_rows.rs`: `agent_host` takes `&HostView` and answers the
  label a remote's rows name.

`Mutes::silenced_in` narrowed from `&Endpoint` to `&Daemon` for the same reason:
its two callers each already know the daemon, and the row no longer has to hold
an endpoint to get one. `window/tab_strip/strip.rs` now passes
`&Daemon::of(endpoint)`.

## What stays

- `Endpoint` itself: the bridge, the target, the toasts, the history, the
  palette's last-seen frames, the recovery trigger and every mutation. `view()`
  copies six values out; nothing borrows through the connection.
- Every word. `HostView` holds no `fl!`; the rail still words
  `sidebar-rail-collapsed` and every other label it always did.
- `spaceport-sidebar` is untouched — no dependency on `spaceport-endpoint` or
  `spaceport-client`, no line of `crates/spaceport-sidebar/src` names
  `Endpoint`. The four renderers have always lived in the app
  (`crates/spaceport-gpui/src/sidebar/`), and that is where the boundary was
  drawn.

## Layout tests

Every host the `sidebar/layout_tests/*` fixtures add beside the fixture's own now
comes from one builder in `sidebar/layout_tests.rs`:

```rust
layout_host("ssh:test", "Remote", ssh("unused"))
```

`layout_host(id, label, target)` is the one place these tests spell
`Endpoint::new`; `ssh(host)` is the two-line SSH target they used to write out in
full at fourteen sites. Both are `#[cfg(test)]`, matching their consumers: the
submodules are all `#[cfg(test)]`, and `crates/spaceport-gpui/src/sidebar/layout_tests.rs`
is also built under `feature = "integration-test"` alone, where those submodules
do not exist. `fixture_window`'s own local device keeps its `Endpoint::new`: it
is the fixture, not a host a test adds.

## Validation

Run on `macos`, `CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0`,
`SPARKLE_FRAMEWORK_PATH=$PWD/target/sparkle-sdk`:

```sh
cargo check -p spaceport-endpoint
cargo clippy -p spaceport-gpui -p spaceport-endpoint -p spaceport-sidebar \
    --all-targets --features integration-test
cargo test -p spaceport-endpoint              # 54 passed
cargo test -p spaceport-sidebar               # 79 passed
cargo test -p spaceport-gpui --bin spaceport sidebar    # 315 passed
cargo test -p spaceport-gpui --bin spaceport endpoint   # 248 passed, 1 ignored
cargo test -p spaceport-gpui --bin spaceport mutes      # 53 passed
```

`mutes` was run as well because the mutes key is what the survey moved:
`sidebar::layout_tests::mutes` and `window::tab_strip::tests::mutes` are the two
suites that would notice a row reading the wrong daemon.

## What is left of stage 5

The rail's own measure (`rail_measure`) still reads the endpoint for the muted
map, and `host_row.rs` still mutates it. Neither is a row painter's input, so
neither is part of the boundary this stage draws; both are the app's until the
sidebar program's render lane moves, which is what `HostView` was built for.
