# A11: the endpoint drain split — still deferred

Crew task A11 (`docs/notes-backlog-2026-11-26.md:23`): "Endpoint residual: drain
split deferred at wx75 — revisit if the mailbox routing changes." This note is
the revisit. **Verdict: the routing did not change, so the row closes as still
deferred.** No production code changed.

Base revision `6a678a90f5` (this tree's HEAD). `origin/main` (`925370ab1b`) was
read for the phone's history inbox, because it is ahead of HEAD on that file.

## 1. What the record actually says, and what it does not

- **There is no `docs/notes-wx75.md`, and no commit mentions wx75.** `git log
  --all --oneline --grep=wx75` is empty, `git log --all -- docs/notes-wx75.md`
  is empty, and the only file in the tree that contains the string `wx75` is the
  backlog row itself. So the row's "at wx75" is a board reference with no
  written note behind it.
- **The one written record of a deferred drain split is the phone's.**
  `docs/notes-wx71.md:86-96`: `drain` (`mobile/rust/src/controller.rs:645`) was
  surveyed and deliberately not split. It interleaves ordered `ClientEvent`
  dispatch, the by-request-id history-reply mailbox routing that is the phone's
  alone, and the dirty-flag `version` bump; moving the dispatch onto
  `spaceport-live::apply` would need the mailbox to stay behind a callback while
  `apply` owns the version.
- **The desktop endpoint has never had a drain split pending.** The design
  rules it out: `docs/notes-endpoint-extraction.md` §5 seam 11 calls
  `polling.rs`'s tick "never moves. It *is* the window", and
  `docs/gpui-split-plan.md:381-385` names the two open endpoint follow-ups — 31
  production sites reading `endpoint.live()` instead of `live_of(index)`, and
  `Endpoint::generation` staying app-side. Neither is a drain split, so nothing
  in the endpoint program is waiting on this ticket.

## 2. Did the mailbox routing change? No

**The phone's routing is unchanged in this tree.** `drain` still matches a reply
by request id in the same two arms and falls through to the projection's own
receipt/rejection otherwise: `controller.rs:670` (`ClientEvent::Response`) and
`:683` (`ClientEvent::CommandRejected`) both call `state.history.take_reply(..)`.
The mailbox they read is untouched too: `mobile/rust/src/history.rs:23-32`
still holds `HistoryInbox { generation, replies: HashMap<String, Option<Reply>> }`
with `take_reply` at `:30`. Every change to `controller.rs` since #240 is six
hunks (`git diff 36cb09060c..HEAD`), all in the navigation code before line 560
— `FocusAction`, `create_tab` — and none in `drain` (`:645-744`).

**`origin/main` changed the inbox's staleness, not its routing.**
`git diff HEAD origin/main -- mobile/rust/src/history.rs` adds a `#[cfg(test)]
waiting` signal, moves `settled_revision` onto `HistorySearch`, and refuses a
search whose `generation` moved — all inside the search side. The drain's diff
in the same comparison is one hunk at `controller.rs:267` (`frames.notify_all()`
on disconnect). The routing decision the deferral names — a response whose id
names a pending history reply goes to that reply, everything else to
`state.receipt` — is identical on both trees.

**Both of the split's other blockers also still stand.**

| blocker | today |
| --- | --- |
| `apply` is not reachable from the phone | `crates/spaceport-live/Cargo.toml` still depends on `spaceport-sound`, which brings `rodio`; `mobile/rust/Cargo.toml` depends on `spaceport-activation` and not `spaceport-live`, exactly as `docs/notes-wx71.md:13-16` explains. `LiveState::apply` is `crates/spaceport-live/src/state.rs:482` |
| the same state feeds more than the transcript | the drain's `received` block still calls `routes.invalidate(&state)` (`controller.rs:733`) before bumping the dirty flag (`:735`), beside the `initial_focus` claim (`:710`) |

So a split today would be the same shape as the one wx71's note declined: the
ordered dispatch would leave, and the mailbox callback, the dirty flag and the
`PtyRoutes`/`initial_focus` feeds would stay behind it.

## 3. The desktop half is not pending — it is already split

The app drains a connection in two steps, and both are where the design put them:

- **The reader thread does the ordered dispatch.** `crates/spaceport-gpui/src/connection/reader.rs:55-96` takes one `ClientEvent`, filters it through three app mailboxes — `integrations` (`:85`), `scrollback::Inbox` (`:89`) and `links` (`:93`) — and hands whatever is left to
  `state.apply(event, crate::state::describe, crate::state::localize)` (`:69`, and again on the drop path at `:113`). The desktop's history replies are that `scrollback` sink, not an interception inside the drain.
- **The window takes the coalesced result.** `Endpoint::poll`
  (`crates/spaceport-gpui/src/endpoint.rs:450`) takes the latest state from
  `connection.take_update()`, and `drain_endpoint`
  (`crates/spaceport-gpui/src/endpoint/polling.rs:167`) takes the per-endpoint
  mailboxes it owns: `clipboard_writes` (`:185-187`) and `bells` (`:190`).

`ConnectionBridge` and `reader.rs` stay app-side by decision until `daemon`,
`links`, `scrollback`, `terminal::InputTarget` and `config::Theme` have homes
(`docs/gpui-split-plan.md:371-379`). A11 does not change that.

## 4. What would reopen A11

One of these, and neither has happened:

1. **the phone's history replies stop being a by-request-id interception inside
   the ordered drain** — for instance by moving to a sink of their own, the way
   the desktop's `scrollback::Inbox` is one, which is what frees `drain` to hand
   every event straight to a reduction; or
2. **the reduction becomes reachable from the phone** — `LiveState::apply` below
   a crate that does not pull `spaceport-sound`'s `rodio`, at which point the
   dispatch, the mailbox callback and the dirty flag can be separated as
   `docs/notes-wx71.md:88-96` describes.

Until one does, forcing the split would leave the same three things interleaved,
so the row closes as deferred rather than being re-scoped.

## 5. Commands run

```sh
git log --all --oneline --grep=wx75                       # empty
git log --all -- docs/notes-wx75.md                       # empty
grep -rn "wx75" --include=* .                             # the backlog row only
git log --oneline -40 -- crates/spaceport-gpui/src/endpoint \
    crates/spaceport-gpui/src/endpoint.rs crates/spaceport-live \
    crates/spaceport-endpoint                             # last functional change: #233, then #235
git log --oneline -15 -- crates/spaceport-gpui/src/endpoint/polling.rs  # #233, #217, the rename
git log --oneline -40 -- mobile/rust/src/controller.rs    # newest is 8e9d1cbbc5, then #240
git diff 36cb09060c..HEAD -- mobile/rust/src/controller.rs             # six hunks, all ≥ line 560
git diff HEAD origin/main -- mobile/rust/src/history.rs                # staleness + test plumbing
git diff HEAD origin/main -- mobile/rust/src/controller.rs             # one hunk at :267
grep -rn "take_reply" mobile/rust/src crates                         # the phone only
```

`cargo fmt --all -- --check` and `git diff --check` ran clean on the change;
nothing built or tested differently, because no code changed.
