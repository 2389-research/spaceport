# T04/RM-CLOUD: Container new-provisioning cut

Based on `35e3569d` after Azure removal. This is an interim cut of the excluded
Container client, not Core machine parity. T04 remains blocked on G-LINK; A29
and A30 remain `not_run`.

## Removed route

- Settings Container New/Adopt controls and their worker, plus remote
  `container/setup.rs` creation/adoption/Herdr install path. Settings still
  reads and shows exact saved Container rows. The Container registry no longer
  registers a provisioner or image holder.
- Dispatch's new-sandbox tile, selection, worker, and post-provision setup
  handoff. Its host picker still targets the selected endpoint ID. No new
  machine is provisioned to dispatch a workspace.
- Settings and workspace sandbox-image build/reset entry points and the
  Container image support module. The generic cloud image library and its
  saved catalog remain for the later shared cloud cut; no product entry in
  this slice starts a build.
- Cloud creation jobs, progress rows and badge, and the now-unused Container
  machine creation helper. The footer's automatic visibility now depends on
  saved device count alone.
- Settings no longer offers image, CPU or memory inputs that only changed new
  container creation. Saving the still-live engine/auto-stop fields preserves
  old provisioning keys in the config file without using them to create work.

`control/client`, `control/server`, and `sheprd-control-protocol` have no
Container creation command or request. Startup only installs the existing
Container idle check; it has no creation job. The source contract test checks
these boundaries. The inherited `docs/sandbox.md` is labeled historical.

## Retained interim graph and limits

`cloud/providers.rs` still routes saved Container targets through
`container/connect.rs` and `container/machine.rs::ensure_running`. Settings
still offers stop/forget, the existing device menu still offers Remove, and
`cloud.rs::install_idle_checks` can stop an idle saved container. These are
excluded T04 runtime paths pending the full provider/Core machine cut. No
script in this change calls stop, destroy, Docker, Podman, or a user's Herdr.
The old `[container]` keys and saved `cloud-devices.json` rows remain parsed;
image, CPU and memory values are inert preserved data, not live creation
settings. This cut does not rewrite them. The generic cloud image backend
and explicit existing-device setup rerun also remain pending the full cloud
graph audit.

## Verification

The source contract failed on six live creation/dispatch/image routes before
editing, and the later unused Settings inputs were also red before removal.
The final contract passed 4 tests. `cargo test -p sheprd-remote --features
container --locked -q` passed 170 tests, with 4 opt-in live tests ignored.
`cargo test -p sheprd-gpui --all-features --locked dispatch::tests -- --quiet`
passed 20 tests; `menu::workspace_tests::dispatch` passed 6, including the
exact target endpoint; `settings_window::cloud_devices` passed 7, including
the saved Container row. `config::container::tests` passed 2, including
a save that preserves old image/CPU/memory keys. The first Settings run was
6/7: the revised setup
description lost the searchable agent names. Restoring those names made it
7/7 without reviving New or Adopt.

Default app `cargo check -p sheprd-gpui --locked`, final
`cargo check -p sheprd-gpui --all-targets --all-features --locked`,
`just check-i18n format-check check-file-size check-sorted`, and package
`spec.py check` passed. The GUI link emitted the inherited macOS compact-unwind
warning; the pinned `block` crate emitted Cargo's future-incompatibility
notice. Native GUI, Windows/Linux and real engine behavior were not exercised.
