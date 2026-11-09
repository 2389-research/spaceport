# T08: Remove host-resource dashboards, retain Devices activity

Source base: integration `1bb9be4c3`. The first separate commit,
`df9b67e37`, fixes the pre-existing Devices tally that counted unknown agents
as idle. It carries unknown through totals and the existing legend. This
is a local regression fix, not completion of A18's live program, reachability
and process-state scenarios.

This cut follows SP-011 and D58. It removes the independent local sysinfo and
remote SSH/WSL/GPU resource sampler, status/sidebar gauges and tooltips,
Overview resource meters/dots, Devices metrics columns, metrics settings,
example keys, translations, feature tests, docs and dedicated screenshots.
The `system` CLI family and its private request/reply types are removed end
to end. Private control protocol 35 negotiates the changed schema; version
34 system requests/replies have no compatibility path.

The Devices page, two-hour in-memory chart/history, existing host and agent
rows, search, row expansion, split-group layout and exact endpoint/pane
navigation remain. The stop/wakeup helper moves unchanged to `worker_stop`
with its existing thread test, so retained listening-port and network-quality
workers keep their cancellation behavior. Host selection, shell access,
process inspection, sysinfo diagnostics and process byte formatting remain.
The shared CPU icon stays because rich-chat settings use it. No Cargo manifest
or lockfile changes are part of this cut.

## Validation

Environment: macOS/Darwin arm64, pinned Rust 1.99.0. Product tests use synthetic
headless GPUI fixtures, isolated temporary files and owned loopback sockets;
no personal daemon, transcript, account or remote host was used. An initial
`--offline` attempt lacked cached socket2 0.6.5; a `--locked` run fetched it.
Later checks use `--locked --offline`. A shared target exposed stale artifacts
from a concurrent schema cut, so final checks use a private target.

- Actual Cargo RED for `a_tally_keeps_unknown_agents_out_of_idle` observed
  idle `2` instead of `1`; GREEN Devices model run passed 7 tests.
- Source absence RED failed for the sampler, metrics symbols and Devices
  meters before the cut. Its regression now lives in the existing release
  test discovery at `scripts/release/tests/test_resource_dashboard_absence.py`.
- `cargo +1.99.0 test --locked --offline -p sheprd-gpui --bin spaceport --
  devices_overview:: listening_ports:: network_quality:: processes:: usage::
  control::client::hosts:: control::client::tests::tab_help
  control::server::hosts::
  cli::tests::removed_system worker_stop:: config::tests::status_bar
  config::tests::removed_resource_dashboard
  config::preferences::tests settings_window::pages::status_bar::tests
  settings_window::tests::pages_behaviour overview::tests
  sidebar::layout_tests`: **459 passed**, including narrow/split Devices,
  chart, host rows, translated text, expansion and agent-to-terminal behavior.
  A previous run found one test-migration duplicate preference assertion;
  it was corrected and the selected suite rerun successfully.
- Version-fence RED failed at protocol 34; `cargo +1.99.0 test --locked
  --offline -p sheprd-control-protocol` passed **65 tests** at protocol 35.
- `cargo +1.99.0 check --locked --offline -p sheprd-gpui --all-targets
  --all-features` passed. The selected runtime tests above use default features;
  all-feature runtime tests remain CI work.
- `cargo +1.99.0 fmt --all -- --check`, `just check-file-size`,
  `just check-sorted`, `just check-i18n` and `git diff --check` passed.
- `uv run python -m unittest discover -s scripts/release/tests -p
  'test_resource_dashboard_absence.py'` passed **4 tests**. Fresh-eyes review
  found a leftover unreachable `system` help arm; its added absence check
  failed before the arm was removed, then passed. The Azure source-absence
  test passed **2 tests** after removing deleted sampler paths from its list.
- From `docs/spaceport`, `uv run --with-requirements tools/requirements.txt
  python tools/test_spec.py` passed **21 tests**. `spec.py render` and
  `spec.py check` under the same pinned-requirements environment passed,
  including JSON Schema validation. These check specification integrity only.
- `cargo +1.99.0 clippy --locked --offline -p sheprd-gpui --all-targets
  --all-features -- -D warnings` failed at the client discovery
  `match_like_matches_macro` lint. Running the same command with `--no-deps`
  reached GPUI and failed with **34 production and 267 test diagnostics**,
  including existing source paths such as `endpoint/catalog.rs` and
  `window/tab_start/tests.rs`. This task did not reproduce the whole lint
  set on its base and does not claim a clean lint gate; the integrator captured
  a separate pre-cut baseline for comparison. No allowances were added.
- The final debug test link reported `ld: __eh_frame section too large
  (max 16MB) to encode dwarf unwind offsets in compact unwind table` and
  warned exception-handling performance could be affected. Cargo also reports
  future incompatibility in dependency `block 0.1.6`. Both notices remain
  visible; neither was suppressed. The all-feature type check produced no
  application source warnings.

## Remaining gates

**G-OBS remains open and T08 remains blocked.** No supported usage/rate-limit
or automation feed is established by deleting resource metrics. The excluded
provider polling implementation still needs its separate coherent removal;
its preserved host/shell consumers do not authorize polling.

No native macOS GUI, Windows/Linux native checks, desktop notification,
IME/popup checks, private live-daemon test, remote runtime scenario, release
build, performance budget or full workspace CI was run here. Headless GPUI
checks do not establish native parity. A18/A19/A20/A21/A24/A41 statuses stay
unchanged. Website/design blocks still contain unrelated historical upstream
marketing claims about excluded workflows; those remain T12 work. This cut
removes only dashboard screenshot references from those blocks.
