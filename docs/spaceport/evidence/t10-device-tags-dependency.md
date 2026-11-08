# Local device tags and the phone sync boundary

Source audit at `4e614ffb` for T10/RM-PHONE. No code or user state changed. T10 remains `not_started`; RM-PHONE remains `in_progress`, and A30/A54 are not satisfied.

## Current ownership

- `crates/sheprd-gpui/src/device_tags.rs` stores `device-tags.json` as a local copy keyed by `endpoint.id`. Its `ComputerTags` record and tag normalization come from `sheprd-remote-control/src/control/tags.rs`. Local edits save the copy, then schedule daemon exchange. `device_tags/sync.rs` reaches the local phone daemon or runs `sheprd-remote-control tags --offer` on a remote host. `device_tags/tests.rs` covers local file and early-edit behavior as well as daemon answer handling.
- `sidebar/tag_groups.rs` supplies reachable endpoint IDs and daemon targets to `DeviceTags::reach`; `window.rs` calls it on each tick. The same sidebar module groups and filters those endpoints by tag. `menu/devices/host_menu/tags.rs` edits tags for the host menu's endpoint ID; `control/server/hosts.rs` does likewise for the private desktop CLI. These are retained grouping and customization surfaces.
- `window/status_bar.rs` shows tag sync state only when `remote::Supervisor` reports paired phones. `app.rs` installs both the tag global and the phone supervisor. This status is phone-specific, unlike local grouping.

## Identity gap

`endpoint.rs` gives the legacy list `local`, `ssh:<profile-id>`, WSL, and cloud IDs. `menu/devices/core_catalog.rs` renders Core `MachineView` rows from `core_terminals::machine_catalog` as read-only facts; `sheprd-core-client/src/wire.rs` has a distinct, opaque `Machine.id`. No source-backed map from a legacy endpoint ID or target/name to that Core ID exists. Core machines are not yet the sidebar/host-menu rows that the tag picker edits. Matching by label or address could attach a saved tag to the wrong machine, and a local-only rewrite keyed by legacy IDs would leave the later Core view without stable tags. No Core tag RPC was observed or assumed.

## Proposed cut at `4e614ffb`

After T03 moves the machine rows and actions to Core identity, choose a desktop-local tag policy and bind grouping, menu edits, and the private CLI to `Machine.id` where Core supplies it. Decide explicitly how existing `device-tags.json` endpoint keys are retained or migrated without guessing identity. Then extract the needed pure tag rules and storage from the phone crate, remove `DeviceTags::reach`/`tick`/`sync.rs` and its per-window call, and remove the phone-only status. Preserve local grouping and safe concurrent saves. Tests should prove Core-ID grouping, local edit and restart persistence, old-key behavior, no daemon exchange, and retained local CLI targeting. This audit ran no product tests because it changed no product code.

## Current desktop boundary at `706d34aa`

The intervening T10 cuts already moved tag rules into
`sheprd-gpui/src/device_tags/rules.rs` and removed `device_tags/sync.rs`,
`DeviceTags::reach`, the phone supervisor, status badge, pairing UI,
`remote` CLI/control family and phone helper/package inputs. The five
`scripts/release/tests/test_phone_absence.py` checks passed on this tree.
There is no remaining phone-only CLI/control leaf to remove or make red.
`sheprd_remote` in `app.rs` now names the separate cloud-provider crate;
it is not a phone supervisor.

The retained `DeviceTags` global only reads/writes local `device-tags.json`.
`sidebar/tag_groups.rs` groups `self.endpoints` by `endpoint.id` and
`menu/devices/host_menu/tags.rs` edits `HostMenu.id`; the private
`control/server/hosts.rs::tag` resolves and edits the same endpoint ID.
In contrast, `menu/devices/core_catalog.rs` selects Core rows using
`MachineView.machine.id` from `sheprd-core-client/src/wire.rs`. No code maps
those opaque Core IDs to legacy endpoint IDs. Deleting the local tag UI now
would remove retained grouping; assigning old tags by display name or address
would risk putting them on the wrong Core machine. A Core tag RPC is neither
present nor needed to state this local presentation gap.

The next coherent cut follows T03's Core machine-row identity migration:
choose old-key behavior, bind local grouping/menu/CLI to exact Core IDs and
test persistence and exact targeting before retiring endpoint-keyed paths.
This read-only audit used no personal state or service and changed no app
code. T10 and RM-PHONE remain in progress; T04 remains blocked on G-LINK;
A31 and A30 remain `not_run`. The five source checks do not replace A31's
native process/package/network inventory and retained desktop IPC scenario.
