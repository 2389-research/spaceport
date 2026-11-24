# Extracting the endpoint program from `crates/spaceport-gpui`

Design for the extraction the split plan names as the sidebar's second-biggest
blocker after menus (`docs/gpui-split-plan.md:283`: "still needs `menu` (20.6k
lines app-side), `endpoint` (16.2k), `keymap` (3.1k)"). It is a design only: no
production code changed, no build run.

**Base revision, and what this tree is.** This worktree has no shell, so nothing
here could be fetched or rebased: `git log --oneline -1` was not run. HEAD is
`1b02358` (`.git/worktrees/…/logs/HEAD`), and the tree it points at **predates
#223**: `crates/spaceport-widgets` exists (#220 landed) but
`crates/spaceport-menu` does not. §2.5 answers the menu question against
`crate::menu` as it still lives in the app. Everything else — line counts, `gpui`
counts, `fl!` sites, `impl SpaceportWindow` blocks — is measured in this tree,
and §0 gives the commands to re-run before stage 1. Nothing the menus program
(#223, #225) touched is inside the endpoint tree, so the counts should survive
the rebase; re-run them anyway.

**Verdict.** Endpoint is the opposite shape from both settings and menus, and the
opposite of what its size suggests:

| | browser | settings | menus | **endpoint** |
| --- | --- | --- | --- | --- |
| production lines | 4,390 | 11,520 | 20,146 | **3,068** |
| test lines in the same tree | 1,904 | ~8,000 | ~20,000 | **~13,330** |
| of production, `impl SpaceportWindow` | 4,390, invertible | ~9,180 | ~10,100 | **~1,210 (39%)** |
| of production, gpui-free model | most | ~1,240 | ~4,600 | **~1,859 (61%)** |
| `gpui::` / entity uses in production | — | — | — | **0 in the model, `Context` only in the glue** |
| `fl!` sites (production) | — | ~150 | ~377 | **12** |
| paints anything | yes | yes | yes | **no — zero elements, zero `impl Render`** |
| what blocks it | nothing | text | text, two entities, `endpoint`, `keymap` | **two app types with no home** |

The 16.2k the plan counts is real but it is **81% tests** (13,330 of ~16,400).
Endpoint is not a view program that needs a paint kit, like menus;
`grep -n 'impl Render\|div()\|IntoElement\|ScrollHandle'` over the tree finds
exactly one hit, a dummy `render` in a test fixture
(`endpoint/lifecycle_tests.rs:89-90`). It is the app's **connection-state
program**: a state machine over `LiveState`, and the window methods that drive
it.

That makes the honest verdict narrow and sharp. Of the 3,068 production lines:

1. **~1,210 lines never move.** They are `impl SpaceportWindow` in seven files:
   `endpoint.rs:406-778` (372), `polling.rs:30-486` (456), `catalog.rs:211-380`
   (169), `follow.rs:59-119` (60), `order.rs:41-97` (56), `fence.rs:100-155`
   (55), `wake.rs:146-187` (41). They read `self.live`, `self.endpoints`,
   `self.selected_endpoint`, `self.options`, `self.notices`, `self.menu`,
   `self.sessions`, `self.presentation`, `self.sound` and `self.bell`. This is
   the browser program's terminal shape: typed requests in, the window's own
   fields out. It is not a defect to fix.
2. **~1,859 lines are already gpui-free.** Every field of `Endpoint`
   (`endpoint.rs:153-186`) is a gpui-free type — `LiveState`, `ConnectionBridge`,
   `Toasts`, `LastSeen`, `History`, `ConfigDiagnostic`, `spaceport_sound::Policy`
   — and every `impl Endpoint` block (link, phase, history, config_reload,
   fence's `connection_ref`) touches no `Context`, no `Entity`, no `Window`.
   Measured: `grep -c gpui` is 0 in `endpoint.rs`, `link.rs`, `phase.rs`,
   `history.rs`, `wake.rs`, `fence.rs`, `order.rs`, `catalog.rs`,
   `config_reload.rs`.
3. **The gpui-free 61% still cannot move today**, because it is written against
   two app types that have no crate: `LiveState` (`state.rs:63-160`, 826 lines,
   zero `gpui`) and `ConnectionBridge` (`connection.rs:96-115`, 643 + 117 reader
   lines, zero `gpui`). Neither is blocked by GPUI; both are blocked by the
   small app models they name (`notice`, `notifications`, `links`, `scrollback`,
   `daemon`, `terminal::InputTarget`, `config::Theme`). The endpoint program is
   therefore **the trigger §2.2 of `notes-lower-shared-types-design.md` was
   waiting for** ("whichever feature first needs to read live state from outside
   `impl SpaceportWindow`").

So the program is four moves, not one, and two of them are prerequisites the
chat and sidebar programs also want:

| move | lines | new crate | unblocks |
| --- | --- | --- | --- |
| 1. the dependency-free policy (history, wake clock, device order, endpoint IDs) | ~290 | `spaceport-endpoint` | — (proves the crate) |
| 2. `LiveState`, `ConnectionStatus`, the activation fence, the coherence rules | 1,373 + models | `spaceport-live` | chat stage 4, endpoint stage 3, sidebar's `&LiveState` reads |
| 3. `Endpoint` as `Host` + the link machine, as typed actions | ~1,300 | `spaceport-endpoint` (GPUI-free) | sidebar's render lane, palette's host reads |
| 4. `ConnectionBridge`, the socket half | 760 | `spaceport-connection` | nothing yet; the honest remainder |

Each stage below states what to survey before it starts and what proves it done.

## 0. The measurements, and how to repeat them

Every count is the file's own length in this tree. Re-run before each stage:

```sh
git log --oneline -1                                                     # record the base revision
find crates/spaceport-gpui/src/endpoint* -name '*.rs' ! -path '*/tests/*' \
     ! -name '*tests.rs' ! -name 'fixtures.rs' | xargs wc -l | tail -1   # 3,068
find crates/spaceport-gpui/src/endpoint* -name '*.rs' -path '*/tests/*' \
     -o -name '*tests.rs' -o -name 'fixtures.rs' | xargs wc -l | tail -1 # ~13,330
rg -c '^impl SpaceportWindow' crates/spaceport-gpui/src/endpoint*          # 6 files + endpoint.rs
rg -o 'fl!\("' crates/spaceport-gpui/src/endpoint.rs \
     crates/spaceport-gpui/src/endpoint/*.rs | wc -l                      # 12
rg -l '\bgpui\b' crates/spaceport-gpui/src/endpoint.rs \
     crates/spaceport-gpui/src/endpoint/*.rs                              # polling.rs, catalog.rs, follow.rs only
rg -n 'crate::(direction|toggles|accessibility|fonts|widgets)|spaceport_widgets' \
     crates/spaceport-gpui/src/endpoint*                                   # nothing
rg -n 'self\.menu|self\.config|self\.options|self\.sessions|self\.presentation|self\.notices' \
     crates/spaceport-gpui/src/endpoint*                                   # ~25, all in polling.rs and follow.rs
```

| file | Lines | `impl SpaceportWindow` | `gpui` | what it is |
| --- | --- | --- | --- | --- |
| `endpoint.rs` | 784 | 406-778 (372) | `Context` only | `Endpoint`, `Release`, `Redraw`, the endpoint IDs, `live_of`, `hosts`, `reset_selected`, switching |
| `endpoint/polling.rs` | 495 | 30-486 (456) | `Context` | the tick: drain, activate, sessions probe, OSC 52 writes |
| `endpoint/catalog.rs` | 383 | 211-380 (169) | `Context` | saved-host catalog loads/writes on workers, device reconcile |
| `endpoint/link.rs` | 375 | — | none | `Link`, `Failure`, `Transition`, `enter`, retry/backoff |
| `endpoint/wake.rs` | 187 | 146-187 (41) | none | sleep/path clock, `Trigger`, `recover_connections` |
| `endpoint/wake/network.rs` | 101 | — | none | macOS `Network.framework` monitor, `unsafe extern "C"` |
| `endpoint/fence.rs` | 161 | 100-155 (55) | none | `InputFence`, `SelectionFence`, `ConnectionRef` |
| `endpoint/fence/changes.rs` | 126 | — | `Context` | test-only: the identity changes each fence test applies |
| `endpoint/history.rs` | 123 | — | none | Back/Forward trail over focused panes |
| `endpoint/follow.rs` | 119 | 59-119 (60) | `App`, `WindowId` | whose focus a background host may take |
| `endpoint/phase.rs` | 109 | — | none | `Phase` projection from `Link`, `serving`, `may_dial` |
| `endpoint/order.rs` | 100 | 41-97 (56) | none | device order: `rank`, `moved`, arrangement |
| `endpoint/config_reload.rs` | 45 | — | none | the held `server.reload_config` |
| `endpoint/origin.rs` | 21 | — | none | four `fl!` words for `spaceport_sidebar::origin::Origin` |
| `endpoint/error.rs` | 12 | — | none | `NoHost`, `SurfaceSwitchingUnsupported` |
| `endpoint/link/fixtures.rs` | 73 | — | — | `cfg(test)` fixtures for `link.rs` |
| **production** | **3,068** | **~1,209** | | |
| `endpoint/lifecycle_tests.rs` + 60 files | 10,688 | — | `gpui::test` | the socket-driving lifecycle suite |
| `endpoint/tests.rs` + 11 files | ~1,658 | — | `gpui::test` | unit-shaped tests, several over a real window |
| `endpoint/catalog/tests*` | 281 | — | — | saved hosts and WSL, worker-free |
| `history/tests.rs`, `order/tests.rs`, `wake/tests.rs`, `origin/tests.rs` | 506 | — | none | the pure suites, and the ones that move first |
| `fence/changes.rs`, `link/fixtures.rs` | 199 | — | — | test-only support |
| **tests** | **~13,330** | | | |
| **total** | **~16,400** | | | the plan's 16.2k, grown |

Two measurements decide most of what follows:

- **`gpui` reaches only three production files** — `polling.rs`, `catalog.rs`
  and `follow.rs`, and only for `Context<SpaceportWindow>`, `App::windows()` and
  `WindowId`. Nothing in the tree builds an element. The extraction is not a
  paint problem at all.
- **Text is not a blocker here.** 12 `fl!` sites over ~11 message IDs
  (`locales/en-US/endpoint.ftl`), against menus' 377. Only
  `Phase::status_label` (`phase.rs:104-106`) and `Origin::describe`
  (`origin.rs:10-17`) sit in code that would move; both are one-line app-side
  words (`spaceport-chat-core`'s `Labels` pattern) rather than a reason to keep
  the module.

## 1. Code context

The tree is one struct and a state machine around it.

`Endpoint` (`endpoint.rs:153-186`) is a plain struct with no GPUI handle, no
`Entity`, no `Subscription`:

```rust
pub(super) struct Endpoint {
    pub id: String,
    pub label: String,
    pub connection: ConnectionBridge,
    link: Link,
    /// The target the saved entry named when this endpoint was last reconciled
    /// against it. …
    saved: Option<ConnectTarget>,
    pub collapsed: bool,
    pub collapsed_repos: HashSet<String>,
    pub live: LiveState,
    pub(crate) last_seen: crate::palette::LastSeen,
    pub(crate) history: history::History,
    pub generation: u64,
    pub(crate) toasts: crate::notifications::Toasts,
    pub(crate) config_diagnostic: crate::config_diagnostic::ConfigDiagnostic,
    attempts: u32,
    recovery: Option<wake::Trigger>,
    initial_surface: bool,
    config_reload_pending: bool,
    focus_requested: bool,
    left_at: Option<Instant>,
    sounds: spaceport_sound::Policy,
}
```

`Link` (`endpoint/link.rs:5-21`) is the state machine: `Disabled`, `Detached`,
`Waiting`, `Dialing`, `Up`, each carrying `retry_at` and an optional `Failure`
(`Transport{reason, needs_user}`, `Missing`, `Stopped`, `Version`). The only
writer is `Endpoint::enter` (`link.rs:120-292`), entered by a `Transition`
(`link.rs:99-118`): `Stop`, `Detach`, `Retarget`, `Connect`, `Observe`, `Update`,
`Lost`, `RetryAt`, `ResetAttempts`, `Trade`, and a `cfg(test)` `Fixture`. The
retry curve is `retry_delay` (`link.rs:288-290`), `500ms << attempts.min(8)`
capped at `MAX_RETRY_DELAY` (`endpoint.rs:59-61`, 30 s), reset by a 60 s stable
period (`endpoint.rs:57`).

`Phase` (`endpoint/phase.rs:11-25`) is the *readable* projection of `Link` for
commands, pickers and the sidebar: nine variants, `as_str` for `endpoint-status`,
and `serving()`/`may_dial()` as the two narrow predicates other code asks for.

`polling.rs` is the window's tick (`poll_endpoints`, `polling.rs:75-112`) and is
the whole of the module's activity: `poll_connection_setup` (catalog + wake),
`poll_connections` (`drain_endpoint` per endpoint: snapshot, mutes, sound,
bell, OS clipboard, redial), `sync_selected_endpoint` (copy the endpoint's state
into the window's), `renew_activation` / `queue_retry_activation` /
`settle_navigation` (the activation fence dance), then `poll_sessions` (the
device picker's session probes).

`catalog.rs` loads upstream's saved SSH hosts and this client's WSL list on a
worker every two seconds until first success (`catalog.rs:104-124`), serializes
the selection write (`catalog.rs:161-186`), and `reconcile_devices`
(`catalog.rs:243-330`) rebuilds `self.endpoints[1..]` to match, retargeting the
ones whose saved entry changed.

`link.rs`'s `Release` (`endpoint.rs:73-151`) is a separate agreement machine for
the case that matters most and is least visible: an endpoint that said hello with
an *active surface* cannot be demoted without its boot ID, so switching away
defers the release, fences the destination, and treats queue-full as
backpressure rather than a dead transport.

## 2. Dependency audit

### 2.1 What the endpoint tree reads

Measured over the 15 production files, `self.<field>` on
`SpaceportWindow` — every one of them in `polling.rs` (21) and `follow.rs` (1):

| read | sites | where |
| --- | --- | --- |
| `self.notices` (show/clear) | 6 | `polling.rs:116,129,230,258,350,437` |
| `self.menu` (`page`, `unfolded_device`) | 5 | `polling.rs:156,450,451,466`, `follow.rs:90` |
| `self.options` (`ConnectOptions`) | 3 | `polling.rs:221,343,367` |
| `self.sessions` (probe/mutation/departure) | 3 | `polling.rs:454,457,458` |
| `self.presentation` (hold/resume) | 2 | `polling.rs:262,264` |
| `self.config.clipboard_toast` | 1 | `polling.rs:69` |
| `self.toasts_hidden`, `self.bell`, `self.sound`, `self.active`, `self.last_queued_options` | 5 | `polling.rs:156-172,214,367` |

Notably **absent**: `keymap` (zero reads; `polling.rs:105` calls
`self.sync_server_keymap(cx)`, which is `keymap`'s own method), `i18n` beyond
`fl!`, `direction`, `toggles`, `accessibility`, `fonts`. Endpoint drives input
gating but reads no bindings.

The one settings value read is cosmetic: `self.config.clipboard_toast.enabled`
(`polling.rs:69`) decides whether the OSC 52 flash is shown. Everything else
endpoint reads of the settings is `ConnectOptions`, a `spaceport-client` value.

### 2.2 `spaceport-widgets`: nothing, and that is the correct answer

`rg` for `crate::direction|crate::toggles|crate::accessibility|crate::fonts|
spaceport_widgets` over `src/endpoint*` returns **no matches**, and there is
nothing for them to serve: the tree builds no element. The endpoint program
needs no widget layer, before or after #220. Its UI (host rows, device picker,
status bar, overview cards) lives in `sidebar/`, `menu/devices/` and
`overview.rs`, which are those programs' files.

### 2.3 `spaceport-menu`: no uses, four reads — and the arrow points the other way

Same result: no `spaceport_menu` / `crate::menu::` use in the endpoint model.
The four `self.menu` reads above are *window state* reads (is a page open? which
device is unfolded?), not menu-crate calls; after #223/#225 they become
`self.menu.page()`-shaped reads and change nothing here.

The direction of the dependency is worth stating because it inverts the plan's
"mutual blocker" framing: **menus read endpoint far more than endpoint reads
menus.** `menu/devices/host_menu.rs:78,104,116` uses `WSL_PREFIX`, `LOCAL` and
`saved_profile_id`; `menu/devices/picker.rs:568` and
`palette/go_to.rs:228` take `&crate::endpoint::Endpoint`; `devices_overview/
model.rs:9` takes `{Endpoint, Phase}`; `notification_mutes.rs:78,467` takes
`&Endpoint`. So endpoint stage 3, not menu, is what unblocks those files — and
endpoint stage 1 (the ID policy and `Phase`) can land without waiting for
`spaceport-menu` at all.

### 2.4 The two app types that block the model

Everything `Endpoint` holds is gpui-free, but two of those types are only
gpui-free *by accident* and have no crate:

| type | file | lines | `gpui` | what it names that keeps it in the app |
| --- | --- | --- | --- | --- |
| `LiveState` | `state.rs:63-160`, `state/activation.rs` | 826 + 318 | 0 | `notice::{Notices, Cause}`, `notifications::Notice`, `daemon::LocalPeerWarning`, `i18n::fl` (9 sites), `OwnedNavigationTarget` (`navigation.rs`, 37) |
| `ConnectionBridge` | `connection.rs` + `connection/reader.rs` | 643 + 117 | 0 | `daemon::{connect, LocalPeer, is_missing_installation}` (379), `links::{LinkInbox, LinkCell}` (254), `scrollback::Inbox`, `terminal::InputTarget`, `config::Theme` (`host_theme`, `connection.rs:186`), `i18n::fl` (reader) |

`state/projection.rs` (172) and `state/agent_recency.rs` (57) go with
`LiveState` and name only `spaceport_client`. `notice.rs` is 166,
`notifications.rs` 619 (of which the `Notice`/`Toasts` model is a fraction),
`palette/last_seen.rs` 243 and `config_diagnostic.rs` 219 are all gpui-free and
already model-shaped — they need a home, not a rewrite.

### 2.5 What the sidebar needs out of this

`spaceport-sidebar` today receives host facts as loose values (`index`, `id`,
`label`, `status`) and the app does the paint. The app-side files that hand it an
`&Endpoint` are `sidebar/rail/render.rs:128,212,370`,
`sidebar/render/rows/workspaces.rs:27`, `sidebar/render/agent_rows.rs:102`,
`sidebar/render/device_tree.rs`, `sidebar/host_row.rs`, `sidebar/order.rs:19-84`,
`sidebar/search.rs:268-269` and `sidebar/render/rows.rs:128-212`. Between them
they read eight things: `id`, `label`, `collapsed`, `collapsed_repos`,
`enabled()`, `live.status`, `phase()`/`outage()`, and `generation` (through
`live_of`).

So the value type the sidebar program is actually waiting for is a **`Host`
view** in the endpoint crate — the sidebar's `Painters` pattern: `HostId(&str)`,
`label(&str)`, `Phase`, `ConnectionStatus`, `collapsed: bool`,
`collapsed_repos: &HashSet<String>` — prepared app-side from `&Endpoint` and
`live_of`. `Endpoint` itself cannot cross that line: it owns a socket bridge, a
`Toasts` queue and a `LastSeen` cache, and none of those belong in a row painter.

## 3. The constraint: `self.live` is the window's own copy of the selected endpoint

This is the subtle rule the whole design has to preserve, and it is worth
stating exactly, because the code that enforces it is three lines and the code
that depends on it is 26 call sites.

The rule is documented on the accessor (`endpoint.rs:465-476`, verified current):

```rust
    /// The current state of endpoint `index`: the window's own copy for the
    /// selected endpoint, the endpoint's otherwise. `reset_selected` copies
    /// the endpoint's state into the window's; from then on only the window's
    /// copy follows the selected connection.
    pub(crate) fn live_of(&self, index: usize) -> &LiveState {
        if index == self.selected_endpoint {
            &self.live
        } else {
            &self.endpoints[index].live
        }
    }
```

Two fields hold the same thing for the selected endpoint, and they are **not**
interchangeable at any instant: `SpaceportWindow.live` (`window.rs:222`) is the
one the window has *stamped* — `selection_epoch` (`window.rs:268`),
`selected_generation` (`window.rs:264`), the activation fence
(`state/activation.rs`), `last_queued_options` — while `Endpoint.live`
(`endpoint.rs:166`) is whatever that endpoint's own poll last took from the
inbox. `endpoint/tests/live_of.rs` is the guard: its fixture deliberately makes
the selected endpoint's own copy differ from the window's (`booted("stale")`
against `booted("window")`, `live_of.rs:40-53`) so a reader of the wrong copy
shows, and it pins the three consumers that must not regress — `live_of`
itself, `hosts()`, and the overview's rows (`live_of.rs:56-95`).

**`reset_selected` is not the only writer.** Measured, `self.live = ` appears
five times:

| site | what it writes | why |
| --- | --- | --- |
| `endpoint.rs:497` (`reset_selected`) | `endpoint.live.clone()` | the selection changed, or the generation moved on |
| `polling.rs:251` (`sync_selected_endpoint`) | `endpoint.live.clone()` | the selected endpoint's poll moved on |
| `polling.rs:372` (`queue_retry_activation`) | `state.clone()` from the **inbox** | the window queues an activation and takes the inbox's stamped state |
| `window/commands.rs:317` | `state.clone()` from the inbox | the same, for a command's own surface claim |
| `group_terminals.rs:428` | `endpoints[selected].live.clone()` | a workspace switch parks and resumes the group |

So the invariant is not "one writer"; it is **one direction with two
authorities**: the window's copy is authoritative for the selected endpoint
between polls, and every write is either "take the endpoint's copy after it
moved" or "take the inbox's copy after the window stamped it". A design that
collapses the two fields into one breaks `polling.rs:372` and
`window/commands.rs:317` (the window's queue position would be overwritten by
the next poll) *and* `live_of` for the other hosts.

### 3.1 How the extraction preserves it

Four rules, each mechanical enough to review:

1. **Two fields stay two fields.** `SpaceportWindow.live` and
   `Endpoint.live` both survive the program unchanged. No stage merges them, and
   no stage replaces `Endpoint.live` with a borrow of the window's.
2. **Only the rule lowers, not the storage.** The crate exports the predicate,
   not the fields:
   ```rust
   pub fn live_of<'a>(
       hosts: &'a [Host],
       selected: usize,
       selected_live: &'a LiveState,
       index: usize,
   ) -> &'a LiveState { … }
   ```
   The window keeps `self.live` and calls
   `spaceport_endpoint::live_of(&self.endpoints, self.selected_endpoint, &self.live, index)`.
   `hosts()` becomes a tiny app-side iterator over the same call.
3. **`live` never becomes a public field of the moved `Host`.** In the crate it
   stays `pub(crate)`, reachable only by the crate's own poll and transition
   code; the app's read paths go through `live_of`. If a stage-3 reviewer finds
   `endpoint.live` at a call site outside `endpoint/`, the stage is wrong.
4. **The guard test stays app-side and gains the write sites.** `live_of.rs` is a
   `#[gpui::test]` (it needs a window) so it does not travel with the crate. It
   must gain one case per row of the table above — at minimum a
   `queue_retry_activation` case asserting the window's copy is the inbox's and
   the endpoint's copy is not — before stage 3 lands. `fence/changes.rs:34-84`
   is the existing harness for applying exactly those identity changes one term
   at a time; `Change::ReconnectUnseen` ("polling has not yet reset the window
   onto it") is precisely the disagreement window this rule is about.

### 3.2 Chat hit the same distinction, and its answer is the precedent

`chat/start.rs:274,355-359` fences a chat start on
`(self.selection_epoch, self.endpoints[self.selected_endpoint].generation)` plus
`self.live.status.is_connected()` — the window's stamped copy, checked against
the endpoint's generation. `chat::Source` (`Source::Herdr` vs
`Source::Program(EndpointId)`) is the same statement one level up: *which*
snapshot a chat belongs to, not just which host. Chat's stage 4 waited for the
live-state crate for exactly this reason, and the endpoint program should reuse
that answer rather than invent a second one: the epoch/generation pair is a
fence value (`endpoint/fence.rs:103-127`), the live state is the crate's, and
the rule that the selected endpoint reads the window's copy is a function.

## 4. `LiveState` and the snapshot coherence rules: lower or leave

`notes-lower-shared-types-design.md` §1.2 is the prior art and it is still
accurate: `state.rs` is gpui-free, `LiveState` is 99 lines of fields
(`state.rs:63-160`) and "nothing in it is an `Entity`"; §2.2 declined to create
`sheprd-live` yet and named its trigger ("whichever feature first needs to read
live state from outside `impl SpaceportWindow`"). Endpoint stage 3 is that
feature, so the crate becomes justified — but only for the parts that are
*rules*, not for the parts that are words and effects.

| piece | lines | worth lowering? | why |
| --- | --- | --- | --- |
| `ConnectionStatus` (`state.rs:18-59`) | 42 | **yes** | six variants and `is_connected`; the `label` words stay app-side (six `fl!`) |
| `LiveState` fields + `Clone` | ~99 | **yes** | the value the endpoint model must name; no GPUI, no I/O |
| `apply(ClientEvent)` (`state.rs:520-…`) | ~250 | **yes, with a `Labels`** | a pure reduce over protocol events; the 9 `fl!` sites become `Labels` fields, as `spaceport-chat-core` did |
| `only_surface_changed` (`state.rs:301`) | — | **yes** | the repaint predicate; `Redraw` (`endpoint.rs:65-72`) is its return type and lowers with it |
| `coherent` (`state.rs:819-826`) | 8 | **yes** | the snapshot/surface same-boot-same-revision rule, already a free function |
| `state/projection.rs` | 172 | **yes** | the "one newer projection waits for its cells" pair |
| `state/activation.rs` (`Fence`, `Transition`, 5 s `TIMEOUT`) | 318 | **yes** | pure; needs `OwnedNavigationTarget` (37) to travel with it |
| `AgentRecency` (`state/agent_recency.rs`) | 57 | **yes** | pure |
| `notices` / `notifications` mailboxes | — | **as data, not as policy** | the `VecDeque`s and `Notice`/`Notices` types move; the drain sites and every word stay in the window (`toasts.receive`, `notices.show`) |
| `sound_events`, `bells`, `clipboard_writes`, `sound_cancel` | — | **as data** | they are already `VecDeque<String>`/`u16`/`Arc<AtomicBool>`; the effects (`spaceport_sound::Policy`, `cx.write_to_clipboard`, `bell.queue`) are the app's |
| `daemon::LocalPeerWarning` (`local_peer_warning`) | — | **no, yet** | it is `daemon`'s; it waits for stage 4 or stays a field the app does not lower |
| `window_title` sanitizers, `Display` for `ConnectionStatus` | — | **no** | words; they live where the window and the log are |

The net is `spaceport-live` at roughly **1,373 production lines** from
`state*` (826 + 318 + 172 + 57), plus the model halves of `notice.rs` (166) and
`notifications.rs`, plus `navigation.rs` (37), with a `Labels` struct carrying
the ~15 `fl!` sites that move. That is the largest single piece of the endpoint
program, it is shared with chat stage 4, and it is the cheapest *real* win in
the sidebar's dependency chain (three features read `LiveState`).

## 5. The natural seams, ranked

Ranked by (moves alone) ÷ (lines), with what each needs:

| # | seam | lines | needs | verdict |
| --- | --- | --- | --- | --- |
| 1 | `History` + `Step` (`history.rs`) | 123 + 191t | `spaceport_client::protocol::ClientShellSnapshot` only | **moves now.** Zero app types, zero time, zero I/O; a browser-style trail over one snapshot field. Its own module doc already reads like a crate's. |
| 2 | `WakeClock`, `Trigger`, `NetworkChanges` (`wake.rs:38-143`) | 106 + 192t | `std::time` | **moves now**, with the macOS monitor (`wake/network.rs`, 101) as a `cfg(target_os = "macos")` module. Carries `unsafe extern "C"` and the `block2`/`dispatch2` deps; §7 R5. |
| 3 | device order `rank` / `moved` (`order.rs:21-39`) | 40 + 114t | `spaceport_reorder::Beside` | **moves now.** Pure functions over `&[&str]`; the window half (`displayed_devices`, `arrange_devices`, `move_device`) is glue and stays. |
| 4 | endpoint IDs (`endpoint.rs:42-58`) | 17 | nothing | **moves now.** `LOCAL`, `SAVED_PREFIX`, `WSL_PREFIX`, `saved_profile_id`; three other programs already spell these (`menu/devices/host_menu.rs:78-116`). |
| 5 | `focus_requested` (`follow.rs:23-44`) | 22 + (tests in `tests/follow_focus.rs`) | two `ClientShellSnapshot`s | **moves now.** The predicate only; `follow_focus` (WindowId, `App::windows`, typing quiet) stays. |
| 6 | `Phase` + `serving`/`may_dial` (`phase.rs`) | 109, minus one word | `Link`, `ConnectionBridge`, `VersionMismatch` | **waits for 8.** The enum and `as_str` are pure; the projection needs `Link`, and `serving`/`may_dial` need the bridge's `handle`. |
| 7 | `Catalog` (`catalog.rs:15-208`) | 197 + 281t | `spaceport_client`'s saved-host loaders (already a crate), `storage_warning`, order's `rank` | **moves after 3**, as soon as the loaders are reachable. `Catalog` never touches `Endpoint` — only `reconcile_devices` does. |
| 8 | `Link`, `Failure`, `Transition`, `enter`, retry curve (`link.rs`) | ~290 | `Endpoint` ⇒ `LiveState` + `ConnectOptions` | **waits for stage 3** (`spaceport-live`). This is the state machine, and it is the part worth moving. |
| 9 | fence values (`fence.rs:19-45,72-94`) | ~70 | `Arc<Mutex<LiveState>>`, ids, generations | **waits for stage 3.** The predicates (`fence_holds`, `selection_fence_holds`, `endpoint_for`) are window reads and stay. |
| 10 | `Release` + `Redraw` (`endpoint.rs:65-151`) | 89 | `ClientHandle`, inbox CAS, `Instant` | **stays.** It is a window-owned agreement about a socket; `Redraw` may lower with `only_surface_changed` (seam 4's row). |
| 11 | `polling.rs`'s tick (456) | 456 | `self.notices`, `self.menu`, `self.sessions`, `self.sound`, `cx: Context` | **never moves.** It *is* the window. |
| 12 | `reset_selected` (372 within `endpoint.rs`'s block) | 372 | 15 window fields, `cancel_file_transfer`, `clear_pending_input`, `discard_composition` | **never moves.** |

The ranking is the plan: 1-5 are stage 1, 7 is stage 2, 8-9 are stage 3 after
`spaceport-live`, 6 rides along with 8, and 10-12 are permanent app-side.

## 6. The staged plan

Every stage is independently shippable, changes no behaviour, and states what to
survey first and what proves it done. Verification for each, as the split plan's
§"Next extractions" requires: `cargo check -p <crate> -p spaceport-gpui`,
`cargo test -p <crate>`, `cargo fmt --all -- --check`,
`just check-file-size`, `just check-sorted`, `just check-i18n`,
`cargo metadata` for the lockfile.

### Stage 1 — `crates/spaceport-endpoint`, the dependency-free policy (~290 production, ~500 test)

The four pieces that need nothing from the app: `History`, `WakeClock` +
`Trigger` + `NetworkChanges` (and the macOS monitor), `order::{rank, moved}`,
`focus_requested`, and the endpoint-ID policy. Dependencies: `spaceport-client`
(the snapshot type), `spaceport-reorder` (`Beside`), `std`. `tracing` for the
wake log lines.

**Survey first.** Confirm `History` names only `ClientShellSnapshot`; confirm the
five `order` tests and the four `history` tests drive public functions and not a
window; confirm `wake/tests.rs` builds a `WakeClock` directly; confirm nothing
under `src/endpoint*` outside `history.rs`/`wake.rs`/`order.rs` reads
`History`'s private `travel` field.

**Exit criteria.**
- `crate::endpoint::{History, Step, WakeClock, Trigger, NetworkChanges, watch_network, network_changes, rank, moved, LOCAL, SAVED_PREFIX, WSL_PREFIX, saved_profile_id, focus_requested}` resolve exactly as today, through re-exports, so **no file outside `src/endpoint/` changes** — the `#143` discipline.
- The moved suites (`history/tests.rs` 191, `wake/tests.rs` 192, `order/tests.rs` 114) pass in the crate, unchanged; the app keeps no copy.
- `Endpoint`'s fields change type, not shape (`history: spaceport_endpoint::History`, `recovery: Option<spaceport_endpoint::Trigger>`, `wake: spaceport_endpoint::WakeClock`).
- The crate's `error.rs` holds `endpoint::Error`? **No** — `NoHost`/`SurfaceSwitchingUnsupported` are `switch_endpoint`'s and stay in the app until stage 3 changes the caller.
- `just check-file-size` clean (`wake.rs` splits anyway: the clock levers out, `recover_connections` stays).

### Stage 2 — `Catalog` and the device order (~200 production, 281 test)

Move `Catalog`, `CatalogUpdate`, `load`, `store`, `devices`, `entry_changed`
into the crate; keep `reconcile_devices` and `restore_selection` app-side.

**Survey first.** Whether `StorageFailure::find` can be reached without moving
`storage_warning` (it is a value type over `spaceport-state-file`); whether
`spaceport_client::load_saved_hosts`/`load_wsl_hosts`/
`load_saved_host_selection`/`store_*` are public (they are called from
`catalog.rs:78-99` today, so yes); that `Catalog::new(&ConnectTarget)` reads
only `ConnectTarget`.

**Exit criteria.** `reconcile_devices`'s 88 lines shrink to calls
(`crate::endpoint::devices(hosts, wsl)` returns the `Device` list the app sorts
and reconciles); `catalog/tests*` (281) move; `Catalog` has no `Context`
parameter anywhere; the reconcile behaviour tests
(`lifecycle_tests/catalog_link.rs`, `tests/catalog_link.rs`) pass untouched.

### Stage 3 — `spaceport-live`, the live-state crate (~1,400 production + models)

The prerequisite stage, shared with chat stage 4. Move `state.rs`,
`state/activation.rs`, `state/projection.rs`, `state/agent_recency.rs`,
`navigation.rs`, and the model halves of `notice.rs`/`notifications.rs`; supply
a `Labels` struct for the ~15 `fl!` sites.

**Survey first.** Enumerate every `LiveState` construction outside `state.rs`
(tests included: `tests/live_of.rs:26-37`, `sidebar/layout_tests` fixtures) —
`Default` must cover them; count `apply`'s `ClientEvent` arms that raise a
notice (7 today) and decide which become `Labels` fields versus `Cause` values;
confirm `Answer`/`DialogResponse` (`state.rs:18`, `Arc<crate::Error>`) can name
a crate error rather than the app's root `Error`.

**Exit criteria.** `spaceport-live` has zero `gpui` and zero app imports; the
app's `crate::state` module is re-exports plus `apply`'s callers; chat can
construct its `Source` fence from the crate's values; `state/tests/*` move or
are re-pointed one file at a time.

### Stage 4 — `Endpoint` becomes `Host`, the link machine lowers (~1,300 production)

The stage this document is named for: `Link`, `Failure`, `Transition`, `Phase`,
the fence values, `live_of`'s rule, and `Endpoint` minus its bridge. The crate
returns **typed actions** the window performs on its socket:

```rust
pub enum Action { None, Detach, Dial { active: bool }, Retire, CheckLiveness }
```

with `Host::enter(Transition) -> Action`, so `ConnectionBridge` never crosses the
line. `connect`, `stop`, `reconnect`, `detach_endpoint`, `switch_endpoint`,
`reset_selected` and the whole of `polling.rs` stay in the app and become the
action performers.

**Survey first.** Every place that mutates `Endpoint.connection` directly
(`catalog.rs:288-306`'s two `Retarget` calls, `fence/changes.rs:66-72`,
`chat/tests/start.rs:167`, `tests/live_of.rs`'s `never_connect`) — each becomes
either a `Host` transition or a documented test hook; and every read of
`endpoint.live` outside `src/endpoint/` (should be **zero** — if not, stage 3's
rule 3 was violated).

**Exit criteria.** `spaceport_endpoint::Host` is gpui-free, holds `Link`,
`LiveState` and the counters, exposes `phase()`, `serving()`, `may_dial()`,
`outage()`, `retry_at()`, `enabled()`, `detached()`, `live()` (crate-internal);
`live_of` is a crate function called by the window; `endpoint/tests/live_of.rs`
passes with its new write-site cases; the crate's tests are the pure suites
(`link/fixtures.rs` 73 travels, the socket suites do not).

### Stage 5 — the `Host` view the sidebar asks for (~80 production)

`spaceport_endpoint::HostView { id: &str, label: &str, phase: Phase, status:
ConnectionStatus, collapsed: bool, collapsed_repos: &HashSet<String> }`,
prepared in one app-side function. Then `spaceport-sidebar`'s row and rail
renderers take the crate's view instead of `&crate::endpoint::Endpoint`, which is
what lets the sidebar program's render lane follow (§2.5).

**Exit criteria.** No file under `crates/spaceport-sidebar/src` names
`Endpoint`; `sidebar/rail/render.rs`, `render/rows{,/workspaces}.rs`,
`render/agent_rows.rs` take `HostView`; the layout tests
(`sidebar/layout_tests/*`, which build `Endpoint::new`) go through one helper.

### The honest remainder

| after the program | lines | where |
| --- | --- | --- |
| the window glue (12 in §5) | ~1,210 | app, forever |
| `ConnectionBridge` + `reader.rs` (stage 4 of the original list, never in this plan) | 760 | app, until `daemon`, `links`, `scrollback`, `terminal::InputTarget` and `config::Theme` all have homes — i.e. four other programs |
| the socket-driving lifecycle suite | 10,688 | app, forever (it opens sockets and real processes) |
| `Release`, `Redraw`, the endpoint `Error` | ~100 | app |

So endpoint ships **~1,900 production lines below GPUI (62% of its production,
18% of its tree) in five PRs**, two of which (`spaceport-live` and the `Host`
view) other programs want anyway. What it does **not** ship is the 760-line
socket half or any of the 13,330 test lines. That is the number the split plan's
sidebar row should carry.

## 7. Risks, ranked

**R1 — the double copy of `LiveState` (highest).** §3. Two fields, five writers,
26 `live_of` call sites. A stage-4 refactor that "simplifies" `Endpoint.live`
away breaks the window's stamped copy at `polling.rs:372` and
`window/commands.rs:317` and silently makes the selected host read one poll
stale. The only guard is a `#[gpui::test]` that must stay app-side; keep it and
extend it *before* stage 3.
**Mitigation:** §3.1's four rules, rule 3 enforced by review (`endpoint.live`
must not appear outside `src/endpoint/`), plus the `fence/changes.rs` harness.

**R2 — the program pays for itself only after two prerequisite crates.**
`spaceport-live` is 1,400 lines before a single endpoint line crosses a crate
boundary, and it carries its own text problem (9-15 `fl!`). If `spaceport-live`
stalls, stages 4-5 stall with it and endpoint delivers stage 1's ~290 lines.
**Mitigation:** the plan puts `spaceport-live` second, after a stage that proves
the crate, and stage 3 is justified independently by chat stage 4 and the
sidebar's three `&LiveState` reads.

**R3 — 81% of the tree is tests, and most of them cannot move.** The lifecycle
suite opens bound sockets and real processes (`endpoint/lifecycle_tests.rs:1-4`
says so). A reviewer reading "16.2k" will expect 16k to leave the app; 13.3k
stays. The crates ship ~500-800 test lines.
**Mitigation:** state the split in the PR body every time; never count test
lines as extraction progress.

**R4 — `polling.rs` is a single 456-line function cluster over 21 window reads.**
`queue_retry_activation` (`polling.rs:309-380`) clones the whole inbox state
into the window while holding a `try_lock`, and `settle_navigation` writes both
the window's and the inbox's fence. Any attempt to "move polling" drags
`notices`, `menu`, `sessions`, `sound` and `presentation` with it.
**Mitigation:** stage 4's boundary is the `impl SpaceportWindow` line, full stop.
`polling.rs` is not in scope for any stage.

**R5 — the macOS network monitor carries `unsafe` and two crates.** Moving
`wake/network.rs` (101) into `spaceport-endpoint` moves five `unsafe extern "C"`
declarations and the `block2`/`dispatch2` dependencies, and the only CI job that
proves it is the macOS one — a Linux or Windows run cannot see the gate
(`AGENTS.md` "A `cfg` gate is only proven on the platform it names").
**Mitigation:** if the reviewer prefers, keep `watch_network`'s monitor app-side
and move only `NetworkChanges`/`WakeClock` (stage 1 then touches no `unsafe`).
That is the conservative default; the doc counts the monitor as optional.

**R6 — `spaceport-menu` may change `self.menu`'s shape under stage 1.** Polling
and `follow.rs` read `self.menu.page`/`unfolded_device` (5 sites). If #225's
`MenuState` split renames them while stage 1 lands, the conflict is small but
certain.
**Mitigation:** stage 1 touches no `self.menu` line; rebase before starting and
re-run §0.

**R7 — the sidebar will want `Endpoint` and should get `HostView`.** Handing
`&Endpoint` to `spaceport-sidebar` (the tempting shortcut, since seven app-side
files already do) would move a socket bridge and a toast queue into a row
painter. Stage 5 exists to prevent it, and it must land before the sidebar's
render lane moves, not after.

## 8. Stage 1's exact task list

One PR. Every task names its files; nothing outside `src/endpoint/` changes
except the two `Cargo.toml`s, the workspace members, `AGENTS.md`'s crate table
and `docs/gpui-split-plan.md`'s feature table.

1. **Survey and record.** Run §0's commands; write the base revision and the five
   counted numbers into the PR body. Confirm `spaceport-endpoint` does not exist
   and that no crate in `crates/` is named `spaceport-endpoint`.
2. **Create the crate skeleton.** `crates/spaceport-endpoint/src/lib.rs`
   (module decls and re-exports only), `src/error.rs` (empty `pub enum Error`
   with a doc comment saying why nothing fails yet, or omitted until something
   does), `Cargo.toml` with `spaceport-client` and `spaceport-reorder` inherited
   from the workspace, `[workspace.dependencies]` entries, `Cargo.toml`'s
   `[dependencies]` in `spaceport-gpui`.
3. **`history.rs`.** Move `History`, `Step`, `LIMIT`, and the doc comment
   (`endpoint/history.rs:1-123`); move `endpoint/history/tests.rs` (191) to
   `src/tests/history.rs` unchanged except `use super::*;`. Fields become
   `pub(crate)`-to-crate; the app only uses `observe`, `can`, `peek`, `begin`
   (`endpoint.rs:215-217`, `polling.rs:186`, and `Step` consumers).
4. **`wake.rs`.** Move `Trigger`, `NetworkChanges`, `SLEEP_GAP`,
   `NETWORK_SETTLE`, `WakeClock` and its three tests' inputs
   (`endpoint/wake.rs:38-143`); move `endpoint/wake/tests.rs` (192). Keep
   `recover_connections` app-side (`wake.rs:146-187`) — it writes
   `endpoint.recovery` for every endpoint. Decide R5: conservative default moves
   only `NetworkChanges`/`WakeClock`, leaving `network.rs` behind a one-line
   shim; the aggressive option moves `network.rs` too and needs a macOS run.
5. **`order.rs`.** Move `rank` and `moved` (`order.rs:21-39`) plus their tests
   (`order/tests.rs` 114). Keep `displayed_devices`, `arrange_devices`,
   `move_device` app-side.
6. **`ids.rs`.** Move `LOCAL`, `SAVED_PREFIX`, `WSL_PREFIX`,
   `saved_profile_id` (`endpoint.rs:42-58`) and re-export from
   `crate::endpoint` for `menu/devices/host_menu.rs`, `palette.rs:541`,
   `core_terminals.rs:616`, `palette/project_open.rs`.
7. **`follow.rs`.** Move `focus_requested` (`follow.rs:23-44`) and its tests
   from `endpoint/tests/follow_focus.rs`; keep `observe_focus`, `follow_focus`,
   `leave_selected`, `shown_elsewhere` app-side.
8. **Façade and re-exports.** `src/endpoint.rs` keeps every path: `pub(super)
   use spaceport_endpoint::{History, Step, WakeClock, Trigger, NetworkChanges,
   watch_network, network_changes, LOCAL, SAVED_PREFIX, WSL_PREFIX,
   saved_profile_id, rank, moved, focus_requested};` or a `pub(crate) use` where
   the old item was. No other file's import changes.
9. **Sorting and size.** `// keep-sorted start`/`end` around the crate table row
   in `AGENTS.md` and any new `use` list that is a growing list; confirm no file
   crosses 1,000 lines (`lib.rs` will be wiring only).
10. **Verify.** `cargo check -p spaceport-endpoint -p spaceport-gpui`,
    `cargo test -p spaceport-endpoint`, `cargo test -p spaceport-gpui
    endpoint::` for the pure suites that stayed, `cargo fmt --all -- --check`,
    `just check-file-size`, `just check-sorted`, `just check-i18n`,
    `cargo metadata` for the lockfile. Report what ran and what was deferred.

## 9. What this does not do

- It does not merge `SpaceportWindow.live` and `Endpoint.live`, and §3.1 forbids
  any stage from doing so.
- It does not move `polling.rs`, `reset_selected`, `Release`, or the lifecycle
  suite, and it says plainly that 1,210 production and 13,330 test lines stay.
- It does not create `spaceport-connection`; the 760-line socket half has four
  prerequisite programs and no consumer of its own yet.
- It does not decide the `spaceport-live` name or its `Labels` shape — stage 3's
  survey does, with `spaceport-chat-core`'s `Labels` as the precedent.
- It does not touch menus, palette, notifications or sidebar code. Stages 4 and
  5 change their imports; that churn is theirs to review.
