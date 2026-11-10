# Spaceport

<img src="assets/icons/sheprd-icon.png" alt="Spaceport rocket app icon" width="128" height="128">

Spaceport is an independent desktop fork of
[Sheprd](https://github.com/sheprd-labs/sheprd). It is under development. A signed,
notarized [macOS arm64 preview](https://github.com/harperreed/spaceport/releases/tag/v20261110.5)
is available for macOS 14.2 or later. The [Spaceport specification](docs/spaceport/START_HERE.md)
defines the product and ownership rules. The [task plan](docs/spaceport/plan/tasks.json),
[capability matrix](docs/spaceport/plan/capabilities.json), and
[acceptance record](docs/spaceport/plan/acceptance.json) show what has been
checked and what remains open. The [feature reachability snapshot](docs/spaceport/evidence/removal-reachability-54cb41c2.md)
maps retained and excluded source routes at a named revision.

The target app keeps native terminals, rich chats, workspaces, customization,
file interactions, service access, notifications, diagnostics, and a local
desktop CLI. It removes the old IDE, Git, GitHub, embedded browser, cloud
provisioning, Teleport, orchestration, mod, scheduler, and phone control
features. This branch still contains some of those paths; a build is not proof
that the fork meets the specification.

Spaceport permits **separately installed Herdr 1 and Herdr Core** runtimes.
Existing Herdr 1 startup may remain to preserve the desktop UX. Each session
keeps its owning runtime; Spaceport does not install, update or migrate either
runtime. Tailscale is external. Core routes use Core's machine connections and
Spaceport presentation documents; Herdr 1 may use its native workspace model.
Native rich chat needs a verified host-side adapter
route; remote parity, forwarding, file transfer, native platform behavior, and
production release identity still have [open gates](docs/spaceport/plan/gaps.json).

## Work on the fork

Use the pinned Rust toolchain in `rust-toolchain.toml`. From the repository
root, the current development executable is named `spaceport`:

```sh
cargo +1.99.0 check --locked -p sheprd-gpui --bin spaceport
```

Install the validator dependency as described in
[tools/README.md](docs/spaceport/tools/README.md), then run the specification
tools from their package root:

```sh
cd docs/spaceport
python tools/spec.py check
python tools/spec.py ready
python tools/spec.py context T00
```

The package validator checks the plan and its links. It does not run the
desktop app. See [engineering guidance](AGENTS.md), the [Core boundary
evidence](docs/spaceport/evidence/t02-core-boundary.md), [native platform
results](docs/spaceport/evidence/native-platform-results.md), and the [current
release identity snapshot](docs/spaceport/evidence/t12-release-identity-147491e2.md) before
claiming a feature or build is ready.

The [preview tag workflow](scripts/release/README.md#spaceport-macos-previews)
builds, signs, notarizes and publishes macOS previews from a qualified main
commit. See its [artifact verification](docs/spaceport/evidence/t12-macos-preview-20261110.5-signed.md).
General production releases, automatic updates and other-platform distribution
still have open gates; the inherited Sheprd publishing route remains disabled
for Spaceport.

## Origin and license

Spaceport is not affiliated with or endorsed by Sheprd or Herdr. The fork
retains the applicable Apache-2.0 notices in [LICENSE](LICENSE),
[NOTICE](NOTICE), and the [protocol attribution](crates/sheprd-protocol/NOTICE.md).
Git history preserves the upstream documentation and implementation for
reference.
