# T03 Core tab tear-off: Core-only window prerequisite

Source revision: `c3cfdf82`. This is a source-grounded dependency map after a
headless red test. No production route changed, and no Core or personal daemon
was started.

## Observed route

`window/tab_drag.rs:347-352` sends an outside drop to `tear_off_tab`. That
method reads `browser_key()` (`window/tab_tear.rs:52-59`), which deliberately
returns `None` while a Core group is selected (`browser/view.rs:178-183`). A
Core page therefore produces no `TornTab` and stays in its source window.
The headless test first had a test-only closure signature error; after fixing
it, `cargo +1.99.0 test --locked --offline -p sheprd-gpui --bin spaceport
a_core_tab_tears_to_the_same_group_and_keeps_the_other_tabs_hidden` failed
because no `TornOff` record existed. The test was removed with the proposed
patch when the constructor boundary below ruled out a safe narrow fix. This
failure is a route observation, not a passing regression.

`TornTab.workspace` can carry the typed `Scope::Core` and canonical group
UUID introduced by the prior slice (`window/tab_tear.rs:18-38`,
`browser/store.rs:70-162`). Hiding and returning a tab use that key
(`browser/hidden.rs:24-78`). Yet `arrive` and `land_arrival` select a legacy
endpoint and navigate to a legacy workspace (`window/tab_tear.rs:132-187`).
A key-only change there would still open a new window through
`app::open_additional_window_then` (`app.rs:178-204`). Its only launch argument
is `ConnectTarget`, and `open_window_placed` passes that to
`SheprdWindow::new` (`app.rs:107-151`). The constructor creates a legacy
endpoint catalog and calls `reconnect()` (`window/construct.rs:87,326-334`).
The bridge then calls `daemon::connect` (`connection.rs:165-200`); a Local
target may start `herdr server` when no socket answers (`daemon.rs:40-73`).
Using a copied legacy target for a Core-only tear-off would create a new
legacy connection and could start Herdr 1. This is outside the Core-only
runtime contract.

The next coherent cut needs an explicit Core-first window launch identity
through the app constructor and window/control target selection, with an
attach-only Core Hub join. It must make the retained window chrome, layout,
tab close/focus, and CLI `window new --window` resolve without a legacy
endpoint (`control/server/app.rs:199-249`). The arrival must recheck the
exact hub/document/group and TabId against a fresh observation
(`core_terminals/workspace_ui.rs:224-268`), return the source tab if that
check fails, and preserve the existing legacy tear-off tests in
`window/tab_strip/tests/tear_off.rs`. A local Core tab is not a remote
machine mapping; no guessed alias or new Core RPC is required.

## Core-first cut order for review

The source review at `aa95cb07` recommends migrating retained window readers
and commands to Core before removing the Herdr 1 constructor/bridge, then
implementing tear-off through the sole Core window path. A temporary dual
launch enum would keep the legacy reconnect path alive while more than 500
endpoint/selected-index references across the app still assume it. The
specification already requires Core as the only runtime owner; this note does
not authorize a smaller view or a fabricated endpoint.

The final window launch needs the expected persistent local Core hub ID, and
an ephemeral observation epoch for a delayed handoff. The full
hub/machine/document/group key and `TabId` stay with `TornTab` and the Store.
The existing per-window Hub can join with `Start::Join`, which never starts
Core, but a blank Core-first window needs an explicit join kickoff. Before
changing hidden maps or layout, arrival must compare the joined hub ID and a
fresh observation's epoch, group and tab identity. A failed open or stale
handoff must return the tab to its source window. UI New Window and CLI
`window new --window` need the same launch identity and exact refusal;
`window list` must report the Core target truthfully, with a control-protocol
version bump if its response changes. Tests must cover no legacy connection
or start on Core window construction, join-only attachment, stale handoff
rollback, tab return on close, CLI targeting, and native drag/focus.

## Verification limit

The red test used an isolated headless window and the warm Cargo target with
`CARGO_INCREMENTAL=0`. The existing macOS compact-unwind linker warning and
`block v0.1.6` future-compatibility notice appeared. No green app test or
native window interaction ran for this investigation. T03 remains
`in_progress`; A33 remains `not_run`. The Core tear-off path still needs a
Core-first window constructor before it can be presented as working.
