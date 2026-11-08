# Cloud provider and Teleport removal reachability

Read-only source audit at `784bd9c9`. T04 and the RM-CLOUD/RM-TELEPORT audits remain `in_progress`; T03 is still in progress, T07 blocked, G-MACHINE/G-PORT/G-FILE open, and A30 `not_run`. No provider API, credential, host script, personal environment or Core state was touched.

## Provider graph

- `crates/sheprd-remote/src/lib.rs` gates Azure, Boat, boxd, Coder, Container, Daytona, OpenSandbox, Railway and Sprites modules; `cloud/{providers,relay,unsynced}.rs` and `config/*.rs` provide their saved-device, command and settings paths. `crates/sheprd-client/src/discovery.rs` defines the provider enum and `ConnectTarget::Cloud`; `crates/sheprd-client/Cargo.toml`, `crates/sheprd-remote/Cargo.toml` and `crates/sheprd-gpui/Cargo.toml` propagate provider features. Thus even one provider's API module is only a leaf of a wider connector/config/UI/CLI cut.
- GPUI wires cloud jobs, batch create/destroy/exec, setup scripts, image builds and idle checks through `cloud.rs`, `cloud/{batch,image,jobs,worker}.rs`, `cloud_setup.rs`, `app.rs` and `window.rs`. `menu/devices/{picker,host_menu}.rs` merges legacy cloud devices/actions with the ordinary picker; `settings_window/cloud_devices.rs` and its provider cards expose setup/credential controls. `config.rs`, `config/cloud.rs` and provider config modules still parse cloud tables. `cloud.rs::install_idle_checks` starts periodic Azure and Container host checks at app startup; a feature flag alone is not a removal.
- CLI/control paths are `cli/control/{hosts,batch,exec}.rs`, `control/{client/hosts,server/cloud,server/hosts}.rs`, `crates/sheprd-control-protocol/src/protocol/{cloud,hosts,events}.rs`, `docs/cli.md` and `skills/herdr-gpui-browser/SKILL.md`. They expose cloud add/list/setup/create/exec/destroy/wait/images/batches and event reports. `sheprd-remote/src/credentials.rs` uses the generic `sheprd-credentials` store for providers, while Coder OAuth and Railway key setup have their own paths. The generic credential crate also serves other feature families; remove provider use without reading or deleting saved secrets. Provider settings/messages/tests span `crates/sheprd-gpui/locales/*/settings-cloud-*.ftl`, `settings_window/cloud_devices/*`, `crates/sheprd-remote/src/*/tests*` and CLI/control tests.
- The package graph contains `sheprd-remote` as a workspace member and GPUI dependency. `Justfile` exercises default and no-default feature combinations; `Cargo.lock` records their dependencies. The Azure OIDC signing steps in `.github/workflows/release.yml` sign Windows artifacts and are **not** the Azure VM provider; a provider cut must not delete them by keyword.

## Teleport and retained seams

- `crates/sheprd-remote/src/teleport.rs` owns the migration job, Git/worktree carry, process/agent session snapshot, host scripts, and persisted marks. `crates/sheprd-gpui/src/teleport.rs`, `menu/teleport.rs`, `teleport/{ui,text}.rs`, window tick/state, sidebar badges and `WorkOp::Teleport` (`crates/sheprd-control-protocol/src/protocol/work.rs`, `control/{client,server}/work.rs`, CLI help/docs) make it reachable. Its `teleport/host.rs::Host` is also used by pending excluded automation, fan-out, pane move, phone-link and cloud-image paths. Deleting `teleport` by name would break those callers before their cuts; keeping this `herdr` CLI script runner after removal would leave a second host authority.
- Retained Core machine facts flow through `core_terminals/observation.rs::refresh_machines` into `menu/devices/core_catalog.rs::render_core_machine_catalog`, beside the legacy cloud list in `menu/devices/picker.rs`. Preserve that bounded read-only display and ordinary manual Add Device form. T03 must replace legacy endpoint/action ownership with Core machine state before the shared picker can lose its cloud rows. G-MACHINE still blocks exact-identity Core actions; G-PORT and G-FILE still block complete service/transfer routes. The old `sheprd-client::ScriptHost`/`run_script`, `sheprd-gpui/src/usage::Host`, `host_build.rs`, generic credential primitives and picker UI must be classified by their remaining callers, not deleted with the provider crate.

## Next proof

`SPEC.md:29,33,65,69` and `SYSTEM.md:44,48,71` exclude provider provisioning and client migration while retaining Core-owned machine display/actions, transfer and service access. No complete bounded provider-specific cut was isolated here: removing a provider module alone leaves its enum/config/commands/UI reachable, and the full cut touches the legacy endpoint/action graph that T03 has not replaced. After that boundary is settled, remove provider and Teleport behavior, settings, credential uses, workers, protocol/CLI commands, messages, tests and unused dependencies in coherent units. Negative tests should reject old provider/Teleport config and commands and prove app startup makes no provider/host-script calls. Retained tests should cover Core catalog display, manual Add Device, exact-target refusal, and file/service behavior only through verified Core routes. This source audit ran no product tests and does not claim A30 or native parity.

## Teleport control route removal

At `93ebc401`, `sheprd teleport --to HOST` still parsed and the
`work` control operation could open the native Teleport dialog. A new
CLI test first failed because the command parsed successfully; a new
protocol test first failed because `WorkOp::Teleport` decoded. The
CLI family, help, client dispatch, wire variant, server handler and
dialog-opening flow are now removed together. The one host-offer method
used only by that flow is gone. CLI docs and the installed agent skill
no longer tell agents to use this command.

The tests now reject both the old CLI command and its JSON work
operation. They round-trip the caller's exact workspace ID on a
retained `worktree` operation and parse its explicit window target.
Nine retained work-control tests passed, including caller checkout
identity, muting, and the checkpoint dialog. The native Teleport menu
and backend still exist, so this proves only this control route is
absent. Core machine catalog, Tailscale and service forwarding were
not changed. The isolated T03 target with `CARGO_INCREMENTAL=0` ran
`cargo +1.99.0 test --locked -p sheprd-control-protocol --lib`
(58 passed), `cargo +1.99.0 test --locked -p sheprd-gpui --bin
sheprd removed_teleport` (1 passed), the nine-test
`control::server::work::tests` filter, and
`cargo +1.99.0 check --locked -p sheprd-gpui --bin sheprd`.
`just format-check`, `just check-file-size`, `just check-sorted`,
`git diff --check`, and the package `spec.py check` passed.
The GUI test link reported its existing macOS unwind-size warning and
Cargo reported the existing `block v0.1.6` future-incompatibility
notice; the production app check reported no new code warnings.
T04 remains `in_progress`; A30, A33 and A34 remain
`not_run`. This appended cut needs a new observation after integration.

## Cloud command and control route removal

The older audit and Teleport section above were already recorded in the evidence
ledger. This section records a later bounded cut; it does not replace those
observations. The `sheprd cloud` family, its `Request::Cloud` command, cloud
host operations, batch event kind, server dispatch, client response handling,
help, CLI docs, and managed-skill claims are removed together. Control protocol
version 18 refuses the old wire forms in the merged branch. The app-local batch registry, its
create/destroy confirmation pages, their error mapping, and the sole message
file were orphaned by that route and removed. The provider-owned `Shape` enum
remains in `sheprd-control-protocol::cloud` because the retained remote provider
client still uses it. No provider API, saved credential, backend crate, Core
machine catalog, Tailscale path, or service forward was changed.

The new protocol test failed first because the old cloud request decoded;
the new CLI test failed first because bare `cloud` parsed. They now reject
the cloud family and old cloud host/event wire forms. The protocol test also
round-trips a retained port-forward request with its exact daemon socket,
window, and workspace ID. The CLI test retains explicit-window host list and
port forward parsing. The existing host, Core catalog, event, cloud status,
and menu title tests cover retained adjacent behavior. The old `--` parser
already rejected unconsumed trailing words; its now-unused accessor was removed
without changing that rule.

With `CARGO_INCREMENTAL=0` and the isolated T03 target, the following passed:
`cargo +1.99.0 test --locked -p sheprd-control-protocol --lib` (52 tests),
`cargo +1.99.0 test --locked -p sheprd-gpui --bin sheprd removed_cloud`
(1 test), `control::server::hosts::tests` (35), `core_catalog` (5),
`control::server::events` (5), `cloud::status::tests` (2), and
`menu::page_title::tests` (2). Default and `--all-features` production
`cargo +1.99.0 check --locked -p sheprd-gpui --bin sheprd` passed with no
new code warnings. `just format-check`, `just check-file-size`,
`just check-sorted`, `just check-i18n`, `git diff --check`, and
`uv run --with jsonschema==4.26.0 python tools/spec.py check` passed; the
last command validates package integrity, not application behavior. The test
link reported its existing macOS unwind-size warning, and Cargo reported the
existing `block v0.1.6` future-compatibility notice. Native GUI and full
workspace CI were not run locally.

The first `just check-i18n` run found only messages left by the removed
batch dialogs. Their two catalog files and two page labels were removed;
the rerun passed with 5,296 valid, used messages.

Cloud provider settings, device picker rows, provider jobs and their backend
are still reachable. T04 stays `in_progress` and A30 stays `not_run`; this cut
does not prove that excluded cloud provisioning has left the runtime. The
earlier evidence was signed, so the new section needs its own observation
after integration.

## Merged cloud-route verification

At `ff8ff4ed`, the cloud route was integrated after the Teleport and terminal
editor control cuts. Control protocol version 18 retains all three refusals.
The merged protocol library passed 55 tests, including the removed-cloud wire
case. The default and all-feature app checks passed with only Cargo's existing
`block v0.1.6` future-compatibility notice. The merged i18n check passed with
5,226 valid messages in 273 files, and format, file-size, sorted-list, package,
and diff checks passed. The merged GUI removed-route filter passed 30 tests,
including cloud, Teleport and terminal editor refusal. Its existing macOS
linker unwind-size warning remains. Provider settings, jobs, picker rows and backend remain reachable;
A30 and native product verification remain open.
