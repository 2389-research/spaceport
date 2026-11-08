# T03 typed Core group key and local view selection

Source: isolated `wip/spaceport-t03-typed-group` branch based on `42664731`.
This implements the [route map](t03-typed-group-key-dependency.md) in the
existing Store and Layouts authority. It does not change Herdr Core or the
legacy terminal engine.

`Scope` now reads old flat strings as legacy scopes and writes Core scopes as
objects with an explicit `kind: core`, hub Core ID, optional permanent machine
ID, and document name. The group UUID remains the existing workspace field,
but must use canonical UUID spelling. Store and Layouts refuse malformed,
unbounded, or noncanonical Core keys; Core layouts also refuse Herdr tab picks.
Old flat records are still legacy, even if their workspace string matches a
Core UUID. Tabs keep one app-global ID counter; Core groups use the existing
layout, hidden-tab, and closed-tab maps. The identity includes the document
name and permanent machine ID, so equal labels or group UUIDs do not merge
across machines or documents. Remote Core groups are representable as saved
keys, but the GUI does not claim a remote document action route from them.

The Core workspace click uses a fresh app-global observation to select an
exact local key. The strip, group pick, pane, sidebar, closed-tab history,
and saved layout all read that key. A stale observation, changed hub ID, or
missing group hides old Core tabs; the next observation poll clears the old
selection. A Core member click opens or reuses its terminal tab in that group.
Legacy navigation and host selection leave the Core group. A new tab in a Core
group uses the existing Core terminal command even when the legacy preference
would choose Herdr 1. A queued Core tab spawn checks the hub identity again
before the worker starts it. Core chat starts through the existing in-pane
adapter only when a Core terminal tab is selected; a blank group refuses the
chat action instead of sending it to a Herdr 1 workspace.

The local control socket still targets tabs by their app-global TabId and
preserves `--window` selection. `tab focus` uses the exact observed Core key,
not a machine alias or a legacy workspace string. `tab close` rechecks that
key in the window update, including after a prior lookup; closing a background
Core tab leaves the window's current group selected. The older flat
`TabInfo.host`/`workspace` fields remain display fields; the saved Core key,
not those strings, drives actions.

## Verification and limits

- Red: the headless Core-only selection test returned an empty tab list after
  a valid Core group click because the strip still read the legacy key.
  The first compile also failed on the absent test observer identity setter.
- Green: `CARGO_TARGET_DIR=/tmp/spaceport-t04-install-offer-target
  CARGO_INCREMENTAL=0 cargo +1.99.0 test --locked --offline -p sheprd-gpui
  --bin spaceport core_`: 125 passed, 7 opt-in private Core tests ignored.
  This includes Core Store/Layout identity, local group switching, queued
  spawn identity, headless CLI focus and background close, closed-page reopen,
  pending focus cleared after a group switch, and blank-group chat refusal.
  The initial layout assertion failed only
  because a deserialized f32 writes `1.0` rather than JSON `1`; it now compares
  typed `SavedLayout` and its page pick.
- `just check-i18n`: 20 tool tests passed; 4,327 messages valid and used.
- `cargo +1.99.0 check --locked --offline -p sheprd-gpui --bin spaceport`
  with the same target and `CARGO_INCREMENTAL=0`: passed with no new source
  warnings. The same command with `--all-features` passed. Cargo reported the
  pre-existing `block v0.1.6` future-compatibility notice; the test linker
  reported the macOS `__eh_frame` compact-unwind warning.
- `just format-check`, `just check-file-size`, and `just check-sorted`: passed.
  Package `tools/spec.py check` passed its schema and 174 local-link checks.
  `git diff --check` passed. These checks do not prove native UI behavior.

The suite is headless. Native macOS focus, input, narrow-window layout, and
Windows/Linux behavior were not run. The pinned private Core binary was not
available in this worktree, so the local saved-group path has no new live
Core scenario. Existing private Core document/adapter evidence does not prove
this tab-selection path end to end. Core tab tear-off still needs a new-window
identity handoff; the legacy tear-off route cannot carry a Core key. Remote
group UI needs a fresh remote document keyed by permanent machine ID. G-CHAT
native adapter parity remains open. T03 stays `in_progress`; A09 and A24 stay
`not_run` because their full restart/offline scenarios are broader than this
slice. The legacy bridge still runs during this transition.
