# wx71: the activation fence as one shared library

Branch `feat/wx71-activation-crate`, cut from `f778a8626c`. The fence that keeps
keystrokes off an unacknowledged projection existed twice — once in
`crates/spaceport-live/src/activation.rs` and once in the phone's
`mobile/rust/src/projection.rs` with its `projection/go_to.rs` and
`projection/resume.rs` — and the two copies had already drifted on outcomes.
This branch puts one of them below both clients and turns the phone into a
caller.

## 1. What moved, and to where

New leaf crate `crates/spaceport-activation` (972 lines including tests). Its
dependencies are `serde_json` and `spaceport-client`, nothing else: the phone
already compiles both, and `spaceport-live` was unusable as a home because it
depends on `spaceport-sound`, which brings `rodio` and an audio stack onto iOS.

| Item | From | Rule it holds |
| --- | --- | --- |
| `Fence`, `Failure`, `Transition`, `TIMEOUT` | `spaceport-live/src/activation.rs` (317 lines, moved whole) | one activation's lifecycle, its four failure causes plus the timeout, and its 5 s barrier budget |
| `NavigationTarget`, `OwnedNavigationTarget`, `as_deref`, the owning `From` | `spaceport-live/src/navigation.rs` (40 lines) | where a focus request is aimed |
| `NavigationTarget::is_focused_in`, `reachable_in` | new here, from the phone's `Navigation::{matches, present}` and the fence's inline focus match | has this destination been focused, and does it sit in the focused workspace and tab |
| `SurfaceActivation` | `spaceport-live/src/state.rs:184` | the activation one barrier was asked for |
| `Route`, `Step`, `ROUTE_BUDGET` | new here, from `mobile/rust/src/projection/go_to.rs` | a pane reached one level per request, with no step ever replayed |

`spaceport-live` re-exports rather than duplicates, so every existing path still
resolves: `spaceport_live::activation::{Fence, Failure, Transition, TIMEOUT}`,
`spaceport_live::SurfaceActivation`, `spaceport_live::NavigationTarget`, and the
app's `crate::navigation` and `crate::state::activation` façades are the same
names for the same types. `spaceport-live`'s `test-support` feature now forwards
to the crate's, so `Fence::for_test` and `Transition::Reset` still reach the
app's headless tests. Nothing in `spaceport-gpui` changed.

One widening came with the move. `Transition::Settled` used to clear the
fence's destination only on `Acknowledged`; it now does so on `Sent` and
`Retry` too. The desktop never calls it in either — the only call site is
behind `surface_ready()`, which needs `Fence::ready`, which answers only for
`Acknowledged` and `Ready` — so desktop behaviour is unchanged. The phone needs
it because its focus request carries its own id and the shared fence records
only the barrier: a refused focus must drop the destination whether or not the
barrier had entered the queue yet, and that is exactly what `Settled` says.

## 2. The three questions, answered

**(1) The budgets differ because they measure different things.** `TIMEOUT` is
5 s and bounds one barrier, from submission to acknowledgement. The phone's
`resume` barrier used 4 s for the same quantity and now uses `TIMEOUT`; the
walk's 16 s bounds the whole workspace → tab → pane sequence, up to three
barriers with a settled projection between each, and is `ROUTE_BUDGET`. Two
named constants, no per-step parameter.

**(2) The phone now matches the desktop on a refused barrier.** Both drifts were
real, and both changed phone behaviour on purpose:

| | Before | After |
| --- | --- | --- |
| `set_surface_active` answered `Error::Full` | `state.end(...)` and `handle.disconnect()` — a healthy session dropped | `Fence::Retry` keeps the destination, and the drain offers the barrier again on its next tick |
| `set_surface_active` refused outright | same, `end` and `disconnect` | reported in `state.error`, left fenced, and the session ends through the fence's `Failure` in its own words |
| resume barrier budget | 4 s | 5 s (`TIMEOUT`), so `ResumeTimeout` arrives a second later |

The `handle.disconnect()` call is gone from that path entirely: a transport that
is really gone reports `ClientEvent::Disconnected`, which the drain already
ends the session on.

**(3) The crate holds the walk; only the phone calls it.** Desktop did not adopt
it, and the reason is in the code rather than in taste: the phone's own
pre-check `reachable_in` refuses a single hop to a pane that is not already in
the focused tab and workspace, so a cross-workspace jump *must* go one level per
request. The desktop does not gate on `reachable_in` — `handle.focus_pane`
reaches across workspaces in one request and `Fence::ready` confirms the
destination with `is_focused_in` — so the walk would triple its round trips for
no additional guarantee. Adopting it would mean a `Route` held by the window, a
per-event `route.step(snapshot)` in the polling path, and `dispatch_navigation`
becoming a loop that re-enters after each settled fence: real window work, not a
mechanical move, and worth nothing while desktop does not gate on
`reachable_in`. What matters is that the walk is now a rule instead of a copy, so
a desktop that ever does start gating adopts it rather than writing a third one.

## 3. What the phone kept

| Owner | Why it is not the crate's |
| --- | --- |
| `resume::Visibility` (parked/checking) | a retained session re-proving its target against a *snapshot* revision; the fence is proved against a *surface*. Two different proofs, and the crate has one of them |
| `Walk.request_version`, `go_to_arrival` | projection bookkeeping: which version Swift is told the jump belongs to |
| `MobileError` | the UniFFI boundary. Its variants stay where Swift can see them; `fence_failure_reason` words the shared `Failure` for that boundary and nothing else |
| `drain` | see below |

`drain` (`controller.rs:571-678`) was surveyed and deliberately not split. It
interleaves three things: ordered `ClientEvent` dispatch, the history-reply
mailbox routing that is the phone's alone, and the `version` bump that stands in
for a dirty flag. Moving the dispatch onto `spaceport-live::apply` would need
that mailbox to stay behind a callback while `apply` owns the version, and the
phone's `apply` also feeds `PtyRoutes` and `initial_focus` — a split that is not
clean, so it stays as a follow-up rather than being forced here. The two
behaviours this branch did have to reach are now calls into the crate
(`retry_barrier`, `barrier_queued`, `fence_failure`), not re-coded rules.

## 4. Merging with wx70

wx70 moves `NavigationTarget`, `OwnedNavigationTarget` and the predicates into
`spaceport-client/src/navigation.rs` and makes `spaceport-live::navigation` a
re-export. This branch is cut from `f778a8626c`, before that, so it keeps the
copy — the phone cannot reach `spaceport-live` but already depends on
`spaceport-client`, so either order works. If wx70 lands first, resolve by
deleting `crates/spaceport-activation/src/navigation.rs` and re-pointing
`crate::navigation` at `spaceport_client::navigation`; the fence, the walk and
the phone already use the names and need no other edit. If this branch lands
first, wx70's rebaser does the mirror. Either way, one copy survives.

wx72's `spaceport-backend` wheel rule is untouched and no file it changed
overlaps this branch.

## 5. Tests

| Added | Where | What it pins |
| --- | --- | --- |
| `an_open_fence_holds_nothing_back`, `a_barrier_the_queue_refused_is_offered_again_rather_than_failing`, `a_settled_destination_is_dropped_from_a_barrier_the_queue_refused`, `a_barrier_refused_outright_fails_without_a_retry`, `a_submitted_barrier_expires_and_is_never_offered_again`, `a_settled_fence_drops_a_destination_the_barrier_is_still_waiting_on`, `an_acknowledgement_unlocks_only_the_projection_it_named`, `a_boot_that_changed_under_the_barrier_fails_it`, `a_receipt_for_another_request_cannot_acknowledge_this_barrier` | `crates/spaceport-activation/src/tests/fence.rs` | the lifecycle, the retry-vs-fail split, and what an acknowledgement is good for |
| `a_walk_takes_the_shallowest_level_the_snapshot_has_not_focused`, `a_walk_that_is_overtaken_stops_instead_of_replaying_its_step`, `a_walk_whose_destination_vanished_is_no_longer_present`, `a_walk_expecting_an_agent_is_stale_once_that_agent_leaves_the_pane`, `a_walk_stops_at_the_budget_it_was_given` | `crates/spaceport-activation/src/tests/route.rs` | the walk order, the anti-replay guard, and the route's own budget |
| `a_barrier_the_queue_refused_keeps_its_destination_and_the_connection`, `a_barrier_offered_again_settles_once_its_revision_arrives`, `a_barrier_refused_outright_is_reported_without_dropping_the_connection` | `mobile/rust/src/tests/fence.rs` (new) | the two phone behaviour changes, from the phone's side |
| `a_resume_waits_as_long_as_any_other_barrier` | `mobile/rust/src/tests/resume.rs` | the 4 s → 5 s budget, asserted against the crate's constant |

`an_open_fence_holds_nothing_back` exists because it caught a real regression in
this branch: the move dropped `Fence::ready`'s `Open => true` arm, which made
every `LiveState::surface_ready()` false and broke 9 desktop tests. It is now
pinned at the fence rather than only through the app.

## 6. What ran

`CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0` throughout, and
`SPARKLE_FRAMEWORK_PATH=$PWD/target/sparkle-sdk` (acquired with
`python3 scripts/release/sparkle_framework.py acquire target/sparkle-sdk`) for
the app.

```sh
cargo fmt --all -- --check                                              # clean
cargo clippy -p spaceport-activation -p spaceport-live -p spaceport-client --all-targets
cargo clippy -p spaceport-gpui --all-targets --features integration-test
cargo test -p spaceport-activation                                      # 14 passed
cargo test -p spaceport-live                                            # 5 passed
cargo test -p spaceport-gpui --features integration-test --bin spaceport activation   # 21 passed
cargo test -p spaceport-gpui --features integration-test --bin spaceport              # 3439 passed, 10 ignored
cd mobile/rust && cargo fmt && cargo clippy --all-targets                # clean
cd mobile/rust && cargo test                                            # 59 passed, 2 ignored
```

Not run locally, deferred to CI: the full workspace suite, `just ci`, the
Windows and Linux lint legs, and the mobile Xcode and Gradle suites. No UniFFI
record or error signature changed, so the generated Swift and Kotlin are
unchanged and were not regenerated.
