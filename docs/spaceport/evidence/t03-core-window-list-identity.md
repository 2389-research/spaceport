# T03 Core group identity in window listings

Source revision: `acd4f0efb` plus this bounded change. The control socket reports
the selected Core presentation group from one `CoreGroupKey` as hub Core ID,
optional machine ID, document name and group UUID. It sets the legacy
`workspace` field to null under Core selection. `current` is true only when
`core_group_available` finds the exact group in a fresh observation of the
matching hub; an offline or stale document keeps the identity visible but
marks it stale.
The control protocol advances to version 28. The CLI's human listing exposes
the same identity and stale marker.

## Red and green evidence

The headless regression first selected a Core group while a legacy snapshot
still held `w_legacy`. Before the change, `control_info` reported that legacy
workspace as selected. A separate CLI test deserialized a Core group response
and failed because the human listing omitted the Core identity. After the
change, the server app tests passed 10/10, including exact JSON identity and
fresh-to-stale transition. The focused CLI test passed 1/1 for current and
stale text. Control protocol tests passed 59/59. The retained Core workspace
UI tests passed 19, with three live tests ignored. GPUI all-target and
all-feature checks passed with Rust 1.99.0, `--locked --offline` and
`CARGO_INCREMENTAL=0`. The macOS compact-unwind linker warning and `block`
future-compatibility note were already present; this change added no warning.

## Constructor dependency remains

`SheprdWindow::new` still builds a legacy `endpoint::Catalog`, reads
`endpoints[0]` for preferences and inbox, then calls `reconnect`. A local
`ConnectTarget` may still start Herdr 1. The Core terminal polling path
currently joins Core only after a local legacy endpoint exists; its fallback
starts Core only when Herdr 1 is missing and can switch back. Thus neither an
initial Core-only window nor an exact-hub attach-only additional window exists
yet. `window new --window` still takes the selected legacy target, and
`control_hosts` still lists the legacy endpoint catalog. The Core group in
`window list` is a presentation group, not a claimed legacy workspace or a
Core connection made by this change.

The next constructor slice must replace the endpoint-index-zero readers for
retained window state and supply explicit Core launch identity. Its initial
window may start only a separately installed Core when absent; additional
windows must join the exact expected hub and refuse a mismatch, with no
Herdr 1 start. The tear-off dependency and fresh `TabId` checks remain in
`t03-core-tear-off-dependency.md`.

T03 remains `in_progress`; A24 remains `not_run`. No native window, real
Core, or personal daemon was exercised. This is a headless control-contract
regression, not evidence that the Core-only constructor or native app works.
