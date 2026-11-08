# T04/RM-CLOUD: retired saved Container connector

This cut follows the [saved connector dependency](t04-container-connector-dependency.md). `sheprd-client::ConnectTarget` no longer has a Cloud target. The desktop endpoint catalog reads Core and manual SSH/WSL sources only; it does not read `cloud-devices.json` or offer a saved Container row. The provider rail, Settings card, host menu routes, cloud worker, Container engine connector, cloud feature wiring, exclusive locale/assets, and remote provider modules are removed. `sheprd-remote` still holds Teleport and shared host code for separately tracked T04 work.

The client does not rewrite `cloud-devices.json` or `sandbox-images.json`, and it does not start, stop, delete, or query a Container engine. The private child startup fixture in `endpoint/catalog/tests.rs` places a legacy row with account, machine, session, label, and an unknown field under private `HOME`/XDG paths, then asserts its exact bytes survive catalog load. The second headless test checks that Local and exact SSH targets remain selectable; its temp file is outside the app's state path and proves selection, not file preservation. Retired `[cloud]` and `[container]` config keys now load as ignored unknown keys, with no provider setting structs or migration. The old config and saved files remain on disk; no real user data or engine was touched.

Red first: `old_container_catalog_stays_foreign_to_startup_devices` failed while the old catalog still offered `container:w1`/`w2`. Green: GPUI default, all-feature, and no-default-feature checks before the final config-type deletion; focused catalog 12/12, device menu 47/47, retained workspace scripts 4/4. The final config unknown-key test passed 1/1 through Cargo, compiling the final default app; the updated private startup fixture and exact Local/SSH selection test passed 1/1 each from that linked test binary. The full release policy suite passed 227 tests with 8 skipped. The package spec check, file-size and sorted-region checks passed. One stale OpenSandbox policy test expected the now-retired shared cloud socket and was updated. A headless test initially expected a visible single-source rail on macOS; the corrected test checks the actual hidden rail and retained SSH form. Native desktop checks did not run because the desktop session was unavailable. The GPUI link emitted the existing macOS `__eh_frame` warning and Rust reported future incompatibility in `block 0.1.6`. All/no-default feature checks after the final config-type deletion are deferred to merged QA because shared disk fell below 8 GiB.

T04 stays blocked on G-LINK. A29/A30 remain not_run: this cut does not prove all excluded feature families absent. RM-CLOUD remains in_progress pending integration and merged reachability review. No Core repository, user environment, provider API, or release path was changed.

## Integrated branch check

Cherry-picked as `74ecb419` after the Core group title and T08 audit. The
merged source has no `CloudProvider`, `ConnectTarget::Cloud`, `CloudConfig`, or
`ContainerConfig` Rust references. The remaining `cloud` mentions in the
usage crate name distinct AI plan providers; Teleport's old comment and other
excluded families remain separate cuts. The old saved Container file is not a
startup endpoint source.

On the merged branch, `cargo +1.99.0 check -p sheprd-gpui --all-targets
--locked --offline` passed with default, `--all-features`, and
`--no-default-features` (shared target, `CARGO_INCREMENTAL=0`). The focused
endpoint catalog passed 12/12 and the same linked binary's device menu suite
passed 47/47. `just format-check`, `just check-file-size`, `just check-sorted`,
the cloud connector source guard (2/2), and package integrity (182 local
links) passed. The existing `block v0.1.6` future-compatibility notice and
macOS compact-unwind linker warning appeared; no native window was tested.
RM-CLOUD stays `in_progress` until the full excluded-feature reachability
review; A30 is still `not_run`.
