# T04/RM-TELEPORT: current cut order

Read-only source audit at `6ab2fa44`, after the Teleport CLI/control cut. This
updates the older reachability map in `t04-cloud-teleport-dependency.md`; it
does not change the app, Core, a user checkout, or a service.

## Live routes and shared state

- The old `teleport --to` CLI and `WorkOp::Teleport` wire request are absent;
  `crates/sheprd-gpui/src/cli/tests/removed_teleport.rs` and
  `crates/sheprd-control-protocol/src/protocol/work/tests/removed_teleport.rs`
  assert refusal and retained target identity. The native workspace menu is
  still live: `menu/workspace.rs` offers and dispatches Teleport, Back, Go To,
  and Clear actions; `menu/teleport.rs` builds the migration source and
  destinations from legacy endpoint snapshots; `menu/chrome.rs`,
  `menu/chrome/keys.rs` and `teleport/ui.rs` draw and drive the dialog.
- Each window starts `sheprd_remote::teleport::Marks::start()` in
  `window/construct.rs`. `teleport/marks.rs` starts a thread to read and write
  `teleported.json` under the app state directory. `window.rs` calls
  `poll_teleport` each tick; it polls marks, migration progress, and follow
  navigation. `sidebar/render.rs` and `sidebar/workspaces.rs` draw marks.
  Removing only the menu row would leave this Teleport state reader running.
- `teleport_follow` and its `follow_teleport` tick logic are also used by
  `fan_out/ui.rs`, `dispatch/hosts.rs`, and `pane_menu/carry.rs`. The latter
  sets a pane target; a Teleport-only deletion of `teleport/ui.rs` would break
  those live callers. `pane_move.rs`, `chat/existing.rs`, `chat/convert.rs`,
  `dispatch/job.rs`, `fan_out/job.rs`, `cloud/image/prebuild.rs`, and
  `sheprd-remote/src/cloud/{image,unsynced}.rs` still use Teleport's `Host`,
  scripts, snapshots, or checkout helpers. `chat/existing.rs` and
  `chat/convert.rs` run `herdr_cli` scripts through that host; rich-chat
  preservation needs its separate Core-backed attach path, not this script
  runner kept under another name.

## Bounded removal boundary

First settle each shared follow producer and host-script caller under its own
T03/T04/T07 owner. Preserve any Core-backed terminal/chat view and exact
machine, pane and tab identity as those routes change. Then remove the native
Teleport menu and dialog, its mark read/write worker and sidebar badge, and
the window tick/state together. Finally delete the migration backend and its
host-script helpers once the remaining callers are gone; search aliases and
re-exports as well as `teleport` names. The first complete native unit is
**menu + dialog + mark worker + tick**, with follow navigation moved to its
actual surviving owner or removed with its excluded callers. A menu-only cut
is not complete.

`core_terminals/observation.rs::refresh_machines` and
`menu/devices/core_catalog.rs::render_core_machine_catalog` use a separate
Core machine feed; `terminal/links.rs` keeps explicit local file URL/path
classification. Neither depends on the mark worker. Keep those routes and
ordinary manual file actions while checking this cut. Negative checks should
prove no Teleport action, mark-file worker, migration tick, or `herdr_cli`
host-script path remains; retained checks should cover Core machine display,
exact target refusal, terminal/chat state, and deliberate local file opening.

No product tests ran for this source-only audit. T04 remains blocked on
G-LINK; A29 and A30 remain `not_run`. This note does not prove Core remote
migration, native behavior, or full RM-TELEPORT absence.
