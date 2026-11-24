# wx64 — endpoint stage 4: `Endpoint` becomes `Host`, the link machine lowers

Stage 4 of `docs/notes-endpoint-extraction.md` (§"Stage 4 — `Endpoint` becomes
`Host`, the link machine lowers"), built on `2a98f9b7cb` (`refactor(live): Move a
connection's live state below the app`, #232), which supplies
`spaceport-live`'s `LiveState`, `ConnectionStatus` and activation fence for it.

## What the survey found, before any code

Three findings changed the plan the design wrote, and each is recorded here
because the reviewer will ask:

1. **The design's premise about `endpoint.live` was wrong.** §"Survey first" (2)
   and §3.1 rule 3 both assume no file outside `src/endpoint/` reads
   `Endpoint.live`. Measured on `2a98f9b7cb` with

   ```sh
   git grep -n -e 'endpoint\.live' -e 'endpoints\[[^]]*\]\.live' \
       HEAD -- crates/spaceport-gpui/src
   ```

   it is **31 production sites in 22 files** (`sidebar/host_row.rs`,
   `sidebar/rail/render.rs`, `menu/devices{,/picker}.rs`, `menu/sessions{,/management}.rs`,
   `window/{status_bar,toasts,system_notifications,construct}.rs`, `overview.rs`,
   `notifications.rs`, `integrations.rs`, `devices_overview.rs`, `app_badge.rs`,
   `settings_window/source.rs`, `group_terminals.rs`, `palette.rs`,
   `chat/preview.rs`, `control/server/work.rs`, `window.rs`, `smoke_hosts.rs`)
   plus **103 in 53 test files**, and more inside `src/endpoint/` itself.
2. **The design contradicts itself about the storage.** §3.1 rule 1 says "the
   two fields stay two fields … `Endpoint.live` survives the program unchanged"
   and rule 2 says "only the rule lowers, not the storage", while rule 3 and the
   exit criteria say `Host` holds the `LiveState` and `live` never becomes a
   public field. Both cannot hold at once. The supervisor chose "Host owns
   `live`", with `Host::live()`/`live_mut()` and thin `Endpoint::live()`/
   `live_mut()` accessors, so every one of those sites is now `.live()` (read)
   or `.live_mut()` (write) and the storage moved with the machine that owns it.
   Nothing changed meaning: `SpaceportWindow.live` is still the window's stamped
   copy, `live_of` still answers with it for the selected host only, and
   `endpoint/tests/live_of.rs` still makes the two copies disagree
   (`booted("stale")` in the endpoint, `booted("window")` in the window) and
   passes through the new accessors.
3. **`recover` is where `Action::CheckLiveness` comes from.** It reads three
   socket facts (`handle.is_some()`, `reports_connected()`, the target's
   restartability), so if it stayed app-side nothing would ever produce that
   action. The rule lowered; the socket work did not.

The three direct mutations of `Endpoint.connection` the design named are now:
`endpoint/fence/changes.rs` still replaces `connection.inbox` by hand (a
documented test hook, unchanged); `chat/tests/start.rs` still writes
`connection.target` and then the endpoint's live state through
`live_mut()`; `catalog.rs`'s `Retarget` is `Endpoint::retarget(target, enabled)`,
which enters the transition and performs the `Retire` it comes back with.

## What moved into `crates/spaceport-endpoint`

New module `src/host.rs` with `host/{link,phase,fence,live_of,fixtures}.rs`,
gpui-free and socket-free:

- `Host { link, live, attempts }` — the link, the live state it is read from and
  the retry counter, with `phase()`, `serving(Transport)`, `may_dial(bool)`,
  `outage()`, `retry_at()`, `enabled()`, `detached()`, `waits_for_user()`,
  `herdr_missing()`, `herdr_stopped()`, `version_mismatch()`, `retry_delay()`,
  and `live()`/`live_mut()`.
- `Link`, `Failure`, `Transition` (with the `cfg(test-support)` `Fixture`), and
  `Host::enter(Transition) -> Action`, whose body is `Endpoint::enter`'s, moved
  line for line from `endpoint/link.rs`.
- `Action { None, Detach, Retire { target }, CheckLiveness }`.
- `Transport { handled, alive, can_restart }`: what a poll can see of a host's
  socket and the link rules need, so `Observe`, `Update` and `Trade` carry one
  value instead of loose booleans.
- `Host::recover(trigger, now, target, transport, reports_connected)` →
  `Option<Action>`: `None` asks to be called again next poll, and
  `reports_connected` is a closure so the inbox is read only when the rules need
  it, exactly as before.
- `Host::retired()`: the one line of the old `Stop` arm about the *daemon*
  status ("a retired attempt still waiting for a retry is not a user detach"),
  called by the window after it has taken the state its bridge left.
- `Phase` with `as_str()` and `status()`; the English `status()` table is
  command output (`// i18n: data`), and the window still words
  `Phase::status_label()` itself.
- `InputFence`, `SelectionFence`, `ConnectionRef` (`new`, `downgrade`,
  `names`, `for_test` under test-support), `WeakConnectionRef`; the window keeps
  `input_fence`, `fence_holds`, `selection_fence`, `selection_fence_holds` and
  `endpoint_for`, which are reads over its own endpoint list.
- `live_of(selected, selected_live, index, host)` — the rule, called by the
  window's `live_of`; the design's `&[Host]` signature cannot be called with
  `Vec<Endpoint>`, so the caller passes `|index| &self.endpoints[index].host`.
- `MAX_RETRY_DELAY` and `STABLE_CONNECTION_PERIOD`, which the app's
  socket-driving tests time against and which no longer have a second
  definition app-side.
- `src/host/fixtures.rs` (`cfg(any(test, feature = "test-support"))`): the nine
  link fixtures the app's 34 test files call, now on `Host` with the app keeping
  the same wrapper names on `Endpoint`, so no test body changed.

`spaceport-endpoint` gains `spaceport-live` and a `test-support` feature;
`spaceport-gpui`'s dev-dependencies enable it, as they already do for
`spaceport-live`.

## What stayed in the app

`crates/spaceport-gpui/src/endpoint.rs` is now the bridge and the performers:
`Endpoint { connection: ConnectionBridge, host: Host, sequential app facts }`,
`enter` (enter, then perform), `perform` (`Detach` → `retire_transport`,
`Retire` → retire then a fresh bridge for the target, `CheckLiveness` → the
handle's own probe), `stop_side`, `take_live`, `connect`, `recover`, `sync_live`
and its projection, `trade_connection`, `poll`, `Drop`, `reset_selected`,
`switch_endpoint`, `release_selected`, plus the delegating reads
(`phase`, `status`, `status_label`, `serving`, `may_dial`, `enabled`, `detached`,
`retry_at`, `outage`, `herdr_missing`, `herdr_stopped`, `version_mismatch`,
`retry_delay`) that keep every app call site spelling them on its endpoint.
`polling.rs`, `catalog.rs`, `follow.rs`, `wake.rs`, `order.rs`, `origin.rs` and
`config_reload.rs` stay; `endpoint/link.rs`, `endpoint/phase.rs` and
`endpoint/link/fixtures.rs` are gone (`fixtures.rs` replaced the last).

Socket facts that stay app-side by decision, because they are the window's and
not the link's: `connection`, `generation`, `initial_surface`,
`focus_requested`, `recovery`, `saved`, `config_reload_pending`, `left_at`,
`toasts`, `last_seen`, `history`, `config_diagnostic`, `sounds`, `collapsed*`,
`id`, `label`. `generation` is still bumped once per retirement by
`stop_side`, which is what `Transition::Stop`/`Connect` did before.

## Deviations from the approved direction, for review

Each was forced by the code and is small; none changes behaviour.

1. `Action::Dial { active }` is dropped. Only `Endpoint::connect` can dial and it
   holds the window's `ConnectOptions`, so a dial needs a parameter no other
   action does; `Transition::Connect` therefore records the attempt
   (`attempts += 1`, link dialing) and returns `Action::None`, and `connect`
   performs the dial. Every action that remains is one the *crate decides* from
   its own state, which is the property the enum was for.
2. `Action::Retire { target }` carries the `ConnectTarget` instead of the app
   matching an impossible arm.
3. `Host::recover` is a method, not a `Transition` variant: it returns
   `Option<Action>`, which `enter` (which returns `Action`) cannot express.
4. `Observe`/`Update`/`Trade` take a `Transport` rather than `can_restart: bool`
   (plus `handled`, which the survey found they also need) — one name for the
   socket facts at seven call sites.
5. `Host::retired()` exists because the daemon-status line of the old `Stop` arm
   has to run *after* the window takes the bridge's fresh state, not inside the
   transition.

## Validation run

```sh
cargo fmt --all -- --check                                   # clean
cargo clippy -p spaceport-endpoint --all-targets --all-features -- -D warnings
cargo clippy -p spaceport-endpoint --all-targets --no-default-features -- -D warnings
cargo clippy -p spaceport-live -p spaceport-client -p spaceport-gpui \
    --all-targets -- -D warnings
cargo clippy -p spaceport-gpui --all-targets --features integration-test -- -D warnings
cargo check -p spaceport-gpui --all-targets --features integration-test
cargo test -p spaceport-endpoint --all-features              # 53 passed (18 new)
cargo test -p spaceport-live                                 # 5 passed
cargo test -p spaceport-gpui --bin spaceport endpoint        # 248 passed, 1 ignored
cargo test -p spaceport-gpui --bin spaceport                 # 3405 passed, 10 ignored
just check-file-size / check-sorted / check-i18n              # clean
```

## Follow-ups this stage does not take

- §3.1 rule 3 ("no `endpoint.live` outside `src/endpoint/`") is still violated
  in spirit: those 31 production sites now read `endpoint.live()` instead of
  `live_of(index)`. Fixing them is a semantic change, not a mechanical one — the
  two copies disagree between a poll and a reset — and it is what stage 5's
  `HostView` is for.
- `Endpoint::generation` is app-side by decision; a later stage that gives the
  fences a home in the crate may take it too.
- `ConnectionBridge` (760 lines) is untouched, as the design's "honest
  remainder" expects.
