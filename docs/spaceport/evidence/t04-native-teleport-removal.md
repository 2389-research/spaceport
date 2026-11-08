# T04/RM-TELEPORT: native migration UI and mark worker

At base `678197ed`, `menu/workspace.rs` offered Teleport, Teleport Back,
Go to copy, and Clear mark. `menu/teleport.rs` built its source and host
choices; `teleport/ui.rs` owned its dialog and migration worker. Each window
started `sheprd_remote::teleport::Marks::start()` and polled migration and
mark-file state. The sidebar rendered that client-local mark. This cut removes
the menu, dialog, migration worker, mark-file reader/writer, sidebar badge,
icons, and their unused English messages together. It does not read or change
a user's `teleported.json` or contact a host.

The workspace action tiles now use a two-column grid so Close stays in the
lower right when the two Teleport tiles disappear.
The crate README's former Teleport walkthrough now names the cut and links
this evidence; the icon catalog no longer lists the deleted artwork.

`fan_out/ui.rs`, `dispatch/hosts.rs`, and `pane_menu/carry.rs` still set a
completed-operation follow target. The exact endpoint, workspace, and optional
pane IDs now live in `window/target_follow.rs`; the window tick steers to those
IDs with the same bounded 20-second window and 750-millisecond cadence.
`pane_menu/tests/carry.rs` verifies that a focused workspace alone does not
complete a requested pane follow. This is still the existing legacy endpoint
follow route; it does not claim Core machine mapping or remote native parity.

The shared `sheprd_remote::teleport` host scripts, snapshot types, checkout
helpers, and GUI `teleport/text.rs` pane/step/warning labels remain because
pane carry, dispatch, fan-out, cloud image work, and rich-chat host scripts
still call them. Their owners must retire those callers before removing the
backend. No second transport or session owner was added. Core machine catalog
and ordinary file actions remain separate from this cut.

The new source absence regression failed before implementation on the live
native files, then passed 2/2 after removal. Default and all-feature
`cargo +1.99.0 check --locked -p sheprd-gpui` pass. The focused pane follow
headless test passes 1/1. Workspace tile tests pass 4/4, sidebar layout tests
pass 9/9, and retained Core machine catalog tests pass 3/3. Format, file
size, sorted-list and i18n checks pass; i18n reports 4,354 used messages in
245 files. The spec package's schema/link check and 21 tool tests pass;
they do not test application behavior. Cargo notes the existing third-party `block 0.1.6`
future compatibility warning; the focused GUI test link prints the existing
macOS compact-unwind warning.

This proves the native Teleport initiator and mark worker are absent from the
current source. It does not prove full RM-TELEPORT absence: fan-out, dispatch,
pane carry, and shared host scripts still run. Native GUI, other platforms,
release packages, and real host actions were not run. T04 stays blocked on
G-LINK, RM-TELEPORT stays `in_progress`, and A30/A33 stay `not_run` until
their complete scenarios and native retention checks pass.
