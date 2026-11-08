# Local device tags and the phone sync boundary

Source audit at `4e614ffb` for T10/RM-PHONE. No code or user state changed. T10 remains `not_started`; RM-PHONE remains `in_progress`, and A30/A54 are not satisfied.

## Current ownership

- `crates/sheprd-gpui/src/device_tags.rs` stores `device-tags.json` as a local copy keyed by `endpoint.id`. Its `ComputerTags` record and tag normalization come from `sheprd-remote-control/src/control/tags.rs`. Local edits save the copy, then schedule daemon exchange. `device_tags/sync.rs` reaches the local phone daemon or runs `sheprd-remote-control tags --offer` on a remote host. `device_tags/tests.rs` covers local file and early-edit behavior as well as daemon answer handling.
- `sidebar/tag_groups.rs` supplies reachable endpoint IDs and daemon targets to `DeviceTags::reach`; `window.rs` calls it on each tick. The same sidebar module groups and filters those endpoints by tag. `menu/devices/host_menu/tags.rs` edits tags for the host menu's endpoint ID; `control/server/hosts.rs` does likewise for the private desktop CLI. These are retained grouping and customization surfaces.
- `window/status_bar.rs` shows tag sync state only when `remote::Supervisor` reports paired phones. `app.rs` installs both the tag global and the phone supervisor. This status is phone-specific, unlike local grouping.

## Identity gap

`endpoint.rs` gives the legacy list `local`, `ssh:<profile-id>`, WSL, and cloud IDs. `menu/devices/core_catalog.rs` renders Core `MachineView` rows from `core_terminals::machine_catalog` as read-only facts; `sheprd-core-client/src/wire.rs` has a distinct, opaque `Machine.id`. No source-backed map from a legacy endpoint ID or target/name to that Core ID exists. Core machines are not yet the sidebar/host-menu rows that the tag picker edits. Matching by label or address could attach a saved tag to the wrong machine, and a local-only rewrite keyed by legacy IDs would leave the later Core view without stable tags. No Core tag RPC was observed or assumed.

## Next bounded cut

After T03 moves the machine rows and actions to Core identity, choose a desktop-local tag policy and bind grouping, menu edits, and the private CLI to `Machine.id` where Core supplies it. Decide explicitly how existing `device-tags.json` endpoint keys are retained or migrated without guessing identity. Then extract the needed pure tag rules and storage from the phone crate, remove `DeviceTags::reach`/`tick`/`sync.rs` and its per-window call, and remove the phone-only status. Preserve local grouping and safe concurrent saves. Tests should prove Core-ID grouping, local edit and restart persistence, old-key behavior, no daemon exchange, and retained local CLI targeting. This audit ran no product tests because it changed no product code.
