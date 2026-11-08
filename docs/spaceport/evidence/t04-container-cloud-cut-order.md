# T04/RM-CLOUD: Container and shared cloud cut order

Read-only audit of `6bf15041`, using the T04 context packet. Azure has a
separate removal owner. No app, Core, saved catalog, container engine, provider
account or user service was touched. SP-006 and SP-015 exclude this provider
and its client-owned registry; T04 remains blocked on G-LINK and A30 is
`not_run`.

## Production routes still live

- `crates/sheprd-gpui/Cargo.toml` enables `azure` and `container` by default.
  `sheprd-client/src/discovery.rs` has `CloudProvider::Container` and
  `ConnectTarget::Cloud`; `sheprd-remote/src/cloud/providers.rs` generates
  Container connect, command, deletion, image and provision functions from
  one row. `sheprd-remote/src/container/{connect,engine,machine,lifecycle}.rs`
  owns the Docker/Podman connection and start/stop path. The GUI's
  `endpoint.rs` consults Container stop intent and resumes it on reconnect.
- `settings_window/cloud_devices/container.rs` renders New, saved-device and
  unsaved-device cards. New calls `setup::add_device` through
  `cloud/jobs.rs::start_cloud_job`; Adopt calls `setup::adopt`. Both can
  install Herdr and save a device (`sheprd-remote/src/container/setup.rs`).
  The same card can save `[container]`, stop or forget a device, and build,
  reset or view sandbox images. `settings_window/cloud_devices.rs` switches
  provider tabs; `menu/devices/{catalog,rail}.rs` exposes the provider from
  Add Device. The generic `cloud_setup` pending/tick path can run user setup
  or agent-install scripts after a new cloud device appears, and the host
  menu can rerun setup on an existing one.
- `dispatch/hosts.rs::dispatch_sandboxes` offers Container when a saved
  endpoint exists and `cloud::provision` is registered. `dispatch/job.rs`
  creates it through the provider. `menu/workspace/sandbox_images.rs`,
  `cloud/image/prebuild.rs` and Settings image controls can build a recipe;
  `sheprd-remote/src/cloud/image/{builds,catalog}.rs` stores the result.
  `app.rs` starts `cloud.rs::install_idle_checks` in normal mode; its
  Container timer calls `container/idle.rs::check_all` each minute, probes
  Herdr agent status and may stop a saved container. Removing creation alone
  leaves this executable worker and the connector in place.

## Smallest coherent interim cut

Remove both Settings New and Adopt actions, their install/setup handoffs,
Container's registered `provision` callback and dispatch sandbox offer,
and the Container image build/reset entry points as one new-provisioning
unit. Remove only helper code, assets, locale text and docs orphaned by those
routes. A `provision: None` registry edit alone leaves Settings create and
adopt live; deleting the Settings New card alone leaves dispatch and image
build live. Keep the saved-device reader and connection behavior only as an
explicit interim excluded graph, never as claimed Core parity. If existing
device management remains during this cut, its stop/forget actions and idle
timer must be named as pending deletion, not hidden behind a feature flag.

The subsequent full provider cut removes Container connect, lifecycle, idle
timer, settings, device picker rows, config, and the provider registry entry
after Core machine/view ownership replaces the legacy endpoint path. Core's
catalog is already separate (`core_terminals/observation.rs` and
`menu/devices/core_catalog.rs`); G-MACHINE still blocks exact-ID mutation
actions. T03's Core tab/view ownership is still open, so a bulk deletion of
shared view state cannot be inferred from this provider cut. G-FILE and G-PORT
remain separate gates for retained file/service routes. The new-provisioning
unit itself does not need a Core RPC or tab-store change.

When Azure and Container are both gone, remove the generic cloud feature and
Settings section coherently: `main.rs` rejects `cloud` with no provider, and
`settings_window/cloud_devices.rs::first_tab` indexes `CloudProvider::ALL[0]`.
Do not leave an empty provider section or dormant `cloud_setup`, image,
install, jobs or idle machinery. Preserve ordinary manual SSH, generic
credential primitives, Core machine onboarding, terminals, chats, workspaces
and deliberate local file opening; none needs Container provisioning.

## Saved state and checks for the code cut

`sheprd-remote/src/cloud/catalog.rs` stores all providers in
`cloud-devices.json`. Its foreign-entry arm keeps unknown provider rows on
later writes; `cloud/image/catalog.rs` does the same for images in
`sandbox-images.json`. Neither file is a license to delete a user's Docker
container, home volume or image. `container/machine.rs` has ownership labels,
and `container/setup.rs::destroy_saved` deletes a volume; removal code must
not call it. `[container]` and `[cloud.container]` are currently parsed by
`config/container.rs` and GPUI `config/cloud.rs`; if later made unknown,
document the load behavior and leave the original config untouched.

Red/green tests for the interim cut should reject Settings create/adopt,
dispatch Container selection and workspace/Settings image builds, and prove
no Container creation worker can be started from those entry points. The
separate startup idle worker remains pending. Retained checks
should cover Core Add Device's exact target and host-key rules, Core catalog,
ordinary file opening, and a saved Container row if the interim connector
still exists. For full removal, test that a retired `provider: "container"`
device retains its exact structured JSON value beside an active Azure row while
the shared catalog still writes. For the image catalog, preserve the old
Container record without deleting its file; if a shared writer survives,
test foreign-image round trips before using it. Current catalog tests cover
retired Boat/Daytona/Coder cases, not retired Container. Check
default and all-feature builds plus locales after each code cut. This audit
ran no Cargo, product test, native GUI or provider action.
